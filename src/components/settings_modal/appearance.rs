use crate::components::Hint;
use crate::state::AppStore;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdPalette, LdSparkles, LdZap};

use super::{accent_presets, catppuccin_themes, classic_themes};

#[derive(Props, Clone, PartialEq)]
pub struct AppearancePaneProps {
    pub store: Signal<AppStore>,
    pub t: &'static crate::i18n::Translations,
    pub active_accent: String,
    pub has_custom_accent: bool,
    #[props(default)]
    pub search_filter: Option<String>,
}

pub fn has_matches(query: &str, t: &'static crate::i18n::Translations) -> bool {
    matches_themes(query, t) || matches_accent(query, t)
}

fn matches_themes(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    let haystacks = [
        "theme", "themes", "design", "farbschema", "catppuccin", "mocha", "macchiato",
        "frappe", "frappé", "latte", "classic", "github", "dark", "light", "midnight",
        "nord", "nordic", "solarized",
        t.settings.theme_presets_title,
        t.settings.theme_presets_desc,
        t.settings.catppuccin_flavors,
        t.settings.classic_themes,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(&q))
}

fn matches_accent(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    let haystacks = [
        "accent", "color", "colour", "primary", "akzent", "akzentfarbe", "hex", "picker", "custom", "reset",
        "mauve", "pink", "flamingo", "red", "peach", "yellow", "green", "teal", "sky", "sapphire", "blue", "lavender",
        t.settings.primary_accent_title,
        t.settings.primary_accent_desc,
        t.settings.reset_theme_default,
        t.settings.pick_custom_color_title,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(&q))
}

#[component]
pub fn AppearancePane(props: AppearancePaneProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = props.t;
    let active_accent = props.active_accent;
    let has_custom_accent = props.has_custom_accent;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim();

    let show_themes = filter.is_empty() || matches_themes(filter, t);
    let show_accent = filter.is_empty() || matches_accent(filter, t);

    rsx! {
        div {
            class: "settings-section flex flex-col gap-4",

            if show_themes {
                div {
                    class: "section-header",
                    h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "{t.settings.theme_presets_title}" }
                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.theme_presets_desc}" }
                }

                div {
                    class: "theme-group flex flex-col gap-1.5",
                    span {
                        class: "text-[11px] font-semibold text-[var(--text-muted)] uppercase tracking-wider inline-flex items-center gap-1.5",
                        Icon { width: 12, height: 12, icon: LdSparkles, class: "text-[var(--accent)]" }
                        "{t.settings.catppuccin_flavors}"
                    }
                    div {
                        class: "grid grid-cols-2 sm:grid-cols-4 gap-2",
                        for (theme_item, label, accent_color, bg_color) in catppuccin_themes() {
                            button {
                                class: if store_read.theme == theme_item { "theme-card active-card flex items-center gap-2.5 p-2.5 rounded-xl border-2 border-[var(--accent)] bg-[var(--bg-subtle)] text-left cursor-pointer transition-all shadow-sm" } else { "theme-card flex items-center gap-2.5 p-2.5 rounded-xl border border-[var(--border-color)] bg-[var(--bg-app)] hover:border-[var(--text-muted)] text-left cursor-pointer transition-all" },
                                onclick: move |_| store.write().set_theme(theme_item),
                                span {
                                    class: "w-4 h-4 rounded-full border border-solid flex items-center justify-center shrink-0",
                                    style: "background-color: {bg_color}; border-color: {accent_color};",
                                    span { class: "w-1.5 h-1.5 rounded-full", style: "background-color: {accent_color};" }
                                }
                                span { class: "text-xs font-medium text-[var(--text-heading)] truncate", "{label}" }
                            }
                        }
                    }
                }

                div {
                    class: "theme-group flex flex-col gap-1.5 mt-2",
                    span {
                        class: "text-[11px] font-semibold text-[var(--text-muted)] uppercase tracking-wider inline-flex items-center gap-1.5",
                        Icon { width: 12, height: 12, icon: LdZap, class: "text-[var(--accent)]" }
                        "{t.settings.classic_themes}"
                    }
                    div {
                        class: "grid grid-cols-2 sm:grid-cols-3 gap-2",
                        for (theme_item, label, accent_color, bg_color) in classic_themes() {
                            button {
                                class: if store_read.theme == theme_item { "theme-card active-card flex items-center gap-2.5 p-2.5 rounded-xl border-2 border-[var(--accent)] bg-[var(--bg-subtle)] text-left cursor-pointer transition-all shadow-sm" } else { "theme-card flex items-center gap-2.5 p-2.5 rounded-xl border border-[var(--border-color)] bg-[var(--bg-app)] hover:border-[var(--text-muted)] text-left cursor-pointer transition-all" },
                                onclick: move |_| store.write().set_theme(theme_item),
                                span {
                                    class: "w-4 h-4 rounded-full border border-solid flex items-center justify-center shrink-0",
                                    style: "background-color: {bg_color}; border-color: {accent_color};",
                                    span { class: "w-1.5 h-1.5 rounded-full", style: "background-color: {accent_color};" }
                                }
                                span { class: "text-xs font-medium text-[var(--text-heading)] truncate", "{label}" }
                            }
                        }
                    }
                }
            }

            if show_themes && show_accent {
                div { class: "w-full h-[1px] bg-[var(--border-color)] my-2" }
            }

            if show_accent {
                div {
                    class: "section-header flex items-center justify-between",
                    div {
                        h3 {
                            class: "text-sm font-semibold text-[var(--text-heading)] m-0 inline-flex items-center gap-1.5",
                            Icon { width: 14, height: 14, icon: LdPalette, class: "text-[var(--accent)]" }
                            "{t.settings.primary_accent_title}"
                        }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.primary_accent_desc}" }
                    }
                    if has_custom_accent {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            onclick: move |_| store.write().set_primary_color(None),
                            "{t.settings.reset_theme_default}"
                        }
                    }
                }

                div {
                    class: "grid grid-cols-6 sm:grid-cols-12 gap-1.5",
                    for (hex, name) in accent_presets() {
                        Hint {
                            text: format!("{name} ({hex})"),
                            div {
                                class: if active_accent.eq_ignore_ascii_case(hex) { "accent-color-chip active-chip w-full aspect-square rounded-lg cursor-pointer flex items-center justify-center ring-2 ring-white/60 transition-transform scale-105" } else { "accent-color-chip w-full aspect-square rounded-lg cursor-pointer flex items-center justify-center hover:scale-105 transition-transform" },
                                style: "background-color: {hex};",
                                onclick: move |_| store.write().set_primary_color(Some(hex.to_string())),
                                if active_accent.eq_ignore_ascii_case(hex) {
                                    span { class: "w-2 h-2 rounded-full bg-white shadow-sm" }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center gap-3 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl p-3",
                    div {
                        class: "w-8 h-8 rounded-lg border border-[var(--border-color)] overflow-hidden shrink-0 relative cursor-pointer",
                        style: "background-color: {active_accent};",
                        Hint {
                            text: t.settings.pick_custom_color_title,
                            input {
                                class: "opacity-0 absolute inset-0 w-full h-full cursor-pointer",
                                r#type: "color",
                                value: "{active_accent}",
                                oninput: move |evt| store.write().set_primary_color(Some(evt.value())),
                            }
                        }
                    }
                    Input {
                        class: "bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg px-3 py-1.5 text-xs font-mono text-[var(--text-heading)] flex-1 outline-none focus:border-[var(--accent)]",
                        r#type: "text",
                        placeholder: "#hexcode",
                        value: "{active_accent}",
                        oninput: move |evt: FormEvent| {
                            let val = evt.value().trim().to_string();
                            if val.starts_with('#') && (val.len() == 7 || val.len() == 4) {
                                store.write().set_primary_color(Some(val));
                            }
                        },
                    }
                }
            }
        }
    }
}
