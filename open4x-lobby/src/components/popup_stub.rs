//! Placeholder for the Gwern-style hover popup component used heavily in the
//! design. The full popup needs floating-position math, hover-debounce, pin
//! state, and esc-to-close handling — non-trivial. Until that ships, every
//! `<Popup><span class="trigger">…</span></Popup>` from the design is
//! rendered as a plain `<Trigger>` here, with the popup body discarded.
//!
//! When the real component lands, the call sites can be updated in place.

use leptos::prelude::*;

/// Underlined help-trigger text. Falls back to the `title` attribute so the
/// popup body is still discoverable as a tooltip in the meantime.
#[component]
pub fn Trigger(
    #[prop(optional, into)] hint: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class="trigger" title=hint>
            {children()}
        </span>
    }
}
