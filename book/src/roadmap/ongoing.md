# Ongoing Work

> **Current task**: Web UI port — post-plan extensions
> **Plan**: [book/src/roadmap/web-ui.md](./web-ui.md)
> **Status**: All 5 phases of §7 complete. Working through the post-plan
> backlog below. This file is a running log — each cron tick should pick the
> next unchecked item, land a single conventional commit, then update this
> file.
> **Cron**: `04ad50ff` — fires hourly at :13 (session-only, expires
> after 7 days). Cancel with `CronDelete 04ad50ff` when the backlog is
> empty or no longer wanted.

## Web UI port (Leptos + REST)

- [x] Phase 0 — Scaffolding
- [x] Phase 1 — HUD MVP (HexMap rebind deferred)
- [x] Phase 2 — server reads + writes (libciv extensions deferred)
- [x] Phase 3 — server tech/civics surface (libciv extensions + tabs deferred)
- [x] Phase 4 — server outer-loop reads + NotificationRecord ring buffer
      (libciv pending_actions / victory_progress + tabs/drawers deferred)
- [x] Phase 5 — Cleanup
  - [x] Move open4x-webui/ → docs/legacy-wireframe/
  - [x] Drop /api/game/* legacy routes (server/{api,reports}.rs
        modules retained as dead code; full deletion below)
  - [x] Integration tests in open4x-server/tests/rest_api.rs (10 tests)
  - [x] Document API in book/src/multiplayer/web-client.md

## Remaining (post-plan)

These items were deferred from earlier phases. Pick the top unchecked
item next.

### libciv RulesEngine extensions

Strategy: where it fits, expose the new method through `open4x-cli`'s
`status` / `list` subcommand first (cheap testing arena), then plumb
into the server projector.

- [x] `pending_actions(gs, civ) -> Vec<PendingAction>` — replaced the
      hand-rolled "choose_research" check; new
      `build_turn_queue_from_room` calls into the engine and surfaces
      `choose_research` / `choose_civic` (required) plus per-unit and
      per-city advisory items. Exposed via CLI `status pending`.
- [ ] `victory_progress(gs) -> Vec<VictoryProgress>` — replaces the
      `player_pct=0` placeholder in `build_victory`.
- [ ] `available_unit_actions(gs, unit) -> Vec<UnitAction>` — replaces
      the hardcoded action set in `web_projection::build_units`.
- [ ] `preview_combat(gs, attacker, defender_coord) -> CombatPreview` —
      replaces the heuristic in `build_combat_preview`.
- [ ] `policy_catalogue(gs, civ) -> Vec<PolicyCardEntry>` — lets
      `build_government` populate the catalogue.

### GameAction variants + REST mutations

- [ ] `GameAction::AssignCityFocus` + `POST /cities/{id}/focus`
- [ ] `GameAction::RenameCity` + `POST /cities/{id}/rename`
- [ ] `GameAction::CancelResearch` + `DELETE /tech/research`
- [ ] `GameAction::CancelCivic` + `DELETE /civics/research`
- [ ] `GameAction::ChangeGovernment` + `POST /government/change`

### Client (Leptos)

- [ ] HexMap refactor to consume `WorldSnapshot` directly (currently
      RestGamePage shows a placeholder "World w × h · N tiles" line)
- [ ] `tabs/city.rs` — extend existing, REST-driven
- [ ] `tabs/units.rs` — new
- [ ] `tabs/government.rs` — new
- [ ] `tabs/diplomacy.rs` — new
- [ ] `tabs/empire.rs` — new
- [ ] `tabs/victory.rs` — new
- [ ] drawers: notifications / turn-queue / overlays

### Cleanup

- [ ] Full deletion of `server/{api,reports}.rs` and `types/reports.rs`

## Civsim Non-REPL CLI — ALL 5 PHASES COMPLETE

- [x] Phase 0–5 (see git log).
- 554 tests, 0 failures.

## Changelog (post-plan)

Most recent first. Each entry: `<jj change short> — <subject>`.

- `lrrxwtmv` — feat(libciv): RulesEngine::pending_actions + wire
  through web turn-queue + CLI `status pending`.
- `qrykmkqp` — feat(open4x-server): NotificationRecord ring buffer +
  DELETE handlers (post-plan).
