use super::{AppTheme, DocumentMode, Language, ShortcutsConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Active sidebar tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SidebarTab {
    #[default]
    Toc,
    Files,
}

/// Sidebar position (Left or Right side of workspace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SidebarPosition {
    #[default]
    Left,
    Right,
}

impl SidebarPosition {
    #[must_use]
    pub const fn is_left(self) -> bool {
        matches!(self, Self::Left)
    }

    #[must_use]
    pub const fn is_right(self) -> bool {
        matches!(self, Self::Right)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// File visibility filter mode in the sidebar file explorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileFilterMode {
    #[default]
    MarkdownAndConfig,
    MarkdownOnly,
    AllSupported,
    AllFiles,
}

impl FileFilterMode {
    #[must_use]
    pub fn matches_extension(self, ext: &str) -> bool {
        let ext_lower = ext.to_ascii_lowercase();
        match self {
            Self::MarkdownOnly => matches!(ext_lower.as_str(), "md" | "mdx" | "markdown" | "mdown"),
            Self::MarkdownAndConfig => matches!(
                ext_lower.as_str(),
                "md" | "mdx" | "markdown" | "mdown" | "json" | "jsonc" | "json5" | "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" | "ron" | "xml" | "txt" | "rst"
            ),
            Self::AllSupported => matches!(
                ext_lower.as_str(),
                "md" | "mdx" | "markdown" | "mdown" | "json" | "jsonc" | "json5" | "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" | "ron" | "xml" | "txt" | "rst"
                | "rs" | "js" | "ts" | "jsx" | "tsx" | "html" | "css" | "scss" | "py" | "sh" | "bat" | "cmd" | "ps1" | "sql" | "c" | "cpp" | "h" | "go" | "java" | "csv" | "tsv" | "log"
            ),
            Self::AllFiles => true,
        }
    }

    #[must_use]
    pub fn matches_path(self, path: &std::path::Path) -> bool {
        if matches!(self, Self::AllFiles) {
            return true;
        }
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| self.matches_extension(ext))
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MarkdownOnly => "MD / MDX",
            Self::MarkdownAndConfig => "MD + Config",
            Self::AllSupported => "All Supported",
            Self::AllFiles => "All Files",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MarkdownOnly => "md",
            Self::MarkdownAndConfig => "config",
            Self::AllSupported => "supported",
            Self::AllFiles => "all",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn next(self) -> Self {
        match self {
            Self::MarkdownAndConfig => Self::MarkdownOnly,
            Self::MarkdownOnly => Self::AllSupported,
            Self::AllSupported => Self::AllFiles,
            Self::AllFiles => Self::MarkdownAndConfig,
        }
    }
}

const fn default_true() -> bool {
    true
}

const fn default_zoom() -> u32 {
    100
}

const fn default_font_size() -> u32 {
    16
}

const fn default_sidebar_width() -> u32 {
    260
}

/// Persistent user configuration and application preferences stored in `settings.json`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// User interface language.
    #[serde(default)]
    pub language: Language,

    /// Visual theme.
    #[serde(default)]
    pub theme: AppTheme,

    /// Custom primary accent hex color (e.g. "#cba6f7" or None for theme default).
    #[serde(default)]
    pub primary_color: Option<String>,

    /// Full-width layout mode (true) vs centered reading column (false).
    #[serde(default)]
    pub is_full_width: bool,

    /// Default viewer zoom percentage (e.g. 100 for 100%).
    #[serde(default = "default_zoom")]
    pub zoom_level: u32,

    /// Whether the sidebar is shown by default.
    #[serde(default = "default_true")]
    pub show_sidebar: bool,

    /// Initial sidebar tab (Outline / Files).
    #[serde(default)]
    pub sidebar_tab: SidebarTab,

    /// Sidebar position (Left or Right).
    #[serde(default)]
    pub sidebar_position: SidebarPosition,

    /// Sidebar width in pixels.
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,

    /// Active file visibility filter for the sidebar file explorer.
    #[serde(default)]
    pub file_filter_mode: FileFilterMode,

    /// Automatically reload files when modified on disk.
    #[serde(default = "default_true")]
    pub auto_reload: bool,

    /// Pin markdown headings (H1-H6) to top of viewport while scrolling.
    #[serde(default)]
    pub sticky_headers: bool,

    /// Document base font size in pixels (default 16).
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Soft-wrap long lines in the source editor instead of horizontal scrolling.
    #[serde(default)]
    pub line_wrap: bool,

    /// Default startup document viewing/editing mode.
    #[serde(default)]
    pub default_mode: DocumentMode,

    /// Automatically format Markdown source (align tables, clean whitespace) on save.
    #[serde(default = "default_true")]
    pub format_on_save: bool,

    /// Optional custom font family override (e.g. "Inter", "Fira Code").
    #[serde(default)]
    pub font_family: Option<String>,

    /// Recently opened file paths (capped at 10 most recent).
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,

    /// Recently opened folder paths (capped at 10 most recent).
    #[serde(default)]
    pub recent_folders: Vec<PathBuf>,

    /// Optional custom user CSS injected into the viewer.
    #[serde(default)]
    pub custom_css: Option<String>,

    /// Automatically check for application updates on startup.
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,

    /// Configurable keyboard shortcuts for app actions.
    #[serde(default)]
    pub shortcuts: ShortcutsConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::En,
            theme: AppTheme::Dark,
            primary_color: None,
            is_full_width: false,
            zoom_level: default_zoom(),
            show_sidebar: default_true(),
            sidebar_tab: SidebarTab::Toc,
            sidebar_position: SidebarPosition::Left,
            sidebar_width: default_sidebar_width(),
            file_filter_mode: FileFilterMode::MarkdownAndConfig,
            auto_reload: default_true(),
            sticky_headers: false,
            font_size: default_font_size(),
            line_wrap: false,
            default_mode: DocumentMode::View,
            format_on_save: default_true(),
            font_family: None,
            recent_files: Vec::new(),
            recent_folders: Vec::new(),
            custom_css: None,
            auto_check_updates: default_true(),
            shortcuts: ShortcutsConfig::default(),
        }
    }
}

/// Detailed metadata about an available GitHub release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub name: String,
    pub release_notes: String,
    pub asset_name: String,
    pub download_url: String,
    pub published_at: String,
    pub html_url: String,
}

/// Reactive application update lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(ReleaseInfo),
    Downloading {
        version: String,
        progress: u8,
    },
    Installing {
        version: String,
    },
    ReadyToRestart {
        version: String,
    },
    Error(String),
}

impl UpdateStatus {
    #[must_use]
    pub const fn is_checking(&self) -> bool {
        matches!(self, Self::Checking)
    }

    #[must_use]
    pub const fn is_downloading(&self) -> bool {
        matches!(self, Self::Downloading { .. } | Self::Installing { .. })
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn is_ready_to_restart(&self) -> bool {
        matches!(self, Self::ReadyToRestart { .. })
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn available_release(&self) -> Option<&ReleaseInfo> {
        if let Self::Available(info) = self {
            Some(info)
        } else {
            None
        }
    }
}

impl AppSettings {
    /// Add a path to the recent files list (most recent first, deduplicated, capped at 10).
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    /// Add a path to the recent folders list (most recent first, deduplicated, capped at 10).
    pub fn add_recent_folder(&mut self, path: PathBuf) {
        self.recent_folders.retain(|p| p != &path);
        self.recent_folders.insert(0, path);
        if self.recent_folders.len() > 10 {
            self.recent_folders.truncate(10);
        }
    }

    /// Clear recent files history.
    #[allow(dead_code)]
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    /// Clear recent folders history.
    #[allow(dead_code)]
    pub fn clear_recent_folders(&mut self) {
        self.recent_folders.clear();
    }
}

/// Recursive file tree node.
#[derive(Debug, Clone, Eq)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Arc<Vec<Self>>,
}

impl PartialEq for FileTreeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.path == other.path
            && self.is_dir == other.is_dir
            && Arc::ptr_eq(&self.children, &other.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentFormat;

    #[test]
    fn test_catppuccin_themes() {
        assert_eq!(AppTheme::CatppuccinMocha.as_str(), "theme-catppuccin-mocha");
        assert_eq!(AppTheme::CatppuccinMacchiato.as_str(), "theme-catppuccin-macchiato");
        assert_eq!(AppTheme::CatppuccinFrappe.as_str(), "theme-catppuccin-frappe");
        assert_eq!(AppTheme::CatppuccinLatte.as_str(), "theme-catppuccin-latte");

        assert_eq!(AppTheme::CatppuccinMocha.label(), "Catppuccin Mocha");
        assert_eq!(AppTheme::CatppuccinLatte.label(), "Catppuccin Latte");

        assert!(AppTheme::CatppuccinMocha.is_dark());
        assert!(AppTheme::CatppuccinMacchiato.is_dark());
        assert!(AppTheme::CatppuccinFrappe.is_dark());
        assert!(!AppTheme::CatppuccinLatte.is_dark());

        assert_eq!(AppTheme::CatppuccinMocha.default_accent(), "#cba6f7");
        assert_eq!(AppTheme::CatppuccinLatte.default_accent(), "#8839ef");
    }

    #[test]
    fn test_app_settings_serialization_and_defaults() {
        let default_settings = AppSettings::default();
        let json_str = serde_json::to_string_pretty(&default_settings).unwrap_or_default();
        assert!(json_str.contains("\"theme\": \"dark\""));
        assert!(json_str.contains("\"zoom-level\"") || json_str.contains("\"zoom_level\": 100"));

        let deserialized: AppSettings = serde_json::from_str(&json_str).unwrap_or_default();
        assert_eq!(deserialized, default_settings);
    }

    #[test]
    fn test_partial_settings_json_deserialization() {
        let partial_json = r##"{
            "theme": "catppuccin-mocha",
            "primary_color": "#ff007f",
            "is_full_width": true,
            "sticky_headers": true
        }"##;

        let parsed: AppSettings = serde_json::from_str(partial_json).unwrap_or_default();
        assert_eq!(parsed.theme, AppTheme::CatppuccinMocha);
        assert_eq!(parsed.primary_color, Some("#ff007f".to_string()));
        assert!(parsed.is_full_width);
        assert_eq!(parsed.zoom_level, 100);
        assert!(parsed.show_sidebar);
        assert_eq!(parsed.sidebar_tab, SidebarTab::Toc);
        assert_eq!(parsed.sidebar_position, SidebarPosition::Left);
        assert!(parsed.auto_reload);
        assert!(parsed.sticky_headers);
        assert_eq!(parsed.font_size, 16);
        assert!(!parsed.line_wrap);
        assert!(parsed.format_on_save);
        assert_eq!(parsed.shortcuts, ShortcutsConfig::default());
    }

    #[test]
    fn test_sidebar_position() {
        assert_eq!(SidebarPosition::Left.as_str(), "left");
        assert_eq!(SidebarPosition::Right.as_str(), "right");
        assert!(SidebarPosition::Left.is_left());
        assert!(!SidebarPosition::Left.is_right());
        assert!(SidebarPosition::Right.is_right());
        assert!(!SidebarPosition::Right.is_left());

        let json_left = serde_json::to_string(&SidebarPosition::Left).unwrap_or_default();
        assert_eq!(json_left, "\"left\"");

        let json_right = serde_json::to_string(&SidebarPosition::Right).unwrap_or_default();
        assert_eq!(json_right, "\"right\"");

        let parsed: SidebarPosition = serde_json::from_str("\"right\"").unwrap_or_default();
        assert_eq!(parsed, SidebarPosition::Right);
    }

    #[test]
    fn test_recent_items_management() {
        let mut settings = AppSettings::default();
        for i in 1..=15 {
            settings.add_recent_file(PathBuf::from(format!("file_{i}.md")));
        }
        assert_eq!(settings.recent_files.len(), 10);
        assert_eq!(settings.recent_files[0], PathBuf::from("file_15.md"));

        // Deduplication test: re-adding file_5 moves it to the top
        settings.add_recent_file(PathBuf::from("file_5.md"));
        assert_eq!(settings.recent_files.len(), 10);
        assert_eq!(settings.recent_files[0], PathBuf::from("file_5.md"));
    }

    #[test]
    fn test_language_enum_and_serialization() {
        assert_eq!(Language::En.code(), "en");
        assert_eq!(Language::De.code(), "de");
        assert_eq!(Language::En.label(), "English");
        assert_eq!(Language::De.label(), "Deutsch");

        let json_en = serde_json::to_string(&Language::En).unwrap_or_default();
        assert_eq!(json_en, "\"en\"");

        let json_de = serde_json::to_string(&Language::De).unwrap_or_default();
        assert_eq!(json_de, "\"de\"");

        let parsed_de: Language = serde_json::from_str("\"de\"").unwrap_or_default();
        assert_eq!(parsed_de, Language::De);

        let parsed_en: Language = serde_json::from_str("\"en\"").unwrap_or_default();
        assert_eq!(parsed_en, Language::En);
    }

    #[test]
    fn test_document_mode() {
        assert_eq!(DocumentMode::View.as_str(), "view");
        assert_eq!(DocumentMode::Split.as_str(), "split");
        assert_eq!(DocumentMode::Wysiwyg.as_str(), "wysiwyg");
        assert_eq!(DocumentMode::Source.as_str(), "source");

        assert_eq!(DocumentMode::View.next(), DocumentMode::Split);
        assert_eq!(DocumentMode::Split.next(), DocumentMode::Wysiwyg);
        assert_eq!(DocumentMode::Wysiwyg.next(), DocumentMode::Source);
        assert_eq!(DocumentMode::Source.next(), DocumentMode::View);

        assert_eq!(DocumentMode::View.label(), "View");
        assert_eq!(DocumentMode::Split.label(), "Split Preview");
        assert_eq!(DocumentMode::Wysiwyg.label(), "Editor");
        assert_eq!(DocumentMode::Source.label(), "Source");

        let json_wysiwyg = serde_json::to_string(&DocumentMode::Wysiwyg).unwrap_or_default();
        assert_eq!(json_wysiwyg, "\"wysiwyg\"");

        let parsed: DocumentMode = serde_json::from_str("\"split\"").unwrap_or_default();
        assert_eq!(parsed, DocumentMode::Split);
    }

    #[test]
    fn test_document_format_detection_and_helpers() {
        assert_eq!(DocumentFormat::from_extension("md"), DocumentFormat::Markdown);
        assert_eq!(DocumentFormat::from_extension("MDX"), DocumentFormat::Mdx);
        assert_eq!(DocumentFormat::from_extension("json"), DocumentFormat::Json);
        assert_eq!(DocumentFormat::from_extension("toml"), DocumentFormat::Toml);
        assert_eq!(DocumentFormat::from_extension("yaml"), DocumentFormat::Yaml);
        assert_eq!(DocumentFormat::from_extension("yml"), DocumentFormat::Yaml);
        assert_eq!(DocumentFormat::from_extension("ini"), DocumentFormat::Ini);
        assert_eq!(DocumentFormat::from_extension("ron"), DocumentFormat::Ron);
        assert_eq!(DocumentFormat::from_extension("xml"), DocumentFormat::Xml);
        assert_eq!(DocumentFormat::from_extension("txt"), DocumentFormat::PlainText);

        assert!(DocumentFormat::Json.is_config());
        assert!(DocumentFormat::Toml.is_config());
        assert!(DocumentFormat::Yaml.is_config());
        assert!(!DocumentFormat::Markdown.is_config());
        assert!(DocumentFormat::Markdown.is_markdown());
        assert!(DocumentFormat::Mdx.is_markdown());
        assert!(!DocumentFormat::Json.is_markdown());
    }

    #[test]
    fn test_file_filter_mode_matching() {
        let md_mode = FileFilterMode::MarkdownOnly;
        assert!(md_mode.matches_extension("md"));
        assert!(md_mode.matches_extension("mdx"));
        assert!(!md_mode.matches_extension("json"));
        assert!(!md_mode.matches_extension("toml"));

        let config_mode = FileFilterMode::MarkdownAndConfig;
        assert!(config_mode.matches_extension("md"));
        assert!(config_mode.matches_extension("mdx"));
        assert!(config_mode.matches_extension("json"));
        assert!(config_mode.matches_extension("toml"));
        assert!(config_mode.matches_extension("yaml"));
        assert!(config_mode.matches_extension("yml"));
        assert!(!config_mode.matches_extension("rs"));

        let all_supported = FileFilterMode::AllSupported;
        assert!(all_supported.matches_extension("rs"));
        assert!(all_supported.matches_extension("js"));

        let all_files = FileFilterMode::AllFiles;
        assert!(all_files.matches_extension("anything"));
        assert!(all_files.matches_path(&PathBuf::from("test.unknown")));
    }
}
