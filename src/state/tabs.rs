use super::AppStore;
use crate::services::fs::save_document_file;
use crate::services::fts;
use crate::services::markdown::{parse_document, parse_markdown_document};
use crate::types::{DocumentFormat, DocumentMode, TabItem};
use std::path::{Path, PathBuf};

impl AppStore {
    /// Select an active tab by its id.
    pub fn select_tab(&mut self, id: usize) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = id;
            self.snapshot_current_workspace();
        }
    }

    /// Close a tab by its id.
    pub fn close_tab(&mut self, id: usize) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.active_tab_id == id {
                if self.tabs.is_empty() {
                    self.active_tab_id = 0;
                } else if pos < self.tabs.len() {
                    self.active_tab_id = self.tabs[pos].id;
                } else {
                    self.active_tab_id = self.tabs[pos - 1].id;
                }
            }
            self.snapshot_current_workspace();
        }
    }

    /// Close every tab except the one with `keep_id`.
    pub fn close_other_tabs(&mut self, keep_id: usize) {
        self.tabs.retain(|t| t.id == keep_id);
        self.active_tab_id = keep_id;
    }

    /// Create a new blank tab without content.
    pub fn new_empty_tab(&mut self) {
        let tab_id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);
        let parsed = parse_markdown_document("");

        self.tabs.push(TabItem {
            id: tab_id,
            path: None,
            title: format!("Doc-{tab_id}.md"),
            content: String::new(),
            parsed,
            is_dirty: false,
            html_revision: 0,
        });
        self.active_tab_id = tab_id;
    }

    /// Update active tab content when user types in editor or formats in WYSIWYG.
    pub fn update_active_tab_content(&mut self, new_content: String) {
        let active_id = self.active_tab_id;
        let skip_parse = self.mode == DocumentMode::Wysiwyg;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            if tab.content != new_content {
                tab.content = new_content;
                tab.is_dirty = true;
                if !skip_parse {
                    let format = DocumentFormat::from_path(tab.path.as_deref());
                    tab.parsed = parse_document(&tab.content, format);
                }
            }
        }
    }

    fn reparse_active_tab(&mut self) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            let format = DocumentFormat::from_path(tab.path.as_deref());
            tab.parsed = parse_document(&tab.content, format);
        }
    }

    fn bump_active_tab_html_revision(&mut self) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            tab.html_revision = tab.html_revision.wrapping_add(1);
        }
    }

    fn sync_mode_transition(&mut self, prev: DocumentMode, next: DocumentMode) {
        if prev == DocumentMode::Wysiwyg || next == DocumentMode::Wysiwyg {
            self.reparse_active_tab();
            if next == DocumentMode::Wysiwyg {
                self.bump_active_tab_html_revision();
            }
        }
    }

    /// Toggle a Markdown task checkbox (- [ ] / - [x]) at target index in active tab.
    pub fn toggle_active_tab_task(&mut self, target_idx: usize, is_checked: bool) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
            let mut current_idx = 0;
            let mut new_lines = Vec::new();
            let check_char = if is_checked { 'x' } else { ' ' };

            for line in tab.content.lines() {
                let trimmed = line.trim_start();
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];

                let is_task = (trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] "))
                    || (trimmed.starts_with("* [ ] ") || trimmed.starts_with("* [x] ") || trimmed.starts_with("* [X] "))
                    || (trimmed.starts_with("+ [ ] ") || trimmed.starts_with("+ [x] ") || trimmed.starts_with("+ [X] "));

                if is_task {
                    if current_idx == target_idx {
                        let bullet = &trimmed[..1];
                        let rest = &trimmed[6..];
                        new_lines.push(format!("{indent}{bullet} [{check_char}] {rest}"));
                    } else {
                        new_lines.push(line.to_string());
                    }
                    current_idx += 1;
                } else if let Some(pos) = trimmed.find(". [") {
                    let prefix_num = &trimmed[..pos];
                    let after_dot = &trimmed[pos..];
                    if prefix_num.chars().all(|c| c.is_ascii_digit())
                        && (after_dot.starts_with(". [ ] ") || after_dot.starts_with(". [x] ") || after_dot.starts_with(". [X] "))
                    {
                        if current_idx == target_idx {
                            let rest = &after_dot[6..];
                            new_lines.push(format!("{indent}{prefix_num}. [{check_char}] {rest}"));
                        } else {
                            new_lines.push(line.to_string());
                        }
                        current_idx += 1;
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    new_lines.push(line.to_string());
                }
            }

            let mut new_content = new_lines.join("\n");
            if tab.content.ends_with('\n') {
                new_content.push('\n');
            }
            if tab.content != new_content {
                tab.content = new_content;
                let format = DocumentFormat::from_path(tab.path.as_deref());
                tab.parsed = parse_document(&tab.content, format);
                tab.is_dirty = true;
                tab.html_revision = tab.html_revision.wrapping_add(1);
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
                    tab.html_revision = tab.html_revision.wrapping_add(1);
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
                let path = path.clone();
                let content = tab.content.clone();
                std::thread::spawn(move || {
                    let _ = fts::upsert_path(&path, &content);
                });
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
    pub fn set_mode(&mut self, mode: DocumentMode) {
        let prev = self.mode;
        if prev != mode {
            self.mode = mode;
            self.sync_mode_transition(prev, mode);
        }
    }

    /// Cycle to the next document viewing/editing mode.
    pub fn cycle_mode(&mut self) {
        let prev = self.mode;
        self.mode = self.mode.next();
        if prev != self.mode {
            self.sync_mode_transition(prev, self.mode);
        }
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
                    tab.html_revision = tab.html_revision.wrapping_add(1);
                    let path = p.clone();
                    let content = new_content.to_string();
                    std::thread::spawn(move || {
                        let _ = fts::upsert_path(&path, &content);
                    });
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_active_tab_task() {
        let mut store = AppStore::default();
        store.new_empty_tab();
        let markdown = "# Tasks\n\n- [ ] Task 1\n- [x] Task 2\n- [ ] Task 3\n";
        store.update_active_tab_content(markdown.to_string());

        // Toggle index 0 from unchecked to checked
        store.toggle_active_tab_task(0, true);
        let tab = store.active_tab().unwrap();
        assert!(tab.content.contains("- [x] Task 1"));
        assert!(tab.content.contains("- [x] Task 2"));
        assert!(tab.content.contains("- [ ] Task 3"));

        // Toggle index 1 from checked to unchecked
        store.toggle_active_tab_task(1, false);
        let tab = store.active_tab().unwrap();
        assert!(tab.content.contains("- [ ] Task 2"));

        // Toggle index 2 from unchecked to checked
        store.toggle_active_tab_task(2, true);
        let tab = store.active_tab().unwrap();
        assert!(tab.content.contains("- [x] Task 3"));
    }

    #[test]
    fn test_new_empty_tab_is_blank() {
        let mut store = AppStore::default();
        let initial_tabs_len = store.tabs.len();
        store.new_empty_tab();

        assert_eq!(store.tabs.len(), initial_tabs_len + 1);
        let new_tab = store.active_tab().expect("active tab should exist");
        assert_eq!(new_tab.content, "");
        assert!(new_tab.path.is_none());
        assert!(!new_tab.is_dirty);
    }

    #[test]
    fn test_close_tab_and_close_last_tab() {
        let mut store = AppStore::default();
        assert_eq!(store.tabs.len(), 1);
        let first_id = store.tabs[0].id;

        // Close the only remaining tab
        store.close_tab(first_id);
        assert!(store.tabs.is_empty());
        assert_eq!(store.active_tab_id, 0);
        assert!(store.active_tab().is_none());

        // Opening a new tab works after all tabs are closed
        store.new_empty_tab();
        assert_eq!(store.tabs.len(), 1);
        assert!(store.active_tab().is_some());
    }

    #[test]
    fn test_line_count_with_newlines() {
        assert_eq!("".split('\n').count(), 1);
        assert_eq!("\n".split('\n').count(), 2);
        assert_eq!("\n\n".split('\n').count(), 3);
        assert_eq!("\n\n\n".split('\n').count(), 4);
        assert_eq!("hello\n".split('\n').count(), 2);
        assert_eq!("hello\nworld\n".split('\n').count(), 3);
    }
}

