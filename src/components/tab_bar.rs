use crate::services::fs::pick_file_async;
use crate::state::AppStore;
use crate::types::TabItem;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileCode2, LdFileText, LdPlus, LdX};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct TabBarProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn TabBar(props: TabBarProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();
    let total_tabs = store_read.tabs.len();

    rsx! {
        nav {
            class: "app-tab-bar flex items-center h-9 min-h-[36px] bg-[var(--bg-surface)] border-b border-[var(--border-color)] px-2 gap-1",
            div {
                class: "tabs-list-container flex items-center gap-1 overflow-x-auto max-w-[calc(100%-40px)]",
                for tab in &store_read.tabs {
                    TabItemElement {
                        key: "{tab.id}",
                        tab: tab.clone(),
                        is_active: tab.id == store_read.active_tab_id,
                        can_close: total_tabs > 1,
                        close_tooltip: t.tab_bar.close_tab,
                        on_select: move |id| {
                            store.write().select_tab(id);
                        },
                        on_close: move |id| {
                            store.write().close_tab(id);
                        },
                    }
                }
            }
            button {
                class: "tab-new-button w-6.5 h-6.5 bg-transparent border border-dashed border-[var(--border-color)] rounded-md text-[var(--text-muted)] flex items-center justify-center cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] hover:border-solid transition-all duration-150",
                title: "{t.tab_bar.new_file_or_tab}",
                onclick: move |_| {
                    spawn(async move {
                        if let Some(path) = pick_file_async().await {
                            store.write().open_file_from_path(path);
                        } else {
                            store.write().new_empty_tab();
                        }
                    });
                },
                Icon { width: 12, height: 12, icon: LdPlus }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TabItemElementProps {
    tab: TabItem,
    is_active: bool,
    can_close: bool,
    close_tooltip: &'static str,
    on_select: EventHandler<usize>,
    on_close: EventHandler<usize>,
}

#[component]
fn TabItemElement(props: TabItemElementProps) -> Element {
    let tab_id = props.tab.id;
    let is_code = std::path::Path::new(&props.tab.title)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "mdx" | "rs" | "json" | "ts" | "js" | "toml"));

    rsx! {
        div {
            class: if props.is_active { "tab-item active inline-flex items-center gap-2 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--accent)] text-[var(--text-heading)] font-medium rounded-md text-xs cursor-pointer whitespace-nowrap max-w-[220px] transition-all duration-150" } else { "tab-item inline-flex items-center gap-2 h-7 px-2.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-md text-[var(--text-muted)] text-xs cursor-pointer whitespace-nowrap max-w-[220px] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
            onclick: move |_| props.on_select.call(tab_id),
            span {
                class: "tab-file-icon shrink-0 flex items-center text-[var(--accent)]",
                if is_code {
                    Icon { width: 13, height: 13, icon: LdFileCode2 }
                } else {
                    Icon { width: 13, height: 13, icon: LdFileText }
                }
            }
            span { class: "tab-title truncate", "{props.tab.title}" }
            if props.can_close {
                button {
                    class: "tab-close-button bg-transparent border-0 text-[var(--text-muted)] rounded w-4 h-4 flex items-center justify-center cursor-pointer hover:bg-white/10 hover:text-[var(--text-heading)] transition-colors",
                    title: "{props.close_tooltip}",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        props.on_close.call(tab_id);
                    },
                    Icon { width: 11, height: 11, icon: LdX }
                }
            }
        }
    }
}

