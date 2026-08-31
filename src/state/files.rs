use super::{AppStore, WELCOME_DOC};
use crate::services::fs::{read_document_file, scan_file_tree};
use crate::services::markdown::parse_document;
use crate::types::{DocumentFormat, FileFilterMode, FileTreeEntry, SidebarTab, TabItem};
use std::path::{Path, PathBuf};

impl AppStore {
    /// Open a file from a `PathBuf` into a tab (or activate if already open).
    /// Returns parent directory when an async tree scan should be started.
    pub fn open_file_from_path(&mut self, path: PathBuf) -> Option<PathBuf> {
        // Track in recent files
        self.settings.add_recent_file(path.clone());
        self.persist_settings();

        // If file is already open, activate that tab
        if let Some(existing) = self.tabs.iter().find(|t| t.path.as_ref() == Some(&path)) {
            self.active_tab_id = existing.id;
            self.snapshot_current_workspace();
            return None;
        }

        let format = DocumentFormat::from_path(Some(&path));
        let content = read_document_file(&path).unwrap_or_else(|_| WELCOME_DOC.to_string());
        let parsed = parse_document(&content, format);
        let title = path.file_name().map_or_else(|| "Document".to_string(), |n| n.to_string_lossy().to_string());

        let tab_id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);

        let mut scan_parent = None;
        if self.file_tree.is_empty() {
            if let Some(parent) = path.parent() {
                let parent = parent.to_path_buf();
                self.start_loading_directory(parent.clone());
                self.pending_tree_scan = Some(parent.clone());
                scan_parent = Some(parent);
            }
        }

        self.tabs.push(TabItem {
            id: tab_id,
            path: Some(path),
            title,
            content,
            parsed,
            is_dirty: false,
            html_revision: 0,
        });

        self.active_tab_id = tab_id;
        self.snapshot_current_workspace();
        scan_parent
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
            let parent_opt = self.active_tab()
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
