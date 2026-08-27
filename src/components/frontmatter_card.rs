use crate::types::DocMetadata;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCalendar, LdTag, LdUser};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct FrontmatterCardProps {
    pub metadata: DocMetadata,
}

#[component]
pub fn FrontmatterCard(props: FrontmatterCardProps) -> Element {
    let meta = &props.metadata;

    rsx! {
        div {
            class: "frontmatter-card bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg p-4 mb-7 shadow-sm",
            if let Some(ref title) = meta.title {
                h1 { class: "frontmatter-title text-2xl font-bold text-[var(--text-heading)] mb-1.5", "{title}" }
            }
            if let Some(ref desc) = meta.description {
                p { class: "frontmatter-desc text-[var(--text-muted)] text-sm mb-3 leading-relaxed", "{desc}" }
            }
            div {
                class: "frontmatter-meta-row flex flex-wrap items-center gap-2",
                if let Some(ref author) = meta.author {
                    div {
                        class: "frontmatter-badge inline-flex items-center bg-[var(--bg-subtle)] px-2 py-0.5 rounded text-xs text-[var(--text-muted)] gap-1",
                        Icon { width: 12, height: 12, icon: LdUser, class: "text-[var(--accent)] shrink-0" }
                        span { class: "badge-label font-semibold", "Author: " }
                        span { class: "badge-value", "{author}" }
                    }
                }
                if let Some(ref date) = meta.date {
                    div {
                        class: "frontmatter-badge inline-flex items-center bg-[var(--bg-subtle)] px-2 py-0.5 rounded text-xs text-[var(--text-muted)] gap-1",
                        Icon { width: 12, height: 12, icon: LdCalendar, class: "text-[var(--accent)] shrink-0" }
                        span { class: "badge-label font-semibold", "Date: " }
                        span { class: "badge-value", "{date}" }
                    }
                }
                for tag in &meta.tags {
                    div {
                        class: "frontmatter-tag-badge inline-flex items-center bg-[var(--bg-subtle)] text-[var(--accent)] px-2.5 py-0.5 rounded-full text-xs font-medium gap-1",
                        Icon { width: 10, height: 10, icon: LdTag, class: "shrink-0 opacity-75" }
                        span { "{tag}" }
                    }
                }
            }
        }
    }
}

