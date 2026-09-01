use crate::components::context_menu::{EditorContextMenu, PreviewContextMenu};
use crate::components::editor_toolbar::EditorToolbar;
use crate::components::frontmatter_card::FrontmatterCard;
use crate::components::line_preview::{ConfigPreviewPane, PREVIEW_ROW_HEIGHT};
use crate::state::AppStore;
use crate::types::{DocumentFormat, DocumentMode, Language, ParseStatus, ParsedDocument};
use crate::ui::virtual_list::{total_list_height, visible_range};
use dioxus::prelude::*;

const GUTTER_ROW_HEIGHT: u32 = PREVIEW_ROW_HEIGHT;

#[derive(Props, Clone, PartialEq)]
pub struct EditorProps {
    #[props(default)]
    pub tab_id: usize,
    pub store: Signal<AppStore>,
    pub mode: DocumentMode,
    pub document: ParsedDocument,
    pub raw_content: String,
    pub html_revision: u64,
    pub parse_status: ParseStatus,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub sticky_headers: bool,
    pub language: Language,
}

#[component]
fn VirtualSourceGutter(
    line_count: usize,
    scroll_top: f64,
    viewport_height: f64,
    width_class: String,
    pad_class: String,
) -> Element {
    let (start, end, top_pad, bottom_pad) = visible_range(
        scroll_top,
        viewport_height,
        line_count,
        GUTTER_ROW_HEIGHT,
        16,
    );
    let total_height = total_list_height(line_count, GUTTER_ROW_HEIGHT);

    rsx! {
        div {
            id: "source-line-gutter",
            class: "editor-gutter {width_class} text-right text-[var(--text-muted)] bg-[var(--bg-subtle)]/50 select-none overflow-hidden shrink-0 font-mono text-xs border-r border-[var(--border-subtle)] pointer-events-none",
            "data-virtual-gutter": "1",
            div {
                id: "source-line-gutter-inner",
                class: "{pad_class}",
                style: "height: {total_height}px; position: relative;",
                if top_pad > 0 {
                    div { style: "height: {top_pad}px;" }
                }
                for line_num in (start + 1)..=(end) {
                    div {
                        class: "gutter-line",
                        style: "height: {GUTTER_ROW_HEIGHT}px; line-height: {GUTTER_ROW_HEIGHT}px;",
                        "data-line": "{line_num}",
                        "{line_num}"
                    }
                }
                if bottom_pad > 0 {
                    div { style: "height: {bottom_pad}px;" }
                }
            }
        }
    }
}

#[component]
pub fn Editor(props: EditorProps) -> Element {
    let mut store = props.store;
    let mode = props.mode;
    let tab_id = props.tab_id;
    let raw_content = props.raw_content.clone();
    let doc = &props.document;
    let t = props.language.strings();
    let is_plain_text = doc.format == DocumentFormat::PlainText;
    let show_wysiwyg = mode == DocumentMode::Wysiwyg && !is_plain_text;
    let show_source = mode == DocumentMode::Source || (mode == DocumentMode::Wysiwyg && is_plain_text);
    let zoom_factor = f64::from(props.zoom_level) / 100.0;
    let zoom_style = format!("zoom: {zoom_factor};");

    let line_count = raw_content.split('\n').count().max(1);
    let line_wrap = store().settings.line_wrap;
    let use_virtual_gutter = !line_wrap;
    let textarea_wrap_class = if line_wrap {
        "whitespace-pre-wrap overflow-x-hidden break-all"
    } else {
        "whitespace-pre overflow-x-auto"
    };
    let source_editor_wrap_class = if line_wrap {
        " is-wrapped"
    } else {
        ""
    };

    let mut gutter_scroll_top = use_signal(|| 0.0_f64);
    let mut gutter_viewport = use_signal(|| 400.0_f64);
    let mut preview_window_tick = use_signal(|| 0u32);

    let container_class = if props.is_full_width {
        "viewer-container full-width mx-auto max-w-[1280px] w-full pt-4 pb-12"
    } else {
        "viewer-container reading-width mx-auto max-w-[860px] w-full pt-4 pb-12"
    };

    let bind_editor_scroll = move || {
        dioxus::prelude::document::eval(&format!(
            "window.bindEditorScroll && window.bindEditorScroll({tab_id:?});"
        ));
    };

    use_effect(move || {
        let _ = store();
        let _ = mode;
        let _ = tab_id;
        let _ = line_wrap;
        bind_editor_scroll();
    });

    let textarea_scroll_handler = move |evt: Event<ScrollData>| {
        gutter_scroll_top.set(evt.scroll_top());
        gutter_viewport.set(f64::from(evt.data().client_height()).max(1.0));
        dioxus::prelude::document::eval("window.onEditorSourceScroll && window.onEditorSourceScroll();");
    };

    rsx! {
        div {
            class: "editor-wrapper flex-1 h-full flex flex-col relative overflow-hidden bg-[var(--bg-app)]",
            "data-tab-id": "{tab_id}",

            EditorToolbar {
                store: store,
                mode: mode,
            }

            div {
                class: "editor-workspace-area flex-1 flex overflow-hidden relative",

                if mode == DocumentMode::Split {
                    div {
                        class: "editor-split-container flex-1 flex w-full h-full overflow-hidden",

                        EditorContextMenu {
                            t: t,
                            div {
                                class: "split-editor-pane flex-1 h-full flex flex-col border-r border-[var(--border-color)] overflow-hidden bg-[var(--bg-app)]",
                                div {
                                    class: "editor-pane-header h-6.5 px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] text-[10.5px] uppercase font-mono font-bold tracking-wider text-[var(--text-muted)] flex items-center justify-between select-none shrink-0",
                                    span { "{doc.format.label()} Source" }
                                    span { "{line_count} lines" }
                                }
                                div {
                                    class: "source-code-editor{source_editor_wrap_class} flex-1 flex h-full overflow-hidden relative font-mono text-sm",
                                    if use_virtual_gutter {
                                        VirtualSourceGutter {
                                            line_count: line_count,
                                            scroll_top: gutter_scroll_top(),
                                            viewport_height: gutter_viewport(),
                                            width_class: "w-8".to_string(),
                                            pad_class: "".to_string(),
                                        }
                                    } else {
                                        div {
                                            id: "source-line-gutter",
                                            class: "editor-gutter w-8 text-right text-[var(--text-muted)] bg-[var(--bg-subtle)]/50 select-none overflow-hidden shrink-0 font-mono text-xs border-r border-[var(--border-subtle)] pointer-events-none",
                                            div {
                                                id: "source-line-gutter-inner",
                                                class: "",
                                                for line_num in 1..=line_count {
                                                    div {
                                                        class: "gutter-line",
                                                        "data-line": "{line_num}",
                                                        "{line_num}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    textarea {
                                        id: "source-markdown-textarea",
                                        class: "editor-textarea flex-1 h-full w-full bg-transparent text-[var(--text-main)] caret-[var(--accent)] font-mono text-xs border-0 outline-none resize-none overflow-y-auto {textarea_wrap_class}",
                                        "data-line-wrap": if line_wrap { "1" } else { "0" },
                                        "data-tab-id": "{tab_id}",
                                        value: "{raw_content}",
                                        placeholder: "{t.editor.source_placeholder}",
                                        spellcheck: false,
                                        onmounted: move |_| bind_editor_scroll(),
                                        onscroll: textarea_scroll_handler,
                                        oninput: move |evt| {
                                            let val = evt.value();
                                            store.write().update_active_tab_content(val);
                                        },
                                    }
                                }
                            }
                        }

                        div {
                            class: "split-preview-pane flex-1 h-full flex flex-col overflow-hidden bg-[var(--reader-glass-bg)]",
                            div {
                                class: "editor-pane-header h-6.5 px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] text-[10.5px] uppercase font-mono font-bold tracking-wider text-[var(--text-muted)] flex items-center justify-between select-none shrink-0",
                                span { if doc.format.is_markdown() { "Live Preview" } else { "{doc.format.label()} Preview" } }
                                span { class: "text-[var(--accent)] font-semibold", "{doc.word_count} words" }
                            }
                            div {
                                id: "split-preview-scroll-area",
                                class: "flex-1 h-full overflow-y-auto overflow-x-hidden p-6",
                                style: "{zoom_style}",
                                onmounted: move |_| {
                                    preview_window_tick.set(preview_window_tick().saturating_add(1));
                                    bind_editor_scroll();
                                },
                                onscroll: move |_| {
                                    preview_window_tick.set(preview_window_tick().saturating_add(1));
                                },
                                div {
                                    class: "{container_class}",
                                    if let Some(ref meta) = doc.metadata {
                                        FrontmatterCard {
                                            metadata: meta.clone(),
                                            language: props.language,
                                        }
                                    }
                                    PreviewContextMenu {
                                        t: t,
                                        if doc.uses_line_preview() {
                                            ConfigPreviewPane {
                                                tab_id: tab_id,
                                                document: doc.clone(),
                                                parse_status: props.parse_status,
                                                window_tick: Some(preview_window_tick),
                                            }
                                        } else {
                                            article {
                                                class: "markdown-body leading-relaxed",
                                                dangerous_inner_html: "{doc.html_content}",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if show_source {
                    EditorContextMenu {
                        t: t,
                        div {
                            class: "editor-source-container flex-1 h-full flex flex-col overflow-hidden bg-[var(--bg-app)]",
                            div {
                                class: "source-code-editor{source_editor_wrap_class} flex-1 flex h-full overflow-hidden relative font-mono",
                                if use_virtual_gutter {
                                    VirtualSourceGutter {
                                        line_count: line_count,
                                        scroll_top: gutter_scroll_top(),
                                        viewport_height: gutter_viewport(),
                                            width_class: "w-9".to_string(),
                                            pad_class: "".to_string(),
                                    }
                                } else {
                                    div {
                                        id: "source-line-gutter",
                                        class: "editor-gutter w-9 text-right text-[var(--text-muted)] bg-[var(--bg-subtle)]/40 select-none overflow-hidden shrink-0 font-mono text-sm border-r border-[var(--border-color)] pointer-events-none",
                                        div {
                                            id: "source-line-gutter-inner",
                                            class: "",
                                            for line_num in 1..=line_count {
                                                div {
                                                    class: "gutter-line",
                                                    "data-line": "{line_num}",
                                                    "{line_num}"
                                                }
                                            }
                                        }
                                    }
                                }
                                textarea {
                                    id: "source-markdown-textarea",
                                    class: "editor-textarea flex-1 h-full w-full bg-transparent text-[var(--text-main)] caret-[var(--accent)] font-mono text-sm border-0 outline-none resize-none overflow-y-auto {textarea_wrap_class}",
                                    "data-line-wrap": if line_wrap { "1" } else { "0" },
                                    "data-tab-id": "{tab_id}",
                                    value: "{raw_content}",
                                    placeholder: "{t.editor.source_placeholder}",
                                    spellcheck: false,
                                    onmounted: move |_| bind_editor_scroll(),
                                    onscroll: textarea_scroll_handler,
                                    oninput: move |evt| {
                                        let val = evt.value();
                                        store.write().update_active_tab_content(val);
                                    },
                                }
                            }
                        }
                    }
                }

                if show_wysiwyg {
                    EditorContextMenu {
                        t: t,
                        div {
                            id: "wysiwyg-scroll-area",
                            class: "editor-wysiwyg-container flex-1 h-full flex flex-col overflow-y-auto overflow-x-hidden p-6 bg-[var(--reader-glass-bg)]",
                            style: "{zoom_style}",
                            onmounted: move |_| bind_editor_scroll(),
                            div {
                                class: "{container_class}",
                                if let Some(ref meta) = doc.metadata {
                                    FrontmatterCard {
                                        metadata: meta.clone(),
                                        language: props.language,
                                    }
                                }
                                div {
                                    id: "wysiwyg-editor-surface",
                                    key: "{tab_id}-wysiwyg-{props.html_revision}",
                                    class: "wysiwyg-editor-surface markdown-body leading-relaxed outline-none min-h-[500px] p-2 rounded-xl focus:ring-1 focus:ring-[var(--accent)]/30 transition-all",
                                    contenteditable: "true",
                                    spellcheck: "true",
                                    dangerous_inner_html: "{doc.html_content}",
                                    onblur: move |_| {
                                        dioxus::prelude::document::eval(
                                            "if (window.syncWysiwygContent) window.syncWysiwygContent();",
                                        );
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
