mod source;
mod wysiwyg;

use crate::components::Hint;
use crate::state::AppStore;
use crate::types::DocumentMode;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCornerUpLeft, LdCornerUpRight, LdSave, LdSparkles};
use dioxus_primitives::{ContentAlign, ContentSide};
use source::SourceFormattingTools;
use wysiwyg::WysiwygFormattingTools;

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
    let is_markdown = format.is_markdown();
    let validation_error = active_tab.and_then(|t| t.parsed.validation_error.clone());

    rsx! {
        div {
            class: "editor-toolbar flex items-stretch justify-between px-0 py-0 bg-[var(--bg-surface)] border-b border-[var(--border-color)] overflow-visible select-none shrink-0 z-30",

            div {
                class: "flex items-stretch h-full min-w-0",

                if is_config {
                    div {
                        class: "inline-flex items-stretch h-full",
                        span {
                            class: "px-2 text-[10.5px] uppercase font-mono font-bold tracking-wider text-[var(--text-muted)] flex items-center h-full shrink-0",
                            "{format.label()}"
                        }
                        if format == crate::types::DocumentFormat::Json {
                            Hint {
                                text: t.editor.insert_json_object,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Start,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('{\\n  \"key\": \"value\"\\n}');");
                                    },
                                    "{{ }}"
                                }
                            }
                            Hint {
                                text: t.editor.insert_json_array,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Center,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('[\\n  \"value\"\\n]');");
                                    },
                                    "[ ... ]"
                                }
                            }
                            Hint {
                                text: t.editor.insert_json_kv,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Center,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\"key\": \"value\",\\n');");
                                    },
                                    "\":\""
                                }
                            }
                        }
                        if format == crate::types::DocumentFormat::Toml {
                            Hint {
                                text: t.editor.insert_toml_section,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Start,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\\n[section]\\nkey = \"value\"\\n');");
                                    },
                                    "[section]"
                                }
                            }
                            Hint {
                                text: t.editor.insert_toml_array_table,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Center,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('\\n[[array_table]]\\nname = \"item\"\\n');");
                                    },
                                    "[[array]]"
                                }
                            }
                        }
                        if format == crate::types::DocumentFormat::Yaml {
                            Hint {
                                text: t.editor.insert_yaml_kv,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Start,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('key: value\\n');");
                                    },
                                    "key: val"
                                }
                            }
                            Hint {
                                text: t.editor.insert_yaml_list,
                                side: ContentSide::Bottom,
                                align: ContentAlign::Center,
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    class: "toolbar-action-btn-flat",
                                    onclick: move |_| {
                                        dioxus::prelude::document::eval("window.insertSourceSnippet && window.insertSourceSnippet('- item\\n');");
                                    },
                                    "- list"
                                }
                            }
                        }
                    }

                    div {
                        class: "inline-flex items-stretch h-full px-2 text-xs font-medium select-none",
                        if let Some(ref err) = validation_error {
                            span {
                                class: "text-amber-500 flex items-center gap-1 px-2",
                                title: "{err}",
                                "⚠️ {t.editor.invalid_syntax}"
                            }
                        } else {
                            span {
                                class: "text-emerald-500 flex items-center gap-1 px-2",
                                "✓ {t.editor.valid_syntax}"
                            }
                        }
                    }
                }

                if is_markdown {
                    if is_wysiwyg {
                        WysiwygFormattingTools { t }
                    } else {
                        SourceFormattingTools { t }
                    }
                }
            }

            div {
                class: "flex items-stretch h-full shrink-0",

                Hint {
                    text: t.editor.format_document,
                    side: ContentSide::Bottom,
                    align: ContentAlign::Center,
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "toolbar-action-btn-flat",
                        onclick: move |_| {
                            store.write().format_active_tab();
                        },
                        Icon { width: 13, height: 13, icon: LdSparkles, class: "text-[var(--accent)]" }
                        span { class: "hidden sm:inline text-[11px]", "Format" }
                    }
                }

                div { class: "toolbar-inner-sep" }

                div {
                    class: "inline-flex items-stretch h-full",
                    Hint {
                        text: t.editor.undo,
                        side: ContentSide::Bottom,
                        align: ContentAlign::Center,
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "toolbar-action-btn-flat",
                            onclick: move |_| {
                                dioxus::prelude::document::eval("window.editorUndo && window.editorUndo();");
                            },
                            Icon { width: 13, height: 13, icon: LdCornerUpLeft }
                        }
                    }
                    Hint {
                        text: t.editor.redo,
                        side: ContentSide::Bottom,
                        align: ContentAlign::Center,
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            class: "toolbar-action-btn-flat",
                            onclick: move |_| {
                                dioxus::prelude::document::eval("window.editorRedo && window.editorRedo();");
                            },
                            Icon { width: 13, height: 13, icon: LdCornerUpRight }
                        }
                    }
                }

                div { class: "toolbar-inner-sep" }

                Hint {
                    text: t.toolbar.save_file,
                    side: ContentSide::Bottom,
                    align: ContentAlign::End,
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: if is_dirty { "toolbar-action-btn-flat toolbar-dirty-flat" } else { "toolbar-action-btn-flat" },
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
}
