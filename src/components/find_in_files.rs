use crate::services::fts::{self, SearchHit};
use crate::state::{kick_pending_document_loads, AppStore, OpenKind};
use crate::types::{FileTreeEntry, TabItem};
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

fn collect_name_hits(entries: &[FileTreeEntry], query: &str, limit: usize) -> Vec<SearchHit> {
    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    walk_name_hits(entries, &query_lower, limit, &mut hits);
    hits
}

fn walk_name_hits(
    entries: &[FileTreeEntry],
    query_lower: &str,
    limit: usize,
    hits: &mut Vec<SearchHit>,
) {
    for entry in entries {
        if hits.len() >= limit {
            return;
        }
        if entry.is_dir {
            walk_name_hits(&entry.children, query_lower, limit, hits);
            continue;
        }
        let name_hit = entry.name.to_lowercase().contains(query_lower);
        let path_hit = entry
            .path
            .to_string_lossy()
            .to_lowercase()
            .contains(query_lower);
        if name_hit || path_hit {
            hits.push(SearchHit {
                path: entry.path.clone(),
                snippet: String::new(),
                match_start: 0,
                match_len: 0,
                name_match: true,
            });
        }
    }
}

fn merge_hits(mut name_hits: Vec<SearchHit>, fts_hits: Vec<SearchHit>, limit: usize) -> Vec<SearchHit> {
    for fts_hit in fts_hits {
        if let Some(existing) = name_hits.iter_mut().find(|hit| hit.path == fts_hit.path) {
            if existing.snippet.is_empty() && !fts_hit.snippet.is_empty() {
                existing.snippet = fts_hit.snippet;
                existing.match_start = fts_hit.match_start;
                existing.match_len = fts_hit.match_len;
            }
            existing.name_match = true;
        } else {
            name_hits.push(fts_hit);
        }
    }
    name_hits.truncate(limit);
    name_hits
}

fn spawn_find_search(
    query: String,
    tabs: Vec<TabItem>,
    tree: Vec<FileTreeEntry>,
    root: Option<PathBuf>,
    has_index: bool,
    mut results: Signal<Vec<SearchHit>>,
    mut selected: Signal<usize>,
    mut is_searching: Signal<bool>,
    gen: u32,
    search_gen: Signal<u32>,
) {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        results.set(Vec::new());
        selected.set(0);
        return;
    }

    is_searching.set(true);
    spawn(async move {
        let search_res = tokio::task::spawn_blocking(move || {
            let name_hits = collect_name_hits(&tree, &trimmed, 50);
            match fts::search_all(&trimmed, 50, &tabs, has_index, root.as_deref()) {
                Ok(fts_hits) => merge_hits(name_hits, fts_hits, 50),
                Err(_) => name_hits,
            }
        })
        .await;
        if search_gen() != gen {
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
    let is_searching = use_signal(|| false);
    let mut search_gen = use_signal(|| 0u32);

    let t = store().language.strings().find_in_files;
    let open = store().show_find_in_files;

    use_effect(move || {
        if !open {
            return;
        }
        query.set(String::new());
        results.set(Vec::new());
        selected.set(0);

        let root = store().opened_folder.clone();
        let filter = store().file_filter_mode;
        let needs_rebuild = root.as_ref().is_some_and(|dir| !fts::is_index_for(dir));

        if let Some(dir) = root {
            if needs_rebuild {
                is_indexing.set(true);
                spawn(async move {
                    let dir_clone = dir.clone();
                    let res =
                        tokio::task::spawn_blocking(move || fts::rebuild_root(dir_clone, filter))
                            .await;
                    is_indexing.set(false);
                    if res.ok().is_some_and(|r| r.is_ok()) {
                        let q = query();
                        if !q.trim().is_empty() {
                            let s = store();
                            spawn_find_search(
                                q,
                                s.tabs.clone(),
                                s.file_tree.clone(),
                                s.opened_folder.clone(),
                                s.opened_folder
                                    .as_ref()
                                    .is_some_and(|d| fts::is_index_for(d)),
                                results,
                                selected,
                                is_searching,
                                search_gen(),
                                search_gen,
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
        store.write().set_find_in_files(false);
    };

    let mut open_hit = {
        move |hit: SearchHit, search_text: String| {
            let path = hit.path;
            store.write().open_file_from_path(path, OpenKind::Preview);
            kick_pending_document_loads(store);
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
                                search_gen += 1;
                                let my_gen = search_gen();
                                spawn(async move {
                                    tokio::time::sleep(SEARCH_DEBOUNCE).await;
                                    if search_gen() != my_gen {
                                        return;
                                    }
                                    let s = store();
                                    spawn_find_search(
                                        val,
                                        s.tabs.clone(),
                                        s.file_tree.clone(),
                                        s.opened_folder.clone(),
                                        s.opened_folder.as_ref().is_some_and(|d| fts::is_index_for(d)),
                                        results,
                                        selected,
                                        is_searching,
                                        my_gen,
                                        search_gen,
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
                                    search_gen += 1;
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
    let match_start = props.hit.match_start;
    let match_len = props.hit.match_len;
    let query = props.query.clone();

    rsx! {
        button {
            class: "{row_class}",
            r#type: "button",
            onclick: move |_| props.onclick.call(()),
            Icon { width: 14, height: 14, icon: LdFileText, class: "shrink-0 text-[var(--text-muted)]" }
            div { class: "find-in-files-result-body",
                HighlightedText {
                    text: name,
                    query: query.clone(),
                    text_class: "find-in-files-result-name",
                }
                if !dir.is_empty() {
                    span { class: "find-in-files-result-dir", "{dir}" }
                }
                if !snippet.is_empty() {
                    SnippetText {
                        snippet,
                        match_start,
                        match_len,
                    }
                }
            }
        }
    }
}

#[component]
fn HighlightedText(text: String, query: String, text_class: &'static str) -> Element {
    let query_trim = query.trim();
    if query_trim.is_empty() {
        return rsx! { span { class: "{text_class}", "{text}" } };
    }
    let lower = text.to_lowercase();
    let needle = query_trim.to_lowercase();
    let Some(pos) = lower.find(&needle) else {
        return rsx! { span { class: "{text_class}", "{text}" } };
    };
    let end = pos + needle.len();
    if !text.is_char_boundary(pos) || !text.is_char_boundary(end) {
        return rsx! { span { class: "{text_class}", "{text}" } };
    }
    let (pre, rest) = text.split_at(pos);
    let (mid, post) = rest.split_at(needle.len());
    rsx! {
        span { class: "{text_class}",
            "{pre}"
            span { class: "find-in-files-result-mark", "{mid}" }
            "{post}"
        }
    }
}

#[component]
fn SnippetText(snippet: String, match_start: usize, match_len: usize) -> Element {
    let end = match_start.saturating_add(match_len);
    if match_len == 0 || match_start >= snippet.len() || end > snippet.len()
        || !snippet.is_char_boundary(match_start)
        || !snippet.is_char_boundary(end)
    {
        return rsx! { span { class: "find-in-files-result-snippet", "{snippet}" } };
    }
    let (pre, rest) = snippet.split_at(match_start);
    let (mid, post) = rest.split_at(match_len);
    rsx! {
        span { class: "find-in-files-result-snippet",
            "{pre}"
            span { class: "find-in-files-result-mark", "{mid}" }
            "{post}"
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
