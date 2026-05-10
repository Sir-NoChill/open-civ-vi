-- Phase 4.1 of book/src/roadmap/accounts-and-login.md.
--
-- Tracks games owned by accounts on this lobby (one row per game)
-- plus the per-game member list (who's invited / playing / waiting).
-- The `server_url` + `server_token` columns store the
-- shared-server-multi-room handle the orchestrator hands out on
-- POST /api/v1/games — see Phase 4.3.
--
-- Soft-delete via `deleted_at`; the `status` enum encodes the
-- runtime view ('your_turn' / 'waiting' / 'completed' / 'archived').

CREATE TABLE IF NOT EXISTS games (
    game_id          TEXT PRIMARY KEY,    -- ULID
    owner_player_id  TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,

    -- Display metadata pulled from the new-game wizard.
    name             TEXT NOT NULL DEFAULT '',
    leader           TEXT NOT NULL DEFAULT '',
    civ_id           TEXT NOT NULL DEFAULT '',
    difficulty       TEXT NOT NULL DEFAULT 'prince',
    players_human    INTEGER NOT NULL DEFAULT 1,
    players_ai       INTEGER NOT NULL DEFAULT 0,

    -- World-generation params (stored for the Resume / re-spawn flow).
    map_type         TEXT NOT NULL DEFAULT 'continents',
    map_size         TEXT NOT NULL DEFAULT 'std',
    seed             TEXT NOT NULL DEFAULT '',

    -- Live runtime view, refreshed by the orchestrator.
    turn             INTEGER NOT NULL DEFAULT 0,
    era              TEXT NOT NULL DEFAULT 'Ancient',
    score            INTEGER NOT NULL DEFAULT 0,
    status           TEXT NOT NULL DEFAULT 'waiting',  -- 'your_turn' | 'waiting' | 'completed' | 'archived'

    -- Orchestration handles to reach the in-game server.
    server_url       TEXT NOT NULL DEFAULT '',
    server_token     TEXT NOT NULL DEFAULT '',

    last_played_at   TEXT,
    created_at       TEXT NOT NULL,
    deleted_at       TEXT
);

CREATE INDEX IF NOT EXISTS idx_games_owner       ON games(owner_player_id);
CREATE INDEX IF NOT EXISTS idx_games_status      ON games(status);
CREATE INDEX IF NOT EXISTS idx_games_last_played ON games(last_played_at);

CREATE TABLE IF NOT EXISTS game_members (
    game_id     TEXT NOT NULL REFERENCES games(game_id) ON DELETE CASCADE,
    player_id   TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    role        TEXT NOT NULL DEFAULT 'player', -- 'owner' | 'player' | 'invited' | 'observer'
    invited_at  TEXT NOT NULL,
    joined_at   TEXT,
    PRIMARY KEY (game_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_game_members_player ON game_members(player_id);
