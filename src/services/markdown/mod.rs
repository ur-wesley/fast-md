mod config;
mod frontmatter;
mod highlight;
mod mdx;

pub use config::{extract_config_toc, parse_config_document, parse_plain_text, validate_document};
pub use frontmatter::extract_frontmatter;
pub use highlight::{highlighted_config_lines_for_string, parse_markdown_document};
pub use mdx::preprocess_mdx;

use crate::types::{DocumentFormat, ParsedDocument};

/// Parse document according to its format (Markdown or Config format).
#[must_use]
pub fn parse_document(raw: &str, format: DocumentFormat) -> ParsedDocument {
    if format.is_markdown() {
        let mut doc = parse_markdown_document(raw);
        doc.format = format;
        doc
    } else if format == DocumentFormat::PlainText {
        parse_plain_text(raw)
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
        let name_item = doc.toc.iter().find(|t| t.title.contains("name")).expect("name toc item");
        assert_eq!(name_item.line, Some(1));
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
        assert_eq!(doc.toc[0].line, Some(0));
        assert_eq!(doc.toc[1].title, "[dependencies]");
        assert_eq!(doc.toc[1].line, Some(4));
        assert!(doc.html_content.contains("TOML"));
    }

    #[test]
    fn test_parse_yaml_document_and_toc_line() {
        let yaml_raw = "title: fast-md\nversion: 0.1.2\n";
        let doc = parse_document(yaml_raw, DocumentFormat::Yaml);
        assert!(doc.validation_error.is_none());
        assert_eq!(doc.toc.len(), 2);
        assert_eq!(doc.toc[0].title, "title");
        assert_eq!(doc.toc[0].line, Some(0));
        assert_eq!(doc.toc[1].title, "version");
        assert_eq!(doc.toc[1].line, Some(1));
    }

    #[test]
    fn test_parse_yaml_document_and_validation_error() {
        let invalid_yaml = "key: [unclosed array\nanother: 123";
        let doc = parse_document(invalid_yaml, DocumentFormat::Yaml);
        assert!(doc.validation_error.is_some());
        assert!(doc.html_content.contains("config-syntax-error-banner"));
    }

    #[test]
    fn test_config_rainbow_brackets_nested_json() {
        let json_raw = r#"{"a":[1]}"#;
        let doc = parse_document(json_raw, DocumentFormat::Json);
        assert!(doc.html_content.contains("class=\"rb rb-0\">{</span>"));
        assert!(doc.html_content.contains("class=\"rb rb-0\">}</span>"));
        assert!(doc.html_content.contains("class=\"rb rb-1\">[</span>"));
        assert!(doc.html_content.contains("class=\"rb rb-1\">]</span>"));
    }

    #[test]
    fn test_config_rainbow_brackets_skip_inside_strings() {
        let json_raw = r#""hello { world }""#;
        let doc = parse_document(json_raw, DocumentFormat::Json);
        assert!(!doc.html_content.contains("class=\"rb"));
    }

    #[test]
    fn test_parse_plain_text_document() {
        let raw = "# Hello\n**bold**\n<script>alert(1)</script>";
        let doc = parse_document(raw, DocumentFormat::PlainText);
        assert_eq!(doc.format, DocumentFormat::PlainText);
        assert!(doc.validation_error.is_none());
        assert!(doc.toc.is_empty());
        assert!(doc.html_content.contains("plain-text-doc"));
        assert!(!doc.html_content.contains("config-doc-container"));
        assert!(!doc.html_content.contains("<h1>"));
        assert!(!doc.html_content.contains("<strong>"));
        assert!(!doc.html_content.contains("<script>"));
        assert!(doc.html_content.contains("&lt;script&gt;"));
        assert!(doc.html_content.contains("# Hello"));
        assert!(doc.html_content.contains("**bold**"));
        assert_eq!(doc.preview_lines.len(), 3);
    }
}
