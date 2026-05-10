// New game wizard — Map → Civ → Rules → Players → Review

const WIZARD_STEPS = [
  { id: "map", label: "Map" },
  { id: "civ", label: "Civilization" },
  { id: "rules", label: "Rules" },
  { id: "players", label: "Players" },
  { id: "review", label: "Review" },
];

const StepStrip = ({ current, onJump }) => (
  <div className="wizard-steps">
    {WIZARD_STEPS.map((s, i) => {
      const idx = WIZARD_STEPS.findIndex(x => x.id === current);
      const state = i < idx ? "done" : i === idx ? "current" : "";
      return (
        <React.Fragment key={s.id}>
          <button className={`step ${state}`} onClick={() => onJump?.(s.id)}
                  style={{ background: "none", border: "none", cursor: "pointer", padding: 0, font: "inherit", color: "inherit" }}>
            <span className="num">{i + 1}</span>
            <span>{s.label}</span>
          </button>
          {i < WIZARD_STEPS.length - 1 && <span className="arrow">›</span>}
        </React.Fragment>
      );
    })}
    <span style={{ marginLeft: "auto" }} className="muted">step {WIZARD_STEPS.findIndex(x => x.id === current) + 1} of {WIZARD_STEPS.length}</span>
  </div>
);

// ---- Step: Map ----
const StepMap = ({ advanced, setAdvanced }) => (
  <div className="wizard-body">
    <Box>
      <div className="row between center-y" style={{ marginBottom: 10 }}>
        <div className="h3">MAP &amp; WORLD</div>
        <div className="row center-y" style={{ gap: 8, fontSize: "var(--fs-sm)" }}>
          <span className="muted">advanced</span>
          <Toggle on={advanced} onChange={setAdvanced} />
        </div>
      </div>

      <div className="param-row">
        <div className="label">map type</div>
        <div className="control">
          <Segmented options={["continents", "pangaea", "archipelago", "fractal", "custom"]} value="continents" onChange={() => {}} />
        </div>
        <div className="value">continents</div>
      </div>
      <div className="param-row">
        <div className="label">map size</div>
        <div className="control">
          <Segmented options={["duel", "tiny", "small", "std", "large", "huge"]} value="std" onChange={() => {}} />
        </div>
        <div className="value">standard · 84×54</div>
      </div>

      {advanced && <>
        <div className="param-row">
          <div className="label">world age</div>
          <div className="control"><Slider value={4} min={1} max={10} format={v => `${v}bn yrs`} /></div>
          <div className="value">4bn</div>
        </div>
        <div className="param-row">
          <div className="label">sea level</div>
          <div className="control"><Slider value={50} format={v => `${v}%`} /></div>
          <div className="value">50%</div>
        </div>
        <div className="param-row">
          <div className="label">temperature</div>
          <div className="control"><Slider value={50} format={v => v < 33 ? "cold" : v > 66 ? "hot" : "temperate"} /></div>
          <div className="value">temperate</div>
        </div>
        <div className="param-row">
          <div className="label">rainfall</div>
          <div className="control"><Slider value={45} format={v => v < 33 ? "arid" : v > 66 ? "wet" : "normal"} /></div>
          <div className="value">normal</div>
        </div>
        <div className="param-row">
          <div className="label">resources</div>
          <div className="control">
            <Segmented options={["sparse", "standard", "abundant", "legendary"]} value="standard" onChange={() => {}} />
          </div>
          <div className="value">standard</div>
        </div>
        <div className="param-row">
          <div className="label">random seed</div>
          <div className="control">
            <input className="sk-input" defaultValue="0xCAFE·B33F·1A77" style={{ fontFamily: "monospace" }} />
            <Btn size="sm" variant="ghost">⟳</Btn>
            <Btn size="sm" variant="ghost">copy</Btn>
          </div>
          <div className="value muted">paste to share</div>
        </div>
      </>}
    </Box>

    <div className="col">
      <Box>
        <div className="h3" style={{ marginBottom: 8 }}>PREVIEW</div>
        <div className="map-preview">
          <span className="corner-mono">// procedurally generated · regenerate as needed</span>
          <span style={{ fontSize: 13 }}>MAP PREVIEW</span>
        </div>
        <div className="row between" style={{ marginTop: 8, fontSize: "var(--fs-sm)" }}>
          <span className="muted">tiles: 4536 · land 47%</span>
          <Btn size="sm" variant="ghost">⟳ regenerate</Btn>
        </div>
      </Box>
      <div className="annotation">advanced toggle exposes 12+ procgen params</div>
    </div>
  </div>
);

// ---- Step: Civ ----
const civs = [
  ["Saladin", "Arabia", "Trade & faith"],
  ["Trajan", "Rome", "Expansionist"],
  ["Catherine", "Russia", "Wide / faith"],
  ["Cleopatra", "Egypt", "Wonders / trade"],
  ["Hojo", "Japan", "Coastal / military"],
  ["Gandhi", "India", "Religion / peace"],
  ["Pedro II", "Brazil", "Cultural"],
  ["Random", "?", "surprise me"],
];

const StepCiv = () => (
  <div className="wizard-body single">
    <Box>
      <div className="h3" style={{ marginBottom: 12 }}>PICK YOUR CIVILIZATION</div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 10 }}>
        {civs.map(([leader, civ, trait], i) => (
          <div key={i} className={`sk-box ${leader === "Saladin" ? "fill" : ""}`} style={{
            cursor: "pointer", padding: 12,
            borderStyle: leader === "Saladin" ? "solid" : "dashed",
            borderColor: leader === "Saladin" ? "var(--accent)" : "var(--ink)",
          }}>
            <div className="row" style={{ gap: 10 }}>
              <div style={{
                width: 40, height: 40, border: "1.2px dashed var(--ink-2)",
                display: "grid", placeItems: "center", fontFamily: "Caveat, cursive", fontSize: 22,
                flexShrink: 0,
              }}>{leader[0]}</div>
              <div style={{ minWidth: 0 }}>
                <div style={{ fontWeight: 700 }}>{leader}</div>
                <div className="muted" style={{ fontSize: "var(--fs-sm)" }}>{civ}</div>
                <div style={{ fontSize: "var(--fs-sm)", marginTop: 2 }}>{trait}</div>
              </div>
            </div>
          </div>
        ))}
      </div>
      <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 12 }}>
        each civ has unique unit + ability. full sheet on hover. (placeholder)
      </div>
    </Box>
  </div>
);

// ---- Step: Rules ----
const StepRules = () => (
  <div className="wizard-body">
    <Box>
      <div className="h3" style={{ marginBottom: 8 }}>DIFFICULTY &amp; PACE</div>
      <div className="param-row">
        <div className="label">difficulty</div>
        <div className="control">
          <Segmented options={["settler", "chieftain", "warlord", "prince", "king", "emperor", "deity"]} value="prince" onChange={() => {}} />
        </div>
        <div className="value">prince</div>
      </div>
      <div className="param-row">
        <div className="label">starting era</div>
        <div className="control">
          <Segmented options={["ancient", "classical", "medieval", "renaissance", "industrial"]} value="ancient" onChange={() => {}} />
        </div>
        <div className="value">ancient</div>
      </div>
      <div className="param-row">
        <div className="label">game speed</div>
        <div className="control">
          <Segmented options={["online", "quick", "std", "epic", "marathon"]} value="std" onChange={() => {}} />
        </div>
        <div className="value">standard</div>
      </div>

      <hr className="sk-divider" />
      <div className="h3" style={{ marginBottom: 8 }}>VICTORY CONDITIONS</div>
      {["Science", "Culture", "Domination", "Religion", "Diplomacy", "Score"].map((v, i) => (
        <div className="param-row" key={v}>
          <div className="label">{v.toLowerCase()}</div>
          <div className="control"><Toggle on={i !== 4} /></div>
          <div className="value muted">{i === 4 ? "off" : "enabled"}</div>
        </div>
      ))}
    </Box>

    <Box>
      <div className="h3" style={{ marginBottom: 8 }}>WORLD DYNAMICS</div>
      <div className="param-row">
        <div className="label">disasters</div>
        <div className="control"><Slider value={2} min={0} max={4} format={v => ["off", "light", "std", "heavy", "apocalyptic"][v]} /></div>
        <div className="value">std</div>
      </div>
      <div className="param-row">
        <div className="label">barbarians</div>
        <div className="control"><Slider value={2} min={0} max={4} format={v => ["off", "rare", "std", "raging", "horde"][v]} /></div>
        <div className="value">std</div>
      </div>
      <div className="param-row">
        <div className="label">city-states</div>
        <div className="control"><Slider value={12} min={0} max={24} /></div>
        <div className="value">12</div>
      </div>
      <div className="param-row">
        <div className="label">AI aggression</div>
        <div className="control"><Slider value={50} format={v => v < 34 ? "passive" : v > 66 ? "warlike" : "balanced"} /></div>
        <div className="value">balanced</div>
      </div>
      <div className="param-row">
        <div className="label">AI personality</div>
        <div className="control">
          <Segmented options={["historic", "random", "scripted"]} value="historic" onChange={() => {}} />
        </div>
        <div className="value">historic</div>
      </div>
    </Box>
  </div>
);

// ---- Step: Players ----
const StepPlayers = () => {
  const [open, setOpen] = React.useState(false);
  return (
    <div className="wizard-body">
      <Box>
        <div className="row between center-y" style={{ marginBottom: 10 }}>
          <div className="h3">PLAYERS &amp; SLOTS</div>
          <Btn size="sm" variant="ghost">+ add slot</Btn>
        </div>

        {[
          { name: "Alice (you)", type: "human", civ: "Arabia · Saladin", you: true },
          { name: "—", type: "open", civ: "—", invite: true },
          { name: "AI", type: "ai", civ: "Rome · Trajan" },
          { name: "AI", type: "ai", civ: "Russia · Catherine" },
          { name: "AI", type: "ai", civ: "Random" },
          { name: "AI", type: "ai", civ: "Random" },
          { name: "AI", type: "ai", civ: "Random" },
          { name: "AI", type: "ai", civ: "Random" },
        ].map((p, i) => (
          <div key={i} className="sk-box" style={{
            padding: 8, marginBottom: 6, borderColor: p.you ? "var(--accent)" : "var(--ink)",
            borderStyle: p.you ? "solid" : "dashed", background: p.you ? "var(--accent-soft)" : "var(--paper)",
          }}>
            <div className="row center-y" style={{ gap: 10 }}>
              <span className="muted" style={{ fontSize: "var(--fs-sm)", width: 24 }}>#{i + 1}</span>
              <div style={{ flex: 1 }}>
                <div className="row center-y" style={{ gap: 8 }}>
                  <span style={{ fontWeight: 600 }}>{p.name}</span>
                  <Tag variant={p.type === "human" ? "accent" : p.type === "open" ? "dim" : ""}>{p.type}</Tag>
                </div>
                <div className="muted" style={{ fontSize: "var(--fs-sm)" }}>{p.civ}</div>
              </div>
              {p.invite ? (
                <Btn size="sm" variant="primary" onClick={() => setOpen(true)}>invite</Btn>
              ) : (
                <Btn size="sm" variant="ghost">···</Btn>
              )}
            </div>
          </div>
        ))}
      </Box>

      <div className="col">
        <Box>
          <div className="h3" style={{ marginBottom: 8 }}>TURN MODE</div>
          <div className="param-row">
            <div className="label">turn timer</div>
            <div className="control">
              <Segmented options={["off", "5min", "10min", "30min", "24hr"]} value="off" onChange={() => {}} />
            </div>
            <div className="value">off</div>
          </div>
          <div className="param-row">
            <div className="label">simultaneous</div>
            <div className="control"><Toggle on={false} /></div>
            <div className="value muted">play-by-turn</div>
          </div>
          <div className="param-row">
            <div className="label">private game</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted">invite-only</div>
          </div>
        </Box>

        {open && (
          <Box className="fill">
            <div className="row between center-y" style={{ marginBottom: 8 }}>
              <div className="h3">INVITE PLAYER</div>
              <Btn size="sm" variant="ghost" onClick={() => setOpen(false)}>×</Btn>
            </div>
            <div className="muted" style={{ fontSize: "var(--fs-sm)", marginBottom: 8 }}>
              paste any of: email · openid url · atproto handle · player ID
            </div>
            <input className="sk-input" placeholder="alice@example.com  /  did:plc:…  /  0xA9C3·…" />
            <div className="row" style={{ gap: 6, marginTop: 8 }}>
              <Btn variant="accent" size="sm">send invite</Btn>
              <Btn size="sm" variant="ghost">copy invite link</Btn>
            </div>
            <hr className="sk-divider" />
            <div className="muted" style={{ fontSize: "var(--fs-sm)" }}>recent friends:</div>
            <div className="row wrap" style={{ gap: 6, marginTop: 6 }}>
              {["bob.bsky.social", "carol@…", "0xFE12·…"].map(f => (
                <button key={f} className="filter-chip">{f}</button>
              ))}
            </div>
          </Box>
        )}
      </div>
    </div>
  );
};

// ---- Step: Review ----
const StepReview = ({ onGenerate }) => (
  <div className="wizard-body">
    <Box>
      <div className="h3" style={{ marginBottom: 12 }}>SUMMARY</div>
      {[
        ["map", "continents · standard (84×54) · 0xCAFE·B33F·1A77"],
        ["world", "4bn yrs · sea 50% · temperate · normal rainfall"],
        ["civ", "Saladin / Arabia"],
        ["difficulty", "prince · standard speed · ancient era"],
        ["victory", "science · culture · domination · religion · score"],
        ["dynamics", "disasters std · barbs std · 12 city-states · AI balanced"],
        ["players", "1 human + 1 invite + 6 AI"],
        ["turn mode", "play-by-turn · invite-only"],
      ].map(([k, v]) => (
        <div key={k} className="param-row">
          <div className="label">{k}</div>
          <div className="control" style={{ fontSize: "var(--fs-base)" }}>{v}</div>
          <div className="value"><Btn size="sm" variant="ghost">edit</Btn></div>
        </div>
      ))}
    </Box>
    <div className="col">
      <Box>
        <div className="h3" style={{ marginBottom: 8 }}>FINAL PREVIEW</div>
        <div className="map-preview">
          <span className="corner-mono">// last preview before lock-in</span>
          <span>MAP</span>
        </div>
      </Box>
      <Box className="fill">
        <div className="muted" style={{ fontSize: "var(--fs-sm)", marginBottom: 8 }}>
          Generation creates the world deterministically from the seed above.
          You can copy the seed to recreate this exact map elsewhere.
        </div>
        <Btn variant="accent" style={{ width: "100%", justifyContent: "center", padding: "12px 14px", fontSize: "var(--fs-lg)" }} onClick={onGenerate}>
          ⌬  GENERATE WORLD
        </Btn>
        <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 8, textAlign: "center" }}>
          // calls /api/games · returns game_id · routes you to gameplay client
        </div>
      </Box>
    </div>
  </div>
);

const NewGame = () => {
  const [step, setStep] = React.useState("map");
  const [advanced, setAdvanced] = React.useState(false);

  const idx = WIZARD_STEPS.findIndex(s => s.id === step);
  const next = () => idx < WIZARD_STEPS.length - 1 && setStep(WIZARD_STEPS[idx + 1].id);
  const prev = () => idx > 0 && setStep(WIZARD_STEPS[idx - 1].id);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <div className="content-header">
        <div className="title">New game</div>
        <span className="crumbs">// procedural worldgen · save preset for re-use</span>
        <div style={{ marginLeft: "auto" }} className="row" style={{ gap: 6 }}>
          <Btn size="sm" variant="ghost">save preset</Btn>
          <Btn size="sm" variant="ghost">load preset</Btn>
        </div>
      </div>

      <StepStrip current={step} onJump={setStep} />

      <div style={{ flex: 1, overflow: "auto", paddingBottom: 60 }}>
        {step === "map" && <StepMap advanced={advanced} setAdvanced={setAdvanced} />}
        {step === "civ" && <StepCiv />}
        {step === "rules" && <StepRules />}
        {step === "players" && <StepPlayers />}
        {step === "review" && <StepReview onGenerate={() => alert("→ /api/games/create (stubbed)")} />}
      </div>

      <div className="row between" style={{
        padding: "10px 14px", borderTop: "1.5px dashed var(--ink)",
        background: "var(--paper)", marginTop: "auto",
      }}>
        <Btn size="sm" variant="ghost" disabled={idx === 0} onClick={prev}>← back</Btn>
        <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>
          <span className="kbd">⏎</span> next · <span className="kbd">esc</span> cancel
        </span>
        {step === "review"
          ? <Btn variant="accent" onClick={() => alert("→ /api/games/create (stubbed)")}>⌬  Generate</Btn>
          : <Btn variant="primary" onClick={next}>next →</Btn>
        }
      </div>
    </div>
  );
};

window.NewGame = NewGame;
