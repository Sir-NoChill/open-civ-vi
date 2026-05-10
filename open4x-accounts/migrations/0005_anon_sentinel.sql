-- Phase 6 — GDPR foreign-membership anonymisation.
--
-- Reserve a single sentinel `accounts` row that the
-- `delete_account` cascade repoints foreign `game_members`
-- rows to before deleting the original account. Without this,
-- the `ON DELETE CASCADE` from `0002_games.sql` would silently
-- drop the deleted user's membership in *other* people's games
-- — breaking the host's roster and erasing audit-relevant
-- evidence that someone was ever there.
--
-- The sentinel's player_id is the canonical all-zero u64
-- (`0x0000·0000·0000·0000`). The 64-bit collision space against a
-- real generated id is ~2^-64; not zero, but well below any
-- threat model that matters.
--
-- INSERT OR IGNORE so re-running the migration on an existing
-- db (test fixtures, dev resets) is a no-op.
INSERT OR IGNORE INTO accounts (
    player_id, preferred_name, pronouns, bio, prefs_json,
    created_at, updated_at
) VALUES (
    '0000000000000000', '(anonymous)', '', '', '{}',
    '1970-01-01T00:00:00+00:00', '1970-01-01T00:00:00+00:00'
);
