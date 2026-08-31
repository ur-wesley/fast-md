use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;
use dioxus_primitives::{ContentAlign, ContentSide};

#[derive(Props, Clone, PartialEq)]
pub struct HintProps {
    #[props(into)]
    pub text: String,
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,
    pub children: Element,
}

#[component]
pub fn Hint(props: HintProps) -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                class: "inline-flex h-full items-center justify-center",
                {props.children}
            }
            TooltipContent {
                side: props.side,
                align: props.align,
                "{props.text}"
            }
        }
    }
}
