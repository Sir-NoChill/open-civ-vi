# Ongoing Work

> **Current task**: Web UI port — client tabs + libciv extensions
> **Plan**: [book/src/roadmap/web-ui.md](./web-ui.md)
> **Status**: In progress (server-side reads/writes for all 5 phases done)

## Web UI port (Leptos + REST)

- [x] Phase 0 — Scaffolding
- [x] Phase 1 — HUD MVP (HexMap rebind deferred)
- [~] Phase 2 — server done; libciv extensions (preview_combat,
      available_unit_actions, AssignCityFocus, RenameCity) + tabs/city.rs +
      tabs/units.rs + HexMap refactor pending
- [~] Phase 3 — server done; CancelResearch/CancelCivic/ChangeGovernment +
      tabs/science.rs + tabs/culture.rs + tabs/government.rs + libciv
      policy_catalogue pending
- [~] Phase 4 — server reads done; libciv pending_actions / victory_progress
      + NotificationRecord ring buffer + tabs/{diplomacy,empire,victory}.rs
      + drawers + 400-on-required-turn-end pending
- [x] Phase 5 — Cleanup
  - [x] Move open4x-webui/ → docs/legacy-wireframe/
  - [x] Drop /api/game/* legacy routes (server/{api,reports}.rs
        modules retained as dead code; full deletion is a follow-up)
  - [x] Integration tests in open4x-server/tests/rest_api.rs (10 tests)
  - [x] Document API in book/src/multiplayer/web-client.md

## Remaining (post-plan)

These items were deferred from earlier phases and are still open:

- libciv extensions
  - `RulesEngine::available_unit_actions` (replaces hardcoded action set
    in `web_projection::build_units`)
  - `RulesEngine::preview_combat` (replaces the heuristic
    `build_combat_preview`)
  - `RulesEngine::pending_actions` (replaces `build_turn_queue`'s
    hand-rolled "choose_research" check)
  - `RulesEngine::victory_progress` (replaces the placeholder
    `player_pct=0` in `build_victory`)
  - `RulesEngine::policy_catalogue` (lets `build_government` populate
    the catalogue array)
  - `GameAction::AssignCityFocus`, `RenameCity`, `CancelResearch`,
    `CancelCivic`, `ChangeGovernment` (and matching REST mutations)
- `NotificationRecord` ring buffer on `GameRoom`, populated from
  `advance_turn` deltas; `build_notifications` reads from it
- HexMap refactor to consume `WorldSnapshot` directly
- Per-tab Leptos ports
  - `tabs/city.rs` (extend existing) → REST-driven
  - `tabs/units.rs` (new)
  - `tabs/government.rs` (new)
  - `tabs/diplomacy.rs` (new)
  - `tabs/empire.rs` (new)
  - `tabs/victory.rs` (new)
  - drawers: notifications / turn-queue / overlays
- Full deletion of `server/{api,reports}.rs` and `types/reports.rs`
- [ ] Phase 3 — Research & policy stacks
- [ ] Phase 4 — Outer loop screens
- [ ] Phase 5 — Cleanup

## Civsim Non-REPL CLI — ALL 5 PHASES COMPLETE

- [x] Phase 0: libciv foundation (serde on StateDelta, turn_done/player_config, save_load fix)
- [x] Phase 1: Infrastructure (state_io, output, player_view)
- [x] Phase 2: CLI definition (42 actions, 9 status, 8 list types)
- [x] Phase 3: Handlers (new_game, action, end_turn, view, status, list)
- [x] Phase 4: Integration (main.rs dispatch, legacy modes preserved)
- [x] Phase 5: Tests (6 CLI integration tests)

554 tests, 0 failures.
