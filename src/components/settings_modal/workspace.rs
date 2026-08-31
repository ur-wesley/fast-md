use crate::services::association::{open_default_apps_settings, register_file_associations};
use crate::state::AppStore;
use crate::types::{FileFilterMode, Language, SidebarPosition, SidebarTab};
use crate::ui::switch::Switch;
use crate::ui::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCheck, LdExternalLink, LdSparkles};
use std::collections::HashSet;

#[derive(Props, Clone, PartialEq)]
pub struct WorkspacePaneProps {
    pub store: Signal<AppStore>,
    pub t: &'static crate::i18n::Translations,
    pub assoc_registered: Signal<bool>,
    #[props(default)]
    pub search_filter: Option<String>,
}

pub fn has_matches(query: &str, t: &'static crate::i18n::Translations) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    matches_language(&q, t)
        || matches_sidebar_position(&q, t)
        || matches_sidebar_tab(&q, t)
        || matches_sidebar_visibility(&q, t)
        || matches_file_filter(&q, t)
        || matches_recent_history(&q, t)
        || matches_explorer_integration(&q, t)
}

fn matches_language(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "language", "sprache", "english", "deutsch", "german",
        t.settings.language_section_title,
        t.settings.language_section_desc,
        t.settings.language_en,
        t.settings.language_de,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_sidebar_position(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "sidebar", "position", "left", "right", "links", "rechts", "seitenleiste",
        t.settings.sidebar_position_title,
        t.settings.sidebar_position_desc,
        t.settings.sidebar_position_left,
        t.settings.sidebar_position_right,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_sidebar_tab(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "sidebar tab", "toc", "outline", "gliederung", "inhalt", "files", "explorer", "dateien", "dateibrowser",
        t.settings.sidebar_tab_title,
        t.settings.sidebar_tab_desc,
        t.settings.sidebar_tab_toc,
        t.settings.sidebar_tab_files,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_sidebar_visibility(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "sidebar visibility", "show sidebar", "hide sidebar", "sichtbarkeit", "einblenden", "ausblenden",
        t.settings.sidebar_visibility_title,
        t.settings.sidebar_visibility_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_file_filter(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "filter", "file filter", "extension", "dateifilter", "dateitypen", "markdown", "md", "mdx", "config", "supported", "all",
        t.settings.file_filter_mode_title,
        t.settings.file_filter_mode_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_recent_history(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "recent", "history", "files", "folders", "zuletzt", "verlauf", "ordner", "dateien",
        t.settings.recent_history_title,
        t.settings.recent_history_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_explorer_integration(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "explorer", "integration", "shell", "default apps", "windows", "open with", "öffnen mit", "standard apps",
        t.settings.explorer_integration_title,
        t.settings.explorer_integration_desc,
        t.settings.register_explorer,
        t.settings.registered_explorer,
        t.settings.open_default_apps,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn language_index(lang: Language) -> usize {
    match lang {
        Language::En => 0,
        Language::De => 1,
    }
}

fn language_from_index(idx: usize) -> Language {
    match idx {
        0 => Language::En,
        _ => Language::De,
    }
}

fn sidebar_tab_index(tab: SidebarTab) -> usize {
    match tab {
        SidebarTab::Toc => 0,
        SidebarTab::Files => 1,
    }
}

fn sidebar_tab_from_index(idx: usize) -> SidebarTab {
    match idx {
        0 => SidebarTab::Toc,
        _ => SidebarTab::Files,
    }
}

fn sidebar_position_index(pos: SidebarPosition) -> usize {
    match pos {
        SidebarPosition::Left => 0,
        SidebarPosition::Right => 1,
    }
}

fn sidebar_position_from_index(idx: usize) -> SidebarPosition {
    match idx {
        0 => SidebarPosition::Left,
        _ => SidebarPosition::Right,
    }
}

fn file_filter_index(mode: FileFilterMode) -> usize {
    match mode {
        FileFilterMode::MarkdownOnly => 0,
        FileFilterMode::MarkdownAndConfig => 1,
        FileFilterMode::AllSupported => 2,
        FileFilterMode::AllFiles => 3,
    }
}

fn file_filter_from_index(idx: usize) -> FileFilterMode {
    match idx {
        0 => FileFilterMode::MarkdownOnly,
        1 => FileFilterMode::MarkdownAndConfig,
        2 => FileFilterMode::AllSupported,
        _ => FileFilterMode::AllFiles,
    }
}

#[component]
pub fn WorkspacePane(props: WorkspacePaneProps) -> Element {
    let mut store = props.store;
    let mut assoc_registered = props.assoc_registered;
    let store_read = store();
    let t = props.t;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim().to_lowercase();

    let show_language = filter.is_empty() || matches_language(&filter, t);
    let show_sidebar_position = filter.is_empty() || matches_sidebar_position(&filter, t);
    let show_sidebar_tab = filter.is_empty() || matches_sidebar_tab(&filter, t);
    let show_sidebar_visibility = filter.is_empty() || matches_sidebar_visibility(&filter, t);
    let show_file_filter = filter.is_empty() || matches_file_filter(&filter, t);
    let show_recent_history = filter.is_empty() || matches_recent_history(&filter, t);
    let show_explorer_integration = filter.is_empty() || matches_explorer_integration(&filter, t);

    let language_pressed = use_memo(move || Some(HashSet::from([language_index(store().language)])));
    let sidebar_tab_pressed =
        use_memo(move || Some(HashSet::from([sidebar_tab_index(store().sidebar_tab)])));
    let sidebar_position_pressed =
        use_memo(move || Some(HashSet::from([sidebar_position_index(store().sidebar_position)])));
    let sidebar_visible_checked = use_memo(move || Some(store().show_sidebar));
    let file_filter_pressed =
        use_memo(move || Some(HashSet::from([file_filter_index(store().file_filter_mode)])));

    rsx! {
        div {
            class: "settings-section flex flex-col gap-5",

            if show_language {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.language_section_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.language_section_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: language_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_language(language_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "{t.settings.language_en}" }
                        ToggleItem { index: 1usize, "{t.settings.language_de}" }
                    }
                }
            }

            if show_sidebar_position {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.sidebar_position_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.sidebar_position_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: sidebar_position_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_sidebar_position(sidebar_position_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "{t.settings.sidebar_position_left}" }
                        ToggleItem { index: 1usize, "{t.settings.sidebar_position_right}" }
                    }
                }
            }

            if show_sidebar_tab {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.sidebar_tab_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.sidebar_tab_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: sidebar_tab_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_sidebar_tab(sidebar_tab_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "{t.settings.sidebar_tab_toc}" }
                        ToggleItem { index: 1usize, "{t.settings.sidebar_tab_files}" }
                    }
                }
            }

            if show_sidebar_visibility {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.sidebar_visibility_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.sidebar_visibility_desc}" }
                    }
                    Switch {
                        checked: sidebar_visible_checked,
                        on_checked_change: move |checked: bool| {
                            if store().show_sidebar != checked {
                                store.write().toggle_sidebar();
                            }
                        },
                    }
                }
            }

            if show_file_filter {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.file_filter_mode_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.file_filter_mode_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: file_filter_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_file_filter_mode(file_filter_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "MD(X)" }
                        ToggleItem { index: 1usize, "+ Config" }
                        ToggleItem { index: 2usize, "Supported" }
                        ToggleItem { index: 3usize, "All" }
                    }
                }
            }

            if show_recent_history {
                div {
                    class: "p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl flex flex-col gap-2",
                    div {
                        class: "flex items-center justify-between",
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.recent_history_title}" }
                        span { class: "text-[11px] font-mono text-[var(--text-muted)]", "{store_read.settings.recent_files.len()} {t.settings.files_label} / {store_read.settings.recent_folders.len()} {t.settings.folders_label}" }
                    }
                    p { class: "text-xs text-[var(--text-muted)] m-0", "{t.settings.recent_history_desc}" }
                }
            }

            if show_explorer_integration {
                div {
                    class: "p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl flex flex-col gap-3",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.explorer_integration_title}" }
                            p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.explorer_integration_desc}" }
                        }
                    }
                    div {
                        class: "flex flex-wrap items-center gap-2 mt-0.5",
                        button {
                            class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)] font-medium hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                            onclick: move |_| {
                                let success = register_file_associations();
                                assoc_registered.set(success);
                            },
                            if assoc_registered() {
                                Icon { width: 13, height: 13, icon: LdCheck, class: "text-emerald-400" }
                            } else {
                                Icon { width: 13, height: 13, icon: LdSparkles, class: "text-[var(--accent)]" }
                            }
                            span { if assoc_registered() { "{t.settings.registered_explorer}" } else { "{t.settings.register_explorer}" } }
                        }
                        button {
                            class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)] font-medium hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                            onclick: move |_| open_default_apps_settings(),
                            Icon { width: 13, height: 13, icon: LdExternalLink }
                            span { "{t.settings.open_default_apps}" }
                        }
                    }
                }
            }
        }
    }
}

