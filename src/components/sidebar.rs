use crate::state::AppStore;
use crate::types::{FileTreeEntry, SidebarTab, TocItem};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdAlignLeft, LdBookOpen, LdChevronDown, LdChevronRight, LdFileCode2, LdFileText, LdFolder,
    LdFolderOpen, LdFolders, LdSearch,
};
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub store: Signal<AppStore>,
    pub on_select_heading: EventHandler<String>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut file_search_query = use_signal(String::new);
    let mut store = props.store;
    let store_read = store();

    let toc = store_read.active_tab().map_or_else(Vec::new, |t| t.parsed.toc.clone());
    let file_tree = store_read.file_tree.clone();
    let current_tab = store_read.sidebar_tab;

    rsx! {
        aside {
            class: "app-sidebar w-64 min-w-[220px] max-w-[380px] h-full bg-[var(--bg-surface)] border-r border-[var(--border-color)] flex flex-col overflow-hidden select-none shrink-0",
            // Sidebar Header Switcher
            div {
                class: "sidebar-mode-switcher flex border-b border-[var(--border-color)] p-1 gap-1",
                button {
                    class: if current_tab == SidebarTab::Toc { "mode-btn active flex-1 h-6.5 bg-[var(--bg-subtle)] text-[var(--accent)] border-0 rounded text-xs font-semibold cursor-pointer transition-all duration-150 inline-flex items-center justify-center gap-1.5" } else { "mode-btn flex-1 h-6.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-semibold cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all duration-150 inline-flex items-center justify-center gap-1.5" },
                    onclick: move |_| store.write().set_sidebar_tab(SidebarTab::Toc),
                    Icon { width: 13, height: 13, icon: LdAlignLeft }
                    span { "Outline ({toc.len()})" }
                }
                button {
                    class: if current_tab == SidebarTab::Files { "mode-btn active flex-1 h-6.5 bg-[var(--bg-subtle)] text-[var(--accent)] border-0 rounded text-xs font-semibold cursor-pointer transition-all duration-150 inline-flex items-center justify-center gap-1.5" } else { "mode-btn flex-1 h-6.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-semibold cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all duration-150 inline-flex items-center justify-center gap-1.5" },
                    onclick: move |_| store.write().set_sidebar_tab(SidebarTab::Files),
                    Icon { width: 13, height: 13, icon: LdFolders }
                    span { "Files" }
                }
            }

            // Outline / TOC Content
            if current_tab == SidebarTab::Toc {
                div {
                    class: "sidebar-toc-container flex-1 overflow-y-auto p-2",
                    if toc.is_empty() {
                        div {
                            class: "sidebar-empty-state py-8 px-4 text-[var(--text-muted)] text-xs text-center leading-relaxed flex flex-col items-center justify-center",
                            Icon { width: 22, height: 22, icon: LdBookOpen, class: "opacity-40 mb-2" }
                            span { "No headings found in document." }
                        }
                    } else {
                        ul {
                            class: "toc-list list-none m-0 p-0",
                            for item in &toc {
                                TocItemLink {
                                    key: "{item.id}",
                                    item: item.clone(),
                                    on_select: props.on_select_heading,
                                }
                            }
                        }
                    }
                }
            }

            // Files Content
            if current_tab == SidebarTab::Files {
                div {
                    class: "sidebar-files-container flex-1 overflow-y-auto p-2 flex flex-col gap-2",
                    div {
                        class: "file-search-box px-0.5",
                        div {
                            class: "flex items-center w-full h-7 bg-[var(--bg-app)] border border-[var(--border-color)] rounded px-2 gap-1.5 focus-within:border-[var(--accent)] transition-colors",
                            Icon { width: 12, height: 12, icon: LdSearch, class: "text-[var(--text-muted)] shrink-0" }
                            input {
                                class: "file-search-input flex-1 bg-transparent border-0 text-[var(--text-main)] text-xs outline-none min-w-0 placeholder:text-[var(--text-muted)]",
                                r#type: "text",
                                placeholder: "Filter files...",
                                value: "{file_search_query}",
                                oninput: move |evt| file_search_query.set(evt.value()),
                            }
                        }
                    }
                    if file_tree.is_empty() {
                        div {
                            class: "sidebar-empty-state py-8 px-4 text-[var(--text-muted)] text-xs text-center leading-relaxed flex flex-col items-center justify-center",
                            Icon { width: 22, height: 22, icon: LdFolderOpen, class: "opacity-40 mb-2" }
                            span { "No folder opened." }
                            span { class: "opacity-75 mt-1", "Click 'Folder' in toolbar to open a documentation directory." }
                        }
                    } else {
                        div {
                            class: "file-tree-list flex flex-col gap-0.5",
                            for entry in &file_tree {
                                FileTreeItem {
                                    entry: entry.clone(),
                                    query: file_search_query(),
                                    on_select: move |path| {
                                        store.write().open_file_from_path(path);
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TocItemLinkProps {
    item: TocItem,
    on_select: EventHandler<String>,
}

#[component]
fn TocItemLink(props: TocItemLinkProps) -> Element {
    let item = &props.item;
    let heading_id = item.id.clone();
    let indent_level = (item.level.saturating_sub(1)).min(4);
    let padding = format!("{}rem", f32::from(indent_level) * 0.75);

    rsx! {
        li {
            class: "toc-item toc-level-{item.level} my-0.5",
            style: "padding-left: {padding};",
            a {
                class: "toc-link block text-[var(--text-muted)] hover:text-[var(--accent)] hover:bg-[var(--bg-hover)] text-xs py-1 px-2 rounded truncate transition-all duration-150 no-underline",
                href: "#{heading_id}",
                onclick: move |evt| {
                    evt.prevent_default();
                    props.on_select.call(heading_id.clone());
                },
                "{item.title}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FileTreeItemProps {
    entry: FileTreeEntry,
    query: String,
    on_select: EventHandler<PathBuf>,
}

#[component]
fn FileTreeItem(props: FileTreeItemProps) -> Element {
    let mut is_expanded = use_signal(|| true);
    let entry = &props.entry;
    let query_lower = props.query.to_lowercase();

    // If searching, check if match or child matches
    if !query_lower.is_empty() && !entry_matches(entry, &query_lower) {
        return rsx! {};
    }

    if entry.is_dir {
        let expanded = is_expanded();
        rsx! {
            div {
                class: "file-tree-dir flex flex-col",
                div {
                    class: "dir-row flex items-center gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150",
                    onclick: move |_| {
                        let exp = is_expanded();
                        is_expanded.set(!exp);
                    },
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
                if expanded {
                    div {
                        class: "dir-children pl-3.5 flex flex-col",
                        for child in &entry.children {
                            FileTreeItem {
                                entry: child.clone(),
                                query: props.query.clone(),
                                on_select: props.on_select,
                            }
                        }
                    }
                }
            }
        }
    } else {
        let path_clone = entry.path.clone();
        let is_code = entry.path.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
            matches!(ext.to_ascii_lowercase().as_str(), "mdx" | "rs" | "js" | "ts" | "json" | "toml" | "yaml" | "yml")
        });

        rsx! {
            div {
                class: "file-tree-file flex items-center gap-1.5 py-1 px-2 rounded text-xs cursor-pointer text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150 pl-5",
                onclick: move |_| props.on_select.call(path_clone.clone()),
                span {
                    class: "file-icon shrink-0 flex items-center text-[var(--text-muted)]",
                    if is_code {
                        Icon { width: 13, height: 13, icon: LdFileCode2 }
                    } else {
                        Icon { width: 13, height: 13, icon: LdFileText }
                    }
                }
                span { class: "file-name truncate", "{entry.name}" }
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
