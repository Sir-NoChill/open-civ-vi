// ===== UNIT SCREEN & ARMY PANEL =====
// Reads from unit-data.json (schema: unit-data/v1) and army-data.json (schema: army-data/v1)
// Renders the full 3-col Unit/Army screen.

let ARMY_DATA = null;
let _unitScreenFilter = "all";
let _unitScreenSelected = null;

async function loadArmyData(url = "army-data.json") {
  let data;
  try { const r = await fetch(url); if (!r.ok) throw 0; data = await r.json(); }
  catch(e) { data = null; }
  ARMY_DATA = data;
  // UNIT_DATA is already loaded by loadUnitData() in wireframe.js
  // Use a small delay to ensure UNIT_DATA is ready
  setTimeout(renderUnitScreen, 50);
}

// ── CLASS → filter bucket ─────────────────────────────────────────────────────
function _unitFilterBucket(u) {
  const c = (u.class || "").toLowerCase();
  if (c.includes("naval"))   return "naval";
  if (c.includes("civilian")) return "civilian";
  return "military";
}

// ── full unit list (left col) ────────────────────────────────────────────────
function renderUnitScreen() {
  const listEl   = document.getElementById("unit-screen-list");
  const countEl  = document.getElementById("unit-screen-count");
  const filterBar = document.getElementById("unit-filter-chips");
  if (!listEl || !window.UNIT_DATA) return;

  const units = window.UNIT_DATA;

  // filter
  const filtered = _unitScreenFilter === "all"  ? units :
                   _unitScreenFilter === "idle"  ? units.filter(u => u.status === "idle") :
                   units.filter(u => _unitFilterBucket(u) === _unitScreenFilter);

  if (countEl) countEl.textContent = units.length + " total";

  // wire filter chips
  if (filterBar) {
    filterBar.querySelectorAll("[data-filter]").forEach(chip => {
      chip.className = "chip" + (chip.dataset.filter === _unitScreenFilter ? " filled" : "");
      chip.onclick = () => { _unitScreenFilter = chip.dataset.filter; renderUnitScreen(); };
    });
  }

  listEl.innerHTML = "";
  filtered.forEach(u => {
    const isSelected = _unitScreenSelected && _unitScreenSelected.id === u.id;
    const hpPct = Math.round((u.hp / u.hp_max) * 100);
    const card = document.createElement("div");
    card.className = "unitcard" + (isSelected ? "" : "");
    card.style.cssText = isSelected
      ? "border-width:2px; border-color:var(--accent); background:var(--accent-soft); cursor:pointer;"
      : "cursor:pointer;";
    const label = u.name ? u.kind + " \u00b7 " + u.name : u.kind;
    const sub = u.class + " \u00b7 HP " + u.hp + "/" + u.hp_max + " \u00b7 " + u.position.q + "," + u.position.r;
    const chargesNote = (u.charges != null) ? " (" + u.charges + "/" + u.charges_max + ")" : "";
    card.innerHTML =
      '<div class="portrait ph" style="width:40px;height:40px;">' + u.kind.slice(0,3) + '</div>' +
      '<div style="flex:1;">' +
        '<div class="v">' + label + chargesNote + '</div>' +
        '<div class="k muted">' + sub + '</div>' +
        '<div class="bar accent" style="margin-top:3px; height:4px;"><span style="width:' + hpPct + '%;display:block;height:100%;background:var(--ink);"></span></div>' +
      '</div>';
    card.onclick = () => { _unitScreenSelected = u; renderUnitScreen(); renderUnitScreenDetail(u); };
    listEl.appendChild(card);
  });

  // auto-select first unit
  if (!_unitScreenSelected && filtered.length) {
    _unitScreenSelected = filtered[0];
    renderUnitScreenDetail(filtered[0]);
  }

  // army panel
  renderArmyPanel();
}

// ── unit detail (center col) ─────────────────────────────────────────────────
function renderUnitScreenDetail(u) {
  const panel = document.getElementById("unit-screen-detail");
  if (!panel || !u) return;

  const hpPct = Math.round((u.hp / u.hp_max) * 100);
  const strLines = [
    u.strength_melee  != null ? "Melee " + u.strength_melee   : null,
    u.strength_ranged != null ? "Ranged " + u.strength_ranged : null,
  ].filter(Boolean).join(" \u00b7 ");

  const promsHTML = (u.promotions || []).length
    ? '<div style="display:grid; grid-template-columns:repeat(3,1fr); gap:6px; margin-top:4px;">' +
      (u.promotions || []).map(p =>
        '<div class="box' + (p.locked ? " dashed" : "") + '" style="padding:6px 8px; opacity:' + (p.locked ? ".5" : "1") + ';">' +
          '<div class="v">' + p.name + (p.chosen ? " \u2713" : "") + '</div>' +
          '<div class="k muted">' + p.desc + '</div>' +
          (p.chosen ? '<span class="chip accent" style="margin-top:3px;">chosen</span>' : "") +
        '</div>'
      ).join("") + '</div>'
    : '<div class="k muted" style="font-size:10px; margin-top:4px;">\u2014 no promotions yet \u2014</div>';

  const actionsHTML = '<div style="display:grid; grid-template-columns:repeat(4,1fr); gap:4px; margin-top:4px;">' +
    (u.actions || []).map(a =>
      '<button class="btn sm"' + (a.enabled ? "" : ' disabled style="opacity:.38;"') +
      (a.hotkey ? ' title="[' + a.hotkey + ']"' : "") + '>' + a.label + '</button>'
    ).join("") + '</div>';

  const chargesRow = (u.charges != null)
    ? '<span class="k">Charges</span><span class="v">' + u.charges + ' / ' + u.charges_max + '</span>'
    : "";

  panel.innerHTML =
    '<h3>' + (u.name ? u.kind + ' \u00b7 "' + u.name + '"' : u.kind) +
      ' <span class="tag">id ' + u.id + '</span></h3>' +
    '<div style="display:grid; grid-template-columns:100px 1fr; gap:12px;">' +
      '<div class="ph" style="height:100px;">unit<br/>sprite</div>' +
      '<div class="kv">' +
        '<span class="k">Class</span><span class="v">' + u.class + ' (' + u.era + ')</span>' +
        '<span class="k">HP</span><span class="v">' + u.hp + ' / ' + u.hp_max + '</span>' +
        (strLines ? '<span class="k">Strength</span><span class="v">' + strLines + '</span>' : '') +
        '<span class="k">Movement</span><span class="v">' + u.mp + ' / ' + u.mp_max + ' MP</span>' +
        '<span class="k">Sight</span><span class="v">' + u.sight + '</span>' +
        (u.xp_next ? '<span class="k">XP</span><span class="v">' + u.xp + ' / ' + u.xp_next + ' (next lvl)</span>' : '') +
        '<span class="k">Upkeep</span><span class="v">' + u.upkeep_gold + ' G / turn</span>' +
        '<span class="k">Position</span><span class="v">' + u.position.q + ',' + u.position.r + ' \u00b7 ' + (u.terrain || '\u2014') + '</span>' +
        '<span class="k">Status</span><span class="v">' + (u.status || 'idle') + '</span>' +
        chargesRow +
      '</div>' +
    '</div>' +
    '<div class="hr dashed"></div>' +
    '<div class="label">Promotions</div>' +
    promsHTML +
    '<div class="hr dashed"></div>' +
    '<div class="label">Orders</div>' +
    actionsHTML;
}

// ── army panel (right col) ───────────────────────────────────────────────────
function renderArmyPanel() {
  const panel = document.getElementById("army-panel");
  if (!panel) return;
  const d = ARMY_DATA;

  let armiesHTML = "";
  if (d && d.armies && d.armies.length) {
    armiesHTML = d.armies.map(army => {
      const memberNames = (army.unit_ids || []).map(id => {
        const u = (window.UNIT_DATA || []).find(x => x.id === id);
        return u ? (u.name || u.kind) : id;
      }).join(" + ");
      return '<div class="box" style="padding:6px 8px; margin-bottom:6px;">' +
        '<div class="label">Army \u201c' + army.name + '\u201d</div>' +
        '<div class="k">' + memberNames + '</div>' +
        '<div class="bar accent" style="margin-top:4px;"><span style="width:' + army.cohesion_pct + '%;display:block;height:100%;background:var(--accent);"></span></div>' +
        '<div class="k muted">cohesion ' + army.cohesion_pct + '%</div>' +
      '</div>';
    }).join("");
  }

  const formCorpsUnlock = d ? d.form_corps_unlock_civic : "nationalism";
  const formCorpsHTML =
    '<div class="box dashed" style="padding:6px 8px; color:var(--ink-muted);">' +
      '<div class="label">+ Form new corps</div>' +
      '<div class="k">requires ' + (formCorpsUnlock || "Nationalism") + '</div>' +
    '</div>';

  let combatHTML = '<div class="k muted" style="font-size:10px; margin-top:4px;">\u2014 select two units to preview combat \u2014</div>';
  if (d && d.combat_preview) {
    const cp = d.combat_preview;
    const attUnit = (window.UNIT_DATA || []).find(u => u.id === cp.attacker.unit_id);
    const attName = attUnit ? (attUnit.name || attUnit.kind) : "Attacker";
    combatHTML =
      '<div style="display:grid; grid-template-columns:1fr 30px 1fr; align-items:center; gap:6px;">' +
        '<div class="box" style="padding:6px;">' +
          '<div class="v">' + attName + '</div>' +
          '<div class="k">' + cp.attacker.str + ' str \u00b7 ' + cp.attacker.hp + ' HP</div>' +
        '</div>' +
        '<div style="text-align:center; font-family:var(--hand); font-size:20px;">vs</div>' +
        '<div class="box" style="padding:6px;">' +
          '<div class="v">' + cp.defender.name + '</div>' +
          '<div class="k">' + cp.defender.str + ' str \u00b7 ' + cp.defender.hp + ' HP</div>' +
        '</div>' +
      '</div>' +
      '<div class="k" style="margin-top:6px;">Predicted: <span class="v">\u2212' +
        cp.predicted.dmg_attacker + ' HP you \u00b7 \u2212' +
        cp.predicted.dmg_defender + ' HP them</span> \u00b7 ' +
        cp.predicted.outcome_label + '</div>';
  }

  panel.innerHTML =
    '<h3>Armies / Corps</h3>' +
    armiesHTML +
    formCorpsHTML +
    '<div class="hr dashed"></div>' +
    '<h3 style="border:none;">Combat preview</h3>' +
    combatHTML;
}
