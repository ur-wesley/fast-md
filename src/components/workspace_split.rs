use crate::state::AppStore;
use crate::types::SidebarPosition;
use dioxus::prelude::*;

pub const SIDEBAR_MIN_WIDTH: u32 = 180;
pub const SIDEBAR_MAX_WIDTH: u32 = 560;

#[derive(Props, Clone, PartialEq)]
pub struct WorkspaceSplitProps {
    pub store: Signal<AppStore>,
    pub show_sidebar: bool,
    pub sidebar_position: SidebarPosition,
    pub sidebar: Element,
    pub content: Element,
}

#[component]
pub fn WorkspaceSplit(props: WorkspaceSplitProps) -> Element {
    let mut store = props.store;
    let is_right = props.sidebar_position.is_right();
    let mut sidebar_width =
        use_signal(|| store().sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH));
    let mut is_resizing = use_signal(|| false);
    let mut initial_mouse_x = use_signal(|| 0.0f64);
    let mut initial_sidebar_w = use_signal(|| 0u32);

    use_effect(move || {
        if is_resizing() {
            return;
        }
        let saved = store().sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        sidebar_width.set(saved);
    });

    let finish_resize = move |_| {
        if !is_resizing() {
            return;
        }
        is_resizing.set(false);
        store.write().set_sidebar_width(sidebar_width());
    };

    rsx! {
        div {
            class: if is_right {
                if is_resizing() { "workspace-split sidebar-is-right is-resizing" } else { "workspace-split sidebar-is-right" }
            } else {
                if is_resizing() { "workspace-split sidebar-is-left is-resizing" } else { "workspace-split sidebar-is-left" }
            },
            id: "workspace-split",

            // Left Sidebar
            if !is_right {
                div {
                    class: if props.show_sidebar {
                        "workspace-split-sidebar sidebar-left is-open"
                    } else {
                        "workspace-split-sidebar sidebar-left is-collapsed"
                    },
                    style: if props.show_sidebar {
                        format!("width: {}px;", sidebar_width())
                    } else {
                        "width: 0px;".to_string()
                    },
                    ontransitionend: move |_| {
                        dioxus::prelude::document::eval("window.refreshTocTreePath && window.refreshTocTreePath();");
                    },
                    div {
                        class: "workspace-split-sidebar-inner",
                        style: format!("width: {}px; min-width: {}px;", sidebar_width(), sidebar_width()),
                        {props.sidebar.clone()}
                    }
                }

                div {
                    class: if props.show_sidebar {
                        "workspace-split-gutter gutter-left is-open"
                    } else {
                        "workspace-split-gutter gutter-left is-collapsed"
                    },
                    onmousedown: move |evt| {
                        if !props.show_sidebar {
                            return;
                        }
                        evt.prevent_default();
                        evt.stop_propagation();
                        let client_x = evt.data().client_coordinates().x;
                        initial_mouse_x.set(client_x);
                        initial_sidebar_w.set(sidebar_width());
                        is_resizing.set(true);
                    },
                }
            }

            // Main Content Area (Tab Bar + Editor/Viewer)
            div {
                class: "workspace-split-content",
                {props.content}
            }

            // Right Sidebar
            if is_right {
                div {
                    class: if props.show_sidebar {
                        "workspace-split-gutter gutter-right is-open"
                    } else {
                        "workspace-split-gutter gutter-right is-collapsed"
                    },
                    onmousedown: move |evt| {
                        if !props.show_sidebar {
                            return;
                        }
                        evt.prevent_default();
                        evt.stop_propagation();
                        let client_x = evt.data().client_coordinates().x;
                        initial_mouse_x.set(client_x);
                        initial_sidebar_w.set(sidebar_width());
                        is_resizing.set(true);
                    },
                }

                div {
                    class: if props.show_sidebar {
                        "workspace-split-sidebar sidebar-right is-open"
                    } else {
                        "workspace-split-sidebar sidebar-right is-collapsed"
                    },
                    style: if props.show_sidebar {
                        format!("width: {}px;", sidebar_width())
                    } else {
                        "width: 0px;".to_string()
                    },
                    ontransitionend: move |_| {
                        dioxus::prelude::document::eval("window.refreshTocTreePath && window.refreshTocTreePath();");
                    },
                    div {
                        class: "workspace-split-sidebar-inner",
                        style: format!("width: {}px; min-width: {}px;", sidebar_width(), sidebar_width()),
                        {props.sidebar}
                    }
                }
            }

            if is_resizing() {
                div {
                    class: "workspace-resize-overlay",
                    onmousemove: move |evt| {
                        let current_x = evt.data().client_coordinates().x;
                        let delta = current_x - initial_mouse_x();
                        let base_w = i64::from(initial_sidebar_w());
                        let next_w = if is_right {
                            (base_w - (delta.round() as i64))
                                .clamp(i64::from(SIDEBAR_MIN_WIDTH), i64::from(SIDEBAR_MAX_WIDTH)) as u32
                        } else {
                            (base_w + (delta.round() as i64))
                                .clamp(i64::from(SIDEBAR_MIN_WIDTH), i64::from(SIDEBAR_MAX_WIDTH)) as u32
                        };
                        sidebar_width.set(next_w);
                    },
                    onmouseup: finish_resize,
                    onmouseleave: finish_resize,
                }
            }
        }
    }
}
