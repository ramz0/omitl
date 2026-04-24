mod cli;
mod config;
mod render;
mod scanner;
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
        Commands::Generate { input, brand, format, output, openapi: is_openapi, templates } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;
            generate_output(contract, brand, format, output, templates)?;
        }

        Commands::Validate { input, openapi: is_openapi } => {
            let contract = load_contract(&input, is_openapi)?;
            validate::validate(&contract)?;
            println!("{} Contract is valid ({} endpoints)", "OK:".green().bold(), contract.endpoints.len());
        }

        Commands::Scan { path, title, base_url, output, generate, brand } => {
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

            // Determine where to save the contract JSON.
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

            // Optionally generate the PDF right away.
            if generate {
                let brand_cfg = load_brand(brand.as_deref().and_then(|p| p.to_str()))?;
                let pdf_path = contract_path.with_extension("pdf");
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

fn generate_output(
    contract: ApiContract,
    brand: Option<PathBuf>,
    format: Format,
    output: Option<PathBuf>,
    templates: Option<PathBuf>,
) -> anyhow::Result<()> {
    let brand_cfg = load_brand(brand.as_deref().and_then(|p| p.to_str()))?;
    let template_dir = templates.as_deref().and_then(|p| p.to_str()).unwrap_or("templates");
    let output_path = output.unwrap_or_else(|| {
        PathBuf::from(match format { Format::Pdf => "contract.pdf", Format::Docx => "contract.docx" })
    });
    match format {
        Format::Pdf  => render::pdf::render(&contract, &brand_cfg, template_dir, &output_path)?,
        Format::Docx => render::docx::render(&contract, &brand_cfg, &output_path)?,
    }
    println!("{} {}", "Generated:".green().bold(), output_path.display());
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
