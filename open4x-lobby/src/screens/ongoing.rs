//! Ongoing-games screen — wired to `GET /api/v1/games`.
//!
//! Reads live data on mount via `LocalResource`; until the resource
//! resolves the screen shows a "Loading…" placeholder. Empty-state
//! copy nudges the user toward `+ New game`. Tile actions
//! (Resume / Notes / ⋯) are still inert beyond the Resume CTA, which
//! requires the Phase 4.3 orchestrator to populate `server_url`.

use std::sync::Arc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::api::games as games_api;
use crate::components::{
    Btn, MiniMap, Popup, PopupList, PopupListItem, PopupSize, PopupTrigger, Tag,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Filter {
    All,
    YourTurn,
    Waiting,
    Completed,
    Multiplayer,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Sort {
    /// `last_played_at` desc, falling back to `created_at` desc.
    Recent,
    /// `created_at` asc.
    Oldest,
    /// `score` desc, then turn desc.
    Score,
    /// `turn` desc.
    Turn,
}

impl Sort {
    fn label(self) -> &'static str {
        match self {
            Sort::Recent => "recent ↓",
            Sort::Oldest => "oldest",
            Sort::Score => "score ↓",
            Sort::Turn => "turn ↓",
        }
    }
}

#[component]
pub fn OngoingGames(on_new: Callback<()>) -> impl IntoView {
    let tick = RwSignal::new(0u32);
    let games = LocalResource::new(move || {
        let _ = tick.get();
        async move { games_api::list().await.ok() }
    });
    let filter = RwSignal::new(Filter::YourTurn);
    let search = RwSignal::new(String::new());
    let sort = RwSignal::new(Sort::Recent);

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
                <Popup
                    title="Sort"
                    size=PopupSize::Narrow
                    trigger=PopupTrigger::Click
                    content=Arc::new(move || view! {
                        <div class="popup-list">
                            {[Sort::Recent, Sort::Oldest, Sort::Score, Sort::Turn].iter().copied().map(|s| {
                                let active = sort.get() == s;
                                view! {
                                    <button
                                        class="item"
                                        type="button"
                                        on:click=move |_| sort.set(s)
                                    >
                                        <span class="icon">{if active { "✓" } else { " " }}</span>
                                        <span>{s.label()}</span>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any())
                >
                    <button class="chip">{move || sort.get().label()}</button>
                </Popup>
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
                        let s = sort.get();
                        let mut filtered: Vec<_> = resp.games.into_iter()
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
                        match s {
                            Sort::Recent => filtered.sort_by(|a, b| {
                                let key = |g: &games_api::GameView| {
                                    g.last_played_at
                                        .clone()
                                        .unwrap_or_else(|| g.created_at.clone())
                                };
                                key(b).cmp(&key(a))
                            }),
                            Sort::Oldest => filtered.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                            Sort::Score => filtered.sort_by(|a, b| {
                                b.score.cmp(&a.score).then(b.turn.cmp(&a.turn))
                            }),
                            Sort::Turn => filtered.sort_by(|a, b| b.turn.cmp(&a.turn)),
                        }
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
                                        {tile_menu_popup(g.game_id.clone(), g.name.clone(), tick)}
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

/// Click-trigger popup wrapping a tile's "···" button.
///
/// Items: View summary (TODO popup-in-popup) / Copy game ID
/// (writes to navigator.clipboard) / Share invite link (TODO) /
/// Archive (TODO — needs a status mutation route) / Resign
/// (DELETE /games/{id} + bump tick).
///
/// Resign and Copy game ID are wired today; Archive / View summary
/// / Share invite are visible-but-inert pending their underlying
/// routes / popups.
fn tile_menu_popup(
    game_id: String,
    game_name: String,
    tick: RwSignal<u32>,
) -> impl IntoView {
    let id_for_resign = game_id.clone();
    let id_for_clipboard = game_id.clone();
    let _ = game_name;

    // Build the menu items inside an Arc'd renderer because the
    // PopupList items embed click closures that capture game_id.
    let content = Arc::new(move || {
        let id_resign = id_for_resign.clone();
        let id_copy = id_for_clipboard.clone();

        // PopupList today renders inert <button class="item"> rows
        // (no on_click plumbing — that lands when the menu rows
        // gain an interactive callback). For now we fan out to
        // bespoke <button> rows for the two interactive items, and
        // use PopupList for the inert ones.
        view! {
            <div class="popup-list">
                <button
                    class="item"
                    type="button"
                    on:click=move |_| {
                        let id = id_copy.clone();
                        if let Some(win) = web_sys::window() {
                            let nav = win.navigator();
                            let _ = nav.clipboard().write_text(&id);
                        }
                    }
                >
                    <span class="icon">"⎘"</span>
                    <span>"Copy game ID"</span>
                </button>
                <button class="item" type="button">
                    <span class="icon">"◑"</span>
                    <span>"View summary"</span>
                    <span class="desc">"(TODO)"</span>
                </button>
                <button class="item" type="button">
                    <span class="icon">"↗"</span>
                    <span>"Share invite link"</span>
                    <span class="desc">"(TODO)"</span>
                </button>
                <div class="sep"></div>
                <button class="item" type="button">
                    <span class="icon">"⊟"</span>
                    <span>"Archive"</span>
                    <span class="desc">"(TODO)"</span>
                </button>
                <button
                    class="item"
                    type="button"
                    on:click=move |_| {
                        let id = id_resign.clone();
                        spawn_local(async move {
                            let _ = games_api::delete_game(&id).await;
                            tick.update(|t| *t += 1);
                        });
                    }
                >
                    <span class="icon">"⊗"</span>
                    <span>"Resign / delete"</span>
                </button>
            </div>
        }
        .into_any()
    });

    // Suppress dead_code on PopupList — it's still used elsewhere
    // (StepPlayers slot menu).
    let _ = (
        PopupListItem::sep,
        PopupList,
    );

    view! {
        <Popup
            title="Game"
            size=PopupSize::Narrow
            trigger=PopupTrigger::Click
            content=content
        >
            <Btn variant="ghost" size="sm">"···"</Btn>
        </Popup>
    }
}
