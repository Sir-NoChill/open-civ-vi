# Legacy wireframe (archived)

This directory holds the original vanilla-HTML/JS wireframe that was used as
the visual reference when porting the UI to Leptos. **It is not built and
not loaded by any code path.**

The live web client is the `open4x-server` crate compiled with the `csr`
feature against the `wasm32-unknown-unknown` target. The REST API surface
that backs it lives under `/api/v1/*` — see
[`book/src/multiplayer/web-client.md`](../../book/src/multiplayer/web-client.md)
for the full reference and
[`book/src/roadmap/web-ui.md`](../../book/src/roadmap/web-ui.md) for the
phased plan that drove the port.

## Why keep it?

- The JSON files (`*.json`) document the data shapes the wire types in
  `open4x-server/src/types/web.rs` mirror. If we extend a screen, this is
  the design reference.
- The HTML is a working visual mock. When porting a new tab, screenshot it
  for parity.
- Lifting & shifting it elsewhere (as a static screenshot or stripped
  Markdown) is a follow-up task once we no longer need the JSON shapes.

## Mapping wireframe → REST

Every `fetch("foo.json")` in the wireframe maps 1:1 to a `/api/v1/*` GET. A
table of correspondences lives in `HANDOFF.md` (kept verbatim from the
design session) and in
[`book/src/roadmap/web-ui.md`](../../book/src/roadmap/web-ui.md) §2.1.
