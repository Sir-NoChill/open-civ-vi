//! Presets screen — Phase 5 polish placeholder.
//!
//! Save / load / import-JSON wizard configurations. The UI shape
//! lives here; persistence + the JSON-import handler are pending.

use leptos::prelude::*;

use crate::components::{Btn, Panel};

#[component]
pub fn Presets() -> impl IntoView {
    view! {
        <div style="flex:1; overflow:auto">
            <div class="content-header">
                <div class="title">"Presets"</div>
                <span class="crumbs">"// save / load / import wizard configs"</span>
                <div class="actions">
                    <Btn variant="ghost" size="sm">"↑ import JSON…"</Btn>
                    <Btn variant="accent">"+ Save current"</Btn>
                </div>
            </div>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"Built-in"</div>
                <div class="col" style="gap:6px">
                    <div class="row between center-y" style="border-bottom:1px solid var(--hairline-2); padding:6px 0">
                        <div>
                            <div style="font-weight:600">"Standard prince"</div>
                            <div class="muted xsmall">"continents · standard · prince · 1H + 7AI"</div>
                        </div>
                        <Btn variant="ghost" size="sm">"load"</Btn>
                    </div>
                    <div class="row between center-y" style="border-bottom:1px solid var(--hairline-2); padding:6px 0">
                        <div>
                            <div style="font-weight:600">"Deity duel"</div>
                            <div class="muted xsmall">"pangaea · duel · deity · 2H"</div>
                        </div>
                        <Btn variant="ghost" size="sm">"load"</Btn>
                    </div>
                    <div class="row between center-y" style="padding:6px 0">
                        <div>
                            <div style="font-weight:600">"Slow marathon"</div>
                            <div class="muted xsmall">"continents · large · prince · marathon speed · 1H + 9AI"</div>
                        </div>
                        <Btn variant="ghost" size="sm">"load"</Btn>
                    </div>
                </div>
            </Panel>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"My presets"</div>
                <p class="muted small">
                    "No saved presets yet. Click " <strong>"+ Save current"</strong>
                    " from the New game wizard."
                </p>
            </Panel>

            <p class="muted xsmall" style="margin-top:14px; text-align:center">
                "// preset persistence pending — Phase 5 polish"
            </p>
        </div>
    }
}
