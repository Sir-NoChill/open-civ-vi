//! Bindings for `/api/v1/tech*` and `/api/v1/civics*`.

use serde::Serialize;

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::civics_tree::CivicsTreeView;
use crate::types::web::tech_tree::TechTreeView;
use crate::types::web::MutationResponse;

pub async fn tech(token: Option<&str>) -> Result<TechTreeView, ApiError> {
    fetch_json::<TechTreeView, ()>("GET", "/api/v1/tech", token, None).await
}

#[derive(Debug, Clone, Serialize)]
pub struct TechResearchBody {
    pub tech_id: String,
}

pub async fn research_tech(
    token: Option<&str>,
    body: &TechResearchBody,
) -> Result<MutationResponse<TechTreeView>, ApiError> {
    fetch_json::<MutationResponse<TechTreeView>, TechResearchBody>(
        "POST",
        "/api/v1/tech/research",
        token,
        Some(body),
    )
    .await
}

pub async fn civics(token: Option<&str>) -> Result<CivicsTreeView, ApiError> {
    fetch_json::<CivicsTreeView, ()>("GET", "/api/v1/civics", token, None).await
}

#[derive(Debug, Clone, Serialize)]
pub struct CivicResearchBody {
    pub civic_id: String,
}

pub async fn research_civic(
    token: Option<&str>,
    body: &CivicResearchBody,
) -> Result<MutationResponse<CivicsTreeView>, ApiError> {
    fetch_json::<MutationResponse<CivicsTreeView>, CivicResearchBody>(
        "POST",
        "/api/v1/civics/research",
        token,
        Some(body),
    )
    .await
}
