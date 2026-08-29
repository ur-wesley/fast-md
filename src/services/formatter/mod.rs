mod json;
mod markdown;
mod toml;
mod yaml;

pub use json::{format_json, minify_json};
pub use markdown::format_markdown;
pub use toml::format_toml;
pub use yaml::format_yaml;

use crate::types::DocumentFormat;

/// Format document according to its format (Markdown, JSON, TOML, YAML, etc.).
pub fn format_document(source: &str, format: DocumentFormat) -> Result<String, String> {
    match format {
        DocumentFormat::Markdown | DocumentFormat::Mdx => Ok(format_markdown(source)),
        DocumentFormat::Json => format_json(source),
        DocumentFormat::Toml => format_toml(source),
        DocumentFormat::Yaml => format_yaml(source),
        _ => Ok(source.to_string()),
    }
}
