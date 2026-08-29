use crate::components::editor_toolbar::EditorToolbar;
use crate::components::frontmatter_card::FrontmatterCard;
use crate::state::AppStore;
use crate::types::{DocumentMode, Language, ParsedDocument};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct EditorProps {
    pub store: Signal<AppStore>,
    pub mode: DocumentMode,
    pub document: ParsedDocument,
    pub raw_content: String,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub sticky_headers: bool,
    pub language: Language,
}

#[component]
pub fn Editor(props: EditorProps) -> Element {
    let mut store = props.store;
    let mode = props.mode;
    let raw_content = props.raw_content.clone();
    let doc = &props.document;
    let t = props.language.strings();
    let zoom_factor = f64::from(props.zoom_level) / 100.0;
    let zoom_style = format!("zoom: {zoom_factor};");

    let line_count = raw_content.lines().count().max(1);
    let line_numbers: String = (1..=line_count)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let container_class = if props.is_full_width {
        "viewer-container full-width mx-auto max-w-[1280px] w-full pt-4 pb-12"
    } else {
        "viewer-container reading-width mx-auto max-w-[860px] w-full pt-4 pb-12"
    };

    rsx! {
        div {
            class: "editor-wrapper flex-1 h-full flex flex-col relative overflow-hidden bg-[var(--bg-app)]",

            // Top Editor Formatting Toolbar
            EditorToolbar {
                store: store,
                mode: mode,
            }

            // Main Editor Workspace based on active mode
            div {
                class: "editor-workspace-area flex-1 flex overflow-hidden relative",

                // 1. SPLIT PREVIEW MODE (Side-by-side Editor & Live Preview)
                if mode == DocumentMode::Split {
                    div {
                        class: "editor-split-container flex-1 flex w-full h-full overflow-hidden",

                        // Left Pane: Source Code Editor
                        div {
                            class: "split-editor-pane flex-1 h-full flex flex-col border-r border-[var(--border-color)] overflow-hidden bg-[var(--bg-app)]",
                            div {
                                class: "editor-pane-header h-6.5 px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] text-[10.5px] uppercase font-mono font-bold tracking-wider text-[var(--text-muted)] flex items-center justify-between select-none shrink-0",
                                span { "Markdown Source" }
                                span { "{line_count} lines" }
                            }
                            div {
                                class: "source-code-editor flex-1 flex h-full overflow-hidden relative font-mono text-sm",
                                // Line Numbers Gutter
                                div {
                                    id: "source-line-gutter",
                                    class: "editor-gutter w-12 py-3 px-1.5 text-right text-[var(--text-muted)] opacity-40 bg-[var(--bg-subtle)]/50 select-none overflow-hidden shrink-0 font-mono text-xs leading-relaxed border-r border-[var(--border-subtle)] pointer-events-none whitespace-pre",
                                    "{line_numbers}"
                                }
                                // Main Textarea
                                textarea {
                                    id: "source-markdown-textarea",
                                    class: "editor-textarea flex-1 h-full w-full py-3 px-3.5 bg-transparent text-[var(--text-main)] caret-[var(--accent)] font-mono text-xs leading-relaxed border-0 outline-none resize-none overflow-y-auto whitespace-pre-wrap",
                                    value: "{raw_content}",
                                    placeholder: "{t.editor.source_placeholder}",
                                    spellcheck: false,
                                    onscroll: move |_| {
                                        dioxus::prelude::document::eval("window.onEditorSourceScroll && window.onEditorSourceScroll();");
                                    },
                                    oninput: move |evt| {
                                        let val = evt.value();
                                        store.write().update_active_tab_content(val);
                                    },
                                    onkeydown: move |evt| {
                                        let key = evt.key();
                                        if key == Key::Tab {
                                            dioxus::prelude::document::eval("window.handleTextareaTab && window.handleTextareaTab(event);");
                                        }
                                    },
                                }
                            }
                        }

                        // Right Pane: Live Rendered HTML Markdown Preview
                        div {
                            class: "split-preview-pane flex-1 h-full flex flex-col overflow-hidden bg-[var(--reader-glass-bg)]",
                            div {
                                class: "editor-pane-header h-6.5 px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] text-[10.5px] uppercase font-mono font-bold tracking-wider text-[var(--text-muted)] flex items-center justify-between select-none shrink-0",
                                span { "Live Preview" }
                                span { class: "text-[var(--accent)] font-semibold", "{doc.word_count} words" }
                            }
                            div {
                                id: "split-preview-scroll-area",
                                class: "flex-1 h-full overflow-y-auto overflow-x-hidden p-6",
                                style: "{zoom_style}",
                                onscroll: move |_| {
                                    dioxus::prelude::document::eval("window.onSplitPreviewScroll && window.onSplitPreviewScroll();");
                                },
                                div {
                                    class: "{container_class}",
                                    if let Some(ref meta) = doc.metadata {
                                        FrontmatterCard {
                                            metadata: meta.clone(),
                                            language: props.language,
                                        }
                                    }
                                    article {
                                        class: "markdown-body leading-relaxed",
                                        dangerous_inner_html: "{doc.html_content}",
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. FULL WIDTH SOURCE EDITOR MODE
                if mode == DocumentMode::Source {
                    div {
                        class: "editor-source-container flex-1 h-full flex flex-col overflow-hidden bg-[var(--bg-app)]",
                        div {
                            class: "source-code-editor flex-1 flex h-full overflow-hidden relative font-mono",
                            // Line Numbers Gutter
                            div {
                                id: "source-line-gutter",
                                class: "editor-gutter w-14 py-4 px-2 text-right text-[var(--text-muted)] opacity-50 bg-[var(--bg-subtle)]/40 select-none overflow-hidden shrink-0 font-mono text-xs leading-relaxed border-r border-[var(--border-color)] pointer-events-none whitespace-pre",
                                "{line_numbers}"
                            }
                            // Main Textarea
                            textarea {
                                id: "source-markdown-textarea",
                                class: "editor-textarea flex-1 h-full w-full py-4 px-5 bg-transparent text-[var(--text-main)] caret-[var(--accent)] font-mono text-sm leading-relaxed border-0 outline-none resize-none overflow-y-auto whitespace-pre-wrap",
                                value: "{raw_content}",
                                placeholder: "{t.editor.source_placeholder}",
                                spellcheck: false,
                                onscroll: move |_| {
                                    dioxus::prelude::document::eval("window.onEditorSourceScroll && window.onEditorSourceScroll();");
                                },
                                oninput: move |evt| {
                                    let val = evt.value();
                                    store.write().update_active_tab_content(val);
                                },
                                onkeydown: move |evt| {
                                    let key = evt.key();
                                    if key == Key::Tab {
                                        dioxus::prelude::document::eval("window.handleTextareaTab && window.handleTextareaTab(event);");
                                    }
                                },
                            }
                        }
                    }
                }

                // 3. VISUAL WYSIWYG EDITABLE MODE
                if mode == DocumentMode::Wysiwyg {
                    div {
                        class: "editor-wysiwyg-container flex-1 h-full flex flex-col overflow-y-auto overflow-x-hidden p-6 bg-[var(--reader-glass-bg)]",
                        style: "{zoom_style}",
                        div {
                            class: "{container_class}",
                            if let Some(ref meta) = doc.metadata {
                                FrontmatterCard {
                                    metadata: meta.clone(),
                                    language: props.language,
                                }
                            }
                            // Interactive ContentEditable Visual Canvas
                            div {
                                id: "wysiwyg-editor-surface",
                                class: "wysiwyg-editor-surface markdown-body leading-relaxed outline-none min-h-[500px] p-2 rounded-xl focus:ring-1 focus:ring-[var(--accent)]/30 transition-all",
                                contenteditable: "true",
                                spellcheck: "true",
                                dangerous_inner_html: "{doc.html_content}",
                                oninput: move |_| {
                                    // Serialize edited HTML to clean Markdown and update store
                                    dioxus::prelude::document::eval(
                                        r"
                                        if (window.serializeWysiwygToMarkdown) {
                                            const md = window.serializeWysiwygToMarkdown();
                                            if (md !== null && md !== undefined) {
                                                window._lastWysiwygMd = md;
                                            }
                                        }
                                        ",
                                    );
                                },
                                onblur: move |_| {
                                    dioxus::prelude::document::eval(
                                        r"
                                        if (window.serializeWysiwygToMarkdown) {
                                            const md = window.serializeWysiwygToMarkdown();
                                            if (md !== null && md !== undefined) {
                                                window._lastWysiwygMd = md;
                                            }
                                        }
                                        ",
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
