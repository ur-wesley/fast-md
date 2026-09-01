use super::AppStore;
use crate::services::fs::scan_file_tree;
use crate::types::{
    DocumentFormat, FileFilterMode, FileTreeEntry, LoadingKind, ParseStatus, ParsedDocument,
    SidebarTab, TabItem,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenKind {
    Preview,
    Pinned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentLoadJob {
    pub tab_id: usize,
    pub path: PathBuf,
    pub gen: u64,
}

impl AppStore {
    pub fn next_parse_generation(&mut self) -> u64 {
        let gen = self.next_parse_gen;
        self.next_parse_gen = self.next_parse_gen.saturating_add(1);
        gen
    }

    pub fn apply_tab_content(&mut self, tab_id: usize, gen: u64, content: String) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) else {
            return false;
        };
        if tab.parse_gen != gen {
            return false;
        }
        tab.content = content;
        tab.parse_status = ParseStatus::Loading {
            kind: LoadingKind::Highlight,
        };
        true
    }

    pub fn apply_tab_parsed(&mut self, tab_id: usize, gen: u64, parsed: crate::types::ParsedDocument) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) else {
            return false;
        };
        if tab.parse_gen != gen {
            return false;
        }
        tab.parsed = parsed;
        tab.parse_status = ParseStatus::Ready;
        tab.html_revision = tab.html_revision.wrapping_add(1);
        true
    }

    pub fn pin_tab(&mut self, id: usize) {
        if self.preview_tab_id == Some(id) {
            self.preview_tab_id = None;
        }
    }

    fn tab_title_for_path(path: &Path) -> String {
        path.file_name()
            .map_or_else(|| "Document".to_string(), |n| n.to_string_lossy().to_string())
    }

    fn queue_tree_scan_for_path(&mut self, path: &Path) {
        if self.file_tree.is_empty() {
            if let Some(parent) = path.parent() {
                let parent = parent.to_path_buf();
                self.start_loading_directory(parent.clone());
                self.pending_tree_scan = Some(parent);
            }
        }
    }

    fn reuse_preview_tab(&mut self, preview_id: usize, path: PathBuf, title: String, format: DocumentFormat) {
        let gen = self.next_parse_generation();
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == preview_id) {
            tab.path = Some(path.clone());
            tab.title = title;
            tab.content = String::new();
            tab.parsed = ParsedDocument {
                format,
                ..ParsedDocument::default()
            };
            tab.is_dirty = false;
            tab.html_revision = 0;
            tab.parse_gen = gen;
            tab.parse_status = ParseStatus::Loading {
                kind: LoadingKind::Content,
            };
        }
        self.active_tab_id = preview_id;
        self.preview_tab_id = Some(preview_id);
        self.snapshot_current_workspace();
        self.pending_document_loads.push(DocumentLoadJob {
            tab_id: preview_id,
            path,
            gen,
        });
    }

  /// Open a file from a `PathBuf` into a tab shell (or activate if already open).
    pub fn open_file_from_path(&mut self, path: PathBuf, kind: OpenKind) {
        self.settings.add_recent_file(path.clone());
        self.persist_settings();

        if let Some(existing) = self.tabs.iter().find(|t| t.path.as_ref() == Some(&path)) {
            self.active_tab_id = existing.id;
            if kind == OpenKind::Pinned {
                self.pin_tab(existing.id);
            }
            self.snapshot_current_workspace();
            return;
        }

        let format = DocumentFormat::from_path(Some(&path));
        let title = Self::tab_title_for_path(&path);
        self.queue_tree_scan_for_path(&path);

        if kind == OpenKind::Preview {
            if let Some(preview_id) = self.preview_tab_id {
                let preview_state = self.tabs.iter().find(|t| t.id == preview_id).map(|t| t.is_dirty);
                match preview_state {
                    Some(false) => {
                        self.reuse_preview_tab(preview_id, path, title, format);
                        return;
                    }
                    Some(true) => self.pin_tab(preview_id),
                    None => self.preview_tab_id = None,
                }
            }
        }

        let tab_id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);
        let gen = self.next_parse_generation();

        self.tabs.push(TabItem {
            id: tab_id,
            path: Some(path.clone()),
            title,
            content: String::new(),
            parsed: ParsedDocument {
                format,
                ..ParsedDocument::default()
            },
            is_dirty: false,
            html_revision: 0,
            parse_gen: gen,
            parse_status: ParseStatus::Loading {
                kind: LoadingKind::Content,
            },
        });

        self.active_tab_id = tab_id;
        if kind == OpenKind::Preview {
            self.preview_tab_id = Some(tab_id);
        }
        self.snapshot_current_workspace();

        self.pending_document_loads.push(DocumentLoadJob {
            tab_id,
            path,
            gen,
        });
    }

    /// Prepare and set state for asynchronously opening a directory.
    pub fn start_loading_directory(&mut self, dir: PathBuf) {
        self.settings.add_recent_folder(dir.clone());
        self.persist_settings();

        self.opened_folder = Some(dir);
        self.is_loading_files = true;
        self.sidebar_tab = SidebarTab::Files;
        self.show_sidebar = true;
        self.snapshot_current_workspace();
    }

    /// Complete loading the file tree after asynchronous background scanning.
    pub fn finish_loading_directory(&mut self, dir: &Path, tree: Vec<FileTreeEntry>) {
        if self.opened_folder.as_deref() == Some(dir) {
            self.file_tree = tree;
            self.is_loading_files = false;
            self.snapshot_current_workspace();
        }
    }

    /// Explicitly set the file tree loading state.
    pub const fn set_loading_files(&mut self, loading: bool) {
        self.is_loading_files = loading;
    }

    /// Open a directory into the file tree sidebar synchronously.
    pub fn open_directory(&mut self, dir: PathBuf) {
        self.settings.add_recent_folder(dir.clone());
        self.persist_settings();

        if let Ok(tree) = scan_file_tree(&dir, self.file_filter_mode) {
            self.file_tree = tree;
        }
        self.opened_folder = Some(dir.clone());
        self.sidebar_tab = SidebarTab::Files;
        self.show_sidebar = true;
        self.is_loading_files = false;
        self.snapshot_current_workspace();
        crate::state::kick_fts_rebuild_forced(dir, self.file_filter_mode);
    }

    /// Refresh file tree based on current opened folder and file filter mode.
    pub fn refresh_file_tree(&mut self) {
        if let Some(ref dir) = self.opened_folder.clone() {
            if let Ok(tree) = scan_file_tree(dir, self.file_filter_mode) {
                self.file_tree = tree;
            }
        } else {
            let parent_opt = self
                .active_tab()
                .and_then(|t| t.path.as_ref())
                .and_then(|p| p.parent().map(std::path::Path::to_path_buf));

            if let Some(parent) = parent_opt {
                if let Ok(tree) = scan_file_tree(&parent, self.file_filter_mode) {
                    self.file_tree = tree;
                    self.opened_folder = Some(parent);
                }
            }
        }
    }

    /// Set file visibility filter mode and refresh the file tree.
    pub fn set_file_filter_mode(&mut self, mode: FileFilterMode) {
        self.file_filter_mode = mode;
        self.settings.file_filter_mode = mode;
        self.persist_settings();
        self.refresh_file_tree();
        if let Some(dir) = self.opened_folder.clone() {
            crate::state::kick_fts_rebuild_forced(dir, mode);
        }
    }

    /// Cycle to the next file visibility filter mode.
    #[allow(dead_code)]
    pub fn cycle_file_filter_mode(&mut self) {
        self.set_file_filter_mode(self.file_filter_mode.next());
    }

    /// Take a pending directory scan requested during file open.
    pub fn take_pending_tree_scan(&mut self) -> Option<PathBuf> {
        self.pending_tree_scan.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::markdown::parse_document;
    use std::env;

    fn temp_open_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "fast_md_open_{name}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_apply_tab_parsed_stale_gen_ignored() {
        let mut store = AppStore::default();
        store.tabs[0].parse_gen = 1;
        let parsed = parse_document("# Hi", DocumentFormat::Markdown);
        assert!(!store.apply_tab_parsed(store.tabs[0].id, 99, parsed.clone()));
        assert_ne!(store.tabs[0].parsed.html_content, parsed.html_content);

        assert!(store.apply_tab_parsed(store.tabs[0].id, 1, parsed.clone()));
        assert_eq!(store.tabs[0].parsed.html_content, parsed.html_content);
        assert_eq!(store.tabs[0].parse_status, ParseStatus::Ready);
    }

    #[test]
    fn test_preview_replaces_preview_tab() {
        let dir = temp_open_dir("preview_replace");
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        let _ = std::fs::write(&a, "# A\n");
        let _ = std::fs::write(&b, "# B\n");

        let mut store = AppStore::default();
        store.drop_welcome_tab();
        let initial_len = store.tabs.len();

        store.open_file_from_path(a.clone(), OpenKind::Preview);
        assert_eq!(store.tabs.len(), initial_len + 1);
        assert_eq!(store.preview_tab_id, Some(store.active_tab_id));
        assert_eq!(store.active_tab().and_then(|t| t.path.as_ref()), Some(&a));

        store.open_file_from_path(b.clone(), OpenKind::Preview);
        assert_eq!(store.tabs.len(), initial_len + 1);
        assert_eq!(store.active_tab().and_then(|t| t.path.as_ref()), Some(&b));
        assert_eq!(store.preview_tab_id, Some(store.active_tab_id));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_preview_click_existing_pinned_tab() {
        let dir = temp_open_dir("pinned_activate");
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        let _ = std::fs::write(&a, "# A\n");
        let _ = std::fs::write(&b, "# B\n");

        let mut store = AppStore::default();
        store.drop_welcome_tab();
        let initial_len = store.tabs.len();

        store.open_file_from_path(a.clone(), OpenKind::Pinned);
        let pinned_id = store.active_tab_id;
        store.open_file_from_path(b.clone(), OpenKind::Preview);
        let preview_id = store.preview_tab_id;

        store.open_file_from_path(a.clone(), OpenKind::Preview);
        assert_eq!(store.active_tab_id, pinned_id);
        assert_eq!(store.preview_tab_id, preview_id);
        assert_eq!(store.tabs.len(), initial_len + 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pinned_open_keeps_other_preview() {
        let dir = temp_open_dir("pinned_keep_preview");
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        let _ = std::fs::write(&a, "# A\n");
        let _ = std::fs::write(&b, "# B\n");

        let mut store = AppStore::default();
        store.drop_welcome_tab();

        store.open_file_from_path(a.clone(), OpenKind::Preview);
        let preview_id = store.preview_tab_id;

        store.open_file_from_path(b.clone(), OpenKind::Pinned);
        assert_eq!(store.preview_tab_id, preview_id);
        assert_eq!(store.tabs.len(), 2);
        assert!(store.preview_tab_id.is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pinned_restore_loop_keeps_all_tabs() {
        let dir = temp_open_dir("restore_loop");
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        let c = dir.join("c.md");
        let _ = std::fs::write(&a, "# A\n");
        let _ = std::fs::write(&b, "# B\n");
        let _ = std::fs::write(&c, "# C\n");

        let mut store = AppStore::default();
        store.drop_welcome_tab();

        for path in [&a, &b, &c] {
            store.open_file_from_path(path.clone(), OpenKind::Pinned);
        }

        assert_eq!(store.tabs.len(), 3);
        assert!(store.preview_tab_id.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pin_tab_clears_preview_tab_id() {
        let dir = temp_open_dir("pin_tab");
        let a = dir.join("a.md");
        let _ = std::fs::write(&a, "# A\n");

        let mut store = AppStore::default();
        store.drop_welcome_tab();
        store.open_file_from_path(a, OpenKind::Preview);
        let tab_id = store.active_tab_id;
        assert_eq!(store.preview_tab_id, Some(tab_id));

        store.pin_tab(tab_id);
        assert!(store.preview_tab_id.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
