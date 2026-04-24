pub mod detect;
pub mod go;

use std::path::Path;
use crate::schema::contract::ApiContract;
use detect::Framework;

pub fn scan(
    path:     &Path,
    title:    Option<&str>,
    base_url: Option<&str>,
) -> anyhow::Result<(ApiContract, Framework)> {
    let framework = detect::detect(path);

    let title = title.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("API")
    });
    let base_url = base_url.unwrap_or("http://localhost:8080");

    let contract = match &framework {
        Framework::Fiber | Framework::Gin | Framework::Echo => {
            go::scan(path, title, base_url)?
        }
        Framework::Unknown => {
            // Try Go as a best-effort guess.
            go::scan(path, title, base_url)
                .map_err(|_| anyhow::anyhow!(
                    "Could not auto-detect framework. Supported: Fiber, Gin, Echo (Go) | Express, Fastify (Node.js) | FastAPI, Flask (Python)"
                ))?
        }
        f => anyhow::bail!(
            "{} scanner is not yet implemented. Use 'omitl generate --input spec.json --openapi' with an exported OpenAPI spec.",
            f
        ),
    };

    Ok((contract, framework))
}
