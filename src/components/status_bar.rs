use crate::types::ParsedDocument;
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
}

#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let path_display = props.file_path.as_ref().map_or_else(
        || "Memory Document".to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    let (doc_type, is_code_type) = props.file_path.as_ref().map_or(("Markdown", false), |p| {
        p.extension()
            .and_then(|e| e.to_str())
            .map_or(("Markdown", false), |ext| match ext.to_ascii_lowercase().as_str() {
                "mdx" => ("MDX Document", true),
                "md" | "markdown" => ("Markdown", false),
                "txt" => ("Plain Text", false),
                _ => ("Document", false),
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
                span { class: "status-item", "{props.document.word_count} words" }
                span {
                    class: "status-item inline-flex items-center gap-1",
                    Icon { width: 11, height: 11, icon: LdClock, class: "opacity-60" }
                    "~{props.document.reading_time_minutes} min"
                }
                span {
                    class: "status-item inline-flex items-center gap-1",
                    Icon { width: 11, height: 11, icon: LdAlignLeft, class: "opacity-60" }
                    "{props.document.toc.len()} sec"
                }
                span { class: "status-item status-zoom font-semibold text-[var(--accent)]", "{props.zoom_level}%" }
                span { class: "status-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold", "{doc_type}" }
                span { class: "status-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold", "UTF-8" }
            }
        }
    }
}

