//! Ongoing-games screen — wired to `GET /api/v1/games`.
//!
//! Reads live data on mount via `LocalResource`; until the resource
//! resolves the screen shows a "Loading…" placeholder. Empty-state
//! copy nudges the user toward `+ New game`. Tile actions
//! (Resume / Notes / ⋯) are still inert beyond the Resume CTA, which
//! requires the Phase 4.3 orchestrator to populate `server_url`.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::api::games as games_api;
use crate::components::{Btn, MiniMap, Tag};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Filter {
    All,
    YourTurn,
    Waiting,
    Completed,
    Multiplayer,
}

#[component]
pub fn OngoingGames(on_new: Callback<()>) -> impl IntoView {
    let games = LocalResource::new(|| async { games_api::list().await.ok() });
    let filter = RwSignal::new(Filter::YourTurn);
    let search = RwSignal::new(String::new());

    let chip_class = move |target: Filter| -> &'static str {
        if filter.get() == target { "chip active" } else { "chip" }
    };

    view! {
        <div style="flex:1; display:flex; flex-direction:column; min-height:0">
            <div class="content-header">
                <div class="title">"Ongoing games"</div>
                <span class="crumbs">
                    {move || games
                        .get()
                        .and_then(|wrap| (*wrap).clone())
                        .map(|resp| {
                            let total = resp.games.len();
                            let yt = resp.games.iter().filter(|g| g.status == "your_turn").count();
                            format!("{total} games · {yt} awaiting you")
                        })
                        .unwrap_or_else(|| "loading…".into())}
                </span>
                <div class="actions">
                    <span class="muted xsmall">"view"</span>
                    <Btn variant="accent"
                         on_click=Callback::new(move |_| on_new.run(()))>
                        "+ New game"
                    </Btn>
                </div>
            </div>

            <div class="filter-bar">
                <span class="muted xsmall" style="padding-left:4px">"⌕"</span>
                <input
                    class="filter-search"
                    placeholder="search games and notes…"
                    prop:value=move || search.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast as _;
                        if let Some(el) = ev.target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        {
                            search.set(el.value());
                        }
                    }
                />
                <span class="sep-v"></span>
                <button
                    class=move || chip_class(Filter::YourTurn)
                    on:click=move |_| filter.set(Filter::YourTurn)
                >
                    "your turn"
                    {move || (filter.get() == Filter::YourTurn).then(|| view! { " " <span class="x">"×"</span> })}
                </button>
                <button class=move || chip_class(Filter::Waiting)
                        on:click=move |_| filter.set(Filter::Waiting)>"waiting"</button>
                <button class=move || chip_class(Filter::Completed)
                        on:click=move |_| filter.set(Filter::Completed)>"completed"</button>
                <button class=move || chip_class(Filter::Multiplayer)
                        on:click=move |_| filter.set(Filter::Multiplayer)>"multiplayer"</button>
                <button class=move || chip_class(Filter::All)
                        on:click=move |_| filter.set(Filter::All)>"all"</button>
                <span style="margin-left:auto" class="muted xsmall">"sort"</span>
                <button class="chip">"recent ↓"</button>
            </div>

            <div style="flex:1; overflow:auto">
                <Suspense fallback=move || view! { <p class="muted xsmall" style="padding:16px">"Loading…"</p> }>
                    {move || games.get().map(|wrap| {
                        let Some(resp) = (*wrap).clone() else {
                            return view! {
                                <p class="muted xsmall" style="padding:16px">
                                    "Couldn't load games — try refreshing."
                                </p>
                            }.into_any();
                        };
                        if resp.games.is_empty() {
                            return view! {
                                <div style="padding:24px; text-align:center; color:var(--dim)">
                                    <p class="small" style="margin-bottom:12px">
                                        "No games yet. Click "
                                        <strong>"+ New game"</strong>
                                        " above to start your first."
                                    </p>
                                </div>
                            }.into_any();
                        }
                        let f = filter.get();
                        let q = search.get().to_lowercase();
                        let filtered: Vec<_> = resp.games.into_iter()
                            .filter(|g| match f {
                                Filter::All => true,
                                Filter::YourTurn => g.status == "your_turn",
                                Filter::Waiting => g.status == "waiting",
                                Filter::Completed => g.status == "completed",
                                Filter::Multiplayer => g.players_human > 1,
                            })
                            .filter(|g| {
                                if q.is_empty() { return true; }
                                g.name.to_lowercase().contains(&q)
                                    || g.leader.to_lowercase().contains(&q)
                                    || g.civ_id.to_lowercase().contains(&q)
                            })
                            .collect();
                        if filtered.is_empty() {
                            return view! {
                                <div style="padding:24px; text-align:center; color:var(--dim)">
                                    <p class="small">"No games match the current filter."</p>
                                </div>
                            }.into_any();
                        }
                        let rows = filtered.into_iter().enumerate().map(|(i, g)| {
                            let is_yours = g.status == "your_turn";
                            let tile_class = if is_yours { "game-tile your-turn" } else { "game-tile" };
                            let resume_label = match g.status.as_str() {
                                "your_turn" => "→ Resume",
                                "waiting" => "Open",
                                "completed" => "Review",
                                _ => "Open",
                            };
                            let resume_variant = if is_yours { "accent" } else { "primary" };
                            let game_id_for_resume = g.game_id.clone();
                            let resume_disabled = g.server_url.is_empty();
                            let on_resume = Callback::new(move |_: ()| {
                                let id = game_id_for_resume.clone();
                                spawn_local(async move {
                                    let Ok(resp) = games_api::resume(&id).await else { return };
                                    if let Some(win) = web_sys::window() {
                                        let target = format!(
                                            "{}/?token={}",
                                            resp.url.trim_end_matches('/'),
                                            resp.token,
                                        );
                                        let _ = win.location().set_href(&target);
                                    }
                                });
                            });
                            let disabled_signal = Signal::derive(move || resume_disabled);
                            let players = format!("{}H · {}AI", g.players_human, g.players_ai);
                            // Seed the MiniMap from the actual world seed when
                            // possible; fall back to row index otherwise so
                            // every tile still gets unique decoration.
                            let seed = g
                                .seed
                                .bytes()
                                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
                                .max(1)
                                .saturating_add(i as u64);
                            view! {
                                <div class=tile_class style="width:100%">
                                    <div class="tile-head">
                                        <div>
                                            <div class="tile-name">{g.name.clone()}</div>
                                            <div class="leader">{format!("{} · {}", g.leader, g.civ_id)}</div>
                                        </div>
                                        <div class="row gap-xs">
                                            {(g.players_human > 1).then(|| view! { <Tag>"MP"</Tag> })}
                                        </div>
                                    </div>
                                    <div class="map-thumb">
                                        <MiniMap seed=seed style="position:absolute; inset:0; width:100%; height:100%" />
                                    </div>
                                    <div class="stats">
                                        <div class="row-stat"><span class="k">"turn"</span><span class="v">{g.turn.to_string()}</span></div>
                                        <div class="row-stat"><span class="k">"era"</span><span class="v">{g.era.clone()}</span></div>
                                        <div class="row-stat"><span class="k">"diff"</span><span class="v">{g.difficulty.clone()}</span></div>
                                        <div class="row-stat"><span class="k">"score"</span><span class="v">{g.score.to_string()}</span></div>
                                        <div class="row-stat"><span class="k">"players"</span><span class="v">{players}</span></div>
                                        <div class="row-stat"><span class="k">"last"</span><span class="v">{g.last_played_at.clone().unwrap_or_else(|| "—".into())}</span></div>
                                    </div>
                                    <div class="actions">
                                        <Btn variant="ghost" size="sm">"📝 Notes"</Btn>
                                        <span style="flex:1"></span>
                                        <Btn
                                            variant=resume_variant
                                            size="sm"
                                            disabled=disabled_signal
                                            on_click=on_resume
                                        >{resume_label}</Btn>
                                        <Btn variant="ghost" size="sm">"···"</Btn>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>();
                        view! { <div class="games-grid">{rows}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}
