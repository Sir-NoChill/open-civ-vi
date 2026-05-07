//! Phase 1 HUD — minimal REST-driven single-player view.
//!
//! On mount this page calls `POST /api/v1/games/new` to bootstrap a game and
//! receive a bearer token; from then on every read uses
//! [`crate::components::api`] bindings against `/api/v1/*`. The End Turn
//! button posts to `/api/v1/turn/end` and bumps a refresh tick that drives the
//! `LocalResource` re-fetch. No WebSocket is involved.
//!
//! Subsequent phases will replace the placeholder map / sidebar / drawers
//! while keeping this single-source-of-truth refresh discipline.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::api;

/// Top-level Leptos component for the REST-driven HUD.
#[component]
pub fn RestGamePage() -> impl IntoView {
    // ── Session state ────────────────────────────────────────────────────────
    let token = RwSignal::new(None::<String>);
    let bootstrap_error = RwSignal::new(None::<String>);
    let action_error = RwSignal::new(None::<String>);
    let tick = RwSignal::new(0u64);

    // Bootstrap once on mount.
    Effect::new(move |_| {
        if token.get_untracked().is_some() {
            return;
        }
        spawn_local(async move {
            let body = api::games::NewGameRequest::default();
            match api::games::new(&body).await {
                Ok(resp) => token.set(Some(resp.token)),
                Err(e) => bootstrap_error.set(Some(e.to_string())),
            }
        });
    });

    // ── Resources keyed on (token, tick) ────────────────────────────────────
    let player_state = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::player_state::get(Some(&tok)).await.ok()
        }
    });

    let snapshot = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::world::snapshot(Some(&tok), None, None, Some(12))
                .await
                .ok()
        }
    });

    // ── End Turn ─────────────────────────────────────────────────────────────
    let on_end_turn = move |_| {
        let tok = token.get();
        spawn_local(async move {
            let Some(tok) = tok else { return };
            match api::turn::end(Some(&tok)).await {
                Ok(_) => tick.update(|t| *t += 1),
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    view! {
        <div class="game-layout">
            // ── Top bar ──────────────────────────────────────────────────────
            <div class="game-topbar">
                <Suspense fallback=move || view! {
                    <span class="turn-label">"Connecting…"</span>
                }>
                    {move || player_state.get().map(|wrap| match wrap.as_ref() {
                        None => view! { <span class="turn-label">"…"</span> }.into_any(),
                        Some(ps) => {
                            let gold = ps.resources.gold.value.unwrap_or(0);
                            let gpt = ps.resources.gold.per_turn;
                            let spt = ps.resources.science.per_turn;
                            let cpt = ps.resources.culture.per_turn;
                            let turn = ps.turn;
                            let turn_max = ps.turn_max;
                            let era = ps.era.clone();
                            view! {
                                <span class="civ-name">
                                    {format!("Turn {turn}/{turn_max}")}
                                </span>
                                <span class="turn-label">{format!("Era: {era}")}</span>
                                <span class="turn-label">{format!("Gold {gold} ({gpt:+})")}</span>
                                <span class="turn-label">{format!("Sci {spt:+}")}</span>
                                <span class="turn-label">{format!("Cul {cpt:+}")}</span>
                            }.into_any()
                        }
                    })}
                </Suspense>
                <div style="flex:1" />
                <button class="btn btn-primary" on:click=on_end_turn>
                    "End Turn"
                </button>
            </div>

            // ── Map placeholder (Phase 2 replaces this with a real renderer)
            <div class="hex-viewport" style="padding:1rem">
                {move || bootstrap_error.get().map(|e| view! {
                    <p style="color:#e05050">{format!("Bootstrap failed: {e}")}</p>
                })}
                {move || action_error.get().map(|e| view! {
                    <p style="color:#e0a050">{format!("Action failed: {e}")}</p>
                })}
                <Suspense fallback=move || view! { <p>"Loading map…"</p> }>
                    {move || snapshot.get().map(|wrap| match wrap.as_ref() {
                        None => view! {
                            <p style="color:#7a7f99">"No snapshot yet."</p>
                        }.into_any(),
                        Some(s) => {
                            let w = s.world.width;
                            let h = s.world.height;
                            let n = s.tiles.len();
                            view! {
                                <p style="color:#9aa">
                                    {format!("World {w} × {h} · {n} tiles in view")}
                                </p>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
