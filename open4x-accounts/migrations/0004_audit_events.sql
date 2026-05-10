-- Phase 6: append-only audit log for auth + account events.
--
-- Used for incident response and the Phase 6 'lobby db dump' subcommand.
-- Never updated, never deleted by user-facing code paths; retention is a
-- separate Phase 6 task (e.g. 'lobby db prune --older-than 90d').

CREATE TABLE IF NOT EXISTS audit_events (
    id          TEXT PRIMARY KEY,    -- ULID
    ts          TEXT NOT NULL,
    kind        TEXT NOT NULL,       -- 'sign_in' | 'sign_in_failed' | 'sign_out' | …
    player_id   TEXT,                -- nullable for failed sign-ins where we don't know who
    ip          TEXT,                -- request IP if extracted; nullable for CLI / internal events
    detail      TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_audit_events_ts        ON audit_events(ts);
CREATE INDEX IF NOT EXISTS idx_audit_events_player_id ON audit_events(player_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_kind      ON audit_events(kind);
