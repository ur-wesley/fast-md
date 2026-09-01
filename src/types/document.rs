use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Frontmatter metadata extracted from the start of a Markdown/MDX file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub extra: BTreeMap<String, String>,
}

/// Table of Contents entry representing a heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocItem {
    pub id: String,
    pub title: String,
    pub level: u8,
    pub line: Option<usize>,
}

/// Supported document and config file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentFormat {
    #[default]
    Markdown,
    Mdx,
    Json,
    Toml,
    Yaml,
    Ini,
    Ron,
    Xml,
    PlainText,
}

impl DocumentFormat {
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "md" | "markdown" | "mdown" => Self::Markdown,
            "mdx" => Self::Mdx,
            "json" | "jsonc" | "json5" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "ini" | "cfg" | "conf" => Self::Ini,
            "ron" => Self::Ron,
            "xml" | "svg" => Self::Xml,
            _ => Self::PlainText,
        }
    }

    #[must_use]
    pub fn from_path(path: Option<&std::path::Path>) -> Self {
        path.and_then(|p| p.extension())
            .and_then(|ext| ext.to_str())
            .map_or(Self::Markdown, Self::from_extension)
    }

    #[must_use]
    pub const fn is_config(self) -> bool {
        matches!(self, Self::Json | Self::Toml | Self::Yaml | Self::Ini | Self::Ron | Self::Xml)
    }

    #[must_use]
    pub const fn is_markdown(self) -> bool {
        matches!(self, Self::Markdown | Self::Mdx)
    }

    #[must_use]
    pub const fn syntax_token(self) -> &'static str {
        match self {
            Self::Markdown | Self::Mdx => "markdown",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Ini => "ini",
            Self::Ron => "rust",
            Self::Xml => "xml",
            Self::PlainText => "text",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Mdx => "MDX",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
            Self::Ini => "INI",
            Self::Ron => "RON",
            Self::Xml => "XML",
            Self::PlainText => "Plain Text",
        }
    }
}

/// Fully parsed document or config file ready for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedDocument {
    pub html_content: String,
    pub preview_lines: Vec<String>,
    pub toc: Vec<TocItem>,
    pub metadata: Option<DocMetadata>,
    pub word_count: usize,
    pub reading_time_minutes: usize,
    pub format: DocumentFormat,
    pub validation_error: Option<String>,
}

impl ParsedDocument {
    #[must_use]
    pub fn uses_line_preview(&self) -> bool {
        !self.preview_lines.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingKind {
    #[default]
    Content,
    Highlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseStatus {
    #[default]
    Ready,
    Loading {
        kind: LoadingKind,
    },
}

/// Open document tab state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub title: String,
    pub content: String,
    pub parsed: ParsedDocument,
    pub is_dirty: bool,
    pub html_revision: u64,
    pub parse_gen: u64,
    pub parse_status: ParseStatus,
}
