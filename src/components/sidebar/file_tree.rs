use crate::components::context_menu::FileTreeContextMenu;
use crate::state::AppStore;
use crate::types::FileTreeEntry;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdChevronDown, LdChevronRight, LdFileCode2, LdFileText, LdFolder, LdFolderOpen,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
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

pub const QUICK_OPEN_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickOpenItem {
    pub path: PathBuf,
    pub name: String,
}

pub fn flatten_files(entries: &[FileTreeEntry]) -> Vec<QuickOpenItem> {
    let mut items = Vec::new();
    flatten_files_walk(entries, &mut items);
    items
}

fn flatten_files_walk(entries: &[FileTreeEntry], items: &mut Vec<QuickOpenItem>) {
    for entry in entries {
        if entry.is_dir {
            flatten_files_walk(&entry.children, items);
        } else {
            items.push(QuickOpenItem {
                path: entry.path.clone(),
                name: entry.name.clone(),
            });
        }
    }
}

const FILENAME_BIAS: i32 = 1000;
const BONUS_CONSECUTIVE: i32 = 8;
const BONUS_START: i32 = 8;
const BONUS_WORD: i32 = 6;
const BONUS_CAMEL: i32 = 6;
const SCORE_MATCH: i32 = 1;

fn slash_norm(s: &str) -> String {
    s.replace('\\', "/")
}

fn is_word_sep(c: char) -> bool {
    matches!(c, '/' | '_' | '-' | '.' | ' ')
}

pub(crate) fn fuzzy_match(haystack: &str, needle: &str) -> Option<(i32, Vec<usize>)> {
    let needle: String = slash_norm(needle.trim()).chars().map(|c| c.to_ascii_lowercase()).collect();
    if needle.is_empty() {
        return Some((0, Vec::new()));
    }

    let hay_norm = slash_norm(haystack);
    let orig: Vec<char> = hay_norm.chars().collect();
    let lower: Vec<char> = orig.iter().map(|c| c.to_ascii_lowercase()).collect();

    let mut matches = Vec::new();
    let mut search_from = 0usize;
    let mut score = 0i32;

    for nch in needle.chars() {
        let rel = lower[search_from..].iter().position(|&c| c == nch)?;
        let idx = search_from + rel;
        score += SCORE_MATCH;
        if idx == 0 {
            score += BONUS_START;
        } else {
            if matches.last() == Some(&(idx - 1)) {
                score += BONUS_CONSECUTIVE;
            }
            let prev = orig[idx - 1];
            if is_word_sep(prev) {
                score += BONUS_WORD;
            }
            if prev.is_ascii_lowercase() && orig[idx].is_ascii_uppercase() {
                score += BONUS_CAMEL;
            }
        }
        matches.push(idx);
        search_from = idx + 1;
    }

    Some((score, matches))
}

fn quick_open_relative(path: &Path, workspace_root: Option<&Path>) -> String {
    let raw = if let Some(root) = workspace_root {
        path.strip_prefix(root)
            .map_or_else(|_| path.display().to_string(), |rel| rel.display().to_string())
    } else {
        path.display().to_string()
    };
    slash_norm(&raw)
}

fn quick_open_score(item: &QuickOpenItem, query: &str, workspace_root: Option<&Path>) -> Option<i32> {
    let name_score = fuzzy_match(&item.name, query).map(|(s, _)| s);
    let path_score = fuzzy_match(&quick_open_relative(&item.path, workspace_root), query).map(|(s, _)| s);
    match (name_score, path_score) {
        (Some(name), Some(path)) => Some(name.saturating_add(FILENAME_BIAS).max(path)),
        (Some(name), None) => Some(name.saturating_add(FILENAME_BIAS)),
        (None, Some(path)) => Some(path),
        (None, None) => None,
    }
}

pub fn rank_quick_open(
    files: &[QuickOpenItem],
    recents: &[PathBuf],
    query: &str,
    workspace_root: Option<&Path>,
    limit: usize,
) -> Vec<QuickOpenItem> {
    let query = query.trim();
    if query.is_empty() {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for recent in recents {
            if result.len() >= limit {
                break;
            }
            if !seen.insert(recent.clone()) {
                continue;
            }
            if let Some(item) = files.iter().find(|f| f.path == *recent) {
                result.push(item.clone());
            } else {
                let name = recent
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                result.push(QuickOpenItem {
                    path: recent.clone(),
                    name,
                });
            }
        }

        for item in files {
            if result.len() >= limit {
                break;
            }
            if seen.insert(item.path.clone()) {
                result.push(item.clone());
            }
        }

        return result;
    }

    let files_set: HashSet<&PathBuf> = files.iter().map(|f| &f.path).collect();
    let mut scored: Vec<(usize, QuickOpenItem, i32)> = Vec::new();
    let mut order = 0usize;

    for item in files {
        if let Some(score) = quick_open_score(item, query, workspace_root) {
            scored.push((order, item.clone(), score));
        }
        order += 1;
    }

    for recent in recents {
        if files_set.contains(recent) {
            continue;
        }
        let name = recent
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let item = QuickOpenItem {
            path: recent.clone(),
            name,
        };
        if let Some(score) = quick_open_score(&item, query, workspace_root) {
            scored.push((order, item, score));
            order += 1;
        }
    }

    scored.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, item, _)| item)
        .collect()
}

#[derive(Props, Clone)]
pub struct FileTreeRowItemProps {
    pub row: FileTreeRow,
    pub active_path: Option<PathBuf>,
    pub translations: &'static crate::i18n::Translations,
    pub store: Signal<AppStore>,
    pub on_toggle_dir: EventHandler<PathBuf>,
    pub on_select: EventHandler<PathBuf>,
    pub on_pin_select: EventHandler<PathBuf>,
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
        let path_click = row.path.clone();
        let path_dbl = row.path.clone();
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
                    onclick: move |_| props.on_select.call(path_click.clone()),
                    ondoubleclick: move |_| props.on_pin_select.call(path_dbl.clone()),
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

    fn file_at(name: &str, path: &str) -> FileTreeEntry {
        FileTreeEntry {
            name: name.to_string(),
            path: PathBuf::from(path),
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

    #[test]
    fn flatten_files_skips_dirs_collects_nested() {
        let tree = vec![dir(
            "notes",
            vec![file_at("alpha.md", "notes/alpha.md"), dir("nested", vec![file_at("beta.md", "notes/nested/beta.md")])],
        )];
        let items = flatten_files(&tree);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "alpha.md");
        assert_eq!(items[1].name, "beta.md");
    }

    #[test]
    fn rank_quick_open_empty_query_recents_first() {
        let files = vec![
            QuickOpenItem {
                path: PathBuf::from("/ws/a.md"),
                name: "a.md".to_string(),
            },
            QuickOpenItem {
                path: PathBuf::from("/ws/b.md"),
                name: "b.md".to_string(),
            },
        ];
        let recents = vec![PathBuf::from("/ws/b.md"), PathBuf::from("/ws/c.md")];
        let ranked = rank_quick_open(&files, &recents, "", Some(Path::new("/ws")), 50);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].name, "b.md");
        assert_eq!(ranked[1].name, "c.md");
        assert_eq!(ranked[2].name, "a.md");
    }

    #[test]
    fn fuzzy_match_subsequence_hits_and_misses() {
        assert!(fuzzy_match("file_tree.rs", "ftr").is_some());
        assert!(fuzzy_match("quick_open.rs", "qo").is_some());
        assert!(fuzzy_match("file_tree.rs", "xyz").is_none());
    }

    #[test]
    fn rank_quick_open_fuzzy_subsequence() {
        let files = vec![
            QuickOpenItem {
                path: PathBuf::from("/ws/file_tree.rs"),
                name: "file_tree.rs".to_string(),
            },
            QuickOpenItem {
                path: PathBuf::from("/ws/readme.md"),
                name: "readme.md".to_string(),
            },
        ];
        let ranked = rank_quick_open(&files, &[], "ftr", Some(Path::new("/ws")), 50);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].name, "file_tree.rs");
        assert!(rank_quick_open(&files, &[], "xyz", Some(Path::new("/ws")), 50).is_empty());
    }

    #[test]
    fn rank_quick_open_starts_with_beats_contains() {
        let files = vec![
            QuickOpenItem {
                path: PathBuf::from("/ws/notes.md"),
                name: "notes.md".to_string(),
            },
            QuickOpenItem {
                path: PathBuf::from("/ws/my-notes.md"),
                name: "my-notes.md".to_string(),
            },
        ];
        let ranked = rank_quick_open(&files, &[], "note", Some(Path::new("/ws")), 50);
        assert_eq!(ranked[0].name, "notes.md");
        assert_eq!(ranked[1].name, "my-notes.md");
    }

    #[test]
    fn rank_quick_open_name_beats_path() {
        let files = vec![
            QuickOpenItem {
                path: PathBuf::from("/ws/src/readme.md"),
                name: "readme.md".to_string(),
            },
            QuickOpenItem {
                path: PathBuf::from("/ws/notes.md"),
                name: "notes.md".to_string(),
            },
        ];
        let by_note = rank_quick_open(&files, &[], "note", Some(Path::new("/ws")), 50);
        assert_eq!(by_note[0].name, "notes.md");
        let by_src = rank_quick_open(&files, &[], "src", Some(Path::new("/ws")), 50);
        assert_eq!(by_src[0].name, "readme.md");
    }

    #[test]
    fn rank_quick_open_case_insensitive() {
        let files = vec![QuickOpenItem {
            path: PathBuf::from("/ws/ReadME.md"),
            name: "ReadME.md".to_string(),
        }];
        let ranked = rank_quick_open(&files, &[], "readme", Some(Path::new("/ws")), 50);
        assert_eq!(ranked.len(), 1);
    }

    #[test]
    fn rank_quick_open_caps_at_limit() {
        let files: Vec<QuickOpenItem> = (0..60)
            .map(|i| QuickOpenItem {
                path: PathBuf::from(format!("/ws/file{i}.md")),
                name: format!("file{i}.md"),
            })
            .collect();
        let ranked = rank_quick_open(&files, &[], "", None, 50);
        assert_eq!(ranked.len(), 50);
    }

    #[test]
    fn rank_quick_open_matches_relative_path() {
        let files = vec![QuickOpenItem {
            path: PathBuf::from("/ws/notes/alpha.md"),
            name: "alpha.md".to_string(),
        }];
        let ranked = rank_quick_open(&files, &[], "notes", Some(Path::new("/ws")), 50);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].name, "alpha.md");
    }

    #[test]
    fn rank_quick_open_normalizes_path_separators() {
        let files = vec![QuickOpenItem {
            path: PathBuf::from("/ws").join("notes").join("alpha.md"),
            name: "alpha.md".to_string(),
        }];
        let ranked = rank_quick_open(&files, &[], "notes/alpha", Some(Path::new("/ws")), 50);
        assert_eq!(ranked.len(), 1);
    }
}
