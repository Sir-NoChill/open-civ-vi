// Tabs + tweaks + tiny interactions

const SCREENS = [
  { id: "hud",      label: "Main HUD" },
  { id: "city",     label: "City" },
  { id: "tech",     label: "Tech Tree" },
  { id: "civics",   label: "Civics" },
  { id: "unit",     label: "Unit/Army" },
  { id: "dipl",     label: "Diplomacy" },
  { id: "overview", label: "Empire Overview" },
  { id: "victory",  label: "Victory" },
];

function showScreen(id) {
  document.querySelectorAll(".screen").forEach(s => s.classList.toggle("active", s.dataset.screen === id));
  document.querySelectorAll(".tab").forEach(t => t.classList.toggle("active", t.dataset.tab === id));
  try { localStorage.setItem("wf:screen", id); } catch(e) {}
}

function applyTweaks(t) {
  document.body.classList.toggle("sketch", t.sketch !== false);
  document.body.classList.toggle("clean", t.sketch === false);
  document.body.classList.toggle("show-anno", t.annotations !== false);
}

window.addEventListener("DOMContentLoaded", () => {
  // build tabs
  const tabs = document.getElementById("tabs");
  SCREENS.forEach(s => {
    const el = document.createElement("div");
    el.className = "tab"; el.dataset.tab = s.id; el.textContent = s.label;
    el.onclick = () => showScreen(s.id);
    tabs.appendChild(el);
  });

  // tweak controls
  const tweakState = window.TWEAKS = { sketch: true, annotations: true };
  const sketchEl = document.getElementById("tw-sketch");
  const annoEl   = document.getElementById("tw-anno");
  sketchEl.checked = tweakState.sketch;
  annoEl.checked   = tweakState.annotations;
  sketchEl.onchange = () => { tweakState.sketch = sketchEl.checked; applyTweaks(tweakState); };
  annoEl.onchange   = () => { tweakState.annotations = annoEl.checked; applyTweaks(tweakState); };
  applyTweaks(tweakState);

  // edit-mode protocol
  window.addEventListener("message", (e) => {
    const d = e.data || {};
    if (d.type === "__activate_edit_mode") document.querySelector(".tweaks").classList.add("visible");
    if (d.type === "__deactivate_edit_mode") document.querySelector(".tweaks").classList.remove("visible");
  });
  try { window.parent.postMessage({ type: "__edit_mode_available" }, "*"); } catch(e) {}

  // restore screen
  const saved = (() => { try { return localStorage.getItem("wf:screen"); } catch(e) { return null; } })();
  showScreen(saved && SCREENS.find(s=>s.id===saved) ? saved : "hud");

  // hex click (HUD)
  document.querySelectorAll("#scr-hud .hex").forEach(h => {
    h.onclick = () => {
      document.querySelectorAll("#scr-hud .hex.selected").forEach(x => x.classList.remove("selected"));
      h.classList.add("selected");
    };
  });
});
