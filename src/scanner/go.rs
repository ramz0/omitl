/// Go framework scanner — supports Fiber v2, Gin, Echo.
///
/// Strategy: regex-based AST-lite scan.
/// 1. Find Group() calls to build a prefix map (var → full path).
/// 2. Find route registrations and resolve their full path via the prefix map.
/// 3. Extract path parameters (:id, *param) as required parameters.
/// 4. Convert handler names from PascalCase to human titles.
use std::collections::HashMap;
use std::path::Path;
use regex::Regex;
use walkdir::WalkDir;

use crate::schema::contract::{
    ApiContract, Endpoint, HttpMethod, Parameter, ParamLocation,
};

pub fn scan(path: &Path, title: &str, base_url: &str) -> anyhow::Result<ApiContract> {
    let group_re = Regex::new(
        r#"(\w+)\s*:?=\s*(\w+)\.Group\s*\(\s*"([^"]+)""#,
    )?;
    let route_re = Regex::new(
        r#"(\w+)\.(Get|Post|Put|Patch|Delete|Head|Options)\s*\(\s*"([^"]+)"\s*,\s*(\w+)"#,
    )?;
    let param_re = Regex::new(r#"[:/]\*?:?(\w+)"#)?;

    // var_name → resolved prefix  (populated across all files)
    let mut groups: HashMap<String, String> = HashMap::new();
    let mut endpoints: Vec<Endpoint> = Vec::new();

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

    // First pass: collect all Group() definitions so nested groups resolve correctly.
    for entry in &go_files {
        let src = std::fs::read_to_string(entry.path())?;
        for cap in group_re.captures_iter(&src) {
            let var    = cap[1].to_string();
            let parent = cap[2].to_string();
            let suffix = normalize_path(&cap[3]);

            let prefix = match groups.get(&parent) {
                Some(p) => format!("{}{}", p, suffix),
                None    => suffix,
            };
            groups.insert(var, prefix);
        }
    }

    // Second pass: collect routes.
    for entry in &go_files {
        let src = std::fs::read_to_string(entry.path())?;
        for cap in route_re.captures_iter(&src) {
            let receiver     = cap[1].to_string();
            let method_str   = cap[2].to_string();
            let route_suffix = normalize_path(&cap[3]);
            let handler_name = cap[4].to_string();

            // Skip obvious middleware registrations.
            if is_middleware(&handler_name) {
                continue;
            }

            let full_path = match groups.get(&receiver) {
                Some(prefix) => format!("{}{}", prefix, route_suffix),
                None         => route_suffix,
            };

            // Extract :param and *param path parameters.
            let path_params: Vec<Parameter> = param_re
                .captures_iter(&full_path)
                .map(|c| Parameter {
                    name:        c[1].to_string(),
                    location:    ParamLocation::Path,
                    param_type:  "string".to_string(),
                    required:    true,
                    description: None,
                    example:     None,
                })
                .collect();

            endpoints.push(Endpoint {
                method:      parse_method(&method_str),
                path:        full_path,
                title:       pascal_to_title(&handler_name),
                description: None,
                parameters:  path_params,
                responses:   vec![],
                tags:        None,
            });
        }
    }

    // Sort by path then method for a predictable output order.
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

// ── Helpers ────────────────────────────────────────────────────────────────

fn normalize_path(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('/') { s.to_string() } else { format!("/{}", s) }
}

fn is_middleware(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("middleware")
        || lower.contains("recover")
        || lower.contains("logger")
        || lower.contains("cors")
        || lower.contains("auth")   // auth middleware — heuristic, not auth handlers
            && lower.len() < 8      // e.g. "auth" alone is middleware, "AuthUser" is handler
}

/// "GetUserById" → "Get User By Id"
/// "createPayment" → "Create Payment"
fn pascal_to_title(name: &str) -> String {
    let mut out = String::new();
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if !out.is_empty() && c.is_uppercase() {
            if chars.peek().map_or(false, |n| n.is_lowercase()) || out.ends_with(|p: char| p.is_lowercase()) {
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
