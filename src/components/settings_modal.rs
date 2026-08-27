use crate::services::settings::{get_settings_file_path, open_settings_in_editor, reveal_settings_folder};
use crate::state::AppStore;
use crate::types::{AppTheme, SidebarTab};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdBookOpen, LdCheck, LdCopy, LdExternalLink, LdFileCode2, LdFolderOpen, LdFolderTree,
    LdPalette, LdRotateCcw, LdSettings, LdSparkles, LdX, LdZap,
};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct SettingsModalProps {
    pub store: Signal<AppStore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Appearance,
    Reader,
    Workspace,
    ConfigFile,
}

#[component]
pub fn SettingsModal(props: SettingsModalProps) -> Element {
    let mut current_tab = use_signal(|| SettingsTab::Appearance);
    let mut copy_feedback = use_signal(|| false);
    let mut store = props.store;
    let store_read = store();

    let catppuccin_themes = [
        (AppTheme::CatppuccinMocha, "Mocha", "#cba6f7", "#1e1e2e"),
        (AppTheme::CatppuccinMacchiato, "Macchiato", "#c6a0f6", "#24273a"),
        (AppTheme::CatppuccinFrappe, "Frappé", "#ca9ee6", "#303446"),
        (AppTheme::CatppuccinLatte, "Latte", "#8839ef", "#eff1f5"),
    ];

    let classic_themes = [
        (AppTheme::Dark, "GitHub Dark", "#58a6ff", "#161b22"),
        (AppTheme::Midnight, "Midnight", "#8b5cf6", "#12141c"),
        (AppTheme::Light, "GitHub Light", "#0969da", "#f6f8fa"),
        (AppTheme::Nord, "Nordic Frost", "#88c0d0", "#3b4252"),
        (AppTheme::SolarizedDark, "Solarized", "#268bd2", "#073642"),
    ];

    let accent_presets = [
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
    ];

    let settings_path = get_settings_file_path();
    let settings_path_display = settings_path.to_string_lossy().to_string();
    let active_accent = store_read.effective_primary_color().to_string();
    let has_custom_accent = store_read.primary_color.is_some();
    let active_tab_enum = current_tab();

    rsx! {
        div {
            class: "settings-modal-overlay fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[200] animate-fade-in p-4",
            onclick: move |_| {
                store.write().set_settings_modal(false);
            },

            div {
                class: "settings-modal-dialog bg-[var(--popover-bg)] border border-[var(--border-color)] rounded-2xl shadow-2xl w-full max-w-[700px] max-h-[85vh] flex flex-col overflow-hidden animate-scale-up",
                onclick: move |evt| evt.stop_propagation(),

                // Modal Header
                div {
                    class: "settings-modal-header flex items-center justify-between px-6 py-4 border-b border-[var(--border-color)] bg-[var(--bg-surface)]",
                    div {
                        class: "flex items-center gap-2.5",
                        Icon {
                            width: 20,
                            height: 20,
                            icon: LdSettings,
                            class: "text-[var(--accent)] shrink-0",
                        }
                        div {
                            h2 { class: "text-base font-bold text-[var(--text-heading)] m-0 leading-tight", "Preferences & Settings" }
                            p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Saved automatically to settings.json" }
                        }
                    }
                    button {
                        class: "settings-close-btn w-8 h-8 rounded-lg flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-heading)] hover:bg-[var(--bg-hover)] transition-colors cursor-pointer border-0 bg-transparent",
                        title: "Close (Esc)",
                        onclick: move |_| {
                            store.write().set_settings_modal(false);
                        },
                        Icon { width: 16, height: 16, icon: LdX }
                    }
                }

                // Modal Navigation Tabs
                div {
                    class: "settings-nav-tabs flex border-b border-[var(--border-color)] bg-[var(--bg-subtle)] px-6 gap-2",
                    button {
                        class: if active_tab_enum == SettingsTab::Appearance { "settings-tab-btn active py-2.5 px-3 border-b-2 border-[var(--accent)] text-[var(--accent)] text-xs font-semibold cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" } else { "settings-tab-btn py-2.5 px-3 border-b-2 border-transparent text-[var(--text-muted)] hover:text-[var(--text-heading)] text-xs font-medium cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" },
                        onclick: move |_| current_tab.set(SettingsTab::Appearance),
                        Icon { width: 14, height: 14, icon: LdPalette }
                        span { "Appearance" }
                    }
                    button {
                        class: if active_tab_enum == SettingsTab::Reader { "settings-tab-btn active py-2.5 px-3 border-b-2 border-[var(--accent)] text-[var(--accent)] text-xs font-semibold cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" } else { "settings-tab-btn py-2.5 px-3 border-b-2 border-transparent text-[var(--text-muted)] hover:text-[var(--text-heading)] text-xs font-medium cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" },
                        onclick: move |_| current_tab.set(SettingsTab::Reader),
                        Icon { width: 14, height: 14, icon: LdBookOpen }
                        span { "Reader & Layout" }
                    }
                    button {
                        class: if active_tab_enum == SettingsTab::Workspace { "settings-tab-btn active py-2.5 px-3 border-b-2 border-[var(--accent)] text-[var(--accent)] text-xs font-semibold cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" } else { "settings-tab-btn py-2.5 px-3 border-b-2 border-transparent text-[var(--text-muted)] hover:text-[var(--text-heading)] text-xs font-medium cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" },
                        onclick: move |_| current_tab.set(SettingsTab::Workspace),
                        Icon { width: 14, height: 14, icon: LdFolderTree }
                        span { "Workspace & Sidebar" }
                    }
                    button {
                        class: if active_tab_enum == SettingsTab::ConfigFile { "settings-tab-btn active py-2.5 px-3 border-b-2 border-[var(--accent)] text-[var(--accent)] text-xs font-semibold cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" } else { "settings-tab-btn py-2.5 px-3 border-b-2 border-transparent text-[var(--text-muted)] hover:text-[var(--text-heading)] text-xs font-medium cursor-pointer bg-transparent border-t-0 border-x-0 transition-all inline-flex items-center gap-1.5" },
                        onclick: move |_| current_tab.set(SettingsTab::ConfigFile),
                        Icon { width: 14, height: 14, icon: LdFileCode2 }
                        span { "Settings File" }
                    }
                }


                // Modal Body Content Area
                div {
                    class: "settings-modal-body flex-1 overflow-y-auto p-6 flex flex-col gap-6",

                    // --- TAB 1: APPEARANCE ---
                    if active_tab_enum == SettingsTab::Appearance {
                        div {
                            class: "settings-section flex flex-col gap-4",

                            div {
                                class: "section-header",
                                h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "Theme Presets" }
                                p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Select your preferred color scheme and glassmorphism styling." }
                            }

                            // Catppuccin Themes
                            div {
                                class: "theme-group flex flex-col gap-1.5",
                                span {
                                    class: "text-[11px] font-semibold text-[var(--text-muted)] uppercase tracking-wider inline-flex items-center gap-1.5",
                                    Icon { width: 12, height: 12, icon: LdSparkles, class: "text-[var(--accent)]" }
                                    "Catppuccin Flavors"
                                }
                                div {
                                    class: "grid grid-cols-2 sm:grid-cols-4 gap-2",
                                    for (theme_item, label, accent_color, bg_color) in catppuccin_themes {
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

                            // Classic Themes
                            div {
                                class: "theme-group flex flex-col gap-1.5 mt-2",
                                span {
                                    class: "text-[11px] font-semibold text-[var(--text-muted)] uppercase tracking-wider inline-flex items-center gap-1.5",
                                    Icon { width: 12, height: 12, icon: LdZap, class: "text-[var(--accent)]" }
                                    "Classic Themes"
                                }
                                div {
                                    class: "grid grid-cols-2 sm:grid-cols-3 gap-2",
                                    for (theme_item, label, accent_color, bg_color) in classic_themes {
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

                            div { class: "w-full h-[1px] bg-[var(--border-color)] my-2" }

                            // Primary Accent Color Picker
                            div {
                                class: "section-header flex items-center justify-between",
                                div {
                                    h3 {
                                        class: "text-sm font-semibold text-[var(--text-heading)] m-0 inline-flex items-center gap-1.5",
                                        Icon { width: 14, height: 14, icon: LdPalette, class: "text-[var(--accent)]" }
                                        "Primary Accent Color"
                                    }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Override the primary UI highlight and link color." }
                                }
                                if has_custom_accent {
                                    button {
                                        class: "inline-flex items-center gap-1 text-xs text-[var(--accent)] hover:underline cursor-pointer bg-transparent border-0 p-0",
                                        onclick: move |_| store.write().set_primary_color(None),
                                        "Reset to Theme Default"
                                    }
                                }
                            }

                            // Accent Presets Grid
                            div {
                                class: "grid grid-cols-6 sm:grid-cols-12 gap-1.5",
                                for (hex, name) in accent_presets {
                                    div {
                                        class: if active_accent.eq_ignore_ascii_case(hex) { "accent-color-chip active-chip w-full aspect-square rounded-lg cursor-pointer flex items-center justify-center ring-2 ring-white/60 transition-transform scale-105" } else { "accent-color-chip w-full aspect-square rounded-lg cursor-pointer flex items-center justify-center hover:scale-105 transition-transform" },
                                        style: "background-color: {hex};",
                                        title: "{name} ({hex})",
                                        onclick: move |_| store.write().set_primary_color(Some(hex.to_string())),
                                        if active_accent.eq_ignore_ascii_case(hex) {
                                            span { class: "w-2 h-2 rounded-full bg-white shadow-sm" }
                                        }
                                    }
                                }
                            }

                            // Custom Hex Color Picker Row
                            div {
                                class: "flex items-center gap-3 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl p-3",
                                div {
                                    class: "w-8 h-8 rounded-lg border border-[var(--border-color)] overflow-hidden shrink-0 relative cursor-pointer",
                                    style: "background-color: {active_accent};",
                                    input {
                                        class: "opacity-0 absolute inset-0 w-full h-full cursor-pointer",
                                        r#type: "color",
                                        value: "{active_accent}",
                                        oninput: move |evt| store.write().set_primary_color(Some(evt.value())),
                                    }
                                }
                                input {
                                    class: "bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg px-3 py-1.5 text-xs font-mono text-[var(--text-heading)] flex-1 outline-none focus:border-[var(--accent)]",
                                    r#type: "text",
                                    placeholder: "#hexcode",
                                    value: "{active_accent}",
                                    oninput: move |evt| {
                                        let val = evt.value().trim().to_string();
                                        if val.starts_with('#') && (val.len() == 7 || val.len() == 4) {
                                            store.write().set_primary_color(Some(val));
                                        }
                                    },
                                }
                            }
                        }
                    }

                    // --- TAB 2: READER & LAYOUT ---
                    if active_tab_enum == SettingsTab::Reader {
                        div {
                            class: "settings-section flex flex-col gap-5",

                            // Layout Mode
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Document Reading Layout" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Centered optimal line-length column or expansive full-width." }
                                }
                                div {
                                    class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                                    button {
                                        class: if !store_read.is_full_width { "h-7 px-3 rounded-md bg-[var(--bg-surface)] text-[var(--accent)] font-semibold text-xs border-0 cursor-pointer shadow-sm" } else { "h-7 px-3 rounded-md bg-transparent text-[var(--text-muted)] text-xs border-0 cursor-pointer hover:text-[var(--text-heading)]" },
                                        onclick: move |_| {
                                            if store_read.is_full_width {
                                                store.write().toggle_full_width();
                                            }
                                        },
                                        "Reading Column"
                                    }
                                    button {
                                        class: if store_read.is_full_width { "h-7 px-3 rounded-md bg-[var(--bg-surface)] text-[var(--accent)] font-semibold text-xs border-0 cursor-pointer shadow-sm" } else { "h-7 px-3 rounded-md bg-transparent text-[var(--text-muted)] text-xs border-0 cursor-pointer hover:text-[var(--text-heading)]" },
                                        onclick: move |_| {
                                            if !store_read.is_full_width {
                                                store.write().toggle_full_width();
                                            }
                                        },
                                        "Full Width"
                                    }
                                }
                            }

                            // Zoom Level
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Default Viewer Zoom" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Scale typography and document images." }
                                }
                                div {
                                    class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg h-7 px-1",
                                    button {
                                        class: "w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] cursor-pointer text-sm font-bold",
                                        title: "Zoom Out",
                                        onclick: move |_| store.write().zoom_out(),
                                        "-"
                                    }
                                    span {
                                        class: "px-2.5 text-xs font-mono font-semibold text-[var(--text-heading)] cursor-pointer hover:text-[var(--accent)]",
                                        title: "Reset Zoom (100%)",
                                        onclick: move |_| store.write().reset_zoom(),
                                        "{store_read.zoom_level}%"
                                    }
                                    button {
                                        class: "w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] cursor-pointer text-sm font-bold",
                                        title: "Zoom In",
                                        onclick: move |_| store.write().zoom_in(),
                                        "+"
                                    }
                                }
                            }

                            // Base Font Size
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Base Document Font Size" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Typography size for paragraphs and body content." }
                                }
                                div {
                                    class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                                    for size in [14, 16, 18, 20] {
                                        button {
                                            class: if store_read.settings.font_size == size { "h-7 px-2.5 rounded-md bg-[var(--bg-surface)] text-[var(--accent)] font-semibold text-xs border-0 cursor-pointer shadow-sm" } else { "h-7 px-2.5 rounded-md bg-transparent text-[var(--text-muted)] text-xs border-0 cursor-pointer hover:text-[var(--text-heading)]" },
                                            onclick: move |_| store.write().set_font_size(size),
                                            "{size}px"
                                        }
                                    }
                                }
                            }

                            // Auto Reload on Save
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Live Auto-Reload" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Automatically reload and re-render document when modified on disk." }
                                }
                                button {
                                    class: if store_read.settings.auto_reload { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--accent)] transition-colors duration-200 ease-in-out focus:outline-none" } else { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--bg-subtle)] border-[var(--border-color)] transition-colors duration-200 ease-in-out focus:outline-none" },
                                    onclick: move |_| {
                                        let current = store().settings.auto_reload;
                                        store.write().set_auto_reload(!current);
                                    },
                                    span {
                                        class: if store_read.settings.auto_reload { "pointer-events-none inline-block h-5 w-5 translate-x-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out" } else { "pointer-events-none inline-block h-5 w-5 translate-x-0 transform rounded-full bg-gray-400 shadow ring-0 transition duration-200 ease-in-out" },
                                    }
                                }
                            }

                            // Sticky Markdown Headings
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Sticky Markdown Headings" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Keep section headings (H1-H6) pinned to the top while scrolling." }
                                }
                                button {
                                    class: if store_read.settings.sticky_headers { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--accent)] transition-colors duration-200 ease-in-out focus:outline-none" } else { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--bg-subtle)] border-[var(--border-color)] transition-colors duration-200 ease-in-out focus:outline-none" },
                                    onclick: move |_| {
                                        let current = store().settings.sticky_headers;
                                        store.write().set_sticky_headers(!current);
                                    },
                                    span {
                                        class: if store_read.settings.sticky_headers { "pointer-events-none inline-block h-5 w-5 translate-x-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out" } else { "pointer-events-none inline-block h-5 w-5 translate-x-0 transform rounded-full bg-gray-400 shadow ring-0 transition duration-200 ease-in-out" },
                                    }
                                }
                            }
                        }
                    }

                    // --- TAB 3: WORKSPACE & SIDEBAR ---
                    if active_tab_enum == SettingsTab::Workspace {
                        div {
                            class: "settings-section flex flex-col gap-5",

                            // Default Sidebar Tab
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Default Sidebar Tab" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Initial mode when opening the sidebar." }
                                }
                                div {
                                    class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                                    button {
                                        class: if store_read.sidebar_tab == SidebarTab::Toc { "h-7 px-3 rounded-md bg-[var(--bg-surface)] text-[var(--accent)] font-semibold text-xs border-0 cursor-pointer shadow-sm" } else { "h-7 px-3 rounded-md bg-transparent text-[var(--text-muted)] text-xs border-0 cursor-pointer hover:text-[var(--text-heading)]" },
                                        onclick: move |_| store.write().set_sidebar_tab(SidebarTab::Toc),
                                        "Outline (TOC)"
                                    }
                                    button {
                                        class: if store_read.sidebar_tab == SidebarTab::Files { "h-7 px-3 rounded-md bg-[var(--bg-surface)] text-[var(--accent)] font-semibold text-xs border-0 cursor-pointer shadow-sm" } else { "h-7 px-3 rounded-md bg-transparent text-[var(--text-muted)] text-xs border-0 cursor-pointer hover:text-[var(--text-heading)]" },
                                        onclick: move |_| store.write().set_sidebar_tab(SidebarTab::Files),
                                        "File Explorer"
                                    }
                                }
                            }

                            // Show Sidebar by Default
                            div {
                                class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                                div {
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Sidebar Visibility" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Show sidebar outline/explorer on launch." }
                                }
                                button {
                                    class: if store_read.show_sidebar { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--accent)] transition-colors duration-200 ease-in-out focus:outline-none" } else { "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-[var(--bg-subtle)] border-[var(--border-color)] transition-colors duration-200 ease-in-out focus:outline-none" },
                                    onclick: move |_| store.write().toggle_sidebar(),
                                    span {
                                        class: if store_read.show_sidebar { "pointer-events-none inline-block h-5 w-5 translate-x-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out" } else { "pointer-events-none inline-block h-5 w-5 translate-x-0 transform rounded-full bg-gray-400 shadow ring-0 transition duration-200 ease-in-out" },
                                    }
                                }
                            }

                            // Recent History Summary
                            div {
                                class: "p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl flex flex-col gap-2",
                                div {
                                    class: "flex items-center justify-between",
                                    h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "Recent Files & Workspaces" }
                                    span { class: "text-[11px] font-mono text-[var(--text-muted)]", "{store_read.settings.recent_files.len()} files / {store_read.settings.recent_folders.len()} folders" }
                                }
                                p { class: "text-xs text-[var(--text-muted)] m-0", "Quickly reopen recent markdown documents and project trees from memory." }
                            }
                        }
                    }

                    // --- TAB 4: CONFIG FILE & MANAGEMENT ---
                    if active_tab_enum == SettingsTab::ConfigFile {
                        div {
                            class: "settings-section flex flex-col gap-4",

                            div {
                                class: "section-header",
                                h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0", "Configuration File Location" }
                                p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Preferences are stored in a human-readable JSON format. You can edit this file in your preferred editor." }
                            }

                            // File Path Pill Box
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
                                    button {
                                        class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)] font-medium hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                                        onclick: move |_| {
                                            let p = settings_path_display.clone();
                                            dioxus::prelude::document::eval(&format!("navigator.clipboard && navigator.clipboard.writeText({p:?});"));
                                            copy_feedback.set(true);
                                        },
                                        if copy_feedback() {
                                            Icon {
                                                width: 13,
                                                height: 13,
                                                icon: LdCheck,
                                            }
                                        } else {
                                            Icon {
                                                width: 13,
                                                height: 13,
                                                icon: LdCopy,
                                            }
                                        }
                                        span { if copy_feedback() { "Copied Path" } else { "Copy Path" } }
                                    }
                                    button {
                                        class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)] font-medium hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                                        onclick: move |_| open_settings_in_editor(),
                                        Icon { width: 13, height: 13, icon: LdExternalLink }
                                        span { "Open in Editor" }
                                    }
                                    button {
                                        class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-[var(--bg-subtle)] border border-[var(--border-color)] text-xs text-[var(--text-heading)] font-medium hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                                        onclick: move |_| reveal_settings_folder(),
                                        Icon { width: 13, height: 13, icon: LdFolderOpen }
                                        span { "Show in Folder" }
                                    }
                                }
                            }

                            div { class: "w-full h-[1px] bg-[var(--border-color)] my-1" }

                            // Factory Reset Option
                            div {
                                class: "p-3.5 bg-red-950/20 border border-red-900/40 rounded-xl flex items-center justify-between",
                                div {
                                    h4 { class: "text-xs font-semibold text-red-400 m-0", "Reset All Preferences" }
                                    p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "Restore factory default theme, zoom level, and settings." }
                                }
                                button {
                                    class: "inline-flex items-center gap-1.5 h-7 px-3 rounded-lg bg-red-600/80 hover:bg-red-600 text-white text-xs font-medium border-0 cursor-pointer transition-colors shadow-sm",
                                    onclick: move |_| store.write().reset_settings_to_default(),
                                    Icon { width: 13, height: 13, icon: LdRotateCcw }
                                    span { "Reset Defaults" }
                                }
                            }
                        }
                    }
                }

                // Modal Footer
                div {
                    class: "settings-modal-footer flex items-center justify-between px-6 py-3.5 border-t border-[var(--border-color)] bg-[var(--bg-surface)]",
                    span { class: "text-[11px] font-mono text-[var(--text-muted)]", "Fast-MD v0.1.0 • Native Dioxus" }
                    button {
                        class: "h-8 px-5 rounded-lg bg-[var(--accent)] hover:opacity-90 text-white text-xs font-semibold cursor-pointer border-0 transition-opacity shadow-md",
                        onclick: move |_| store.write().set_settings_modal(false),
                        "Done"
                    }
                }
            }
        }
    }
}
