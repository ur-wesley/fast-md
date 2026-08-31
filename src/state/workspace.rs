use super::{find_primary_doc_in_dir, AppStore};
use crate::services::workspace::{canonical_workspace_key, save_workspaces};
use crate::types::WorkspaceSnapshot;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

impl AppStore {
    pub fn snapshot_current_workspace(&mut self) {
        let Some(folder) = self.opened_folder.clone() else {
            return;
        };

        let tabs: Vec<PathBuf> = self
            .tabs
            .iter()
            .filter_map(|t| t.path.clone())
            .filter(|p| p.exists())
            .collect();

        let active = self
            .active_tab()
            .and_then(|t| t.path.clone())
            .filter(|p| p.exists());

        let expanded: Vec<PathBuf> = self
            .expanded_dirs
            .iter()
            .filter(|p| p.exists())
            .cloned()
            .collect();

        let folder_key = canonical_workspace_key(&folder).unwrap_or(folder);

        self.workspaces.upsert(WorkspaceSnapshot {
            folder: folder_key,
            tabs,
            active,
            expanded,
        });
        self.persist_workspaces();
    }

    pub fn persist_workspaces(&self) {
        let workspaces = self.workspaces.clone();
        std::thread::spawn(move || {
            let _ = save_workspaces(&workspaces);
        });
    }

    pub fn restore_workspace_snapshot(&mut self, snapshot: &WorkspaceSnapshot) -> bool {
        if !snapshot.folder.is_dir() {
            return false;
        }

        self.tabs.clear();
        self.next_tab_id = 1;
        self.active_tab_id = 0;

        for path in &snapshot.tabs {
            if path.is_file() {
                self.open_file_from_path(path.clone());
            }
        }

        if self.tabs.is_empty() {
            if let Some(primary) = find_primary_doc_in_dir(&snapshot.folder) {
                self.open_file_from_path(primary);
            }
        }

        if let Some(active) = &snapshot.active {
            if let Some(tab) = self.tabs.iter().find(|t| t.path.as_ref() == Some(active)) {
                self.active_tab_id = tab.id;
            }
        } else if let Some(first) = self.tabs.first() {
            self.active_tab_id = first.id;
        }

        self.expanded_dirs = snapshot
            .expanded
            .iter()
            .filter(|p| p.is_dir())
            .cloned()
            .collect();

        self.open_directory(snapshot.folder.clone());
        true
    }

    pub fn switch_workspace(&mut self, folder: PathBuf) {
        self.snapshot_current_workspace();

        let key = canonical_workspace_key(&folder).unwrap_or_else(|| folder.clone());

        if let Some(snapshot) = self.workspaces.find_by_folder(&key).cloned() {
            let _ = self.restore_workspace_snapshot(&snapshot);
        } else {
            self.tabs.clear();
            self.next_tab_id = 1;
            self.active_tab_id = 0;
            self.expanded_dirs.clear();
            self.open_directory(folder.clone());
            if let Some(primary) = find_primary_doc_in_dir(&folder) {
                self.open_file_from_path(primary);
            }
        }

        self.workspaces.last = Some(key);
        self.snapshot_current_workspace();
    }

    pub fn expand_dir_ancestors(&mut self, file_path: &Path) {
        let mut changed = false;
        let mut current = file_path.parent();
        while let Some(dir) = current {
            if self.expanded_dirs.insert(dir.to_path_buf()) {
                changed = true;
            }
            current = dir.parent();
        }
        if changed {
            self.snapshot_current_workspace();
        }
    }

    pub fn toggle_expanded_dir(&mut self, path: PathBuf) {
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }
        self.snapshot_current_workspace();
    }

    pub fn set_expanded_dirs(&mut self, dirs: HashSet<PathBuf>) {
        self.expanded_dirs = dirs;
        self.snapshot_current_workspace();
    }

    pub fn drop_welcome_tab(&mut self) {
        if self.tabs.len() == 1 && self.tabs[0].path.is_none() {
            self.tabs.clear();
            self.active_tab_id = 0;
        }
    }

    pub fn boot_from_cli_path(&mut self, path: &Path) {
        self.drop_welcome_tab();

        let workspace_root = if path.is_dir() {
            path.to_path_buf()
        } else if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            return;
        };

        let key = canonical_workspace_key(&workspace_root).unwrap_or(workspace_root.clone());

        if let Some(snapshot) = self.workspaces.find_by_folder(&key).cloned() {
            let _ = self.restore_workspace_snapshot(&snapshot);
            self.workspaces.last = Some(key);
            return;
        }

        if path.is_dir() {
            self.open_directory(path.to_path_buf());
            if let Some(primary) = find_primary_doc_in_dir(path) {
                self.open_file_from_path(primary);
            }
        } else {
            if path.parent().is_some() {
                self.open_directory(workspace_root);
            }
            self.open_file_from_path(path.to_path_buf());
        }
        self.workspaces.last = Some(key);
        self.snapshot_current_workspace();
    }

    pub fn boot_restore_last_workspace(&mut self) -> bool {
        self.drop_welcome_tab();

        let Some(last) = self.workspaces.last.clone() else {
            return false;
        };

        let Some(snapshot) = self.workspaces.find_by_folder(&last).cloned() else {
            return false;
        };

        if !self.restore_workspace_snapshot(&snapshot) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::workspace::canonical_workspace_key;
    use crate::types::WorkspacesFile;
    use std::env;

    fn temp_workspace_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "fast_md_ws_boot_{name}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_canonical_key_dedupes_dot_and_absolute() {
        let dir = temp_workspace_dir("canon");
        let dot = env::current_dir().unwrap();
        let _ = env::set_current_dir(&dir);

        let from_dot = canonical_workspace_key(Path::new(".")).unwrap();
        let from_abs = canonical_workspace_key(&dir).unwrap();
        assert_eq!(from_dot, from_abs);

        let _ = env::set_current_dir(dot);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_boot_restore_last_workspace() {
        let dir = temp_workspace_dir("restore");
        let readme = dir.join("README.md");
        let _ = std::fs::write(&readme, "# Test\n");

        let mut store = AppStore::default();
        store.workspaces = WorkspacesFile {
            last: Some(dir.clone()),
            workspaces: vec![WorkspaceSnapshot {
                folder: dir.clone(),
                tabs: vec![readme.clone()],
                active: Some(readme),
                expanded: vec![],
            }],
        };

        assert!(store.boot_restore_last_workspace());
        assert_eq!(store.opened_folder.as_ref(), Some(&dir));
        assert!(!store.tabs.is_empty());
        assert!(store.tabs.iter().all(|t| t.path.is_some()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_boot_cli_fresh_workspace_does_not_restore_other_tabs() {
        let dir_a = temp_workspace_dir("a");
        let dir_b = temp_workspace_dir("b");
        let readme_a = dir_a.join("README.md");
        let readme_b = dir_b.join("README.md");
        let extra_a = dir_a.join("extra.md");
        let _ = std::fs::write(&readme_a, "# A\n");
        let _ = std::fs::write(&readme_b, "# B\n");
        let _ = std::fs::write(&extra_a, "# Extra\n");

        let mut store = AppStore::default();
        store.workspaces = WorkspacesFile {
            last: Some(dir_a.clone()),
            workspaces: vec![WorkspaceSnapshot {
                folder: dir_a.clone(),
                tabs: vec![readme_a.clone(), extra_a],
                active: Some(readme_a),
                expanded: vec![],
            }],
        };

        store.boot_from_cli_path(&readme_b);
        assert_eq!(store.opened_folder.as_ref(), Some(&dir_b));
        assert_eq!(store.tabs.len(), 1);
        assert_eq!(store.tabs[0].path.as_ref(), Some(&readme_b));

        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
    }

    #[test]
    fn test_boot_cli_existing_workspace_restores_tabs() {
        let dir = temp_workspace_dir("existing");
        let readme = dir.join("README.md");
        let notes = dir.join("notes.md");
        let _ = std::fs::write(&readme, "# Readme\n");
        let _ = std::fs::write(&notes, "# Notes\n");

        let mut store = AppStore::default();
        store.workspaces = WorkspacesFile {
            last: Some(dir.clone()),
            workspaces: vec![WorkspaceSnapshot {
                folder: dir.clone(),
                tabs: vec![readme.clone(), notes.clone()],
                active: Some(notes.clone()),
                expanded: vec![],
            }],
        };

        store.boot_from_cli_path(&dir);
        assert_eq!(store.tabs.len(), 2);
        assert_eq!(
            store.active_tab().and_then(|t| t.path.as_ref()),
            Some(&notes)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_restore_skips_missing_files() {
        let dir = temp_workspace_dir("missing");
        let readme = dir.join("README.md");
        let missing = dir.join("gone.md");
        let _ = std::fs::write(&readme, "# Readme\n");

        let mut store = AppStore::default();
        let snapshot = WorkspaceSnapshot {
            folder: dir.clone(),
            tabs: vec![readme.clone(), missing],
            active: Some(readme.clone()),
            expanded: vec![],
        };

        assert!(store.restore_workspace_snapshot(&snapshot));
        assert_eq!(store.tabs.len(), 1);
        assert_eq!(store.tabs[0].path.as_ref(), Some(&readme));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
