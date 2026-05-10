// Ongoing games — 3 density variants

const sampleGames = [
  {
    id: "g_2049",
    name: "Cradle of the Indus",
    leader: "Saladin · Arabia",
    turn: 142, era: "Medieval", score: 814,
    difficulty: "Prince", players: "1H · 7AI", map: "Continents · Std",
    last: "2h ago", status: "your-turn", notif: 3, mp: false,
  },
  {
    id: "g_2050",
    name: "Long Winter",
    leader: "Catherine · Russia",
    turn: 87, era: "Renaissance", score: 502,
    difficulty: "King", players: "3H · 5AI", map: "Pangaea · Large",
    last: "yesterday", status: "waiting", notif: 0, mp: true,
  },
  {
    id: "g_2051",
    name: "Test seed 0xCAFE",
    leader: "Trajan · Rome",
    turn: 12, era: "Ancient", score: 41,
    difficulty: "Settler", players: "1H · 3AI", map: "Archipelago · Sm",
    last: "10 min ago", status: "your-turn", notif: 1, mp: false,
  },
  {
    id: "g_2046",
    name: "Pacific Hegemony",
    leader: "Hojo · Japan",
    turn: 230, era: "Modern", score: 1192,
    difficulty: "Emperor", players: "1H · 9AI", map: "Continents · Huge",
    last: "3d ago", status: "waiting", notif: 0, mp: false,
  },
  {
    id: "g_2044",
    name: "Friday night MP",
    leader: "Cleopatra · Egypt",
    turn: 56, era: "Classical", score: 220,
    difficulty: "Prince", players: "4H · 4AI", map: "Fractal · Std",
    last: "5d ago", status: "your-turn", notif: 2, mp: true,
  },
  {
    id: "g_2030",
    name: "Tutorial run",
    leader: "Gandhi · India",
    turn: 312, era: "Atomic", score: 2410,
    difficulty: "Settler", players: "1H · 3AI", map: "Continents · Std",
    last: "2 wk ago", status: "completed", notif: 0, mp: false,
  },
];

const statusLabel = s => s === "your-turn" ? "Your turn"
  : s === "waiting" ? "AI thinking…" : "Completed";

const FilterBar = () => (
  <div className="filter-bar">
    <input className="sk-input filter-search" placeholder="search games / notes…" />
    <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>filter:</span>
    <button className="filter-chip active">your turn <span className="x">×</span></button>
    <button className="filter-chip">waiting</button>
    <button className="filter-chip">completed</button>
    <button className="filter-chip">multiplayer</button>
    <button className="filter-chip">+ difficulty</button>
    <button className="filter-chip">+ map</button>
    <button className="filter-chip">+ players</button>
    <span className="muted" style={{ fontSize: "var(--fs-sm)", marginLeft: "auto" }}>sort:</span>
    <button className="filter-chip">recent ↓</button>
  </div>
);

// Variant A: Large tile grid, prominent map thumbnail
const TilesLarge = () => (
  <div className="games-grid large">
    {sampleGames.map((g, i) => (
      <div key={g.id} className={`game-tile ${g.status === "your-turn" ? "your-turn" : ""}`}>
        <div className="tile-head">
          <div>
            <div className="tile-name">{g.name}</div>
            <div className="leader">{g.leader}</div>
          </div>
          <div className="row" style={{ gap: 4 }}>
            {g.mp && <Tag variant="dim">MP</Tag>}
            {g.notif > 0 && <Tag variant="accent">!{g.notif}</Tag>}
          </div>
        </div>
        <MapThumb seed={i + 1} style={{ height: 70 }} />
        <div className="stats">
          <span><span className="k">turn</span> {g.turn} · {g.era}</span>
          <span><span className="k">score</span> {g.score}</span>
          <span><span className="k">diff</span> {g.difficulty}</span>
          <span><span className="k">players</span> {g.players}</span>
          <span><span className="k">map</span> {g.map}</span>
          <span><span className="k">last</span> {g.last}</span>
        </div>
        <div className="actions">
          <Btn variant={g.status === "your-turn" ? "accent" : "primary"} size="sm">
            {g.status === "your-turn" ? "→ Resume" : "Open"}
          </Btn>
          <Btn variant="ghost" size="sm">Notes</Btn>
          <Btn variant="ghost" size="sm" style={{ marginLeft: "auto" }}>···</Btn>
        </div>
      </div>
    ))}
  </div>
);

// Variant B: Medium 3-up cards, no map thumbnail
const TilesMedium = () => (
  <div className="games-grid medium">
    {sampleGames.map((g, i) => (
      <div key={g.id} className={`game-tile ${g.status === "your-turn" ? "your-turn" : ""}`}>
        <div className="tile-head">
          <div>
            <div className="tile-name">{g.name}</div>
            <div className="leader">{g.leader}</div>
          </div>
          {g.notif > 0 && <Tag variant="accent">!{g.notif}</Tag>}
        </div>
        <div style={{ fontSize: "var(--fs-sm)" }}>
          <div className="row between" style={{ marginBottom: 2 }}>
            <span className="muted">turn</span><span>{g.turn} · {g.era}</span>
          </div>
          <div className="row between" style={{ marginBottom: 2 }}>
            <span className="muted">difficulty</span><span>{g.difficulty}</span>
          </div>
          <div className="row between" style={{ marginBottom: 2 }}>
            <span className="muted">players</span><span>{g.players}</span>
          </div>
          <div className="row between" style={{ marginBottom: 2 }}>
            <span className="muted">map</span><span>{g.map}</span>
          </div>
          <div className="row between">
            <span className="muted">last played</span><span>{g.last}</span>
          </div>
        </div>
        <div className="row between center-y">
          <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>{statusLabel(g.status)}</span>
          <div className="row" style={{ gap: 6 }}>
            <Btn size="sm" variant="ghost">notes</Btn>
            <Btn size="sm" variant={g.status === "your-turn" ? "accent" : "primary"}>resume</Btn>
          </div>
        </div>
      </div>
    ))}
  </div>
);

// Variant C: Compact rows (table-like)
const TilesCompact = () => (
  <div className="games-grid compact">
    <div className="game-row head-row">
      <span></span>
      <span>NAME / LEADER</span>
      <span>STATUS</span>
      <span>TURN</span>
      <span>DIFF</span>
      <span>PLAYERS · MAP</span>
      <span>LAST</span>
      <span></span>
    </div>
    {sampleGames.map(g => (
      <div key={g.id} className={`game-row ${g.status}`}>
        <span className="status-dot" />
        <span>
          <div className="gname">{g.name}{g.mp && <span style={{ marginLeft: 6, fontSize: 10, color: "var(--dim)" }}>MP</span>}</div>
          <div className="muted">{g.leader}</div>
        </span>
        <span>{statusLabel(g.status)}{g.notif > 0 && <span style={{ color: "var(--accent)", marginLeft: 6 }}>!{g.notif}</span>}</span>
        <span>{g.turn}</span>
        <span>{g.difficulty}</span>
        <span>{g.players} / {g.map}</span>
        <span className="muted">{g.last}</span>
        <span style={{ display: "flex", gap: 4, justifyContent: "flex-end" }}>
          <Btn size="sm" variant="ghost">📝</Btn>
          <Btn size="sm" variant={g.status === "your-turn" ? "accent" : "primary"}>→</Btn>
        </span>
      </div>
    ))}
  </div>
);

const OngoingGames = ({ density, onNew }) => {
  // density "spacious" -> Large, "comfortable" -> Medium, "compact" -> Compact
  // but we ALSO let users pick layout independently. Default by density:
  const [layout, setLayout] = React.useState(
    density === "spacious" ? "large" : density === "compact" ? "compact" : "medium"
  );

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <div className="content-header">
        <div className="title">Ongoing games</div>
        <span className="crumbs">{sampleGames.length} games · 3 awaiting you</span>
        <div style={{ marginLeft: "auto" }} className="row" style={{ gap: 8, alignItems: "center" }}>
          <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>view:</span>
          <Segmented
            options={[{value: "large", label: "▣"}, {value: "medium", label: "▦"}, {value: "compact", label: "≡"}]}
            value={layout} onChange={setLayout} />
          <Btn variant="accent" onClick={onNew}>＋ New game</Btn>
        </div>
      </div>
      <FilterBar />
      <div style={{ flex: 1, overflow: "auto" }}>
        {layout === "large" && <TilesLarge />}
        {layout === "medium" && <TilesMedium />}
        {layout === "compact" && <TilesCompact />}
      </div>
      <div className="hand-arrow" style={{ bottom: 18, right: 24 }}>
        ↑ user picks layout<br/>independently of density
      </div>
    </div>
  );
};

window.OngoingGames = OngoingGames;
