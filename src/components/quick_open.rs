use crate::components::sidebar::file_tree::{
    flatten_files, fuzzy_match, rank_quick_open, QuickOpenItem, QUICK_OPEN_LIMIT,
};
use crate::state::{kick_pending_document_loads, AppStore, OpenKind};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileText, LdSearch, LdX};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Props, Clone, PartialEq)]
pub struct QuickOpenProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn QuickOpen(props: QuickOpenProps) -> Element {
    let mut store = props.store;
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);

    let t = store().language.strings().quick_open;
    let open = store().show_quick_open;

    use_effect(move || {
        if !open {
            return;
        }
        query.set(String::new());
        selected.set(0);

        dioxus::prelude::document::eval(
            r"
            setTimeout(() => {
                const input = document.getElementById('quick-open-input');
                if (input) { input.focus(); input.select(); }
            }, 30);
            ",
        );
    });

    use_effect(move || {
        let _ = selected();
        let _ = dioxus::prelude::document::eval(
            r"document.querySelector('.quick-open-results .is-selected')?.scrollIntoView({block:'nearest'});",
        );
    });

    let mut close_overlay = move |()| {
        store.write().set_quick_open(false);
    };

    let mut open_item = move |item: QuickOpenItem| {
        let path = item.path;
        store.write().open_file_from_path(path, OpenKind::Preview);
        kick_pending_document_loads(store);
        store.write().set_quick_open(false);
    };

    let s = store();
    let result_items = rank_quick_open(
        &flatten_files(&s.file_tree),
        &s.settings.recent_files,
        query().trim(),
        s.opened_folder.as_deref(),
        QUICK_OPEN_LIMIT,
    );
    let selected_idx = selected();
    let query_val = query();
    let root = s.opened_folder.clone();
    let no_folder = root.is_none() && result_items.is_empty();
    let no_results = !query_val.trim().is_empty() && result_items.is_empty();
    let key_items = result_items.clone();

    if !open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "quick-open-backdrop",
            onclick: move |_| close_overlay(()),
            div {
                class: "quick-open-dialog",
                onmousedown: move |evt| evt.stop_propagation(),
                onclick: move |evt| evt.stop_propagation(),

                div { class: "quick-open-header",
                    Icon { width: 16, height: 16, icon: LdSearch, class: "text-[var(--accent)] shrink-0" }
                    span { class: "quick-open-title", "{t.title}" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconXs,
                        title: "{t.close}",
                        onclick: move |_| close_overlay(()),
                        Icon { width: 14, height: 14, icon: LdX }
                    }
                }

                div { class: "quick-open-input-row",
                    div { class: "quick-open-input-wrap",
                        Icon { width: 14, height: 14, icon: LdSearch, class: "text-[var(--text-muted)] shrink-0" }
                        Input {
                            id: "quick-open-input",
                            class: "quick-open-input flex-1 min-w-0 w-full",
                            r#type: "text",
                            autocomplete: "off",
                            placeholder: "{t.placeholder}",
                            value: "{query_val}",
                            oninput: move |evt: FormEvent| {
                                query.set(evt.value());
                                selected.set(0);
                            },
                            onkeydown: move |evt: KeyboardEvent| {
                                let len = key_items.len();
                                match evt.key() {
                                    Key::Escape => close_overlay(()),
                                    Key::ArrowDown if len > 0 => {
                                        evt.prevent_default();
                                        let next = (selected() + 1).min(len.saturating_sub(1));
                                        selected.set(next);
                                    }
                                    Key::ArrowUp if len > 0 => {
                                        evt.prevent_default();
                                        let prev = selected().saturating_sub(1);
                                        selected.set(prev);
                                    }
                                    Key::Enter if len > 0 => {
                                        let idx = selected().min(len.saturating_sub(1));
                                        if let Some(item) = key_items.get(idx).cloned() {
                                            open_item(item);
                                        }
                                    }
                                    _ => {}
                                }
                            },
                        }
                        if !query_val.is_empty() {
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconXs,
                                class: "text-[var(--text-muted)] hover:text-[var(--text-heading)] p-0.5",
                                title: "{t.close}",
                                onclick: move |_| {
                                    query.set(String::new());
                                    selected.set(0);
                                },
                                Icon { width: 12, height: 12, icon: LdX }
                            }
                        }
                    }
                }

                div { class: "quick-open-results",
                    if no_folder {
                        div { class: "quick-open-empty",
                            span { class: "quick-open-empty-title", "{t.no_folder}" }
                        }
                    } else if no_results {
                        div { class: "quick-open-empty",
                            span { class: "quick-open-empty-title", "{t.no_results}" }
                            span { class: "quick-open-empty-desc", "{t.no_results_desc}" }
                        }
                    } else if !result_items.is_empty() {
                        for (idx , item) in result_items.iter().enumerate() {
                            QuickOpenResultRow {
                                key: "{item.path.display()}",
                                item: item.clone(),
                                root: root.clone(),
                                query: query_val.clone(),
                                selected: idx == selected_idx,
                                onclick: {
                                    let item = item.clone();
                                    move |_| open_item(item.clone())
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct QuickOpenResultRowProps {
    item: QuickOpenItem,
    root: Option<PathBuf>,
    query: String,
    selected: bool,
    onclick: EventHandler<()>,
}

#[component]
fn QuickOpenResultRow(props: QuickOpenResultRowProps) -> Element {
    let (name, dir) = split_name_dir(&props.item.path, props.root.as_deref());
    let row_class = if props.selected {
        "quick-open-result is-selected"
    } else {
        "quick-open-result"
    };

    rsx! {
        button {
            class: "{row_class}",
            r#type: "button",
            onclick: move |_| props.onclick.call(()),
            Icon { width: 14, height: 14, icon: LdFileText, class: "shrink-0 text-[var(--text-muted)]" }
            div { class: "quick-open-result-body",
                FuzzyText { text: name, query: props.query.clone(), class: "quick-open-result-name" }
                if !dir.is_empty() {
                    FuzzyText { text: dir, query: props.query.clone(), class: "quick-open-result-dir" }
                }
            }
        }
    }
}

#[component]
fn FuzzyText(text: String, query: String, class: String) -> Element {
    let spans = marked_spans(&text, &query);
    rsx! {
        span {
            class: "{class}",
            for (i, (marked, chunk)) in spans.into_iter().enumerate() {
                if marked {
                    mark { key: "{i}", class: "quick-open-result-mark", "{chunk}" }
                } else {
                    span { key: "{i}", "{chunk}" }
                }
            }
        }
    }
}

fn marked_spans(text: &str, query: &str) -> Vec<(bool, String)> {
    let q = query.trim();
    if q.is_empty() {
        return vec![(false, text.to_string())];
    }
    let marks: HashSet<usize> = fuzzy_match(text, q)
        .map(|(_, idx)| idx.into_iter().collect())
        .unwrap_or_default();
    if marks.is_empty() {
        return vec![(false, text.to_string())];
    }
    let mut spans = Vec::new();
    let mut marked = false;
    let mut buf = String::new();
    for (i, ch) in text.chars().enumerate() {
        let hit = marks.contains(&i);
        if !buf.is_empty() && hit != marked {
            spans.push((marked, std::mem::take(&mut buf)));
        }
        marked = hit;
        buf.push(ch);
    }
    if !buf.is_empty() {
        spans.push((marked, buf));
    }
    spans
}

fn split_name_dir(path: &Path, root: Option<&Path>) -> (String, String) {
    let name = path
        .file_name()
        .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
    let dir = match (path.parent(), root) {
        (Some(parent), Some(root)) => parent
            .strip_prefix(root)
            .map_or_else(|_| parent.display().to_string(), |rel| rel.display().to_string()),
        (Some(parent), None) => parent.display().to_string(),
        _ => String::new(),
    };
    (name, dir)
}
