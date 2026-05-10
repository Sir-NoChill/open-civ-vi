//! `<Toggle>` — binary switch styled by the `.toggle` rules in `styles.css`.

use leptos::prelude::*;

#[component]
pub fn Toggle(
    on: RwSignal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
) -> impl IntoView {
    let click = move |_| {
        let next = !on.get_untracked();
        if let Some(cb) = on_change {
            cb.run(next);
        } else {
            on.set(next);
        }
    };
    view! {
        <button
            class="toggle"
            class:on=move || on.get()
            aria-pressed=move || on.get().to_string()
            on:click=click
        ></button>
    }
}
