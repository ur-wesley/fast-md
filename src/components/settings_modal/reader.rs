use crate::components::Hint;
use crate::state::AppStore;
use crate::types::DocumentMode;
use crate::ui::switch::Switch;
use crate::ui::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Props, Clone, PartialEq)]
pub struct ReaderPaneProps {
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
    matches_default_mode(&q, t)
        || matches_format_on_save(&q, t)
        || matches_reading_layout(&q, t)
        || matches_zoom(&q, t)
        || matches_font_size(&q, t)
        || matches_auto_reload(&q, t)
        || matches_sticky_headers(&q, t)
}

fn matches_default_mode(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "mode", "default", "standard", "modus", "view", "split", "wysiwyg", "source", "quelltext", "ansicht", "geteilt",
        t.settings.default_mode_title,
        t.settings.default_mode_desc,
        t.toolbar.mode_view,
        t.toolbar.mode_split,
        t.toolbar.mode_wysiwyg,
        t.toolbar.mode_source,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_format_on_save(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "format", "save", "speichern", "formatieren", "table", "tabellen", "align",
        t.settings.format_on_save_title,
        t.settings.format_on_save_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_reading_layout(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "reading", "layout", "column", "spalte", "width", "full width", "breite", "volle breite",
        t.settings.reading_layout_title,
        t.settings.reading_layout_desc,
        t.settings.reading_column,
        t.settings.full_width,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_zoom(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "zoom", "scale", "skalieren", "vergrößern", "verkleinern", "reset zoom",
        t.settings.zoom_title,
        t.settings.zoom_desc,
        t.toolbar.zoom_in,
        t.toolbar.zoom_out,
        t.toolbar.reset_zoom,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_font_size(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "font", "size", "schrift", "schriftgröße", "typography", "14px", "16px", "18px", "20px",
        t.settings.font_size_title,
        t.settings.font_size_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_auto_reload(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "auto reload", "reload", "live", "disk", "external", "neu laden", "automatisch",
        t.settings.auto_reload_title,
        t.settings.auto_reload_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn matches_sticky_headers(q: &str, t: &'static crate::i18n::Translations) -> bool {
    let haystacks = [
        "sticky", "header", "heading", "überschrift", "fixiert", "anheften", "scroll",
        t.settings.sticky_headers_title,
        t.settings.sticky_headers_desc,
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(q))
}

fn default_mode_index(mode: DocumentMode) -> usize {
    match mode {
        DocumentMode::View => 0,
        DocumentMode::Split => 1,
        DocumentMode::Wysiwyg => 2,
        DocumentMode::Source => 3,
    }
}

fn default_mode_from_index(idx: usize) -> DocumentMode {
    match idx {
        0 => DocumentMode::View,
        1 => DocumentMode::Split,
        2 => DocumentMode::Wysiwyg,
        _ => DocumentMode::Source,
    }
}

fn reading_layout_index(is_full_width: bool) -> usize {
    usize::from(is_full_width)
}

fn font_size_index(size: u32) -> usize {
    match size {
        14 => 0,
        16 => 1,
        18 => 2,
        _ => 3,
    }
}

fn font_size_from_index(idx: usize) -> u32 {
    match idx {
        0 => 14,
        1 => 16,
        2 => 18,
        _ => 20,
    }
}

#[component]
pub fn ReaderPane(props: ReaderPaneProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = props.t;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim().to_lowercase();

    let show_default_mode = filter.is_empty() || matches_default_mode(&filter, t);
    let show_format_on_save = filter.is_empty() || matches_format_on_save(&filter, t);
    let show_reading_layout = filter.is_empty() || matches_reading_layout(&filter, t);
    let show_zoom = filter.is_empty() || matches_zoom(&filter, t);
    let show_font_size = filter.is_empty() || matches_font_size(&filter, t);
    let show_auto_reload = filter.is_empty() || matches_auto_reload(&filter, t);
    let show_sticky_headers = filter.is_empty() || matches_sticky_headers(&filter, t);

    let default_mode_pressed =
        use_memo(move || Some(HashSet::from([default_mode_index(store().settings.default_mode)])));
    let format_on_save_checked = use_memo(move || Some(store().settings.format_on_save));
    let reading_layout_pressed =
        use_memo(move || Some(HashSet::from([reading_layout_index(store().is_full_width)])));
    let font_size_pressed =
        use_memo(move || Some(HashSet::from([font_size_index(store().settings.font_size)])));
    let auto_reload_checked = use_memo(move || Some(store().settings.auto_reload));
    let sticky_headers_checked = use_memo(move || Some(store().settings.sticky_headers));

    rsx! {
        div {
            class: "settings-section flex flex-col gap-5",

            if show_default_mode {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.default_mode_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.default_mode_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: default_mode_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_default_mode(default_mode_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "{t.toolbar.mode_view}" }
                        ToggleItem { index: 1usize, "{t.toolbar.mode_split}" }
                        ToggleItem { index: 2usize, "{t.toolbar.mode_wysiwyg}" }
                        ToggleItem { index: 3usize, "{t.toolbar.mode_source}" }
                    }
                }
            }

            if show_format_on_save {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.format_on_save_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.format_on_save_desc}" }
                    }
                    Switch {
                        checked: format_on_save_checked,
                        on_checked_change: move |checked: bool| {
                            if store().settings.format_on_save != checked {
                                store.write().toggle_format_on_save();
                            }
                        },
                    }
                }
            }

            if show_reading_layout {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.reading_layout_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.reading_layout_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: reading_layout_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                let want_full = idx == 1;
                                if store().is_full_width != want_full {
                                    store.write().toggle_full_width();
                                }
                            }
                        },
                        ToggleItem { index: 0usize, "{t.settings.reading_column}" }
                        ToggleItem { index: 1usize, "{t.settings.full_width}" }
                    }
                }
            }

            if show_zoom {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.zoom_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.zoom_desc}" }
                    }
                    div {
                        class: "inline-flex items-center bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg h-7 px-1",
                        Hint {
                            text: t.toolbar.zoom_out,
                            button {
                                class: "w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] cursor-pointer text-sm font-bold",
                                onclick: move |_| store.write().zoom_out(),
                                "-"
                            }
                        }
                        Hint {
                            text: t.toolbar.reset_zoom,
                            span {
                                class: "px-2.5 text-xs font-mono font-semibold text-[var(--text-heading)] cursor-pointer hover:text-[var(--accent)]",
                                onclick: move |_| store.write().reset_zoom(),
                                "{store_read.zoom_level}%"
                            }
                        }
                        Hint {
                            text: t.toolbar.zoom_in,
                            button {
                                class: "w-6 h-full bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-heading)] cursor-pointer text-sm font-bold",
                                onclick: move |_| store.write().zoom_in(),
                                "+"
                            }
                        }
                    }
                }
            }

            if show_font_size {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.font_size_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.font_size_desc}" }
                    }
                    ToggleGroup {
                        class: "inline-flex bg-[var(--bg-subtle)] border border-[var(--border-color)] rounded-lg p-0.5 gap-1",
                        horizontal: true,
                        allow_multiple_pressed: false,
                        pressed: font_size_pressed,
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            if let Some(&idx) = pressed.iter().next() {
                                store.write().set_font_size(font_size_from_index(idx));
                            }
                        },
                        ToggleItem { index: 0usize, "14px" }
                        ToggleItem { index: 1usize, "16px" }
                        ToggleItem { index: 2usize, "18px" }
                        ToggleItem { index: 3usize, "20px" }
                    }
                }
            }

            if show_auto_reload {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.auto_reload_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.auto_reload_desc}" }
                    }
                    Switch {
                        checked: auto_reload_checked,
                        on_checked_change: move |checked: bool| {
                            store.write().set_auto_reload(checked);
                        },
                    }
                }
            }

            if show_sticky_headers {
                div {
                    class: "flex items-center justify-between p-3.5 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-xl",
                    div {
                        h4 { class: "text-xs font-semibold text-[var(--text-heading)] m-0", "{t.settings.sticky_headers_title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.settings.sticky_headers_desc}" }
                    }
                    Switch {
                        checked: sticky_headers_checked,
                        on_checked_change: move |checked: bool| {
                            store.write().set_sticky_headers(checked);
                        },
                    }
                }
            }
        }
    }
}

