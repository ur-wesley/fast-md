use crate::types::{FileFilterMode, FileTreeEntry};
use eyre::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Read the entire contents of a file safely into a String.
pub fn read_document_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file at {}", path.display()))
}

/// Save content to a file safely.
pub fn save_document_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write file to {}", path.display()))
}

/// Recursively build a file tree filtered according to the specified `FileFilterMode`.
pub fn scan_file_tree(dir: &Path, filter_mode: FileFilterMode) -> Result<Vec<FileTreeEntry>> {
    let mut entries = Vec::new();

    if !dir.is_dir() {
        return Ok(entries);
    }

    let read_dir = fs::read_dir(dir).with_context(|| format!("Failed to scan directory {}", dir.display()))?;

    let mut dir_entries: Vec<_> = read_dir.filter_map(Result::ok).collect();
    dir_entries.sort_by_key(|e| (e.file_type().is_ok_and(|ft| !ft.is_dir()), e.file_name()));

    for entry in dir_entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dotfiles/dotfolders (e.g. .git, .vscode)
        if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
            continue;
        }

        let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());

        if is_dir {
            let children = scan_file_tree(&path, filter_mode).unwrap_or_default();
            // Only keep directory if it contains relevant files or subdirectories
            if !children.is_empty() {
                entries.push(FileTreeEntry {
                    name: file_name,
                    path,
                    is_dir: true,
                    children,
                });
            }
        } else if filter_mode.matches_path(&path) {
            entries.push(FileTreeEntry {
                name: file_name,
                path,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }

    Ok(entries)
}

/// Backward-compatible alias for scanning markdown and config tree.
#[allow(dead_code)]
pub fn scan_markdown_tree(dir: &Path) -> Result<Vec<FileTreeEntry>> {
    scan_file_tree(dir, FileFilterMode::MarkdownAndConfig)
}

/// Prompt native asynchronous file picker dialog for opening documents or configs.
pub async fn pick_file_async() -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("Markdown & MDX", &["md", "mdx", "markdown", "txt"])
        .add_filter("Config Files (JSON, TOML, YAML)", &["json", "jsonc", "toml", "yaml", "yml", "ini", "ron", "xml"])
        .add_filter("All Supported Files", &["md", "mdx", "markdown", "txt", "json", "toml", "yaml", "yml", "ini", "ron", "xml", "rs", "js", "ts", "html", "css"])
        .add_filter("All Files", &["*"])
        .set_title("Open Document / Config File")
        .pick_file()
        .await;

    handle.map(|h| h.path().to_path_buf())
}

/// Prompt native asynchronous folder picker dialog for opening a documentation directory.
pub async fn pick_folder_async() -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open Documentation Directory")
        .pick_folder()
        .await;

    handle.map(|h| h.path().to_path_buf())
}

/// Prompt native asynchronous save dialog for exporting HTML.
pub async fn pick_export_html_async(default_name: &str) -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("HTML Document", &["html", "htm"])
        .set_file_name(default_name)
        .set_title("Export as HTML")
        .save_file()
        .await;

    handle.map(|h| h.path().to_path_buf())
}

/// Prompt native asynchronous save dialog for saving documents or config files.
pub async fn pick_save_file_async(default_name: &str) -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("Markdown Document", &["md", "markdown"])
        .add_filter("MDX Document", &["mdx"])
        .add_filter("JSON Document", &["json"])
        .add_filter("TOML Document", &["toml"])
        .add_filter("YAML Document", &["yaml", "yml"])
        .add_filter("Plain Text Document", &["txt"])
        .add_filter("All Files", &["*"])
        .set_file_name(default_name)
        .set_title("Save Document")
        .save_file()
        .await;

    handle.map(|h| h.path().to_path_buf())
}
