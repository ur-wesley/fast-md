use crate::state::AppStore;
use dioxus::desktop::window;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct TitleBarProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn TitleBar(props: TitleBarProps) -> Element {
    let mut search_query = use_signal(String::new);
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();

    let active_tab = store_read.active_tab();
    let doc_title = active_tab.map_or_else(|| "Fast-MD".to_string(), |t| t.title.clone());

    let breadcrumb = if let Some(ref folder) = store_read.opened_folder {
        let folder_name = folder
            .file_name()
            .map_or_else(|| t.title_bar.workspace.to_string(), |f| f.to_string_lossy().to_string());
        format!("{folder_name} / {doc_title}")
    } else {
        doc_title
    };

    let is_maximized = window().is_maximized();
    let query_val = search_query();
    let has_query = !query_val.is_empty();

    rsx! {
        div {
            class: "app-titlebar flex items-center justify-between h-[38px] min-h-[38px] pl-3 pr-0 bg-[var(--bg-surface)] border-b border-[var(--border-color)] select-none z-[100]",
            onmousedown: move |_| {
                window().drag();
            },
            ondoubleclick: move |_| {
                let win = window();
                win.set_maximized(!win.is_maximized());
            },

            // Left Section: App Logo & Breadcrumb
            div {
                class: "titlebar-left-section flex items-center gap-2 min-w-0",
                onmousedown: move |evt| evt.stop_propagation(),

                div {
                    class: "titlebar-brand flex items-center gap-1.5 text-[var(--accent)] font-bold text-xs shrink-0",
                    svg {
                        class: "titlebar-brand-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "15",
                        height: "15",
                        path {
                            d: "M13 2L3 14h9l-1 8 10-12h-9l1-8z",
                            fill: "currentColor",
                        }
                    }
                    span { class: "titlebar-brand-name text-[var(--text-heading)]", "Fast-MD" }
                }

                div { class: "titlebar-divider w-[1px] h-3.5 bg-[var(--border-color)] shrink-0" }

                div {
                    class: "titlebar-breadcrumb inline-flex items-center gap-1.5 max-w-[320px] text-xs text-[var(--text-muted)] bg-[var(--bg-subtle)] px-2 py-0.5 rounded border border-[var(--border-subtle)] overflow-hidden",
                    title: "{breadcrumb}",
                    svg {
                        class: "breadcrumb-doc-icon text-[var(--accent)] shrink-0",
                        view_box: "0 0 24 24",
                        width: "13",
                        height: "13",
                        path {
                            d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                        }
                        path {
                            d: "M14 2v6h6",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                        }
                    }
                    span { class: "breadcrumb-text truncate", "{breadcrumb}" }
                }
            }

            // Center Section: Interactive Title Bar Search Box
            div {
                class: "titlebar-center-section flex items-center justify-center flex-1 max-w-[360px] mx-4",
                onmousedown: move |evt| evt.stop_propagation(),

                div {
                    class: "titlebar-search-box flex items-center w-full h-6 px-2 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-md gap-1.5 focus-within:border-[var(--accent)] transition-all duration-150",
                    svg {
                        class: "search-input-icon text-[var(--text-muted)] shrink-0",
                        view_box: "0 0 24 24",
                        width: "13",
                        height: "13",
                        circle {
                            cx: "11",
                            cy: "11",
                            r: "8",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                        }
                        path {
                            d: "m21 21-4.3-4.3",
                            stroke: "currentColor",
                            stroke_width: "2",
                        }
                    }

                    input {
                        id: "titlebar-search-input",
                        class: "titlebar-search-input flex-1 bg-transparent border-0 text-[var(--text-main)] text-xs outline-none min-w-0 placeholder:text-[var(--text-muted)]",
                        r#type: "text",
                        placeholder: "{t.title_bar.search_placeholder}",
                        value: "{query_val}",
                        oninput: move |evt| {
                            let val = evt.value();
                            search_query.set(val.clone());
                            dioxus::prelude::document::eval(&format!("window.highlightSearchMatches && window.highlightSearchMatches({val:?});"));
                        },
                        onkeydown: move |evt| {
                            let key = evt.key();
                            if key == Key::Escape {
                                search_query.set(String::new());
                                dioxus::prelude::document::eval("window.clearSearchHighlights && window.clearSearchHighlights(); document.getElementById('titlebar-search-input').blur();");
                            } else if key == Key::Enter {
                                if evt.modifiers().shift() {
                                    dioxus::prelude::document::eval("window.searchPrevMatch && window.searchPrevMatch();");
                                } else {
                                    dioxus::prelude::document::eval("window.searchNextMatch && window.searchNextMatch();");
                                }
                            }
                        }
                    }

                    span {
                        id: "search-match-count",
                        class: "search-match-count text-[11px] font-mono text-[var(--text-muted)] whitespace-nowrap",
                    }

                    if has_query {
                        button {
                            class: "search-control-btn flex items-center justify-center w-4 h-4 bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] rounded cursor-pointer transition-colors",
                            title: "{t.title_bar.prev_match}",
                            onclick: move |_| {
                                dioxus::prelude::document::eval("window.searchPrevMatch && window.searchPrevMatch();");
                            },
                            svg {
                                view_box: "0 0 24 24",
                                width: "11",
                                height: "11",
                                path { d: "m18 15-6-6-6 6", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                            }
                        }
                        button {
                            class: "search-control-btn flex items-center justify-center w-4 h-4 bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] rounded cursor-pointer transition-colors",
                            title: "{t.title_bar.next_match}",
                            onclick: move |_| {
                                dioxus::prelude::document::eval("window.searchNextMatch && window.searchNextMatch();");
                            },
                            svg {
                                view_box: "0 0 24 24",
                                width: "11",
                                height: "11",
                                path { d: "m6 9 6 6 6-6", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                            }
                        }
                        button {
                            class: "search-control-btn search-clear-btn flex items-center justify-center w-4 h-4 bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] rounded cursor-pointer transition-colors",
                            title: "{t.title_bar.clear_search}",
                            onclick: move |_| {
                                search_query.set(String::new());
                                dioxus::prelude::document::eval("window.clearSearchHighlights && window.clearSearchHighlights();");
                            },
                            svg {
                                view_box: "0 0 24 24",
                                width: "11",
                                height: "11",
                                path { d: "M18 6 6 18M6 6l12 12", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round" }
                            }
                        }
                    } else {
                        kbd {
                            class: "search-trigger-kbd text-[10px] font-mono bg-[var(--bg-subtle)] border border-[var(--border-color)] px-1.5 py-0.5 rounded text-[var(--text-muted)] shrink-0 cursor-pointer",
                            onclick: move |_| {
                                dioxus::prelude::document::eval("const el = document.getElementById('titlebar-search-input'); if (el) { el.focus(); el.select(); }");
                            },
                            "{t.title_bar.shortcut_ctrl_f}"
                        }
                    }
                }
            }

            // Right Section: Window Controls (Minimize, Maximize, Close) + Update Indicator
            div {
                class: "titlebar-window-controls flex items-center h-full",
                onmousedown: move |evt| evt.stop_propagation(),

                if let crate::types::UpdateStatus::Available(ref info) = store_read.update_status {
                    button {
                        class: "update-badge-btn inline-flex items-center gap-1.5 px-2.5 py-1 mr-2 rounded-full bg-[var(--accent)]/15 border border-[var(--accent)]/40 text-[var(--accent)] text-[11px] font-semibold hover:bg-[var(--accent)] hover:text-white transition-all cursor-pointer shadow-sm",
                        title: "{t.title_bar.update_available_badge}: v{info.version}",
                        onclick: move |_| {
                            store.write().set_settings_modal(true);
                        },
                        svg {
                            view_box: "0 0 24 24",
                            width: "12",
                            height: "12",
                            path {
                                d: "M12 3v13m0 0l-4-4m4 4l4-4M5 21h14",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2.2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                            }
                        }
                        span { "v{info.version}" }
                    }
                }

                if let crate::types::UpdateStatus::ReadyToRestart { ref version } = store_read.update_status {
                    button {
                        class: "update-badge-btn inline-flex items-center gap-1.5 px-2.5 py-1 mr-2 rounded-full bg-emerald-500/20 border border-emerald-500/50 text-emerald-400 text-[11px] font-semibold hover:bg-emerald-500 hover:text-white transition-all cursor-pointer shadow-sm",
                        title: "{t.settings.restart_and_update_button}",
                        onclick: move |_| {
                            let _ = crate::services::updater::restart_app();
                        },
                        svg {
                            view_box: "0 0 24 24",
                            width: "12",
                            height: "12",
                            path {
                                d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2.2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                            }
                        }
                        span { "Restart v{version}" }
                    }
                }

                button {
                    class: "window-control-btn btn-minimize flex items-center justify-center w-11 h-[38px] bg-transparent border-0 text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] cursor-pointer transition-colors duration-150",
                    title: "{t.title_bar.minimize}",
                    onclick: move |_| {
                        window().set_minimized(true);
                    },
                    svg {
                        view_box: "0 0 12 12",
                        width: "11",
                        height: "11",
                        path {
                            d: "M1 6h10",
                            stroke: "currentColor",
                            stroke_width: "1.2",
                            stroke_linecap: "round",
                        }
                    }
                }

                button {
                    class: "window-control-btn btn-maximize flex items-center justify-center w-11 h-[38px] bg-transparent border-0 text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] cursor-pointer transition-colors duration-150",
                    title: if is_maximized { "{t.title_bar.restore}" } else { "{t.title_bar.maximize}" },
                    onclick: move |_| {
                        let win = window();
                        win.set_maximized(!win.is_maximized());
                    },
                    if is_maximized {
                        svg {
                            view_box: "0 0 12 12",
                            width: "11",
                            height: "11",
                            path {
                                d: "M3.5 3.5V1.5h7v7H8.5M1.5 3.5h7v7h-7z",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.1",
                            }
                        }
                    } else {
                        svg {
                            view_box: "0 0 12 12",
                            width: "11",
                            height: "11",
                            rect {
                                x: "1.5",
                                y: "1.5",
                                width: "9",
                                height: "9",
                                rx: "1",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.1",
                            }
                        }
                    }
                }

                button {
                    class: "window-control-btn btn-close flex items-center justify-center w-11 h-[38px] bg-transparent border-0 text-[var(--text-muted)] hover:bg-[#e81123] hover:text-white cursor-pointer transition-colors duration-150",
                    title: "{t.title_bar.close}",
                    onclick: move |_| {
                        window().close();
                    },
                    svg {
                        view_box: "0 0 12 12",
                        width: "11",
                        height: "11",
                        path {
                            d: "M2 2l8 8M10 2l-8 8",
                            stroke: "currentColor",
                            stroke_width: "1.2",
                            stroke_linecap: "round",
                        }
                    }
                }
            }
        }
    }
}
