//! Login screen — port of `docs/open4x-landing/project/hifi/login.jsx`.
//!
//! Three stacked auth panels (Email · OpenID · atproto). Inputs are
//! present but no actual auth flow is wired — the buttons no-op until
//! `open4x-accounts` and the lobby's HTTP surface land (Phases 2-3 in
//! `book/src/roadmap/accounts-and-login.md`). The underlined help
//! triggers are now real `<Popup>` wrappers.

use std::sync::Arc;

use leptos::prelude::*;

use crate::components::{Btn, Popup, PopupBody, PopupSize};

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
                    <Popup
                        title="player ID"
                        content=Arc::new(|| view! {
                            <PopupBody>
                                <p>"An opaque, unique identifier (e.g. "
                                    <code>"0xA9C3·7F12·EE04"</code>
                                    ") created the first time you sign in."
                                </p>
                                <p>"All three login methods can be linked to the same player ID, so you can reach your games from any device."</p>
                                <p class="muted xsmall">"There are no usernames. Friends find you by your linked email, OpenID URL, or atproto handle."</p>
                            </PopupBody>
                        }.into_any())
                    >
                        <span class="trigger">"player ID"</span>
                    </Popup>
                    "."
                </p>

                // ─── Email ──────────────────────────────────────────
                <div class="panel" style="margin-bottom:12px">
                    <div class="row between center-y" style="margin-bottom:10px">
                        <span class="h3">"Email"</span>
                        <Popup
                            title="How it works"
                            size=PopupSize::Narrow
                            content=Arc::new(|| view! {
                                <PopupBody>
                                    <p>"1. Enter your email."</p>
                                    <p>"2. Receive a one-time magic link (valid 15 min)."</p>
                                    <p>"3. Click it — you're signed in."</p>
                                </PopupBody>
                            }.into_any())
                        >
                            <span class="trigger xsmall muted">"how it works"</span>
                        </Popup>
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
                        <Popup
                            title="OIDC"
                            content=Arc::new(|| view! {
                                <PopupBody>
                                    <p>"Standard OpenID Connect flow. We support common providers and any custom issuer URL."</p>
                                    <p class="muted xsmall">"Tokens are stored client-side; the server only sees signed claims."</p>
                                </PopupBody>
                            }.into_any())
                        >
                            <span class="trigger xsmall muted">"about OIDC"</span>
                        </Popup>
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
                        <Popup
                            title="atproto"
                            content=Arc::new(|| view! {
                                <PopupBody>
                                    <p>"Use your handle (e.g. "
                                        <code>"alice.bsky.social"</code>
                                        ") or DID. We resolve your PDS and start an OAuth flow."
                                    </p>
                                    <p class="muted xsmall">"Works with self-hosted PDSes too."</p>
                                </PopupBody>
                            }.into_any())
                        >
                            <span class="trigger xsmall muted">"about atproto"</span>
                        </Popup>
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
