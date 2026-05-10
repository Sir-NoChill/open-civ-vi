//! Top-level Leptos app component for the lobby SPA.
//!
//! Renders the `app-bar` chrome (brand · HI-FI pill · screen tabs · status
//! dot · `?` kbd hint) and a single `RwSignal<Screen>` that drives which
//! screen mounts below. The "Sign in & play →" CTA on the landing screen
//! switches to [`Screen::Login`]; navigating to `Screen::Menu` drops the
//! user into the [`crate::screens::MenuShell`] with a default tab.

use leptos::prelude::*;

use crate::screens::{Landing, Login, MenuShell, MenuTab, NewGame, OngoingGames, Profile};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Landing,
    Login,
    Menu,
}

impl Screen {
    fn label(self) -> &'static str {
        match self {
            Screen::Landing => "01 Landing",
            Screen::Login => "02 Login",
            Screen::Menu => "03 Menu",
        }
    }

    const ALL: &'static [Screen] = &[Screen::Landing, Screen::Login, Screen::Menu];
}

#[component]
pub fn App() -> impl IntoView {
    let screen = RwSignal::new(Screen::Landing);
    let menu_tab = RwSignal::new(MenuTab::Ongoing);

    // Callbacks reused across screens.
    let go_login = Callback::new(move |_: ()| screen.set(Screen::Login));
    let go_landing = Callback::new(move |_: ()| screen.set(Screen::Landing));
    let go_newgame = Callback::new(move |_: ()| {
        screen.set(Screen::Menu);
        menu_tab.set(MenuTab::NewGame);
    });

    view! {
        <div class="app" data-density="comfortable">
            // ── App bar ─────────────────────────────────────────────────
            <header class="app-bar">
                <div class="brand">
                    <span class="glyph">"⌬"</span>
                    " OPEN4X·VI"
                </div>
                <span class="pill">"HI-FI"</span>
                <nav>
                    {Screen::ALL.iter().copied().map(|s| view! {
                        <button
                            aria-current=move || (screen.get() == s).to_string()
                            on:click=move |_| screen.set(s)
                        >{s.label()}</button>
                    }).collect::<Vec<_>>()}
                </nav>
                <div class="right">
                    <span><span class="dot"></span>" open4x.dev"</span>
                    <span class="kbd">"?"</span>
                </div>
            </header>

            // ── Screen body ─────────────────────────────────────────────
            <div class="screen">
                {move || match screen.get() {
                    Screen::Landing => view! {
                        <Landing on_signin=go_login />
                    }.into_any(),
                    Screen::Login => view! {
                        <Login on_back=go_landing />
                    }.into_any(),
                    Screen::Menu => view! {
                        <MenuShell tab=menu_tab>
                            {move || match menu_tab.get() {
                                MenuTab::Ongoing => view! {
                                    <OngoingGames on_new=go_newgame />
                                }.into_any(),
                                MenuTab::NewGame => view! { <NewGame /> }.into_any(),
                                MenuTab::Profile => view! { <Profile /> }.into_any(),
                            }}
                        </MenuShell>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
