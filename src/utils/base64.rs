use base64::{engine::general_purpose::STANDARD, Engine};

/// Decode a Base64 string to raw bytes (for logo embedding).
pub fn decode(data: &str) -> anyhow::Result<Vec<u8>> {
    Ok(STANDARD.decode(data.trim())?)
}

/// Encode bytes to a Base64 string (utility for tooling / tests).
pub fn encode(data: &[u8]) -> String {
    STANDARD.encode(data)
}
