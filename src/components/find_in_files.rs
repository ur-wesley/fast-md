use crate::services::fts::{self, token_spans, SearchHit};
use crate::state::{kick_pending_document_loads, AppStore, OpenKind};
use crate::types::{FileFilterMode, TabItem};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileText, LdSearch, LdX};
use std::path::{Path, PathBuf};
use std::time::Duration;

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(80);

#[derive(Props, Clone, PartialEq)]
pub struct FindInFilesProps {
    pub store: Signal<AppStore>,
}

fn spawn_find_search(
    query: String,
    tabs: Vec<TabItem>,
    root: Option<PathBuf>,
    has_index: bool,
    mut results: Signal<Vec<SearchHit>>,
    mut selected: Signal<usize>,
    mut is_searching: Signal<bool>,
    epoch: u64,
) {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        results.set(Vec::new());
        selected.set(0);
        is_searching.set(false);
        return;
    }

    is_searching.set(true);
    spawn(async move {
        let search_res = tokio::task::spawn_blocking(move || {
            fts::search_all(&trimmed, 50, &tabs, has_index, root.as_deref(), epoch)
                .unwrap_or_else(|_| Vec::new())
        })
        .await;
        if fts::current_epoch() != epoch {
            return;
        }
        is_searching.set(false);
        match search_res {
            Ok(hits) => {
                results.set(hits);
                selected.set(0);
            }
            Err(_) => results.set(Vec::new()),
        }
    });
}

#[component]
pub fn FindInFiles(props: FindInFilesProps) -> Element {
    let mut store = props.store;
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchHit>::new);
    let mut selected = use_signal(|| 0usize);
    let mut is_indexing = use_signal(|| false);
    let mut is_searching = use_signal(|| false);

    let t = store().language.strings().find_in_files;
    let open = store().show_find_in_files;
    let overlay_open = use_memo(move || store().show_find_in_files);

    use_effect(move || {
        if !overlay_open() {
            is_searching.set(false);
            return;
        }
        query.set(String::new());
        results.set(Vec::new());
        selected.set(0);
        is_searching.set(false);
        is_indexing.set(false);

        let root = store.peek().opened_folder.clone();
        let needs_rebuild = root.as_ref().is_some_and(|dir| !fts::is_full_index_for(dir));

        if let Some(dir) = root {
            if needs_rebuild {
                is_indexing.set(true);
                spawn(async move {
                    let dir_clone = dir.clone();
                    let res =
                        tokio::task::spawn_blocking(move || {
                            fts::rebuild_root(dir_clone, FileFilterMode::AllFiles)
                        })
                            .await;
                    is_indexing.set(false);
                    if res.ok().is_some_and(|r| r.is_ok()) {
                        let q = query();
                        if !q.trim().is_empty() {
                            let s = store.peek();
                            spawn_find_search(
                                q,
                                s.tabs.clone(),
                                s.opened_folder.clone(),
                                s.opened_folder
                                    .as_ref()
                                    .is_some_and(|d| fts::is_full_index_for(d)),
                                results,
                                selected,
                                is_searching,
                                fts::current_epoch(),
                            );
                        }
                    }
                });
            } else {
                is_indexing.set(false);
            }
        } else {
            is_indexing.set(false);
        }

        dioxus::prelude::document::eval(
            r"
            setTimeout(() => {
                const input = document.getElementById('find-in-files-input');
                if (input) { input.focus(); input.select(); }
            }, 30);
            ",
        );
    });

    use_effect(move || {
        let _ = selected();
        let _ = dioxus::prelude::document::eval(
            r"document.querySelector('.find-in-files-results .is-selected')?.scrollIntoView({block:'nearest'});",
        );
    });

    let mut close_overlay = move |()| {
        let _ = fts::bump_epoch();
        is_searching.set(false);
        store.write().set_find_in_files(false);
    };

    let mut open_hit = {
        move |hit: SearchHit, search_text: String| {
            let path = hit.path;
            store.write().open_file_from_path(path, OpenKind::Preview);
            kick_pending_document_loads(store);
            let _ = fts::bump_epoch();
            store.write().set_find_in_files(false);
            let q = search_text.trim().to_string();
            if !q.is_empty() {
                let _ = dioxus::prelude::document::eval(&format!(
                    r#"
                    setTimeout(() => {{
                        const input = document.getElementById('titlebar-search-input');
                        if (input) {{
                            input.value = {q:?};
                            input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            input.focus();
                            input.select();
                        }}
                        window.highlightSearchMatches && window.highlightSearchMatches({q:?});
                    }}, 50);
                    "#
                ));
            }
        }
    };

    let root = store().opened_folder.clone();
    let result_items = results();
    let selected_idx = selected();
    let query_val = query();

    if !open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "find-in-files-backdrop",
            onclick: move |_| close_overlay(()),
            div {
                class: "find-in-files-dialog",
                onmousedown: move |evt| evt.stop_propagation(),

                div { class: "find-in-files-header",
                    Icon { width: 16, height: 16, icon: LdSearch, class: "text-[var(--accent)] shrink-0" }
                    span { class: "find-in-files-title", "{t.title}" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconXs,
                        title: "{t.close}",
                        onclick: move |_| close_overlay(()),
                        Icon { width: 14, height: 14, icon: LdX }
                    }
                }

                div { class: "find-in-files-input-row",
                    div { class: "find-in-files-input-wrap",
                        Icon { width: 14, height: 14, icon: LdSearch, class: "text-[var(--text-muted)] shrink-0" }
                        Input {
                            id: "find-in-files-input",
                            class: "find-in-files-input flex-1 min-w-0 w-full",
                            r#type: "text",
                            autocomplete: "off",
                            placeholder: "{t.placeholder}",
                            value: "{query_val}",
                            oninput: move |evt: FormEvent| {
                                let val = evt.value();
                                query.set(val.clone());
                                let epoch = fts::bump_epoch();
                                spawn(async move {
                                    tokio::time::sleep(SEARCH_DEBOUNCE).await;
                                    if fts::current_epoch() != epoch {
                                        return;
                                    }
                                    let s = store.peek();
                                    spawn_find_search(
                                        val,
                                        s.tabs.clone(),
                                        s.opened_folder.clone(),
                                        s.opened_folder.as_ref().is_some_and(|d| fts::is_full_index_for(d)),
                                        results,
                                        selected,
                                        is_searching,
                                        epoch,
                                    );
                                });
                            },
                            onkeydown: move |evt: KeyboardEvent| {
                                let len = results().len();
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
                                        if let Some(hit) = results().get(idx).cloned() {
                                            open_hit(hit, query());
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
                                    results.set(Vec::new());
                                    selected.set(0);
                                    is_searching.set(false);
                                    let _ = fts::bump_epoch();
                                },
                                Icon { width: 12, height: 12, icon: LdX }
                            }
                        }
                    }
                }

                div { class: "find-in-files-results",
                    if query_val.trim().is_empty() {
                        div { class: "find-in-files-status", "{t.type_to_search}" }
                    } else if !result_items.is_empty() {
                        for (idx , hit) in result_items.iter().enumerate() {
                            FindInFilesResultRow {
                                key: "{hit.path.display()}",
                                hit: hit.clone(),
                                root: root.clone(),
                                query: query_val.clone(),
                                selected: idx == selected_idx,
                                onclick: {
                                    let hit = hit.clone();
                                    let q = query_val.clone();
                                    move |_| open_hit(hit.clone(), q.clone())
                                },
                            }
                        }
                    } else if is_indexing() {
                        div { class: "find-in-files-status", "{t.indexing}" }
                    } else if is_searching() {
                        div { class: "find-in-files-status", "{t.searching}" }
                    } else {
                        div { class: "find-in-files-empty",
                            span { class: "find-in-files-empty-title", "{t.no_results}" }
                            span { class: "find-in-files-empty-desc", "{t.no_results_desc}" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FindInFilesResultRowProps {
    hit: SearchHit,
    root: Option<PathBuf>,
    query: String,
    selected: bool,
    onclick: EventHandler<()>,
}

#[component]
fn FindInFilesResultRow(props: FindInFilesResultRowProps) -> Element {
    let (name, dir) = split_name_dir(&props.hit.path, props.root.as_deref());
    let row_class = if props.selected {
        "find-in-files-result is-selected"
    } else {
        "find-in-files-result"
    };
    let snippet = props.hit.snippet.clone();
    let query = props.query.clone();

    rsx! {
        button {
            class: "{row_class}",
            r#type: "button",
            onclick: move |_| props.onclick.call(()),
            Icon { width: 14, height: 14, icon: LdFileText, class: "shrink-0 text-[var(--text-muted)]" }
            div { class: "find-in-files-result-body",
                TokenText {
                    text: name,
                    query: query.clone(),
                    class: "find-in-files-result-name",
                    mark_class: "find-in-files-result-mark",
                }
                if !dir.is_empty() {
                    TokenText {
                        text: dir,
                        query: query.clone(),
                        class: "find-in-files-result-dir",
                        mark_class: "find-in-files-result-mark",
                    }
                }
                if !snippet.is_empty() {
                    TokenText {
                        text: snippet,
                        query: query.clone(),
                        class: "find-in-files-result-snippet",
                        mark_class: "find-in-files-result-mark",
                    }
                }
            }
        }
    }
}

#[component]
fn TokenText(text: String, query: String, class: String, mark_class: String) -> Element {
    let spans = token_spans(&text, &query);
    rsx! {
        span {
            class: "{class}",
            for (i, (marked, chunk)) in spans.into_iter().enumerate() {
                if marked {
                    mark { key: "{i}", class: "{mark_class}", "{chunk}" }
                } else {
                    span { key: "{i}", "{chunk}" }
                }
            }
        }
    }
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
