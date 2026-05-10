//! `<Btn>` — button primitive matching `hifi/components.jsx`.

use leptos::prelude::*;

/// Render a `<button class="btn {variant} {size} {extra}">`.
///
/// `variant`: "" | "primary" | "accent" | "ghost" | "bare"
/// `size`:    "" | "xs" | "sm" | "lg"
#[component]
pub fn Btn(
    #[prop(optional, into)] variant: &'static str,
    #[prop(optional, into)] size: &'static str,
    #[prop(optional, into)] class: &'static str,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let class = format!("btn {variant} {size} {class}");
    let click = move |_| {
        if let Some(cb) = on_click {
            cb.run(());
        }
    };
    let disabled_attr = move || disabled.map(|s| s.get()).unwrap_or(false);
    view! {
        <button class=class on:click=click disabled=disabled_attr>
            {children()}
        </button>
    }
}
