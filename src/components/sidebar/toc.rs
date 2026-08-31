use crate::types::TocItem;
use dioxus::prelude::*;

pub const TOC_ROW_HEIGHT: u32 = 28;

#[derive(Props, Clone, PartialEq)]
pub(super) struct TocItemLinkProps {
    pub item: TocItem,
    pub index: usize,
}

#[component]
pub(super) fn TocItemLink(props: TocItemLinkProps) -> Element {
    let item = &props.item;
    let heading_id = item.id.clone();
    let line_js = item.line.map_or_else(|| "null".to_string(), |l| l.to_string());
    let indent_level = (item.level.saturating_sub(1)).min(5);
    let indent_px = indent_level * 14;
    let style = format!("padding-left: {indent_px}px;");
    let is_root = item.level == 1;

    rsx! {
        div {
            class: format!("toc-item toc-tree-item toc-level-{} relative group h-full", item.level),
            style: "{style}",
            "data-heading-id": "{heading_id}",
            "data-toc-index": "{props.index}",
            a {
                class: "toc-link flex items-center gap-2.5 text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] text-xs py-1 px-2 rounded-md transition-all duration-150 no-underline cursor-pointer select-none relative h-full",
                href: "#{heading_id}",
                onclick: move |evt| {
                    evt.prevent_default();
                    let id = heading_id.clone();
                    dioxus::prelude::document::eval(&format!(
                        "window.scrollToSection && window.scrollToSection({id:?}, {line_js});"
                    ));
                },
                span {
                    class: if is_root {
                        "toc-node-bullet root-node rounded-full border border-[var(--accent)] bg-[var(--bg-surface)] shrink-0 transition-all duration-200 z-10"
                    } else {
                        "toc-node-bullet sub-node rounded-full border border-[var(--border-color)] bg-[var(--bg-surface)] shrink-0 transition-all duration-200 z-10"
                    }
                }
                span {
                    class: "toc-title truncate flex-1",
                    "{item.title}"
                }
                if item.level > 2 {
                    span {
                        class: "toc-level-tag text-[9px] font-mono opacity-40 uppercase shrink-0",
                        "H{item.level}"
                    }
                }
            }
        }
    }
}
