use crate::ui::virtual_list::{total_list_height, visible_range};
use dioxus::prelude::*;

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
    pub overlay: Element,
    pub render_row: Callback<usize, Element>,
}

#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let mut scroll_top = use_signal(|| 0.0_f64);
    let mut viewport_height = use_signal(|| 400.0_f64);
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
            scroll_top.set(top);
            if let Some(ref id) = list_id_for_effect {
                let id_js = id.replace('\\', "\\\\").replace('\'', "\\'");
                dioxus::prelude::document::eval(&format!(
                    "const el = document.getElementById('{id_js}'); if (el) el.scrollTop = {top};"
                ));
            }
        });
    }

    let (start, end, top_pad, bottom_pad) = visible_range(
        scroll_top(),
        viewport_height(),
        item_count,
        row_height,
        overscan,
    );

    rsx! {
        div {
            id: if scroll_id_attr.is_empty() { None } else { Some(scroll_id_attr) },
            class: "{props.class}",
            style: "overflow-y: auto; flex: 1; min-height: 0;",
            onscroll: move |evt| {
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
