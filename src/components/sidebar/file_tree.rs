use crate::components::context_menu::FileTreeContextMenu;
use crate::state::AppStore;
use crate::types::FileTreeEntry;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdChevronDown, LdChevronRight, LdFileCode2, LdFileText, LdFolder, LdFolderOpen,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub const FILE_TREE_ROW_HEIGHT: u32 = 28;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeRow {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

pub fn filter_file_tree(entries: &[FileTreeEntry], query: &str) -> Vec<FileTreeEntry> {
    if query.is_empty() {
        return entries.to_vec();
    }
    let query_lower = query.to_lowercase();
    entries
        .iter()
        .filter_map(|entry| filter_entry(entry, &query_lower))
        .collect()
}

fn filter_entry(entry: &FileTreeEntry, query_lower: &str) -> Option<FileTreeEntry> {
    if entry.is_dir {
        let children: Vec<FileTreeEntry> = entry
            .children
            .iter()
            .filter_map(|child| filter_entry(child, query_lower))
            .collect();
        if entry.name.to_lowercase().contains(query_lower) || !children.is_empty() {
            return Some(FileTreeEntry {
                name: entry.name.clone(),
                path: entry.path.clone(),
                is_dir: true,
                children: Arc::new(children),
            });
        }
        return None;
    }
    entry
        .name
        .to_lowercase()
        .contains(query_lower)
        .then(|| entry.clone())
}

pub fn flatten_visible(
    entries: &[FileTreeEntry],
    expanded_dirs: &HashSet<PathBuf>,
    searching: bool,
) -> Vec<FileTreeRow> {
    let mut rows = Vec::new();
    flatten_walk(entries, expanded_dirs, searching, 0, &mut rows);
    rows
}

fn flatten_walk(
    entries: &[FileTreeEntry],
    expanded_dirs: &HashSet<PathBuf>,
    searching: bool,
    depth: usize,
    rows: &mut Vec<FileTreeRow>,
) {
    for entry in entries {
        let expanded = entry.is_dir && (searching || expanded_dirs.contains(&entry.path));
        rows.push(FileTreeRow {
            path: entry.path.clone(),
            name: entry.name.clone(),
            depth,
            is_dir: entry.is_dir,
            expanded,
        });
        if expanded {
            flatten_walk(&entry.children, expanded_dirs, searching, depth + 1, rows);
        }
    }
}

#[derive(Props, Clone)]
pub struct FileTreeRowItemProps {
    pub row: FileTreeRow,
    pub active_path: Option<PathBuf>,
    pub translations: &'static crate::i18n::Translations,
    pub store: Signal<AppStore>,
    pub on_toggle_dir: EventHandler<PathBuf>,
    pub on_select: EventHandler<PathBuf>,
}

impl PartialEq for FileTreeRowItemProps {
    fn eq(&self, other: &Self) -> bool {
        let self_active = self.active_path.as_ref() == Some(&self.row.path);
        let other_active = other.active_path.as_ref() == Some(&other.row.path);
        self.row == other.row
            && self_active == other_active
            && std::ptr::eq(self.translations, other.translations)
    }
}

#[component]
pub(super) fn FileTreeRowItem(props: FileTreeRowItemProps) -> Element {
    let row = &props.row;
    let indent_px = row.depth * 14;

    if row.is_dir {
        let dir_path = row.path.clone();
        rsx! {
            FileTreeContextMenu {
                t: props.translations,
                path: row.path.clone(),
                is_dir: true,
                store: props.store,
                on_open: props.on_select,
                div {
                    class: "dir-row flex items-center gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150",
                    style: "padding-left: {indent_px}px;",
                    onclick: move |_| props.on_toggle_dir.call(dir_path.clone()),
                    span {
                        class: "dir-arrow text-[var(--text-muted)] w-3 shrink-0 flex items-center justify-center",
                        if row.expanded {
                            Icon { width: 10, height: 10, icon: LdChevronDown }
                        } else {
                            Icon { width: 10, height: 10, icon: LdChevronRight }
                        }
                    }
                    span {
                        class: "dir-icon text-[var(--accent)] shrink-0 flex items-center",
                        if row.expanded {
                            Icon { width: 13, height: 13, icon: LdFolderOpen }
                        } else {
                            Icon { width: 13, height: 13, icon: LdFolder }
                        }
                    }
                    span { class: "dir-name truncate font-medium", "{row.name}" }
                }
            }
        }
    } else {
        let path_clone = row.path.clone();
        let is_active = props.active_path.as_ref() == Some(&row.path);
        let ext = row
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let format_label = match ext.as_str() {
            "json" | "jsonc" => Some("JSON"),
            "toml" => Some("TOML"),
            "yaml" | "yml" => Some("YAML"),
            "mdx" => Some("MDX"),
            "ini" => Some("INI"),
            "ron" => Some("RON"),
            "xml" => Some("XML"),
            _ => None,
        };
        let is_code = matches!(
            ext.as_str(),
            "mdx" | "rs" | "js" | "ts" | "json" | "jsonc" | "toml" | "yaml" | "yml" | "ini" | "ron" | "xml"
        );
        let file_pad = indent_px + 20;

        rsx! {
            FileTreeContextMenu {
                t: props.translations,
                path: row.path.clone(),
                is_dir: false,
                store: props.store,
                on_open: props.on_select,
                div {
                    class: if is_active {
                        "file-tree-file flex items-center justify-between gap-1.5 py-1 px-2 rounded text-xs cursor-pointer bg-[var(--bg-hover)] text-[var(--accent)] font-semibold transition-colors duration-150 border-l-2 border-[var(--accent)]"
                    } else {
                        "file-tree-file flex items-center justify-between gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150"
                    },
                    style: "padding-left: {file_pad}px;",
                    onclick: move |_| props.on_select.call(path_clone.clone()),
                    div {
                        class: "flex items-center gap-1.5 min-w-0 flex-1",
                        span {
                            class: if is_active {
                                "file-icon shrink-0 flex items-center text-[var(--accent)]"
                            } else {
                                "file-icon shrink-0 flex items-center text-[var(--text-muted)]"
                            },
                            if is_code {
                                Icon { width: 13, height: 13, icon: LdFileCode2 }
                            } else {
                                Icon { width: 13, height: 13, icon: LdFileText }
                            }
                        }
                        span { class: "file-name truncate", "{row.name}" }
                    }
                    if let Some(lbl) = format_label {
                        span {
                            class: "file-format-badge text-[9px] px-1 py-0.2 rounded bg-[var(--bg-app)] border border-[var(--border-color)] text-[var(--text-muted)] font-mono opacity-80 shrink-0",
                            "{lbl}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn file(name: &str) -> FileTreeEntry {
        FileTreeEntry {
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir: false,
            children: Arc::new(Vec::new()),
        }
    }

    fn dir(name: &str, children: Vec<FileTreeEntry>) -> FileTreeEntry {
        FileTreeEntry {
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir: true,
            children: Arc::new(children),
        }
    }

    fn names(entries: &[FileTreeEntry]) -> Vec<&str> {
        entries.iter().map(|e| e.name.as_str()).collect()
    }

    fn count_nodes(entries: &[FileTreeEntry]) -> usize {
        entries
            .iter()
            .map(|entry| 1 + count_nodes(&entry.children))
            .sum()
    }

    fn instantiate_current(entries: &[FileTreeEntry], query_lower: &str) -> usize {
        fn rec(entry: &FileTreeEntry, query_lower: &str, n: &mut usize) {
            *n += 1;
            let matches = entry.name.to_lowercase().contains(query_lower)
                || entry
                    .children
                    .iter()
                    .any(|child| name_or_descendant_matches(child, query_lower));
            if !matches {
                return;
            }
            if entry.is_dir {
                for child in entry.children.iter() {
                    rec(child, query_lower, n);
                }
            }
        }
        let mut n = 0;
        for entry in entries {
            rec(entry, query_lower, &mut n);
        }
        n
    }

    fn name_or_descendant_matches(entry: &FileTreeEntry, query_lower: &str) -> bool {
        entry.name.to_lowercase().contains(query_lower)
            || entry
                .children
                .iter()
                .any(|child| name_or_descendant_matches(child, query_lower))
    }

    fn row_names(rows: &[FileTreeRow]) -> Vec<&str> {
        rows.iter().map(|r| r.name.as_str()).collect()
    }

    #[test]
    fn filter_keeps_ancestors_drops_unrelated_siblings() {
        let tree = vec![dir(
            "notes",
            vec![
                file("alpha.md"),
                file("beta.md"),
                dir("nested", vec![file("alpha-two.md")]),
            ],
        )];

        let filtered = filter_file_tree(&tree, "alpha");
        assert_eq!(names(&filtered), ["notes"]);
        assert_eq!(names(&filtered[0].children), ["alpha.md", "nested"]);
        assert_eq!(names(&filtered[0].children[1].children), ["alpha-two.md"]);
        assert_eq!(count_nodes(&filtered), 4);
    }

    #[test]
    fn filter_keeps_matching_dir_even_without_matching_children() {
        let tree = vec![dir("alpha", vec![file("beta.md")])];
        let filtered = filter_file_tree(&tree, "alpha");
        assert_eq!(names(&filtered), ["alpha"]);
        assert!(filtered[0].children.is_empty());
    }

    #[test]
    fn filter_is_case_insensitive() {
        let tree = vec![file("ReadME.md")];
        assert_eq!(names(&filter_file_tree(&tree, "readme")), ["ReadME.md"]);
    }

    #[test]
    fn empty_query_keeps_full_tree() {
        let tree = vec![dir("notes", vec![file("a.md"), file("b.md")])];
        let filtered = filter_file_tree(&tree, "");
        assert_eq!(count_nodes(&filtered), 3);
    }

    #[test]
    fn prune_instantiates_far_fewer_nodes_than_render_time_filter() {
        let tree: Vec<FileTreeEntry> = (0..80)
            .map(|d| {
                dir(
                    &format!("proj{d}"),
                    (0..60)
                        .map(|f| file(&format!("note{f:02}.md")))
                        .collect(),
                )
            })
            .collect();

        let query = "note00";
        let current = instantiate_current(&tree, query);
        let pruned = count_nodes(&filter_file_tree(&tree, query));

        assert_eq!(pruned, 160);
        assert!(
            current > pruned * 10,
            "current instantiate={current} pruned={pruned}"
        );
    }

    #[test]
    fn flatten_collapsed_dir_emits_one_row() {
        let tree = vec![dir("notes", vec![file("a.md"), file("b.md")])];
        let rows = flatten_visible(&tree, &HashSet::new(), false);
        assert_eq!(row_names(&rows), ["notes"]);
        assert!(!rows[0].expanded);
    }

    #[test]
    fn flatten_expanded_dir_emits_children() {
        let tree = vec![dir("notes", vec![file("a.md"), file("b.md")])];
        let mut expanded = HashSet::new();
        expanded.insert(PathBuf::from("notes"));
        let rows = flatten_visible(&tree, &expanded, false);
        assert_eq!(row_names(&rows), ["notes", "a.md", "b.md"]);
        assert_eq!(rows[1].depth, 1);
    }

    #[test]
    fn flatten_search_expands_pruned_tree() {
        let tree = vec![dir(
            "notes",
            vec![file("alpha.md"), file("beta.md")],
        )];
        let filtered = filter_file_tree(&tree, "alpha");
        let rows = flatten_visible(&filtered, &HashSet::new(), true);
        assert_eq!(row_names(&rows), ["notes", "alpha.md"]);
        assert!(rows[0].expanded);
    }
}
