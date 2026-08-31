mod appearance;
mod config_file;
mod reader;
mod shortcuts;
mod updates;
mod workspace;

use crate::components::Hint;
use crate::services::association::is_file_associations_registered;
use crate::services::settings::get_settings_file_path;
use crate::state::AppStore;
use crate::types::{AppTheme, UpdateStatus};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use appearance::AppearancePane;
use config_file::ConfigFilePane;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdArrowLeft, LdBookOpen, LdCommand, LdDownload, LdFileCode2, LdFolderTree, LdPalette, LdSearch,
    LdSettings, LdX,
};
use reader::ReaderPane;
use shortcuts::ShortcutsPane;
use updates::UpdatesPane;
use workspace::WorkspacePane;

#[derive(Props, Clone, PartialEq, Eq)]
pub struct SettingsModalProps {
    pub store: Signal<AppStore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Appearance,
    Reader,
    Workspace,
    Shortcuts,
    ConfigFile,
    Updates,
}


pub(crate) fn catppuccin_themes() -> [(AppTheme, &'static str, &'static str, &'static str); 4] {
    [
        (AppTheme::CatppuccinMocha, "Mocha", "#cba6f7", "#1e1e2e"),
        (
            AppTheme::CatppuccinMacchiato,
            "Macchiato",
            "#c6a0f6",
            "#24273a",
        ),
        (AppTheme::CatppuccinFrappe, "Frappé", "#ca9ee6", "#303446"),
        (AppTheme::CatppuccinLatte, "Latte", "#8839ef", "#eff1f5"),
    ]
}

pub(crate) fn classic_themes() -> [(AppTheme, &'static str, &'static str, &'static str); 5] {
    [
        (AppTheme::Dark, "GitHub Dark", "#58a6ff", "#161b22"),
        (AppTheme::Midnight, "Midnight", "#8b5cf6", "#12141c"),
        (AppTheme::Light, "GitHub Light", "#0969da", "#f6f8fa"),
        (AppTheme::Nord, "Nordic Frost", "#88c0d0", "#3b4252"),
        (AppTheme::SolarizedDark, "Solarized", "#268bd2", "#073642"),
    ]
}

pub(crate) fn accent_presets() -> [(&'static str, &'static str); 12] {
    [
        ("#cba6f7", "Mauve"),
        ("#f5c2e7", "Pink"),
        ("#f2cdcd", "Flamingo"),
        ("#f38ba8", "Red"),
        ("#fab387", "Peach"),
        ("#f9e2af", "Yellow"),
        ("#a6e3a1", "Green"),
        ("#94e2d5", "Teal"),
        ("#89dceb", "Sky"),
        ("#74c7ec", "Sapphire"),
        ("#89b4fa", "Blue"),
        ("#b4befe", "Lavender"),
    ]
}

#[component]
pub fn SettingsModal(props: SettingsModalProps) -> Element {
    let mut current_tab = use_signal(|| SettingsTab::Appearance);
    let mut search_query = use_signal(String::new);
    let copy_feedback = use_signal(|| false);
    let assoc_registered = use_signal(is_file_associations_registered);
    let mut store = props.store;
    let store_read = store();
    let t = store_read.language.strings();

    let settings_path = get_settings_file_path();
    let settings_path_display = settings_path.to_string_lossy().to_string();
    let active_accent = store_read.effective_primary_color().to_string();
    let has_custom_accent = store_read.primary_color.is_some();

    let query = search_query().trim().to_lowercase();
    let is_searching = !query.is_empty();

    let has_appearance_matches = is_searching && appearance::has_matches(&query, t);
    let has_reader_matches = is_searching && reader::has_matches(&query, t);
    let has_workspace_matches = is_searching && workspace::has_matches(&query, t);
    let has_shortcuts_matches = is_searching && shortcuts::has_matches(&query, t, &store_read);
    let has_config_matches = is_searching && config_file::has_matches(&query, t);
    let has_updates_matches = is_searching && updates::has_matches(&query, t);

    let has_any_matches = has_appearance_matches
        || has_reader_matches
        || has_workspace_matches
        || has_shortcuts_matches
        || has_config_matches
        || has_updates_matches;

    let search_val = search_query();

    rsx! {
        div {
            class: "settings-page flex-1 w-full h-full flex flex-col bg-[var(--bg-app)] overflow-hidden min-h-0 animate-fade-in",

            // Settings Header (Max-width aligned with content)
            div {
                class: "settings-page-header w-full border-b border-[var(--border-color)] bg-[var(--bg-surface)] shrink-0 px-4 md:px-8 py-3.5",
                div {
                    class: "w-full max-w-4xl mx-auto flex items-center justify-between gap-4",

                    // Left: Back button & Title
                    div {
                        class: "flex items-center gap-3 shrink-0",
                        Hint {
                            text: t.settings.close_tooltip,
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconSm,
                                onclick: move |_| {
                                    store.write().set_settings_modal(false);
                                },
                                Icon { width: 16, height: 16, icon: LdArrowLeft }
                            }
                        }
                        div {
                            class: "flex items-center gap-2.5",
                            Icon {
                                width: 20,
                                height: 20,
                                icon: LdSettings,
                                class: "text-[var(--accent)] shrink-0",
                            }
                            div {
                                h2 { class: "text-base font-bold text-[var(--text-heading)] m-0 leading-tight", "{t.settings.modal_title}" }
                                p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5 hidden sm:block", "{t.settings.auto_save_notice}" }
                            }
                        }
                    }

                    // Center/Right: Search Bar & Done Button
                    div {
                        class: "flex items-center gap-3 flex-1 justify-end max-w-md",

                        div {
                            class: "settings-search-input relative flex items-center bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl px-2.5 py-1.5 w-full max-w-xs transition-all focus-within:border-[var(--accent)]",
                            Icon { width: 14, height: 14, icon: LdSearch, class: "text-[var(--text-muted)] shrink-0 mr-2" }
                            Input {
                                class: "bg-transparent border-0 text-[var(--text-heading)] text-xs outline-none flex-1 min-w-0 placeholder:text-[var(--text-muted)]",
                                r#type: "text",
                                placeholder: "{t.settings.search_placeholder}",
                                value: "{search_val}",
                                oninput: move |evt: FormEvent| {
                                    search_query.set(evt.value());
                                },
                                onkeydown: move |evt: KeyboardEvent| {
                                    if evt.key() == Key::Escape {
                                        search_query.set(String::new());
                                    }
                                }
                            }
                            if !search_val.is_empty() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconXs,
                                    class: "text-[var(--text-muted)] hover:text-[var(--text-heading)] p-0.5",
                                    onclick: move |_| {
                                        search_query.set(String::new());
                                    },
                                    Icon { width: 12, height: 12, icon: LdX }
                                }
                            }
                        }

                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            class: "shrink-0",
                            onclick: move |_| {
                                store.write().set_settings_modal(false);
                            },
                            "{t.settings.done_button}"
                        }
                    }
                }
            }

            // Tab Bar Strip (Max-width aligned with content)
            div {
                class: "settings-nav-strip w-full border-b border-[var(--border-color)] bg-[var(--bg-subtle)] shrink-0 px-4 md:px-8",
                div {
                    class: "w-full max-w-4xl mx-auto flex items-center justify-between gap-2 overflow-x-auto py-2",

                    if is_searching {
                        div {
                            class: "flex items-center justify-between w-full py-0.5",
                            div {
                                class: "flex items-center gap-2 text-xs font-semibold text-[var(--text-heading)]",
                                Icon { width: 14, height: 14, icon: LdSearch, class: "text-[var(--accent)]" }
                                span { "{t.settings.search_results_title}" }
                                span {
                                    class: "px-2 py-0.5 rounded-full bg-[var(--accent)]/15 text-[var(--accent)] text-[11px] font-mono",
                                    "\"{search_val}\""
                                }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                class: "text-xs text-[var(--text-muted)] hover:text-[var(--text-heading)] gap-1",
                                onclick: move |_| search_query.set(String::new()),
                                Icon { width: 12, height: 12, icon: LdX }
                                span { "{t.settings.clear_search}" }
                            }
                        }
                    } else {
                        div {
                            class: "flex items-center gap-1.5 w-full",
                            button {
                                class: if current_tab() == SettingsTab::Appearance {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::Appearance),
                                Icon { width: 14, height: 14, icon: LdPalette }
                                span { "{t.settings.tab_appearance}" }
                            }

                            button {
                                class: if current_tab() == SettingsTab::Reader {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::Reader),
                                Icon { width: 14, height: 14, icon: LdBookOpen }
                                span { "{t.settings.tab_reader}" }
                            }

                            button {
                                class: if current_tab() == SettingsTab::Workspace {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::Workspace),
                                Icon { width: 14, height: 14, icon: LdFolderTree }
                                span { "{t.settings.tab_workspace}" }
                            }

                            button {
                                class: if current_tab() == SettingsTab::Shortcuts {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::Shortcuts),
                                Icon { width: 14, height: 14, icon: LdCommand }
                                span { "{t.settings.tab_shortcuts}" }
                            }

                            button {
                                class: if current_tab() == SettingsTab::ConfigFile {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::ConfigFile),
                                Icon { width: 14, height: 14, icon: LdFileCode2 }
                                span { "{t.settings.tab_config_file}" }
                            }

                            button {
                                class: if current_tab() == SettingsTab::Updates {
                                    "settings-tab-trigger active flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-surface)] text-[var(--text-heading)] border border-[var(--border-color)] shadow-xs transition-all cursor-pointer shrink-0"
                                } else {
                                    "settings-tab-trigger flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] border border-transparent transition-all cursor-pointer shrink-0"
                                },
                                onclick: move |_| current_tab.set(SettingsTab::Updates),
                                Icon { width: 14, height: 14, icon: LdDownload }
                                span { "{t.settings.tab_updates}" }
                                if matches!(store_read.update_status, UpdateStatus::Available(_)) {
                                    span { class: "w-2 h-2 rounded-full bg-[var(--accent)] shrink-0" }
                                }
                            }
                        }
                    }
                }
            }

            // Main Scrollable Body (Max-width aligned with header and tab bar)
            div {
                class: "settings-page-body flex-1 overflow-y-auto w-full p-4 md:p-8 flex justify-center min-h-0",

                div {
                    class: "w-full max-w-4xl flex flex-col gap-6",

                    if is_searching {
                        if !has_any_matches {
                            div {
                                class: "flex flex-col items-center justify-center py-16 text-center gap-3",
                                div {
                                    class: "w-12 h-12 rounded-2xl bg-[var(--bg-subtle)] border border-[var(--border-color)] flex items-center justify-center text-[var(--text-muted)]",
                                    Icon { width: 22, height: 22, icon: LdSearch }
                                }
                                h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "{t.settings.no_results_title}" }
                                p { class: "text-xs text-[var(--text-muted)] m-0 max-w-sm", "{t.settings.no_results_desc}" }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    class: "mt-2",
                                    onclick: move |_| search_query.set(String::new()),
                                    "{t.settings.clear_search}"
                                }
                            }
                        } else {
                            if has_appearance_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdPalette, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_appearance}" }
                                    }
                                    AppearancePane {
                                        store,
                                        t,
                                        active_accent: active_accent.clone(),
                                        has_custom_accent,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }

                            if has_reader_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdBookOpen, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_reader}" }
                                    }
                                    ReaderPane {
                                        store,
                                        t,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }

                            if has_workspace_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdFolderTree, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_workspace}" }
                                    }
                                    WorkspacePane {
                                        store,
                                        t,
                                        assoc_registered,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }

                            if has_shortcuts_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdCommand, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_shortcuts}" }
                                    }
                                    ShortcutsPane {
                                        store,
                                        t,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }

                            if has_config_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdFileCode2, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_config_file}" }
                                    }
                                    ConfigFilePane {
                                        store,
                                        t,
                                        settings_path_display: settings_path_display.clone(),
                                        copy_feedback,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }

                            if has_updates_matches {
                                div {
                                    class: "flex flex-col gap-3 p-4 md:p-5 rounded-2xl bg-[var(--bg-subtle)]/40 border border-[var(--border-color)]",
                                    div {
                                        class: "flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]",
                                        Icon { width: 15, height: 15, icon: LdDownload, class: "text-[var(--accent)] shrink-0" }
                                        span { class: "text-xs font-bold uppercase tracking-wider text-[var(--text-heading)]", "{t.settings.tab_updates}" }
                                    }
                                    UpdatesPane {
                                        store,
                                        t,
                                        search_filter: Some(query.clone()),
                                    }
                                }
                            }
                        }
                    } else {
                        match current_tab() {
                            SettingsTab::Appearance => rsx! {
                                AppearancePane {
                                    store,
                                    t,
                                    active_accent,
                                    has_custom_accent,
                                }
                            },
                            SettingsTab::Reader => rsx! {
                                ReaderPane { store, t }
                            },
                            SettingsTab::Workspace => rsx! {
                                WorkspacePane { store, t, assoc_registered }
                            },
                            SettingsTab::Shortcuts => rsx! {
                                ShortcutsPane { store, t }
                            },
                            SettingsTab::ConfigFile => rsx! {
                                ConfigFilePane {
                                    store,
                                    t,
                                    settings_path_display,
                                    copy_feedback,
                                }
                            },
                            SettingsTab::Updates => rsx! {
                                UpdatesPane { store, t }
                            },
                        }
                    }
                }
            }

            // Settings Footer (Max-width aligned with content)
            div {
                class: "settings-page-footer w-full border-t border-[var(--border-color)] bg-[var(--bg-surface)] shrink-0 px-4 md:px-8 py-3.5",
                div {
                    class: "w-full max-w-4xl mx-auto flex items-center justify-between gap-4",
                    span { class: "text-[11px] font-mono text-[var(--text-muted)]", "Fast-MD v{env!(\"CARGO_PKG_VERSION\")}" }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.write().set_settings_modal(false),
                        "{t.settings.done_button}"
                    }
                }
            }
        }
    }
}


