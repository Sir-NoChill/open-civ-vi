//! Bindings for `/api/v1/notifications` and `/api/v1/turn-queue`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::map_overlays::MapOverlays;
use crate::types::web::notifications::Notifications;
use crate::types::web::turn_queue::TurnQueue;

pub async fn list(token: Option<&str>) -> Result<Notifications, ApiError> {
    fetch_json::<Notifications, ()>("GET", "/api/v1/notifications", token, None).await
}

pub async fn turn_queue(token: Option<&str>) -> Result<TurnQueue, ApiError> {
    fetch_json::<TurnQueue, ()>("GET", "/api/v1/turn-queue", token, None).await
}

pub async fn map_overlays(token: Option<&str>) -> Result<MapOverlays, ApiError> {
    fetch_json::<MapOverlays, ()>("GET", "/api/v1/map/overlays", token, None).await
}

/// `DELETE /api/v1/notifications/{id}` — dismiss a single notification.
pub async fn dismiss(token: Option<&str>, id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/notifications/{id}");
    fetch_json::<serde_json::Value, ()>("DELETE", &url, token, None)
        .await
        .map(|_| ())
}

/// `DELETE /api/v1/notifications` — dismiss all notifications for this player.
pub async fn dismiss_all(token: Option<&str>) -> Result<(), ApiError> {
    fetch_json::<serde_json::Value, ()>("DELETE", "/api/v1/notifications", token, None)
        .await
        .map(|_| ())
}
