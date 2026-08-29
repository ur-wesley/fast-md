use crate::services::fs::{read_document_file, save_document_file, scan_markdown_tree};
use crate::services::markdown::parse_markdown_document;
use crate::services::settings::{load_settings, save_settings};
use crate::types::{
    AppSettings, AppTheme, DocumentMode, FileTreeEntry, Language, SidebarTab, TabItem, UpdateStatus,
};
use std::path::{Path, PathBuf};

pub const WELCOME_DOC: &str = r#"---
title: Fast Native Markdown & MDX Viewer
description: Ultra-fast, lightweight desktop documentation viewer built with Dioxus and Rust.
author: Dioxus Fast-MD
date: 2026-08-26
tags: [rust, dioxus, markdown, mdx, desktop]
---

# Welcome to Fast-MD 🚀

**Fast-MD** is a native, high-performance Markdown and MDX reader engineered in **Rust** with **Dioxus 0.6**. It starts up instantly, parses documents natively with zero JavaScript lag, and provides live auto-reload when files are modified in your favorite editor.

---

## ⚡ Key Highlights

- **Instant Launch**: Native binary performance with Windows WebView2 integration.
- **GFM & MDX Native Rendering**: Full support for tables, task lists, footnotes, frontmatter, and JSX components.
- **Native Syntax Highlighting**: Powered by `syntect` tokenization for blazing fast code rendering.
- **Live File Watcher**: Automatically re-renders files on save via `notify`.
- **Keyboard Navigation**: `Ctrl+O` (Open), `Ctrl+F` (Search), `Ctrl+T` (New Tab), `Ctrl+W` (Close Tab), `Ctrl+,` (Settings), `Esc` / `Ctrl+Shift+F` (Zen Mode).

---

## 💻 Rust Code Snippet Example

```rust
use dioxus::prelude::*;

#[component]
pub fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            class: "p-4 border rounded shadow",
            button {
                onclick: move |_| count += 1,
                "Clicks: {count}"
            }
        }
    }
}
```

---

## 📦 MDX Component Showcase

<Callout type="info">
  This is a custom MDX **Callout** component rendered natively without external web dependencies.
</Callout>

<Warning>
  You can edit this file in VSCode or Neovim and watch it live-update in real time!
</Warning>

<Card>
  ### Interactive Documentation
  Organize docs with sidebars, search through headings instantly, and export to standalone HTML.
</Card>

---

## 📋 Task List & Tables

- [x] High-performance Rust parsing
- [x] Zero-panic runtime architecture
- [x] Catppuccin theme family (Mocha, Macchiato, Frappé, Latte) & Classic themes
- [x] Interactive primary color picker with palette presets
- [x] Centralized reactive state store with file-based JSON persistence

| Feature | Fast-MD (Dioxus) | Typical Electron Viewer |
| :--- | :--- | :--- |
| **Startup Time** | **< 150ms** | 1.5s - 3.5s |
| **Memory Usage** | **~35 MB** | 180MB - 350MB |
| **Code Highlighting** | **Native Rust (syntect)** | Client-side JS |
| **Live Reload** | **OS Event (notify)** | Polling / Dev server |

---

*Enjoy reading documentation at native speed!*
"#;

/// Central Application State Store managing all tabs, preferences, layout, and document data.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppStore {
    pub tabs: Vec<TabItem>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub mode: DocumentMode,
    pub language: Language,
    pub theme: AppTheme,
    pub primary_color: Option<String>,
    pub is_zen: bool,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub show_sidebar: bool,
    pub sidebar_tab: SidebarTab,
    pub sticky_headers: bool,
    pub show_search: bool,
    pub show_settings_modal: bool,
    pub file_tree: Vec<FileTreeEntry>,
    pub opened_folder: Option<PathBuf>,
    pub settings: AppSettings,
    pub update_status: UpdateStatus,
}

impl Default for AppStore {
    fn default() -> Self {
        let welcome_parsed = parse_markdown_document(WELCOME_DOC);
        let settings = load_settings();
        Self {
            tabs: vec![TabItem {
                id: 1,
                path: None,
                title: "Welcome.md".to_string(),
                content: WELCOME_DOC.to_string(),
                parsed: welcome_parsed,
                is_dirty: false,
            }],
            active_tab_id: 1,
            next_tab_id: 2,
            mode: settings.default_mode,
            language: settings.language,
            theme: settings.theme,
            primary_color: settings.primary_color.clone(),
            is_zen: false,
            is_full_width: settings.is_full_width,
            zoom_level: settings.zoom_level,
            show_sidebar: settings.show_sidebar,
            sidebar_tab: settings.sidebar_tab,
            sticky_headers: settings.sticky_headers,
            show_search: false,
            show_settings_modal: false,
            file_tree: Vec::new(),
            opened_folder: None,
            settings,
            update_status: UpdateStatus::Idle,
        }
    }
}

impl AppStore {
    /// Initialize state with optional startup path, CLI theme/language override, and zen mode flags.
    #[must_use]
    pub fn new_with_options(
        initial_path: Option<&Path>,
        cli_theme: Option<AppTheme>,
        cli_lang: Option<Language>,
        zen: bool,
    ) -> Self {
        let settings = load_settings();
        let effective_theme = cli_theme.unwrap_or(settings.theme);
        let effective_language = cli_lang.unwrap_or(settings.language);

        let mut store = Self {
            mode: settings.default_mode,
            language: effective_language,
            theme: effective_theme,
            primary_color: settings.primary_color.clone(),
            is_zen: zen,
            is_full_width: settings.is_full_width,
            zoom_level: settings.zoom_level,
            show_sidebar: settings.show_sidebar,
            sidebar_tab: settings.sidebar_tab,
            sticky_headers: settings.sticky_headers,
            settings,
            ..Default::default()
        };

        if let Some(path) = initial_path {
            if path.is_file() {
                store.open_file_from_path(path.to_path_buf());
                // Remove the default welcome tab so only the requested file is open
                store.tabs.retain(|t| t.id != 1);
            } else if path.is_dir() {
                store.open_directory(path.to_path_buf());
            }
        }

        store
    }

    /// Persist current settings to the settings.json file safely.
    pub fn persist_settings(&self) {
        let _ = save_settings(&self.settings);
    }

    /// Retrieve the currently active tab if one exists.
    #[must_use]
    pub fn active_tab(&self) -> Option<&TabItem> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
    }

    /// Open a file from a `PathBuf` into a tab (or activate if already open).
    pub fn open_file_from_path(&mut self, path: PathBuf) {
        // Track in recent files
        self.settings.add_recent_file(path.clone());
        self.persist_settings();

        // If file is already open, activate that tab
        if let Some(existing) = self.tabs.iter().find(|t| t.path.as_ref() == Some(&path)) {
            self.active_tab_id = existing.id;
            return;
        }

        let content = read_document_file(&path).unwrap_or_else(|_| WELCOME_DOC.to_string());
        let parsed = parse_markdown_document(&content);
        let title = path.file_name().map_or_else(|| "Document".to_string(), |n| n.to_string_lossy().to_string());

        let tab_id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);

        // If we haven't loaded a file tree yet, load the parent directory
        if self.file_tree.is_empty() {
            if let Some(parent) = path.parent() {
                if let Ok(tree) = scan_markdown_tree(parent) {
                    self.file_tree = tree;
                    self.opened_folder = Some(parent.to_path_buf());
                }
            }
        }

        self.tabs.push(TabItem {
            id: tab_id,
            path: Some(path),
            title,
            content,
            parsed,
            is_dirty: false,
        });

        self.active_tab_id = tab_id;
    }

    /// Open a directory into the file tree sidebar.
    pub fn open_directory(&mut self, dir: PathBuf) {
        self.settings.add_recent_folder(dir.clone());
        self.persist_settings();

        if let Ok(tree) = scan_markdown_tree(&dir) {
            self.file_tree = tree;
            self.opened_folder = Some(dir);
            self.sidebar_tab = SidebarTab::Files;
            self.show_sidebar = true;
        }
    }

    /// Select an active tab by its id.
    pub fn select_tab(&mut self, id: usize) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = id;
        }
    }

    /// Close a tab by its id.
    pub fn close_tab(&mut self, id: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.retain(|t| t.id != id);
        if self.active_tab_id == id {
            self.active_tab_id = self.tabs.first().map_or(0, |t| t.id);
        }
    }

    /// Create a new blank or welcome tab.
    pub fn new_empty_tab(&mut self) {
        let tab_id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);
        let parsed = parse_markdown_document(WELCOME_DOC);

        self.tabs.push(TabItem {
            id: tab_id,
            path: None,
            title: format!("Doc-{tab_id}.md"),
            content: WELCOME_DOC.to_string(),
            parsed,
            is_dirty: false,
        });
        self.active_tab_id = tab_id;
    }

    /// Update active tab content when user types in editor or formats in WYSIWYG.
    pub fn update_active_tab_content(&mut self, new_content: String) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            if tab.content != new_content {
                tab.content = new_content;
                tab.parsed = parse_markdown_document(&tab.content);
                tab.is_dirty = true;
            }
        }
    }

    /// Format the Markdown source of the active tab (aligns tables, normalizes whitespace).
    pub fn format_active_tab(&mut self) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            let formatted = crate::services::formatter::format_markdown(&tab.content);
            if tab.content != formatted {
                tab.content = formatted;
                tab.parsed = parse_markdown_document(&tab.content);
                tab.is_dirty = true;
            }
        }
    }

    /// Save the active tab to disk if it has a file path. Returns true if saved directly.
    pub fn save_active_tab(&mut self) -> Result<bool, eyre::Report> {
        if self.settings.format_on_save {
            self.format_active_tab();
        }

        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            if let Some(ref path) = tab.path {
                save_document_file(path, &tab.content)?;
                tab.is_dirty = false;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Save a tab with a newly specified path (e.g. after Save As dialog).
    pub fn save_tab_with_path(&mut self, tab_id: usize, path: PathBuf) -> Result<(), eyre::Report> {
        if self.settings.format_on_save {
            self.format_active_tab();
        }

        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            save_document_file(&path, &tab.content)?;
            let file_name = path
                .file_name()
                .map_or_else(|| "Document.md".to_string(), |f| f.to_string_lossy().to_string());
            tab.path = Some(path.clone());
            tab.title = file_name;
            tab.is_dirty = false;

            self.settings.add_recent_file(path);
            self.persist_settings();
        }
        Ok(())
    }

    /// Set whether to automatically format Markdown on save.
    #[allow(dead_code)]
    pub fn set_format_on_save(&mut self, enabled: bool) {
        self.settings.format_on_save = enabled;
        self.persist_settings();
    }

    /// Toggle whether to automatically format Markdown on save.
    pub fn toggle_format_on_save(&mut self) {
        self.settings.format_on_save = !self.settings.format_on_save;
        self.persist_settings();
    }

    /// Set document viewing/editing mode.
    pub const fn set_mode(&mut self, mode: DocumentMode) {
        self.mode = mode;
    }

    /// Cycle to the next document viewing/editing mode.
    pub const fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    /// Set and persist default startup document mode.
    pub fn set_default_mode(&mut self, mode: DocumentMode) {
        self.settings.default_mode = mode;
        self.persist_settings();
    }

    /// Update file content if changed on disk.
    pub fn update_file_content_if_modified(&mut self, path: &Path, new_content: &str) {
        for tab in &mut self.tabs {
            if let Some(ref p) = tab.path {
                if p == path && tab.content != new_content {
                    tab.content.clone_from(&new_content.to_string());
                    tab.parsed = parse_markdown_document(new_content);
                    tab.is_dirty = false;
                }
            }
        }
    }

    /// Zoom in by 10% (up to 250%).
    pub fn zoom_in(&mut self) {
        if self.zoom_level < 250 {
            self.zoom_level = self.zoom_level.saturating_add(10);
            self.settings.zoom_level = self.zoom_level;
            self.persist_settings();
        }
    }

    /// Zoom out by 10% (down to 50%).
    pub fn zoom_out(&mut self) {
        if self.zoom_level > 50 {
            self.zoom_level = self.zoom_level.saturating_sub(10);
            self.settings.zoom_level = self.zoom_level;
            self.persist_settings();
        }
    }

    /// Reset zoom to default 100%.
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 100;
        self.settings.zoom_level = self.zoom_level;
        self.persist_settings();
    }

    /// Toggle Zen mode.
    pub const fn toggle_zen(&mut self) {
        self.is_zen = !self.is_zen;
    }

    /// Set Zen mode explicitly.
    pub const fn set_zen(&mut self, zen: bool) {
        self.is_zen = zen;
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        self.settings.show_sidebar = self.show_sidebar;
        self.persist_settings();
    }

    /// Toggle reading column vs full width.
    pub fn toggle_full_width(&mut self) {
        self.is_full_width = !self.is_full_width;
        self.settings.is_full_width = self.is_full_width;
        self.persist_settings();
    }

    /// Toggle search overlay.
    #[allow(dead_code)]
    pub const fn toggle_search(&mut self) {
        self.show_search = !self.show_search;
    }

    /// Toggle settings modal dialog.
    pub const fn toggle_settings_modal(&mut self) {
        self.show_settings_modal = !self.show_settings_modal;
    }

    /// Set settings modal dialog visibility explicitly.
    pub const fn set_settings_modal(&mut self, show: bool) {
        self.show_settings_modal = show;
    }

    /// Set theme and persist.
    pub fn set_theme(&mut self, theme: AppTheme) {
        self.theme = theme;
        self.settings.theme = theme;
        self.persist_settings();
    }

    /// Set custom primary color (or None to reset to default) and persist.
    pub fn set_primary_color(&mut self, color: Option<String>) {
        self.primary_color.clone_from(&color);
        self.settings.primary_color = color;
        self.persist_settings();
    }

    /// Retrieve the effective primary accent color.
    #[must_use]
    pub fn effective_primary_color(&self) -> &str {
        self.primary_color
            .as_deref()
            .unwrap_or_else(|| self.theme.default_accent())
    }

    /// Set sidebar tab mode and persist.
    pub fn set_sidebar_tab(&mut self, tab: SidebarTab) {
        self.sidebar_tab = tab;
        self.settings.sidebar_tab = tab;
        self.persist_settings();
    }

    /// Set auto-reload setting and persist.
    pub fn set_auto_reload(&mut self, auto_reload: bool) {
        self.settings.auto_reload = auto_reload;
        self.persist_settings();
    }

    /// Set sticky markdown headers setting and persist.
    pub fn set_sticky_headers(&mut self, sticky: bool) {
        self.sticky_headers = sticky;
        self.settings.sticky_headers = sticky;
        self.persist_settings();
    }

    /// Toggle sticky markdown headers setting and persist.
    #[allow(dead_code)]
    pub fn toggle_sticky_headers(&mut self) {
        self.sticky_headers = !self.sticky_headers;
        self.settings.sticky_headers = self.sticky_headers;
        self.persist_settings();
    }

    /// Set font size setting and persist.
    pub fn set_font_size(&mut self, size: u32) {
        self.settings.font_size = size;
        self.persist_settings();
    }

    /// Set language and persist.
    #[allow(dead_code)]
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.settings.language = language;
        self.persist_settings();
    }

    /// Update the current reactive update status.
    pub fn set_update_status(&mut self, status: UpdateStatus) {
        self.update_status = status;
    }

    /// Toggle or set auto check for updates and persist.
    pub fn set_auto_check_updates(&mut self, enabled: bool) {
        self.settings.auto_check_updates = enabled;
        self.persist_settings();
    }

    /// Reset all settings to application defaults and persist.
    pub fn reset_settings_to_default(&mut self) {
        let defaults = AppSettings::default();
        self.language = defaults.language;
        self.theme = defaults.theme;
        self.primary_color.clone_from(&defaults.primary_color);
        self.is_full_width = defaults.is_full_width;
        self.zoom_level = defaults.zoom_level;
        self.show_sidebar = defaults.show_sidebar;
        self.sidebar_tab = defaults.sidebar_tab;
        self.sticky_headers = defaults.sticky_headers;
        self.settings = defaults;
        self.update_status = UpdateStatus::Idle;
        self.persist_settings();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_color_selection() {
        let mut store = AppStore::default();
        store.set_primary_color(None);
        // Select Catppuccin Mocha
        store.set_theme(AppTheme::CatppuccinMocha);
        assert_eq!(store.effective_primary_color(), "#cba6f7"); // Mocha default accent

        // Select custom primary color
        store.set_primary_color(Some("#ff007f".to_string()));
        assert_eq!(store.effective_primary_color(), "#ff007f");

        // Reset to theme default
        store.set_primary_color(None);
        assert_eq!(store.effective_primary_color(), "#cba6f7");
    }

    #[test]
    fn test_store_settings_sync() {
        let mut store = AppStore::default();
        store.reset_settings_to_default();
        assert_eq!(store.language, Language::En);

        store.set_language(Language::De);
        assert_eq!(store.language, Language::De);
        assert_eq!(store.settings.language, Language::De);

        store.set_theme(AppTheme::CatppuccinFrappe);
        assert_eq!(store.settings.theme, AppTheme::CatppuccinFrappe);

        store.toggle_full_width();
        assert!(store.settings.is_full_width);

        store.set_auto_reload(false);
        assert!(!store.settings.auto_reload);

        assert!(!store.sticky_headers);
        store.toggle_sticky_headers();
        assert!(store.sticky_headers);
        assert!(store.settings.sticky_headers);

        store.reset_settings_to_default();
        assert_eq!(store.settings.language, Language::En);
        assert_eq!(store.language, Language::En);
        assert_eq!(store.settings.theme, AppTheme::Dark);
        assert_eq!(store.theme, AppTheme::Dark);
        assert!(!store.is_full_width);
        assert!(store.settings.auto_reload);
        assert!(!store.sticky_headers);
    }

    #[test]
    fn test_tab_editing_and_dirty_state() {
        let mut store = AppStore::default();
        assert_eq!(store.mode, DocumentMode::View);
        assert!(!store.tabs[0].is_dirty);

        store.cycle_mode();
        assert_eq!(store.mode, DocumentMode::Split);
        store.set_mode(DocumentMode::Wysiwyg);
        assert_eq!(store.mode, DocumentMode::Wysiwyg);

        store.update_active_tab_content("# Modified Title\n\nNew edited body text.".to_string());
        assert!(store.tabs[0].is_dirty);
        assert_eq!(store.tabs[0].parsed.toc.len(), 1);
        assert_eq!(store.tabs[0].parsed.toc[0].title, "Modified Title");
    }

    #[test]
    fn test_formatting_and_format_on_save() {
        let mut store = AppStore::default();
        let unformatted = "# Unformatted\n| A | B |\n|---|---|\n| 1 | 2 |\n";
        store.update_active_tab_content(unformatted.to_string());

        assert!(store.settings.format_on_save);
        store.format_active_tab();

        let formatted = &store.tabs[0].content;
        assert!(formatted.contains("| A   | B   |"));
        assert!(formatted.contains("| --- | --- |"));
        assert!(formatted.contains("| 1   | 2   |"));

        store.set_format_on_save(false);
        assert!(!store.settings.format_on_save);
        store.toggle_format_on_save();
        assert!(store.settings.format_on_save);
    }
}


