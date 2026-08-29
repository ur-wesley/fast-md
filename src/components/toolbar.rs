use crate::services::fs::{
    pick_export_html_async, pick_file_async, pick_folder_async, pick_save_file_async,
    save_document_file,
};
use crate::state::AppStore;
use crate::types::DocumentMode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdBookOpen, LdColumns2, LdFileCode2, LdSave, LdSparkles,
};

const APP_STYLES: &str = include_str!("../assets/style.css");

#[derive(Props, Clone, PartialEq, Eq)]
pub struct ToolbarProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();
    let current_mode = store_read.mode;
    let is_dirty = store_read.active_tab().is_some_and(|t| t.is_dirty);

    rsx! {
        header {
            class: "app-toolbar flex items-center justify-between h-10 min-h-[40px] px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] z-50",

            // Left Action Group: Sidebar & File Management
            div {
                class: "toolbar-left-group flex items-center gap-2",

                // Sidebar Toggle
                button {
                    class: if store_read.show_sidebar { "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md text-[var(--accent)] text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "{t.toolbar.toggle_sidebar}",
                    onclick: move |_| {
                        store.write().toggle_sidebar();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "15",
                        height: "15",
                        rect { width: "18", height: "18", x: "3", y: "3", rx: "2", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "M9 3v18", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "{t.toolbar.sidebar}" }
                    if store_read.show_sidebar {
                        span { class: "btn-indicator-dot w-1.5 h-1.5 rounded-full bg-[var(--accent)] ml-0.5" }
                    }
                }

                div { class: "toolbar-sep w-[1px] h-4.5 bg-[var(--border-color)] mx-0.5" }

                // Segmented File Controls
                div {
                    class: "toolbar-segmented-group inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md p-0.5 gap-0.5",
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "{t.toolbar.open_file}",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(path) = pick_file_async().await {
                                    store.write().open_file_from_path(path);
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", fill: "none", stroke: "currentColor", stroke_width: "2" }
                            path { d: "M14 2v6h6", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "{t.toolbar.open}" }
                    }
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "{t.toolbar.open_folder}",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(dir) = pick_folder_async().await {
                                    store.write().open_directory(dir);
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "{t.toolbar.folder}" }
                    }
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "{t.toolbar.new_tab}",
                        onclick: move |_| {
                            store.write().new_empty_tab();
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M12 5v14M5 12h14", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round" }
                        }
                        span { class: "btn-text", "{t.toolbar.new}" }
                    }
                }

                // Quick Save Button
                button {
                    class: if is_dirty {
                        "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--accent)]/15 border border-[var(--accent)]/40 rounded-md text-[var(--accent)] text-xs font-semibold cursor-pointer hover:bg-[var(--accent)] hover:text-white transition-all duration-150"
                    } else {
                        "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150"
                    },
                    title: "{t.toolbar.save_file}",
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

            // Center Action Group: Document Mode Switcher (View | Split | WYSIWYG | Source)
            div {
                class: "toolbar-center-group flex items-center justify-center",

                div {
                    class: "toolbar-segmented-group inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5 shadow-sm",

                    // View Mode
                    button {
                        class: if current_mode == DocumentMode::View {
                            "segmented-btn active-segment inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-[var(--bg-surface)] rounded-md text-[var(--accent)] font-semibold text-xs transition-all shadow-sm"
                        } else {
                            "segmented-btn inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all"
                        },
                        title: "{t.toolbar.mode_view}",
                        onclick: move |_| store.write().set_mode(DocumentMode::View),
                        Icon { width: 13, height: 13, icon: LdBookOpen }
                        span { "{t.toolbar.mode_view}" }
                    }

                    // Split Live Preview Mode
                    button {
                        class: if current_mode == DocumentMode::Split {
                            "segmented-btn active-segment inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-[var(--bg-surface)] rounded-md text-[var(--accent)] font-semibold text-xs transition-all shadow-sm"
                        } else {
                            "segmented-btn inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all"
                        },
                        title: "{t.toolbar.mode_split}",
                        onclick: move |_| store.write().set_mode(DocumentMode::Split),
                        Icon { width: 13, height: 13, icon: LdColumns2 }
                        span { "{t.toolbar.mode_split}" }
                    }

                    // WYSIWYG Mode
                    button {
                        class: if current_mode == DocumentMode::Wysiwyg {
                            "segmented-btn active-segment inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-[var(--bg-surface)] rounded-md text-[var(--accent)] font-semibold text-xs transition-all shadow-sm"
                        } else {
                            "segmented-btn inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all"
                        },
                        title: "{t.toolbar.mode_wysiwyg}",
                        onclick: move |_| store.write().set_mode(DocumentMode::Wysiwyg),
                        Icon { width: 13, height: 13, icon: LdSparkles }
                        span { "{t.toolbar.mode_wysiwyg}" }
                    }

                    // Source Mode
                    button {
                        class: if current_mode == DocumentMode::Source {
                            "segmented-btn active-segment inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-[var(--bg-surface)] rounded-md text-[var(--accent)] font-semibold text-xs transition-all shadow-sm"
                        } else {
                            "segmented-btn inline-flex items-center gap-1.5 h-6.5 px-2.5 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-all"
                        },
                        title: "{t.toolbar.mode_source}",
                        onclick: move |_| store.write().set_mode(DocumentMode::Source),
                        Icon { width: 13, height: 13, icon: LdFileCode2 }
                        span { "{t.toolbar.mode_source}" }
                    }
                }
            }

            // Right Group: Export, Zen Mode & Settings
            div {
                class: "toolbar-right-group flex items-center gap-2",

                // Export to HTML
                button {
                    class: "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                    title: "{t.toolbar.export_html}",
                    onclick: move |_| {
                        spawn(async move {
                            let s = store();
                            if let Some(active_tab) = s.active_tab() {
                                let title = active_tab.title.replace(".md", "").replace(".mdx", "");
                                let html_content = active_tab.parsed.html_content.clone();
                                let theme_str = s.theme.as_str();
                                let custom_accent_style = s.primary_color.as_ref().map_or_else(String::new, |color| {
                                    format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40;")
                                });
                                if let Some(save_path) = pick_export_html_async(&format!("{title}.html")).await {
                                    let standalone_html = format!(
                                        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n<style>{APP_STYLES}</style>\n</head>\n<body class=\"{theme_str}\" style=\"{custom_accent_style}\">\n<div class=\"viewer-container reading-width\">\n<article class=\"markdown-body\">\n{html_content}\n</article>\n</div>\n</body>\n</html>"
                                    );
                                    let _ = save_document_file(&save_path, &standalone_html);
                                }
                            }
                        });
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "m7 10 5 5 5-5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "M12 15V3", fill: "none", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "{t.toolbar.export}" }
                }

                // Zen Mode
                button {
                    class: if store_read.is_zen { "toolbar-btn zen-btn active-zen inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--accent)] text-white border border-transparent rounded-md text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn zen-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "{t.toolbar.focus_zen_mode}",
                    onclick: move |_| {
                        store.write().toggle_zen();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path { d: "M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round" }
                    }
                    span { class: "btn-text", "{t.toolbar.zen}" }
                }

                div { class: "toolbar-sep w-[1px] h-4.5 bg-[var(--border-color)] mx-0.5" }

                // Settings Modal Button
                button {
                    class: if store_read.show_settings_modal { "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md text-[var(--accent)] text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "{t.toolbar.preferences}",
                    onclick: move |_| {
                        store.write().toggle_settings_modal();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path {
                            d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                        circle { cx: "12", cy: "12", r: "3", fill: "none", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "{t.toolbar.settings}" }
                }
            }
        }
    }
}

