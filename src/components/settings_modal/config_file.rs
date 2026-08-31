use crate::services::settings::{open_settings_in_editor, reveal_settings_folder};
use crate::state::AppStore;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdCheck, LdCopy, LdExternalLink, LdFileCode2, LdFolderOpen, LdRotateCcw,
};

#[derive(Props, Clone, PartialEq)]
pub struct ConfigFilePaneProps {
    pub store: Signal<AppStore>,
    pub t: &'static crate::i18n::Translations,
    pub settings_path_display: String,
    pub copy_feedback: Signal<bool>,
    #[props(default)]
    pub search_filter: Option<String>,
}

pub fn has_matches(query: &str, t: &'static crate::i18n::Translations) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    matches_config_location(&q, t) || matches_reset_defaults(&q, t)
}

fn matches_config_location(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "config", "json", "settings.json", "path", "pfad", "speicherort", "copy", "kopieren", "editor", "folder", "ordner", "open", "show",
        t.settings.config_location_title,
        t.settings.config_location_desc,
        t.settings.copy_path,
        t.settings.open_in_editor,
        t.settings.show_in_folder,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_reset_defaults(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "reset", "defaults", "restore", "zurücksetzen", "werkseinstellungen", "standard", "factory",
        t.settings.reset_all_title,
        t.settings.reset_all_desc,
        t.settings.reset_defaults,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

#[component]
pub fn ConfigFilePane(props: ConfigFilePaneProps) -> Element {
    let mut store = props.store;
    let mut copy_feedback = props.copy_feedback;
    let t = props.t;
    let settings_path_display = props.settings_path_display;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim().to_lowercase();

    let show_location = filter.is_empty() || matches_config_location(&filter, t);
    let show_reset = filter.is_empty() || matches_reset_defaults(&filter, t);

    rsx! {
        div {
            class: "settings-section flex flex-col gap-4",

            if show_location {
                div {
                    class: "section-header",
                    h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "{t.settings.config_location_title}" }
                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.config_location_desc}" }
                }

                div {
                    class: "bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl p-3 flex flex-col gap-2.5",
                    div {
                        class: "flex items-center gap-2",
                        Icon {
                            width: 15,
                            height: 15,
                            icon: LdFileCode2,
                            class: "text-[var(--accent)] shrink-0",
                        }
                        span { class: "text-xs font-mono font-medium text-[var(--text-heading)] break-all select-all", "{settings_path_display}" }
                    }

                    div {
                        class: "flex flex-wrap items-center gap-2 mt-1",
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let p = settings_path_display.clone();
                                dioxus::prelude::document::eval(&format!("navigator.clipboard && navigator.clipboard.writeText({p:?});"));
                                copy_feedback.set(true);
                            },
                            if copy_feedback() {
                                Icon { width: 13, height: 13, icon: LdCheck }
                            } else {
                                Icon { width: 13, height: 13, icon: LdCopy }
                            }
                            span { if copy_feedback() { "{t.settings.copied_path}" } else { "{t.settings.copy_path}" } }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| open_settings_in_editor(),
                            Icon { width: 13, height: 13, icon: LdExternalLink }
                            span { "{t.settings.open_in_editor}" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| reveal_settings_folder(),
                            Icon { width: 13, height: 13, icon: LdFolderOpen }
                            span { "{t.settings.show_in_folder}" }
                        }
                    }
                }
            }

            if show_location && show_reset {
                div { class: "w-full h-[1px] bg-[var(--border-color)] my-1" }
            }

            if show_reset {
                div {
                    class: "p-3.5 bg-red-950/20 border border-red-900/40 rounded-xl flex items-center justify-between",
                    div {
                        h4 { class: "text-xs font-semibold text-red-400 m-0", "{t.settings.reset_all_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.reset_all_desc}" }
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.write().reset_settings_to_default(),
                        Icon { width: 13, height: 13, icon: LdRotateCcw }
                        span { "{t.settings.reset_defaults}" }
                    }
                }
            }
        }
    }
}
