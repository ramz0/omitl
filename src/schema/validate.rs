use crate::schema::contract::ApiContract;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Contract has no endpoints")]
    NoEndpoints,
    #[error("Endpoint '{0}' has an empty path")]
    EmptyPath(String),
    #[error("base_url is empty")]
    EmptyBaseUrl,
}

pub fn validate(contract: &ApiContract) -> Result<(), ValidationError> {
    if contract.endpoints.is_empty() {
        return Err(ValidationError::NoEndpoints);
    }
    if contract.base_url.is_empty() {
        return Err(ValidationError::EmptyBaseUrl);
    }
    for ep in &contract.endpoints {
        if ep.path.is_empty() {
            return Err(ValidationError::EmptyPath(ep.title.clone()));
        }
    }
    Ok(())
}
