// Hi-fi New Game wizard

const STEPS = [
  { id: "map",     label: "Map" },
  { id: "civ",     label: "Civilization" },
  { id: "rules",   label: "Rules" },
  { id: "players", label: "Players" },
  { id: "review",  label: "Review" },
];

const StepStrip = ({ current, onJump }) => {
  const idx = STEPS.findIndex(s => s.id === current);
  return (
    <div className="wizard-steps">
      {STEPS.map((s, i) => {
        const state = i < idx ? "done" : i === idx ? "current" : "";
        return (
          <React.Fragment key={s.id}>
            <button className={`step ${state}`} onClick={() => onJump?.(s.id)}>
              <span className="num">{i + 1}</span>
              <span>{s.label}</span>
            </button>
            {i < STEPS.length - 1 && <span className="arrow">›</span>}
          </React.Fragment>
        );
      })}
      <span style={{ marginLeft: "auto" }} className="muted xsmall">
        step {idx + 1} of {STEPS.length}
      </span>
    </div>
  );
};

// Helper for popup-explained labels
const ParamLabel = ({ children, help }) => (
  <Popup title={children} content={<PopupBody>{help}</PopupBody>}>
    <span className="trigger">{children}</span>
  </Popup>
);

// ----- step: map -----
const StepMap = ({ advanced, setAdvanced }) => (
  <div className="wizard-body">
    <div className="panel flush">
      <div className="panel-head">
        <span className="title">Map &amp; world</span>
        <span className="sub">// procgen parameters</span>
        <div style={{ marginLeft: "auto" }} className="row center-y gap-sm">
          <span className="muted xsmall">advanced</span>
          <Toggle on={advanced} />
        </div>
      </div>
      <div className="panel-body">
        <div className="param-row stack">
          <div className="label">
            <ParamLabel help={
              <>
                <p><strong>Continents</strong> — 2-3 large landmasses with ocean separation.</p>
                <p><strong>Pangaea</strong> — one supercontinent.</p>
                <p><strong>Archipelago</strong> — many small islands.</p>
                <p><strong>Fractal</strong> — Perlin-noise seeded; unpredictable shapes.</p>
                <p><strong>Custom</strong> — paste a seed or import from JSON.</p>
              </>
            }>map type</ParamLabel>
          </div>
          <div className="control">
            <Segmented options={["continents","pangaea","archipelago","fractal","custom"]} value="continents" />
          </div>
        </div>

        <div className="param-row stack">
          <div className="label">
            <ParamLabel help={<><p>Tile dimensions of the world.</p><div className="kv xsmall"><span className="k">duel</span><span>44×26</span><span className="k">tiny</span><span>60×38</span><span className="k">small</span><span>74×46</span><span className="k">std</span><span>84×54</span><span className="k">large</span><span>96×60</span><span className="k">huge</span><span>106×66</span></div></>}>map size <span className="muted xsmall" style={{textTransform:'none',letterSpacing:0,marginLeft:6}}>standard · 84×54</span></ParamLabel>
          </div>
          <div className="control">
            <Segmented options={["duel","tiny","small","std","large","huge"]} value="std" />
          </div>
        </div>

        {advanced && <>
          <div className="param-row">
            <div className="label"><ParamLabel help={<p>Older worlds have smoother terrain — fewer mountains, more hills, more flat plains.</p>}>world age</ParamLabel></div>
            <div className="control"><Slider value={4} min={1} max={10} format={v=>`${v}bn yrs`} /></div>
            <div className="value">4 bn</div>
          </div>
          <div className="param-row">
            <div className="label"><ParamLabel help={<p>Higher = more ocean, smaller islands; lower = more exposed land bridges.</p>}>sea level</ParamLabel></div>
            <div className="control"><Slider value={50} format={v=>`${v}%`} /></div>
            <div className="value">50%</div>
          </div>
          <div className="param-row">
            <div className="label"><ParamLabel help={<p>Cold worlds: more tundra/snow at poles. Hot: more desert near equator.</p>}>temperature</ParamLabel></div>
            <div className="control"><Slider value={50} format={v=>v<33?"cold":v>66?"hot":"temperate"} /></div>
            <div className="value">temperate</div>
          </div>
          <div className="param-row">
            <div className="label"><ParamLabel help={<p>Drier worlds: more desert, fewer forests. Wetter: more jungle and floodplains.</p>}>rainfall</ParamLabel></div>
            <div className="control"><Slider value={45} format={v=>v<33?"arid":v>66?"wet":"normal"} /></div>
            <div className="value">normal</div>
          </div>
          <div className="param-row stack">
            <div className="label">resources</div>
            <div className="control"><Segmented options={["sparse","standard","abundant","legendary"]} value="standard" /></div>
          </div>
          <div className="param-row">
            <div className="label"><ParamLabel help={<><p>Deterministic seed for the entire generation. Copy and share with friends to play the exact same map.</p><p className="muted xsmall">Same seed + same parameters = same world, every time.</p></>}>random seed</ParamLabel></div>
            <div className="control">
              <input className="input mono" defaultValue="0xCAFE·B33F·1A77" style={{ fontSize: "var(--fs-xs)" }} />
              <Btn variant="ghost" size="xs">⟳</Btn>
              <Btn variant="ghost" size="xs">⎘</Btn>
            </div>
            <div className="value muted xsmall">share to clone</div>
          </div>
        </>}
      </div>
    </div>

    <div className="col">
      <div className="panel flush">
        <div className="panel-head">
          <span className="title">Preview</span>
          <span className="sub">// regenerable</span>
        </div>
        <div style={{ padding: 1 }}>
          <div className="map-preview">
            <span className="corner">// 84×54 · continents · seed 0xCAFE…</span>
            <span className="corner tr">⟳</span>
            <MiniMap seed={42} style={{ position: "absolute", inset: 0, width: "100%", height: "100%" }} />
          </div>
        </div>
        <div className="row between center-y" style={{ padding: "10px 14px", borderTop: "1px solid var(--hairline)", fontSize: "var(--fs-xs)" }}>
          <span className="muted">tiles 4536 · land 47% · climate temperate</span>
          <Btn variant="ghost" size="sm">⟳ regenerate</Btn>
        </div>
      </div>

      <div className="panel">
        <div className="h3" style={{ marginBottom: 8 }}>Hint</div>
        <p className="muted small" style={{ margin: 0 }}>
          The advanced toggle exposes 12+ procgen parameters. Hover any
          underlined label to see what it does.
        </p>
      </div>
    </div>
  </div>
);

// ----- step: civ -----
const civs = [
  ["Saladin",  "Arabia",  "Trade & faith"],
  ["Trajan",   "Rome",    "Expansionist"],
  ["Catherine","Russia",  "Wide / faith"],
  ["Cleopatra","Egypt",   "Wonders / trade"],
  ["Hojo",     "Japan",   "Coastal / military"],
  ["Gandhi",   "India",   "Religion / peace"],
  ["Pedro II", "Brazil",  "Cultural"],
  ["Random",   "?",       "surprise me"],
];

const CivSheet = ({ leader, civ, trait }) => (
  <>
    <PopupBody>
      <div style={{ fontWeight: 600, fontSize: "var(--fs-md)" }}>{leader} · {civ}</div>
      <p className="muted xsmall" style={{ marginBottom: 8 }}>{trait}</p>
      <div className="kv xsmall">
        <span className="k">unique unit</span><span>Mamluk</span>
        <span className="k">unique bldg</span><span>Madrasa</span>
        <span className="k">leader ability</span><span>Righteousness of the Faith</span>
        <span className="k">civ ability</span><span>The Last Prophet</span>
      </div>
    </PopupBody>
    <PopupActions right>
      <Btn variant="ghost" size="sm">view full sheet</Btn>
      <Btn variant="primary" size="sm">select</Btn>
    </PopupActions>
  </>
);

const StepCiv = () => (
  <div className="wizard-body single">
    <div className="panel flush">
      <div className="panel-head">
        <span className="title">Pick your civilization</span>
        <span className="sub">// hover any leader to see their unique units &amp; abilities</span>
      </div>
      <div className="panel-body">
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 8 }}>
          {civs.map(([leader, civ, trait], i) => (
            <Popup key={i} title="civ sheet" content={<CivSheet leader={leader} civ={civ} trait={trait} />}>
              <div className="panel" style={{
                cursor: "pointer", padding: 12, width: "100%",
                borderColor: leader === "Saladin" ? "var(--accent)" : "var(--hairline)",
                background: leader === "Saladin" ? "var(--accent-soft)" : "var(--paper)",
              }}>
                <div className="row gap-sm">
                  <div style={{
                    width: 40, height: 40,
                    background: "var(--ink)", color: "var(--paper)",
                    display: "grid", placeItems: "center",
                    fontFamily: "var(--font-serif)", fontSize: 22,
                  }}>{leader[0]}</div>
                  <div style={{ minWidth: 0 }}>
                    <div style={{ fontWeight: 600 }}>{leader}</div>
                    <div className="muted xsmall">{civ}</div>
                    <div className="xsmall" style={{ marginTop: 2 }}>{trait}</div>
                  </div>
                </div>
              </div>
            </Popup>
          ))}
        </div>
      </div>
    </div>
  </div>
);

// ----- step: rules -----
const StepRules = () => (
  <div className="wizard-body">
    <div className="panel flush">
      <div className="panel-head"><span className="title">Difficulty &amp; pace</span></div>
      <div className="panel-body">
        <div className="param-row stack">
          <div className="label"><ParamLabel help={<><p>Affects AI bonuses, barbarian aggression, and yield modifiers.</p><div className="kv xsmall" style={{marginTop:6}}><span className="k">settler</span><span>−40% AI yields</span><span className="k">prince</span><span>baseline</span><span className="k">deity</span><span>+50% AI yields</span></div></>}>difficulty</ParamLabel></div>
          <div className="control"><Segmented options={["settler","chieftain","warlord","prince","king","emperor","deity"]} value="prince" /></div>
        </div>
        <div className="param-row stack">
          <div className="label">starting era</div>
          <div className="control"><Segmented options={["ancient","classical","medieval","renaissance","industrial"]} value="ancient" /></div>
        </div>
        <div className="param-row stack">
          <div className="label"><ParamLabel help={<p>Game speed scales tech, civic, production, and unit costs uniformly.</p>}>game speed</ParamLabel></div>
          <div className="control"><Segmented options={["online","quick","std","epic","marathon"]} value="std" /></div>
          <div className="value">standard</div>
        </div>

        <hr className="divider" />
        <div className="h3" style={{ marginBottom: 10 }}>Victory conditions</div>
        {[
          ["Science", "Launch a colony to a habitable exoplanet."],
          ["Culture", "Attract more tourists than any other civ has domestic visitors."],
          ["Domination", "Capture every other civ's original capital."],
          ["Religion", "Convert every other civ to your founded religion."],
          ["Diplomacy", "Earn the most diplomatic favor in the World Congress."],
          ["Score", "Highest score when the time runs out."],
        ].map(([v, desc], i) => (
          <div className="param-row" key={v}>
            <div className="label"><ParamLabel help={<p>{desc}</p>}>{v.toLowerCase()}</ParamLabel></div>
            <div className="control"><Toggle on={i !== 4} /></div>
            <div className="value muted xsmall">{i === 4 ? "off" : "enabled"}</div>
          </div>
        ))}
      </div>
    </div>

    <div className="panel flush">
      <div className="panel-head"><span className="title">World dynamics</span></div>
      <div className="panel-body">
        <div className="param-row">
          <div className="label"><ParamLabel help={<p>Volcanoes, floods, droughts, blizzards. Higher intensity = more frequent &amp; severe.</p>}>disasters</ParamLabel></div>
          <div className="control"><Slider value={2} min={0} max={4} format={v=>["off","light","std","heavy","apocalyptic"][v]} /></div>
          <div className="value">std</div>
        </div>
        <div className="param-row">
          <div className="label">barbarians</div>
          <div className="control"><Slider value={2} min={0} max={4} format={v=>["off","rare","std","raging","horde"][v]} /></div>
          <div className="value">std</div>
        </div>
        <div className="param-row">
          <div className="label">city-states</div>
          <div className="control"><Slider value={12} min={0} max={24} /></div>
          <div className="value">12</div>
        </div>
        <div className="param-row">
          <div className="label"><ParamLabel help={<p>Affects how often AI civs declare war, denounce, or accept peace.</p>}>AI aggression</ParamLabel></div>
          <div className="control"><Slider value={50} format={v=>v<34?"passive":v>66?"warlike":"balanced"} /></div>
          <div className="value">balanced</div>
        </div>
        <div className="param-row">
          <div className="label"><ParamLabel help={<><p><strong>Historic</strong> — each leader behaves like their flavor text.</p><p><strong>Random</strong> — personalities reshuffled each game.</p><p><strong>Scripted</strong> — load a JSON personality pack.</p></>}>AI personality</ParamLabel></div>
          <div className="control stack-sm"><Segmented options={["historic","random","scripted"]} value="historic" /></div>
          <div className="value">historic</div>
        </div>
      </div>
    </div>
  </div>
);

// ----- step: players -----
const StepPlayers = () => {
  const slots = [
    { name: "Alice (you)", type: "human", civ: "Arabia · Saladin", you: true },
    { name: "—", type: "open", civ: "—", invite: true, open: true },
    { name: "AI", type: "ai", civ: "Rome · Trajan" },
    { name: "AI", type: "ai", civ: "Russia · Catherine" },
    { name: "AI", type: "ai", civ: "Random" },
    { name: "AI", type: "ai", civ: "Random" },
    { name: "AI", type: "ai", civ: "Random" },
    { name: "AI", type: "ai", civ: "Random" },
  ];
  return (
    <div className="wizard-body">
      <div className="panel flush">
        <div className="panel-head">
          <span className="title">Players &amp; slots</span>
          <span className="sub">// {slots.filter(s=>s.type==="human"||s.you).length}H · {slots.filter(s=>s.type==="ai").length}AI</span>
          <div style={{ marginLeft: "auto" }}><Btn variant="ghost" size="sm">+ slot</Btn></div>
        </div>
        <div className="panel-body">
          {slots.map((p, i) => (
            <div key={i} className={`slot ${p.you ? "you" : ""} ${p.open ? "open" : ""}`}>
              <span className="num">#{i + 1}</span>
              <div style={{ minWidth: 0 }}>
                <div className="row gap-sm center-y">
                  <span className="name">{p.name}</span>
                  <Tag variant={p.type === "human" ? "accent-soft" : p.type === "open" ? "" : "dim"}>{p.type}</Tag>
                </div>
                <div className="civ">{p.civ}</div>
              </div>
              {p.invite ? (
                <Popup title="Invite player" trigger="click" content={
                  <>
                    <PopupBody>
                      <p className="xsmall muted" style={{ marginBottom: 6 }}>
                        Paste any email, OpenID URL, atproto handle, or player ID:
                      </p>
                      <input className="input mono" placeholder="alice@…  did:plc:…  0xA9C3·…" />
                      <div className="row wrap gap-xs" style={{ marginTop: 8 }}>
                        <span className="xsmall muted" style={{ alignSelf: "center", marginRight: 4 }}>recent:</span>
                        <button className="chip">bob.bsky.social</button>
                        <button className="chip">carol@…</button>
                        <button className="chip">0xFE12·…</button>
                      </div>
                    </PopupBody>
                    <PopupActions right>
                      <Btn variant="ghost" size="sm">⎘ copy invite link</Btn>
                      <Btn variant="accent" size="sm">send invite</Btn>
                    </PopupActions>
                  </>
                }>
                  <Btn variant="primary" size="sm">invite</Btn>
                </Popup>
              ) : (
                <Popup title="Slot" size="narrow" trigger="click" content={
                  <PopupList items={[
                    { icon: "◔", label: "Change civ" },
                    { icon: "⚙", label: "AI personality" },
                    { icon: "↔", label: "Swap with…" },
                    "sep",
                    { icon: "✕", label: "Remove slot" },
                  ]} />
                }>
                  <Btn variant="ghost" size="sm">···</Btn>
                </Popup>
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="panel flush">
        <div className="panel-head"><span className="title">Turn mode</span></div>
        <div className="panel-body">
          <div className="param-row stack">
            <div className="label">turn timer</div>
            <div className="control"><Segmented options={["off","5min","10min","30min","24hr"]} value="off" /></div>
          </div>
          <div className="param-row">
            <div className="label"><ParamLabel help={<p>All human players take their turns at the same time. Falls back to play-by-turn for AI phases.</p>}>simultaneous</ParamLabel></div>
            <div className="control"><Toggle on={false} /></div>
            <div className="value muted xsmall">play-by-turn</div>
          </div>
          <div className="param-row">
            <div className="label">private game</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted xsmall">invite-only</div>
          </div>
          <div className="param-row">
            <div className="label">cross-play</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted xsmall">web · API</div>
          </div>
        </div>
      </div>
    </div>
  );
};

// ----- step: review -----
const StepReview = () => (
  <div className="wizard-body">
    <div className="panel flush">
      <div className="panel-head">
        <span className="title">Summary</span>
        <span className="sub">// last chance to tweak</span>
      </div>
      <div className="panel-body">
        {[
          ["map", "continents · standard · 84×54"],
          ["seed", "0xCAFE·B33F·1A77"],
          ["world", "4 bn yrs · sea 50% · temperate · normal rainfall"],
          ["civilization", "Saladin / Arabia"],
          ["difficulty", "prince · standard speed · ancient era"],
          ["victory", "science · culture · domination · religion · score"],
          ["dynamics", "disasters std · barbs std · 12 city-states · AI balanced"],
          ["players", "1 human + 1 invite pending + 6 AI"],
          ["turn mode", "play-by-turn · invite-only · cross-play"],
        ].map(([k, v]) => (
          <div key={k} className="param-row">
            <div className="label">{k}</div>
            <div className="control" style={{ fontSize: "var(--fs-sm)" }}>{v}</div>
            <div className="value"><Btn variant="bare" size="xs">edit</Btn></div>
          </div>
        ))}
      </div>
    </div>
    <div className="col">
      <div className="panel flush">
        <div className="panel-head">
          <span className="title">Final preview</span>
          <span className="sub">// world will be locked at generate</span>
        </div>
        <div style={{ padding: 1 }}>
          <div className="map-preview">
            <MiniMap seed={42} style={{ position: "absolute", inset: 0, width: "100%", height: "100%" }} />
          </div>
        </div>
      </div>
      <div className="panel" style={{ borderColor: "var(--ink)" }}>
        <p className="small" style={{ marginTop: 0, marginBottom: 12 }}>
          Generation deterministically builds the world from your seed. You can copy the seed
          to recreate this exact map elsewhere.
        </p>
        <Popup title="Generate world" content={
          <>
            <PopupBody>
              <p>This will:</p>
              <ol style={{ paddingLeft: 18, margin: "4px 0" }}>
                <li>Lock the seed and parameters</li>
                <li>Run procgen on the server</li>
                <li>Send invites to pending players</li>
                <li>Drop you into turn 1</li>
              </ol>
              <p className="muted xsmall">/api/games — typically &lt; 800ms</p>
            </PopupBody>
            <PopupActions right>
              <Btn variant="accent" size="sm">⌬ generate now</Btn>
            </PopupActions>
          </>
        }>
          <Btn variant="accent" size="lg" className="block">⌬  Generate world</Btn>
        </Popup>
        <p className="muted xsmall" style={{ textAlign: "center", marginTop: 10, marginBottom: 0 }}>
          // calls /api/games · returns game_id · routes you to gameplay client
        </p>
      </div>
    </div>
  </div>
);

const NewGame = () => {
  const [step, setStep] = React.useState("map");
  const [advanced, setAdvanced] = React.useState(false);
  const idx = STEPS.findIndex(s => s.id === step);
  const next = () => idx < STEPS.length - 1 && setStep(STEPS[idx + 1].id);
  const prev = () => idx > 0 && setStep(STEPS[idx - 1].id);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <div className="content-header">
        <div className="title">New game</div>
        <span className="crumbs">// procedural worldgen</span>
        <div className="actions">
          <Popup title="Presets" size="narrow" trigger="click" content={
            <PopupList items={[
              { icon: "⎘", label: "Standard prince",   desc: "default" },
              { icon: "⎘", label: "Deity duel" },
              { icon: "⎘", label: "Slow marathon" },
              "sep",
              { icon: "↑", label: "Save current as preset" },
              { icon: "↓", label: "Import JSON…" },
            ]} />
          }>
            <Btn variant="ghost" size="sm">presets</Btn>
          </Popup>
        </div>
      </div>

      <StepStrip current={step} onJump={setStep} />

      <div style={{ flex: 1, overflow: "auto", paddingBottom: 12 }}>
        {step === "map" && <StepMap advanced={advanced} setAdvanced={setAdvanced} />}
        {step === "civ" && <StepCiv />}
        {step === "rules" && <StepRules />}
        {step === "players" && <StepPlayers />}
        {step === "review" && <StepReview />}
      </div>

      <div className="wizard-footer">
        <Btn variant="ghost" size="sm" disabled={idx === 0} onClick={prev}>← back</Btn>
        <span>
          <span className="kbd">⏎</span> next · <span className="kbd">⌘K</span> jump · <span className="kbd">esc</span> cancel
        </span>
        {step === "review"
          ? <Btn variant="accent">⌬ generate</Btn>
          : <Btn variant="primary" onClick={next}>next →</Btn>
        }
      </div>
    </div>
  );
};

window.NewGame = NewGame;
