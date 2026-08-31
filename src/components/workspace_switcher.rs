use crate::services::fs::pick_folder_async;
use crate::services::workspace::{canonical_workspace_key, workspace_keys_equal};
use crate::state::AppStore;
use crate::ui::input::Input;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCheck, LdChevronDown, LdFolderOpen};

#[derive(Props, Clone, PartialEq)]
pub struct WorkspaceSwitcherProps {
    pub store: Signal<AppStore>,
}

#[component]
pub fn WorkspaceSwitcher(props: WorkspaceSwitcherProps) -> Element {
    let mut store = props.store;
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let store_read = store();
    let t = store_read.language.strings();

    let workspace_label = store_read
        .opened_folder
        .as_ref()
        .and_then(|folder| folder.file_name())
        .map_or_else(|| t.title_bar.workspace.to_string(), |name| name.to_string_lossy().to_string());

    let current_key = store_read
        .opened_folder
        .as_ref()
        .and_then(|folder| canonical_workspace_key(folder));

    let query_lower = search_query().to_lowercase();

    let recent_workspaces: Vec<_> = store_read
        .workspaces
        .workspaces
        .iter()
        .filter(|ws| {
            if let Some(ref key) = current_key {
                !workspace_keys_equal(&ws.folder, key)
            } else {
                true
            }
        })
        .filter(|ws| {
            if query_lower.is_empty() {
                return true;
            }
            let name = ws
                .folder
                .file_name()
                .map_or_else(|| ws.folder.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());
            name.to_lowercase().contains(&query_lower)
                || ws.folder.to_string_lossy().to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();

    let show_current = current_key.is_some()
        && (query_lower.is_empty()
            || workspace_label.to_lowercase().contains(&query_lower)
            || store_read
                .opened_folder
                .as_ref()
                .is_some_and(|f| f.to_string_lossy().to_lowercase().contains(&query_lower)));

    rsx! {
        div {
            class: "workspace-switcher relative shrink-0",
            onmousedown: move |evt| evt.stop_propagation(),

            button {
                class: "workspace-switcher-trigger inline-flex items-center gap-1 max-w-[200px] text-[11px] text-[var(--text-heading)] bg-[var(--bg-subtle)] px-2 py-0.5 rounded border border-[var(--border-subtle)] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors",
                title: "{workspace_label}",
                onclick: move |_| is_open.set(!is_open()),
                span { class: "truncate font-medium", "{workspace_label}" }
                Icon { width: 11, height: 11, icon: LdChevronDown, class: "shrink-0 text-[var(--text-muted)]" }
            }

            if is_open() {
                div {
                    class: "workspace-switcher-backdrop fixed inset-0 z-[150]",
                    onclick: move |_| {
                        is_open.set(false);
                        search_query.set(String::new());
                    },
                }
                div {
                    class: "workspace-switcher-menu absolute left-0 top-[calc(100%+4px)] z-[160] w-[280px] max-h-[420px] overflow-hidden flex flex-col bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg shadow-lg",
                    onmousedown: move |evt| evt.stop_propagation(),

                    div { class: "p-2 border-b border-[var(--border-color)]",
                        Input {
                            class: "w-full bg-[var(--bg-app)] border border-[var(--border-color)] rounded-md px-2 py-1 text-xs text-[var(--text-main)] outline-none focus:border-[var(--accent)]",
                            r#type: "text",
                            placeholder: "{t.title_bar.workspace_search_placeholder}",
                            value: "{search_query()}",
                            oninput: move |evt: FormEvent| search_query.set(evt.value()),
                            onkeydown: move |evt: KeyboardEvent| {
                                if evt.key() == Key::Escape {
                                    is_open.set(false);
                                    search_query.set(String::new());
                                }
                            },
                        }
                    }

                    div { class: "overflow-y-auto flex-1 py-1",
                        if show_current {
                            div { class: "px-2 py-1",
                                div { class: "text-[10px] uppercase tracking-wide text-[var(--text-muted)] px-2 py-1",
                                    "{t.title_bar.workspace_this_window}"
                                }
                                button {
                                    class: "workspace-switcher-item w-full flex items-center justify-between gap-2 px-2 py-1.5 rounded text-xs text-[var(--text-heading)] bg-[var(--bg-hover)] cursor-default",
                                    span { class: "truncate font-medium", "{workspace_label}" }
                                    Icon { width: 13, height: 13, icon: LdCheck, class: "shrink-0 text-[var(--accent)]" }
                                }
                            }
                        }

                        if !recent_workspaces.is_empty() {
                            div { class: "px-2 py-1",
                                div { class: "text-[10px] uppercase tracking-wide text-[var(--text-muted)] px-2 py-1",
                                    "{t.title_bar.workspace_recent_projects}"
                                }
                                for ws in recent_workspaces {
                                    {
                                        let folder = ws.folder.clone();
                                        let label = folder
                                            .file_name()
                                            .map_or_else(|| folder.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());
                                        rsx! {
                                            button {
                                                key: "{folder.display()}",
                                                class: "workspace-switcher-item w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-heading)] cursor-pointer text-left",
                                                onclick: move |_| {
                                                    store.write().switch_workspace(folder.clone());
                                                    is_open.set(false);
                                                    search_query.set(String::new());
                                                },
                                                span { class: "truncate", "{label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "border-t border-[var(--border-color)] p-2",
                        button {
                            class: "workspace-switcher-open w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-[var(--text-heading)] hover:bg-[var(--bg-hover)] cursor-pointer text-left",
                            onclick: move |_| {
                                is_open.set(false);
                                search_query.set(String::new());
                                spawn(async move {
                                    if let Some(dir) = pick_folder_async().await {
                                        store.write().switch_workspace(dir);
                                    }
                                });
                            },
                            Icon { width: 13, height: 13, icon: LdFolderOpen, class: "shrink-0 text-[var(--accent)]" }
                            span { "{t.title_bar.workspace_open_folder}" }
                        }
                    }
                }
            }
        }
    }
}
