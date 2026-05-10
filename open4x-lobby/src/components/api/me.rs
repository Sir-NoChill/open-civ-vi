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
    /// Public URL to the player's avatar PNG when one has been
    /// uploaded. `None` falls back to the initial-letter circle.
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentityView {
    /// Stable row id — pass to `unlink_identity` / `set_primary`.
    /// May be empty on responses that didn't carry id metadata
    /// (e.g. the PATCH /me echo).
    #[serde(default)]
    pub id: String,
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

/// `DELETE /api/v1/me/identities/{id}`.
pub async fn unlink_identity(id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/me/identities/{id}");
    fetch_json::<serde_json::Value, ()>("DELETE", &url, None)
        .await
        .map(|_| ())
}

/// `POST /api/v1/me/identities/{id}/verify-start`. Mints + mails a
/// fresh magic-link for an unverified email identity. The verify
/// route on the resulting click flips the row's `verified` column.
pub async fn start_verify_identity(id: &str) -> Result<(), ApiError> {
    let url = format!("/api/v1/me/identities/{id}/verify-start");
    fetch_json::<serde_json::Value, ()>("POST", &url, None)
        .await
        .map(|_| ())
}

/// `POST /api/v1/me/avatar` (multipart). Wraps the user's
/// `<input type=file>`-selected `File` in a fresh `FormData` and
/// hands it to `fetch`. Returns the new public URL on success.
pub async fn upload_avatar(file: web_sys::File) -> Result<String, ApiError> {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{FormData, Request, RequestInit, Response};

    let form = FormData::new().map_err(|e| ApiError::transport(format!("{e:?}")))?;
    form.append_with_blob("file", &file)
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;

    let init = RequestInit::new();
    init.set_method("POST");
    init.set_credentials(web_sys::RequestCredentials::SameOrigin);
    init.set_body(&JsValue::from(form));
    let req = Request::new_with_str_and_init("/api/v1/me/avatar", &init)
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;

    let window = web_sys::window().ok_or_else(|| ApiError::transport("no window"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&req))
        .await
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| ApiError::transport("non-Response result"))?;
    let status = resp.status();
    let text = JsFuture::from(
        resp.text()
            .map_err(|e| ApiError::transport(format!("{e:?}")))?,
    )
    .await
    .map_err(|e| ApiError::transport(format!("{e:?}")))?
    .as_string()
    .unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(ApiError {
            status,
            code: "upload_failed".into(),
            message: Some(text),
        });
    }
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| ApiError::transport(e.to_string()))?;
    Ok(v.get("avatar_url")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string())
}
