-- Phase 5 — user-saved wizard presets.
--
-- One row per saved configuration. `body_json` is opaque to the
-- accounts crate; the SPA serializes its `WizardState` shape and
-- echoes it back at load time. Schema is intentionally minimal
-- (no FTS, no tags) — we'll grow the surface as the UI does.
CREATE TABLE IF NOT EXISTS presets (
    id          TEXT PRIMARY KEY,
    player_id   TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    body_json   TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presets_player ON presets(player_id);
