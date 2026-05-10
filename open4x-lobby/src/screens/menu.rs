//! Menu shell — port of `docs/open4x-landing/project/hifi/menu.jsx`'s
//! `Shell` component: 240 px sidebar with user-card, primary nav (Ongoing /
//! New game / Profile), secondary nav (Friends / Presets / Docs as TODO
//! popups), versioned footer with online dot.

use std::sync::Arc;

use leptos::prelude::*;

use crate::components::{Popup, PopupBody, PopupSize};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MenuTab {
    Ongoing,
    NewGame,
    Profile,
}

impl MenuTab {
    pub fn label(self) -> &'static str {
        match self {
            MenuTab::Ongoing => "Ongoing games",
            MenuTab::NewGame => "New game",
            MenuTab::Profile => "Profile",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            MenuTab::Ongoing => "▣",
            MenuTab::NewGame => "+",
            MenuTab::Profile => "◔",
        }
    }
}

const SECONDARY: &[(&str, &str)] = &[("◎", "Friends"), ("≡", "Presets"), ("?", "Docs")];

#[component]
pub fn MenuShell(tab: RwSignal<MenuTab>, children: Children) -> impl IntoView {
    view! {
        <div class="menu-shell">
            <aside class="sidebar">
                <div class="user-card">
                    <div class="avatar-sm">"A"</div>
                    <div style="min-width:0">
                        <div class="name">"Alice"</div>
                        <div class="uid">"0xA9C3·7F12"</div>
                    </div>
                    <span class="chev">"▾"</span>
                </div>

                <div class="group-label">"PLAY"</div>
                {[MenuTab::Ongoing, MenuTab::NewGame, MenuTab::Profile].iter().copied().map(|t| {
                    let badge = matches!(t, MenuTab::Ongoing).then_some(3u32);
                    view! {
                        <button
                            class="nav-item"
                            aria-current=move || (tab.get() == t).to_string()
                            on:click=move |_| tab.set(t)
                        >
                            <span class="icon">{t.icon()}</span>
                            <span>{t.label()}</span>
                            {badge.map(|n| view! { <span class="badge">{n.to_string()}</span> })}
                        </button>
                    }
                }).collect::<Vec<_>>()}

                <div class="group-label">"MORE"</div>
                {SECONDARY.iter().copied().map(|(icon, label)| view! {
                    <button class="nav-item" style="width:100%" title="// out of scope for this hi-fi pass">
                        <span class="icon">{icon}</span>
                        <span>{label}</span>
                    </button>
                }).collect::<Vec<_>>()}

                <div class="footer">
                    <span>"v0.1.0"</span>
                    <Popup
                        title="Server status"
                        size=PopupSize::Narrow
                        content=Arc::new(|| view! {
                            <PopupBody>
                                <div class="kv xsmall">
                                    <span class="k">"api"</span><span style="color:var(--good)">"● operational"</span>
                                    <span class="k">"latency"</span><span>"42 ms"</span>
                                    <span class="k">"region"</span><span>"auto"</span>
                                </div>
                            </PopupBody>
                        }.into_any())
                    >
                        <span class="trigger" style="color:var(--good)">"● online"</span>
                    </Popup>
                </div>
            </aside>
            <div class="content">{children()}</div>
        </div>
    }
}
