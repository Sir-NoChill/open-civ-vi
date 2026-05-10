//! Friends screen — Phase 5 polish placeholder.
//!
//! Visual port of the design's "Friends" tab. The interactive
//! search-by-identity / friend-requests / friends-list flows land
//! when the friends schema + routes ship.

use leptos::prelude::*;

use crate::components::{Btn, Panel};

#[component]
pub fn Friends() -> impl IntoView {
    view! {
        <div style="flex:1; overflow:auto">
            <div class="content-header">
                <div class="title">"Friends"</div>
                <span class="crumbs">"// search by email · OpenID · atproto · player ID"</span>
                <div class="actions">
                    <Btn variant="accent">"+ Add friend"</Btn>
                </div>
            </div>

            <Panel>
                <div class="filter-bar" style="margin:0">
                    <span class="muted xsmall" style="padding-left:4px">"⌕"</span>
                    <input
                        class="filter-search"
                        placeholder="paste an identity to add — alice@… · did:plc:… · 0xA9C3·…"
                    />
                </div>
            </Panel>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"Friends"</div>
                <p class="muted small">
                    "No friends yet. Search above to send your first friend request."
                </p>
            </Panel>

            <Panel>
                <div class="h3" style="margin-bottom:10px">"Requests"</div>
                <p class="muted small">"No pending requests."</p>
            </Panel>

            <p class="muted xsmall" style="margin-top:14px; text-align:center">
                "// schema + routes pending — Phase 5 polish · "
                <code>"book/src/roadmap/accounts-and-login.md"</code>
            </p>
        </div>
    }
}
