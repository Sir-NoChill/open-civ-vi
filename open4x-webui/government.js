// ===== GOVERNMENT & POLICIES =====
// Schema: government-policies/v1 — see government-policies.json
let GOVT_DATA = null;
let _activePolicies = [];  // working copy: { slot, id, name, effect }
let _pendingChanges = false;

async function loadGovernmentPolicies(url = "government-policies.json") {
  let data;
  try { const r = await fetch(url); if (!r.ok) throw 0; data = await r.json(); }
  catch(e) { data = null; }
  if (!data) return;
  GOVT_DATA = data;
  _activePolicies = (data.active_policies || []).map(p => ({ ...p }));
  renderGovernmentPolicies();
}

function renderGovernmentPolicies() {
  if (!GOVT_DATA) return;
  const govt  = GOVT_DATA.government;
  const slots = govt.slots;

  // ── header ──
  const set = (id, v) => { const el = document.getElementById(id); if (el) el.textContent = v; };
  set("govt-name",  govt.name);
  set("govt-era",   govt.era);
  set("govt-bonus", "Legacy: " + govt.legacy_bonus);
  set("sb-mil", slots.military);
  set("sb-eco", slots.economic);
  set("sb-dip", slots.diplomatic);
  set("sb-wc",  slots.wildcard);

  _renderSlotPanel(slots);
  _renderCatalogue();
  _renderGovtOptions();

  // ── header buttons ──
  const changeBtn  = document.getElementById("govt-change-btn");
  const confirmBtn = document.getElementById("govt-confirm-btn");
  const cancelBtn  = document.getElementById("govt-modal-cancel");

  if (changeBtn) changeBtn.onclick = () => {
    const modal = document.getElementById("govt-change-modal");
    if (modal) modal.style.display = "flex";
  };
  if (cancelBtn) cancelBtn.onclick = () => {
    const modal = document.getElementById("govt-change-modal");
    if (modal) modal.style.display = "none";
  };
  if (confirmBtn) confirmBtn.onclick = () => {
    _pendingChanges = false;
    confirmBtn.textContent = "Confirmed \u2713";
    setTimeout(() => { confirmBtn.textContent = "Confirm policies"; }, 1800);
  };
}

// ── helpers ──────────────────────────────────────────────────────────────────

function _slotTypeClass(s) {
  return { military: "mil", economic: "eco", diplomatic: "dip", wildcard: "wc" }[s] || "";
}
function _slotTypeLabel(s) {
  return { military: "Military", economic: "Economic", diplomatic: "Diplomatic", wildcard: "Wildcard" }[s] || s;
}
function _markPending() {
  const btn = document.getElementById("govt-confirm-btn");
  if (btn && !btn.textContent.includes("\u25cf")) btn.textContent += " \u25cf";
}

// ── slot panel (left column) ──────────────────────────────────────────────────

function _renderSlotPanel(slots) {
  const panel = document.getElementById("policy-slots-panel");
  if (!panel) return;
  panel.innerHTML = "";

  ["military", "economic", "diplomatic", "wildcard"].forEach(slotType => {
    const count  = slots[slotType] || 0;
    const filled = _activePolicies.filter(p => p.slot === slotType).length;

    const group = document.createElement("div");
    group.className = "slot-group";

    const lbl = document.createElement("div");
    lbl.className = "slot-group-label";
    lbl.innerHTML = _slotTypeLabel(slotType) +
      ' <span class="sg-count">' + filled + "/" + count + "</span>";
    group.appendChild(lbl);

    if (count === 0) {
      const el = document.createElement("div");
      el.className = "policy-card empty";
      el.innerHTML = '<div class="pc-name">No ' + slotType + ' slots in current government</div><div></div>';
      group.appendChild(el);
    } else {
      for (let i = 0; i < count; i++) {
        const policy = _activePolicies.filter(p => p.slot === slotType)[i] || null;
        const card   = document.createElement("div");
        if (policy) {
          const cat   = (GOVT_DATA.catalogue || []).find(c => c.id === policy.id) || {};
          card.className = "policy-card active" + (cat.dark_age ? " dark-age" : "");
          card.innerHTML =
            '<div>' +
              '<div class="pc-name">' + policy.name + '</div>' +
              '<div class="pc-effect">' + policy.effect + '</div>' +
            '</div>' +
            '<button class="pc-remove" data-id="' + policy.id + '" data-slot="' + slotType + '">\u00d7 remove</button>';
          card.querySelector(".pc-remove").addEventListener("click", (function(pid, pslot) {
            return function(e) {
              e.stopPropagation();
              _activePolicies = _activePolicies.filter(p => !(p.id === pid && p.slot === pslot));
              _pendingChanges = true; _markPending();
              _renderSlotPanel(GOVT_DATA.government.slots);
              _renderCatalogue();
            };
          })(policy.id, slotType));
        } else {
          card.className = "policy-card empty";
          card.innerHTML = '<div class="pc-name">\u2014 empty slot \u2014</div><div></div>';
        }
        group.appendChild(card);
      }
    }
    panel.appendChild(group);
  });
}

// track collapsed state per section key across re-renders
const _catCollapsed = {};

// ── catalogue (right column) ──────────────────────────────────────────────────

function _renderCatalogue() {
  const catalogue = document.getElementById("policy-catalogue");
  if (!catalogue || !GOVT_DATA) return;
  catalogue.innerHTML = "";

  const TYPES  = ["military", "economic", "diplomatic", "wildcard"];
  const LABELS = { military: "Military", economic: "Economic", diplomatic: "Diplomatic", wildcard: "Wildcard" };

  TYPES.forEach(function(type) {
    const cards = GOVT_DATA.catalogue.filter(c => c.type === type && !c.dark_age);
    if (!cards.length) return;
    _appendCatSection(catalogue, type, LABELS[type] + " Policies", cards, false);
  });

  // dark age section at the bottom
  const darkCards = GOVT_DATA.catalogue.filter(c => c.dark_age);
  if (darkCards.length) {
    _appendCatSection(catalogue, "dark_age",
      '<span class="cs-dark">\u26ab Dark Age Policies</span> <span class="cs-era">only during dark ages</span>',
      darkCards, true);
  }
}

function _appendCatSection(catalogue, key, labelHTML, cards, isDarkAge) {
  const collapsed = !!_catCollapsed[key];

  const section = document.createElement("div");
  section.className = "cat-section";

  const sLbl = document.createElement("div");
  sLbl.className = "cat-section-label";
  sLbl.style.cssText = "cursor:pointer; user-select:none;";
  // chevron + label
  const chevron = collapsed ? "\u25b6" : "\u25bc";  // ▶ / ▼
  sLbl.innerHTML =
    '<span class="cs-chevron" style="font-size:8px; margin-right:5px; opacity:.6;">' + chevron + '</span>' +
    labelHTML +
    ' <span style="font-size:8px; color:var(--ink-faint); margin-left:4px;">' + cards.length + '</span>';
  section.appendChild(sLbl);

  // rows container — hidden when collapsed
  const rowsWrap = document.createElement("div");
  rowsWrap.style.display = collapsed ? "none" : "flex";
  rowsWrap.style.flexDirection = "column";
  rowsWrap.style.gap = "4px";
  if (!collapsed) {
    cards.forEach(card => _appendCatRow(rowsWrap, card));
  }
  section.appendChild(rowsWrap);

  // toggle on label click
  sLbl.addEventListener("click", function() {
    _catCollapsed[key] = !_catCollapsed[key];
    const nowCollapsed = _catCollapsed[key];
    sLbl.querySelector(".cs-chevron").textContent = nowCollapsed ? "\u25b6" : "\u25bc";
    if (nowCollapsed) {
      rowsWrap.style.display = "none";
    } else {
      rowsWrap.style.display = "flex";
      if (!rowsWrap.children.length) {
        cards.forEach(card => _appendCatRow(rowsWrap, card));
      }
    }
  });

  catalogue.appendChild(section);
}

function _appendCatRow(section, card) {
  const isActive  = _activePolicies.some(p => p.id === card.id);
  const typeClass = _slotTypeClass(card.type);

  const row = document.createElement("div");
  const classes = ["cat-row"];
  if (isActive)              classes.push("active");
  if (card.status === "locked") classes.push("locked");
  if (card.dark_age)         classes.push("dark-age");
  row.className = classes.join(" ");

  let actionHTML;
  if (card.status === "locked") {
    const req = card.unlock_civic ? card.unlock_civic.replace(/_/g, " ") : "\u2014";
    actionHTML = '<button class="cr-btn locked-btn" disabled title="Requires ' + req + '">Locked</button>';
  } else if (isActive) {
    actionHTML = '<button class="cr-btn active-btn" data-id="' + card.id + '">Equipped</button>';
  } else {
    actionHTML = '<button class="cr-btn" data-id="' + card.id + '">+ Equip</button>';
  }

  const unlockText = (card.status === "locked" && card.unlock_civic)
    ? "Req: " + card.unlock_civic.replace(/_/g, " ")
    : (card.era || "");

  row.innerHTML =
    '<div class="cr-type ' + typeClass + '"></div>' +
    '<div>' +
      '<div class="cr-name">' + card.name + '</div>' +
      '<div class="cr-effect">' + card.effect + '</div>' +
    '</div>' +
    '<div class="cr-unlock">' + unlockText + '</div>' +
    '<div class="cr-action">' + actionHTML + '</div>';

  if (card.status !== "locked") {
    const btn = row.querySelector(".cr-btn");
    if (isActive) {
      btn.addEventListener("click", (function(cid) {
        return function(e) {
          e.stopPropagation();
          _activePolicies = _activePolicies.filter(p => p.id !== cid);
          _pendingChanges = true; _markPending();
          _renderSlotPanel(GOVT_DATA.government.slots);
          _renderCatalogue();
        };
      })(card.id));
    } else {
      btn.addEventListener("click", (function(c) {
        return function(e) {
          e.stopPropagation();
          _equipPolicy(c);
        };
      })(card));
    }
  }
  section.appendChild(row);
}

function _equipPolicy(card) {
  const slots     = GOVT_DATA.government.slots;
  const slotType  = card.type;
  const filledOf  = _activePolicies.filter(p => p.slot === slotType).length;
  const maxOf     = slots[slotType] || 0;
  const filledWC  = _activePolicies.filter(p => p.slot === "wildcard").length;
  const maxWC     = slots.wildcard || 0;

  let targetSlot;
  if (filledOf < maxOf) {
    targetSlot = slotType;
  } else if (slotType !== "wildcard" && filledWC < maxWC) {
    targetSlot = "wildcard";
  } else {
    // no room — flash the slot badge
    const badgeId = { military: "sb-mil", economic: "sb-eco", diplomatic: "sb-dip", wildcard: "sb-wc" }[slotType];
    const badge = document.getElementById(badgeId);
    if (badge) {
      const parent = badge.closest(".slot-badge");
      if (parent) {
        parent.style.background = "#f5eeee";
        setTimeout(() => { parent.style.background = ""; }, 500);
      }
    }
    return;
  }
  _activePolicies.push({ slot: targetSlot, id: card.id, name: card.name, effect: card.effect });
  _pendingChanges = true; _markPending();
  _renderSlotPanel(GOVT_DATA.government.slots);
  _renderCatalogue();
}

// ── government change modal ───────────────────────────────────────────────────

function _renderGovtOptions() {
  const list = document.getElementById("govt-options-list");
  if (!list || !GOVT_DATA) return;
  list.innerHTML = "";

  (GOVT_DATA.available_governments || []).forEach(function(g) {
    const s   = g.slots;
    const req = g.unlock_civic ? " \u00b7 req: " + g.unlock_civic.replace(/_/g, " ") : "";
    const row = document.createElement("div");
    row.className = "box" + (g.current ? " heavy" : "");
    row.style.cssText = "padding:8px 12px; display:grid; grid-template-columns:1fr auto; gap:8px; align-items:center; opacity:" + (g.locked ? ".45" : "1") + ";";

    const currentTag = g.current
      ? ' <span style="color:var(--accent);font-size:9px;font-weight:400;">current</span>'
      : "";
    const btnLabel = g.current ? "Active" : g.locked ? "Locked" : "Switch \u2192";
    const btnDisabled = (g.locked || g.current) ? "disabled" : "";
    const btnClass = "btn" + (g.current ? " primary" : "");

    row.innerHTML =
      "<div>" +
        '<div style="font-weight:700;font-size:11px;text-transform:uppercase;letter-spacing:.04em;">' +
          g.name + currentTag +
        "</div>" +
        '<div style="font-size:9.5px;color:var(--ink-muted);margin-top:2px;">' +
          g.era + " \u00b7 " + s.military + " mil \u00b7 " + s.economic + " eco \u00b7 " +
          s.diplomatic + " dip \u00b7 " + s.wildcard + " wild" + req +
        "</div>" +
      "</div>" +
      '<button class="' + btnClass + '" ' + btnDisabled + ">" + btnLabel + "</button>";

    if (!g.current && !g.locked) {
      row.querySelector("button").addEventListener("click", (function(gov) {
        return function() {
          document.getElementById("govt-change-modal").style.display = "none";
          // swap government
          GOVT_DATA.government = {
            id: gov.id, name: gov.name, era: gov.era, slots: gov.slots,
            legacy_bonus: GOVT_DATA.government.legacy_bonus
          };
          GOVT_DATA.available_governments.forEach(x => { x.current = (x.id === gov.id); });
          // trim policies that exceed new slot counts
          const kept = [];
          ["military", "economic", "diplomatic", "wildcard"].forEach(st => {
            const max = gov.slots[st] || 0;
            _activePolicies.filter(p => p.slot === st).slice(0, max).forEach(p => kept.push(p));
          });
          _activePolicies = kept;
          _pendingChanges = true; _markPending();
          renderGovernmentPolicies();
        };
      })(g));
    }
    list.appendChild(row);
  });
}
