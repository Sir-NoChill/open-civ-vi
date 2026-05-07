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
- [ ] Phase 5 — Cleanup
  - [ ] Delete or move open4x-webui/
  - [ ] Drop /api/game/* legacy routes + server/reports.rs
  - [ ] Integration tests in open4x-server/tests/rest_api.rs
  - [ ] Document API in book/src/multiplayer/web-client.md
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
