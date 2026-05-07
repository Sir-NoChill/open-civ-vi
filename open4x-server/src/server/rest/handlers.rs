//! REST handler functions for `/api/v1/*`.
//!
//! Phase 1 wires the HUD MVP: `health`, `player_state`, `world_snapshot`,
//! `world_tile`, and `end_turn`. Subsequent phases extend the surface from
//! `book/src/roadmap/web-ui.md` §4.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::server::api_token::{generate_token, ApiTokenRecord};
use crate::server::projection::project_game_view;
use crate::server::rest::auth::{auth_or_401, ApiError};
use crate::server::state::{AppState, GameRoom, GameRoomConfig, PlayerRecord};
use crate::server::web_projection;
use crate::types::ids::{CivId, GameId};
use crate::types::messages::{CreateGameRequest, GameStatus};
use crate::types::view::GameView;
use crate::types::web::{MutationResponse, TurnStatusBlock};

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub api: &'static str,
}

/// `GET /api/v1/health` — unauthenticated liveness check.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        api: "v1",
    })
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Project a fresh `GameView` for the authenticated player and pull the
/// configured `turn_limit` from the room.
fn view_and_turn_limit(
    state: &Arc<AppState>,
    game_id: GameId,
    civ_id: CivId,
) -> Result<(GameView, Option<u32>), ApiError> {
    let room = state
        .games
        .get(&game_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("game not found"))?;
    let libciv_civ_id = libciv::CivId::from_ulid(civ_id.as_ulid());
    let view = project_game_view(&room.state, libciv_civ_id);
    Ok((view, room.config.turn_limit))
}

fn turn_status_block(state: &Arc<AppState>, game_id: GameId) -> TurnStatusBlock {
    let turn = state
        .games
        .get(&game_id)
        .map(|r| r.state.turn)
        .unwrap_or(0);
    TurnStatusBlock { turn, ended: false }
}

// ── POST /games/new — bootstrap a single-player game over REST ───────────────

#[derive(Deserialize, Default)]
pub struct NewGameRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub num_ai: Option<u32>,
    #[serde(default)]
    pub turn_limit: Option<u32>,
}

#[derive(Serialize)]
pub struct NewGameResponse {
    pub game_id: GameId,
    pub civ_id: CivId,
    pub token: String,
    pub turn: u32,
}

/// `POST /api/v1/games/new` — create a fresh single-player game and mint a
/// bearer token for it. Unauthenticated; intended for the single-player REST
/// loop (multiplayer keeps using the WS auth handshake).
pub async fn new_game(
    State(state): State<Arc<AppState>>,
    Json(req): Json<NewGameRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use rand::Rng;

    let display_name = req.display_name.clone().unwrap_or_else(|| "Player".into());

    // Generate an anonymous pubkey for this single-player session.
    let mut rng = rand::rng();
    let pubkey: [u8; 32] = rng.random();

    // Register a PlayerRecord so build_server_session can look it up.
    state.players.insert(
        pubkey,
        PlayerRecord {
            pubkey,
            display_name: display_name.clone(),
            selected_template: state.templates[0].id,
            games_played: 0,
        },
    );

    let game_id = GameId::from_ulid(ulid::Ulid::new());

    let create_req = CreateGameRequest {
        name: format!("{display_name}'s game"),
        width: req.width.unwrap_or(40),
        height: req.height.unwrap_or(24),
        seed: req.seed.unwrap_or(42),
        num_ai: req.num_ai.unwrap_or(1),
        max_players: 1,
        turn_limit: req.turn_limit.or(Some(500)),
    };

    let session = crate::server::session::build_server_session(&create_req, &pubkey, &state, game_id);
    let civ_id = session
        .players
        .first()
        .map(|s| s.civ_id)
        .ok_or_else(|| crate::server::rest::auth::bad_request("no_player_slot", "session has no player"))?;

    let (tx, _rx) = broadcast::channel(64);
    let room = GameRoom {
        game_id,
        name: create_req.name.clone(),
        state: session.state,
        rules: libciv::DefaultRulesEngine,
        players: session.players,
        ai_agents: session.ai_agents,
        status: GameStatus::InProgress,
        config: GameRoomConfig {
            max_players: 1,
            turn_limit: create_req.turn_limit,
        },
        tx,
    };

    let initial_turn = room.state.turn;
    state.games.insert(game_id, room);

    // Mint and store the bearer token.
    let token = generate_token();
    state.api_tokens.insert(
        token.clone(),
        ApiTokenRecord {
            token: token.clone(),
            pubkey,
            game_id,
            civ_id: CivId::from_ulid(civ_id.as_ulid()),
        },
    );

    Ok((
        StatusCode::CREATED,
        Json(NewGameResponse {
            game_id,
            civ_id: CivId::from_ulid(civ_id.as_ulid()),
            token,
            turn: initial_turn,
        }),
    ))
}

// ── /player-state ────────────────────────────────────────────────────────────

pub async fn player_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, turn_limit) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_player_state(&view, turn_limit)))
}

// ── /world/snapshot ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WorldSnapshotQuery {
    pub q: Option<i32>,
    pub r: Option<i32>,
    pub radius: Option<u32>,
}

pub async fn world_snapshot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WorldSnapshotQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let q = params.q.unwrap_or(0);
    let r = params.r.unwrap_or(0);
    // Default radius 0 = "all explored". The plan caps the radius at 32.
    let radius = params.radius.unwrap_or(0).min(32);
    Ok(Json(web_projection::build_world_snapshot(&view, q, r, radius)))
}

// ── /world/tile/{q}/{r} ──────────────────────────────────────────────────────

pub async fn world_tile(
    State(state): State<Arc<AppState>>,
    Path((q, r)): Path<(i32, i32)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let snapshot = web_projection::build_world_snapshot(&view, q, r, 1);
    snapshot
        .tiles
        .into_iter()
        .find(|t| t.q == q && t.r == r)
        .map(Json)
        .ok_or_else(|| crate::server::rest::auth::not_found("tile not in player view"))
}

// ── POST /turn/end ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EndTurnView {
    pub turn: u32,
}

pub async fn end_turn(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ_id = libciv::CivId::from_ulid(civ_id.as_ulid());

    let mut room = state
        .games
        .get_mut(&game_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("game not found"))?;

    // Mark this player's slot submitted.
    if let Some(slot) = room.players.iter_mut().find(|s| s.civ_id == libciv_civ_id) {
        slot.submitted_turn = true;
    }

    // For Phase 1 single-player, advance immediately. Multiplayer gating
    // (room.all_submitted()) lives on the WS path and is out of scope here.
    room.resolve_turn();

    let new_turn = room.state.turn;
    drop(room);

    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view: EndTurnView { turn: new_turn },
            turn_status: TurnStatusBlock {
                turn: new_turn,
                ended: false,
            },
        }),
    ))
}

// ── /cities ──────────────────────────────────────────────────────────────────

pub async fn cities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_cities(&view)))
}

pub async fn city_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let cities = web_projection::build_cities(&view);
    cities
        .cities
        .into_iter()
        .find(|c| c.id == id)
        .map(Json)
        .ok_or_else(|| crate::server::rest::auth::not_found("city not found"))
}

pub async fn city_tiles(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    web_projection::build_city_tiles(&view, &id)
        .map(Json)
        .ok_or_else(|| crate::server::rest::auth::not_found("city not found"))
}

// ── /units ───────────────────────────────────────────────────────────────────

pub async fn units(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_units(&view)))
}

pub async fn unit_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let units = web_projection::build_units(&view);
    units
        .units
        .into_iter()
        .find(|u| u.id == id)
        .map(Json)
        .ok_or_else(|| crate::server::rest::auth::not_found("unit not found"))
}

// ── /armies (stub for Phase 4) ───────────────────────────────────────────────

pub async fn armies(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_armies(&view)))
}

// ── /combat/preview ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CombatPreviewQuery {
    pub attacker_id: String,
    pub defender_q: i32,
    pub defender_r: i32,
}

pub async fn combat_preview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CombatPreviewQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    web_projection::build_combat_preview(
        &view,
        &params.attacker_id,
        params.defender_q,
        params.defender_r,
    )
    .map(Json)
    .ok_or_else(|| crate::server::rest::auth::not_found("attacker not found"))
}

// ── mutations: city production ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct QueueProductionBody {
    pub item_id: String,
    pub item_type: String, // "unit" | "building" | "wonder" | "district" | "project"
}

/// `POST /api/v1/cities/{id}/production` — append `{item_id, item_type}` to the
/// city's production queue.
pub async fn queue_production(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<QueueProductionBody>,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ = libciv::CivId::from_ulid(civ_id.as_ulid());

    let city_ulid: ulid::Ulid = id
        .parse()
        .map_err(|_| crate::server::rest::auth::bad_request("invalid_id", "invalid city id"))?;
    let city_id = crate::types::ids::CityId::from_ulid(city_ulid);

    let item_ulid: ulid::Ulid = body.item_id.parse().map_err(|_| {
        crate::server::rest::auth::bad_request("invalid_id", "invalid item id")
    })?;
    let item = match body.item_type.as_str() {
        "unit" => crate::types::enums::ProductionItemView::Unit(
            crate::types::ids::UnitTypeId::from_ulid(item_ulid),
        ),
        "building" => crate::types::enums::ProductionItemView::Building(
            crate::types::ids::BuildingId::from_ulid(item_ulid),
        ),
        "wonder" => crate::types::enums::ProductionItemView::Wonder(
            crate::types::ids::WonderId::from_ulid(item_ulid),
        ),
        "project" => crate::types::enums::ProductionItemView::Project(
            crate::types::ids::ProjectId::from_ulid(item_ulid),
        ),
        // District is a plain enum (not ULID); unsupported here.
        other => {
            return Err(crate::server::rest::auth::bad_request(
                "invalid_item_type",
                &format!("item_type {other:?} not supported via REST yet"),
            ));
        }
    };

    let action = crate::types::messages::GameAction::QueueProduction { city: city_id, item };

    let new_turn = mutate_room(&state, game_id, |room| room.apply_action(libciv_civ, &action))?;

    let view = view_after_mutation_city(&state, game_id, civ_id, &id)?;
    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view,
            turn_status: TurnStatusBlock { turn: new_turn, ended: false },
        }),
    ))
}

/// `DELETE /api/v1/cities/{id}/production/{pos}` — remove queue entry at index.
pub async fn cancel_production(
    State(state): State<Arc<AppState>>,
    Path((id, pos)): Path<(String, usize)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ = libciv::CivId::from_ulid(civ_id.as_ulid());

    let city_ulid: ulid::Ulid = id
        .parse()
        .map_err(|_| crate::server::rest::auth::bad_request("invalid_id", "invalid city id"))?;
    let city_id = crate::types::ids::CityId::from_ulid(city_ulid);

    let action = crate::types::messages::GameAction::CancelProduction { city: city_id, index: pos };
    let new_turn = mutate_room(&state, game_id, |room| room.apply_action(libciv_civ, &action))?;

    let view = view_after_mutation_city(&state, game_id, civ_id, &id)?;
    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view,
            turn_status: TurnStatusBlock { turn: new_turn, ended: false },
        }),
    ))
}

// ── mutations: unit actions ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UnitActionBody {
    pub action_id: String,                 // "move" | "attack" | "fortify" | "sleep" | "found_city"
    #[serde(default)]
    pub target_q: Option<i32>,
    #[serde(default)]
    pub target_r: Option<i32>,
    /// City name when `action_id == "found_city"`. Defaults to "New City".
    #[serde(default)]
    pub name: Option<String>,
}

/// `POST /api/v1/units/{id}/action` — dispatch a unit action through libciv.
pub async fn unit_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<UnitActionBody>,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ = libciv::CivId::from_ulid(civ_id.as_ulid());

    let unit_ulid: ulid::Ulid = id
        .parse()
        .map_err(|_| crate::server::rest::auth::bad_request("invalid_id", "invalid unit id"))?;
    let unit_id = crate::types::ids::UnitId::from_ulid(unit_ulid);

    let target = match (body.target_q, body.target_r) {
        (Some(q), Some(r)) => Some(crate::types::coord::HexCoord { q, r, s: -q - r }),
        _ => None,
    };

    let action = match body.action_id.as_str() {
        "move" => {
            let to = target.ok_or_else(|| {
                crate::server::rest::auth::bad_request("missing_target", "move requires target_q/target_r")
            })?;
            crate::types::messages::GameAction::MoveUnit { unit: unit_id, to }
        }
        "attack" => {
            // Resolve defender by coord lookup against the player's view.
            let to = target.ok_or_else(|| {
                crate::server::rest::auth::bad_request("missing_target", "attack requires target_q/target_r")
            })?;
            let view = view_only(&state, game_id, civ_id)?;
            let defender = view
                .units
                .iter()
                .find(|u| u.coord == to && !u.is_own)
                .ok_or_else(|| crate::server::rest::auth::not_found("no enemy unit at target"))?;
            crate::types::messages::GameAction::Attack {
                attacker: unit_id,
                defender: defender.id,
            }
        }
        "found_city" => crate::types::messages::GameAction::FoundCity {
            settler: unit_id,
            name: body.name.clone().unwrap_or_else(|| "New City".into()),
        },
        "fortify" | "sleep" => {
            // No matching GameAction variant yet; treat as a UI no-op so the
            // wireframe doesn't fail. Will plumb through libciv in Phase 4.
            let new_turn = state.games.get(&game_id).map(|r| r.state.turn).unwrap_or(0);
            return Ok((
                StatusCode::ACCEPTED,
                Json(MutationResponse {
                    ok: true,
                    view: serde_json::Value::Null,
                    turn_status: TurnStatusBlock { turn: new_turn, ended: false },
                }),
            ));
        }
        other => {
            return Err(crate::server::rest::auth::bad_request(
                "unknown_action",
                &format!("unit action {other:?} not supported"),
            ));
        }
    };

    let new_turn = mutate_room(&state, game_id, |room| room.apply_action(libciv_civ, &action))?;

    let view = view_only(&state, game_id, civ_id)?;
    let unit = web_projection::build_units(&view)
        .units
        .into_iter()
        .find(|u| u.id == id);

    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view: serde_json::to_value(unit).unwrap_or(serde_json::Value::Null),
            turn_status: TurnStatusBlock { turn: new_turn, ended: false },
        }),
    ))
}

// ── mutation helpers ─────────────────────────────────────────────────────────

fn mutate_room<F>(
    state: &Arc<AppState>,
    game_id: GameId,
    f: F,
) -> Result<u32, ApiError>
where
    F: FnOnce(&mut GameRoom) -> Result<(), String>,
{
    let mut room = state
        .games
        .get_mut(&game_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("game not found"))?;
    f(&mut room).map_err(|e| crate::server::rest::auth::bad_request("rule_violation", &e))?;
    Ok(room.state.turn)
}

fn view_only(
    state: &Arc<AppState>,
    game_id: GameId,
    civ_id: CivId,
) -> Result<GameView, ApiError> {
    let (view, _) = view_and_turn_limit(state, game_id, civ_id)?;
    Ok(view)
}

fn view_after_mutation_city(
    state: &Arc<AppState>,
    game_id: GameId,
    civ_id: CivId,
    city_id: &str,
) -> Result<crate::types::web::city_data::CityRow, ApiError> {
    let view = view_only(state, game_id, civ_id)?;
    web_projection::build_cities(&view)
        .cities
        .into_iter()
        .find(|c| c.id == city_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("city not found"))
}

// ── /tech ────────────────────────────────────────────────────────────────────

pub async fn tech(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_tech_tree(&view)))
}

#[derive(Deserialize)]
pub struct TechResearchBody {
    pub tech_id: String,
}

pub async fn tech_research(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<TechResearchBody>,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ = libciv::CivId::from_ulid(civ_id.as_ulid());

    let tech_ulid: ulid::Ulid = body
        .tech_id
        .parse()
        .map_err(|_| crate::server::rest::auth::bad_request("invalid_id", "invalid tech id"))?;
    let tech = crate::types::ids::TechId::from_ulid(tech_ulid);
    let action = crate::types::messages::GameAction::QueueResearch { tech };

    let new_turn = mutate_room(&state, game_id, |room| room.apply_action(libciv_civ, &action))?;
    let view = view_only(&state, game_id, civ_id)?;
    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view: web_projection::build_tech_tree(&view),
            turn_status: TurnStatusBlock { turn: new_turn, ended: false },
        }),
    ))
}

// ── /civics ──────────────────────────────────────────────────────────────────

pub async fn civics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_civics_tree(&view)))
}

#[derive(Deserialize)]
pub struct CivicResearchBody {
    pub civic_id: String,
}

pub async fn civic_research(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CivicResearchBody>,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ = libciv::CivId::from_ulid(civ_id.as_ulid());

    let civic_ulid: ulid::Ulid = body
        .civic_id
        .parse()
        .map_err(|_| crate::server::rest::auth::bad_request("invalid_id", "invalid civic id"))?;
    let civic = crate::types::ids::CivicId::from_ulid(civic_ulid);
    let action = crate::types::messages::GameAction::QueueCivic { civic };

    let new_turn = mutate_room(&state, game_id, |room| room.apply_action(libciv_civ, &action))?;
    let view = view_only(&state, game_id, civ_id)?;
    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view: web_projection::build_civics_tree(&view),
            turn_status: TurnStatusBlock { turn: new_turn, ended: false },
        }),
    ))
}

// ── /government ──────────────────────────────────────────────────────────────

pub async fn government(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_government(&view)))
}

// ── /map/overlays (Phase 4 stub) ─────────────────────────────────────────────

pub async fn map_overlays(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_map_overlays(&view)))
}
