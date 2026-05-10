//! Profile screen — scaffold port of `hifi/menu.jsx::Profile`.
//!
//! Renders the page chrome (header, three-panel grid: avatar/quick-actions,
//! profile fields, linked identities, preferences). Form inputs are
//! present but no save / unlink wiring — that lands when the
//! `open4x-accounts` HTTP surface ships.

use leptos::prelude::*;

use crate::components::{Btn, Panel, Segmented, segmented::Segment, Toggle};

#[component]
pub fn Profile() -> impl IntoView {
    let density = RwSignal::new("comfortable".to_string());
    let scheme = RwSignal::new("paper".to_string());
    let kbd = RwSignal::new(true);
    let notifs = RwSignal::new(true);
    let discoverable = RwSignal::new(true);

    let density_opts = Signal::derive(|| {
        ["compact", "comfortable", "spacious"].iter().map(|s| Segment::from_str(s)).collect()
    });
    let scheme_opts = Signal::derive(|| {
        ["paper", "ink", "auto"].iter().map(|s| Segment::from_str(s)).collect()
    });

    view! {
        <div style="flex:1; overflow:auto">
            <div class="content-header">
                <div class="title">"Profile & settings"</div>
                <span class="crumbs">
                    "player_id: "
                    <code class="trigger" style="font-family:var(--font-mono)"
                          title="Your unique identifier on the platform. Share it with friends so they can invite you.">
                        "0xA9C3·7F12·EE04"
                    </code>
                </span>
            </div>

            <div class="profile-grid">
                <Panel>
                    <div class="col" style="align-items:center; gap:12px">
                        <div class="avatar">"A"</div>
                        <Btn variant="ghost" size="sm">"change"</Btn>
                    </div>
                    <hr class="divider" />
                    <div class="h3" style="margin-bottom:10px">"Quick actions"</div>
                    <div class="col" style="gap:4px">
                        <Btn variant="bare" size="sm">"⎘ Copy player ID"</Btn>
                        <Btn variant="bare" size="sm">"▦ Show invite QR"</Btn>
                        <Btn variant="bare" size="sm">"↓ Export save data"</Btn>
                        <Btn variant="bare" size="sm">"→ Sign out"</Btn>
                    </div>
                </Panel>

                <div class="col">
                    <Panel>
                        <div class="h3" style="margin-bottom:14px">"Profile"</div>
                        <div class="field">
                            <label>"Preferred name"</label>
                            <input class="input" prop:value="Alice" />
                            <span class="hint">"Shown to other players in invites & chat."</span>
                        </div>
                        <div class="field">
                            <label>"Pronouns (optional)"</label>
                            <input class="input" prop:value="she/her" />
                        </div>
                        <div class="field">
                            <label>"Bio"</label>
                            <textarea class="input" rows="2">"Plays slow. Reads everything."</textarea>
                            <span class="hint">"Markdown supported. Visible on invite cards."</span>
                        </div>
                    </Panel>

                    <Panel>
                        <div class="row between center-y" style="margin-bottom:12px">
                            <div class="h3">"Linked identities"</div>
                            <Btn variant="ghost" size="sm">"+ link another"</Btn>
                        </div>

                        <div class="id-row primary">
                            <span class="id-type">"EMAIL · primary"</span>
                            <span class="id-val">"alice@example.com"</span>
                            <Btn variant="bare" size="sm">"manage"</Btn>
                        </div>
                        <div class="id-row">
                            <span class="id-type">"OPENID"</span>
                            <span class="id-val">"google.com / 110293·a73f"</span>
                            <Btn variant="bare" size="sm">"unlink"</Btn>
                        </div>
                        <div class="id-row">
                            <span class="id-type">"OPENID"</span>
                            <span class="id-val">"github.com/alice"</span>
                            <Btn variant="bare" size="sm">"unlink"</Btn>
                        </div>
                        <div class="id-row">
                            <span class="id-type">"ATPROTO"</span>
                            <span class="id-val">"did:plc:abcd1234efgh5678 · alice.bsky.social"</span>
                            <Btn variant="bare" size="sm">"unlink"</Btn>
                        </div>

                        <p class="muted xsmall" style="margin-top:10px">
                            "All four identities map to the same player. "
                            "Friends can find you by any of them."
                        </p>
                    </Panel>

                    <Panel>
                        <div class="h3" style="margin-bottom:12px">"Preferences"</div>

                        <div class="param-row">
                            <div class="label">"Density"</div>
                            <div class="control"><Segmented options=density_opts value=density /></div>
                            <div class="value muted xsmall">"→ tweaks panel"</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Color scheme"</div>
                            <div class="control"><Segmented options=scheme_opts value=scheme /></div>
                            <div class="value muted xsmall">{move || scheme.get()}</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Keyboard nav"</div>
                            <div class="control">
                                <Toggle on=kbd
                                        on_change=Callback::new(move |v| kbd.set(v)) />
                            </div>
                            <div class="value muted xsmall">"vim bindings"</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Turn notifications"</div>
                            <div class="control">
                                <Toggle on=notifs
                                        on_change=Callback::new(move |v| notifs.set(v)) />
                            </div>
                            <div class="value muted xsmall">"email + push"</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Discoverable by ID"</div>
                            <div class="control">
                                <Toggle on=discoverable
                                        on_change=Callback::new(move |v| discoverable.set(v)) />
                            </div>
                            <div class="value muted xsmall">"others can invite"</div>
                        </div>
                    </Panel>
                </div>
            </div>
        </div>
    }
}
