//! Orchestrator — Phase 4.3 of
//! `book/src/roadmap/accounts-and-login.md`.
//!
//! Today: shared-server-multi-room v1. The lobby reaches out to a
//! single configured `open4x-server` instance (env
//! `OPEN4X_GAME_SERVER_URL`, default `http://localhost:3001`),
//! POSTs the wizard params to that server's
//! `POST /api/v1/games/new`, and records the returned
//! anonymous bearer token + the server URL on the lobby's `games`
//! row so subsequent Resume requests can hand them to the client.
//!
//! The cross-crate auth-key handshake (lobby-issued tokens that
//! open4x-server validates via shared HMAC) is reserved for v2 and
//! tracked as a follow-up — for v1 the in-game server's anonymous
//! bootstrap path keeps the surface working without changing
//! `open4x-server`.

#![cfg(feature = "ssr")]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("game-server transport: {0}")]
    Transport(String),
    #[error("game-server returned {status}: {body}")]
    Server { status: u16, body: String },
    #[error("game-server response missing required field: {0}")]
    BadResponse(&'static str),
}

#[derive(Debug, Clone, Serialize)]
pub struct NewGameRequest {
    pub display_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<u64>,
    pub num_ai: Option<u32>,
    pub turn_limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewGameResponse {
    pub game_id: String,
    #[serde(default)]
    pub civ_id: String,
    pub token: String,
    #[serde(default)]
    pub turn: u32,
}

/// Result of [`bootstrap_game`]: enough to populate the lobby's
/// `games.server_url` + `games.server_token` columns.
#[derive(Debug, Clone)]
pub struct BootstrappedGame {
    pub server_url: String,
    pub server_token: String,
    pub remote_game_id: String,
}

/// Hit the configured open4x-server's `POST /api/v1/games/new` and
/// return the URL + token the lobby should remember. Idempotent on
/// the lobby side: callers store the result regardless.
pub async fn bootstrap_game(
    server_url: &str,
    req: &NewGameRequest,
) -> Result<BootstrappedGame, OrchestratorError> {
    let server_url = server_url.trim_end_matches('/').to_string();
    let url = format!("{server_url}/api/v1/games/new");
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| OrchestratorError::Transport(e.to_string()))?;
    let resp = client
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| OrchestratorError::Transport(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(OrchestratorError::Server {
            status: status.as_u16(),
            body,
        });
    }
    let parsed: NewGameResponse = resp
        .json()
        .await
        .map_err(|e| OrchestratorError::Transport(e.to_string()))?;

    if parsed.token.is_empty() {
        return Err(OrchestratorError::BadResponse("token"));
    }
    if parsed.game_id.is_empty() {
        return Err(OrchestratorError::BadResponse("game_id"));
    }

    Ok(BootstrappedGame {
        server_url,
        server_token: parsed.token,
        remote_game_id: parsed.game_id,
    })
}
