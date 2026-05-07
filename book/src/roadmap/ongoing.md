# Ongoing Work

> **Current task**: Web UI port — Phase 0 (scaffolding)
> **Plan**: [book/src/roadmap/web-ui.md](./web-ui.md)
> **Status**: In progress

## Web UI port (Leptos + REST)

- [ ] Phase 0 — Scaffolding
  - [ ] `types/web.rs` skeleton
  - [ ] `server/web_projection.rs` stubs
  - [ ] `server/rest/` module + `/api/v1/health`
  - [ ] `components/api/` with `http.rs` + stub
  - [ ] trunk build instructions in `book/getting-started.md`
- [ ] Phase 1 — HUD MVP
- [ ] Phase 2 — Cities + Units
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
