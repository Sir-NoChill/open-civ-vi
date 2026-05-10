//! `/api/v1/me` — read / update / delete the authenticated account.

#![cfg(feature = "ssr")]

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use open4x_accounts::store::AccountStore;
use open4x_accounts::{Account, Preferences};
use serde::{Deserialize, Serialize};

use crate::server::auth::RequireSession;
use crate::server::AppState;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

pub async fn get_me(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
) -> Response {
    match state.store.get_by_player_id(player_id).await {
        Ok(Some(acct)) => Json(MeView::from(acct)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "account_not_found",
                message: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "store_error",
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct PatchMeBody {
    pub preferred_name: Option<String>,
    pub pronouns: Option<String>,
    pub bio: Option<String>,
    pub prefs: Option<Preferences>,
}

pub async fn patch_me(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    Json(body): Json<PatchMeBody>,
) -> Response {
    match state
        .store
        .update_profile(
            player_id,
            body.preferred_name,
            body.pronouns,
            body.bio,
            body.prefs,
        )
        .await
    {
        Ok(acct) => Json(MeView::from(acct)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "store_error",
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

pub async fn delete_me(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
) -> Response {
    match state.store.delete_account(player_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "store_error",
                message: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

/// Wire shape returned for `/me`. Renames the `PlayerId` to its
/// human-readable hex form so the client never sees the raw u64.
#[derive(Debug, Serialize)]
pub struct MeView {
    pub player_id: String,
    pub preferred_name: String,
    pub pronouns: String,
    pub bio: String,
    pub identities: Vec<IdentityView>,
    pub prefs: Preferences,
}

#[derive(Debug, Serialize)]
pub struct IdentityView {
    pub kind: &'static str,
    pub label: String,
    pub primary_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

impl From<Account> for MeView {
    fn from(a: Account) -> Self {
        let identities = a
            .identities
            .into_iter()
            .map(|id| match id {
                open4x_accounts::Identity::Email {
                    address,
                    verified,
                    primary,
                } => IdentityView {
                    kind: "email",
                    label: address.clone(),
                    primary_key: address,
                    verified: Some(verified),
                    primary: Some(primary),
                },
                open4x_accounts::Identity::OpenId {
                    issuer,
                    subject,
                    label,
                } => IdentityView {
                    kind: "oidc",
                    label,
                    primary_key: format!("{issuer}|{subject}"),
                    verified: None,
                    primary: None,
                },
                open4x_accounts::Identity::Atproto { did, handle } => IdentityView {
                    kind: "atproto",
                    label: handle,
                    primary_key: did,
                    verified: None,
                    primary: None,
                },
            })
            .collect();
        Self {
            player_id: a.player_id.display(),
            preferred_name: a.preferred_name,
            pronouns: a.pronouns,
            bio: a.bio,
            identities,
            prefs: a.prefs,
        }
    }
}
