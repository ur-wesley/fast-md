mod cli;
mod components;
mod services;
mod state;
mod types;

use cli::CliArgs;
use components::{SettingsModal, Sidebar, StatusBar, TabBar, TitleBar, Toolbar, Viewer, ZenExitButton};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use services::fs::{pick_file_async, read_document_file};
use services::watcher::LiveFileWatcher;
use state::AppStore;
use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use types::AppTheme;

static CLI_ARGS: OnceLock<CliArgs> = OnceLock::new();

const APP_STYLES: &str = include_str!("assets/style.css");

const HELPER_JS: &str = r"
window.copyCodeSnippet = function(btn) {
    const code = btn.getAttribute('data-code');
    if (code) {
        navigator.clipboard.writeText(code).then(() => {
            const span = btn.querySelector('span');
            if (span) {
                const orig = span.innerText;
                span.innerText = 'Copied!';
                setTimeout(() => { span.innerText = orig; }, 1800);
            }
        }).catch(err => console.error('Copy failed:', err));
    }
};

window.scrollToSection = function(id) {
    const el = document.getElementById(id);
    if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
};

// --- In-Document Search & Highlighting Engine ---
window._searchState = {
    matches: [],
    currentIndex: -1,
    query: ''
};

window.clearSearchHighlights = function() {
    const marks = document.querySelectorAll('mark.fastmd-search-match');
    marks.forEach(mark => {
        const parent = mark.parentNode;
        if (parent) {
            parent.replaceChild(document.createTextNode(mark.textContent), mark);
            parent.normalize();
        }
    });
    window._searchState.matches = [];
    window._searchState.currentIndex = -1;
    window._searchState.query = '';
    window.updateSearchCountUI(0, 0);
};

window.highlightSearchMatches = function(query) {
    window.clearSearchHighlights();
    if (!query || query.trim() === '') {
        return;
    }

    const root = document.querySelector('.app-main-viewer') || document.querySelector('.markdown-body');
    if (!root) return;

    window._searchState.query = query;
    const lowerQuery = query.toLowerCase();

    function walkTextNodes(node, callback) {
        if (node.nodeType === Node.TEXT_NODE) {
            callback(node);
        } else if (node.nodeType === Node.ELEMENT_NODE) {
            if (['SCRIPT', 'STYLE', 'BUTTON', 'INPUT', 'HEADER', 'NAV'].includes(node.tagName) || node.classList.contains('app-titlebar') || node.classList.contains('app-toolbar')) {
                return;
            }
            Array.from(node.childNodes).forEach(child => walkTextNodes(child, callback));
        }
    }

    const textNodes = [];
    walkTextNodes(root, n => textNodes.push(n));

    const matches = [];
    textNodes.forEach(textNode => {
        const text = textNode.textContent;
        const lowerText = text.toLowerCase();
        let startIndex = 0;
        let index = lowerText.indexOf(lowerQuery, startIndex);

        if (index === -1) return;

        const fragment = document.createDocumentFragment();
        let lastIdx = 0;

        while (index !== -1) {
            if (index > lastIdx) {
                fragment.appendChild(document.createTextNode(text.substring(lastIdx, index)));
            }

            const mark = document.createElement('mark');
            mark.className = 'fastmd-search-match';
            mark.textContent = text.substring(index, index + query.length);
            fragment.appendChild(mark);
            matches.push(mark);

            lastIdx = index + query.length;
            startIndex = lastIdx;
            index = lowerText.indexOf(lowerQuery, startIndex);
        }

        if (lastIdx < text.length) {
            fragment.appendChild(document.createTextNode(text.substring(lastIdx)));
        }

        if (textNode.parentNode) {
            textNode.parentNode.replaceChild(fragment, textNode);
        }
    });

    window._searchState.matches = matches;
    if (matches.length > 0) {
        window._searchState.currentIndex = 0;
        window.activateMatch(0);
    } else {
        window.updateSearchCountUI(0, 0);
    }
};

window.activateMatch = function(index) {
    const s = window._searchState;
    if (!s.matches || s.matches.length === 0) return;

    if (index < 0) index = s.matches.length - 1;
    if (index >= s.matches.length) index = 0;
    s.currentIndex = index;

    s.matches.forEach((m, idx) => {
        if (idx === index) {
            m.classList.add('active-match');
            m.scrollIntoView({ behavior: 'smooth', block: 'center' });
        } else {
            m.classList.remove('active-match');
        }
    });

    window.updateSearchCountUI(s.currentIndex + 1, s.matches.length);
};

window.searchNextMatch = function() {
    const s = window._searchState;
    if (s.matches && s.matches.length > 0) {
        window.activateMatch(s.currentIndex + 1);
    }
};

window.searchPrevMatch = function() {
    const s = window._searchState;
    if (s.matches && s.matches.length > 0) {
        window.activateMatch(s.currentIndex - 1);
    }
};

window.updateSearchCountUI = function(current, total) {
    const el = document.getElementById('search-match-count');
    if (el) {
        if (total === 0) {
            el.innerText = window._searchState && window._searchState.query ? '0 results' : '';
        } else {
            el.innerText = `${current} / ${total}`;
        }
    }
};

// Global Shortcut Interceptor (captures Ctrl+F / Cmd+F anywhere in the window)
function handleSearchShortcut(e) {
    if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F' || e.code === 'KeyF' || e.keyCode === 70)) {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        const input = document.getElementById('titlebar-search-input');
        if (input) {
            input.focus();
            input.select();
        }
        return false;
    }
}
window.addEventListener('keydown', handleSearchShortcut, true);
document.addEventListener('keydown', handleSearchShortcut, true);
";

fn resolve_cli_path(raw_path: Option<&PathBuf>) -> Option<PathBuf> {
    raw_path.and_then(|p| {
        if p.is_absolute() && p.exists() {
            Some(p.clone())
        } else if let Ok(current_dir) = env::current_dir() {
            let combined = current_dir.join(p);
            if combined.exists() {
                Some(combined)
            } else if p.exists() {
                Some(p.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn main() {
    let args = CliArgs::parse_safe();

    if args.register {
        if services::association::register_file_associations() {
            println!("Successfully registered Fast-MD in Windows Explorer and Default Apps.");
        } else {
            eprintln!("Failed to register file associations.");
        }
        return;
    }

    if args.unregister {
        if services::association::unregister_file_associations() {
            println!("Successfully unregistered Fast-MD file associations.");
        } else {
            eprintln!("Failed to unregister file associations.");
        }
        return;
    }

    // Auto-register file associations on startup in background
    std::thread::spawn(|| {
        let _ = services::association::register_file_associations();
    });

    let _ = CLI_ARGS.set(args);

    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Fast-MD Viewer")
                .with_decorations(false)
                .with_transparent(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1180.0, 800.0))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(640.0, 420.0)),
        )
        .with_background_color((0, 0, 0, 0))
        .with_custom_head(format!("<script src=\"https://cdn.tailwindcss.com\"></script><style>{APP_STYLES}</style><script>{HELPER_JS}</script>"));

    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    let cli_args = CLI_ARGS.get().cloned().unwrap_or(CliArgs {
        path: None,
        zen: false,
        theme: None,
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

    let resolved_path = resolve_cli_path(cli_args.path.as_ref());
    let initial_zen = cli_args.zen;

    // Central application state store
    let mut store = use_signal(move || {
        AppStore::new_with_options(resolved_path.as_deref(), cli_theme, initial_zen)
    });

    // Dynamically apply OS native glass / acrylic / mica effect based on active theme
    use_effect(move || {
        let current_theme = store().theme;
        let win = dioxus::desktop::window();

        #[cfg(target_os = "windows")]
        {
            let is_dark = current_theme.is_dark();
            let _ = window_vibrancy::apply_mica(&**win, Some(is_dark));
        }

        #[cfg(target_os = "macos")]
        {
            let is_dark = current_theme.is_dark();
            let material = if is_dark {
                window_vibrancy::NSVisualEffectMaterial::HudWindow
            } else {
                window_vibrancy::NSVisualEffectMaterial::Light
            };
            let _ = window_vibrancy::apply_vibrancy(&**win, material, None, None);
        }
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

    let store_read = store();
    let current_theme_class = store_read.theme.as_str();
    let is_zen = store_read.is_zen;
    let is_full_width = store_read.is_full_width;
    let zoom_level = store_read.zoom_level;
    let show_sidebar = store_read.show_sidebar;
    let show_settings_modal = store_read.show_settings_modal;

    let root_style = store_read.primary_color.as_ref().map_or_else(String::new, |color| {
        format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40;")
    });

    let active_tab = store_read.active_tab().cloned().unwrap_or_else(|| state::AppStore::default().tabs.remove(0));

    rsx! {
        div {
            class: format!("app-root {current_theme_class}"),
            style: "{root_style}",
            tabindex: 0,
            onkeydown: move |evt| {
                let key = evt.key();
                let ctrl = evt.modifiers().ctrl();

                if key == Key::Escape {
                    let mut s = store.write();
                    if s.show_settings_modal {
                        s.set_settings_modal(false);
                    } else if s.is_zen {
                        s.set_zen(false);
                    } else if s.show_search {
                        s.show_search = false;
                    }
                } else if ctrl && (key == Key::Character(",".to_string()) || key == Key::Character("<".to_string())) {
                    store.write().toggle_settings_modal();
                } else if ctrl && key == Key::Character("o".to_string()) {
                    spawn(async move {
                        if let Some(path) = pick_file_async().await {
                            store.write().open_file_from_path(path);
                        }
                    });
                } else if ctrl && (key == Key::Character("f".to_string()) || key == Key::Character("F".to_string())) {
                    dioxus::prelude::document::eval(
                        r"
                        const input = document.getElementById('titlebar-search-input');
                        if (input) { input.focus(); input.select(); }
                        ",
                    );
                } else if ctrl && (key == Key::Character("=".to_string()) || key == Key::Character("+".to_string())) {
                    store.write().zoom_in();
                } else if ctrl && key == Key::Character("-".to_string()) {
                    store.write().zoom_out();
                } else if ctrl && key == Key::Character("0".to_string()) {
                    store.write().reset_zoom();
                } else if ctrl && evt.modifiers().shift() && key == Key::Character("F".to_string()) {
                    store.write().toggle_zen();
                } else if ctrl && key == Key::Character("t".to_string()) {
                    spawn(async move {
                        if let Some(path) = pick_file_async().await {
                            store.write().open_file_from_path(path);
                        } else {
                            store.write().new_empty_tab();
                        }
                    });
                } else if ctrl && key == Key::Character("w".to_string()) {
                    let current_id = store().active_tab_id;
                    store.write().close_tab(current_id);
                }
            },

            // Floating Zen Exit Button (visible only in Zen mode)
            if is_zen {
                ZenExitButton {
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

            // Top Toolbar (hidden in Zen mode)
            if !is_zen {
                Toolbar {
                    store: store,
                }
            }

            // Tab Bar (hidden in Zen mode)
            if !is_zen {
                TabBar {
                    store: store,
                }
            }

            // Main Workspace Layout
            div {
                class: if is_zen { "app-workspace-body zen-active" } else { "app-workspace-body" },

                // Sidebar
                if show_sidebar && !is_zen {
                    Sidebar {
                        store: store,
                        on_select_heading: move |id| {
                            dioxus::prelude::document::eval(&format!("window.scrollToSection && window.scrollToSection('{id}');"));
                        },
                    }
                }

                // Main Content Viewer
                Viewer {
                    document: active_tab.parsed.clone(),
                    is_full_width: is_full_width,
                    zoom_level: zoom_level,
                    sticky_headers: store_read.sticky_headers,
                }
            }

            // Bottom Status Bar (hidden in Zen mode)
            if !is_zen {
                StatusBar {
                    title: active_tab.title,
                    file_path: active_tab.path,
                    document: active_tab.parsed,
                    zoom_level: zoom_level,
                }
            }

            // Settings Modal Dialog
            if show_settings_modal {
                SettingsModal {
                    store: store,
                }
            }
        }
    }
}

