use crate::components::frontmatter_card::FrontmatterCard;
use crate::types::ParsedDocument;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct ViewerProps {
    pub document: ParsedDocument,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub sticky_headers: bool,
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
        main {
            class: "{viewer_class}",
            id: "viewer-scroll-area",
            style: "{zoom_style}",
            div {
                class: "{container_class}",
                if let Some(ref meta) = doc.metadata {
                    FrontmatterCard {
                        metadata: meta.clone(),
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
