// Hi-fi Menu shell + Ongoing Games + Profile

const NAV = [
  { id: "ongoing",  icon: "▣", label: "Ongoing games", badge: 3 },
  { id: "newgame",  icon: "+", label: "New game" },
  { id: "profile",  icon: "◔", label: "Profile" },
];
const NAV_SECONDARY = [
  { id: "friends",  icon: "◎", label: "Friends" },
  { id: "presets",  icon: "≡", label: "Presets" },
  { id: "docs",     icon: "?", label: "Docs" },
];

const Shell = ({ tab, setTab, children }) => (
  <div className="menu-shell">
    <aside className="sidebar">
      <Popup title="Account" size="narrow" trigger="click" content={
        <PopupList items={[
          { icon: "◔", label: "Profile & settings" },
          { icon: "⎘", label: "Copy player ID", desc: "0xA9C3·7F12" },
          { icon: "▦", label: "Show invite QR" },
          "sep",
          { icon: "→", label: "Sign out" },
        ]} />
      }>
        <button className="user-card" style={{
          background: "transparent", border: "none", width: "100%",
          fontFamily: "inherit", textAlign: "left", cursor: "pointer", padding: 0,
        }}>
          <div className="user-card">
            <div className="avatar-sm">A</div>
            <div style={{ minWidth: 0 }}>
              <div className="name">Alice</div>
              <div className="uid">0xA9C3·7F12</div>
            </div>
            <span className="chev">▾</span>
          </div>
        </button>
      </Popup>

      <div className="group-label">PLAY</div>
      {NAV.map(n => (
        <button key={n.id} className="nav-item" aria-current={tab === n.id} onClick={() => setTab(n.id)}>
          <span className="icon">{n.icon}</span>
          <span>{n.label}</span>
          {n.badge && <span className="badge">{n.badge}</span>}
        </button>
      ))}

      <div className="group-label">MORE</div>
      {NAV_SECONDARY.map(n => (
        <Popup key={n.id} title={n.label} content={
          <PopupBody>
            <p className="muted xsmall">// {n.label.toLowerCase()} surface — out of scope for this hi-fi pass.</p>
          </PopupBody>
        }>
          <button className="nav-item" style={{ width: "100%" }}>
            <span className="icon">{n.icon}</span>
            <span>{n.label}</span>
          </button>
        </Popup>
      ))}

      <div className="footer">
        <span>v0.1.0</span>
        <Popup title="Server status" size="narrow" content={
          <PopupBody>
            <div className="kv xsmall">
              <span className="k">api</span><span style={{ color: "var(--good)" }}>● operational</span>
              <span className="k">latency</span><span>42 ms</span>
              <span className="k">region</span><span>auto</span>
            </div>
          </PopupBody>
        }>
          <span className="trigger" style={{ color: "var(--good)" }}>● online</span>
        </Popup>
      </div>
    </aside>
    <div className="content">{children}</div>
  </div>
);

window.Shell = Shell;

// ===== Ongoing Games =====

const sampleGames = [
  { id:"g_2049", name:"Cradle of the Indus", leader:"Saladin · Arabia",
    turn:142, era:"Medieval", score:814, difficulty:"Prince",
    players:"1H · 7AI", map:"Continents · Std", last:"2h ago",
    status:"your-turn", notif:3, mp:false },
  { id:"g_2051", name:"Test seed 0xCAFE", leader:"Trajan · Rome",
    turn:12, era:"Ancient", score:41, difficulty:"Settler",
    players:"1H · 3AI", map:"Archipelago · Sm", last:"10m ago",
    status:"your-turn", notif:1, mp:false },
  { id:"g_2044", name:"Friday night MP", leader:"Cleopatra · Egypt",
    turn:56, era:"Classical", score:220, difficulty:"Prince",
    players:"4H · 4AI", map:"Fractal · Std", last:"5d ago",
    status:"your-turn", notif:2, mp:true },
  { id:"g_2050", name:"Long Winter", leader:"Catherine · Russia",
    turn:87, era:"Renaissance", score:502, difficulty:"King",
    players:"3H · 5AI", map:"Pangaea · Large", last:"yesterday",
    status:"waiting", notif:0, mp:true },
  { id:"g_2046", name:"Pacific Hegemony", leader:"Hojo · Japan",
    turn:230, era:"Modern", score:1192, difficulty:"Emperor",
    players:"1H · 9AI", map:"Continents · Huge", last:"3d ago",
    status:"waiting", notif:0, mp:false },
  { id:"g_2030", name:"Tutorial run", leader:"Gandhi · India",
    turn:312, era:"Atomic", score:2410, difficulty:"Settler",
    players:"1H · 3AI", map:"Continents · Std", last:"2 wk ago",
    status:"completed", notif:0, mp:false },
];

const GameTilePopup = ({ g }) => (
  <>
    <PopupBody>
      <div style={{ fontWeight: 600, fontSize: "var(--fs-md)", marginBottom: 4 }}>{g.name}</div>
      <p className="muted xsmall" style={{ marginBottom: 8 }}>{g.leader} · {g.map}</p>
      <div className="kv xsmall">
        <span className="k">turn</span><span>{g.turn} · {g.era}</span>
        <span className="k">score</span><span>{g.score}</span>
        <span className="k">difficulty</span><span>{g.difficulty}</span>
        <span className="k">players</span><span>{g.players}</span>
        <span className="k">last played</span><span>{g.last}</span>
        {g.notif > 0 && (<><span className="k">events</span><span style={{ color: "var(--accent)" }}>{g.notif} pending</span></>)}
      </div>
    </PopupBody>
    <PopupActions right>
      <Btn variant="ghost" size="sm">Notes</Btn>
      <Btn variant={g.status === "your-turn" ? "accent" : "primary"} size="sm">→ Resume</Btn>
    </PopupActions>
  </>
);

const NotesPopup = ({ g }) => (
  <PopupBody>
    <p className="xsmall muted" style={{ letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: 6 }}>Notes — {g.name}</p>
    <p>Stuck in a tech-trade war with Trajan. Save coal for railroads — don't burn it on industrial zones yet.</p>
    <p className="muted xsmall">Edited 3h ago · markdown ok</p>
  </PopupBody>
);

const OngoingGames = ({ onNew }) => (
  <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
    <div className="content-header">
      <div className="title">Ongoing games</div>
      <span className="crumbs">{sampleGames.length} games · {sampleGames.filter(g=>g.status==="your-turn").length} awaiting you</span>
      <div className="actions">
        <span className="muted xsmall">view</span>
        <Segmented options={[{value:"large",label:"▣ tiles"},{value:"compact",label:"≡ list"}]} value="large" />
        <Btn variant="accent" onClick={onNew}>+ New game</Btn>
      </div>
    </div>

    <div className="filter-bar">
      <span className="muted xsmall" style={{ paddingLeft: 4 }}>⌕</span>
      <input className="filter-search" placeholder="search games and notes…" />
      <span className="sep-v"></span>
      <button className="chip active">your turn <span className="x">×</span></button>
      <button className="chip">waiting</button>
      <button className="chip">completed</button>
      <button className="chip">multiplayer</button>
      <Popup title="Add filter" size="narrow" trigger="click" content={
        <PopupList items={[
          { icon: "◆", label: "Difficulty",   desc: "settler — deity" },
          { icon: "◎", label: "Map type",     desc: "continents, pangaea…" },
          { icon: "⏿", label: "Player count" },
          { icon: "⊞", label: "Map size" },
          "sep",
          { icon: "↑", label: "Sort by turn" },
          { icon: "↓", label: "Sort by date" },
        ]} />
      }>
        <button className="chip">+ filter</button>
      </Popup>
      <span style={{ marginLeft: "auto" }} className="muted xsmall">sort</span>
      <button className="chip">recent ↓</button>
    </div>

    <div style={{ flex: 1, overflow: "auto" }}>
      <div className="games-grid">
        {sampleGames.map((g, i) => (
          <Popup key={g.id} title="game preview" content={<GameTilePopup g={g} />}>
            <div className={`game-tile ${g.status === "your-turn" ? "your-turn" : ""}`} style={{ width: "100%" }}>
              <div className="tile-head">
                <div>
                  <div className="tile-name">{g.name}</div>
                  <div className="leader">{g.leader}</div>
                </div>
                <div className="row gap-xs">
                  {g.mp && <Tag>MP</Tag>}
                  {g.notif > 0 && <Tag variant="accent">!{g.notif}</Tag>}
                </div>
              </div>
              <div className="map-thumb">
                <MiniMap seed={i + 1} style={{ width: "100%", height: "100%" }} />
              </div>
              <div className="stats">
                <div className="row-stat"><span className="k">turn</span><span className="v">{g.turn}</span></div>
                <div className="row-stat"><span className="k">era</span><span className="v">{g.era}</span></div>
                <div className="row-stat"><span className="k">diff</span><span className="v">{g.difficulty}</span></div>
                <div className="row-stat"><span className="k">score</span><span className="v">{g.score}</span></div>
                <div className="row-stat"><span className="k">players</span><span className="v">{g.players}</span></div>
                <div className="row-stat"><span className="k">last</span><span className="v">{g.last}</span></div>
              </div>
              <div className="actions">
                <Popup title="Notes" content={<NotesPopup g={g} />}>
                  <Btn variant="ghost" size="sm">📝 Notes</Btn>
                </Popup>
                <Btn variant={g.status === "your-turn" ? "accent" : "primary"} size="sm" style={{ marginLeft: "auto" }}>
                  {g.status === "your-turn" ? "→ Resume" : g.status === "waiting" ? "Open" : "Review"}
                </Btn>
                <Popup title="More" size="narrow" trigger="click" content={
                  <PopupList items={[
                    { icon: "◑", label: "View summary" },
                    { icon: "⎘", label: "Copy game ID", desc: g.id },
                    { icon: "↗", label: "Share invite link" },
                    "sep",
                    { icon: "⊟", label: "Archive" },
                    { icon: "⊗", label: "Resign / delete" },
                  ]} />
                }>
                  <Btn variant="ghost" size="sm">···</Btn>
                </Popup>
              </div>
            </div>
          </Popup>
        ))}
      </div>
    </div>
  </div>
);

window.OngoingGames = OngoingGames;

// ===== Profile =====

const Profile = () => (
  <div style={{ flex: 1, overflow: "auto" }}>
    <div className="content-header">
      <div className="title">Profile &amp; settings</div>
      <span className="crumbs">
        player_id:{" "}
        <Popup title="player ID" content={
          <>
            <PopupBody>
              <p>Your unique identifier on the platform. Share it with friends so they can invite you.</p>
              <div className="kv xsmall" style={{ marginTop: 6 }}>
                <span className="k">format</span><span>16-char hex, dot-grouped</span>
                <span className="k">scope</span><span>global</span>
              </div>
            </PopupBody>
            <PopupActions right>
              <Btn variant="ghost" size="sm">⎘ Copy</Btn>
              <Btn variant="primary" size="sm">▦ Show QR</Btn>
            </PopupActions>
          </>
        }>
          <code className="trigger" style={{ fontFamily: "var(--font-mono)" }}>0xA9C3·7F12·EE04</code>
        </Popup>
      </span>
    </div>

    <div className="profile-grid">
      <div className="panel">
        <div className="col" style={{ alignItems: "center", gap: 12 }}>
          <div className="avatar">A</div>
          <Popup title="Avatar" size="narrow" trigger="click" content={
            <PopupList items={[
              { icon: "↑", label: "Upload image", desc: "PNG, JPG, ≤ 2 MB" },
              { icon: "◔", label: "Use Gravatar", desc: "alice@example.com" },
              { icon: "A", label: "Initials only" },
              "sep",
              { icon: "✕", label: "Remove avatar" },
            ]} />
          }>
            <Btn variant="ghost" size="sm">change</Btn>
          </Popup>
        </div>
        <hr className="divider" />
        <div className="h3" style={{ marginBottom: 10 }}>Quick actions</div>
        <div className="col" style={{ gap: 4 }}>
          <Btn variant="bare" size="sm" style={{ justifyContent: "flex-start", width: "100%" }}>⎘ Copy player ID</Btn>
          <Btn variant="bare" size="sm" style={{ justifyContent: "flex-start", width: "100%" }}>▦ Show invite QR</Btn>
          <Btn variant="bare" size="sm" style={{ justifyContent: "flex-start", width: "100%" }}>↓ Export save data</Btn>
          <Btn variant="bare" size="sm" style={{ justifyContent: "flex-start", width: "100%", color: "var(--accent)" }}>→ Sign out</Btn>
        </div>
      </div>

      <div className="col">
        <div className="panel">
          <div className="h3" style={{ marginBottom: 14 }}>Profile</div>
          <div className="field">
            <label>Preferred name</label>
            <input className="input" defaultValue="Alice" />
            <span className="hint">Shown to other players in invites &amp; chat.</span>
          </div>
          <div className="field">
            <label>Pronouns (optional)</label>
            <input className="input" defaultValue="she/her" />
          </div>
          <div className="field">
            <label>Bio</label>
            <textarea className="input" rows={2} defaultValue="Plays slow. Reads everything." />
            <span className="hint">Markdown supported. Visible on invite cards.</span>
          </div>
        </div>

        <div className="panel">
          <div className="row between center-y" style={{ marginBottom: 12 }}>
            <div className="h3">Linked identities</div>
            <Popup title="Link another" size="narrow" trigger="click" content={
              <PopupList items={[
                { icon: "@", label: "Add another email" },
                { icon: "○", label: "Connect OpenID" },
                { icon: "@", label: "Connect atproto" },
              ]} />
            }>
              <Btn variant="ghost" size="sm">+ link another</Btn>
            </Popup>
          </div>

          <div className="id-row primary">
            <span className="id-type">EMAIL · primary</span>
            <span className="id-val">alice@example.com</span>
            <Popup title="Manage" size="narrow" trigger="click" content={
              <PopupList items={[
                { icon: "✓", label: "Verify email" },
                { icon: "★", label: "Set as primary", desc: "already primary" },
                { icon: "✉", label: "Change address" },
                "sep",
                { icon: "✕", label: "Unlink" },
              ]} />
            }>
              <Btn variant="bare" size="sm">manage</Btn>
            </Popup>
          </div>
          <div className="id-row">
            <span className="id-type">OPENID</span>
            <span className="id-val">google.com / 110293·a73f</span>
            <Btn variant="bare" size="sm">unlink</Btn>
          </div>
          <div className="id-row">
            <span className="id-type">OPENID</span>
            <span className="id-val">github.com/alice</span>
            <Btn variant="bare" size="sm">unlink</Btn>
          </div>
          <div className="id-row">
            <span className="id-type">ATPROTO</span>
            <span className="id-val">did:plc:abcd1234efgh5678 · alice.bsky.social</span>
            <Btn variant="bare" size="sm">unlink</Btn>
          </div>

          <p className="muted xsmall" style={{ marginTop: 10 }}>
            All four identities map to the same player. Friends can find you by any of them.
          </p>
        </div>

        <div className="panel">
          <div className="h3" style={{ marginBottom: 12 }}>Preferences</div>
          <div className="param-row">
            <div className="label">Density</div>
            <div className="control"><Segmented options={["compact","comfortable","spacious"]} value="comfortable" /></div>
            <div className="value muted xsmall">→ tweaks panel</div>
          </div>
          <div className="param-row">
            <div className="label">Color scheme</div>
            <div className="control"><Segmented options={["paper","ink","auto"]} value="paper" /></div>
            <div className="value muted xsmall">paper</div>
          </div>
          <div className="param-row">
            <div className="label">Keyboard nav</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted xsmall">vim bindings</div>
          </div>
          <div className="param-row">
            <div className="label">Turn notifications</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted xsmall">email + push</div>
          </div>
          <div className="param-row">
            <div className="label">Discoverable by ID</div>
            <div className="control"><Toggle on={true} /></div>
            <div className="value muted xsmall">others can invite</div>
          </div>
        </div>
      </div>
    </div>
  </div>
);

window.Profile = Profile;
