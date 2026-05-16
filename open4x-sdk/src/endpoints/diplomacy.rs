//! `/api/v1/diplomacy*` — civ relations summary.

use open4x_protocol::v1::web::diplomacy::{CivRow, Diplomacy};

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn get<T: Transport>(t: &T) -> Result<Diplomacy, ApiError> {
    let body = t.request(Method::Get, "/api/v1/diplomacy", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}

pub async fn civ<T: Transport>(t: &T, id: &str) -> Result<CivRow, ApiError> {
    let url = format!("/api/v1/diplomacy/civs/{id}");
    let body = t.request(Method::Get, &url, None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
