//! `GET /api/v1/victory` — leaderboard and per-condition progress.

use open4x_protocol::v1::web::victory::Victory;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn get<T: Transport>(t: &T) -> Result<Victory, ApiError> {
    let body = t.request(Method::Get, "/api/v1/victory", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
