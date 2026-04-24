use serde::{Deserialize, Serialize};

/// Position of a logo image in the document header.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogoPosition {
    #[default]
    Left,
    Right,
    Watermark,
    None,
}

/// A logo encoded as Base64 so the config file is fully self-contained.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogoAsset {
    /// Base64-encoded image (PNG or SVG).
    pub data: String,
    /// MIME type: "image/png" | "image/svg+xml"
    pub mime: String,
    pub position: LogoPosition,
    /// Width in pt for PDF rendering.
    pub width_pt: Option<f64>,
}

/// Corporate brand configuration — kept separate from API content.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrandConfig {
    pub company_name: String,
    pub primary_color: String,   // hex, e.g. "#1A3C5E"
    pub secondary_color: String, // hex
    pub accent_color: String,    // hex, used for HTTP method badges
    pub font_family: String,     // e.g. "Inter" — must be available to Typst
    pub logo: Option<LogoAsset>,
    pub footer_text: Option<String>,
    /// ISO 8601 date string to stamp the document; defaults to today.
    pub document_date: Option<String>,
}

impl Default for BrandConfig {
    fn default() -> Self {
        Self {
            company_name: "Your Company".into(),
            primary_color: "#1A3C5E".into(),
            secondary_color: "#F5F5F5".into(),
            accent_color: "#E8631A".into(),
            font_family: "Liberation Sans".into(),
            logo: None,
            footer_text: None,
            document_date: None,
        }
    }
}
