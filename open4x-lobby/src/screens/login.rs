//! Login screen — port of `docs/open4x-landing/project/hifi/login.jsx`.
//!
//! Three stacked auth panels (Email · OpenID · atproto). Inputs are present
//! but no actual auth flow is wired — the buttons no-op until
//! `open4x-accounts` and the lobby's HTTP surface land. See the design's
//! `<Popup>` triggers — they're rendered as plain `<Trigger>` elements here
//! pending the full popup component.

use leptos::prelude::*;

use crate::components::{Btn, Trigger};

#[component]
pub fn Login(on_back: Callback<()>) -> impl IntoView {
    view! {
        <div style="flex:1; display:flex; flex-direction:column">
            <div class="row between center-y" style="padding:10px 20px; border-bottom:1px solid var(--hairline)">
                <Btn variant="bare" size="sm"
                     on_click=Callback::new(move |_| on_back.run(()))>
                    "← back"
                </Btn>
                <span class="muted xsmall" style="letter-spacing:0.08em">
                    "OPEN4X·VI / SIGN IN"
                </span>
            </div>

            <div style="width:460px; max-width:100%; margin:32px auto; padding:4px">
                <h1 class="h1" style="margin-bottom:4px">"Sign in"</h1>
                <p class="muted small" style="margin-bottom:24px">
                    "Any method below — they all link to the same "
                    <Trigger hint="An opaque, unique identifier (e.g. 0xA9C3·7F12·EE04) created the first time you sign in. All three login methods can be linked to the same player ID, so you can reach your games from any device.">
                        "player ID"
                    </Trigger>
                    "."
                </p>

                // ─── Email ──────────────────────────────────────────
                <div class="panel" style="margin-bottom:12px">
                    <div class="row between center-y" style="margin-bottom:10px">
                        <span class="h3">"Email"</span>
                        <Trigger hint="1. Enter your email. 2. Receive a one-time magic link (valid 15 min). 3. Click it — you're signed in.">
                            <span class="xsmall muted">"how it works"</span>
                        </Trigger>
                    </div>
                    <div class="field" style="margin-bottom:10px">
                        <input class="input mono" placeholder="you@example.com" />
                    </div>
                    <Btn variant="primary" class="block">"Send magic link →"</Btn>
                </div>

                // ─── OpenID ─────────────────────────────────────────
                <div class="panel" style="margin-bottom:12px">
                    <div class="row between center-y" style="margin-bottom:10px">
                        <span class="h3">"OpenID"</span>
                        <Trigger hint="Standard OpenID Connect flow. We support common providers and any custom issuer URL.">
                            <span class="xsmall muted">"about OIDC"</span>
                        </Trigger>
                    </div>
                    <div class="row wrap" style="gap:6px">
                        {["Google", "GitHub", "GitLab", "Microsoft"].iter().map(|p| view! {
                            <Btn size="sm">{*p}</Btn>
                        }).collect::<Vec<_>>()}
                        <Btn variant="ghost" size="sm" class="has-popup">"Custom OIDC…"</Btn>
                    </div>
                </div>

                // ─── atproto ────────────────────────────────────────
                <div class="panel" style="margin-bottom:16px">
                    <div class="row between center-y" style="margin-bottom:10px">
                        <span class="h3">"atproto"</span>
                        <Trigger hint="Use your handle (e.g. alice.bsky.social) or DID. We resolve your PDS and start an OAuth flow. Works with self-hosted PDSes too.">
                            <span class="xsmall muted">"about atproto"</span>
                        </Trigger>
                    </div>
                    <div class="field" style="margin-bottom:10px">
                        <input class="input mono"
                               placeholder="alice.bsky.social  or  did:plc:…" />
                    </div>
                    <Btn class="block">"Continue with atproto →"</Btn>
                </div>

                <p class="muted xsmall" style="text-align:center; margin-top:14px">
                    "New here? A player ID is created automatically on first sign-in."
                </p>
            </div>
        </div>
    }
}
