//! `/api/v1/world/*` — terrain snapshot and per-tile lookup.

use open4x_protocol::v1::web::world::{TileView, WorldSnapshot};

use crate::error::ApiError;
use crate::transport::{Method, Transport};

/// `GET /api/v1/world/snapshot[?q&r&radius]`.
///
/// All three query params are optional. Server defaults: `(q, r) = (0, 0)`
/// and `radius = 0` (interpreted as "all explored", capped at 32).
pub async fn snapshot<T: Transport>(
    t: &T,
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
    let body = t.request(Method::Get, &url, None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}

/// `GET /api/v1/world/tile/{q}/{r}`.
pub async fn tile<T: Transport>(t: &T, q: i32, r: i32) -> Result<TileView, ApiError> {
    let url = format!("/api/v1/world/tile/{q}/{r}");
    let body = t.request(Method::Get, &url, None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
