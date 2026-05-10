//! `<Panel>` and `<PanelHead>` — recurring container shapes used across
//! every screen. Match the design's `.panel` / `.panel.flush` /
//! `.panel-head` / `.panel-body` CSS.

use leptos::prelude::*;

#[component]
pub fn Panel(
    #[prop(optional)] flush: bool,
    #[prop(optional, into)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let mut full_class = String::from("panel");
    if flush {
        full_class.push_str(" flush");
    }
    if !class.is_empty() {
        full_class.push(' ');
        full_class.push_str(class);
    }
    view! { <div class=full_class>{children()}</div> }
}

#[component]
pub fn PanelHead(
    #[prop(into)] title: String,
    #[prop(optional, into)] sub: String,
    #[prop(optional)] right: Option<Children>,
) -> impl IntoView {
    let has_sub = !sub.is_empty();
    view! {
        <div class="panel-head">
            <span class="title">{title}</span>
            {has_sub.then(|| view! { <span class="sub">{sub}</span> })}
            {right.map(|r| view! { <div style="margin-left:auto">{r()}</div> })}
        </div>
    }
}
