use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "omitl",
    version,
    about = "Generate corporate API contract documentation (PDF/DOCX)",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate documentation from an Omitl contract file or OpenAPI spec.
    Generate {
        /// Path to the contract JSON/YAML file.
        #[arg(short, long)]
        input: PathBuf,

        /// Path to the brand config JSON/YAML file.
        #[arg(short, long)]
        brand: Option<PathBuf>,

        /// Logo image file (PNG/SVG) to place in the page header.
        #[arg(long, value_name = "FILE")]
        logo: Option<PathBuf>,

        /// Image file to use as a full-page watermark background.
        #[arg(long, value_name = "FILE")]
        watermark: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value_t = Format::Pdf)]
        format: Format,

        /// Output file path (default: ./contract.<format>).
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Treat the input as an OpenAPI 3.x spec instead of a native contract.
        #[arg(long)]
        openapi: bool,

        /// Directory containing Tera templates (overrides built-in defaults).
        #[arg(long)]
        templates: Option<PathBuf>,
    },

    /// Validate a contract file without generating output.
    Validate {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(long)]
        openapi: bool,
    },

    /// Scan a project and auto-generate a contract from its source code.
    Scan {
        /// Path to the project root (must contain go.mod, package.json, etc.)
        #[arg(short, long)]
        path: PathBuf,

        /// API title (default: directory name).
        #[arg(long)]
        title: Option<String>,

        /// Base URL of the API (default: http://localhost:8080).
        #[arg(long)]
        base_url: Option<String>,

        /// Where to save the generated contract JSON.
        /// Default: contracts/<project-name>.json
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Also generate a PDF after scanning.
        #[arg(long)]
        generate: bool,

        /// Brand config to use when --generate is set.
        #[arg(short, long)]
        brand: Option<PathBuf>,

        /// Logo image file (PNG/SVG) for the page header.
        #[arg(long, value_name = "FILE")]
        logo: Option<PathBuf>,

        /// Image file to use as a full-page watermark background.
        #[arg(long, value_name = "FILE")]
        watermark: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Pdf,
    Docx,
}
