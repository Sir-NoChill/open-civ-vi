//! `/api/v1/games` — list / create / read / soft-delete game rows.
//!
//! Phase 4.2 of `book/src/roadmap/accounts-and-login.md`. The
//! orchestrator (Phase 4.3) is still pending: `POST /api/v1/games`
//! creates a record but leaves `server_url` + `server_token`
//! empty so the lobby can wire the wizard end-to-end against this
//! surface today, and the orchestrator fills them in later.
//! `POST /games/{id}/resume` returns 503 until that's done.

#![cfg(feature = "ssr")]

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use open4x_accounts::games::{GameRecord, GameStatus, GameStore, GameStoreError, NewGame};
use serde::{Deserialize, Serialize};

use crate::server::auth::RequireSession;
use crate::server::orchestrator::{self, NewGameRequest};
use crate::server::AppState;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

// ───────────────────────────── Wire shape ────────────────────────────────────

/// What the browser sees. Mirrors `GameRecord` but never exposes
/// `server_token` (the lobby-only secret used at Resume time).
#[derive(Debug, Serialize)]
pub struct GameView {
    pub game_id: String,
    pub owner_player_id: String,
    pub name: String,
    pub leader: String,
    pub civ_id: String,
    pub difficulty: String,
    pub players_human: u32,
    pub players_ai: u32,
    pub map_type: String,
    pub map_size: String,
    pub seed: String,
    pub turn: u32,
    pub era: String,
    pub score: i32,
    pub status: GameStatus,
    pub server_url: String,
    pub last_played_at: Option<String>,
    pub created_at: String,
}

impl From<GameRecord> for GameView {
    fn from(g: GameRecord) -> Self {
        Self {
            game_id: g.game_id,
            owner_player_id: g.owner_player_id.display(),
            name: g.name,
            leader: g.leader,
            civ_id: g.civ_id,
            difficulty: g.difficulty,
            players_human: g.players_human,
            players_ai: g.players_ai,
            map_type: g.map_type,
            map_size: g.map_size,
            seed: g.seed,
            turn: g.turn,
            era: g.era,
            score: g.score,
            status: g.status,
            server_url: g.server_url,
            last_played_at: g.last_played_at,
            created_at: g.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GamesListResp {
    pub games: Vec<GameView>,
}

// ───────────────────────────── GET /api/v1/games ──────────────────────────────

pub async fn list(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
) -> Response {
    match state.games.list_for_player(player_id).await {
        Ok(rows) => Json(GamesListResp {
            games: rows.into_iter().map(GameView::from).collect(),
        })
        .into_response(),
        Err(e) => store_error_response(e),
    }
}

// ───────────────────────────── POST /api/v1/games ─────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateGameBody {
    pub name: String,
    pub leader: String,
    pub civ_id: String,
    pub difficulty: String,
    pub players_human: u32,
    pub players_ai: u32,
    pub map_type: String,
    pub map_size: String,
    pub seed: String,
}

pub async fn create(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    Json(body): Json<CreateGameBody>,
) -> Response {
    if body.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: "name_required",
                message: None,
            }),
        )
            .into_response();
    }

    // Best-effort orchestration: ask the configured open4x-server to
    // bootstrap a GameRoom. Failure here doesn't fail the lobby write
    // — the game row still lands with empty server_url/server_token,
    // and Resume returns 503 until a follow-up RetryBootstrap path
    // exists. The wizard's UX is intentionally robust to a flaky
    // game-server.
    let (server_url, server_token) = if state.game_server_url.is_empty() {
        (String::new(), String::new())
    } else {
        let map_dims = map_size_to_dims(&body.map_size);
        let req = NewGameRequest {
            display_name: Some(body.name.clone()),
            width: Some(map_dims.0),
            height: Some(map_dims.1),
            seed: parse_seed(&body.seed),
            num_ai: Some(body.players_ai),
            turn_limit: None,
        };
        match orchestrator::bootstrap_game(&state.game_server_url, &req).await {
            Ok(boot) => (boot.server_url, boot.server_token),
            Err(e) => {
                eprintln!("[orchestrator] bootstrap failed: {e}");
                (String::new(), String::new())
            }
        }
    };

    let new = NewGame {
        owner_player_id: player_id,
        name: body.name,
        leader: body.leader,
        civ_id: body.civ_id,
        difficulty: body.difficulty,
        players_human: body.players_human,
        players_ai: body.players_ai,
        map_type: body.map_type,
        map_size: body.map_size,
        seed: body.seed,
        server_url,
        server_token,
    };
    match state.games.create_game(new).await {
        Ok(g) => (StatusCode::CREATED, Json(GameView::from(g))).into_response(),
        Err(e) => store_error_response(e),
    }
}

/// Map the wire's coarse `map_size` enum into width/height tile counts
/// matching the JSX wizard's `map size` Popup ("duel 44×26 · tiny
/// 60×38 · small 74×46 · std 84×54 · large 96×60 · huge 106×66").
fn map_size_to_dims(size: &str) -> (u32, u32) {
    match size {
        "duel"  => (44, 26),
        "tiny"  => (60, 38),
        "small" => (74, 46),
        "large" => (96, 60),
        "huge"  => (106, 66),
        _       => (84, 54), // std default
    }
}

/// Parse the wizard's seed string (e.g. `"0xCAFE·B33F·1A77"`) into a
/// u64 by ignoring non-hex characters. Returns `None` if no hex
/// digits remain — open4x-server will pick its own seed.
fn parse_seed(s: &str) -> Option<u64> {
    let hex: String = s
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(16) // 64-bit ceiling
        .collect();
    if hex.is_empty() {
        None
    } else {
        u64::from_str_radix(&hex, 16).ok()
    }
}

// ───────────────────────────── GET /api/v1/games/{id} ─────────────────────────

pub async fn get_one(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    Path(game_id): Path<String>,
) -> Response {
    match state.games.get_game(&game_id).await {
        Ok(Some(g)) if g.owner_player_id == player_id => {
            Json(GameView::from(g)).into_response()
        }
        Ok(Some(_)) => (
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                error: "not_a_member",
                message: None,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "game_not_found",
                message: None,
            }),
        )
            .into_response(),
        Err(e) => store_error_response(e),
    }
}

// ───────────────────────────── DELETE /api/v1/games/{id} ──────────────────────

pub async fn delete_one(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    Path(game_id): Path<String>,
) -> Response {
    match state.games.soft_delete(&game_id, player_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => store_error_response(e),
    }
}

// ───────────────────────────── POST /games/{id}/resume ────────────────────────

pub async fn resume(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    Path(game_id): Path<String>,
) -> Response {
    let game = match state.games.get_game(&game_id).await {
        Ok(Some(g)) if g.owner_player_id == player_id => g,
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "not_a_member",
                    message: None,
                }),
            )
                .into_response()
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "game_not_found",
                    message: None,
                }),
            )
                .into_response()
        }
        Err(e) => return store_error_response(e),
    };

    if game.server_url.is_empty() || game.server_token.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorBody {
                error: "orchestrator_not_ready",
                message: Some(
                    "POST /games/{id}/resume needs the Phase 4.3 orchestrator wired to populate server_url and server_token."
                        .into(),
                ),
            }),
        )
            .into_response();
    }
    let _ = state.games.touch_last_played(&game.game_id).await;
    // For now: return the URL + a one-shot token for the client to
    // attach. Phase 4.3 may turn this into a 302 with a session-bridge
    // cookie instead.
    Json(ResumeResp {
        url: game.server_url,
        token: game.server_token,
    })
    .into_response()
}

#[derive(Debug, Serialize)]
struct ResumeResp {
    url: String,
    token: String,
}

// ───────────────────────────── error mapping ──────────────────────────────────

fn store_error_response(e: GameStoreError) -> Response {
    let (status, code) = match &e {
        GameStoreError::NotFound => (StatusCode::NOT_FOUND, "game_not_found"),
        GameStoreError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
        GameStoreError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, "store_error"),
    };
    (
        status,
        Json(ErrorBody {
            error: code,
            message: Some(e.to_string()),
        }),
    )
        .into_response()
}
