//! Profile screen — wired to `GET /api/v1/me` and `PATCH /api/v1/me`.
//!
//! On mount fetches the authenticated profile via
//! `components::api::me::get` and seeds the form fields. Each field /
//! preference is a local `RwSignal`; the "Save profile" button posts
//! the diff back through `me::patch`. Linked-identities list reads
//! straight from `MeView.identities` and `+ link another` / `unlink`
//! buttons are still TODO behind their respective HTTP routes.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::api::me as me_api;
use crate::components::{Btn, Panel, Segmented, segmented::Segment, Toggle};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum SaveState {
    #[default]
    Idle,
    Saving,
    Saved,
    Error(String),
}

#[component]
pub fn Profile() -> impl IntoView {
    let me = LocalResource::new(|| async { me_api::get().await.ok() });

    let preferred_name = RwSignal::new(String::new());
    let pronouns = RwSignal::new(String::new());
    let bio = RwSignal::new(String::new());
    let density = RwSignal::new("comfortable".to_string());
    let scheme = RwSignal::new("paper".to_string());
    let kbd = RwSignal::new(true);
    let notifs = RwSignal::new(true);
    let discoverable = RwSignal::new(true);
    let player_id_text = RwSignal::new(String::from("—"));
    let identities = RwSignal::new(Vec::<me_api::IdentityView>::new());
    let save_state = RwSignal::new(SaveState::Idle);

    // Seed signals once the resource arrives.
    Effect::new(move |_| {
        let Some(wrap) = me.get() else { return };
        let Some(view) = (*wrap).clone() else { return };
        preferred_name.set(view.preferred_name);
        pronouns.set(view.pronouns);
        bio.set(view.bio);
        density.set(view.prefs.density);
        scheme.set(view.prefs.color_scheme);
        kbd.set(view.prefs.keyboard_nav);
        notifs.set(view.prefs.turn_notifications);
        discoverable.set(view.prefs.discoverable_by_id);
        player_id_text.set(view.player_id);
        identities.set(view.identities);
    });

    let density_opts = Signal::derive(|| {
        ["compact", "comfortable", "spacious"]
            .iter()
            .map(|s| Segment::from_str(s))
            .collect()
    });
    let scheme_opts = Signal::derive(|| {
        ["paper", "ink", "auto"]
            .iter()
            .map(|s| Segment::from_str(s))
            .collect()
    });

    let on_save = move |_| {
        save_state.set(SaveState::Saving);
        let body = me_api::PatchMeBody {
            preferred_name: Some(preferred_name.get_untracked()),
            pronouns: Some(pronouns.get_untracked()),
            bio: Some(bio.get_untracked()),
            prefs: Some(me_api::Preferences {
                density: density.get_untracked(),
                color_scheme: scheme.get_untracked(),
                keyboard_nav: kbd.get_untracked(),
                turn_notifications: notifs.get_untracked(),
                discoverable_by_id: discoverable.get_untracked(),
            }),
        };
        spawn_local(async move {
            match me_api::patch(body).await {
                Ok(_) => save_state.set(SaveState::Saved),
                Err(e) => save_state.set(SaveState::Error(e.to_string())),
            }
        });
    };

    let saving = Signal::derive(move || matches!(save_state.get(), SaveState::Saving));

    view! {
        <div style="flex:1; overflow:auto">
            <div class="content-header">
                <div class="title">"Profile & settings"</div>
                <span class="crumbs">
                    "player_id: "
                    <code class="trigger" style="font-family:var(--font-mono)"
                          title="Your unique identifier on the platform.">
                        {move || player_id_text.get()}
                    </code>
                </span>
            </div>

            <div class="profile-grid">
                <Panel>
                    <div class="col" style="align-items:center; gap:12px">
                        <div class="avatar">
                            {move || preferred_name
                                .get()
                                .chars()
                                .next()
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "?".into())}
                        </div>
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
                        <div class="row between center-y" style="margin-bottom:14px">
                            <div class="h3">"Profile"</div>
                            <div class="row gap-sm center-y">
                                {move || match save_state.get() {
                                    SaveState::Saved => view! {
                                        <span class="xsmall" style="color:var(--good)">"saved ✓"</span>
                                    }.into_any(),
                                    SaveState::Error(msg) => view! {
                                        <span class="xsmall" style="color:var(--accent)">{msg}</span>
                                    }.into_any(),
                                    _ => view! { <span /> }.into_any(),
                                }}
                                <Btn
                                    variant="primary"
                                    size="sm"
                                    disabled=saving
                                    on_click=Callback::new(on_save)
                                >
                                    {move || if saving.get() { "saving…" } else { "save" }}
                                </Btn>
                            </div>
                        </div>
                        <div class="field">
                            <label>"Preferred name"</label>
                            <input
                                class="input"
                                prop:value=move || preferred_name.get()
                                on:input=move |ev| {
                                    use wasm_bindgen::JsCast as _;
                                    if let Some(el) = ev.target()
                                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                    {
                                        preferred_name.set(el.value());
                                    }
                                }
                            />
                            <span class="hint">"Shown to other players in invites & chat."</span>
                        </div>
                        <div class="field">
                            <label>"Pronouns (optional)"</label>
                            <input
                                class="input"
                                prop:value=move || pronouns.get()
                                on:input=move |ev| {
                                    use wasm_bindgen::JsCast as _;
                                    if let Some(el) = ev.target()
                                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                    {
                                        pronouns.set(el.value());
                                    }
                                }
                            />
                        </div>
                        <div class="field">
                            <label>"Bio"</label>
                            <textarea
                                class="input"
                                rows="2"
                                prop:value=move || bio.get()
                                on:input=move |ev| {
                                    use wasm_bindgen::JsCast as _;
                                    if let Some(el) = ev.target()
                                        .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                                    {
                                        bio.set(el.value());
                                    }
                                }
                            ></textarea>
                            <span class="hint">"Markdown supported. Visible on invite cards."</span>
                        </div>
                    </Panel>

                    <Panel>
                        <div class="row between center-y" style="margin-bottom:12px">
                            <div class="h3">"Linked identities"</div>
                            <Btn variant="ghost" size="sm">"+ link another"</Btn>
                        </div>

                        <Suspense fallback=move || view! {
                            <p class="muted xsmall">"Loading…"</p>
                        }>
                            {move || identities.get().is_empty().then(|| view! {
                                <p class="muted xsmall">"No identities linked yet."</p>
                            })}
                            {move || identities.get().into_iter().enumerate().map(|(i, id)| {
                                let is_primary_email = id.kind == "email"
                                    && id.primary.unwrap_or(false);
                                let kind_label = match id.kind.as_str() {
                                    "email"   => if is_primary_email { "EMAIL · primary".into() } else { "EMAIL".into() },
                                    "oidc"    => "OPENID".into(),
                                    "atproto" => "ATPROTO".into(),
                                    other     => other.to_uppercase(),
                                };
                                let row_class = if is_primary_email { "id-row primary" } else { "id-row" };
                                let action = if is_primary_email { "manage" } else { "unlink" };
                                view! {
                                    <div class=row_class data-i=i>
                                        <span class="id-type">{kind_label}</span>
                                        <span class="id-val">{id.label.clone()}</span>
                                        <Btn variant="bare" size="sm">{action}</Btn>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </Suspense>

                        <p class="muted xsmall" style="margin-top:10px">
                            "All identities map to the same player. "
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
                                <Toggle on=kbd on_change=Callback::new(move |v| kbd.set(v)) />
                            </div>
                            <div class="value muted xsmall">"vim bindings"</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Turn notifications"</div>
                            <div class="control">
                                <Toggle on=notifs on_change=Callback::new(move |v| notifs.set(v)) />
                            </div>
                            <div class="value muted xsmall">"email + push"</div>
                        </div>

                        <div class="param-row">
                            <div class="label">"Discoverable by ID"</div>
                            <div class="control">
                                <Toggle on=discoverable on_change=Callback::new(move |v| discoverable.set(v)) />
                            </div>
                            <div class="value muted xsmall">"others can invite"</div>
                        </div>
                    </Panel>
                </div>
            </div>
        </div>
    }
}
