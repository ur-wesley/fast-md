use crate::components::Hint;
use crate::i18n::Translations;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdBold, LdCheck, LdCode, LdFileCode2, LdHeading1, LdHeading2, LdHeading3, LdImage, LdItalic,
    LdLink, LdList, LdListOrdered, LdMessageSquare, LdQuote, LdStrikethrough, LdTable,
};
use dioxus_primitives::{ContentAlign, ContentSide};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct WysiwygFormattingToolsProps {
    pub t: &'static Translations,
}

#[component]
pub fn WysiwygFormattingTools(props: WysiwygFormattingToolsProps) -> Element {
    let t = props.t;

    rsx! {
        div {
            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
            Hint {
                text: t.editor.bold,
                side: ContentSide::Bottom,
                align: ContentAlign::Start,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-bold",
                    "data-tool": "bold",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('bold');");
                    },
                    Icon { width: 13, height: 13, icon: LdBold }
                }
            }
            Hint {
                text: t.editor.italic,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-italic",
                    "data-tool": "italic",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('italic');");
                    },
                    Icon { width: 13, height: 13, icon: LdItalic }
                }
            }
            Hint {
                text: t.editor.strikethrough,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-strikethrough",
                    "data-tool": "strikethrough",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('strikeThrough');");
                    },
                    Icon { width: 13, height: 13, icon: LdStrikethrough }
                }
            }
            Hint {
                text: t.editor.inline_code,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-code",
                    "data-tool": "code",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwygCode && window.formatWysiwygCode();");
                    },
                    Icon { width: 13, height: 13, icon: LdCode }
                }
            }
        }

        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

        div {
            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
            Hint {
                text: t.editor.h1,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-h1",
                    "data-tool": "h1",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h1');");
                    },
                    Icon { width: 14, height: 14, icon: LdHeading1 }
                }
            }
            Hint {
                text: t.editor.h2,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-h2",
                    "data-tool": "h2",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h2');");
                    },
                    Icon { width: 14, height: 14, icon: LdHeading2 }
                }
            }
            Hint {
                text: t.editor.h3,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-h3",
                    "data-tool": "h3",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwygHeading && window.formatWysiwygHeading('h3');");
                    },
                    Icon { width: 14, height: 14, icon: LdHeading3 }
                }
            }
        }

        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

        div {
            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
            Hint {
                text: t.editor.bullet_list,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-ul",
                    "data-tool": "ul",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('insertUnorderedList');");
                    },
                    Icon { width: 13, height: 13, icon: LdList }
                }
            }
            Hint {
                text: t.editor.numbered_list,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-ol",
                    "data-tool": "ol",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwyg && window.formatWysiwyg('insertOrderedList');");
                    },
                    Icon { width: 13, height: 13, icon: LdListOrdered }
                }
            }
            Hint {
                text: t.editor.task_list,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-task",
                    "data-tool": "task",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.insertWysiwygTaskList && window.insertWysiwygTaskList();");
                    },
                    Icon { width: 13, height: 13, icon: LdCheck }
                }
            }
            Hint {
                text: t.editor.blockquote,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-quote",
                    "data-tool": "quote",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.formatWysiwygBlockquote && window.formatWysiwygBlockquote();");
                    },
                    Icon { width: 13, height: 13, icon: LdQuote }
                }
            }
        }

        div { class: "w-[1px] h-4 bg-[var(--border-color)] mx-0.5" }

        div {
            class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-0.5",
            Hint {
                text: t.editor.code_block,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-codeblock",
                    "data-tool": "codeblock",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.insertWysiwygCodeBlock && window.insertWysiwygCodeBlock();");
                    },
                    Icon { width: 13, height: 13, icon: LdFileCode2 }
                }
            }
            Hint {
                text: t.editor.table,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-table",
                    "data-tool": "table",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.insertWysiwygTable && window.insertWysiwygTable();");
                    },
                    Icon { width: 13, height: 13, icon: LdTable }
                }
            }
            Hint {
                text: t.editor.link,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-link",
                    "data-tool": "link",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.promptWysiwygLink && window.promptWysiwygLink();");
                    },
                    Icon { width: 13, height: 13, icon: LdLink }
                }
            }
            Hint {
                text: t.editor.image,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-image",
                    "data-tool": "image",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.promptWysiwygImage && window.promptWysiwygImage();");
                    },
                    Icon { width: 13, height: 13, icon: LdImage }
                }
            }
            Hint {
                text: t.editor.callout,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    id: "toolbar-btn-callout",
                    "data-tool": "callout",
                    onclick: move |_| {
                        dioxus::prelude::document::eval("window.insertWysiwygCallout && window.insertWysiwygCallout('info');");
                    },
                    Icon { width: 13, height: 13, icon: LdMessageSquare }
                }
            }
        }
    }
}
