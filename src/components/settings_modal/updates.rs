use crate::services::updater::{check_github_release, download_and_apply_update, restart_app};
use crate::state::AppStore;
use crate::types::UpdateStatus;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::switch::Switch;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdCheck, LdDownload, LdExternalLink, LdRefreshCw, LdSparkles, LdX,
};

#[derive(Props, Clone, PartialEq)]
pub struct UpdatesPaneProps {
    pub store: Signal<AppStore>,
    pub t: &'static crate::i18n::Translations,
    #[props(default)]
    pub search_filter: Option<String>,
}

pub fn has_matches(query: &str, t: &'static crate::i18n::Translations) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    matches_updates(&q, t) || matches_auto_check(&q, t)
}

fn matches_updates(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "update", "version", "github", "release", "download", "install", "check", "aktualisierung", "herunterladen", "neu",
        t.settings.updates_title,
        t.settings.updates_desc,
        t.settings.current_version_label,
        t.settings.check_for_updates_button,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_auto_check(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "auto check", "startup", "start", "automatisch prüfen", "auto update",
        t.settings.auto_check_updates_title,
        t.settings.auto_check_updates_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

#[component]
pub fn UpdatesPane(props: UpdatesPaneProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = props.t;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim().to_lowercase();

    let show_updates = filter.is_empty() || matches_updates(&filter, t);
    let show_auto_check = filter.is_empty() || matches_auto_check(&filter, t);

    let auto_check_checked = use_memo(move || Some(store().settings.auto_check_updates));
    let is_busy = store_read.update_status.is_checking() || store_read.update_status.is_downloading();

    rsx! {
        div {
            class: "settings-section flex flex-col gap-5",

            if show_updates {
                div {
                    class: "section-header",
                    h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "{t.settings.updates_title}" }
                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.updates_desc}" }
                }

                div {
                    class: "bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-3",

                    div {
                        class: "flex items-center justify-between",
                        div {
                            class: "flex flex-col gap-0.5",
                            span { class: "text-[11px] font-semibold text-[var(--text-muted)] uppercase tracking-wider", "{t.settings.current_version_label}" }
                            span { class: "text-sm font-mono font-bold text-[var(--text-heading)]", "Fast-MD v{env!(\"CARGO_PKG_VERSION\")}" }
                        }

                        Button {
                            variant: if is_busy { ButtonVariant::Outline } else { ButtonVariant::Primary },
                            size: ButtonSize::Sm,
                            disabled: is_busy,
                            onclick: move |_| {
                                store.write().set_update_status(UpdateStatus::Checking);
                                to_owned![store];
                                spawn(async move {
                                    let res = tokio::task::spawn_blocking(check_github_release).await;
                                    match res {
                                        Ok(Ok(Some(release))) => {
                                            store.write().set_update_status(UpdateStatus::Available(release));
                                        }
                                        Ok(Ok(None)) => {
                                            store.write().set_update_status(UpdateStatus::UpToDate);
                                        }
                                        Ok(Err(err)) => {
                                            store.write().set_update_status(UpdateStatus::Error(err.to_string()));
                                        }
                                        Err(join_err) => {
                                            store.write().set_update_status(UpdateStatus::Error(join_err.to_string()));
                                        }
                                    }
                                });
                            },
                            if store_read.update_status.is_checking() {
                                Icon { width: 14, height: 14, icon: LdRefreshCw, class: "animate-spin" }
                                span { "{t.settings.checking_for_updates}" }
                            } else {
                                Icon { width: 14, height: 14, icon: LdRefreshCw }
                                span { "{t.settings.check_for_updates_button}" }
                            }
                        }
                    }

                    match &store_read.update_status {
                        UpdateStatus::Idle => rsx! {},
                        UpdateStatus::Checking => rsx! {
                            div {
                                class: "flex items-center gap-2 text-xs text-[var(--text-muted)] pt-2 border-t border-[var(--border-color)]",
                                Icon { width: 14, height: 14, icon: LdRefreshCw, class: "animate-spin text-[var(--accent)] shrink-0" }
                                span { "{t.settings.checking_for_updates}" }
                            }
                        },
                        UpdateStatus::UpToDate => rsx! {
                            div {
                                class: "flex items-center gap-2 p-2.5 rounded-lg bg-emerald-950/20 border border-emerald-800/40 text-emerald-400 text-xs font-medium",
                                Icon { width: 15, height: 15, icon: LdCheck, class: "shrink-0" }
                                span { "{t.settings.up_to_date_message}" }
                            }
                        },
                        UpdateStatus::Available(release) => rsx! {
                            div {
                                class: "flex flex-col gap-3 p-3.5 rounded-xl bg-[var(--bg-subtle)] border border-[var(--accent)]/50",

                                div {
                                    class: "flex items-start justify-between gap-2",
                                    div {
                                        class: "flex items-center gap-2",
                                        Icon { width: 16, height: 16, icon: LdSparkles, class: "text-[var(--accent)] shrink-0" }
                                        div {
                                            h4 { class: "text-xs font-bold text-[var(--text-heading)] m-0", "{t.settings.update_available_title}: v{release.version}" }
                                            p { class: "text-[11px] text-[var(--text-muted)] m-0 font-mono", "{release.asset_name}" }
                                        }
                                    }
                                    a {
                                        class: "inline-flex items-center gap-1 text-[11px] text-[var(--accent)] hover:underline",
                                        href: "{release.html_url}",
                                        target: "_blank",
                                        Icon { width: 12, height: 12, icon: LdExternalLink }
                                        span { "{t.settings.view_release_notes}" }
                                    }
                                }

                                if !release.release_notes.is_empty() {
                                    div {
                                        class: "p-2.5 max-h-32 overflow-y-auto rounded-lg bg-[var(--bg-app)] border border-[var(--border-subtle)] text-xs text-[var(--text-main)] font-mono whitespace-pre-wrap leading-relaxed",
                                        "{release.release_notes}"
                                    }
                                }

                                div {
                                    class: "flex items-center gap-2 pt-1",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Sm,
                                        onclick: {
                                            let rel = release.clone();
                                            to_owned![store];
                                            move |_| {
                                                let rel_clone = rel.clone();
                                                store.write().set_update_status(UpdateStatus::Downloading {
                                                    version: rel_clone.version.clone(),
                                                    progress: 0,
                                                });

                                                spawn(async move {
                                                    let rel_for_task = rel_clone.clone();
                                                    let ver = rel_clone.version.clone();
                                                    let res = tokio::task::spawn_blocking(move || {
                                                        download_and_apply_update(&rel_for_task, |_pct| {})
                                                    }).await;

                                                    match res {
                                                        Ok(Ok(())) => {
                                                            store.write().set_update_status(UpdateStatus::ReadyToRestart { version: ver });
                                                        }
                                                        Ok(Err(e)) => {
                                                            store.write().set_update_status(UpdateStatus::Error(e.to_string()));
                                                        }
                                                        Err(join_err) => {
                                                            store.write().set_update_status(UpdateStatus::Error(join_err.to_string()));
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                        Icon { width: 14, height: 14, icon: LdDownload }
                                        span { "{t.settings.update_download_button}" }
                                    }
                                }
                            }
                        },
                        UpdateStatus::Downloading { version, progress } => rsx! {
                            div {
                                class: "flex flex-col gap-2 p-3.5 rounded-xl bg-[var(--bg-subtle)] border border-[var(--border-color)]",
                                div {
                                    class: "flex items-center justify-between text-xs font-medium text-[var(--text-heading)]",
                                    span { "{t.settings.downloading_update} v{version}" }
                                    span { "{progress}%" }
                                }
                                div {
                                    class: "w-full h-2 rounded-full bg-[var(--bg-app)] overflow-hidden border border-[var(--border-color)]",
                                    div {
                                        class: "h-full bg-[var(--accent)] transition-all duration-200",
                                        style: "width: {progress}%;",
                                    }
                                }
                            }
                        },
                        UpdateStatus::Installing { version } => rsx! {
                            div {
                                class: "flex items-center gap-2.5 p-3.5 rounded-xl bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)]",
                                Icon { width: 15, height: 15, icon: LdRefreshCw, class: "animate-spin text-[var(--accent)] shrink-0" }
                                span { "{t.settings.installing_update} v{version}" }
                            }
                        },
                        UpdateStatus::ReadyToRestart { version } => rsx! {
                            div {
                                class: "flex items-center justify-between p-3.5 rounded-xl bg-emerald-950/25 border border-emerald-700/50",
                                div {
                                    class: "flex items-center gap-2",
                                    Icon { width: 16, height: 16, icon: LdCheck, class: "text-emerald-400 shrink-0" }
                                    div {
                                        h4 { class: "text-xs font-bold text-emerald-400 m-0", "{t.settings.update_ready_title}" }
                                        p { class: "text-[11px] text-[var(--text-muted)] m-0", "Fast-MD v{version} installed." }
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    size: ButtonSize::Sm,
                                    onclick: move |_| {
                                        let _ = restart_app();
                                    },
                                    Icon { width: 14, height: 14, icon: LdRefreshCw }
                                    span { "{t.settings.restart_and_update_button}" }
                                }
                            }
                        },
                        UpdateStatus::Error(err_msg) => rsx! {
                            div {
                                class: "flex flex-col gap-1.5 p-3.5 rounded-xl bg-red-950/20 border border-red-900/40 text-red-400 text-xs",
                                div {
                                    class: "flex items-center gap-2 font-semibold",
                                    Icon { width: 15, height: 15, icon: LdX, class: "shrink-0" }
                                    span { "{t.settings.update_error_title}" }
                                }
                                p { class: "m-0 font-mono text-[11px] opacity-90 break-all", "{err_msg}" }
                            }
                        },
                    }
                }
            }

            if show_updates && show_auto_check {
                div { class: "w-full h-[1px] bg-[var(--border-color)] my-1" }
            }

            if show_auto_check {
                div {
                    class: "settings-option-row",
                    div {
                        class: "flex flex-col gap-0.5",
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.auto_check_updates_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0", "{t.settings.auto_check_updates_desc}" }
                    }
                    Switch {
                        checked: auto_check_checked,
                        on_checked_change: move |checked: bool| {
                            store.write().set_auto_check_updates(checked);
                        },
                    }
                }
            }
        }
    }
}

