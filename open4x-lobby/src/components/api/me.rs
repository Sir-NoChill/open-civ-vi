//! Bindings for `/api/v1/me`.

use serde::{Deserialize, Serialize};

use super::http::{ApiError, fetch_json};

#[derive(Debug, Clone, Deserialize)]
pub struct MeView {
    pub player_id: String,
    pub preferred_name: String,
    pub pronouns: String,
    pub bio: String,
    pub identities: Vec<IdentityView>,
    pub prefs: Preferences,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentityView {
    pub kind: String,
    pub label: String,
    pub primary_key: String,
    pub verified: Option<bool>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Preferences {
    pub density: String,
    pub color_scheme: String,
    pub keyboard_nav: bool,
    pub turn_notifications: bool,
    pub discoverable_by_id: bool,
}

pub async fn get() -> Result<MeView, ApiError> {
    fetch_json::<MeView, ()>("GET", "/api/v1/me", None).await
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PatchMeBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefs: Option<Preferences>,
}

pub async fn patch(body: PatchMeBody) -> Result<MeView, ApiError> {
    fetch_json::<MeView, PatchMeBody>("PATCH", "/api/v1/me", Some(&body)).await
}

pub async fn delete_me() -> Result<(), ApiError> {
    fetch_json::<serde_json::Value, ()>("DELETE", "/api/v1/me", None)
        .await
        .map(|_| ())
}
