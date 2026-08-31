use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::types::Language;
use dioxus::prelude::*;
use dioxus_primitives::ContentSide;
use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipTrigger};

#[derive(Props, Clone, PartialEq)]
pub struct ZenExitButtonProps {
    pub language: Language,
    pub on_exit: EventHandler<()>,
}

#[component]
pub fn ZenExitButton(props: ZenExitButtonProps) -> Element {
    let t = props.language.strings();

    rsx! {
        Tooltip {
            TooltipTrigger {
                r#as: move |attrs| rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        class: "floating-zen-exit-btn absolute bottom-5 right-6 z-50 opacity-85 hover:opacity-100 hover:-translate-y-0.5 shadow-lg rounded-full",
                        onclick: move |_| props.on_exit.call(()),
                        attributes: attrs,
                        svg {
                            class: "zen-exit-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { "{t.zen.exit_button}" }
                    }
                },
            }
            TooltipContent {
                side: ContentSide::Top,
                "{t.zen.exit_tooltip}"
            }
        }
    }
}
