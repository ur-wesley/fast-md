use super::{AppStore, WELCOME_DOC};
use crate::services::fs::save_document_file;
use crate::services::markdown::{parse_document, parse_markdown_document};
use crate::types::{DocumentFormat, DocumentMode, TabItem};
use std::path::{Path, PathBuf};

impl AppStore {
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

    /// Close every tab except the one with `keep_id`.
    pub fn close_other_tabs(&mut self, keep_id: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.retain(|t| t.id == keep_id);
        self.active_tab_id = keep_id;
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
                let format = DocumentFormat::from_path(tab.path.as_deref());
                tab.parsed = parse_document(&tab.content, format);
                tab.is_dirty = true;
            }
        }
    }

    /// Format the source of the active tab (Markdown table alignment or JSON/TOML/YAML pretty printing).
    pub fn format_active_tab(&mut self) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            let format = DocumentFormat::from_path(tab.path.as_deref());
            if let Ok(formatted) = crate::services::formatter::format_document(&tab.content, format) {
                if tab.content != formatted {
                    tab.content = formatted;
                    tab.parsed = parse_document(&tab.content, format);
                    tab.is_dirty = true;
                }
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
                    let format = DocumentFormat::from_path(Some(p));
                    tab.parsed = parse_document(new_content, format);
                    tab.is_dirty = false;
                }
            }
        }
    }

    /// Toggle settings modal dialog.
    pub const fn toggle_settings_modal(&mut self) {
        self.show_settings_modal = !self.show_settings_modal;
    }

    /// Set settings modal dialog visibility explicitly.
    pub const fn set_settings_modal(&mut self, show: bool) {
        self.show_settings_modal = show;
    }
}
