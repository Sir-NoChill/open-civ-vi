// Landing page — 3 variants

const LandingA = ({ onLogin }) => (
  // Variant A: Centered, single column, ASCII-banner vibe
  <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "center", padding: 40, position: "relative" }}>
    <div style={{ maxWidth: 720, margin: "0 auto", textAlign: "center" }}>
      <div style={{ fontSize: "var(--fs-sm)", color: "var(--dim)", letterSpacing: "0.3em", marginBottom: 20 }}>
        v0.1.0 / pre-alpha / open source
      </div>
      <pre style={{
        fontFamily: "JetBrains Mono, monospace",
        fontWeight: 700,
        fontSize: 14,
        lineHeight: 1.1,
        color: "var(--ink)",
        margin: "0 0 22px",
      }}>{`
  ┌─┐┌─┐┌─┐┌┐┌  ╦ ╦ ╦
  │ │├─┘├┤ │││  ║ ║ ║
  └─┘┴  └─┘┘└┘  ╩ ╩ ╩
       open  4x  vi
`}</pre>
      <h1 className="h1" style={{ marginBottom: 14 }}>
        A 4X game without the graphics tax<Caret/>
      </h1>
      <p style={{ color: "var(--dim)", maxWidth: 520, margin: "0 auto 28px" }}>
        [ blurb placeholder — civ-vi inspired, deeply moddable, runs on a potato.
        author will write this. keep it short. keep it sharp. ]
      </p>
      <div className="row center-x" style={{ gap: 10 }}>
        <Btn variant="accent" onClick={onLogin}>→ Sign in &amp; play</Btn>
        <Btn variant="ghost">View source</Btn>
      </div>
      <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 18, letterSpacing: "0.06em" }}>
        EMAIL · OPENID · ATPROTO
      </div>
    </div>
    <div className="hand-arrow" style={{ top: 60, right: 60, transform: "rotate(8deg)" }}>
      ascii logo →<br/>placeholder
    </div>
  </div>
);

const LandingB = ({ onLogin }) => (
  // Variant B: Split screen — text left, terminal/preview right
  <div style={{ flex: 1, display: "grid", gridTemplateColumns: "1fr 1fr", gap: 40, padding: 40 }}>
    <div style={{ alignSelf: "center" }}>
      <div className="row" style={{ gap: 8, marginBottom: 14, alignItems: "center" }}>
        <Tag variant="solid">OPEN4X·VI</Tag>
        <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>// pre-alpha</span>
      </div>
      <h1 className="h1" style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}>
        Civ-style strategy<br/>
        <span style={{ color: "var(--accent)" }}>built for power users.</span>
      </h1>
      <p className="muted" style={{ marginBottom: 8 }}>
        [ author blurb goes here ]
      </p>
      <p className="muted" style={{ marginBottom: 24, fontSize: "var(--fs-sm)" }}>
        Keyboard-first. API-driven. No engine, no install, no nonsense.
      </p>
      <div className="row" style={{ gap: 10 }}>
        <Btn variant="accent" onClick={onLogin}>Sign in</Btn>
        <Btn>Read the docs</Btn>
      </div>
      <div className="row" style={{ gap: 18, marginTop: 28, fontSize: "var(--fs-sm)", color: "var(--dim)" }}>
        <span>★ keyboard nav</span>
        <span>★ scriptable</span>
        <span>★ self-hostable</span>
      </div>
    </div>
    <Box className="fill" style={{ padding: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}>
      <div style={{ padding: "8px 12px", borderBottom: "1.5px dashed var(--ink)", fontSize: "var(--fs-sm)", color: "var(--dim)" }}>
        $ open4x --preview
      </div>
      <div style={{ flex: 1, padding: 16, fontSize: "var(--fs-sm)", color: "var(--ink-2)", lineHeight: 1.6 }}>
        <CmdLine>booting world (seed: 0xCAFE)</CmdLine>
        <CmdLine>mapgen: continents · standard · age=4bn</CmdLine>
        <CmdLine>civs: 8 / city-states: 12 / barbs: rare</CmdLine>
        <CmdLine>rendering disabled — text mode</CmdLine>
        <div style={{ marginTop: 12 }}>
          <pre style={{ fontSize: 11, lineHeight: 1.1, color: "var(--ink)", margin: 0 }}>{`
  . . ~ ~ ▲ ▲ . . ~ ~ ~ . . ▲ ▲
  ~ ~ ▲ ▣ ▲ . . ~ . . ▲ ▣ ▲ . .
  . ~ . . ▲ . . ~ ~ ~ ~ . . ▲ .
  ▣ . . ~ ~ . . ▲ ▣ ▲ . ~ . . .
  ▲ ▲ ~ ~ . . ▲ . . . ~ ~ ▣ ▲ .
`}</pre>
        </div>
        <CmdLine>ready. <Caret/></CmdLine>
      </div>
    </Box>
  </div>
);

const LandingC = ({ onLogin }) => (
  // Variant C: Full-bleed manifesto / single huge type
  <div style={{ flex: 1, display: "flex", flexDirection: "column", padding: "32px 40px", position: "relative" }}>
    <div className="row between" style={{ marginBottom: 40 }}>
      <div style={{ fontWeight: 700, letterSpacing: "0.1em" }}>OPEN4X·VI</div>
      <div className="row" style={{ gap: 10, fontSize: "var(--fs-sm)", color: "var(--dim)" }}>
        <span>github</span><span>·</span><span>docs</span><span>·</span><span>discord</span>
      </div>
    </div>
    <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "center" }}>
      <div style={{ fontSize: 64, fontWeight: 700, lineHeight: 0.95, letterSpacing: "-0.02em", maxWidth: 920 }}>
        Strategy as <br/>
        <span style={{ color: "var(--accent)" }}>data,</span> not<br/>
        spectacle.
      </div>
      <p style={{ maxWidth: 520, color: "var(--dim)", marginTop: 28, lineHeight: 1.6 }}>
        [ blurb goes here. four sentences max. tells the story of why we
        built a civ-style game without the rendering. tells you it's free
        and open. tells you to log in. ]
      </p>
      <div className="row" style={{ gap: 12, marginTop: 28 }}>
        <Btn variant="accent" onClick={onLogin}>Sign in to play →</Btn>
        <Btn variant="ghost">What is this?</Btn>
      </div>
    </div>
    <div className="row between" style={{ marginTop: 30, fontSize: "var(--fs-sm)", color: "var(--dim)" }}>
      <span>v0.1.0-prealpha</span>
      <span>// auth: email · openid · atproto</span>
      <span>self-hostable</span>
    </div>
    <div className="hand-arrow" style={{ top: 220, right: 60, transform: "rotate(-6deg)", textAlign: "center" }}>
      manifesto-style<br/>copy ↘
    </div>
  </div>
);

window.LandingVariants = { A: LandingA, B: LandingB, C: LandingC };
