use std::path::Path;
use docx_rs::*;
use crate::config::BrandConfig;
use crate::schema::contract::ApiContract;

/// Renders the contract to a DOCX file at `output_path`.
/// Uses docx-rs to build the document programmatically (no template engine needed).
pub fn render(
    contract: &ApiContract,
    brand: &BrandConfig,
    output_path: &Path,
) -> anyhow::Result<()> {
    let mut doc = Docx::new();

    // Title
    doc = doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(&contract.title).bold())
            .style("Heading1"),
    );

    // Subtitle / version
    doc = doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(format!("v{} — {}", contract.version, contract.base_url))),
    );

    if let Some(desc) = &contract.description {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(desc)));
    }

    // Footer branding note
    let footer_note = brand
        .footer_text
        .as_deref()
        .unwrap_or(&brand.company_name);

    // Endpoints
    for ep in &contract.endpoints {
        let method_label = format!("{:?}", ep.method).to_uppercase();
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("[{}] {} — {}", method_label, ep.path, ep.title)).bold())
                .style("Heading2"),
        );

        if let Some(d) = &ep.description {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(d)));
        }

        // Parameter table header
        doc = doc.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Parameters").bold()),
        );

        if ep.parameters.is_empty() {
            doc = doc.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text("Ninguno")),
            );
        } else {
            let mut table = Table::new(vec![]);
            // Header row
            table = table.add_row(
                TableRow::new(vec![
                    cell("Name"), cell("In"), cell("Type"), cell("Required"), cell("Description"),
                ])
            );
            for p in &ep.parameters {
                table = table.add_row(TableRow::new(vec![
                    cell(&p.name),
                    cell(&format!("{:?}", p.location).to_lowercase()),
                    cell(&p.param_type),
                    cell(if p.required { "Yes" } else { "No" }),
                    cell(p.description.as_deref().unwrap_or("-")),
                ]));
            }
            doc = doc.add_table(table);
        }
    }

    let _ = footer_note; // used in PDF footer, docx footer TBD
    let file = std::fs::File::create(output_path)?;
    doc.build().pack(file)?;
    Ok(())
}

fn cell(text: &str) -> TableCell {
    TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)))
}
