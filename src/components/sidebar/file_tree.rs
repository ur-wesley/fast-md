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

pub(super) fn dir_ancestors_of_file(path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut current = path.parent();
    while let Some(dir) = current {
        dirs.push(dir.to_path_buf());
        current = dir.parent();
    }
    dirs
}

#[derive(Props, Clone)]
pub(super) struct FileTreeItemProps {
    pub entry: FileTreeEntry,
    pub query: String,
    pub active_path: Option<PathBuf>,
    pub expanded_dirs: Signal<HashSet<PathBuf>>,
    pub translations: &'static crate::i18n::Translations,
    pub store: Signal<AppStore>,
    pub on_toggle_dir: EventHandler<PathBuf>,
    pub on_select: EventHandler<PathBuf>,
}

impl PartialEq for FileTreeItemProps {
    fn eq(&self, other: &Self) -> bool {
        let self_active = self.active_path.as_ref() == Some(&self.entry.path);
        let other_active = other.active_path.as_ref() == Some(&other.entry.path);
        self.entry == other.entry
            && self.query == other.query
            && self_active == other_active
            && self.expanded_dirs == other.expanded_dirs
            && std::ptr::eq(self.translations, other.translations)
    }
}

#[component]
pub(super) fn FileTreeItem(props: FileTreeItemProps) -> Element {
    let entry = &props.entry;
    let query_lower = props.query.to_lowercase();

    if !query_lower.is_empty() && !entry_matches(entry, &query_lower) {
        return rsx! {};
    }

    if entry.is_dir {
        let force_expand = !query_lower.is_empty();
        let expanded = force_expand || (props.expanded_dirs)().contains(&entry.path);
        let dir_path = entry.path.clone();
        rsx! {
            div {
                class: "file-tree-dir flex flex-col",
                FileTreeContextMenu {
                    t: props.translations,
                    path: entry.path.clone(),
                    is_dir: true,
                    store: props.store,
                    on_open: props.on_select,
                    div {
                        class: "dir-row flex items-center gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150",
                        onclick: move |_| props.on_toggle_dir.call(dir_path.clone()),
                        span {
                            class: "dir-arrow text-[var(--text-muted)] w-3 shrink-0 flex items-center justify-center",
                            if expanded {
                                Icon { width: 10, height: 10, icon: LdChevronDown }
                            } else {
                                Icon { width: 10, height: 10, icon: LdChevronRight }
                            }
                        }
                        span {
                            class: "dir-icon text-[var(--accent)] shrink-0 flex items-center",
                            if expanded {
                                Icon { width: 13, height: 13, icon: LdFolderOpen }
                            } else {
                                Icon { width: 13, height: 13, icon: LdFolder }
                            }
                        }
                        span { class: "dir-name truncate font-medium", "{entry.name}" }
                    }
                }
                if expanded {
                    div {
                        class: "dir-children pl-3.5 flex flex-col",
                        for child in entry.children.iter() {
                            FileTreeItem {
                                key: "{child.path.display()}",
                                entry: child.clone(),
                                query: props.query.clone(),
                                active_path: props.active_path.clone(),
                                expanded_dirs: props.expanded_dirs,
                                translations: props.translations,
                                store: props.store,
                                on_toggle_dir: props.on_toggle_dir,
                                on_select: props.on_select,
                            }
                        }
                    }
                }
            }
        }
    } else {
        let path_clone = entry.path.clone();
        let is_active = props.active_path.as_ref() == Some(&entry.path);
        let ext = entry.path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
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

        let is_code = matches!(ext.as_str(), "mdx" | "rs" | "js" | "ts" | "json" | "jsonc" | "toml" | "yaml" | "yml" | "ini" | "ron" | "xml");

        rsx! {
            FileTreeContextMenu {
                t: props.translations,
                path: entry.path.clone(),
                is_dir: false,
                store: props.store,
                on_open: props.on_select,
                div {
                    class: if is_active {
                        "file-tree-file flex items-center justify-between gap-1.5 py-1 px-2 rounded text-xs cursor-pointer bg-[var(--bg-hover)] text-[var(--accent)] font-semibold transition-colors duration-150 pl-5 border-l-2 border-[var(--accent)]"
                    } else {
                        "file-tree-file flex items-center justify-between gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150 pl-5"
                    },
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
                        span { class: "file-name truncate", "{entry.name}" }
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

fn entry_matches(entry: &FileTreeEntry, query: &str) -> bool {
    if entry.name.to_lowercase().contains(query) {
        return true;
    }
    entry.children.iter().any(|c| entry_matches(c, query))
}
