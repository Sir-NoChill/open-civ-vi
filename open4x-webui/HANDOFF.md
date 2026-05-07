# 4X Wireframe — Backend Wiring Handoff

## Overview

This project is a fully interactive HTML wireframe for a 4X strategy game UI.
Every screen is driven by local JSON mock files. The job of the backend integration
is to replace each `fetch("some-file.json")` call with a fetch to the equivalent
REST endpoint documented in `api-manifest.json`.

No frontend restructuring is required — the data shapes are already stable and
documented with `$schema` / `$doc` headers in every JSON file.

---

## File Map

| File | Schema | REST endpoint | Screen(s) |
|---|---|---|---|
| `player-state.json` | `player-state/v1` | `GET /player-state` | HUD resource bar (all screens) |
| `world-snapshot.json` | `world-snapshot/v1` | `GET /world/snapshot` | Main HUD hex map |
| `map-overlays.json` | `map-overlays/v1` | `GET /map/overlays` | HUD → Map drawer pane |
| `unit-data.json` | `unit-data/v1` | `GET /units` | Unit/Army screen, HUD unit drawer |
| `army-data.json` | `army-data/v1` | `GET /armies` | Unit/Army screen right panel |
| `city-data.json` | `city-data/v1` | `GET /cities` | City screen |
| `city-tiles.json` | `city-tiles/v1` | `GET /cities/:id/tiles` | City hex view |
| `notifications.json` | `notifications/v1` | `GET /notifications` | HUD → Notifications drawer |
| `turn-queue.json` | `turn-queue/v1` | `GET /turn-queue` | HUD → Turn Queue drawer |
| `tech-tree.json` | `tech-tree/v1` | `GET /tech` | Tech Tree screen + Research drawer |
| `civics-tree.json` | `civics-tree/v1` | `GET /civics` | Civics Tree screen + Civics drawer |
| `government-policies.json` | `government-policies/v1` | `GET /government` | Government screen |
| `diplomacy.json` | `diplomacy/v1` | `GET /diplomacy` | Diplomacy screen |
| `empire-overview.json` | `empire-overview/v1` | `GET /empire/overview` | Empire Overview screen |
| `victory.json` | `victory/v1` | `GET /victory` | Victory screen |

---

## How Each Screen Loads Data

Each screen has an `async load*()` function in the HTML `<script>` block or a
companion `.js` file. Each follows this pattern:

```js
async function loadFoo(url = "foo.json") {
  let data;
  try {
    const r = await fetch(url);
    if (!r.ok) throw 0;
    data = await r.json();
  } catch(e) {
    data = { /* minimal empty fallback */ };
  }
  renderFoo(data);
}
```

**To wire to the backend:** change the default `url` parameter to the REST
endpoint path, or call `loadFoo("/api/v1/foo")` from the `DOMContentLoaded`
handler in the HTML.

All `load*()` calls are in the `DOMContentLoaded` block near line 1820 of
`4X Wireframes.html`:

```js
window.addEventListener("DOMContentLoaded", () => {
  initHexWorld();
  loadPlayerState();      // → GET /player-state
  loadUnitData();         // → GET /units
  loadCityData();         // → GET /cities
  loadCityTiles();        // → GET /cities (tiles embedded) or GET /cities/:id/tiles
  initNotifications();    // → calls loadNotifications() → GET /notifications
  initDrawer();
  loadTechCivicData();    // → GET /tech + GET /civics
  loadTurnQueue();        // → GET /turn-queue
  loadDiplomacy();        // → GET /diplomacy
  loadEmpireOverview();   // → GET /empire/overview
  loadVictory();          // → GET /victory
  loadGovernmentPolicies(); // → GET /government
  loadArmyData();         // → GET /armies
  loadMapOverlays();      // → GET /map/overlays
});
```

---

## Mutation Endpoints

The following user actions currently have no backend wiring. Each is annotated
below with the endpoint it should call:

| Action | Current behaviour | Target endpoint |
|---|---|---|
| End Turn button | Visual only | `POST /turn/end` |
| Research now / queue | Mutates `window.TECH_DATA` in memory | `POST /tech/research`, `POST /tech/queue`, `DELETE /tech/queue/:id` |
| Civic now / queue | Mutates `window.CIVIC_DATA` in memory | `POST /civics/research`, `POST /civics/queue`, `DELETE /civics/queue/:id` |
| Production queue add/remove | Mutates city object in memory | `POST /cities/:id/production`, `DELETE /cities/:id/production/:pos` |
| Citizen focus chips | Mutates city object in memory | `POST /cities/:id/citizens` |
| Rename city | Prompts and mutates DOM | `PATCH /cities/:id/rename` |
| Unit actions (Move, Attack, …) | Button renders only, no dispatch | `POST /units/:id/action` |
| Dismiss notification | Removes from in-memory array | `DELETE /notifications/:id` |
| Clear all notifications | Clears in-memory array | `DELETE /notifications` |
| Skip turn queue item | Filters in-memory array | `POST /turn-queue/:id/skip` |
| Skip all skippable | Filters in-memory array | `POST /turn-queue/:id/skip` (batch) |
| Propose deal | Button renders only | `POST /diplomacy/deal/propose` |
| Diplomacy actions (Declare War, etc.) | Buttons render only | `POST /diplomacy/civs/:id/action` |
| Government change | Mutates `GOVT_DATA` in memory | `POST /government/change` |
| Confirm policies | Clears pending flag in memory | `PUT /government/policies` |
| Overlay toggle chips | Mutates `MAP_OVERLAYS_DATA` in memory | `POST /map/overlays/:id/toggle` |

---

## Data Shape Notes

### Hex coordinates
All tile positions use **axial (q, r) coordinates** (pointy-top hexagons).
The world wraps cylindrically in X (`world.wrapX = true`). The renderer
normalises `q` to `[0, world.width)` as `wq`.

### Tech / Civic status field
`status` is a view-model field computed server-side:
- `"done"` — research complete
- `"current"` — actively being researched (has `progress` field)
- `"available"` — prerequisites met, not yet started
- `"locked"` — prerequisites not met

### Policy slots
`government.slots` counts total slots per type. The `active_policies[]` array
holds currently equipped cards. The frontend enforces slot limits client-side
for immediate feedback; the server must validate on `PUT /government/policies`.

### Turn queue `required` flag
Items with `required: true` block `POST /turn/end`. The server should return
HTTP 400 with `{ error: "unresolved_required_actions", items: [...] }` if the
client attempts to end turn with required items pending.

### Notifications `target`
`target.screen` drives tab navigation; `target.q` + `target.r` drives camera
jump to a hex. Null target = no navigation (informational only).

---

## Suggested Integration Order

1. **Player state** — resource bar visible on all screens, high value immediately.
2. **World snapshot** — hex map is the centrepiece; load tiles from server.
3. **Units + Turn queue** — unit actions and end-turn flow are core gameplay.
4. **Cities** — production queue and citizen management.
5. **Notifications** — event feed driven by server-side turn events.
6. **Tech / Civics / Government** — research and policy management.
7. **Diplomacy** — deal flow and relation modifiers.
8. **Empire overview + Victory** — read-only dashboards, lowest priority.

---

## Polling Strategy

The frontend does not currently poll. For a real-time or hot-seat multiplayer
game, consider:

- **Short-poll** `GET /notifications` every 5s during the player's turn.
- **WebSocket** push for multiplayer: server emits `turn_started`, `unit_attacked`,
  `city_captured` events; frontend re-fetches the relevant endpoint on receipt.
- All `load*()` functions are idempotent and safe to call repeatedly.

---

## Tech Stack Notes

- Pure HTML + vanilla JS. No build step, no framework.
- Companion scripts: `wireframe.js`, `government.js`, `unit-screen.js`, `map-overlay.js`
- All scripts are loaded as plain `<script src>` tags at the bottom of `4X Wireframes.html`.
- `window.TECH_DATA`, `window.CIVIC_DATA`, `window.RESEARCH_QUEUE`, `window.CIVIC_QUEUE`
  are intentionally global so the tech-tree HTML nodes and the drawer list share state.
- The hex renderer (`initHexWorld`, `render`) is self-contained and only reads `WORLD` /
  `TILES` globals populated by `loadWorld()`.
