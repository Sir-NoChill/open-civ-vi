// Menu shell — 3 sidebar variants

const NAV = [
  { id: "ongoing",  icon: "▣", label: "Ongoing games", badge: 3 },
  { id: "newgame",  icon: "＋", label: "New game" },
  { id: "profile",  icon: "◔", label: "Profile" },
];
const NAV_SECONDARY = [
  { id: "friends",  icon: "◎", label: "Friends" },
  { id: "presets",  icon: "≡", label: "Presets" },
  { id: "docs",     icon: "?", label: "Docs" },
];

// ----- Variant A: persistent left rail -----
const ShellA = ({ tab, setTab, children }) => (
  <div className="menu-shell">
    <aside className="sidebar">
      <div className="row" style={{ gap: 8, padding: "0 6px 12px", borderBottom: "1.2px dashed var(--dim-2)", marginBottom: 8 }}>
        <div className="avatar" style={{ width: 30, height: 30, fontSize: 18 }}>A</div>
        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: "var(--fs-sm)", fontWeight: 700 }}>Alice</div>
          <div className="muted" style={{ fontSize: 10, fontFamily: "monospace" }}>0xA9C3·7F12</div>
        </div>
      </div>
      <div className="group-label">PLAY</div>
      {NAV.map(n => (
        <button key={n.id} className="nav-item" aria-current={tab === n.id} onClick={() => setTab(n.id)}>
          <span className="icon">{n.icon}</span>
          <span className="label">{n.label}</span>
          {n.badge && <span className="badge">{n.badge}</span>}
        </button>
      ))}
      <div className="group-label">MORE</div>
      {NAV_SECONDARY.map(n => (
        <button key={n.id} className="nav-item">
          <span className="icon">{n.icon}</span>
          <span className="label">{n.label}</span>
        </button>
      ))}
      <div style={{ marginTop: "auto", padding: "8px 4px", color: "var(--dim)", fontSize: 10 }}>
        v0.1.0 · status: ●
      </div>
    </aside>
    <div className="content">{children}</div>
  </div>
);

// ----- Variant B: collapsible icon rail -----
const ShellB = ({ tab, setTab, children }) => {
  const [collapsed, setCollapsed] = React.useState(true);
  return (
    <div className={`menu-shell ${collapsed ? "collapsed" : ""}`}>
      <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
        <button className="nav-item" onClick={() => setCollapsed(c => !c)} title="toggle sidebar">
          <span className="icon">≡</span>
          <span className="label">collapse</span>
        </button>
        <div className="group-label">PLAY</div>
        {NAV.map(n => (
          <button key={n.id} className="nav-item" aria-current={tab === n.id} onClick={() => setTab(n.id)}
                  title={n.label}>
            <span className="icon">{n.icon}</span>
            <span className="label">{n.label}</span>
            {n.badge && !collapsed && <span className="badge">{n.badge}</span>}
          </button>
        ))}
        <div className="group-label">MORE</div>
        {NAV_SECONDARY.map(n => (
          <button key={n.id} className="nav-item" title={n.label}>
            <span className="icon">{n.icon}</span>
            <span className="label">{n.label}</span>
          </button>
        ))}
      </aside>
      <div className="content">{children}</div>
    </div>
  );
};

// ----- Variant C: top tab bar -----
const ShellC = ({ tab, setTab, children }) => (
  <div className="menu-shell top-tabs">
    <div className="top-tabbar">
      {NAV.map(n => (
        <button key={n.id} className="tab" aria-current={tab === n.id} onClick={() => setTab(n.id)}>
          <span style={{ marginRight: 6, color: "var(--dim)" }}>{n.icon}</span>{n.label}
          {n.badge && <span className="badge">{n.badge}</span>}
        </button>
      ))}
      <div style={{ marginLeft: "auto", padding: "0 6px", display: "flex", gap: 6, alignItems: "center", marginBottom: 6 }}>
        {NAV_SECONDARY.map(n => <Btn key={n.id} size="sm" variant="ghost">{n.label}</Btn>)}
        <div className="avatar" style={{ width: 28, height: 28, fontSize: 16 }}>A</div>
      </div>
    </div>
    <div className="content">{children}</div>
  </div>
);

window.ShellVariants = { A: ShellA, B: ShellB, C: ShellC };
