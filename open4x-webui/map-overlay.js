// ===== MAP OVERLAYS & MINIMAP =====
// Reads from map-overlays.json (schema: map-overlays/v1)
// Populates the overlay chip strip and minimap in the HUD Map drawer pane.

let MAP_OVERLAYS_DATA = null;

async function loadMapOverlays(url = "map-overlays.json") {
  let data;
  try { const r = await fetch(url); if (!r.ok) throw 0; data = await r.json(); }
  catch(e) { data = null; }
  if (!data) return;
  MAP_OVERLAYS_DATA = data;
  renderOverlayChips();
  renderMinimapPane();
  // update world size label
  const lbl = document.getElementById("map-world-label");
  if (lbl && data.world_size_label) lbl.textContent = data.world_size_label;
  // update header chips
  const own = data.minimap.cities.filter(c => !c.enemy);
  const enemy = data.minimap.cities.filter(c => c.enemy);
  const citiesChip  = document.getElementById("map-cities-chip");
  const enemiesChip = document.getElementById("map-enemies-chip");
  if (citiesChip)  citiesChip.textContent  = own.length + " cities";
  if (enemiesChip) enemiesChip.textContent = enemy.length + " enemies visible";
}

function renderOverlayChips() {
  const container = document.querySelector(
    ".drawer-pane[data-pane='map'] .drawer-col:last-child > div[style*='flex-wrap']"
  );
  if (!container || !MAP_OVERLAYS_DATA) return;
  container.innerHTML = "";
  MAP_OVERLAYS_DATA.overlays.forEach(ov => {
    const chip = document.createElement("span");
    chip.className = "chip" + (ov.active ? " filled" : "");
    chip.textContent = ov.label;
    chip.style.cursor = "pointer";
    chip.onclick = () => {
      ov.active = !ov.active;
      chip.className = "chip" + (ov.active ? " filled" : "");
      // In production: POST /api/map/overlays { id: ov.id, active: ov.active }
    };
    container.appendChild(chip);
  });
}

function renderMinimapPane() {
  const container = document.getElementById("minimap-container");
  if (!container || !MAP_OVERLAYS_DATA) return;
  const mm = MAP_OVERLAYS_DATA.minimap;

  // clear any previous markers (keep mm-grid)
  container.querySelectorAll(".mm-view, .mm-city, .mm-enemy").forEach(el => el.remove());

  // viewport rect
  const view = document.createElement("div");
  view.className = "mm-view";
  view.style.cssText =
    "left:" + mm.view.left_pct + "%; top:" + mm.view.top_pct + "%; " +
    "width:" + mm.view.width_pct + "%; height:" + mm.view.height_pct + "%;";
  container.appendChild(view);

  // city / enemy dots
  mm.cities.forEach(c => {
    const dot = document.createElement("div");
    dot.className = c.enemy ? "mm-enemy" : "mm-city";
    dot.style.cssText = "left:" + c.left_pct + "%; top:" + c.top_pct + "%;";
    if (c.name) dot.title = c.name;
    container.appendChild(dot);
  });
}
