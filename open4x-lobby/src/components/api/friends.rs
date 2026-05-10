//! Bindings for `/api/v1/friends`.

use serde::{Deserialize, Serialize};

use super::http::{fetch_json, ApiError};

#[derive(Debug, Clone, Deserialize)]
pub struct FriendView {
    pub player_id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ListResp {
    friends: Vec<FriendView>,
}

pub async fn list() -> Result<Vec<FriendView>, ApiError> {
    let r: ListResp = fetch_json::<ListResp, ()>("GET", "/api/v1/friends", None).await?;
    Ok(r.friends)
}

#[derive(Debug, Serialize)]
struct ReqBody {
    player_id: String,
}

pub async fn request(player_id: String) -> Result<(), ApiError> {
    let body = ReqBody { player_id };
    fetch_json::<serde_json::Value, ReqBody>("POST", "/api/v1/friends/request", Some(&body))
        .await
        .map(|_| ())
}

pub async fn accept(player_id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/friends/{player_id}/accept");
    fetch_json::<serde_json::Value, ()>("POST", &url, None)
        .await
        .map(|_| ())
}

pub async fn unfriend(player_id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/friends/{player_id}");
    fetch_json::<serde_json::Value, ()>("DELETE", &url, None)
        .await
        .map(|_| ())
}
