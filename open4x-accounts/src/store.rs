//! Persistence layer — `AccountStore` trait + the sqlite-backed
//! `SqliteAccountStore` implementation.
//!
//! Gated on the `persistence` Cargo feature; type-only consumers
//! (the lobby's csr/wasm build) don't drag sqlx in.
//!
//! Phase 2.1 of `book/src/roadmap/accounts-and-login.md`. Magic-link
//! minting / OIDC client / atproto resolver / session token issuance
//! land in 2.2-2.5; this module just owns the durable rows.

#![cfg(feature = "persistence")]

use std::path::Path;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use ulid::Ulid;

use crate::{Account, Identity, PlayerId, Preferences};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage backend: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("identity already linked to a different account")]
    IdentityConflict,
    #[error("account not found")]
    NotFound,
    #[error("invalid input: {0}")]
    Invalid(&'static str),
    #[error("serialization: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type StoreResult<T> = Result<T, StoreError>;

/// Persistence interface used by the lobby HTTP layer (Phase 3) and
/// by the auth-flow runtimes (Phase 2.2 onwards). Implemented today by
/// [`SqliteAccountStore`] and the in-memory `MemAccountStore` test
/// double.
#[async_trait]
pub trait AccountStore: Send + Sync {
    /// Resolve the account that owns this identity, if any. Used at
    /// the tail of every sign-in flow.
    async fn lookup_by_identity(&self, identity: &Identity) -> StoreResult<Option<Account>>;

    /// Sign-in landing point: if the identity is already linked, return
    /// its account; otherwise mint a fresh `PlayerId`, create an
    /// `accounts` row, and link the identity. Idempotent — repeated
    /// calls with the same identity return the same account.
    async fn find_or_create_account_for_identity(
        &self,
        identity: Identity,
    ) -> StoreResult<Account>;

    /// Add a new identity to an existing account. Refused with
    /// `IdentityConflict` if the (kind, primary_key) is already linked
    /// elsewhere.
    async fn link_identity(&self, player_id: PlayerId, identity: Identity) -> StoreResult<()>;

    /// Remove an identity. The account is left intact even if this was
    /// the last identity — the lobby decides whether to refuse the
    /// final unlink.
    async fn unlink_identity(&self, player_id: PlayerId, identity_id: &str) -> StoreResult<()>;

    /// Patch the editable profile fields. `None` leaves a field alone.
    async fn update_profile(
        &self,
        player_id: PlayerId,
        preferred_name: Option<String>,
        pronouns: Option<String>,
        bio: Option<String>,
        prefs: Option<Preferences>,
    ) -> StoreResult<Account>;

    /// Hard delete: cascades sessions + identities. Lobby Phase 6 GDPR
    /// path.
    async fn delete_account(&self, player_id: PlayerId) -> StoreResult<()>;
}

// ───────────────────────────── Sqlite impl ────────────────────────────────────

/// Sqlite-backed `AccountStore`. Default for self-host deploys; tests
/// can use [`MemAccountStore`].
pub struct SqliteAccountStore {
    pool: Pool<Sqlite>,
}

impl SqliteAccountStore {
    /// Open or create a sqlite database at `path` and run the embedded
    /// migrations to bring it to the current schema.
    pub async fn connect(path: impl AsRef<Path>) -> StoreResult<Self> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    /// Run any pending migrations under `open4x-accounts/migrations/`.
    async fn migrate(&self) -> StoreResult<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl AccountStore for SqliteAccountStore {
    async fn lookup_by_identity(&self, identity: &Identity) -> StoreResult<Option<Account>> {
        let (kind, primary_key) = identity_key(identity);
        let row: Option<IdentityRow> = sqlx::query_as::<_, IdentityRow>(
            "SELECT id, player_id, kind, primary_key, label, is_primary, verified \
             FROM identities WHERE kind = ?1 AND primary_key = ?2",
        )
        .bind(kind)
        .bind(primary_key)
        .fetch_optional(&self.pool)
        .await?;

        let Some(idr) = row else { return Ok(None) };
        load_account(&self.pool, &idr.player_id).await.map(Some)
    }

    async fn find_or_create_account_for_identity(
        &self,
        identity: Identity,
    ) -> StoreResult<Account> {
        if let Some(existing) = self.lookup_by_identity(&identity).await? {
            return Ok(existing);
        }
        // No existing account — mint a new one and link in a single
        // transaction.
        let mut tx = self.pool.begin().await?;
        let now = Utc::now().to_rfc3339();
        let player_id = PlayerId::new(rand_player_id());
        let player_id_text = player_id_text(&player_id);
        sqlx::query(
            "INSERT INTO accounts (player_id, preferred_name, pronouns, bio, \
                                   prefs_json, created_at, updated_at) \
             VALUES (?1, '', '', '', '{}', ?2, ?2)",
        )
        .bind(&player_id_text)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        insert_identity(&mut tx, &player_id_text, &identity, &now).await?;
        tx.commit().await?;

        load_account(&self.pool, &player_id_text).await
    }

    async fn link_identity(&self, player_id: PlayerId, identity: Identity) -> StoreResult<()> {
        let now = Utc::now().to_rfc3339();
        let player_id_text = player_id_text(&player_id);
        let mut tx = self.pool.begin().await?;
        let result = insert_identity(&mut tx, &player_id_text, &identity, &now).await;
        match result {
            Ok(_) => {
                tx.commit().await?;
                Ok(())
            }
            Err(StoreError::Sqlx(sqlx::Error::Database(db))) if is_unique_violation(&*db) => {
                Err(StoreError::IdentityConflict)
            }
            Err(other) => Err(other),
        }
    }

    async fn unlink_identity(&self, player_id: PlayerId, identity_id: &str) -> StoreResult<()> {
        let player_id_text = player_id_text(&player_id);
        let res = sqlx::query("DELETE FROM identities WHERE id = ?1 AND player_id = ?2")
            .bind(identity_id)
            .bind(&player_id_text)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn update_profile(
        &self,
        player_id: PlayerId,
        preferred_name: Option<String>,
        pronouns: Option<String>,
        bio: Option<String>,
        prefs: Option<Preferences>,
    ) -> StoreResult<Account> {
        let player_id_text = player_id_text(&player_id);
        let now = Utc::now().to_rfc3339();
        let prefs_json = match &prefs {
            Some(p) => Some(serde_json::to_string(p)?),
            None => None,
        };
        sqlx::query(
            "UPDATE accounts \
             SET preferred_name = COALESCE(?1, preferred_name), \
                 pronouns       = COALESCE(?2, pronouns), \
                 bio            = COALESCE(?3, bio), \
                 prefs_json     = COALESCE(?4, prefs_json), \
                 updated_at     = ?5 \
             WHERE player_id = ?6",
        )
        .bind(preferred_name)
        .bind(pronouns)
        .bind(bio)
        .bind(prefs_json)
        .bind(&now)
        .bind(&player_id_text)
        .execute(&self.pool)
        .await?;
        load_account(&self.pool, &player_id_text).await
    }

    async fn delete_account(&self, player_id: PlayerId) -> StoreResult<()> {
        let player_id_text = player_id_text(&player_id);
        let res = sqlx::query("DELETE FROM accounts WHERE player_id = ?1")
            .bind(&player_id_text)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }
}

// ───────────────────────────── Helpers ────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct IdentityRow {
    id: String,
    player_id: String,
    kind: String,
    primary_key: String,
    label: String,
    is_primary: i64,
    verified: i64,
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    player_id: String,
    preferred_name: String,
    pronouns: String,
    bio: String,
    prefs_json: String,
}

fn identity_key(identity: &Identity) -> (&'static str, String) {
    match identity {
        Identity::Email { address, .. } => ("email", address.to_lowercase()),
        Identity::OpenId { issuer, subject, .. } => ("oidc", format!("{issuer}|{subject}")),
        Identity::Atproto { did, .. } => ("atproto", did.clone()),
    }
}

fn rand_player_id() -> u64 {
    // Derive a u64 from a fresh ULID so the IdGenerator path stays
    // single-source (ULIDs are seeded with rand under the hood).
    let raw = Ulid::new().0 as u64;
    raw
}

fn player_id_text(id: &PlayerId) -> String {
    format!("{:016X}", id.0)
}

fn parse_player_id_text(s: &str) -> Result<PlayerId, StoreError> {
    let raw = u64::from_str_radix(s, 16).map_err(|_| StoreError::Invalid("player_id"))?;
    Ok(PlayerId::new(raw))
}

fn is_unique_violation(err: &(dyn sqlx::error::DatabaseError)) -> bool {
    err.code()
        .map(|c| c == "2067" /* SQLITE_CONSTRAINT_UNIQUE */ || c == "1555" /* SQLITE_CONSTRAINT_PRIMARYKEY */)
        .unwrap_or(false)
        || err.message().contains("UNIQUE")
}

async fn insert_identity(
    tx: &mut sqlx::SqliteConnection,
    player_id_text: &str,
    identity: &Identity,
    now: &str,
) -> StoreResult<()> {
    let id = Ulid::new().to_string();
    let (kind, primary_key) = identity_key(identity);
    let label = identity.label();
    let (is_primary, verified) = match identity {
        Identity::Email { primary, verified, .. } => (*primary as i64, *verified as i64),
        _ => (0, 1),
    };
    sqlx::query(
        "INSERT INTO identities (id, player_id, kind, primary_key, label, \
                                 is_primary, verified, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&id)
    .bind(player_id_text)
    .bind(kind)
    .bind(&primary_key)
    .bind(&label)
    .bind(is_primary)
    .bind(verified)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn load_account(pool: &Pool<Sqlite>, player_id_text: &str) -> StoreResult<Account> {
    let row: AccountRow = sqlx::query_as::<_, AccountRow>(
        "SELECT player_id, preferred_name, pronouns, bio, prefs_json \
         FROM accounts WHERE player_id = ?1",
    )
    .bind(player_id_text)
    .fetch_optional(pool)
    .await?
    .ok_or(StoreError::NotFound)?;
    let prefs: Preferences = serde_json::from_str(&row.prefs_json).unwrap_or_default();
    let identities = load_identities(pool, &row.player_id).await?;
    let player_id = parse_player_id_text(&row.player_id)?;
    Ok(Account {
        player_id,
        preferred_name: row.preferred_name,
        pronouns: row.pronouns,
        bio: row.bio,
        identities,
        prefs,
    })
}

async fn load_identities(pool: &Pool<Sqlite>, player_id_text: &str) -> StoreResult<Vec<Identity>> {
    let rows: Vec<IdentityRow> = sqlx::query_as::<_, IdentityRow>(
        "SELECT id, player_id, kind, primary_key, label, is_primary, verified \
         FROM identities WHERE player_id = ?1",
    )
    .bind(player_id_text)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(row_to_identity(&r));
    }
    Ok(out)
}

fn row_to_identity(r: &IdentityRow) -> Identity {
    match r.kind.as_str() {
        "email" => Identity::Email {
            address: r.primary_key.clone(),
            verified: r.verified != 0,
            primary: r.is_primary != 0,
        },
        "oidc" => {
            let (issuer, subject) = r
                .primary_key
                .split_once('|')
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .unwrap_or_else(|| (r.primary_key.clone(), String::new()));
            Identity::OpenId {
                issuer,
                subject,
                label: r.label.clone(),
            }
        }
        "atproto" => Identity::Atproto {
            did: r.primary_key.clone(),
            handle: r.label.clone(),
        },
        _ => Identity::Email {
            address: r.primary_key.clone(),
            verified: r.verified != 0,
            primary: r.is_primary != 0,
        },
    }
}

// ───────────────────────────── In-memory test impl ────────────────────────────

#[cfg(test)]
mod mem {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    pub struct MemAccountStore {
        accounts: Mutex<HashMap<PlayerId, Account>>,
    }

    impl Default for MemAccountStore {
        fn default() -> Self {
            Self {
                accounts: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl AccountStore for MemAccountStore {
        async fn lookup_by_identity(&self, identity: &Identity) -> StoreResult<Option<Account>> {
            let key = identity_key(identity);
            let map = self.accounts.lock().unwrap();
            for acct in map.values() {
                for id in &acct.identities {
                    if identity_key(id) == key {
                        return Ok(Some(acct.clone()));
                    }
                }
            }
            Ok(None)
        }

        async fn find_or_create_account_for_identity(
            &self,
            identity: Identity,
        ) -> StoreResult<Account> {
            if let Some(a) = self.lookup_by_identity(&identity).await? {
                return Ok(a);
            }
            let player_id = PlayerId::new(rand_player_id());
            let acct = Account {
                player_id,
                identities: vec![identity],
                ..Account::default()
            };
            self.accounts.lock().unwrap().insert(player_id, acct.clone());
            Ok(acct)
        }

        async fn link_identity(&self, player_id: PlayerId, identity: Identity) -> StoreResult<()> {
            // Conflict if another account already has this identity.
            if let Some(other) = self.lookup_by_identity(&identity).await? {
                if other.player_id != player_id {
                    return Err(StoreError::IdentityConflict);
                }
                return Ok(());
            }
            let mut map = self.accounts.lock().unwrap();
            let acct = map.get_mut(&player_id).ok_or(StoreError::NotFound)?;
            acct.identities.push(identity);
            Ok(())
        }

        async fn unlink_identity(
            &self,
            player_id: PlayerId,
            _identity_id: &str,
        ) -> StoreResult<()> {
            // The mem store doesn't carry stable identity IDs; tests use
            // direct field manipulation.
            let mut map = self.accounts.lock().unwrap();
            map.get_mut(&player_id).ok_or(StoreError::NotFound)?;
            Ok(())
        }

        async fn update_profile(
            &self,
            player_id: PlayerId,
            preferred_name: Option<String>,
            pronouns: Option<String>,
            bio: Option<String>,
            prefs: Option<Preferences>,
        ) -> StoreResult<Account> {
            let mut map = self.accounts.lock().unwrap();
            let acct = map.get_mut(&player_id).ok_or(StoreError::NotFound)?;
            if let Some(v) = preferred_name {
                acct.preferred_name = v;
            }
            if let Some(v) = pronouns {
                acct.pronouns = v;
            }
            if let Some(v) = bio {
                acct.bio = v;
            }
            if let Some(v) = prefs {
                acct.prefs = v;
            }
            Ok(acct.clone())
        }

        async fn delete_account(&self, player_id: PlayerId) -> StoreResult<()> {
            self.accounts
                .lock()
                .unwrap()
                .remove(&player_id)
                .ok_or(StoreError::NotFound)
                .map(|_| ())
        }
    }

    #[tokio::test]
    async fn find_or_create_is_idempotent() {
        let store = MemAccountStore::default();
        let id = Identity::Email {
            address: "alice@example.com".into(),
            verified: false,
            primary: true,
        };
        let a = store
            .find_or_create_account_for_identity(id.clone())
            .await
            .unwrap();
        let b = store
            .find_or_create_account_for_identity(id)
            .await
            .unwrap();
        assert_eq!(a.player_id, b.player_id);
    }

    #[tokio::test]
    async fn link_identity_rejects_cross_account_dup() {
        let store = MemAccountStore::default();
        let alice = Identity::Email {
            address: "alice@example.com".into(),
            verified: false,
            primary: true,
        };
        let bob = Identity::Email {
            address: "bob@example.com".into(),
            verified: false,
            primary: true,
        };
        let _a = store
            .find_or_create_account_for_identity(alice.clone())
            .await
            .unwrap();
        let b = store
            .find_or_create_account_for_identity(bob)
            .await
            .unwrap();
        let err = store.link_identity(b.player_id, alice).await.unwrap_err();
        assert!(matches!(err, StoreError::IdentityConflict));
    }
}
