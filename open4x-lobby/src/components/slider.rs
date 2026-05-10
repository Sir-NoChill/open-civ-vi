//! `<Slider>` — port of the JSX `Slider` primitive.
//!
//! Wraps `<input type="range">` and a sibling `<span class="value">` that
//! displays the current value. Pass an optional `format` callback to
//! map the integer to a richer label (e.g. `"4 bn"`, `"temperate"`,
//! `"warlike"`).

use std::sync::Arc;

use leptos::prelude::*;

/// Callback shape for the `format` prop. Boxed so the prop type is sized
/// and `Clone`. Wrap any `Fn(i32) -> String + Send + Sync + 'static` in
/// [`Arc::new`] before passing.
pub type FormatFn = Arc<dyn Fn(i32) -> String + Send + Sync>;

#[component]
pub fn Slider(
    value: RwSignal<i32>,
    #[prop(default = 0)] min: i32,
    #[prop(default = 100)] max: i32,
    #[prop(optional)] format: Option<FormatFn>,
) -> impl IntoView {
    let display = {
        let format = format.clone();
        move || match &format {
            Some(f) => f(value.get()),
            None => value.get().to_string(),
        }
    };

    let on_input = move |ev: web_sys::Event| {
        use wasm_bindgen::JsCast as _;
        if let Some(el) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            if let Ok(v) = el.value().parse::<i32>() {
                value.set(v);
            }
        }
    };

    view! {
        <input
            type="range"
            class="range"
            min=min
            max=max
            prop:value=move || value.get()
            on:input=on_input
        />
        <span class="value">{display}</span>
    }
}
