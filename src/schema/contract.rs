use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ParamLocation {
    Path,
    Query,
    Body,
    Header,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub location: ParamLocation,
    #[serde(rename(deserialize = "type"))]
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseExample {
    pub status: u16,
    pub description: Option<String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub responses: Vec<ResponseExample>,
    pub tags: Option<Vec<String>>,
}

impl Endpoint {
    /// Returns true when the endpoint has no parameters (drives the "Ninguno" empty state).
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiContract {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub base_url: String,
    pub endpoints: Vec<Endpoint>,
}
