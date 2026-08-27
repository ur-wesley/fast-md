use crate::services::fs::{pick_export_html_async, pick_file_async, pick_folder_async, save_document_file};
use crate::state::AppStore;
use crate::types::AppTheme;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdPalette, LdSparkles};

const APP_STYLES: &str = include_str!("../assets/style.css");

#[derive(Props, Clone, PartialEq, Eq)]
pub struct ToolbarProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut show_theme_menu = use_signal(|| false);
    let mut store = props.store;
    let store_read = store();

    let catppuccin_themes = [
        (AppTheme::CatppuccinMocha, "Mocha", "#cba6f7", "#1e1e2e"),
        (AppTheme::CatppuccinMacchiato, "Macchiato", "#c6a0f6", "#24273a"),
        (AppTheme::CatppuccinFrappe, "Frappé", "#ca9ee6", "#303446"),
        (AppTheme::CatppuccinLatte, "Latte", "#8839ef", "#eff1f5"),
    ];

    let classic_themes = [
        (AppTheme::Dark, "Dark", "#58a6ff", "#161b22"),
        (AppTheme::Midnight, "Midnight", "#8b5cf6", "#12141c"),
        (AppTheme::Light, "Light", "#0969da", "#f6f8fa"),
        (AppTheme::Nord, "Nord", "#88c0d0", "#3b4252"),
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

    let active_accent = store_read.effective_primary_color().to_string();
    let has_custom_accent = store_read.primary_color.is_some();

    rsx! {
        header {
            class: "app-toolbar flex items-center justify-between h-10 min-h-[40px] px-3 bg-[var(--bg-surface)] border-b border-[var(--border-color)] z-50",

            // Left Action Group: Sidebar & File Management
            div {
                class: "toolbar-left-group flex items-center gap-2",

                // Sidebar Toggle
                button {
                    class: if store_read.show_sidebar { "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md text-[var(--accent)] text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "Toggle Outline / File Tree",
                    onclick: move |_| {
                        store.write().toggle_sidebar();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "15",
                        height: "15",
                        rect { width: "18", height: "18", x: "3", y: "3", rx: "2", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "M9 3v18", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "Sidebar" }
                    if store_read.show_sidebar {
                        span { class: "btn-indicator-dot w-1.5 h-1.5 rounded-full bg-[var(--accent)] ml-0.5" }
                    }
                }

                div { class: "toolbar-sep w-[1px] h-4.5 bg-[var(--border-color)] mx-0.5" }

                // Segmented File Controls
                div {
                    class: "toolbar-segmented-group inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md p-0.5 gap-0.5",
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "Open File (Ctrl+O)",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(path) = pick_file_async().await {
                                    store.write().open_file_from_path(path);
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", fill: "none", stroke: "currentColor", stroke_width: "2" }
                            path { d: "M14 2v6h6", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "Open" }
                    }
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "Open Folder / Workspace",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(dir) = pick_folder_async().await {
                                    store.write().open_directory(dir);
                                }
                            });
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "Folder" }
                    }
                    button {
                        class: "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "New Tab (Ctrl+T)",
                        onclick: move |_| {
                            store.write().new_empty_tab();
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "14",
                            height: "14",
                            path { d: "M12 5v14M5 12h14", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round" }
                        }
                        span { class: "btn-text", "New" }
                    }
                }
            }

            // Center Group: View Layout & Zoom Stepper
            div {
                class: "toolbar-center-group flex items-center gap-2",

                // Layout Switcher Segmented Control
                div {
                    class: "toolbar-segmented-group layout-toggle-group inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md p-0.5 gap-0.5",
                    button {
                        class: if !store_read.is_full_width { "segmented-btn active-segment inline-flex items-center gap-1 h-6 px-2 bg-[var(--bg-surface)] text-[var(--accent)] font-semibold shadow-sm border-0 rounded text-xs cursor-pointer transition-all duration-150" } else { "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                        title: "Centered Reading Column Layout",
                        onclick: move |_| {
                            if store_read.is_full_width {
                                store.write().toggle_full_width();
                            }
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            rect { x: "7", y: "3", width: "10", height: "18", rx: "1.5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "Column" }
                    }
                    button {
                        class: if store_read.is_full_width { "segmented-btn active-segment inline-flex items-center gap-1 h-6 px-2 bg-[var(--bg-surface)] text-[var(--accent)] font-semibold shadow-sm border-0 rounded text-xs cursor-pointer transition-all duration-150" } else { "segmented-btn inline-flex items-center gap-1 h-6 px-2 bg-transparent border-0 rounded text-[var(--text-muted)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                        title: "Full Width Layout",
                        onclick: move |_| {
                            if !store_read.is_full_width {
                                store.write().toggle_full_width();
                            }
                        },
                        svg {
                            class: "toolbar-btn-icon shrink-0",
                            view_box: "0 0 24 24",
                            width: "13",
                            height: "13",
                            rect { x: "3", y: "3", width: "18", height: "18", rx: "1.5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        }
                        span { class: "btn-text", "Full Width" }
                    }
                }

                // Zoom Stepper
                div {
                    class: "toolbar-zoom-stepper inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md h-7 px-0.5",
                    button {
                        class: "zoom-step-btn w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] rounded cursor-pointer flex items-center justify-center transition-all duration-150",
                        title: "Zoom Out (Ctrl+-)",
                        onclick: move |_| {
                            store.write().zoom_out();
                        },
                        svg {
                            view_box: "0 0 24 24",
                            width: "12",
                            height: "12",
                            path { d: "M5 12h14", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round" }
                        }
                    }
                    span {
                        class: "zoom-value-pill text-xs font-mono font-semibold px-2 text-[var(--text-main)] hover:text-[var(--accent)] cursor-pointer select-none transition-colors duration-150",
                        onclick: move |_| {
                            store.write().reset_zoom();
                        },
                        title: "Reset Zoom (Ctrl+0)",
                        "{store_read.zoom_level}%"
                    }
                    button {
                        class: "zoom-step-btn w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] rounded cursor-pointer flex items-center justify-center transition-all duration-150",
                        title: "Zoom In (Ctrl++)",
                        onclick: move |_| {
                            store.write().zoom_in();
                        },
                        svg {
                            view_box: "0 0 24 24",
                            width: "12",
                            height: "12",
                            path { d: "M12 5v14M5 12h14", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round" }
                        }
                    }
                }

                // Sticky Headings Toggle Button
                button {
                    class: if store_read.sticky_headers { "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md text-[var(--accent)] text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: if store_read.sticky_headers { "Sticky Markdown Headings: Enabled" } else { "Sticky Markdown Headings: Disabled" },
                    onclick: move |_| {
                        store.write().toggle_sticky_headers();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "13",
                        height: "13",
                        path { d: "M4 6h16M4 12h16M4 18h8", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", fill: "none" }
                        circle { cx: "18", cy: "18", r: "2.5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "Sticky" }
                    if store_read.sticky_headers {
                        span { class: "btn-indicator-dot w-1.5 h-1.5 rounded-full bg-[var(--accent)] ml-0.5" }
                    }
                }
            }

            // Right Group: Theme Switcher, Export & Zen Mode
            div {
                class: "toolbar-right-group flex items-center gap-2",

                // Theme & Primary Color Customizer Dropdown
                div {
                    class: "theme-dropdown-container relative",
                    button {
                        class: "toolbar-btn theme-selector-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                        title: "Customize Theme & Primary Accent Color",
                        onclick: move |_| {
                            let open = show_theme_menu();
                            show_theme_menu.set(!open);
                        },
                        span {
                            class: "theme-preview-swatch w-3.5 h-3.5 rounded-full border border-solid flex items-center justify-center shrink-0",
                            style: "background-color: {store_read.theme.default_bg()}; border-color: {active_accent};",
                            span {
                                class: "theme-preview-dot w-1.5 h-1.5 rounded-full",
                                style: "background-color: {active_accent};"
                            }
                        }
                        span { class: "btn-text", "{store_read.theme.label()}" }
                        svg {
                            class: "dropdown-arrow-icon opacity-60 ml-0.5",
                            view_box: "0 0 24 24",
                            width: "11",
                            height: "11",
                            path { d: "m6 9 6 6 6-6", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round" }
                        }
                    }

                    if show_theme_menu() {
                        div {
                            class: "theme-popover-backdrop",
                            onclick: move |_| {
                                show_theme_menu.set(false);
                            },
                        }
                        div {
                            class: "theme-popover-menu absolute top-9 right-0 bg-[var(--popover-bg)] border border-[var(--border-color)] rounded-xl shadow-2xl p-2.5 flex flex-col gap-2 z-[100]",
                            // Catppuccin Flavors Section
                            div {
                                class: "theme-menu-section",
                                div {
                                    class: "theme-menu-section-title inline-flex items-center gap-1.5",
                                    Icon { width: 12, height: 12, icon: LdSparkles, class: "text-[var(--accent)]" }
                                    span { "Catppuccin Flavors" }
                                }
                                div {
                                    class: "theme-grid",
                                    for (theme_item, label, accent_color, bg_color) in catppuccin_themes {
                                        button {
                                            class: if store_read.theme == theme_item { "theme-popover-item active-theme" } else { "theme-popover-item" },
                                            title: "Switch to {label}",
                                            onclick: move |_| {
                                                store.write().set_theme(theme_item);
                                            },
                                            span {
                                                class: "theme-preview-swatch",
                                                style: "background-color: {bg_color}; border-color: {accent_color};",
                                                span {
                                                    class: "theme-preview-dot",
                                                    style: "background-color: {accent_color};"
                                                }
                                            }
                                            span { class: "theme-popover-label", "{label}" }
                                            if store_read.theme == theme_item {
                                                svg {
                                                    class: "theme-check-icon",
                                                    view_box: "0 0 24 24",
                                                    width: "12",
                                                    height: "12",
                                                    path { d: "M20 6 9 17l-5-5", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Classic Themes Section
                            div {
                                class: "theme-menu-section",
                                div {
                                    class: "theme-menu-section-title",
                                    span { "Classic Themes" }
                                }
                                div {
                                    class: "theme-grid",
                                    for (theme_item, label, accent_color, bg_color) in classic_themes {
                                        button {
                                            class: if store_read.theme == theme_item { "theme-popover-item active-theme" } else { "theme-popover-item" },
                                            title: "Switch to {label}",
                                            onclick: move |_| {
                                                store.write().set_theme(theme_item);
                                            },
                                            span {
                                                class: "theme-preview-swatch",
                                                style: "background-color: {bg_color}; border-color: {accent_color};",
                                                span {
                                                    class: "theme-preview-dot",
                                                    style: "background-color: {accent_color};"
                                                }
                                            }
                                            span { class: "theme-popover-label", "{label}" }
                                            if store_read.theme == theme_item {
                                                svg {
                                                    class: "theme-check-icon",
                                                    view_box: "0 0 24 24",
                                                    width: "12",
                                                    height: "12",
                                                    path { d: "M20 6 9 17l-5-5", stroke: "currentColor", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "theme-menu-divider" }

                            // Primary Accent Color Picker Section
                            div {
                                class: "theme-menu-section",
                                div {
                                    class: "theme-menu-section-title flex items-center justify-between",
                                    div {
                                        class: "inline-flex items-center gap-1.5",
                                        Icon { width: 12, height: 12, icon: LdPalette, class: "text-[var(--accent)]" }
                                        span { "Primary Accent Color" }
                                    }
                                    if has_custom_accent {
                                        span { class: "text-[9.5px] font-mono text-[var(--accent)] font-semibold", "CUSTOM" }
                                    } else {
                                        span { class: "text-[9.5px] font-mono text-[var(--text-muted)]", "DEFAULT" }
                                    }
                                }

                                // Quick Accent Swatch Grid
                                div {
                                    class: "accent-palette-grid",
                                    for (hex, name) in accent_presets {
                                        div {
                                            class: if active_accent.eq_ignore_ascii_case(hex) { "accent-color-chip active-chip" } else { "accent-color-chip" },
                                            style: "background-color: {hex};",
                                            title: "{name} ({hex})",
                                            onclick: move |_| {
                                                store.write().set_primary_color(Some(hex.to_string()));
                                            },
                                            if active_accent.eq_ignore_ascii_case(hex) {
                                                span { class: "accent-chip-inner" }
                                            }
                                        }
                                    }
                                }

                                // Interactive Custom Color Picker & Hex Input
                                div {
                                    class: "custom-color-picker-row",
                                    div {
                                        class: "color-input-wrapper",
                                        style: "background-color: {active_accent};",
                                        title: "Pick custom primary color",
                                        input {
                                            class: "native-color-input",
                                            r#type: "color",
                                            value: "{active_accent}",
                                            oninput: move |evt| {
                                                let val = evt.value();
                                                store.write().set_primary_color(Some(val));
                                            },
                                        }
                                    }
                                    input {
                                        class: "color-hex-input",
                                        r#type: "text",
                                        placeholder: "#cba6f7",
                                        value: "{active_accent}",
                                        oninput: move |evt| {
                                            let val = evt.value().trim().to_string();
                                            if val.starts_with('#') && (val.len() == 7 || val.len() == 4) {
                                                store.write().set_primary_color(Some(val));
                                            }
                                        },
                                    }
                                    if has_custom_accent {
                                        button {
                                            class: "color-reset-btn",
                                            title: "Reset to theme default accent color",
                                            onclick: move |_| {
                                                store.write().set_primary_color(None);
                                            },
                                            svg {
                                                view_box: "0 0 24 24",
                                                width: "11",
                                                height: "11",
                                                path { d: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round" }
                                                path { d: "M3 3v5h5", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round" }
                                            }
                                            span { "Reset" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Export to HTML
                button {
                    class: "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150",
                    title: "Export Document as Standalone HTML",
                    onclick: move |_| {
                        spawn(async move {
                            let s = store();
                            if let Some(active_tab) = s.active_tab() {
                                let title = active_tab.title.replace(".md", "").replace(".mdx", "");
                                let html_content = active_tab.parsed.html_content.clone();
                                let theme_str = s.theme.as_str();
                                let custom_accent_style = s.primary_color.as_ref().map_or_else(String::new, |color| {
                                    format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40;")
                                });
                                if let Some(save_path) = pick_export_html_async(&format!("{title}.html")).await {
                                    let standalone_html = format!(
                                        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n<style>{APP_STYLES}</style>\n</head>\n<body class=\"{theme_str}\" style=\"{custom_accent_style}\">\n<div class=\"viewer-container reading-width\">\n<article class=\"markdown-body\">\n{html_content}\n</article>\n</div>\n</body>\n</html>"
                                    );
                                    let _ = save_document_file(&save_path, &standalone_html);
                                }
                            }
                        });
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "m7 10 5 5 5-5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "M12 15V3", fill: "none", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "Export" }
                }

                // Zen Mode
                button {
                    class: if store_read.is_zen { "toolbar-btn zen-btn active-zen inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--accent)] text-white border border-transparent rounded-md text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn zen-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "Focus Zen Mode (Ctrl+Shift+F or Esc)",
                    onclick: move |_| {
                        store.write().toggle_zen();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path { d: "M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3", stroke: "currentColor", stroke_width: "2", fill: "none", stroke_linecap: "round" }
                    }
                    span { class: "btn-text", "Zen" }
                }

                // Settings Modal Button
                button {
                    class: if store_read.show_settings_modal { "toolbar-btn active-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-md text-[var(--accent)] text-xs font-medium cursor-pointer transition-all duration-150" } else { "toolbar-btn inline-flex items-center gap-1.5 h-7 px-2.5 bg-transparent border border-transparent rounded-md text-[var(--text-main)] text-xs font-medium cursor-pointer hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] transition-all duration-150" },
                    title: "Preferences & Settings (Ctrl+,)",
                    onclick: move |_| {
                        store.write().toggle_settings_modal();
                    },
                    svg {
                        class: "toolbar-btn-icon shrink-0",
                        view_box: "0 0 24 24",
                        width: "14",
                        height: "14",
                        path {
                            d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                        circle { cx: "12", cy: "12", r: "3", fill: "none", stroke: "currentColor", stroke_width: "2" }
                    }
                    span { class: "btn-text", "Settings" }
                }
            }
        }
    }
}
