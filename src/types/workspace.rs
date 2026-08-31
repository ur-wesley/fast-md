use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persisted workspace state for a single folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub folder: PathBuf,
    #[serde(default)]
    pub tabs: Vec<PathBuf>,
    #[serde(default)]
    pub active: Option<PathBuf>,
    #[serde(default)]
    pub expanded: Vec<PathBuf>,
}

/// On-disk workspace store (`workspaces.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkspacesFile {
    #[serde(default)]
    pub last: Option<PathBuf>,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceSnapshot>,
}

const MAX_WORKSPACES: usize = 20;

impl WorkspacesFile {
    pub fn find_index_by_folder(&self, folder: &PathBuf) -> Option<usize> {
        self.workspaces
            .iter()
            .position(|w| crate::services::workspace::workspace_keys_equal(&w.folder, folder))
    }

    pub fn find_by_folder(&self, folder: &PathBuf) -> Option<&WorkspaceSnapshot> {
        let idx = self.find_index_by_folder(folder)?;
        self.workspaces.get(idx)
    }

    pub fn upsert(&mut self, snapshot: WorkspaceSnapshot) {
        let folder_key = crate::services::workspace::canonical_workspace_key(&snapshot.folder)
            .unwrap_or_else(|| snapshot.folder.clone());

        self.workspaces.retain(|w| {
            !crate::services::workspace::workspace_keys_equal(&w.folder, &snapshot.folder)
        });

        let mut entry = snapshot;
        entry.folder = folder_key.clone();
        self.workspaces.insert(0, entry);

        if self.workspaces.len() > MAX_WORKSPACES {
            self.workspaces.truncate(MAX_WORKSPACES);
        }

        self.last = Some(folder_key);
    }
}
