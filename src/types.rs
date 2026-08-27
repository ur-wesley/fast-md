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
}

/// Fully parsed Markdown/MDX document ready for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedDocument {
    pub html_content: String,
    pub toc: Vec<TocItem>,
    pub metadata: Option<DocMetadata>,
    pub word_count: usize,
    pub reading_time_minutes: usize,
}

/// Open document tab state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub title: String,
    pub content: String,
    pub parsed: ParsedDocument,
}

/// Available visual themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppTheme {
    #[default]
    Dark,
    Midnight,
    Light,
    Nord,
    SolarizedDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

impl AppTheme {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "theme-dark",
            Self::Midnight => "theme-midnight",
            Self::Light => "theme-light",
            Self::Nord => "theme-nord",
            Self::SolarizedDark => "theme-solarized",
            Self::CatppuccinLatte => "theme-catppuccin-latte",
            Self::CatppuccinFrappe => "theme-catppuccin-frappe",
            Self::CatppuccinMacchiato => "theme-catppuccin-macchiato",
            Self::CatppuccinMocha => "theme-catppuccin-mocha",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dark => "GitHub Dark",
            Self::Midnight => "Obsidian Night",
            Self::Light => "GitHub Light",
            Self::Nord => "Nordic Frost",
            Self::SolarizedDark => "Solarized Dark",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    #[must_use]
    pub const fn default_accent(self) -> &'static str {
        match self {
            Self::Dark => "#58a6ff",
            Self::Midnight => "#8b5cf6",
            Self::Light => "#0969da",
            Self::Nord => "#88c0d0",
            Self::SolarizedDark => "#268bd2",
            Self::CatppuccinLatte => "#8839ef",
            Self::CatppuccinFrappe => "#ca9ee6",
            Self::CatppuccinMacchiato => "#c6a0f6",
            Self::CatppuccinMocha => "#cba6f7",
        }
    }

    #[must_use]
    pub const fn default_bg(self) -> &'static str {
        match self {
            Self::Dark => "#161b22",
            Self::Midnight => "#12141c",
            Self::Light => "#f6f8fa",
            Self::Nord => "#3b4252",
            Self::SolarizedDark => "#073642",
            Self::CatppuccinLatte => "#eff1f5",
            Self::CatppuccinFrappe => "#303446",
            Self::CatppuccinMacchiato => "#24273a",
            Self::CatppuccinMocha => "#1e1e2e",
        }
    }

    #[must_use]
    pub const fn is_dark(self) -> bool {
        !matches!(self, Self::Light | Self::CatppuccinLatte)
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn acrylic_tint(self) -> (u8, u8, u8, u8) {
        match self {
            Self::Light => (255, 255, 255, 120),
            Self::CatppuccinLatte => (239, 241, 245, 120),
            Self::CatppuccinFrappe => (48, 52, 70, 130),
            Self::CatppuccinMacchiato => (36, 39, 58, 130),
            Self::CatppuccinMocha => (30, 30, 46, 130),
            Self::Midnight => (9, 10, 15, 130),
            Self::Nord => (46, 52, 64, 130),
            Self::SolarizedDark => (0, 43, 54, 130),
            Self::Dark => (13, 17, 23, 130),
        }
    }
}

/// Active sidebar tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SidebarTab {
    #[default]
    Toc,
    Files,
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

/// Persistent user configuration and application preferences stored in `settings.json`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
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

    /// Automatically reload files when modified on disk.
    #[serde(default = "default_true")]
    pub auto_reload: bool,

    /// Pin markdown headings (H1-H6) to top of viewport while scrolling.
    #[serde(default)]
    pub sticky_headers: bool,

    /// Document base font size in pixels (default 16).
    #[serde(default = "default_font_size")]
    pub font_size: u32,

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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: AppTheme::Dark,
            primary_color: None,
            is_full_width: false,
            zoom_level: default_zoom(),
            show_sidebar: default_true(),
            sidebar_tab: SidebarTab::Toc,
            auto_reload: default_true(),
            sticky_headers: false,
            font_size: default_font_size(),
            font_family: None,
            recent_files: Vec::new(),
            recent_folders: Vec::new(),
            custom_css: None,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<Self>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(parsed.auto_reload);
        assert!(parsed.sticky_headers);
        assert_eq!(parsed.font_size, 16);
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
}
