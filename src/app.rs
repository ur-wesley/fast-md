use crate::cli::CliArgs;
use crate::components::{
    Editor, SettingsModal, Sidebar, StatusBar, TabBar, TitleBar, Toolbar, Viewer, WorkspaceSplit,
    ZenExitButton,
};
use crate::services::fs::read_document_file;
use crate::services::updater;
use crate::services::watcher::LiveFileWatcher;
use crate::state::{kick_pending_tree_scan, AppStore};
use crate::types::{AppTheme, DocumentMode, Language, UpdateStatus};
use crate::{resolve_cli_path, CLI_ARGS};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileText, LdPlus};
use std::time::Duration;

#[derive(serde::Deserialize, Clone, Debug)]
struct GlobalShortcutPayload {
    key: String,
    #[serde(rename = "ctrlKey")]
    ctrl_key: bool,
    #[serde(rename = "metaKey")]
    meta_key: bool,
    #[serde(rename = "altKey")]
    alt_key: bool,
    #[serde(rename = "shiftKey")]
    shift_key: bool,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(tag = "action")]
enum DocSyncPayload {
    #[serde(rename = "set_markdown")]
    SetMarkdown { content: String },
    #[serde(rename = "toggle_checkbox")]
    ToggleCheckbox { index: usize, checked: bool },
}


#[component]
pub fn App() -> Element {
    let cli_args = CLI_ARGS.get().cloned().unwrap_or(CliArgs {
        path: None,
        zen: false,
        theme: None,
        lang: None,
        register: false,
        unregister: false,
    });

    let cli_theme = cli_args.theme.as_deref().and_then(|t| match t.to_lowercase().as_str() {
        "light" => Some(AppTheme::Light),
        "midnight" => Some(AppTheme::Midnight),
        "nord" => Some(AppTheme::Nord),
        "solarized" | "solarized-dark" => Some(AppTheme::SolarizedDark),
        "latte" | "catppuccin-latte" => Some(AppTheme::CatppuccinLatte),
        "frappe" | "frappé" | "catppuccin-frappe" | "catppuccin-frappé" => Some(AppTheme::CatppuccinFrappe),
        "macchiato" | "catppuccin-macchiato" => Some(AppTheme::CatppuccinMacchiato),
        "mocha" | "catppuccin" | "catppuccin-mocha" => Some(AppTheme::CatppuccinMocha),
        "dark" => Some(AppTheme::Dark),
        _ => None,
    });

    let cli_lang = cli_args.lang.as_deref().and_then(|l| match l.to_lowercase().as_str() {
        "de" | "german" | "deutsch" => Some(Language::De),
        "en" | "english" => Some(Language::En),
        _ => None,
    });

    let resolved_path = resolve_cli_path(cli_args.path.as_ref());
    let initial_zen = cli_args.zen;

    // Central application state store
    let mut store = use_signal(move || {
        AppStore::new_with_options(resolved_path.as_deref(), cli_theme, cli_lang, initial_zen)
    });

    use_effect(move || {
        kick_pending_tree_scan(store);
    });

    // Dynamically apply OS native glass / acrylic / mica effect based on active theme
    let mut last_applied_dark = use_signal(|| None::<bool>);
    use_effect(move || {
        let current_theme = store().theme;
        let is_dark = current_theme.is_dark();
        let theme_class = current_theme.as_str();
        if last_applied_dark() != Some(is_dark) {
            last_applied_dark.set(Some(is_dark));
            let win = dioxus::desktop::window();

            #[cfg(target_os = "windows")]
            {
                let _ = window_vibrancy::apply_mica(&**win, Some(is_dark));
            }

            #[cfg(target_os = "macos")]
            {
                let material = if is_dark {
                    window_vibrancy::NSVisualEffectMaterial::FullScreenUI
                } else {
                    window_vibrancy::NSVisualEffectMaterial::WindowBackground
                };
                let _ = window_vibrancy::apply_vibrancy(&**win, material, None, None);
            }
        }

        let data_theme = if is_dark { "dark" } else { "light" };
        let _ = document::eval(&format!(
            r#"
            document.documentElement.setAttribute('data-theme', '{data_theme}');
            for (const cls of Array.from(document.documentElement.classList)) {{
                if (cls.startsWith('theme-')) document.documentElement.classList.remove(cls);
            }}
            document.documentElement.classList.add('{theme_class}');
            "#
        ));
    });

    // File watcher setup attached to central store (respects auto_reload setting)
    let _watcher_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            if let Ok((mut watcher, _tx)) = LiveFileWatcher::new() {
                loop {
                    tokio::time::sleep(Duration::from_millis(600)).await;

                    let s = store();
                    if s.settings.auto_reload {
                        if let Some(active_tab) = s.active_tab() {
                            if let Some(ref path) = active_tab.path {
                                let _ = watcher.watch_path(path);
                            }
                        }

                        while let Ok(changed_path) = watcher.receiver.try_recv() {
                            if let Ok(new_content) = read_document_file(&changed_path) {
                                store.write().update_file_content_if_modified(&changed_path, &new_content);
                            }
                        }
                    }
                }
            }
        }
    });

    // Background GitHub release auto-checker on application startup
    let _update_checker_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            let should_check = store().settings.auto_check_updates;
            if should_check {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let res = tokio::task::spawn_blocking(updater::check_github_release).await;
                if let Ok(Ok(Some(release))) = res {
                    store.write().set_update_status(UpdateStatus::Available(release));
                }
            }
        }
    });

    // Global Window-level Shortcut Listener Task
    let _global_shortcuts_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            let mut eval = dioxus::prelude::document::eval(r#"
                window.__globalShortcutHandler = function(payload) {
                    dioxus.send(payload);
                };
            "#);

            while let Ok(evt) = eval.recv::<GlobalShortcutPayload>().await {
                if evt.key == "Escape" {
                    crate::state::handle_escape_action(store);
                } else {
                    let s = store();
                    if let Some(action) = s.settings.shortcuts.match_event(
                        &evt.key,
                        evt.ctrl_key,
                        evt.alt_key,
                        evt.shift_key,
                        evt.meta_key,
                    ) {
                        crate::state::execute_shortcut_action(store, action);
                    }
                }
            }
        }
    });

    // Global Document Content Sync Listener Task (WYSIWYG, interactive checkboxes, etc.)
    let _doc_sync_task = use_coroutine(move |_: UnboundedReceiver<()>| {
        to_owned![store];
        async move {
            let mut eval = dioxus::prelude::document::eval(r#"
                window.__docSyncHandler = function(payload) {
                    dioxus.send(payload);
                };
                window.__wysiwygChangeHandler = function(markdown) {
                    dioxus.send({ action: 'set_markdown', content: markdown });
                };
            "#);

            while let Ok(payload) = eval.recv::<DocSyncPayload>().await {
                match payload {
                    DocSyncPayload::SetMarkdown { content } => {
                        store.write().update_active_tab_content(content);
                    }
                    DocSyncPayload::ToggleCheckbox { index, checked } => {
                        store.write().toggle_active_tab_task(index, checked);
                    }
                }
            }
        }
    });


    let store_read = store();
    let current_theme_class = store_read.theme.as_str();
    let is_zen = store_read.is_zen;
    let is_full_width = store_read.is_full_width;
    let zoom_level = store_read.zoom_level;
    let show_sidebar = store_read.show_sidebar;
    let document_mode = store_read.mode;

    let effective_accent = store_read.effective_primary_color();
    let accent_text_color = crate::types::accent_contrast_text_color(effective_accent);
    let root_style = store_read.primary_color.as_ref().map_or_else(
        || format!("--accent-text: {accent_text_color};"),
        |color| {
            format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40; --accent-text: {accent_text_color};")
        },
    );

    let active_tab_opt = store_read.active_tab().cloned();
    let t = store_read.language.strings();

    rsx! {
        div {
            class: format!("app-root {current_theme_class}"),
            style: "{root_style}",
            tabindex: 0,

            // Floating Zen Exit Button (visible only in Zen mode)
            if is_zen {
                ZenExitButton {
                    language: store_read.language,
                    on_exit: move |()| {
                        store.write().set_zen(false);
                    },
                }
            }

            // Custom Window Title Bar (hidden in Zen mode)
            if !is_zen {
                TitleBar {
                    store: store,
                }
            }

            if store_read.show_settings_modal {
                SettingsModal {
                    store: store,
                }
            } else {
                // Top Toolbar (hidden in Zen mode)
                if !is_zen {
                    Toolbar {
                        store: store,
                    }
                }

                // Main Workspace Layout
                div {
                    class: if is_zen { "app-workspace-body zen-active" } else { "app-workspace-body" },

                    WorkspaceSplit {
                        store: store,
                        show_sidebar: show_sidebar && !is_zen,
                        sidebar_position: store_read.sidebar_position,
                        sidebar: rsx! {
                            Sidebar {
                                store: store,
                                on_select_heading: move |id: String| {
                                    dioxus::prelude::document::eval(&format!(
                                        "window.scrollToSection && window.scrollToSection({id:?});"
                                    ));
                                },
                            }
                        },
                        content: rsx! {
                            if !is_zen {
                                TabBar {
                                    store: store,
                                }
                            }
                            if let Some(ref active_tab) = active_tab_opt {
                                if document_mode == DocumentMode::View {
                                    Viewer {
                                        tab_id: active_tab.id,
                                        document: active_tab.parsed.clone(),
                                        is_full_width: is_full_width,
                                        zoom_level: zoom_level,
                                        sticky_headers: store_read.sticky_headers,
                                        language: store_read.language,
                                    }
                                } else {
                                    Editor {
                                        tab_id: active_tab.id,
                                        store: store,
                                        mode: document_mode,
                                        document: active_tab.parsed.clone(),
                                        raw_content: active_tab.content.clone(),
                                        is_full_width: is_full_width,
                                        zoom_level: zoom_level,
                                        sticky_headers: store_read.sticky_headers,
                                        language: store_read.language,
                                    }
                                }
                            } else {
                                div {
                                    class: "flex-1 flex flex-col items-center justify-center h-full text-[var(--text-muted)] select-none p-8 bg-[var(--bg-app)]",
                                    div {
                                        class: "flex flex-col items-center gap-4 max-w-sm text-center",
                                        div {
                                            class: "w-14 h-14 rounded-2xl bg-[var(--bg-subtle)] border border-[var(--border-color)] flex items-center justify-center text-[var(--accent)] shadow-sm",
                                            Icon { width: 28, height: 28, icon: LdFileText }
                                        }
                                        div {
                                            h3 { class: "text-base font-semibold text-[var(--text-heading)] m-0", "{t.tab_bar.no_open_tabs}" }
                                            p { class: "text-xs text-[var(--text-muted)] mt-1 mb-0", "{t.tab_bar.no_open_tabs_desc}" }
                                        }
                                        div {
                                            class: "flex flex-col gap-2 w-full mt-2",
                                            button {
                                                class: "inline-flex items-center justify-center gap-2 px-4 py-2 bg-[var(--accent)] hover:brightness-110 text-[var(--accent-text)] text-xs font-semibold rounded-lg shadow cursor-pointer transition-all border-0",
                                                onclick: move |_| store.write().new_empty_tab(),
                                                Icon { width: 14, height: 14, icon: LdPlus }
                                                span { "{t.toolbar.new_tab}" }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                // Bottom Status Bar (hidden in Zen mode)
                if !is_zen {
                    if let Some(ref active_tab) = active_tab_opt {
                        StatusBar {
                            title: active_tab.title.clone(),
                            file_path: active_tab.path.clone(),
                            document: active_tab.parsed.clone(),
                            raw_content: active_tab.content.clone(),
                            mode: document_mode,
                            is_dirty: active_tab.is_dirty,
                            zoom_level: zoom_level,
                            language: store_read.language,
                            on_cycle_mode: move |()| store.write().cycle_mode(),
                        }
                    } else {
                        footer {
                            class: "app-status-bar flex items-center justify-between h-6.5 min-h-[26px] bg-[var(--bg-surface)] border-t border-[var(--border-color)] px-3 text-xs text-[var(--text-muted)] font-mono select-none z-50",
                            div {
                                class: "status-left-group flex items-center gap-2 min-w-0",
                                span { class: "status-item opacity-60 text-[11px]", "{t.tab_bar.no_open_tabs}" }
                            }
                            div {
                                class: "status-right-group flex items-center gap-2 shrink-0",
                                span { class: "status-badge bg-[var(--bg-subtle)] text-[var(--text-muted)] px-1.5 py-0.5 rounded text-[10px] uppercase font-semibold", "UTF-8" }
                            }
                        }
                    }
                }
            }
        }
    }
}


