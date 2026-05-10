//! Bindings for `/api/v1/games`.

use serde::{Deserialize, Serialize};

use super::http::{ApiError, fetch_json};

#[derive(Debug, Clone, Deserialize)]
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
    pub status: String, // 'your_turn' | 'waiting' | 'completed' | 'archived'
    pub server_url: String,
    pub last_played_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GamesListResp {
    pub games: Vec<GameView>,
}

pub async fn list() -> Result<GamesListResp, ApiError> {
    fetch_json::<GamesListResp, ()>("GET", "/api/v1/games", None).await
}

#[derive(Debug, Clone, Default, Serialize)]
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

pub async fn create(body: CreateGameBody) -> Result<GameView, ApiError> {
    fetch_json::<GameView, CreateGameBody>("POST", "/api/v1/games", Some(&body)).await
}

pub async fn delete_game(game_id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/games/{game_id}");
    fetch_json::<serde_json::Value, ()>("DELETE", &url, None)
        .await
        .map(|_| ())
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResumeResp {
    pub url: String,
    pub token: String,
}

/// `POST /api/v1/games/{id}/resume`. Returns the open4x-server URL +
/// bearer token the browser should use to enter the in-game SPA.
pub async fn resume(game_id: &str) -> Result<ResumeResp, ApiError> {
    let url = format!("/api/v1/games/{game_id}/resume");
    fetch_json::<ResumeResp, ()>("POST", &url, None).await
}
