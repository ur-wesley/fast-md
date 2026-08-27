use crate::components::frontmatter_card::FrontmatterCard;
use crate::types::{Language, ParsedDocument};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct ViewerProps {
    pub document: ParsedDocument,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub sticky_headers: bool,
    pub language: Language,
}

#[component]
pub fn Viewer(props: ViewerProps) -> Element {
    let doc = &props.document;
    let zoom_factor = f64::from(props.zoom_level) / 100.0;
    let zoom_style = format!("zoom: {zoom_factor};");
    let container_class = if props.is_full_width {
        "viewer-container full-width mx-auto max-w-[1280px] w-full pt-6"
    } else {
        "viewer-container reading-width mx-auto max-w-[860px] w-full pt-6"
    };
    let viewer_class = if props.sticky_headers {
        "app-main-viewer has-sticky-headers flex-1 h-full overflow-y-auto overflow-x-hidden scroll-smooth pt-0 pb-8 px-6 bg-[var(--reader-glass-bg)] transition-all duration-150"
    } else {
        "app-main-viewer flex-1 h-full overflow-y-auto overflow-x-hidden scroll-smooth pt-0 pb-8 px-6 bg-[var(--reader-glass-bg)] transition-all duration-150"
    };

    rsx! {
        div {
            class: "viewer-wrapper flex-1 h-full flex flex-col relative overflow-hidden",
            // Sticky top light animated reading progress bar spanning full width of content window
            div {
                class: "viewer-progress-container w-full h-[3px] bg-[var(--bg-subtle)] overflow-hidden shrink-0 z-40 pointer-events-none relative",
                div {
                    id: "viewer-scroll-progress-bar",
                    class: "viewer-progress-bar h-full w-0 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)] transition-all duration-75 relative",
                    div {
                        class: "progress-head-pip absolute top-0 right-0 h-full w-4 bg-white/60 blur-[1px] shadow-[0_0_8px_var(--accent)]",
                    }
                }
            }
            main {
                class: "{viewer_class}",
                id: "viewer-scroll-area",
                style: "{zoom_style}",
                onscroll: move |_| {
                    dioxus::prelude::document::eval("window.onViewerScroll && window.onViewerScroll();");
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
