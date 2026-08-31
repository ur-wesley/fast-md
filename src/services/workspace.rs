use crate::types::{WorkspaceSnapshot, WorkspacesFile};
use eyre::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::settings::get_settings_dir;

const WORKSPACES_FILE_NAME: &str = "workspaces.json";

#[must_use]
pub fn get_workspaces_file_path() -> PathBuf {
    get_settings_dir().join(WORKSPACES_FILE_NAME)
}

#[must_use]
pub fn load_workspaces() -> WorkspacesFile {
    let path = get_workspaces_file_path();
    load_workspaces_from_path(&path)
}

#[must_use]
pub fn load_workspaces_from_path(path: &Path) -> WorkspacesFile {
    if !path.exists() {
        return WorkspacesFile::default();
    }

    fs::read_to_string(path).map_or_else(
        |_| WorkspacesFile::default(),
        |content| serde_json::from_str(&content).unwrap_or_default(),
    )
}

pub fn save_workspaces(workspaces: &WorkspacesFile) -> Result<()> {
    let path = get_workspaces_file_path();
    save_workspaces_to_path(&path, workspaces)
}

pub fn save_workspaces_to_path(path: &Path, workspaces: &WorkspacesFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {}", parent.display()))?;
        }
    }

    let json_content = serde_json::to_string_pretty(workspaces)
        .with_context(|| "Failed to serialize WorkspacesFile to JSON")?;

    fs::write(path, json_content)
        .with_context(|| format!("Failed to write workspaces to {}", path.display()))?;

    Ok(())
}

/// Returns true when two paths refer to the same workspace folder.
#[must_use]
pub fn workspace_keys_equal(a: &Path, b: &Path) -> bool {
    match (canonical_workspace_key(a), canonical_workspace_key(b)) {
        (Some(ka), Some(kb)) => ka == kb,
        _ => a == b,
    }
}

/// Canonical folder path used as workspace identity (dedupes `.` vs absolute paths).
#[must_use]
pub fn canonical_workspace_key(path: &Path) -> Option<PathBuf> {
    let folder = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    let canonical = fs::canonicalize(&folder).ok()?;
    Some(strip_unc_prefix(canonical))
}

fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    let rendered = path.to_string_lossy();
    if let Some(stripped) = rendered.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_workspaces_save_and_load_roundtrip() {
        let temp_dir = env::temp_dir().join(format!(
            "fast_md_ws_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("workspaces.json");

        let mut file = WorkspacesFile::default();
        let readme = temp_dir.join("README.md");
        let _ = std::fs::write(&readme, "# Test\n");
        file.upsert(WorkspaceSnapshot {
            folder: temp_dir.clone(),
            tabs: vec![readme],
            active: None,
            expanded: vec![],
        });

        assert!(save_workspaces_to_path(&path, &file).is_ok());
        let loaded = load_workspaces_from_path(&path);
        assert_eq!(loaded.workspaces.len(), 1);
        assert!(loaded.last.is_some());
        assert!(workspace_keys_equal(
            loaded.last.as_ref().unwrap(),
            &temp_dir,
        ));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_upsert_same_path_moves_to_front() {
        let dir_a = env::temp_dir().join(format!(
            "fast_md_ws_upsert_a_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let dir_b = env::temp_dir().join(format!(
            "fast_md_ws_upsert_b_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir_a);
        let _ = std::fs::create_dir_all(&dir_b);

        let mut file = WorkspacesFile::default();
        file.upsert(WorkspaceSnapshot {
            folder: dir_a.clone(),
            tabs: vec![],
            active: None,
            expanded: vec![],
        });
        file.upsert(WorkspaceSnapshot {
            folder: dir_b.clone(),
            tabs: vec![],
            active: None,
            expanded: vec![],
        });
        file.upsert(WorkspaceSnapshot {
            folder: dir_a.clone(),
            tabs: vec![dir_a.join("x.md")],
            active: None,
            expanded: vec![],
        });

        assert_eq!(file.workspaces.len(), 2);
        assert_eq!(file.workspaces[0].tabs.len(), 1);
        assert!(file.last.is_some());

        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
    }
}
