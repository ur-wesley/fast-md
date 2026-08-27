use crate::types::{Language, ParsedDocument};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdAlignLeft, LdClock, LdFileCode2, LdFileText};
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct StatusBarProps {
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub document: ParsedDocument,
    pub zoom_level: u32,
    pub language: Language,
}

#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let t = props.language.strings();

    let path_display = props.file_path.as_ref().map_or_else(
        || t.status_bar.memory_doc.to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    let (doc_type, is_code_type) = props.file_path.as_ref().map_or((t.status_bar.markdown_doc, false), |p| {
        p.extension()
            .and_then(|e| e.to_str())
            .map_or((t.status_bar.markdown_doc, false), |ext| match ext.to_ascii_lowercase().as_str() {
                "mdx" => (t.status_bar.mdx_doc, true),
                "md" | "markdown" => (t.status_bar.markdown_doc, false),
                "txt" => (t.status_bar.text_doc, false),
                _ => (t.status_bar.generic_doc, false),
            })
    });

    rsx! {
        footer {
            class: "app-status-bar flex items-center justify-between h-6.5 min-h-[26px] bg-[var(--bg-surface)] border-t border-[var(--border-color)] px-3 text-xs text-[var(--text-muted)] font-mono select-none z-50",
            div {
                class: "status-left-group flex items-center gap-2 min-w-0",
                span {
                    class: "status-item status-title font-semibold text-[var(--text-heading)] max-w-[250px] truncate inline-flex items-center gap-1.5",
                    title: "{props.title}",
                    if is_code_type {
                        Icon { width: 12, height: 12, icon: LdFileCode2, class: "text-[var(--accent)] shrink-0" }
                    } else {
                        Icon { width: 12, height: 12, icon: LdFileText, class: "text-[var(--accent)] shrink-0" }
                    }
                    span { class: "truncate", "{props.title}" }
                }
                span {
                    class: "status-item status-path max-w-[320px] truncate opacity-75",
                    title: "{path_display}",
                    "({path_display})"
                }
            }
            div {
                class: "status-right-group flex items-center gap-2 shrink-0",
                span { class: "status-item", "{props.document.word_count} {t.status_bar.words_suffix}" }
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

