-- Phase 5 — friends graph.
--
-- Single row per directed (requester, target) pair. Status flips
-- in place: 'pending' (a→b request open), 'accepted' (mutual),
-- 'blocked' (a has blocked b — b never sees the row at all).
--
-- Either party deletes via DELETE; queries from b's POV synthesize
-- the inverse direction at read-time so we don't need duplicate
-- rows.
CREATE TABLE IF NOT EXISTS friends (
    a_player_id TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    b_player_id TEXT NOT NULL REFERENCES accounts(player_id) ON DELETE CASCADE,
    status      TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    PRIMARY KEY (a_player_id, b_player_id)
);

CREATE INDEX IF NOT EXISTS idx_friends_b ON friends(b_player_id);
CREATE INDEX IF NOT EXISTS idx_friends_status ON friends(status);
