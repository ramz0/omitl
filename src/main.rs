mod cli;
mod config;
mod render;
mod scanner;
mod schema;
mod utils;

use clap::Parser;
use colored::Colorize;
use std::path::{Path, PathBuf};

use cli::{Cli, Commands, Format};
use config::{defaults::load_brand, BrandConfig, LogoAsset, LogoPosition};
use schema::{openapi, validate, ApiContract};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {:#}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { input, brand, logo, watermark, format, output, openapi: is_openapi, templates } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;
            let mut brand_cfg = load_brand(brand.as_deref().and_then(|p| p.to_str()))?;
            apply_image_flags(&mut brand_cfg, logo.as_deref(), watermark.as_deref())?;
            let template_dir = templates.as_deref().and_then(|p| p.to_str()).unwrap_or("templates");
            let output_path = output.unwrap_or_else(|| {
                PathBuf::from(match format { Format::Pdf => "contract.pdf", Format::Docx => "contract.docx" })
            });
            match format {
                Format::Pdf  => render::pdf::render(&contract, &brand_cfg, template_dir, &output_path)?,
                Format::Docx => render::docx::render(&contract, &brand_cfg, &output_path)?,
            }
            println!("{} {}", "Generated:".green().bold(), output_path.display());
        }

        Commands::Validate { input, openapi: is_openapi } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;
            println!("{} Contract is valid ({} endpoints)", "OK:".green().bold(), contract.endpoints.len());
        }

        Commands::Scan { path, title, base_url, output, generate, brand, logo, watermark } => {
            println!("{} {}", "Scanning:".cyan().bold(), path.display());

            let (contract, framework) = scanner::scan(
                &path,
                title.as_deref(),
                base_url.as_deref(),
            )?;

            println!("{} {} — {} endpoints found",
                "Detected:".cyan().bold(),
                framework,
                contract.endpoints.len()
            );

            let contract_path = output.unwrap_or_else(|| {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("api");
                std::fs::create_dir_all("contracts").ok();
                PathBuf::from(format!("contracts/{}.json", name))
            });

            let json = serde_json::to_string_pretty(&contract)?;
            std::fs::write(&contract_path, &json)?;
            println!("{} {}", "Contract:".green().bold(), contract_path.display());

            if generate {
                let mut brand_cfg = load_brand(brand.as_deref().and_then(|p| p.to_str()))?;
                apply_image_flags(&mut brand_cfg, logo.as_deref(), watermark.as_deref())?;
                let name = contract_path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("api");
                let out = PathBuf::from(format!("output/{}/contract.pdf", name));
                std::fs::create_dir_all(out.parent().unwrap())?;
                render::pdf::render(&contract, &brand_cfg, "templates", &out)?;
                println!("{} {}", "Generated:".green().bold(), out.display());
            }
        }
    }

    Ok(())
}

/// Load an image file, base64-encode it, and inject it into the brand config.
/// `--logo` sets position Left; `--watermark` sets position Watermark.
fn apply_image_flags(
    brand: &mut BrandConfig,
    logo: Option<&Path>,
    watermark: Option<&Path>,
) -> anyhow::Result<()> {
    if let Some(path) = watermark.or(logo) {
        let position = if watermark.is_some() { LogoPosition::Watermark } else { LogoPosition::Left };
        let bytes = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("cannot read image '{}': {}", path.display(), e))?;
        let mime = mime_from_path(path);
        use base64::Engine as _;
        let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
        brand.logo = Some(LogoAsset { data, mime, position, width_pt: None });
    }
    Ok(())
}

fn mime_from_path(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("svg") => "image/svg+xml".into(),
        _           => "image/png".into(),
    }
}

fn load_contract(path: &PathBuf, is_openapi: bool) -> anyhow::Result<ApiContract> {
    let content = std::fs::read_to_string(path)?;
    if is_openapi {
        openapi::from_openapi_str(&content)
    } else {
        serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(Into::into)
    }
}
