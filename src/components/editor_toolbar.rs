use crate::state::AppStore;
use crate::types::DocumentMode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdBold, LdCheck, LdCode, LdCornerUpLeft, LdCornerUpRight, LdFileCode2, LdHeading1, LdHeading2,
    LdHeading3, LdImage, LdItalic, LdLink, LdList, LdListOrdered, LdMessageSquare, LdQuote, LdSave,
    LdSparkles, LdStrikethrough, LdTable,
};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct EditorToolbarProps {
    pub store: Signal<AppStore>,
    pub mode: DocumentMode,
}

#[component]
pub fn EditorToolbar(props: EditorToolbarProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();
    let active_tab = store_read.active_tab();
    let is_dirty = active_tab.is_some_and(|tab| tab.is_dirty);
    let is_wysiwyg = props.mode == DocumentMode::Wysiwyg;
    let format = active_tab.map_or(crate::types::DocumentFormat::Markdown, |t| t.parsed.format);
    let is_config = format.is_config();
    let validation_error = active_tab.and_then(|t| t.parsed.validation_error.clone());

    rsx! {
        div {
            class: "editor-toolbar flex items-center justify-between px-3 py-1.5 bg-[var(--bg-surface)] border-b border-[var(--border-color)] overflow-x-auto select-none gap-1 shrink-0 z-30",

            // Left Formatting Actions
            div {
                class: "flex items-center gap-1 flex-wrap min-w-0",

                if is_config {
                    // Config Editing Tools Group
                    div {
                        class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        span {
                            class: "px-2 py-0.5 text-[11px] font-mono font-semibold text-[var(--accent)] bg-[var(--bg-app)] rounded border border-[var(--border-color)]",
                            "{format.label()}"
                        }
                        if format == crate::types::DocumentFormat::Json {
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_json_object}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('{\\n  \"key\": \"value\"\\n}');");
                                },
                                "{{ }}"
                            }
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_json_array}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('[\\n  \"value\"\\n]');");
                                },
                                "[ ... ]"
                            }
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_json_kv}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\"key\": \"value\",\\n');");
                                },
                                "\":\""
                            }
                        }
                        if format == crate::types::DocumentFormat::Toml {
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_toml_section}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\\n[section]\\nkey = \"value\"\\n');");
                                },
                                "[section]"
                            }
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "Insert Array Table",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\\n[[array_table]]\\nname = \"item\"\\n');");
                                },
                                "[[array]]"
                            }
                        }
                        if format == crate::types::DocumentFormat::Yaml {
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_yaml_kv}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('key: value\\n');");
                                },
                                "key: val"
                            }
                            button {
                                class: "toolbar-action-btn h-6 px-2 flex items-center gap-1 rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-xs border-0 bg-transparent font-medium",
                                title: "{t.editor.insert_yaml_list}",
                                onclick: move |_| {
                                    dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('- item\\n');");
                                },
                                "- list"
                            }
                        }
                    }

                    // Config Validation Indicator
                    div {
                        class: "inline-flex items-center px-2 py-1 rounded text-xs gap-1.5 font-medium select-none",
                        if let Some(ref err) = validation_error {
                            span {
                                class: "text-amber-500 flex items-center gap-1 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/30",
                                title: "{err}",
                                "⚠️ {t.editor.invalid_syntax}"
                            }
                        } else {
                            span {
                                class: "text-emerald-500 flex items-center gap-1 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/30",
                                "✓ {t.editor.valid_syntax}"
                            }
                        }
                    }
                }

                if !is_config {      // Standard Markdown Formatting Tools
                    // Text Styling Group (Bold, Italic, Strikethrough, Inline Code)
                        div {
                            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
                            button {
                                id: "toolbar-btn-bold",
                                "data-tool": "bold",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.bold}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('bold');");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('**', '**', 'bold text');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdBold }
                            }
                            button {
                                id: "toolbar-btn-italic",
                                "data-tool": "italic",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.italic}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('italic');");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('*', '*', 'italic text');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdItalic }
                            }
                            button {
                                id: "toolbar-btn-strikethrough",
                                "data-tool": "strikethrough",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.strikethrough}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('strikeThrough');");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('~~', '~~', 'strikethrough text');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdStrikethrough }
                            }
                            button {
                                id: "toolbar-btn-code",
                                "data-tool": "code",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.inline_code}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwygCode && window.formatWysiwygCode();");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('`', '`', 'code');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdCode }
                            }
                        }

                        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

                        // Headings Group (H1, H2, H3)
                        div {
                            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
                            button {
                                id: "toolbar-btn-h1",
                                "data-tool": "h1",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.h1}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h1');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('# ');");
                                    }
                                },
                                Icon { width: 14, height: 14, icon: LdHeading1 }
                            }
                            button {
                                id: "toolbar-btn-h2",
                                "data-tool": "h2",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.h2}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h2');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('## ');");
                                    }
                                },
                                Icon { width: 14, height: 14, icon: LdHeading2 }
                            }
                            button {
                                id: "toolbar-btn-h3",
                                "data-tool": "h3",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.h3}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h3');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('### ');");
                                    }
                                },
                                Icon { width: 14, height: 14, icon: LdHeading3 }
                            }
                        }

                        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

                        // Lists & Blockquotes Group
                        div {
                            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
                            button {
                                id: "toolbar-btn-ul",
                                "data-tool": "ul",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.bullet_list}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('insertUnorderedList');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('- ');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdList }
                            }
                            button {
                                id: "toolbar-btn-ol",
                                "data-tool": "ol",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.numbered_list}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('insertOrderedList');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('1. ');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdListOrdered }
                            }
                            button {
                                id: "toolbar-btn-task",
                                "data-tool": "task",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.task_list}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.insertWysiwygTaskList && window.insertWysiwygTaskList();");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('- [ ] ');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdCheck }
                            }
                            button {
                                id: "toolbar-btn-quote",
                                "data-tool": "quote",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.blockquote}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.formatWysiwygBlockquote && window.formatWysiwygBlockquote();");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceLinePrefix && window.insertSourceLinePrefix('> ');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdQuote }
                            }
                        }

                        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

                        // Insert Components (Codeblock, Table, Link, Image, MDX Callout)
                        div {
                            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
                            button {
                                id: "toolbar-btn-codeblock",
                                "data-tool": "codeblock",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.code_block}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.insertWysiwygCodeBlock && window.insertWysiwygCodeBlock();");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('```rust\\n// Code here\\n```\\n');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdFileCode2 }
                            }
                            button {
                                id: "toolbar-btn-table",
                                "data-tool": "table",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.table}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.insertWysiwygTable && window.insertWysiwygTable();");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('| Header 1 | Header 2 |\\n| :--- | :--- |\\n| Value 1 | Value 2 |\\n\\n');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdTable }
                            }
                            button {
                                id: "toolbar-btn-link",
                                "data-tool": "link",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.link}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.promptWysiwygLink && window.promptWysiwygLink();");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('[', '](https://)', 'link text');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdLink }
                            }
                            button {
                                id: "toolbar-btn-image",
                                "data-tool": "image",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.image}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.promptWysiwygImage && window.promptWysiwygImage();");
                                    } else {
                                        dioxus::prelude::document::eval("window.wrapSourceSelection && window.wrapSourceSelection('![', '](https://)', 'alt text');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdImage }
                            }
                            button {
                                id: "toolbar-btn-callout",
                                "data-tool": "callout",
                                class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                                title: "{t.editor.callout}",
                                onclick: move |_| {
                                    if is_wysiwyg {
                                        dioxus::prelude::document::eval("window.insertWysiwygCallout && window.insertWysiwygCallout('info');");
                                    } else {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('<Callout type=\"info\">\\n  Callout note description.\\n</Callout>\\n\\n');");
                                    }
                                },
                                Icon { width: 13, height: 13, icon: LdMessageSquare }
                            }
                        }
                    }
                }

            // Right Group: Format Document, Undo, Redo & Save Button
            div {
                class: "flex items-center gap-1.5 shrink-0",

                // Format Document Button
                button {
                    class: "toolbar-action-btn h-7 px-2 flex items-center justify-center gap-1 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-[var(--text-muted)] hover:text-[var(--accent)] hover:border-[var(--accent)] cursor-pointer transition-colors text-xs font-medium",
                    title: "{t.editor.format_document}",
                    onclick: move |_| {
                        store.write().format_active_tab();
                    },
                    Icon { width: 13, height: 13, icon: LdSparkles, class: "text-[var(--accent)]" }
                    span { class: "hidden sm:inline text-[11px]", "Format" }
                }

                // Undo / Redo
                div {
                    class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
                    button {
                        class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                        title: "{t.editor.undo}",
                        onclick: move |_| {
                            dioxus::prelude::document::eval("window.editorUndo && window.editorUndo();");
                        },
                        Icon { width: 13, height: 13, icon: LdCornerUpLeft }
                    }
                    button {
                        class: "toolbar-action-btn w-7 h-7 flex items-center justify-center rounded text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors border-0 bg-transparent",
                        title: "{t.editor.redo}",
                        onclick: move |_| {
                            dioxus::prelude::document::eval("window.editorRedo && window.editorRedo();");
                        },
                        Icon { width: 13, height: 13, icon: LdCornerUpRight }
                    }
                }

                // Save Document Button
                button {
                    class: if is_dirty {
                        "h-7 px-2.5 rounded-lg bg-[var(--accent)] text-white text-xs font-semibold cursor-pointer border-0 inline-flex items-center gap-1.5 shadow-sm hover:opacity-90 transition-all"
                    } else {
                        "h-7 px-2.5 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-[var(--text-muted)] text-xs font-medium cursor-pointer inline-flex items-center gap-1.5 hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all"
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
                                    if let Some(save_path) = crate::services::fs::pick_save_file_async(&title).await {
                                        let tab_id = active_tab.id;
                                        let _ = store.write().save_tab_with_path(tab_id, save_path);
                                    }
                                }
                            }
                        });
                    },
                    Icon { width: 13, height: 13, icon: LdSave }
                    span { if is_dirty { "{t.toolbar.unsaved}" } else { "{t.toolbar.saved}" } }
                    if is_dirty {
                        span { class: "w-1.5 h-1.5 rounded-full bg-amber-300 animate-pulse" }
                    }
                }
            }
        }
    }
}
