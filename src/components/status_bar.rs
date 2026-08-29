use crate::types::{DocumentMode, Language, ParsedDocument};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdAlignLeft, LdBookOpen, LdClock, LdColumns2, LdFileCode2, LdFileText, LdSparkles,
};
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub document: ParsedDocument,
    pub raw_content: String,
    pub mode: DocumentMode,
    pub is_dirty: bool,
    pub zoom_level: u32,
    pub language: Language,
    pub on_cycle_mode: EventHandler<()>,
}

#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let t = props.language.strings();
    let line_count = props.raw_content.lines().count().max(1);
    let char_count = props.raw_content.chars().count();

    let path_display = props.file_path.as_ref().map_or_else(
        || t.status_bar.memory_doc.to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    let (doc_type, is_code_type) = if props.document.format.is_config() {
        (props.document.format.label(), true)
    } else {
        props.file_path.as_ref().map_or((t.status_bar.markdown_doc, false), |p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map_or((t.status_bar.markdown_doc, false), |ext| match ext.to_ascii_lowercase().as_str() {
                    "mdx" => (t.status_bar.mdx_doc, true),
                    "md" | "markdown" => (t.status_bar.markdown_doc, false),
                    "txt" => (t.status_bar.text_doc, false),
                    _ => (t.status_bar.generic_doc, false),
                })
        })
    };

    let mode_label = match props.mode {
        DocumentMode::View => t.toolbar.mode_view,
        DocumentMode::Split => t.toolbar.mode_split,
        DocumentMode::Wysiwyg => t.toolbar.mode_wysiwyg,
        DocumentMode::Source => t.toolbar.mode_source,
    };

    rsx! {
        footer {
            class: "app-status-bar flex items-center justify-between h-6.5 min-h-[26px] bg-[var(--bg-surface)] border-t border-[var(--border-color)] px-3 text-xs text-[var(--text-muted)] font-mono select-none z-50",
            div {
                class: "status-left-group flex items-center gap-2 min-w-0",

                // Clickable Mode Badge
                button {
                    class: "status-mode-pill inline-flex items-center gap-1 bg-[var(--bg-subtle)] hover:bg-[var(--bg-hover)] text-[var(--accent)] px-1.5 py-0.5 rounded text-[10.5px] font-semibold cursor-pointer border border-[var(--border-subtle)] transition-all",
                    title: "Click to cycle mode (Ctrl+E)",
                    onclick: move |_| props.on_cycle_mode.call(()),
                    match props.mode {
                        DocumentMode::View => rsx! { Icon { width: 11, height: 11, icon: LdBookOpen } },
                        DocumentMode::Split => rsx! { Icon { width: 11, height: 11, icon: LdColumns2 } },
                        DocumentMode::Wysiwyg => rsx! { Icon { width: 11, height: 11, icon: LdSparkles } },
                        DocumentMode::Source => rsx! { Icon { width: 11, height: 11, icon: LdFileCode2 } },
                    }
                    span { "{mode_label}" }
                }

                // Dirty State Pill
                if props.is_dirty {
                    span {
                        class: "status-dirty-badge bg-amber-500/15 border border-amber-500/40 text-amber-400 px-1.5 py-0.5 rounded text-[10px] uppercase font-bold flex items-center gap-1",
                        span { class: "w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" }
                        "{t.status_bar.status_unsaved}"
                    }
                } else {
                    span {
                        class: "status-dirty-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] opacity-60 px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold",
                        "{t.status_bar.status_saved}"
                    }
                }

                // Syntax Error Pill if invalid config
                if let Some(ref err) = props.document.validation_error {
                    span {
                        class: "status-error-badge bg-red-500/15 border border-red-500/40 text-red-400 px-1.5 py-0.5 rounded text-[10px] uppercase font-bold flex items-center gap-1 cursor-help",
                        title: "{err}",
                        "⚠️ {t.editor.invalid_syntax}"
                    }
                }

                span {
                    class: "status-item status-title font-semibold text-[var(--text-heading)] max-w-[220px] truncate inline-flex items-center gap-1.5",
                    title: "{props.title}",
                    if is_code_type {
                        Icon { width: 12, height: 12, icon: LdFileCode2, class: "text-[var(--accent)] shrink-0" }
                    } else {
                        Icon { width: 12, height: 12, icon: LdFileText, class: "text-[var(--accent)] shrink-0" }
                    }
                    span { class: "truncate", "{props.title}" }
                }
                span {
                    class: "status-item status-path max-w-[260px] truncate opacity-60 text-[11px]",
                    title: "{path_display}",
                    "({path_display})"
                }
            }
            div {
                class: "status-right-group flex items-center gap-2 shrink-0",
                span { class: "status-item", "{line_count} {t.status_bar.lines_suffix}" }
                span { class: "status-item", "{props.document.word_count} {t.status_bar.words_suffix}" }
                span { class: "status-item", "{char_count} {t.status_bar.chars_suffix}" }
                span {
                    class: "status-item inline-flex items-center gap-1",
                    Icon { width: 11, height: 11, icon: LdClock, class: "opacity-60" }
                    "~{props.document.reading_time_minutes} {t.status_bar.min_suffix}"
                }
                span {
                    class: "status-item inline-flex items-center gap-1",
                    Icon { width: 11, height: 11, icon: LdAlignLeft, class: "opacity-60" }
                    "{props.document.toc.len()} {t.status_bar.sec_suffix}"
                }
                span { class: "status-item status-zoom font-semibold text-[var(--accent)]", "{props.zoom_level}%" }
                span { class: "status-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold", "{doc_type}" }
                span { class: "status-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold", "UTF-8" }
            }
        }
    }
}


