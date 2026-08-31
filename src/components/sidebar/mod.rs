mod file_tree;
mod toc;

use crate::state::{kick_pending_tree_scan, AppStore};
use crate::types::{FileFilterMode, SidebarTab};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use crate::ui::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdAlignLeft, LdBookOpen, LdCheck, LdFilter, LdFolderOpen, LdFolders, LdRefreshCw, LdSearch,
    LdX,
};
use std::collections::HashSet;
use std::path::PathBuf;

use file_tree::{dir_ancestors_of_file, FileTreeItem};
use toc::TocItemLink;

fn sidebar_tab_index(tab: SidebarTab) -> usize {
    match tab {
        SidebarTab::Toc => 0,
        SidebarTab::Files => 1,
    }
}

fn sidebar_tab_from_index(idx: usize) -> SidebarTab {
    match idx {
        0 => SidebarTab::Toc,
        _ => SidebarTab::Files,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub store: Signal<AppStore>,
    pub on_select_heading: EventHandler<String>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut store = props.store;
    let mut file_search_query = use_signal(String::new);
    let mut expanded_dirs = use_signal(HashSet::<PathBuf>::new);
    let mut is_filter_menu_open = use_signal(|| false);
    let store_read = store();
    let t = store_read.language.strings();

    let toc = store_read.active_tab().map_or_else(Vec::new, |t| t.parsed.toc.clone());
    let file_tree = store_read.file_tree.clone();
    let current_tab = store_read.sidebar_tab;
    let is_loading_files = store_read.is_loading_files;
    let current_filter_mode = store_read.file_filter_mode;
    let active_path = store_read.active_tab().and_then(|tab| tab.path.clone());
    let sidebar_tab_pressed = use_memo(move || Some(HashSet::from([sidebar_tab_index(store().sidebar_tab)])));

    use_effect({
        let active_path = active_path.clone();
        move || {
            if let Some(ref path) = active_path {
                let mut next = expanded_dirs();
                for ancestor in dir_ancestors_of_file(path) {
                    next.insert(ancestor);
                }
                expanded_dirs.set(next);
            }
        }
    });

    let open_file = move |path: PathBuf| {
        store.write().open_file_from_path(path);
        kick_pending_tree_scan(store);
    };

    let toc_len = toc.len();
    use_effect(move || {
        let _ = toc_len;
        let _ = current_tab;
        dioxus::prelude::document::eval(
            "setTimeout(() => { window.refreshTocTreePath && window.refreshTocTreePath(); }, 20);"
        );
    });

    rsx! {
        aside {
            class: "app-sidebar h-full w-full bg-[var(--bg-surface)] flex flex-col overflow-hidden select-none",
            ToggleGroup {
                class: "sidebar-mode-switcher flex items-center border-b border-[var(--border-color)] p-1 gap-1 w-full box-border",
                horizontal: true,
                allow_multiple_pressed: false,
                pressed: sidebar_tab_pressed,
                on_pressed_change: move |pressed: HashSet<usize>| {
                    if let Some(&idx) = pressed.iter().next() {
                        store.write().set_sidebar_tab(sidebar_tab_from_index(idx));
                    }
                },
                ToggleItem {
                    index: 0usize,
                    class: "flex-1 flex flex-row items-center justify-center gap-1.5 whitespace-nowrap min-w-0 overflow-hidden",
                    Icon { width: 13, height: 13, icon: LdAlignLeft, class: "shrink-0" }
                    span { class: "truncate whitespace-nowrap", "{t.sidebar.outline} ({toc.len()})" }
                }
                ToggleItem {
                    index: 1usize,
                    class: "flex-1 flex flex-row items-center justify-center gap-1.5 whitespace-nowrap min-w-0 overflow-hidden",
                    if is_loading_files {
                        Icon { width: 12, height: 12, icon: LdRefreshCw, class: "animate-spin text-[var(--accent)] shrink-0" }
                    } else {
                        Icon { width: 13, height: 13, icon: LdFolders, class: "shrink-0" }
                    }
                    span { class: "truncate whitespace-nowrap", "{t.sidebar.files}" }
                }
            }

            if current_tab == SidebarTab::Toc {
                div {
                    class: "sidebar-toc-container flex-1 overflow-y-auto p-2",
                    if toc.is_empty() {
                        div {
                            class: "sidebar-empty-state py-8 px-4 text-[var(--text-muted)] text-xs text-center leading-relaxed flex flex-col items-center justify-center",
                            Icon { width: 22, height: 22, icon: LdBookOpen, class: "opacity-40 mb-2" }
                            span { "{t.sidebar.no_headings}" }
                        }
                    } else {
                        div {
                            class: "toc-wrapper relative min-h-full",
                            svg {
                                class: "toc-progress-svg absolute top-0 left-0 w-full h-full pointer-events-none overflow-visible",
                                id: "toc-progress-svg",
                                path {
                                    id: "toc-track-path",
                                    class: "toc-track-path",
                                    fill: "none",
                                }
                                path {
                                    id: "toc-progress-fill-path",
                                    class: "toc-progress-fill-path",
                                    fill: "none",
                                }
                                circle {
                                    id: "toc-progress-head",
                                    class: "toc-progress-head",
                                    r: "3.5",
                                    style: "opacity: 0;",
                                }
                            }
                            ul {
                                class: "toc-tree-list list-none m-0 p-0 relative",
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
            } else {
                div {
                    class: "sidebar-files-container flex-1 overflow-y-auto p-2 flex flex-col gap-2",
                    div {
                        class: "file-search-box flex items-center gap-1.5 px-0.5 relative",
                        div {
                            class: "flex items-center flex-1 min-w-0 h-7 bg-[var(--bg-app)] border border-[var(--border-color)] rounded px-2 gap-1.5 focus-within:border-[var(--accent)] transition-colors",
                            Icon { width: 12, height: 12, icon: LdSearch, class: "text-[var(--text-muted)] shrink-0" }
                            Input {
                                class: "file-search-input flex-1 bg-transparent border-0 text-[var(--text-main)] text-xs outline-none min-w-0 placeholder:text-[var(--text-muted)]",
                                r#type: "text",
                                placeholder: "{t.sidebar.filter_files}",
                                value: "{file_search_query()}",
                                oninput: move |evt: FormEvent| file_search_query.set(evt.value()),
                            }
                            if !file_search_query().is_empty() {
                                button {
                                    class: "text-[var(--text-muted)] hover:text-[var(--text-main)] cursor-pointer p-0.5 rounded transition-colors flex items-center justify-center",
                                    title: "Clear",
                                    onclick: move |_| file_search_query.set(String::new()),
                                    Icon { width: 11, height: 11, icon: LdX }
                                }
                            }
                        }

                        div {
                            class: "relative shrink-0",
                            button {
                                class: if is_filter_menu_open() || current_filter_mode != FileFilterMode::MarkdownAndConfig {
                                    "file-filter-dropdown-btn flex items-center justify-center w-7 h-7 bg-[var(--bg-subtle)] border border-[var(--accent)] text-[var(--accent)] rounded transition-colors cursor-pointer select-none"
                                } else {
                                    "file-filter-dropdown-btn flex items-center justify-center w-7 h-7 bg-[var(--bg-app)] hover:bg-[var(--bg-subtle)] border border-[var(--border-color)] hover:border-[var(--accent)] text-[var(--text-muted)] hover:text-[var(--text-main)] rounded transition-colors cursor-pointer select-none"
                                },
                                title: "{t.sidebar.filter_tooltip}",
                                onclick: move |_| is_filter_menu_open.toggle(),
                                Icon { width: 13, height: 13, icon: LdFilter, class: "shrink-0" }
                            }

                            if is_filter_menu_open() {
                                div {
                                    class: "fixed inset-0 z-40 bg-transparent",
                                    onclick: move |_| is_filter_menu_open.set(false),
                                }

                                div {
                                    class: "file-filter-menu-popover absolute right-0 top-full mt-1 z-50 w-56 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg shadow-xl p-1 flex flex-col gap-0.5",
                                    for (mode, label, desc) in [
                                        (FileFilterMode::MarkdownOnly, t.sidebar.filter_md_only, ".md, .mdx"),
                                        (FileFilterMode::MarkdownAndConfig, t.sidebar.filter_md_config, ".md, .json, .toml, .yaml..."),
                                        (FileFilterMode::AllSupported, t.sidebar.filter_all_supported, "Markdown, configs & code"),
                                        (FileFilterMode::AllFiles, t.sidebar.filter_all_files, "All directory files"),
                                    ] {
                                        {
                                            let is_active = current_filter_mode == mode;
                                            rsx! {
                                                button {
                                                    key: "{mode:?}",
                                                    class: if is_active {
                                                        "filter-menu-item active flex items-center justify-between w-full px-2.5 py-1.5 rounded-md text-left text-xs bg-[var(--bg-subtle)] text-[var(--accent)] font-medium cursor-pointer transition-colors"
                                                    } else {
                                                        "filter-menu-item flex items-center justify-between w-full px-2.5 py-1.5 rounded-md text-left text-xs text-[var(--text-main)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors"
                                                    },
                                                    onclick: move |_| {
                                                        store.write().set_file_filter_mode(mode);
                                                        is_filter_menu_open.set(false);
                                                    },
                                                    div {
                                                        class: "flex flex-col min-w-0 flex-1 pr-1",
                                                        span { class: "font-medium text-[11.5px] leading-tight truncate", "{label}" }
                                                        span { class: "text-[10px] text-[var(--text-muted)] opacity-80 leading-none mt-0.5 truncate", "{desc}" }
                                                    }
                                                    if is_active {
                                                        Icon { width: 13, height: 13, icon: LdCheck, class: "text-[var(--accent)] shrink-0 ml-1" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if is_loading_files {
                        div {
                            class: "sidebar-loading-container",
                            div {
                                class: "sidebar-loading-spinner-box shadow-sm",
                                Icon { width: 18, height: 18, icon: LdRefreshCw, class: "animate-spin" }
                            }
                            div {
                                class: "sidebar-loading-title",
                                "{t.sidebar.loading_folder}"
                            }
                            if let Some(ref dir) = store_read.opened_folder {
                                div {
                                    class: "sidebar-loading-folder-name",
                                    title: "{dir.display()}",
                                    "{dir.file_name().map_or_else(|| dir.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string())}"
                                }
                            }
                            div {
                                class: "sidebar-loading-subtitle",
                                "{t.sidebar.scanning_files}"
                            }
                            div {
                                class: "sidebar-skeleton-tree",
                                div { class: "sidebar-skeleton-row w-3/4" }
                                div { class: "sidebar-skeleton-row w-5/6 ml-3" }
                                div { class: "sidebar-skeleton-row w-2/3 ml-3" }
                                div { class: "sidebar-skeleton-row w-4/5" }
                                div { class: "sidebar-skeleton-row w-3/5 ml-3" }
                            }
                        }
                    } else if file_tree.is_empty() {
                        div {
                            class: "sidebar-empty-state py-8 px-4 text-[var(--text-muted)] text-xs text-center leading-relaxed flex flex-col items-center justify-center gap-2",
                            Icon { width: 24, height: 24, icon: LdFolderOpen, class: "opacity-40 mb-1" }
                            span { "{t.sidebar.no_folder_opened}" }
                            span { class: "opacity-75 text-[11px]", "{t.sidebar.open_folder_hint}" }
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                class: "mt-2",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Some(dir) = crate::services::fs::pick_folder_async().await {
                                            store.write().start_loading_directory(dir.clone());
                                            let filter_mode = store().file_filter_mode;
                                            let scan_dir = dir.clone();
                                            let tree_res = tokio::task::spawn_blocking(move || {
                                                crate::services::fs::scan_file_tree(&scan_dir, filter_mode)
                                            }).await;

                                            if let Ok(Ok(tree)) = tree_res {
                                                store.write().finish_loading_directory(&dir, tree);
                                            } else {
                                                store.write().set_loading_files(false);
                                            }
                                        }
                                    });
                                },
                                Icon { width: 13, height: 13, icon: LdFolderOpen }
                                span { "{t.sidebar.open_folder_button}" }
                            }
                        }
                    } else {
                        div {
                            class: "file-tree-list flex flex-col gap-0.5",
                            for entry in &file_tree {
                                FileTreeItem {
                                    key: "{entry.path.display()}",
                                    entry: entry.clone(),
                                    query: file_search_query(),
                                    active_path: active_path.clone(),
                                    expanded_dirs: expanded_dirs,
                                    translations: t,
                                    store: store,
                                    on_toggle_dir: move |path| {
                                        let mut next = expanded_dirs();
                                        if next.contains(&path) {
                                            next.remove(&path);
                                        } else {
                                            next.insert(path);
                                        }
                                        expanded_dirs.set(next);
                                    },
                                    on_select: open_file,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
