-- Initial schema for the Open4X accounts substrate.
--
-- Three tables, all keyed by ULID-as-text so they're sortable and
-- compatible with both sqlite and postgres without per-engine type
-- definitions. Timestamps are ISO-8601 strings (sqlite) / TIMESTAMPTZ
-- (postgres) — applied via the runtime helper, not the schema, to
-- keep this migration portable.

CREATE TABLE IF NOT EXISTS accounts (
    player_id        TEXT PRIMARY KEY,    -- u64 packed as 16-hex
    preferred_name   TEXT NOT NULL DEFAULT '',
    pronouns         TEXT NOT NULL DEFAULT '',
    bio              TEXT NOT NULL DEFAULT '',
    prefs_json       TEXT NOT NULL DEFAULT '{}',  -- serde_json'd Preferences
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS identities (
    id               TEXT PRIMARY KEY,    -- ULID
    player_id        TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    kind             TEXT NOT NULL,       -- 'email' | 'oidc' | 'atproto'
    primary_key      TEXT NOT NULL,       -- email addr | iss+sub | did
    label            TEXT NOT NULL DEFAULT '',
    is_primary       INTEGER NOT NULL DEFAULT 0,  -- bool; one per account for kind='email'
    verified         INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL,
    UNIQUE (kind, primary_key)
);

CREATE INDEX IF NOT EXISTS idx_identities_player_id ON identities(player_id);

CREATE TABLE IF NOT EXISTS sessions (
    token_hash       TEXT PRIMARY KEY,    -- SHA-256 hex of the bearer token
    player_id        TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    created_at       TEXT NOT NULL,
    expires_at       TEXT NOT NULL,
    revoked_at       TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_player_id ON sessions(player_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Single-use nonce table for magic-link tokens (Phase 2.2).
CREATE TABLE IF NOT EXISTS magic_link_nonces (
    nonce            TEXT PRIMARY KEY,
    email            TEXT NOT NULL,
    expires_at       TEXT NOT NULL,
    consumed_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_magic_link_nonces_expires ON magic_link_nonces(expires_at);
