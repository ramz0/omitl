use tera::Context;
use crate::config::BrandConfig;
use crate::schema::ApiContract;

/// Builds the Tera rendering context from a contract and brand config.
/// The template can access:
///   {{ brand.company_name }}, {{ brand.primary_color }}, ...
///   {{ contract.title }}, {{ contract.endpoints }}, ...
///   For each endpoint: {{ ep.has_parameters }} drives the "Ninguno" empty state.
pub fn build(contract: &ApiContract, brand: &BrandConfig) -> anyhow::Result<Context> {
    let mut ctx = Context::new();

    ctx.insert("brand", brand);
    ctx.insert("contract", contract);

    // Pre-compute the document date (brand override or today).
    let date = brand
        .document_date
        .clone()
        .unwrap_or_else(|| chrono_today());
    ctx.insert("document_date", &date);

    Ok(ctx)
}

fn chrono_today() -> String {
    // Using std only — no chrono dependency needed for a simple date stamp.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Minimal ISO date from epoch seconds (accurate post-2000).
    let days = now / 86400;
    epoch_days_to_iso(days)
}

fn epoch_days_to_iso(days: u64) -> String {
    // Tomohiko Sakamoto's algorithm adapted for day-of-year extraction.
    let mut y = 1970u64;
    let mut d = days;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year { break; }
        d -= days_in_year;
        y += 1;
    }
    let months = [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mut m = 0usize;
    for (i, &days_in_month) in months.iter().enumerate() {
        let dim = if i == 1 && leap { 29 } else { days_in_month };
        if d < dim { m = i; break; }
        d -= dim;
    }
    format!("{:04}-{:02}-{:02}", y, m + 1, d + 1)
}
