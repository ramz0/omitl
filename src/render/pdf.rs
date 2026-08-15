use std::collections::HashMap;
use std::path::Path;
use tera::{Tera, Value};
use crate::config::{BrandConfig, LogoPosition};
use crate::schema::ApiContract;
use super::context::build;

const DEFAULT_TEMPLATE: &str = include_str!("../../templates/contract.typ.tera");

pub fn render(
    contract: &ApiContract,
    brand: &BrandConfig,
    template_dir: &str,
    output_path: &Path,
) -> anyhow::Result<()> {
    let template_src = custom_template(template_dir)
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());

    let mut tera = Tera::default();
    tera.add_raw_template("contract.typ.tera", &template_src)?;
    tera.register_filter("typst_json", |value: &Value, _: &HashMap<String, Value>| {
        let s = serde_json::to_string_pretty(value)
            .map_err(|e| tera::Error::msg(e.to_string()))?;
        let escaped: String = s.chars().flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '"'  => vec!['\\', '"'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec![],
            c    => vec![c],
        }).collect();
        Ok(Value::String(escaped))
    });

    let tmp_dir = std::env::temp_dir();

    // Decode logo and write to a temp file so Typst can reference it by path.
    let mut logo_path:      Option<String> = None;
    let mut watermark_path: Option<String> = None;

    if let Some(ref logo) = brand.logo {
        if !matches!(logo.position, LogoPosition::None) {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(logo.data.trim())
                .map_err(|e| anyhow::anyhow!("logo base64 decode failed: {}", e))?;
            let ext = if logo.mime.contains("svg") { "svg" } else { "png" };
            let tmp_img = tmp_dir.join(format!("omitl_logo.{}", ext));
            std::fs::write(&tmp_img, &bytes)?;
            let path_str = tmp_img.to_string_lossy().into_owned();
            match logo.position {
                LogoPosition::Watermark => watermark_path = Some(path_str),
                _                       => logo_path      = Some(path_str),
            }
        }
    }

    let mut ctx = build(contract, brand)?;
    ctx.insert("logo_path",      &logo_path);
    ctx.insert("watermark_path", &watermark_path);

    let typ_source = tera.render("contract.typ.tera", &ctx)?;
    let pdf_bytes  = compile_typst(&typ_source, &tmp_dir)?;
    std::fs::write(output_path, pdf_bytes)?;
    Ok(())
}

fn custom_template(template_dir: &str) -> Option<String> {
    let path = Path::new(template_dir).join("contract.typ.tera");
    std::fs::read_to_string(path).ok()
}

fn compile_typst(source: &str, tmp_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let src_path = tmp_dir.join("omitl_contract.typ");
    let pdf_path = tmp_dir.join("omitl_contract.pdf");

    std::fs::write(&src_path, source)?;

    let status = std::process::Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg("/")
        .arg(&src_path)
        .arg(&pdf_path)
        .status()
        .map_err(|_| anyhow::anyhow!("typst not found — install it: https://typst.app"))?;

    anyhow::ensure!(status.success(), "typst compile failed — check /tmp/omitl_contract.typ for errors");
    Ok(std::fs::read(&pdf_path)?)
}
