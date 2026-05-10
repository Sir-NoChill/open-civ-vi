-- Phase 4 polish: per-game notes column.
--
-- Plain markdown text the user keeps next to a game ('save coal for
-- railroads — don't burn it on industrial zones yet'). Visible to the
-- owner only today; future shared games may surface a separate
-- shared-notes table.

ALTER TABLE games ADD COLUMN notes TEXT NOT NULL DEFAULT '';
