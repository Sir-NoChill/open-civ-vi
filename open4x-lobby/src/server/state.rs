//! Shared `AppState` plumbed into every Axum handler.

#![cfg(feature = "ssr")]

use std::path::PathBuf;
use std::sync::Arc;

use open4x_accounts::magic_link::MagicLinkSigner;
use open4x_accounts::mailer::{LogMailer, Mailer};
use open4x_accounts::store::SqliteAccountStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
    pub store: Arc<SqliteAccountStore>,
    pub signer: MagicLinkSigner,
    pub mailer: Arc<dyn Mailer>,
    /// Base URL for outgoing magic-link URLs (e.g. `https://lobby.example`).
    /// Empty string means "use the current request's Host header" — fine
    /// for dev / single-machine.
    pub public_base_url: String,
}

impl AppState {
    /// Boot the AppState: open / migrate the sqlite db at `data_dir/accounts.sqlite`,
    /// load (or generate) the HMAC key at `data_dir/lobby.key`, default to
    /// `LogMailer`. The lobby's `main.rs` calls this once at startup.
    pub async fn boot(data_dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let data_dir: PathBuf = data_dir.into();
        std::fs::create_dir_all(&data_dir)?;

        // Open + migrate the sqlite db. We re-open via SqliteAccountStore
        // for the trait surface, but ALSO keep a Pool around for the
        // session/magic-link helpers (they take a `&Pool<Sqlite>` directly).
        let db_path = data_dir.join("accounts.sqlite");
        let opts = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .pragma("foreign_keys", "ON");
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        sqlx::migrate!("../open4x-accounts/migrations")
            .run(&pool)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let store = SqliteAccountStore::connect(&db_path)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let signer = MagicLinkSigner::from_env_or_path(data_dir.join("lobby.key"))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let mailer: Arc<dyn Mailer> = Arc::new(LogMailer);
        let public_base_url =
            std::env::var("OPEN4X_LOBBY_PUBLIC_URL").unwrap_or_default();

        Ok(Self {
            pool,
            store: Arc::new(store),
            signer,
            mailer,
            public_base_url,
        })
    }
}
