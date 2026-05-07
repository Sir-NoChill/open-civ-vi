//! Bindings for `/api/v1/world/*`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::world::{TileView, WorldSnapshot};

/// `GET /api/v1/world/snapshot[?q&r&radius]`
pub async fn snapshot(
    token: Option<&str>,
    q: Option<i32>,
    r: Option<i32>,
    radius: Option<u32>,
) -> Result<WorldSnapshot, ApiError> {
    let mut url = String::from("/api/v1/world/snapshot");
    let mut sep = '?';
    if let Some(v) = q {
        url.push(sep);
        url.push_str(&format!("q={v}"));
        sep = '&';
    }
    if let Some(v) = r {
        url.push(sep);
        url.push_str(&format!("r={v}"));
        sep = '&';
    }
    if let Some(v) = radius {
        url.push(sep);
        url.push_str(&format!("radius={v}"));
    }
    fetch_json::<WorldSnapshot, ()>("GET", &url, token, None).await
}

/// `GET /api/v1/world/tile/{q}/{r}`
pub async fn tile(token: Option<&str>, q: i32, r: i32) -> Result<TileView, ApiError> {
    let url = format!("/api/v1/world/tile/{q}/{r}");
    fetch_json::<TileView, ()>("GET", &url, token, None).await
}
