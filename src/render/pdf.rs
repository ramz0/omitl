use std::path::Path;
use tera::Tera;
use crate::config::BrandConfig;
use crate::schema::ApiContract;
use super::context::build;

/// Renders the contract to a PDF file at `output_path`.
///
/// Pipeline: ApiContract + BrandConfig → Tera → .typ source → Typst → PDF bytes → file.
pub fn render(
    contract: &ApiContract,
    brand: &BrandConfig,
    template_dir: &str,
    output_path: &Path,
) -> anyhow::Result<()> {
    let tera = Tera::new(&format!("{}/**/*.tera", template_dir))?;
    let ctx = build(contract, brand)?;
    let typ_source = tera.render("contract.typ.tera", &ctx)?;

    let pdf_bytes = compile_typst(&typ_source)?;
    std::fs::write(output_path, pdf_bytes)?;
    Ok(())
}

/// Compiles Typst source string to PDF bytes.
///
/// This is a thin wrapper — full Typst integration requires implementing
/// the `typst::World` trait. Replace the body with a real World impl
/// or shell out to `typst compile` during early development.
fn compile_typst(source: &str) -> anyhow::Result<Vec<u8>> {
    // Placeholder: write source to a temp file and invoke `typst compile`.
    // Replace with in-process World impl once the schema stabilises.
    let tmp_dir = std::env::temp_dir();
    let src_path = tmp_dir.join("omitl_contract.typ");
    let pdf_path = tmp_dir.join("omitl_contract.pdf");

    std::fs::write(&src_path, source)?;

    let status = std::process::Command::new("typst")
        .arg("compile")
        .arg(&src_path)
        .arg(&pdf_path)
        .status()?;

    anyhow::ensure!(status.success(), "typst compile failed");
    Ok(std::fs::read(&pdf_path)?)
}
