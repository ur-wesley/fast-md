use crate::services::fs::{
    pick_file_async, pick_folder_async, pick_save_file_async, scan_file_tree,
};
use crate::state::{kick_pending_tree_scan, AppStore};
use crate::types::DocumentMode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdBookOpen, LdColumns2, LdFileCode2, LdSave, LdSparkles,
};

use crate::components::Hint;
use dioxus_primitives::{ContentAlign, ContentSide};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct ToolbarProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();
    let is_dirty = store_read.active_tab().is_some_and(|tab| tab.is_dirty);
    let current_mode = store_read.mode;
    let is_zen = store_read.is_zen;

    rsx! {
        header {
            class: "app-toolbar flex items-stretch justify-between h-10 min-h-[40px] px-2 bg-[var(--bg-surface)] border-b border-[var(--border-color)]",

            // Left Group: Transparent Full-Height File Actions (no rounded borders)
            div {
                class: "toolbar-left-group flex items-stretch h-full min-w-0",

                Hint {
                    text: t.toolbar.open_file,
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    button {
                        class: "toolbar-action-btn-flat",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(path) = pick_file_async().await {
                                    store.write().open_file_from_path(path);
                                    kick_pending_tree_scan(store);
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M14 2v6h6", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                        span { class: "btn-text", "{t.toolbar.open}" }
                    }
                }

                Hint {
                    text: t.toolbar.open_folder,
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    button {
                        class: "toolbar-action-btn-flat",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(dir) = pick_folder_async().await {
                                    store.write().start_loading_directory(dir.clone());
                                    let filter_mode = store().file_filter_mode;
                                    let scan_dir = dir.clone();
                                    let tree_res = tokio::task::spawn_blocking(move || {
                                        scan_file_tree(&scan_dir, filter_mode)
                                    }).await;

                                    if let Ok(Ok(tree)) = tree_res {
                                        store.write().finish_loading_directory(&dir, tree);
                                    } else {
                                        store.write().set_loading_files(false);
                                    }
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            path { d: "M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                        span { class: "btn-text", "{t.toolbar.folder}" }
                    }
                }

                Hint {
                    text: t.toolbar.new_tab,
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    button {
                        class: "toolbar-action-btn-flat",
                        onclick: move |_| {
                            store.write().new_empty_tab();
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            path { d: "M12 5v14M5 12h14", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round" }
                        }
                        span { class: "btn-text", "{t.toolbar.new}" }
                    }
                }

                Hint {
                    text: t.toolbar.save_file,
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    button {
                        class: if is_dirty { "toolbar-action-btn-flat toolbar-dirty-flat" } else { "toolbar-action-btn-flat" },
                        onclick: move |_| {
                            spawn(async move {
                                let s = store();
                                if let Some(active_tab) = s.active_tab() {
                                    if active_tab.path.is_some() {
                                        let _ = store.write().save_active_tab();
                                    } else {
                                        let title = active_tab.title.clone();
                                        if let Some(save_path) = pick_save_file_async(&title).await {
                                            let tab_id = active_tab.id;
                                            let _ = store.write().save_tab_with_path(tab_id, save_path);
                                        }
                                    }
                                }
                            });
                        },
                        Icon { width: 13, height: 13, icon: LdSave }
                        span { class: "btn-text", "{t.toolbar.save}" }
                        if is_dirty {
                            span { class: "btn-indicator-dot w-1.5 h-1.5 rounded-full bg-amber-400 ml-0.5" }
                        }
                    }
                }
            }

            // Center Group: Document View Modes Segmented Bar + Additional Zen Modifier
            div {
                class: "toolbar-center-group flex items-center justify-center gap-2.5 shrink-0",

                // View Modes Segmented Bar
                div {
                    class: "toolbar-segmented-group inline-flex items-stretch bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md overflow-hidden h-7 p-0 gap-0",

                    Hint {
                        text: t.toolbar.mode_view,
                        side: ContentSide::Bottom,
                        button {
                            class: if current_mode == DocumentMode::View { "toolbar-integrated-btn active" } else { "toolbar-integrated-btn" },
                            title: "{t.toolbar.mode_view}",
                            onclick: move |_| store.write().set_mode(DocumentMode::View),
                            Icon { width: 13, height: 13, icon: LdBookOpen }
                            span { class: "btn-text", "{t.toolbar.mode_view}" }
                        }
                    }

                    div { class: "toolbar-inner-sep w-[1px] h-full bg-[var(--border-color)] shrink-0" }

                    Hint {
                        text: t.toolbar.mode_split,
                        side: ContentSide::Bottom,
                        button {
                            class: if current_mode == DocumentMode::Split { "toolbar-integrated-btn active" } else { "toolbar-integrated-btn" },
                            title: "{t.toolbar.mode_split}",
                            onclick: move |_| store.write().set_mode(DocumentMode::Split),
                            Icon { width: 13, height: 13, icon: LdColumns2 }
                            span { class: "btn-text", "{t.toolbar.mode_split}" }
                        }
                    }

                    div { class: "toolbar-inner-sep w-[1px] h-full bg-[var(--border-color)] shrink-0" }

                    Hint {
                        text: t.toolbar.mode_wysiwyg,
                        side: ContentSide::Bottom,
                        button {
                            class: if current_mode == DocumentMode::Wysiwyg { "toolbar-integrated-btn active" } else { "toolbar-integrated-btn" },
                            title: "{t.toolbar.mode_wysiwyg}",
                            onclick: move |_| store.write().set_mode(DocumentMode::Wysiwyg),
                            Icon { width: 13, height: 13, icon: LdSparkles }
                            span { class: "btn-text", "{t.toolbar.mode_wysiwyg}" }
                        }
                    }

                    div { class: "toolbar-inner-sep w-[1px] h-full bg-[var(--border-color)] shrink-0" }

                    Hint {
                        text: t.toolbar.mode_source,
                        side: ContentSide::Bottom,
                        button {
                            class: if current_mode == DocumentMode::Source { "toolbar-integrated-btn active" } else { "toolbar-integrated-btn" },
                            title: "{t.toolbar.mode_source}",
                            onclick: move |_| store.write().set_mode(DocumentMode::Source),
                            Icon { width: 13, height: 13, icon: LdFileCode2 }
                            span { class: "btn-text", "{t.toolbar.mode_source}" }
                        }
                    }
                }

                // Additional Zen Option (distinct modifier toggle)
                Hint {
                    text: t.toolbar.focus_zen_mode,
                    side: ContentSide::Bottom,
                    button {
                        class: if is_zen { "toolbar-zen-toggle-btn active" } else { "toolbar-zen-toggle-btn" },
                        title: "{t.toolbar.zen}",
                        onclick: move |_| store.write().toggle_zen(),
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            path { d: "M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                        span { class: "btn-text", "{t.toolbar.zen}" }
                        if is_zen {
                            span { class: "zen-status-badge", "ON" }
                        }
                    }
                }
            }

            // Right Group: Flex spacer to keep center group balanced
            div {
                class: "toolbar-right-group flex items-center justify-end min-w-0",
            }
        }
    }
}
