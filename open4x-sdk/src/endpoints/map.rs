//! `GET /api/v1/map/overlays` — layer toggles (yields/appeal/etc.).

use open4x_protocol::v1::web::map_overlays::MapOverlays;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn overlays<T: Transport>(t: &T) -> Result<MapOverlays, ApiError> {
    let body = t.request(Method::Get, "/api/v1/map/overlays", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
