use crate::ui::virtual_list::{relative_scroll, total_list_height, visible_range};
use dioxus::prelude::*;

#[derive(serde::Deserialize)]
struct NestedWindowMsg {
    parent_top: f64,
    list_top: f64,
    viewport: f64,
}

#[derive(Props, Clone, PartialEq)]
pub struct VirtualListProps {
    pub item_count: usize,
    pub row_height: u32,
    #[props(default = 8)]
    pub overscan: usize,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub list_id: Option<String>,
    #[props(default)]
    pub scroll_top: Option<Signal<f64>>,
    #[props(default)]
    pub window_tick: Option<Signal<u32>>,
    pub overlay: Element,
    pub render_row: Callback<usize, Element>,
}

#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let mut scroll_top = use_signal(|| 0.0_f64);
    let mut viewport_height = use_signal(|| 400.0_f64);
    let nested = props.window_tick.is_some();
    let item_count = props.item_count;
    let row_height = props.row_height;
    let overscan = props.overscan;
    let total_height = total_list_height(item_count, row_height);
    let list_id = props.list_id.clone();
    let scroll_id_attr = list_id.clone().unwrap_or_default();

    if let Some(external_scroll) = props.scroll_top {
        let list_id_for_effect = list_id.clone();
        use_effect(move || {
            let top = external_scroll();
            if nested {
                return;
            }
            scroll_top.set(top);
            if let Some(ref id) = list_id_for_effect {
                let id_js = id.replace('\\', "\\\\").replace('\'', "\\'");
                dioxus::prelude::document::eval(&format!(
                    "const el = document.getElementById('{id_js}'); if (el) el.scrollTop = {top};"
                ));
            }
        });
    }

    let nested_list_id = if nested { list_id.clone() } else { None };
    let _nested_bind = use_coroutine(move |_: UnboundedReceiver<()>| {
        let nested_list_id = nested_list_id.clone();
        async move {
            let Some(id) = nested_list_id else {
                return;
            };
            let id_js = id.replace('\\', "\\\\").replace('\'', "\\'");
            let mut eval = dioxus::prelude::document::eval(&format!(
                r"
                (async () => {{
                    let list = null;
                    for (let i = 0; i < 90; i++) {{
                        list = document.getElementById('{id_js}');
                        if (list) break;
                        await new Promise((r) => requestAnimationFrame(r));
                    }}
                    if (!list) return;
                    const parent = list.closest('#viewer-scroll-area, #split-preview-scroll-area');
                    if (!parent) return;
                    const send = () => {{
                        const pr = parent.getBoundingClientRect();
                        const lr = list.getBoundingClientRect();
                        dioxus.send({{ parent_top: pr.top, list_top: lr.top, viewport: parent.clientHeight }});
                    }};
                    send();
                    parent.addEventListener('scroll', send, {{ passive: true }});
                    window.addEventListener('resize', send);
                    new ResizeObserver(send).observe(parent);
                }})();
                "
            ));
            while let Ok(msg) = eval.recv::<NestedWindowMsg>().await {
                scroll_top.set(relative_scroll(msg.parent_top, msg.list_top));
                viewport_height.set(msg.viewport.max(1.0));
            }
        }
    });

    if let Some(tick) = props.window_tick {
        use_effect(move || {
            let _ = tick();
            let Some(id) = list_id.clone() else {
                return;
            };
            spawn(async move {
                let id_js = id.replace('\\', "\\\\").replace('\'', "\\'");
                let mut eval = dioxus::prelude::document::eval(&format!(
                    r"
                    const list = document.getElementById('{id_js}');
                    if (!list) return;
                    const parent = list.closest('#viewer-scroll-area, #split-preview-scroll-area');
                    if (!parent) return;
                    const pr = parent.getBoundingClientRect();
                    const lr = list.getBoundingClientRect();
                    dioxus.send({{ parent_top: pr.top, list_top: lr.top, viewport: parent.clientHeight }});
                    "
                ));
                if let Ok(msg) = eval.recv::<NestedWindowMsg>().await {
                    scroll_top.set(relative_scroll(msg.parent_top, msg.list_top));
                    viewport_height.set(msg.viewport.max(1.0));
                }
            });
        });
    }

    let (start, end, top_pad, bottom_pad) = visible_range(
        scroll_top(),
        viewport_height(),
        item_count,
        row_height,
        overscan,
    );

    let overflow = if nested { "visible" } else { "auto" };

    rsx! {
        div {
            id: if scroll_id_attr.is_empty() { None } else { Some(scroll_id_attr) },
            class: "{props.class}",
            style: "overflow-y: {overflow}; flex: 1; min-height: 0;",
            onmounted: move |_| {
                if let Some(mut tick) = props.window_tick {
                    tick.set(tick().saturating_add(1));
                }
            },
            onscroll: move |evt| {
                if nested {
                    return;
                }
                scroll_top.set(evt.scroll_top());
                viewport_height.set(f64::from(evt.client_height()));
            },
            div {
                style: "height: {total_height}px; position: relative; width: 100%;",
                {props.overlay}
                if top_pad > 0 {
                    div { style: "height: {top_pad}px;" }
                }
                for idx in start..end {
                    div {
                        key: "{idx}",
                        style: "height: {row_height}px; box-sizing: border-box; position: relative; z-index: 2;",
                        {props.render_row.call(idx)}
                    }
                }
                if bottom_pad > 0 {
                    div { style: "height: {bottom_pad}px;" }
                }
            }
        }
    }
}
