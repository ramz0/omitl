/// Minimal OpenAPI 3.x → ApiContract converter.
/// Supports the subset relevant for generating contracts:
/// paths, operationId, parameters, requestBody, responses.
use serde::Deserialize;
use std::collections::HashMap;
use crate::schema::contract::{
    ApiContract, Endpoint, HttpMethod, Parameter, ParamLocation, ResponseExample,
};

#[derive(Debug, Deserialize)]
struct OaInfo {
    title: String,
    version: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OaServer {
    url: String,
}

#[derive(Debug, Deserialize)]
struct OaParameter {
    name: String,
    #[serde(rename = "in")]
    location: String,
    required: Option<bool>,
    description: Option<String>,
    schema: Option<OaSchema>,
}

#[derive(Debug, Deserialize)]
struct OaSchema {
    #[serde(rename = "type")]
    schema_type: Option<String>,
    example: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OaResponse {
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OaOperation {
    #[serde(rename = "operationId")]
    operation_id: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    parameters: Option<Vec<OaParameter>>,
    responses: Option<HashMap<String, OaResponse>>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OaSpec {
    info: OaInfo,
    servers: Option<Vec<OaServer>>,
    paths: HashMap<String, HashMap<String, OaOperation>>,
}

pub fn from_openapi_str(input: &str) -> anyhow::Result<ApiContract> {
    let spec: OaSpec = serde_json::from_str(input)
        .or_else(|_| serde_yaml::from_str(input))?;

    let base_url = spec
        .servers
        .as_ref()
        .and_then(|s| s.first())
        .map(|s| s.url.clone())
        .unwrap_or_default();

    let mut endpoints = Vec::new();

    for (path, methods) in &spec.paths {
        for (method_str, op) in methods {
            let method = parse_method(method_str);
            let parameters = op.parameters.as_deref().unwrap_or(&[]).iter().map(|p| {
                Parameter {
                    name: p.name.clone(),
                    location: parse_location(&p.location),
                    param_type: p.schema.as_ref()
                        .and_then(|s| s.schema_type.clone())
                        .unwrap_or_else(|| "string".into()),
                    required: p.required.unwrap_or(false),
                    description: p.description.clone(),
                    example: p.schema.as_ref().and_then(|s| s.example.clone()),
                }
            }).collect();

            let responses = op.responses.as_ref().map(|r| {
                r.iter().filter_map(|(status, resp)| {
                    status.parse::<u16>().ok().map(|code| ResponseExample {
                        status: code,
                        description: resp.description.clone(),
                        body: None,
                    })
                }).collect()
            }).unwrap_or_default();

            endpoints.push(Endpoint {
                method,
                path: path.clone(),
                title: op.summary.clone()
                    .or_else(|| op.operation_id.clone())
                    .unwrap_or_else(|| format!("{} {}", method_str.to_uppercase(), path)),
                description: op.description.clone(),
                parameters,
                responses,
                tags: op.tags.clone(),
            });
        }
    }

    Ok(ApiContract {
        title: spec.info.title,
        version: spec.info.version,
        description: spec.info.description,
        base_url,
        endpoints,
    })
}

fn parse_method(s: &str) -> HttpMethod {
    match s.to_lowercase().as_str() {
        "get"     => HttpMethod::Get,
        "post"    => HttpMethod::Post,
        "put"     => HttpMethod::Put,
        "patch"   => HttpMethod::Patch,
        "delete"  => HttpMethod::Delete,
        "head"    => HttpMethod::Head,
        "options" => HttpMethod::Options,
        _         => HttpMethod::Get,
    }
}

fn parse_location(s: &str) -> ParamLocation {
    match s {
        "path"   => ParamLocation::Path,
        "query"  => ParamLocation::Query,
        "header" => ParamLocation::Header,
        _        => ParamLocation::Body,
    }
}
