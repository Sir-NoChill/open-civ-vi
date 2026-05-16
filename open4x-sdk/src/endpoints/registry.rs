//! `GET /api/v1/registry` — catalogue of unit types and buildings.

use open4x_protocol::v1::web::registry::Registry;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn get<T: Transport>(t: &T) -> Result<Registry, ApiError> {
    let body = t.request(Method::Get, "/api/v1/registry", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
