//! `/api/v1/me` — read / update / delete the authenticated account.

#![cfg(feature = "ssr")]

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use open4x_accounts::audit::{AuditEventKind, AuditStore, NewAuditEvent};
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
        Ok(Some(acct)) => {
            // Pull identity ids alongside the account so the wire
            // shape exposes them. This is the only place we need
            // the id-augmented view; PATCH /me etc just round-trip
            // the identities list.
            let with_ids = state
                .store
                .list_identities_with_ids(player_id)
                .await
                .unwrap_or_default();
            Json(MeView::from_account_and_ids(acct, with_ids)).into_response()
        }
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
        Ok(()) => {
            let _ = state
                .audit
                .record(NewAuditEvent {
                    kind: AuditEventKind::AccountDeleted,
                    player_id: Some(player_id),
                    ip: None,
                    detail: String::new(),
                })
                .await;
            StatusCode::NO_CONTENT.into_response()
        }
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
    /// Stable row id — the lobby exposes this so the SPA can call
    /// `DELETE /api/v1/me/identities/{id}` without having to round-
    /// trip through the kind+primary_key tuple.
    pub id: String,
    pub kind: &'static str,
    pub label: String,
    pub primary_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

impl MeView {
    /// Used by `GET /me` where the caller has already pulled
    /// `(id, identity)` pairs alongside the account. Stable ids
    /// land in the `identities[].id` field so the SPA can reference
    /// them by id (DELETE / set-primary).
    fn from_account_and_ids(a: Account, with_ids: Vec<(String, open4x_accounts::Identity)>) -> Self {
        let identities = with_ids
            .into_iter()
            .map(|(id, identity)| identity_view(id, identity))
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

impl From<Account> for MeView {
    fn from(a: Account) -> Self {
        // Fallback: callers that don't have ids (PATCH /me) get an
        // empty id field. PATCH responses don't drive identity
        // mutations directly — the SPA refetches /me to pick up
        // ids after a link/unlink.
        let identities = a
            .identities
            .into_iter()
            .map(|id| identity_view(String::new(), id))
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

fn identity_view(id: String, identity: open4x_accounts::Identity) -> IdentityView {
    match identity {
        open4x_accounts::Identity::Email {
            address,
            verified,
            primary,
        } => IdentityView {
            id,
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
            id,
            kind: "oidc",
            label,
            primary_key: format!("{issuer}|{subject}"),
            verified: None,
            primary: None,
        },
        open4x_accounts::Identity::Atproto { did, handle } => IdentityView {
            id,
            kind: "atproto",
            label: handle,
            primary_key: did,
            verified: None,
            primary: None,
        },
    }
}

// ───────────────────────────── DELETE /me/identities/{id} ─────────────────────

pub async fn unlink_identity(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    axum::extract::Path(identity_id): axum::extract::Path<String>,
) -> Response {
    // Refuse to orphan the account: leave at least one linked
    // identity so the user can still sign back in.
    let current = state
        .store
        .list_identities_with_ids(player_id)
        .await
        .unwrap_or_default();
    if !current.iter().any(|(id, _)| id == &identity_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "identity_not_found",
                message: None,
            }),
        )
            .into_response();
    }
    if current.len() <= 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: "would_orphan_account",
                message: Some("link another identity before unlinking the last one".into()),
            }),
        )
            .into_response();
    }
    match state.store.unlink_identity(player_id, &identity_id).await {
        Ok(()) => {
            let _ = state
                .audit
                .record(NewAuditEvent {
                    kind: AuditEventKind::IdentityUnlinked,
                    player_id: Some(player_id),
                    ip: None,
                    detail: identity_id,
                })
                .await;
            StatusCode::NO_CONTENT.into_response()
        }
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

// ───────────────────────── POST /me/identities/{id}/verify-start ──────────────

/// Mint a fresh magic-link for an unverified email identity already
/// linked to the requester's account, and hand it to the configured
/// mailer. The user clicks the link → /auth/email/verify consumes
/// the nonce, signs them in (no-op if already signed in), and the
/// shared `mark_email_verified` hook flips this row's `verified`
/// column to true.
pub async fn verify_email_identity(
    State(state): State<AppState>,
    RequireSession(player_id): RequireSession,
    axum::extract::Path(identity_id): axum::extract::Path<String>,
) -> Response {
    let current = state
        .store
        .list_identities_with_ids(player_id)
        .await
        .unwrap_or_default();
    let target = current.iter().find(|(id, _)| id == &identity_id);
    let Some((_, ident)) = target else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "identity_not_found",
                message: None,
            }),
        )
            .into_response();
    };
    let (address, already_verified) = match ident {
        open4x_accounts::Identity::Email { address, verified, .. } => {
            (address.clone(), *verified)
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "not_an_email_identity",
                    message: Some("verify-start only applies to email identities".into()),
                }),
            )
                .into_response();
        }
    };
    if already_verified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: "already_verified",
                message: None,
            }),
        )
            .into_response();
    }

    // Mint + mail. Reuses the same MagicLinkSigner the sign-in path
    // does, so the existing /auth/email/verify endpoint consumes
    // the nonce and runs the verified-flag flip via
    // `mark_email_verified`.
    let minted = match state
        .signer
        .mint_and_record(
            &state.pool,
            &address,
            open4x_accounts::magic_link::DEFAULT_TTL,
        )
        .await
    {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "mint_failed",
                    message: Some(e.to_string()),
                }),
            )
                .into_response();
        }
    };
    let base = if state.public_base_url.is_empty() {
        String::new()
    } else {
        state.public_base_url.trim_end_matches('/').to_string()
    };
    let link = format!("{base}/api/v1/auth/email/verify?token={token}", token = minted.token);
    if let Err(e) = state.mailer.send_magic_link(&address, &link).await {
        eprintln!("[verify-start] mailer error for {address}: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "mail_failed",
                message: Some(e.to_string()),
            }),
        )
            .into_response();
    }

    let _ = state
        .audit
        .record(NewAuditEvent {
            kind: AuditEventKind::MagicLinkMint,
            player_id: Some(player_id),
            ip: None,
            detail: format!("verify:{identity_id}"),
        })
        .await;

    Json(serde_json::json!({"ok": true, "message": "verify_email_sent"})).into_response()
}
