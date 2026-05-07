//! REST-driven single-player view.
//!
//! On mount this page calls `POST /api/v1/games/new` to bootstrap a game and
//! receive a bearer token; from then on every read uses
//! [`crate::components::api`] bindings against `/api/v1/*`. The End Turn
//! button posts to `/api/v1/turn/end` and bumps a refresh tick that drives
//! every `LocalResource` to refetch in parallel. No WebSocket is involved.
//!
//! Intentionally compact — this is the first end-to-end demo of the REST
//! pipeline, not a full HUD. The HexMap rebind, real tabs (city/units/tech/
//! diplomacy/empire/victory), and drawers are tracked on the roadmap.

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

    let cities = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::cities::list(Some(&tok)).await.ok()
        }
    });

    let units = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::units::list(Some(&tok)).await.ok()
        }
    });

    let tech = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::tech::tech(Some(&tok)).await.ok()
        }
    });

    let turn_queue = LocalResource::new(move || {
        let _ = tick.get();
        let tok = token.get();
        async move {
            let tok = tok?;
            api::notifications::turn_queue(Some(&tok)).await.ok()
        }
    });

    // ── End Turn ─────────────────────────────────────────────────────────────
    let on_end_turn = move |_| {
        let tok = token.get();
        action_error.set(None);
        spawn_local(async move {
            let Some(tok) = tok else { return };
            match api::turn::end(Some(&tok)).await {
                Ok(_) => tick.update(|t| *t += 1),
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    // Auto-pick first available tech when current research queue is empty.
    let on_pick_research = move |_| {
        let tok = token.get();
        action_error.set(None);
        spawn_local(async move {
            let Some(tok) = tok else { return };
            let Ok(tt) = api::tech::tech(Some(&tok)).await else { return };
            let Some(first) = tt.techs.iter().find(|t| t.status == "available") else { return };
            let body = api::tech::TechResearchBody { tech_id: first.id.clone() };
            match api::tech::research_tech(Some(&tok), &body).await {
                Ok(_) => tick.update(|t| *t += 1),
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    view! {
        <div class="game-layout">
            // ── Top bar (player state) ───────────────────────────────────────
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
                <button class="btn btn-ghost" on:click=on_pick_research>
                    "Pick Research"
                </button>
                <button class="btn btn-primary" on:click=on_end_turn>
                    "End Turn"
                </button>
            </div>

            // ── Main two-column body ─────────────────────────────────────────
            <div class="game-main">
                // Left: world summary (placeholder for HexMap)
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

                // Right: data sidebar
                <div class="sidebar">
                    <h3>"Cities"</h3>
                    <Suspense fallback=move || view! { <p class="no-selection">"…"</p> }>
                        {move || cities.get().map(|wrap| match wrap.as_ref() {
                            None => view! { <p class="no-selection">"–"</p> }.into_any(),
                            Some(c) if c.cities.is_empty() => {
                                view! { <p class="no-selection">"None"</p> }.into_any()
                            }
                            Some(c) => {
                                let rows = c.cities.iter().map(|city| {
                                    let label = if city.is_own {
                                        format!("{} (pop {})", city.name, city.population)
                                    } else {
                                        format!("{} · {} (foreign)", city.name, city.owner)
                                    };
                                    view! { <div class="info-row"><span>{label}</span></div> }
                                }).collect::<Vec<_>>();
                                view! { <div>{rows}</div> }.into_any()
                            }
                        })}
                    </Suspense>

                    <h3>"Units"</h3>
                    <Suspense fallback=move || view! { <p class="no-selection">"…"</p> }>
                        {move || units.get().map(|wrap| match wrap.as_ref() {
                            None => view! { <p class="no-selection">"–"</p> }.into_any(),
                            Some(u) => {
                                let own = u.units.iter().filter(|x| x.is_own).count();
                                let total = u.units.len();
                                view! {
                                    <p class="no-selection">
                                        {format!("{own} own · {total} visible")}
                                    </p>
                                }.into_any()
                            }
                        })}
                    </Suspense>

                    <h3>"Research"</h3>
                    <Suspense fallback=move || view! { <p class="no-selection">"…"</p> }>
                        {move || tech.get().map(|wrap| match wrap.as_ref() {
                            None => view! { <p class="no-selection">"–"</p> }.into_any(),
                            Some(tt) => {
                                let current = tt.techs.iter().find(|t| t.status == "current");
                                let label = match current {
                                    Some(t) => {
                                        let prog = t.progress.unwrap_or(0);
                                        format!("{} {}/{}", t.name, prog, t.cost)
                                    }
                                    None => "(none queued)".into(),
                                };
                                let avail = tt.techs.iter().filter(|t| t.status == "available").count();
                                view! {
                                    <p class="no-selection">{label}</p>
                                    <p class="no-selection">{format!("{avail} available")}</p>
                                }.into_any()
                            }
                        })}
                    </Suspense>

                    <h3>"Turn Queue"</h3>
                    <Suspense fallback=move || view! { <p class="no-selection">"…"</p> }>
                        {move || turn_queue.get().map(|wrap| match wrap.as_ref() {
                            None => view! { <p class="no-selection">"–"</p> }.into_any(),
                            Some(q) => {
                                let req = q.items.iter().filter(|i| i.required).count();
                                let opt = q.items.iter().filter(|i| !i.required).count();
                                view! {
                                    <p class="no-selection" style:color=move || {
                                        if req > 0 { "#e0a050".to_string() }
                                        else { "#7a7f99".to_string() }
                                    }>
                                        {format!("{req} required · {opt} optional")}
                                    </p>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
