use crate::components::context_menu::TabContextMenu;
use crate::components::Hint;
use crate::state::AppStore;
use crate::types::TabItem;
use dioxus::html::input_data::MouseButton;
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
    let preview_tab_id = store_read.preview_tab_id;

    rsx! {
        nav {
            class: "app-tab-bar flex items-center h-6 min-h-[24px] bg-[var(--bg-surface)] border-b border-[var(--border-color)]",
            div {
                class: "tabs-list-container flex items-center overflow-x-auto max-w-[calc(100%-28px)]",
                for tab in &store_read.tabs {
                    TabItemElement {
                        key: "{tab.id}",
                        tab: tab.clone(),
                        is_active: tab.id == store_read.active_tab_id,
                        is_preview: preview_tab_id == Some(tab.id),
                        can_close: true,
                        can_close_others: total_tabs > 1,
                        close_tooltip: t.tab_bar.close_tab,
                        translations: t,
                        store: store,
                        on_select: move |id| {
                            store.write().select_tab(id);
                        },
                        on_close: move |id| {
                            store.write().close_tab(id);
                        },
                        on_pin: move |id| {
                            store.write().pin_tab(id);
                        },
                    }
                }
            }
            Hint {
                text: t.tab_bar.new_file_or_tab,
                button {
                    class: "tab-new-button w-[18px] h-[18px] min-w-[18px] min-h-[18px] shrink-0 bg-transparent border-0 text-[var(--text-muted)] flex items-center justify-center cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150",
                    onclick: move |_| {
                        store.write().new_empty_tab();
                    },
                    Icon { width: 12, height: 12, icon: LdPlus }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TabItemElementProps {
    tab: TabItem,
    is_active: bool,
    is_preview: bool,
    can_close: bool,
    can_close_others: bool,
    close_tooltip: &'static str,
    translations: &'static crate::i18n::Translations,
    store: Signal<AppStore>,
    on_select: EventHandler<usize>,
    on_close: EventHandler<usize>,
    on_pin: EventHandler<usize>,
}

#[component]
fn TabItemElement(props: TabItemElementProps) -> Element {
    let tab_id = props.tab.id;
    let is_code = std::path::Path::new(&props.tab.title)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "mdx" | "rs" | "json" | "ts" | "js" | "toml"));

    let tab_class = if props.is_active {
        "tab-item active inline-flex items-center gap-1.5 h-6 px-2 bg-[var(--bg-app)] text-[var(--text-heading)] text-[11px] cursor-pointer whitespace-nowrap max-w-[200px] transition-colors duration-150"
    } else {
        "tab-item inline-flex items-center gap-1.5 h-6 px-2 text-[var(--text-muted)] text-[11px] cursor-pointer whitespace-nowrap max-w-[200px] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors duration-150"
    };

    let title_class = if props.is_preview {
        "tab-title truncate italic"
    } else {
        "tab-title truncate"
    };

    rsx! {
        TabContextMenu {
            t: props.translations,
            tab_id: tab_id,
            tab_path: props.tab.path.clone(),
            is_preview: props.is_preview,
            can_close: props.can_close,
            can_close_others: props.can_close_others,
            store: props.store,
            div {
                class: "{tab_class}",
                onclick: move |_| props.on_select.call(tab_id),
                ondoubleclick: move |_| props.on_pin.call(tab_id),
                onauxclick: move |evt| {
                    if evt.trigger_button() == Some(MouseButton::Auxiliary) {
                        evt.stop_propagation();
                        props.on_close.call(tab_id);
                    }
                },
                onmousedown: move |evt| {
                    if evt.trigger_button() == Some(MouseButton::Auxiliary) {
                        evt.stop_propagation();
                        evt.prevent_default();
                    }
                },
                span {
                    class: "tab-file-icon shrink-0 flex items-center text-[var(--accent)]",
                    if is_code {
                        Icon { width: 12, height: 12, icon: LdFileCode2 }
                    } else {
                        Icon { width: 12, height: 12, icon: LdFileText }
                    }
                }
                span { class: "{title_class}", "{props.tab.title}" }
                if props.tab.is_dirty {
                    Hint {
                        text: props.translations.tab_bar.unsaved_changes,
                        span { class: "tab-dirty-indicator w-1.5 h-1.5 rounded-full bg-[var(--accent)] shrink-0 animate-pulse" }
                    }
                }
                if props.can_close {
                    Hint {
                        text: props.close_tooltip,
                        button {
                            class: "tab-close-button bg-transparent border-0 text-[var(--text-muted)] rounded w-3.5 h-3.5 flex items-center justify-center cursor-pointer hover:bg-white/10 hover:text-[var(--text-heading)] transition-colors",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                props.on_close.call(tab_id);
                            },
                            Icon { width: 10, height: 10, icon: LdX }
                        }
                    }
                }
            }
        }
    }
}
