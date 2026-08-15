/// Go framework scanner — supports Fiber v2, Gin, Echo.
/// Strategy:
///   1. Collect Group() prefixes.
///   2. Collect route registrations → extract actual handler method name.
///   3. Build handler-body map (func Name → source text of body).
///   4. Build struct-field map (TypeName → Parameters with json tags).
///   5. Enrich each endpoint: query params, body fields, response codes.
use std::collections::HashMap;
use std::path::Path;
use regex::Regex;
use walkdir::WalkDir;

use crate::schema::contract::{
    ApiContract, Endpoint, HttpMethod, Parameter, ParamLocation, ResponseExample,
};

// Packages we ignore when looking for the handler inside route args.
const SKIP_PKGS: &[&str] = &[
    "middleware", "fiber", "http", "models", "c", "context", "fmt",
    "log", "os", "time", "uuid", "errors", "json", "strconv",
    "sync", "io", "math", "rand", "crypto", "validation", "response",
];

pub fn scan(path: &Path, title: &str, base_url: &str) -> anyhow::Result<ApiContract> {
    let group_re = Regex::new(
        r#"(\w+)\s*:?=\s*(\w+)\.Group\s*\(\s*"([^"]+)""#,
    )?;
    // Match "router.Method("path"" — we'll scan the arg list separately.
    let route_start_re = Regex::new(
        r#"(\w+)\.(Get|Post|Put|Patch|Delete|Head|Options)\s*\(\s*"([^"]+)""#,
    )?;
    // "var.Method" pattern used to find the real handler inside route args.
    let handler_in_args_re = Regex::new(r#"(\w+)\.(\w+)"#)?;
    let param_re = Regex::new(r#":(\w+)"#)?;

    let go_files: Vec<_> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().map_or(false, |x| x == "go")
                && !p.to_string_lossy().contains("/vendor/")
                && !p.to_string_lossy().ends_with("_test.go")
        })
        .collect();

    // ── Pass 1: Group prefix map + auth middleware tracking ──────────────────
    let auth_use_re = Regex::new(r#"(\w+)\.Use\s*\(\s*middleware\."#)?;

    let mut groups: HashMap<String, String> = HashMap::new();
    let mut group_parents: HashMap<String, String> = HashMap::new(); // child → parent
    let mut auth_groups: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &go_files {
        let src = std::fs::read_to_string(entry.path())?;
        // Record groups that have auth middleware applied via .Use(middleware.*)
        for cap in auth_use_re.captures_iter(&src) {
            auth_groups.insert(cap[1].to_string());
        }
        for cap in group_re.captures_iter(&src) {
            let var    = cap[1].to_string();
            let parent = cap[2].to_string();
            let suffix = normalize_path(&cap[3]);
            group_parents.insert(var.clone(), parent.clone());
            let prefix = match groups.get(&parent) {
                Some(p) => clean_path(&format!("{}/{}", p, suffix.trim_start_matches('/'))),
                None    => suffix,
            };
            groups.insert(var, prefix);
        }
    }

    // ── Pass 2: Route registrations ──────────────────────────────────────────
    // Collect (handler_var, handler_method, full_path, http_method, path_params, requires_auth).
    let mut raw: Vec<(String, String, String, HttpMethod, Vec<Parameter>, bool)> = Vec::new();

    for entry in &go_files {
        let src = std::fs::read_to_string(entry.path())?;
        for cap in route_start_re.captures_iter(&src) {
            let receiver     = cap[1].to_string();
            let method_str   = cap[2].to_string();
            let route_suffix = normalize_path(&cap[3]);

            // Scan args after the path string to find the real handler.
            let after_path = cap.get(0).unwrap().end();
            let close      = find_close_paren(&src, after_path);
            let args       = &src[after_path..close];

            let (handler_var, handler_method) = find_handler_var_method(args, &handler_in_args_re);
            if handler_method.is_empty() {
                continue; // inline func or no identifiable handler
            }
            if is_middleware(&handler_method) {
                continue;
            }

            let full_path = match groups.get(&receiver) {
                Some(prefix) => clean_path(&format!("{}/{}", prefix, route_suffix.trim_start_matches('/'))),
                None         => route_suffix,
            };

            let path_params: Vec<Parameter> = param_re
                .captures_iter(&full_path)
                .map(|c| Parameter {
                    name:       c[1].to_string(),
                    location:   ParamLocation::Path,
                    param_type: "string".to_string(),
                    required:   true,
                    description: None,
                    example:    None,
                })
                .collect();

            let requires_auth = group_is_protected(&receiver, &group_parents, &auth_groups);
            raw.push((handler_var, handler_method, full_path, parse_method(&method_str), path_params, requires_auth));
        }
    }

    // ── Pass 3: Build enrichment maps ────────────────────────────────────────
    let handler_map = build_handler_map(&go_files);
    let struct_map  = build_struct_map(&go_files);

    // ── Pass 4: Create endpoints with enrichment ─────────────────────────────
    let mut endpoints: Vec<Endpoint> = raw
        .into_iter()
        .map(|(handler_var, method_name, full_path, method, mut path_params, requires_auth)| {
            // Add auth header if the route's group chain has auth middleware.
            if requires_auth {
                path_params.push(Parameter {
                    name:        "Authorization".to_string(),
                    location:    ParamLocation::Header,
                    param_type:  "string".to_string(),
                    required:    true,
                    description: Some("Bearer <token>".to_string()),
                    example:     None,
                });
            }
            let mut ep = Endpoint {
                method,
                path: full_path,
                title: pascal_to_title(&method_name),
                description: None,
                parameters: path_params,
                responses: vec![],
                tags: None,
            };
            if let Some(body) = resolve_handler_body(&handler_map, &handler_var, &method_name) {
                enrich_endpoint(&mut ep, body, &struct_map);
            }
            ep
        })
        .collect();

    endpoints.sort_by(|a, b| {
        a.path.cmp(&b.path)
            .then_with(|| format!("{:?}", a.method).cmp(&format!("{:?}", b.method)))
    });

    anyhow::ensure!(!endpoints.is_empty(), "No routes found. Is this a Go/Fiber/Gin/Echo project?");

    Ok(ApiContract {
        title:       title.to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        base_url:    base_url.to_string(),
        endpoints,
    })
}

// ── Enrichment ────────────────────────────────────────────────────────────────

fn enrich_endpoint(ep: &mut Endpoint, body: &str, struct_map: &HashMap<String, Vec<Parameter>>) {
    // 1. Query parameters: c.Query / c.QueryInt / c.QueryFloat / c.QueryBool
    let query_re = Regex::new(r#"c\.Query(Int|Float|Bool)?\s*\(\s*"([^"]+)""#).unwrap();
    for cap in query_re.captures_iter(body) {
        let param_type = match cap.get(1).map(|m| m.as_str()) {
            Some("Int")   => "integer",
            Some("Float") => "number",
            Some("Bool")  => "boolean",
            _             => "string",
        };
        let name = cap[2].to_string();
        if !ep.parameters.iter().any(|p| p.name == name) {
            ep.parameters.push(Parameter {
                name,
                location:    ParamLocation::Query,
                param_type:  param_type.to_string(),
                required:    false,
                description: None,
                example:     None,
            });
        }
    }

    // 2. Body struct: resolve the type from var declaration then BodyParser call.
    //    Handles both:
    //      var input LoginInput + c.BodyParser(&input)
    //      c.BodyParser(&LoginInput{})
    let type_name = resolve_body_type(body);
    if let Some(type_name) = type_name {
        if let Some(fields) = struct_map.get(&type_name) {
            for field in fields {
                if !ep.parameters.iter().any(|p| p.name == field.name && matches!(p.location, ParamLocation::Body)) {
                    ep.parameters.push(field.clone());
                }
            }
        }
    }

    // 3. Responses: extract project-style helpers first (include message + body).
    //    Pattern: response.Helper(c, "literal message") or response.Helper(c, err.Error())
    //    The second arg may be a string literal (real message) or a dynamic expression
    //    (e.g. an error propagated from a DB/service call) — in that case we can't know
    //    the runtime text, so we say so explicitly instead of inventing one.
    let helper_re = Regex::new(
        r#"response\.(\w+)\s*\(\s*c,\s*(?:"([^"]+)"|([\w][\w.]*(?:\(\))?))"#,
    ).unwrap();
    let mut uses_response_pattern = false;
    for cap in helper_re.captures_iter(body) {
        let helper = &cap[1];
        let Some(code) = response_helper_to_status(helper) else { continue };
        uses_response_pattern = true;
        let message: serde_json::Value = match (cap.get(2), cap.get(3)) {
            (Some(literal), _) => serde_json::Value::String(literal.as_str().to_string()),
            (None, Some(expr)) => serde_json::Value::String(format!("<dinámico: {}>", expr.as_str())),
            (None, None)       => serde_json::Value::Null,
        };
        let resp_body = serde_json::json!({
            "status":  code_to_status_val(code),
            "message": message
        });
        ep.responses.push(ResponseExample {
            status:      code,
            description: Some(status_label(code).to_string()),
            body:        Some(resp_body),
        });
    }

    // Fall back to generic c.Status(N) for any status not yet covered.
    let status_num_re = Regex::new(r#"\.Status\((\d{3})\)"#).unwrap();
    let status_pkg_re = Regex::new(r#"(?:fiber|http)\.Status(\w+)"#).unwrap();
    let mut extra: std::collections::BTreeSet<u16> = std::collections::BTreeSet::new();
    for cap in status_num_re.captures_iter(body) {
        if let Ok(n) = cap[1].parse::<u16>() { extra.insert(n); }
    }
    for cap in status_pkg_re.captures_iter(body) {
        if let Some(n) = http_const_to_status(&cap[1]) { extra.insert(n); }
    }
    for code in extra {
        if !ep.responses.iter().any(|r| r.status == code) {
            // Only guess the {status, message} shape when this same endpoint already
            // proved the project uses that convention elsewhere — otherwise leave it
            // as an empty body rather than fabricate a shape we never observed.
            let body = if uses_response_pattern && code >= 400 {
                Some(serde_json::json!({
                    "status":  code_to_status_val(code),
                    "message": "<dinámico: mensaje real solo disponible en runtime>"
                }))
            } else {
                None
            };
            ep.responses.push(ResponseExample {
                status:      code,
                description: Some(status_label(code).to_string()),
                body,
            });
        }
    }

    // Sort responses by status code so same-status alternatives are adjacent.
    ep.responses.sort_by_key(|r| r.status);
}

fn code_to_status_val(code: u16) -> &'static str {
    if code >= 500 { "ERROR" } else if code < 300 { "OK" } else { "NOK" }
}

fn resolve_body_type(body: &str) -> Option<String> {
    // Form A: c.BodyParser(&TypeName{}) or c.BodyParser(&pkg.TypeName{})
    let lit_re = Regex::new(r#"c\.BodyParser\s*\(\s*&\s*(\w+(?:\.\w+)?)\s*\{"#).unwrap();
    if let Some(cap) = lit_re.captures(body) {
        return Some(strip_pkg_qualifier(&cap[1]));
    }
    // Form B: var input TypeName / var input pkg.TypeName + c.BodyParser(&input)
    let var_re    = Regex::new(r#"var\s+(\w+)\s+(\w+(?:\.\w+)?)"#).unwrap();
    let parser_re = Regex::new(r#"c\.BodyParser\s*\(\s*&\s*(\w+)\s*\)"#).unwrap();
    if let Some(pcap) = parser_re.captures(body) {
        let var_name = &pcap[1];
        for vcap in var_re.captures_iter(body) {
            if &vcap[1] == var_name {
                return Some(strip_pkg_qualifier(&vcap[2]));
            }
        }
    }
    None
}

/// "models.LoginInput" → "LoginInput" (struct_map keys are unqualified).
fn strip_pkg_qualifier(type_name: &str) -> String {
    type_name.rsplit('.').next().unwrap_or(type_name).to_string()
}

// ── Handler body map ──────────────────────────────────────────────────────────

// method_name → list of (receiver_type_name, body)
fn build_handler_map(go_files: &[walkdir::DirEntry]) -> HashMap<String, Vec<(String, String)>> {
    // Captures: (1) receiver type (optional), (2) function name
    let func_re = Regex::new(r"(?m)^func\s+(?:\(\w+\s+\*?(\w+)\)\s+)?(\w+)\s*\(").unwrap();
    let ctx_re  = Regex::new(r"\*fiber\.Ctx|\*gin\.Context|echo\.Context|http\.ResponseWriter").unwrap();
    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for entry in go_files {
        let src = match std::fs::read_to_string(entry.path()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for cap in func_re.captures_iter(&src) {
            let receiver_type = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let func_name     = cap[2].to_string();
            let after_paren   = cap.get(0).unwrap().end();
            if let Some(brace_pos) = find_function_body_start(&src, after_paren) {
                let params_text = &src[after_paren..brace_pos];
                if ctx_re.is_match(params_text) {
                    let body = extract_brace_block(&src, brace_pos);
                    map.entry(func_name).or_default().push((receiver_type, body));
                }
            }
        }
    }
    map
}

/// Choose the best matching handler body given the route variable name (e.g. "asesorHandler").
/// When multiple handlers share the same method name, pick the one whose receiver type
/// contains the stem of the variable name.
fn resolve_handler_body<'a>(
    map: &'a HashMap<String, Vec<(String, String)>>,
    handler_var: &str,
    method_name: &str,
) -> Option<&'a str> {
    let entries = map.get(method_name)?;
    if entries.len() == 1 {
        return Some(&entries[0].1);
    }
    // Derive stem: "asesorHandler" → "asesor", "aforeRojoHandler" → "aforeRojo"
    let stem = handler_var
        .trim_end_matches("Handler")
        .trim_end_matches("handler");
    let stem_lower = stem.to_lowercase();
    for (receiver_type, body) in entries {
        if receiver_type.to_lowercase().contains(&stem_lower) {
            return Some(body);
        }
    }
    entries.first().map(|e| e.1.as_str())
}

/// Walk past the parameter list (tracking paren depth) to find the opening `{`.
fn find_function_body_start(src: &str, after_open_paren: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut paren  = 1i32; // we start inside the '('
    let mut bracket = 0i32;
    let mut i = after_open_paren;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b'{' if paren <= 0 && bracket <= 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn extract_brace_block(src: &str, start: usize) -> String {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return src[start..=i].to_string();
                }
            }
            _ => {}
        }
        i += 1;
    }
    src[start..].to_string()
}

// ── Struct field map ──────────────────────────────────────────────────────────

fn build_struct_map(go_files: &[walkdir::DirEntry]) -> HashMap<String, Vec<Parameter>> {
    let struct_re = Regex::new(r"type\s+(\w+)\s+struct\s*\{").unwrap();
    // Field: Name  Type  `...json:"key"...`
    let field_re  = Regex::new(r#"(?m)^\s+(\w+)\s+([\w\[\]*\.]+)\s+`([^`]+)`"#).unwrap();
    let json_re   = Regex::new(r#"json:"([^"]+)""#).unwrap();
    let validate_re = Regex::new(r#"validate:"([^"]+)""#).unwrap();

    let mut map: HashMap<String, Vec<Parameter>> = HashMap::new();

    for entry in go_files {
        let src = match std::fs::read_to_string(entry.path()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for cap in struct_re.captures_iter(&src) {
            let struct_name = cap[1].to_string();
            // end() - 1 = position of '{'
            let brace_pos = cap.get(0).unwrap().end() - 1;
            let body = extract_brace_block(&src, brace_pos);

            let mut params: Vec<Parameter> = Vec::new();
            for fcap in field_re.captures_iter(&body) {
                let go_type  = &fcap[2];
                let full_tag = &fcap[3];

                let json_name = match json_re.captures(full_tag) {
                    Some(jc) => jc[1].to_string(),
                    None     => continue,
                };
                // Skip fields explicitly excluded from JSON
                let first_tag_part = json_name.split(',').next().unwrap_or("");
                if first_tag_part == "-" { continue; }
                let json_key = first_tag_part.to_string();

                let required = validate_re.captures(full_tag)
                    .map(|vc| vc[1].split(',').any(|p| p.trim() == "required"))
                    .unwrap_or(false);

                params.push(Parameter {
                    name:        json_key,
                    location:    ParamLocation::Body,
                    param_type:  go_type_to_json_type(go_type),
                    required,
                    description: None,
                    example:     None,
                });
            }
            if !params.is_empty() {
                map.insert(struct_name, params);
            }
        }
    }
    map
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn normalize_path(s: &str) -> String {
    let s = s.trim().trim_end_matches('/');
    if s.starts_with('/') { s.to_string() } else { format!("/{}", s) }
}

fn clean_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_slash = false;
    for c in s.chars() {
        if c == '/' {
            if !prev_slash { out.push(c); }
            prev_slash = true;
        } else {
            out.push(c);
            prev_slash = false;
        }
    }
    if !out.starts_with('/') { out.insert(0, '/'); }
    if out.len() > 1 { out = out.trim_end_matches('/').to_string(); }
    out
}

/// Scan route argument list for first "var.Method" that isn't a known package.
/// Returns (handler_var, handler_method).
fn find_handler_var_method(args: &str, re: &Regex) -> (String, String) {
    for cap in re.captures_iter(args) {
        let pkg = &cap[1];
        if !SKIP_PKGS.contains(&pkg) {
            return (cap[1].to_string(), cap[2].to_string());
        }
    }
    // Fallback: first bare PascalCase identifier (package-level handler func)
    let word_re = Regex::new(r#"\b([A-Z]\w+)\b"#).unwrap();
    if let Some(cap) = word_re.captures(args) {
        return (String::new(), cap[1].to_string());
    }
    (String::new(), String::new())
}

/// Walk to the matching `)` starting from inside the argument list.
fn find_close_paren(src: &str, start: usize) -> usize {
    let mut depth = 1i32;
    for (i, c) in src[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 { return start + i; }
            }
            _ => {}
        }
    }
    src.len()
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

fn is_middleware(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("middleware") || lower.contains("recover")
        || lower.contains("logger") || lower.contains("cors")
}

/// "GetUserById" → "Get User By Id"
fn pascal_to_title(name: &str) -> String {
    let mut out = String::new();
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if !out.is_empty() && c.is_uppercase() {
            if chars.peek().map_or(false, |n| n.is_lowercase())
                || out.ends_with(|p: char| p.is_lowercase())
            {
                out.push(' ');
            }
        }
        if out.is_empty() {
            out.push(c.to_uppercase().next().unwrap_or(c));
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_method(s: &str) -> HttpMethod {
    match s.to_lowercase().as_str() {
        "post"    => HttpMethod::Post,
        "put"     => HttpMethod::Put,
        "patch"   => HttpMethod::Patch,
        "delete"  => HttpMethod::Delete,
        "head"    => HttpMethod::Head,
        "options" => HttpMethod::Options,
        _         => HttpMethod::Get,
    }
}

fn go_type_to_json_type(go_type: &str) -> String {
    let base = go_type.trim_start_matches('*');
    if go_type.starts_with("[]") {
        return "array".to_string();
    }
    match base {
        "string" | "UUID" => "string",
        "int" | "int8" | "int16" | "int32" | "int64"
        | "uint" | "uint8" | "uint16" | "uint32" | "uint64" => "integer",
        "float32" | "float64" => "number",
        "bool"    => "boolean",
        t if t.contains("UUID") => "string",
        _ => "object",
    }.to_string()
}

fn http_const_to_status(name: &str) -> Option<u16> {
    match name {
        "StatusOK"                  => Some(200),
        "StatusCreated"             => Some(201),
        "StatusAccepted"            => Some(202),
        "StatusNoContent"           => Some(204),
        "StatusBadRequest"          => Some(400),
        "StatusUnauthorized"        => Some(401),
        "StatusForbidden"           => Some(403),
        "StatusNotFound"            => Some(404),
        "StatusConflict"            => Some(409),
        "StatusUnprocessableEntity" => Some(422),
        "StatusTooManyRequests"     => Some(429),
        "StatusInternalServerError" => Some(500),
        "StatusBadGateway"          => Some(502),
        "StatusServiceUnavailable"  => Some(503),
        _                           => None,
    }
}

fn response_helper_to_status(name: &str) -> Option<u16> {
    match name {
        "OK" | "NoContent"   => Some(200),
        "Created"            => Some(201),
        "BadRequest"         => Some(400),
        "Unauthorized"       => Some(401),
        "Forbidden"          => Some(403),
        "NotFound"           => Some(404),
        "Conflict"           => Some(409),
        "Unprocessable"      => Some(422),
        "TooManyRequests"    => Some(429),
        "InternalError"      => Some(500),
        _                    => None,
    }
}

/// Returns true if `var` or any of its group ancestors has auth middleware applied.
fn group_is_protected(
    var: &str,
    parents: &HashMap<String, String>,
    auth_groups: &std::collections::HashSet<String>,
) -> bool {
    if auth_groups.contains(var) { return true; }
    if let Some(parent) = parents.get(var) {
        return group_is_protected(parent, parents, auth_groups);
    }
    false
}

fn status_label(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Creado",
        202 => "Aceptado",
        204 => "Sin contenido",
        400 => "Solicitud inválida",
        401 => "No autenticado",
        403 => "Sin permiso",
        404 => "No encontrado",
        409 => "Conflicto",
        422 => "Error de validación",
        429 => "Demasiadas solicitudes",
        500 => "Error interno",
        _   => "",
    }
}

#[cfg(test)]
mod tests {
    use super::{enrich_endpoint, resolve_body_type};
    use crate::schema::contract::{Endpoint, HttpMethod};
    use std::collections::HashMap;

    #[test]
    fn enriches_responses_without_fabricating_dynamic_messages() {
        let body = r#"
            if err := c.BodyParser(&input); err != nil {
                return response.BadRequest(c, "invalid body")
            }
            user, err := findUser(input.Email)
            if err != nil {
                return response.InternalError(c, err.Error())
            }
            if user == nil {
                return c.Status(404).JSON(fiber.Map{"error": "not found"})
            }
            return response.OK(c, "login ok")
        "#;
        let mut ep = Endpoint {
            method: HttpMethod::Post,
            path: "/login".to_string(),
            title: "Login".to_string(),
            description: None,
            parameters: vec![],
            responses: vec![],
            tags: None,
        };
        enrich_endpoint(&mut ep, body, &HashMap::new());

        let by_status = |code: u16| ep.responses.iter().find(|r| r.status == code).unwrap();

        // Literal messages are captured verbatim — they are real, not invented.
        assert_eq!(by_status(200).body.as_ref().unwrap()["message"], "login ok");
        assert_eq!(by_status(400).body.as_ref().unwrap()["message"], "invalid body");

        // A message sourced from a runtime expression is labeled as dynamic,
        // citing the exact expression, instead of guessing its text.
        assert_eq!(
            by_status(500).body.as_ref().unwrap()["message"],
            "<dinámico: err.Error()>"
        );

        // A bare c.Status(404) has no helper call of its own, but the endpoint
        // already proved the project uses the {status, message} shape elsewhere,
        // so we reuse that real shape with a placeholder instead of leaving it null.
        assert_eq!(by_status(404).body.as_ref().unwrap()["status"], "NOK");
        assert!(by_status(404).body.as_ref().unwrap()["message"]
            .as_str()
            .unwrap()
            .starts_with("<dinámico:"));
    }

    #[test]
    fn resolves_body_type_across_package_qualifiers() {
        let unqualified = r#"
            var input LoginInput
            if err := c.BodyParser(&input); err != nil {}
        "#;
        assert_eq!(resolve_body_type(unqualified), Some("LoginInput".to_string()));

        let qualified = r#"
            var input models.LoginInput
            if err := c.BodyParser(&input); err != nil {}
        "#;
        assert_eq!(resolve_body_type(qualified), Some("LoginInput".to_string()));

        let literal_qualified = r#"
            if err := c.BodyParser(&models.LoginInput{}); err != nil {}
        "#;
        assert_eq!(resolve_body_type(literal_qualified), Some("LoginInput".to_string()));
    }
}
