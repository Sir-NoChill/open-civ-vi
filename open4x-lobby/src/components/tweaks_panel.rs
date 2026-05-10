//! Floating tweaks panel — port of the JSX `TweaksPanel` shell in
//! `docs/open4x-landing/project/tweaks-panel.jsx`, scoped to what the
//! lobby SPA actually needs today: a runtime density picker.
//!
//! Mounts as a fixed-position card (bottom-right) and exposes a
//! Segmented control bound to a parent-owned `RwSignal<String>` that
//! the [`crate::app::App`] reads back into the root `.app` element's
//! `data-density` attribute.
//!
//! The full JSX panel ships sliders, color pickers, toggles, and a
//! postMessage host protocol — all out of scope for the lobby SPA
//! today. Extend this module if those are needed.

use leptos::prelude::*;

use crate::components::{Segmented, segmented::Segment};

#[component]
pub fn TweaksPanel(density: RwSignal<String>) -> impl IntoView {
    let visible = RwSignal::new(true);
    let density_opts = Signal::derive(|| {
        ["compact", "comfortable", "spacious"]
            .iter()
            .map(|s| Segment::from_str(s))
            .collect()
    });

    view! {
        <div style="position:fixed; right:14px; bottom:14px; z-index:100">
            {move || if visible.get() {
                view! {
                    <div style="background:var(--paper); border:1px solid var(--ink); \
                                padding:10px 12px; min-width:220px; \
                                box-shadow:4px 4px 0 var(--ink); font-family:var(--font-mono); \
                                font-size:var(--fs-xs)">
                        <div class="row between center-y" style="margin-bottom:8px">
                            <span class="h3" style="font-size:var(--fs-xs); letter-spacing:0.08em">
                                "Tweaks"
                            </span>
                            <button
                                class="btn xs ghost"
                                on:click=move |_| visible.set(false)
                                style="border:none; padding:0 4px"
                            >"×"</button>
                        </div>
                        <div class="muted xsmall" style="margin-bottom:6px">"Density"</div>
                        <Segmented options=density_opts value=density />
                    </div>
                }.into_any()
            } else {
                view! {
                    <button
                        class="btn xs ghost"
                        on:click=move |_| visible.set(true)
                        style="background:var(--paper); box-shadow:2px 2px 0 var(--ink)"
                    >
                        "⚙ tweaks"
                    </button>
                }.into_any()
            }}
        </div>
    }
}
