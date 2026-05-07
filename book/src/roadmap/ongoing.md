# Ongoing Work

> **Current task**: Web UI port — Phase 2 (Cities + Units), client + libciv
> **Plan**: [book/src/roadmap/web-ui.md](./web-ui.md)
> **Status**: In progress

## Web UI port (Leptos + REST)

- [x] Phase 0 — Scaffolding
- [x] Phase 1 — HUD MVP (HexMap rebind deferred to Phase 2)
- [~] Phase 2 — Cities + Units (server-side reads + mutations done; client + libciv pending)
  - [x] `build_cities` / `build_city_tiles` / `build_units` projectors
  - [x] REST reads for cities, units, combat preview
  - [x] REST writes: production queue, unit move/attack/found_city
  - [ ] New `RulesEngine::available_unit_actions` / `preview_combat`
  - [ ] New `GameAction::AssignCityFocus` / `RenameCity`
  - [ ] `tabs/city.rs` REST-driven port
  - [ ] `tabs/units.rs` (new)
  - [ ] HexMap refactor: take `WorldSnapshot` instead of `GameView`
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
