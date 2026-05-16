//! `GET /api/v1/armies` — stub for the future army-grouping UI.

use open4x_protocol::v1::web::army_data::ArmyData;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn list<T: Transport>(t: &T) -> Result<ArmyData, ApiError> {
    let body = t.request(Method::Get, "/api/v1/armies", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
