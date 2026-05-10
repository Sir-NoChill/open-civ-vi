//! Docs screen — Phase 5 polish placeholder.
//!
//! Will eventually embed the rendered mdBook (`book/`) inline as
//! an iframe or fetched-and-rendered Markdown. Today it surfaces
//! the most useful jump-points for a new contributor.

use leptos::prelude::*;

use crate::components::Panel;

#[component]
pub fn Docs() -> impl IntoView {
    view! {
        <div style="flex:1; overflow:auto">
            <div class="content-header">
                <div class="title">"Docs"</div>
                <span class="crumbs">"// rendered mdBook + roadmap + API reference"</span>
            </div>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"Quick links"</div>
                <div class="col" style="gap:8px">
                    <a href="/book/" target="_blank">
                        <strong>"📖 Open mdBook (new tab)"</strong>
                        <div class="muted xsmall">"Full project handbook — generated from "
                            <code>"book/src/"</code>
                            "."
                        </div>
                    </a>
                    <a href="/book/roadmap/accounts-and-login.html" target="_blank">
                        <strong>"🛠 Accounts & Login roadmap"</strong>
                        <div class="muted xsmall">"What this lobby is, where it's going, and what just landed."</div>
                    </a>
                    <a href="/book/multiplayer/web-client.html" target="_blank">
                        <strong>"🔌 Web client REST reference"</strong>
                        <div class="muted xsmall">"Endpoint table for the in-game " <code>"open4x-server"</code> " surface."</div>
                    </a>
                </div>
            </Panel>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"Status"</div>
                <p class="muted small" style="margin-top:0">
                    "The lobby serves "
                    <code>"./book/book/"</code>
                    " under "
                    <code>"/book/"</code>
                    " (overridable via "
                    <code>"OPEN4X_LOBBY_BOOK_DIR"</code>
                    "). Run "
                    <code>"mdbook build book/"</code>
                    " from the repo root once to populate the directory; rerun whenever "
                    "you edit the source under "
                    <code>"book/src/"</code>
                    "."
                </p>
            </Panel>
        </div>
    }
}
