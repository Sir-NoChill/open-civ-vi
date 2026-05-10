//! Landing screen — port of `docs/open4x-landing/project/hifi/landing.jsx`.
//!
//! Centred ASCII banner, serif display headline with caret, blurb, two CTAs,
//! triple of underlined auth-method triggers, footer.
//!
//! `on_signin` switches the app route to [`crate::app::Screen::Login`].
//! `View source` is an external link to the planned repo URL — kept inert
//! for now so we don't ship a placeholder URL into production.

use std::sync::Arc;

use leptos::prelude::*;

use crate::components::{Btn, Popup, PopupBody, PopupSize};

const ASCII_BANNER: &str = "┌─┐┌─┐┌─┐┌┐┌    ╦ ╦ ╦
│ │├─┘├┤ │││    ║ ║ ║
└─┘┴  └─┘┘└┘    ╩ ╩ ╩
       open  4x  vi";

#[component]
pub fn Landing(on_signin: Callback<()>) -> impl IntoView {
    view! {
        <div style="flex:1; display:flex; flex-direction:column; padding:40px 32px; position:relative">
            <div style="max-width:680px; margin:auto; text-align:center; padding-bottom:60px">
                <div style="font-size:var(--fs-xs); color:var(--dim); letter-spacing:0.3em; margin-bottom:32px">
                    "V0.1.0 · PRE-ALPHA · OPEN SOURCE"
                </div>

                <pre style="font-family:var(--font-mono); font-weight:500; \
                            font-size:13px; line-height:1.15; color:var(--ink); \
                            margin:0 0 28px; letter-spacing:0">
                    {ASCII_BANNER}
                </pre>

                <h1 class="h-display" style="margin-bottom:18px">
                    "A 4X game without"
                    <br/>
                    "the graphics tax"
                    <span class="caret"></span>
                </h1>

                <p class="muted" style="max-width:480px; margin:0 auto 28px; \
                                        font-size:var(--fs-md); line-height:1.65">
                    "[ blurb placeholder — civ-vi inspired, deeply moddable, runs \
                     on a potato. author will write this. keep it short. keep it \
                     sharp. ]"
                </p>

                <div class="row center-x" style="gap:10px; margin-bottom:36px">
                    <Btn variant="accent" size="lg"
                         on_click=Callback::new(move |_| on_signin.run(()))>
                        "Sign in & play →"
                    </Btn>
                    <Btn variant="ghost" size="lg" class="has-popup">
                        "View source"
                    </Btn>
                </div>

                <div class="muted" style="font-size:var(--fs-xs); letter-spacing:0.16em">
                    <Popup
                        title="Email"
                        size=PopupSize::Narrow
                        content=Arc::new(|| view! {
                            <PopupBody>
                                <p>"Magic-link login. We mail a one-time URL valid for 15 minutes."</p>
                            </PopupBody>
                        }.into_any())
                    >
                        <span class="trigger">"EMAIL"</span>
                    </Popup>
                    " · "
                    <Popup
                        title="OpenID Connect"
                        content=Arc::new(|| view! {
                            <PopupBody>
                                <p>"Sign in with Google, GitHub, GitLab, Microsoft, or any custom OIDC issuer URL."</p>
                            </PopupBody>
                        }.into_any())
                    >
                        <span class="trigger">"OPENID"</span>
                    </Popup>
                    " · "
                    <Popup
                        title="atproto"
                        content=Arc::new(|| view! {
                            <PopupBody>
                                <p>"Use your atproto handle (e.g. "
                                    <code>"alice.bsky.social"</code>
                                    ") or a DID. OAuth flow with your PDS."
                                </p>
                            </PopupBody>
                        }.into_any())
                    >
                        <span class="trigger">"ATPROTO"</span>
                    </Popup>
                </div>
            </div>

            <div style="position:absolute; bottom:18px; left:0; right:0; \
                        display:flex; justify-content:space-between; \
                        padding:0 32px; font-size:var(--fs-xs); color:var(--dim)">
                <span>"open4x.org"</span>
                <span>"// hover any underlined word"</span>
                <span>"self-hostable · API-driven"</span>
            </div>
        </div>
    }
}
