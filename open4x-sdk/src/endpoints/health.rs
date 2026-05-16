//! `/api/v1/health` — unauthenticated liveness probe.

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::transport::{Method, Transport};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub api: String,
}

/// `GET /api/v1/health` — returns `{ok: true, api: "v1"}`.
pub async fn health<T: Transport>(t: &T) -> Result<HealthResponse, ApiError> {
    let body = t.request(Method::Get, "/api/v1/health", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
