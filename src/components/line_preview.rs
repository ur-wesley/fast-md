use crate::types::{DocumentFormat, ParseStatus, ParsedDocument};
use crate::ui::virtual_list::VirtualList;
use dioxus::prelude::*;

pub const PREVIEW_ROW_HEIGHT: u32 = 21;

#[derive(Props, Clone, PartialEq)]
pub struct ConfigPreviewPaneProps {
    pub tab_id: usize,
    pub document: ParsedDocument,
    pub parse_status: ParseStatus,
    #[props(default)]
    pub window_tick: Option<Signal<u32>>,
}

#[component]
pub fn ConfigPreviewPane(props: ConfigPreviewPaneProps) -> Element {
    let doc = &props.document;
    let tab_id = props.tab_id;
    let preview_lines = doc.preview_lines.clone();
    let line_count = preview_lines.len().max(1);
    let is_plain = doc.format == DocumentFormat::PlainText;
    let is_loading = !matches!(props.parse_status, ParseStatus::Ready);

    let validation_banner = doc.validation_error.as_ref().map(|err| {
        rsx! {
            div {
                class: "config-syntax-error-banner",
                span { class: "error-icon", "⚠️" }
                span { class: "error-text", "Syntax Error: {err}" }
            }
        }
    });

    let status_text = if doc.validation_error.is_some() {
        "⚠️ Invalid Syntax"
    } else if is_plain {
        "Plain Text"
    } else {
        "✓ Valid Config"
    };

    let container_class = if is_plain {
        "plain-text-doc-container".to_string()
    } else {
        format!("config-doc-container format-{}", doc.format.syntax_token())
    };

    rsx! {
        div {
            class: "{container_class}",
            {validation_banner}
            if !is_plain {
                div {
                    class: "code-block-container config-code-block",
                    div {
                        class: "code-header",
                        div {
                            class: "flex items-center gap-2",
                            span { class: "code-lang-label", "{doc.format.label()}" }
                            span { class: "config-status-tag", "{status_text}" }
                        }
                        button {
                            class: "copy-code-button",
                            "data-tab-id": "{tab_id}",
                            onclick: move |_| {
                                dioxus::prelude::document::eval(&format!(
                                    "window.copyTabContent && window.copyTabContent({tab_id});"
                                ));
                            },
                            svg {
                                class: "copy-icon",
                                view_box: "0 0 24 24",
                                width: "14",
                                height: "14",
                                rect {
                                    width: "14",
                                    height: "14",
                                    x: "8",
                                    y: "8",
                                    rx: "2",
                                    ry: "2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                }
                                path {
                                    d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                }
                            }
                            span { "Copy" }
                        }
                    }
                    div {
                        class: "code-content config-line-preview-host",
                        if is_loading && preview_lines.is_empty() {
                            div { class: "config-preview-loading text-[var(--text-muted)] text-xs p-3", "Loading…" }
                        } else {
                            VirtualList {
                                item_count: line_count,
                                row_height: PREVIEW_ROW_HEIGHT,
                                class: "config-line-virtual-list".to_string(),
                                list_id: Some(format!("config-preview-{tab_id}")),
                                window_tick: props.window_tick,
                                render_row: Callback::new(move |idx: usize| {
                                    let line_html = preview_lines
                                        .get(idx)
                                        .cloned()
                                        .unwrap_or_default();
                                    rsx! {
                                        pre {
                                            class: "highlight config-preview-line",
                                            code {
                                                dangerous_inner_html: "{line_html}",
                                            }
                                        }
                                    }
                                }),
                                overlay: rsx! {},
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "plain-text-preview-host",
                    if is_loading && preview_lines.is_empty() {
                        div { class: "config-preview-loading text-[var(--text-muted)] text-xs p-3", "Loading…" }
                    } else {
                        VirtualList {
                            item_count: line_count,
                            row_height: PREVIEW_ROW_HEIGHT,
                            class: "plain-text-virtual-list font-mono text-sm".to_string(),
                            list_id: Some(format!("plain-preview-{tab_id}")),
                            window_tick: props.window_tick,
                            render_row: Callback::new(move |idx: usize| {
                                let line_html = preview_lines
                                    .get(idx)
                                    .cloned()
                                    .unwrap_or_default();
                                rsx! {
                                    div {
                                        class: "plain-text-preview-line whitespace-pre",
                                        dangerous_inner_html: "{line_html}",
                                    }
                                }
                            }),
                            overlay: rsx! {},
                        }
                    }
                }
            }
        }
    }
}
