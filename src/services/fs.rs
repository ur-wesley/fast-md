use crate::types::FileTreeEntry;
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

/// Recursively build a file tree filtered to Markdown, MDX, and text documentation files.
pub fn scan_markdown_tree(dir: &Path) -> Result<Vec<FileTreeEntry>> {
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
            let children = scan_markdown_tree(&path).unwrap_or_default();
            // Only keep directory if it contains relevant files or subdirectories
            if !children.is_empty() {
                entries.push(FileTreeEntry {
                    name: file_name,
                    path,
                    is_dir: true,
                    children,
                });
            }
        } else {
            let is_doc = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "md" | "mdx" | "markdown" | "mdown" | "txt" | "json" | "yaml" | "yml" | "toml" | "rst"
                    )
                });

            if is_doc {
                entries.push(FileTreeEntry {
                    name: file_name,
                    path,
                    is_dir: false,
                    children: Vec::new(),
                });
            }
        }
    }

    Ok(entries)
}

/// Prompt native asynchronous file picker dialog for opening Markdown or MDX documents.
pub async fn pick_file_async() -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("Markdown / MDX", &["md", "mdx", "markdown", "txt"])
        .add_filter("All Files", &["*"])
        .set_title("Open Markdown / MDX Document")
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
