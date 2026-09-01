use crate::i18n::Translations;
use crate::state::AppStore;
use crate::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
};
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct EditorContextMenuProps {
    pub t: &'static Translations,
    pub children: Element,
}

#[component]
pub fn EditorContextMenu(props: EditorContextMenuProps) -> Element {
    let cm = props.t.context_menu;
    rsx! {
        ContextMenu {
            class: "flex-1 min-w-0 min-h-0 h-full w-full",
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                ContextMenuItem {
                    index: 0usize,
                    value: "cut".to_string(),
                    on_select: move |_| {
                        let _ = document::eval("document.execCommand('cut');");
                    },
                    "{cm.cut}"
                }
                ContextMenuItem {
                    index: 1usize,
                    value: "copy".to_string(),
                    on_select: move |_| {
                        let _ = document::eval("document.execCommand('copy');");
                    },
                    "{cm.copy}"
                }
                ContextMenuItem {
                    index: 2usize,
                    value: "paste".to_string(),
                    on_select: move |_| {
                        let _ = document::eval("document.execCommand('paste');");
                    },
                    "{cm.paste}"
                }
                ContextMenuItem {
                    index: 3usize,
                    value: "select_all".to_string(),
                    on_select: move |_| {
                        let _ = document::eval("document.execCommand('selectAll');");
                    },
                    "{cm.select_all}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PreviewContextMenuProps {
    pub t: &'static Translations,
    pub children: Element,
}

#[component]
pub fn PreviewContextMenu(props: PreviewContextMenuProps) -> Element {
    let cm = props.t.context_menu;
    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                ContextMenuItem {
                    index: 0usize,
                    value: "copy_selection".to_string(),
                    on_select: move |_| {
                        let _ = document::eval("document.execCommand('copy');");
                    },
                    "{cm.copy_selection}"
                }
                ContextMenuItem {
                    index: 1usize,
                    value: "open_link".to_string(),
                    on_select: move |_| {
                        let _ = document::eval(
                            r"
                            const href = window.__ctxHref;
                            if (href) {
                                window.open(href, '_blank', 'noopener,noreferrer');
                            }
                            ",
                        );
                    },
                    "{cm.open_link}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TabContextMenuProps {
    pub t: &'static Translations,
    pub tab_id: usize,
    pub tab_path: Option<PathBuf>,
    pub is_preview: bool,
    pub can_close: bool,
    pub can_close_others: bool,
    pub store: Signal<AppStore>,
    pub children: Element,
}

#[component]
pub fn TabContextMenu(props: TabContextMenuProps) -> Element {
    let cm = props.t.context_menu;
    let close_label = props.t.tab_bar.close_tab;
    let keep_open_label = props.t.tab_bar.keep_open;
    let tab_id = props.tab_id;
    let mut store = props.store;
    let can_close = props.can_close;
    let can_close_others = props.can_close_others;
    let is_preview = props.is_preview;

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                if is_preview {
                    ContextMenuItem {
                        index: 0usize,
                        value: "keep_open".to_string(),
                        on_select: move |_| {
                            store.write().pin_tab(tab_id);
                        },
                        "{keep_open_label}"
                    }
                }
                if can_close {
                    ContextMenuItem {
                        index: 1usize,
                        value: "close".to_string(),
                        on_select: move |_| {
                            store.write().close_tab(tab_id);
                        },
                        "{close_label}"
                    }
                }
                if can_close_others {
                    ContextMenuItem {
                        index: 2usize,
                        value: "close_others".to_string(),
                        on_select: move |_| {
                            store.write().close_other_tabs(tab_id);
                        },
                        "{cm.close_others}"
                    }
                }
                ContextMenuItem {
                    index: 3usize,
                    value: "export_html".to_string(),
                    on_select: move |_| {
                        let s = store();
                        if let Some(tab) = s.tabs.iter().find(|t| t.id == tab_id) {
                            let title = tab.title.clone();
                            let html_content = tab.parsed.html_content.clone();
                            let theme_str = s.theme.as_str().to_string();
                            let custom_accent_style = s.primary_color.as_ref().map_or_else(String::new, |color| {
                                format!("--accent: {color}; --accent-hover: {color}; --accent-glow: {color}40;")
                            });
                            spawn(async move {
                                let _ = crate::services::fs::export_tab_html_async(
                                    &title,
                                    &html_content,
                                    &theme_str,
                                    &custom_accent_style,
                                    crate::APP_STYLES,
                                ).await;
                            });
                        }
                    },
                    "{props.t.toolbar.export_html}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FileTreeContextMenuProps {
    pub t: &'static Translations,
    pub path: PathBuf,
    pub is_dir: bool,
    pub store: Signal<AppStore>,
    pub on_open: EventHandler<PathBuf>,
    pub children: Element,
}

#[component]
pub fn FileTreeContextMenu(props: FileTreeContextMenuProps) -> Element {
    let cm = props.t.context_menu;
    let settings = props.t.settings;
    let path = props.path.clone();
    let open_path = path.clone();
    let copy_path = path.clone();
    let refresh_path = path.clone();
    let is_dir = props.is_dir;
    let mut store = props.store;

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                ContextMenuItem {
                    index: 0usize,
                    value: "open".to_string(),
                    on_select: move |_| {
                        props.on_open.call(open_path.clone());
                    },
                    "{cm.open}"
                }
                ContextMenuItem {
                    index: 1usize,
                    value: "copy_path".to_string(),
                    on_select: move |_| {
                        let p = copy_path.display().to_string();
                        let _ = document::eval(&format!(
                            "navigator.clipboard && navigator.clipboard.writeText({p:?});"
                        ));
                    },
                    "{settings.copy_path}"
                }
                if is_dir {
                    ContextMenuItem {
                        index: 2usize,
                        value: "refresh".to_string(),
                        on_select: move |_| {
                            let dir = refresh_path.clone();
                            store.write().open_directory(dir);
                        },
                        "{cm.refresh}"
                    }
                }
            }
        }
    }
}
