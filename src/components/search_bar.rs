use crate::types::Language;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    pub language: Language,
    pub on_close: EventHandler<()>,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let mut search_text = use_signal(String::new);
    let t = props.language.strings();

    use_effect(move || {
        // Auto focus and select input when search opens
        dioxus::prelude::document::eval(
            r"
            setTimeout(() => {
                const input = document.querySelector('.search-text-input');
                if (input) { input.focus(); input.select(); }
            }, 30);
            ",
        );
    });

    rsx! {
        div {
            class: "floating-search-bar absolute top-12 right-6 flex items-center bg-[var(--bg-surface)] border border-[var(--border-color)] shadow-2xl rounded-lg px-2.5 py-1.5 gap-2 z-50 animate-fade-in",
            onmousedown: move |evt| evt.stop_propagation(),
            svg {
                class: "search-input-icon text-[var(--text-muted)] shrink-0",
                view_box: "0 0 24 24",
                width: "15",
                height: "15",
                circle { cx: "11", cy: "11", r: "8", fill: "none", stroke: "currentColor", stroke_width: "2" }
                path { d: "m21 21-4.3-4.3", stroke: "currentColor", stroke_width: "2" }
            }
            input {
                class: "search-text-input bg-transparent border-0 text-[var(--text-main)] text-xs outline-none w-52 placeholder:text-[var(--text-muted)]",
                r#type: "text",
                placeholder: "{t.search_bar.placeholder}",
                value: "{search_text}",
                autofocus: true,
                oninput: move |evt| {
                    let val = evt.value();
                    search_text.set(val.clone());
                    dioxus::prelude::document::eval(&format!("window.highlightSearchMatches && window.highlightSearchMatches({val:?});"));
                },
                onkeydown: move |evt| {
                    let key = evt.key();
                    if key == Key::Escape {
                        dioxus::prelude::document::eval("window.clearSearchHighlights && window.clearSearchHighlights();");
                        props.on_close.call(());
                    } else if key == Key::Enter {
                        if evt.modifiers().shift() {
                            dioxus::prelude::document::eval("window.searchPrevMatch && window.searchPrevMatch();");
                        } else {
                            dioxus::prelude::document::eval("window.searchNextMatch && window.searchNextMatch();");
                        }
                    }
                }
            }
            span {
                id: "search-match-count",
                class: "search-match-count text-xs font-mono text-[var(--text-muted)] text-right whitespace-nowrap min-w-10",
            }
            div { class: "search-divider w-[1px] h-3.5 bg-[var(--border-color)] shrink-0" }
            button {
                class: "search-action-btn flex items-center justify-center w-5 h-5 bg-transparent border-0 text-[var(--text-muted)] cursor-pointer text-xs rounded hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors",
                title: "{t.search_bar.prev_match}",
                onclick: move |_| {
                    dioxus::prelude::document::eval("window.searchPrevMatch && window.searchPrevMatch();");
                },
                svg {
                    view_box: "0 0 24 24",
                    width: "13",
                    height: "13",
                    path { d: "m18 15-6-6-6 6", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                }
            }
            button {
                class: "search-action-btn flex items-center justify-center w-5 h-5 bg-transparent border-0 text-[var(--text-muted)] cursor-pointer text-xs rounded hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors",
                title: "{t.search_bar.next_match}",
                onclick: move |_| {
                    dioxus::prelude::document::eval("window.searchNextMatch && window.searchNextMatch();");
                },
                svg {
                    view_box: "0 0 24 24",
                    width: "13",
                    height: "13",
                    path { d: "m6 9 6 6 6-6", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                }
            }
            button {
                class: "search-close-btn flex items-center justify-center w-5 h-5 bg-transparent border-0 text-[var(--text-muted)] cursor-pointer text-xs rounded hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-colors",
                title: "{t.search_bar.close_search}",
                onclick: move |_| {
                    dioxus::prelude::document::eval("window.clearSearchHighlights && window.clearSearchHighlights();");
                    props.on_close.call(());
                },
                svg {
                    view_box: "0 0 24 24",
                    width: "13",
                    height: "13",
                    path { d: "M18 6 6 18M6 6l12 12", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round" }
                }
            }
        }
    }
}
