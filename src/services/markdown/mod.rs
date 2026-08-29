mod config;
mod frontmatter;
mod highlight;
mod mdx;

pub use config::{extract_config_toc, parse_config_document, validate_document};
pub use frontmatter::extract_frontmatter;
pub use highlight::parse_markdown_document;
pub use mdx::preprocess_mdx;

use crate::types::{DocumentFormat, ParsedDocument};

/// Parse document according to its format (Markdown or Config format).
#[must_use]
pub fn parse_document(raw: &str, format: DocumentFormat) -> ParsedDocument {
    if format.is_markdown() {
        let mut doc = parse_markdown_document(raw);
        doc.format = format;
        doc
    } else {
        parse_config_document(raw, format)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::types::DocumentFormat;

    #[test]
    fn test_parse_json_document_and_toc() {
        let json_raw = r#"{
            "name": "fast-md",
            "version": "0.1.2",
            "dependencies": {
                "dioxus": "0.6"
            }
        }"#;
        let doc = parse_document(json_raw, DocumentFormat::Json);
        assert_eq!(doc.format, DocumentFormat::Json);
        assert!(doc.validation_error.is_none());
        assert!(!doc.toc.is_empty());
        assert!(doc.toc.iter().any(|t| t.title.contains("name")));
        assert!(doc.html_content.contains("config-doc-container"));
        assert!(doc.html_content.contains("JSON"));
    }

    #[test]
    fn test_parse_toml_document_and_toc() {
        let toml_raw = "[package]\nname = \"fast-md\"\nversion = \"0.1.2\"\n\n[dependencies]\ndioxus = \"0.6\"\n";
        let doc = parse_document(toml_raw, DocumentFormat::Toml);
        assert_eq!(doc.format, DocumentFormat::Toml);
        assert!(doc.validation_error.is_none());
        assert_eq!(doc.toc.len(), 2);
        assert_eq!(doc.toc[0].title, "[package]");
        assert_eq!(doc.toc[1].title, "[dependencies]");
        assert!(doc.html_content.contains("TOML"));
    }

    #[test]
    fn test_parse_yaml_document_and_validation_error() {
        let invalid_yaml = "key: [unclosed array\nanother: 123";
        let doc = parse_document(invalid_yaml, DocumentFormat::Yaml);
        assert!(doc.validation_error.is_some());
        assert!(doc.html_content.contains("config-syntax-error-banner"));
    }
}
