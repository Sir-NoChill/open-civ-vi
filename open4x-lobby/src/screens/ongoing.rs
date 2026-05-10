//! Ongoing-games screen — scaffold port of `hifi/menu.jsx::OngoingGames`.
//!
//! Renders the content header, filter bar, and tile grid using the design's
//! sample data baked in. Real data will come from the lobby's REST surface
//! once `open4x-accounts` and a games-index endpoint exist (see roadmap).
//! Tile actions (Resume / Notes / ⋯ menu) are inert for now.

use leptos::prelude::*;

use crate::components::{Btn, MiniMap, Tag};

#[derive(Clone)]
struct GameRow {
    id: &'static str,
    name: &'static str,
    leader: &'static str,
    turn: u32,
    era: &'static str,
    score: u32,
    difficulty: &'static str,
    players: &'static str,
    map: &'static str,
    last: &'static str,
    status: GameStatus,
    notif: u32,
    mp: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameStatus {
    YourTurn,
    Waiting,
    Completed,
}

const SAMPLE: &[GameRow] = &[
    GameRow { id: "g_2049", name: "Cradle of the Indus", leader: "Saladin · Arabia", turn: 142, era: "Medieval", score: 814, difficulty: "Prince", players: "1H · 7AI", map: "Continents · Std", last: "2h ago", status: GameStatus::YourTurn, notif: 3, mp: false },
    GameRow { id: "g_2051", name: "Test seed 0xCAFE", leader: "Trajan · Rome", turn: 12, era: "Ancient", score: 41, difficulty: "Settler", players: "1H · 3AI", map: "Archipelago · Sm", last: "10m ago", status: GameStatus::YourTurn, notif: 1, mp: false },
    GameRow { id: "g_2044", name: "Friday night MP", leader: "Cleopatra · Egypt", turn: 56, era: "Classical", score: 220, difficulty: "Prince", players: "4H · 4AI", map: "Fractal · Std", last: "5d ago", status: GameStatus::YourTurn, notif: 2, mp: true },
    GameRow { id: "g_2050", name: "Long Winter", leader: "Catherine · Russia", turn: 87, era: "Renaissance", score: 502, difficulty: "King", players: "3H · 5AI", map: "Pangaea · Large", last: "yesterday", status: GameStatus::Waiting, notif: 0, mp: true },
    GameRow { id: "g_2046", name: "Pacific Hegemony", leader: "Hojo · Japan", turn: 230, era: "Modern", score: 1192, difficulty: "Emperor", players: "1H · 9AI", map: "Continents · Huge", last: "3d ago", status: GameStatus::Waiting, notif: 0, mp: false },
    GameRow { id: "g_2030", name: "Tutorial run", leader: "Gandhi · India", turn: 312, era: "Atomic", score: 2410, difficulty: "Settler", players: "1H · 3AI", map: "Continents · Std", last: "2 wk ago", status: GameStatus::Completed, notif: 0, mp: false },
];

#[component]
pub fn OngoingGames(on_new: Callback<()>) -> impl IntoView {
    let your_turn = SAMPLE.iter().filter(|g| g.status == GameStatus::YourTurn).count();
    let total = SAMPLE.len();

    view! {
        <div style="flex:1; display:flex; flex-direction:column; min-height:0">
            <div class="content-header">
                <div class="title">"Ongoing games"</div>
                <span class="crumbs">{format!("{total} games · {your_turn} awaiting you")}</span>
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
                <input class="filter-search" placeholder="search games and notes…" />
                <span class="sep-v"></span>
                <button class="chip active">"your turn " <span class="x">"×"</span></button>
                <button class="chip">"waiting"</button>
                <button class="chip">"completed"</button>
                <button class="chip">"multiplayer"</button>
                <button class="chip">"+ filter"</button>
                <span style="margin-left:auto" class="muted xsmall">"sort"</span>
                <button class="chip">"recent ↓"</button>
            </div>

            <div style="flex:1; overflow:auto">
                <div class="games-grid">
                    {SAMPLE.iter().enumerate().map(|(i, g)| {
                        let is_yours = g.status == GameStatus::YourTurn;
                        let tile_class = if is_yours { "game-tile your-turn" } else { "game-tile" };
                        let resume_label = match g.status {
                            GameStatus::YourTurn => "→ Resume",
                            GameStatus::Waiting => "Open",
                            GameStatus::Completed => "Review",
                        };
                        let resume_variant = if is_yours { "accent" } else { "primary" };
                        let seed = (i + 1) as u64;

                        view! {
                            <div class=tile_class style="width:100%">
                                <div class="tile-head">
                                    <div>
                                        <div class="tile-name">{g.name}</div>
                                        <div class="leader">{g.leader}</div>
                                    </div>
                                    <div class="row gap-xs">
                                        {g.mp.then(|| view! { <Tag>"MP"</Tag> })}
                                        {(g.notif > 0).then(|| view! {
                                            <Tag variant="accent">{format!("!{}", g.notif)}</Tag>
                                        })}
                                    </div>
                                </div>
                                <div class="map-thumb">
                                    <MiniMap seed=seed style="position:absolute; inset:0; width:100%; height:100%" />
                                </div>
                                <div class="stats">
                                    <div class="row-stat"><span class="k">"turn"</span><span class="v">{g.turn.to_string()}</span></div>
                                    <div class="row-stat"><span class="k">"era"</span><span class="v">{g.era}</span></div>
                                    <div class="row-stat"><span class="k">"diff"</span><span class="v">{g.difficulty}</span></div>
                                    <div class="row-stat"><span class="k">"score"</span><span class="v">{g.score.to_string()}</span></div>
                                    <div class="row-stat"><span class="k">"players"</span><span class="v">{g.players}</span></div>
                                    <div class="row-stat"><span class="k">"last"</span><span class="v">{g.last}</span></div>
                                </div>
                                <div class="actions">
                                    <Btn variant="ghost" size="sm">"📝 Notes"</Btn>
                                    <span style="flex:1"></span>
                                    <Btn variant=resume_variant size="sm">{resume_label}</Btn>
                                    <Btn variant="ghost" size="sm">"···"</Btn>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
