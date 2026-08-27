use crate::types::Language;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ZenExitButtonProps {
    pub language: Language,
    pub on_exit: EventHandler<()>,
}

#[component]
pub fn ZenExitButton(props: ZenExitButtonProps) -> Element {
    let t = props.language.strings();

    rsx! {
        button {
            class: "floating-zen-exit-btn absolute top-4 right-6 inline-flex items-center gap-2 bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-main)] px-3.5 py-1.5 rounded-full shadow-lg text-xs font-medium cursor-pointer z-50 opacity-85 hover:opacity-100 hover:bg-[var(--accent)] hover:text-white hover:border-[var(--accent)] hover:-translate-y-0.5 transition-all duration-150",
            title: "{t.zen.exit_tooltip}",
            onclick: move |_| props.on_exit.call(()),
            svg {
                class: "zen-exit-icon shrink-0",
                view_box: "0 0 24 24",
                width: "14",
                height: "14",
                path { d: "M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3", fill: "none", stroke: "currentColor", stroke_width: "2" }
            }
            span { "{t.zen.exit_button}" }
        }
    }
}
