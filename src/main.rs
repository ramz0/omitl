mod cli;
mod config;
mod render;
mod schema;
mod utils;

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

use cli::{Cli, Commands, Format};
use config::defaults::load_brand;
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
        Commands::Generate {
            input,
            brand,
            format,
            output,
            openapi: is_openapi,
            templates,
        } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;

            let brand_cfg = load_brand(brand.as_deref().and_then(|p| p.to_str()))?;

            let template_dir = templates
                .as_deref()
                .and_then(|p| p.to_str())
                .unwrap_or("templates");

            let output_path = output.unwrap_or_else(|| {
                let ext = match format {
                    Format::Pdf  => "pdf",
                    Format::Docx => "docx",
                };
                PathBuf::from(format!("contract.{}", ext))
            });

            match format {
                Format::Pdf => {
                    render::pdf::render(&contract, &brand_cfg, template_dir, &output_path)?;
                }
                Format::Docx => {
                    render::docx::render(&contract, &brand_cfg, &output_path)?;
                }
            }

            println!(
                "{} {}",
                "Generated:".green().bold(),
                output_path.display()
            );
        }

        Commands::Validate { input, openapi: is_openapi } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;
            println!("{} Contract is valid ({} endpoints)", "OK:".green().bold(), contract.endpoints.len());
        }
    }

    Ok(())
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
