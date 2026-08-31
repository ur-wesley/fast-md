use crate::services::fts::{self, SearchHit};
use crate::state::AppStore;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileText, LdSearch, LdX};
use std::path::{Path, PathBuf};

#[derive(Props, Clone, PartialEq)]
pub struct FindInFilesProps {
    pub store: Signal<AppStore>,
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

    use_effect(move || {
        if !open {
            return;
        }
        query.set(String::new());
        results.set(Vec::new());
        selected.set(0);

        let root = store().opened_folder.clone();
        let filter = store().file_filter_mode;
        let needs_rebuild = root
            .as_ref()
            .is_some_and(|dir| !fts::is_index_for(dir));

        if let Some(dir) = root {
            if needs_rebuild {
                is_indexing.set(true);
                spawn(async move {
                    let dir_clone = dir.clone();
                    let res = tokio::task::spawn_blocking(move || fts::rebuild_root(dir_clone, filter))
                        .await;
                    is_indexing.set(false);
                    if res.is_err() {
                        results.set(Vec::new());
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

    let mut run_search = move |search_query: String| {
        let trimmed = search_query.trim().to_string();
        if trimmed.is_empty() {
            results.set(Vec::new());
            selected.set(0);
            return;
        }

        let tabs = store().tabs.clone();
        let has_index = store()
            .opened_folder
            .as_ref()
            .is_some_and(|dir| fts::is_index_for(dir));
        let q = trimmed.clone();
        is_searching.set(true);

        spawn(async move {
            let search_res =
                tokio::task::spawn_blocking(move || fts::search_all(&q, 50, &tabs, has_index)).await;
            is_searching.set(false);
            match search_res {
                Ok(Ok(hits)) => {
                    results.set(hits);
                    selected.set(0);
                }
                _ => results.set(Vec::new()),
            }
        });
    };

    let mut close_overlay = move |()| {
        store.write().set_find_in_files(false);
    };

    let mut open_hit = {
        move |hit: SearchHit, search_text: String| {
            let path = hit.path;
            store.write().open_file_from_path(path);
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
                    Input {
                        id: "find-in-files-input",
                        class: "find-in-files-input",
                        r#type: "text",
                        placeholder: "{t.placeholder}",
                        value: "{query_val}",
                        oninput: move |evt: FormEvent| {
                            let val = evt.value();
                            query.set(val.clone());
                            run_search(val);
                        },
                        onkeydown: move |evt: KeyboardEvent| {
                            let len = results().len();
                            match evt.key() {
                                Key::Escape => close_overlay(()),
                                Key::ArrowDown if len > 0 => {
                                    let next = (selected() + 1).min(len.saturating_sub(1));
                                    selected.set(next);
                                }
                                Key::ArrowUp if len > 0 => {
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
                }

                div { class: "find-in-files-results",
                    if is_indexing() {
                        div { class: "find-in-files-status", "{t.indexing}" }
                    } else if is_searching() {
                        div { class: "find-in-files-status", "{t.searching}" }
                    } else if query_val.trim().is_empty() {
                        div { class: "find-in-files-status", "{t.type_to_search}" }
                    } else if result_items.is_empty() {
                        div { class: "find-in-files-empty",
                            span { class: "find-in-files-empty-title", "{t.no_results}" }
                            span { class: "find-in-files-empty-desc", "{t.no_results_desc}" }
                        }
                    } else {
                        for (idx , hit) in result_items.iter().enumerate() {
                            FindInFilesResultRow {
                                key: "{hit.path.display()}",
                                hit: hit.clone(),
                                root: root.clone(),
                                selected: idx == selected_idx,
                                onclick: {
                                    let hit = hit.clone();
                                    let q = query_val.clone();
                                    move |_| open_hit(hit.clone(), q.clone())
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
struct FindInFilesResultRowProps {
    hit: SearchHit,
    root: Option<PathBuf>,
    selected: bool,
    onclick: EventHandler<()>,
}

#[component]
fn FindInFilesResultRow(props: FindInFilesResultRowProps) -> Element {
    let display_path = format_display_path(&props.hit.path, props.root.as_deref());
    let row_class = if props.selected {
        "find-in-files-result is-selected"
    } else {
        "find-in-files-result"
    };

    rsx! {
        button {
            class: "{row_class}",
            r#type: "button",
            onclick: move |_| props.onclick.call(()),
            Icon { width: 14, height: 14, icon: LdFileText, class: "shrink-0 text-[var(--text-muted)]" }
            div { class: "find-in-files-result-body",
                span { class: "find-in-files-result-path", "{display_path}" }
                span { class: "find-in-files-result-snippet", "{props.hit.snippet}" }
            }
        }
    }
}

fn format_display_path(path: &Path, root: Option<&Path>) -> String {
    if let Some(root) = root {
        path.strip_prefix(root)
            .map_or_else(|_| path.display().to_string(), |rel| rel.display().to_string())
    } else {
        path.display().to_string()
    }
}
