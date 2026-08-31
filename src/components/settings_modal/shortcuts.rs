use crate::components::Hint;
use crate::state::AppStore;
use crate::types::{ShortcutAction, ShortcutCategory, ShortcutKey};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCommand, LdRotateCcw, LdSparkles};

#[derive(Props, Clone, PartialEq)]
pub struct ShortcutsPaneProps {
    pub store: Signal<AppStore>,
    pub t: &'static crate::i18n::Translations,
    #[props(default)]
    pub search_filter: Option<String>,
}

pub fn has_matches(query: &str, t: &'static crate::i18n::Translations, store: &AppStore) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    if "shortcut".contains(&q)
        || "shortcuts".contains(&q)
        || "tastenkombination".contains(&q)
        || "tastenkürzel".contains(&q)
        || "hotkey".contains(&q)
        || "key".contains(&q)
        || t.shortcuts.title.to_lowercase().contains(&q)
        || t.shortcuts.description.to_lowercase().contains(&q)
    {
        return true;
    }
    ShortcutAction::all().iter().any(|&action| matches_action(action, &q, t, store))
}

fn matches_action(action: ShortcutAction, q: &str, t: &'static crate::i18n::Translations, store: &AppStore) -> bool {
    if q.is_empty() {
        return true;
    }
    let name = t.shortcuts.action_name(action).to_lowercase();
    let desc = t.shortcuts.action_desc(action).to_lowercase();
    let cat = t.shortcuts.category_name(action.category()).to_lowercase();
    let default_bind = action.default_binding().to_lowercase();
    let current_bind = store.settings.shortcuts.get_binding(action).to_lowercase();

    name.contains(q)
        || desc.contains(q)
        || cat.contains(q)
        || default_bind.contains(q)
        || current_bind.contains(q)
}

#[component]
pub fn ShortcutsPane(props: ShortcutsPaneProps) -> Element {
    let mut store = props.store;
    let store_read = store();
    let t = props.t;
    let filter = props.search_filter.as_deref().unwrap_or_default().trim().to_lowercase();

    let mut recording_action = use_signal(|| None::<ShortcutAction>);

    let categories = [
        ShortcutCategory::FileAndTabs,
        ShortcutCategory::LayoutAndModes,
        ShortcutCategory::EditorAndSearch,
        ShortcutCategory::ViewAndPreferences,
    ];

    // Effect to toggle the JS flag so the global listener doesn't intercept keys during recording
    use_effect(move || {
        let is_rec = recording_action().is_some();
        let _ = dioxus::prelude::document::eval(&format!(
            "window.__recordingShortcut = {is_rec};"
        ));
    });

    rsx! {
        div {
            class: "settings-section flex flex-col gap-5",

            // Top Header: Title, Description, Reset All Button
            div {
                class: "flex items-start justify-between gap-4 pb-3 border-b border-[var(--border-color)]",
                div {
                    class: "flex items-center gap-2",
                    Icon { width: 18, height: 18, icon: LdCommand, class: "text-[var(--accent)] shrink-0" }
                    div {
                        h3 { class: "text-sm font-semibold text-[var(--text-heading)] m-0 leading-tight", "{t.shortcuts.title}" }
                        p { class: "text-xs text-[var(--text-muted)] m-0 mt-0.5", "{t.shortcuts.description}" }
                    }
                }

                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        recording_action.set(None);
                        store.write().reset_all_shortcuts();
                    },
                    Icon { width: 13, height: 13, icon: LdRotateCcw, class: "opacity-70" }
                    span { "{t.shortcuts.reset_all}" }
                }
            }

            // Categories Group
            for category in categories {
                {
                    let category_actions: Vec<ShortcutAction> = ShortcutAction::all()
                        .iter()
                        .copied()
                        .filter(|a| a.category() == category && (filter.is_empty() || matches_action(*a, &filter, t, &store_read)))
                        .collect();

                    if category_actions.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div {
                                key: "{category:?}",
                                class: "shortcut-category-group flex flex-col gap-2 bg-[var(--bg-subtle)]/50 border border-[var(--border-color)] rounded-xl p-3.5",

                                div {
                                    class: "text-[11px] font-bold uppercase tracking-wider text-[var(--accent)] px-1 mb-1",
                                    "{t.shortcuts.category_name(category)}"
                                }

                                div {
                                    class: "flex flex-col divide-y divide-[var(--border-subtle)]",

                                    for action in category_actions {
                                        {
                                            let current_binding = store_read.settings.shortcuts.get_binding(action).to_string();
                                            let is_default = current_binding == action.default_binding();
                                            let is_recording = recording_action() == Some(action);

                                            rsx! {
                                                div {
                                                    key: "{action:?}",
                                                    class: "shortcut-row flex items-center justify-between py-2.5 px-2 hover:bg-[var(--bg-surface)]/60 rounded-lg transition-colors gap-3",

                                                    // Left: Action label & description
                                                    div {
                                                        class: "flex flex-col min-w-0 flex-1",
                                                        span { class: "text-xs font-medium text-[var(--text-heading)]", "{t.shortcuts.action_name(action)}" }
                                                        span { class: "text-[11px] text-[var(--text-muted)] truncate", "{t.shortcuts.action_desc(action)}" }
                                                    }

                                                    // Right: Key badge / Recording input & Reset button
                                                div {
                                                    class: "flex items-center gap-2 shrink-0",

                                                    if is_recording {
                                                        div {
                                                            class: "recording-badge inline-flex items-center gap-1.5 px-3 py-1 bg-[var(--accent)]/15 border-2 border-[var(--accent)] text-[var(--accent)] rounded-lg text-xs font-mono font-semibold animate-pulse outline-none cursor-pointer select-none",
                                                            tabindex: 0,
                                                            autofocus: true,
                                                            onkeydown: move |evt: KeyboardEvent| {
                                                                let key = evt.key();
                                                                let raw_key = match key {
                                                                    Key::Character(ref c) => c.clone(),
                                                                    Key::Escape => {
                                                                        recording_action.set(None);
                                                                        return;
                                                                    }
                                                                    _ => format!("{key:?}"),
                                                                };

                                                                let ctrl = evt.modifiers().ctrl();
                                                                let alt = evt.modifiers().alt();
                                                                let shift = evt.modifiers().shift();
                                                                let meta = evt.modifiers().meta();

                                                                // If only modifier key was pressed (Ctrl, Shift, Alt alone), wait for actual key
                                                                let is_modifier_only = matches!(
                                                                    raw_key.to_lowercase().as_str(),
                                                                    "control" | "shift" | "alt" | "meta" | "ctrl"
                                                                );

                                                                if !is_modifier_only {
                                                                    let key_obj = ShortcutKey {
                                                                        ctrl,
                                                                        meta,
                                                                        alt,
                                                                        shift,
                                                                        key: raw_key,
                                                                    };
                                                                    let canonical = key_obj.to_canonical_string();
                                                                    if !canonical.is_empty() {
                                                                        store.write().set_shortcut(action, canonical);
                                                                    }
                                                                    recording_action.set(None);
                                                                }
                                                            },
                                                            onblur: move |_| {
                                                                recording_action.set(None);
                                                            },
                                                            Icon { width: 12, height: 12, icon: LdSparkles }
                                                            span { "{t.shortcuts.press_keys}" }
                                                        }
                                                    } else {
                                                        Hint {
                                                            text: t.shortcuts.click_to_record,
                                                            button {
                                                                class: format!(
                                                                    "shortcut-key-btn inline-flex items-center px-2.5 py-1 rounded-md text-xs font-mono font-semibold cursor-pointer transition-all border {}",
                                                                    if is_default {
                                                                        "bg-[var(--bg-app)] border-[var(--border-color)] text-[var(--text-main)] hover:border-[var(--accent)] hover:text-[var(--accent)]"
                                                                    } else {
                                                                        "bg-[var(--accent)]/10 border-[var(--accent)]/50 text-[var(--accent)] hover:bg-[var(--accent)]/20"
                                                                    }
                                                                ),
                                                                onclick: move |_| {
                                                                    recording_action.set(Some(action));
                                                                },
                                                                "{current_binding}"
                                                            }
                                                        }
                                                    }

                                                    if !is_default {
                                                        Hint {
                                                            text: t.shortcuts.reset_shortcut,
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::IconXs,
                                                                class: "text-[var(--text-muted)] hover:text-[var(--text-heading)]",
                                                                onclick: move |_| {
                                                                    store.write().reset_shortcut(action);
                                                                },
                                                                Icon { width: 12, height: 12, icon: LdRotateCcw }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
}
