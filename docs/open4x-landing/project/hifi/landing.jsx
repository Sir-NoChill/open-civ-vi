// Hi-fi Landing A — centered ASCII banner + login CTA

const Landing = () => {
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", padding: "40px 32px", position: "relative" }}>
      <div style={{ maxWidth: 680, margin: "auto", textAlign: "center", paddingBottom: 60 }}>

        <div style={{ fontSize: "var(--fs-xs)", color: "var(--dim)", letterSpacing: "0.3em", marginBottom: 32 }}>
          V0.1.0 · PRE-ALPHA · OPEN SOURCE
        </div>

        <pre style={{
          fontFamily: "var(--font-mono)", fontWeight: 500,
          fontSize: 13, lineHeight: 1.15,
          color: "var(--ink)", margin: "0 0 28px",
          letterSpacing: 0,
        }}>{`┌─┐┌─┐┌─┐┌┐┌    ╦ ╦ ╦
│ │├─┘├┤ │││    ║ ║ ║
└─┘┴  └─┘┘└┘    ╩ ╩ ╩
       open  4x  vi`}</pre>

        <h1 className="h-display" style={{ marginBottom: 18 }}>
          A 4X game without<br/>the graphics tax<span className="caret"></span>
        </h1>

        <p className="muted" style={{ maxWidth: 480, margin: "0 auto 28px", fontSize: "var(--fs-md)", lineHeight: 1.65 }}>
          [ blurb placeholder — civ-vi inspired, deeply moddable, runs on a potato.
          author will write this. keep it short. keep it sharp. ]
        </p>

        <div className="row center-x" style={{ gap: 10, marginBottom: 36 }}>
          <Popup title="Sign in" size="narrow" content={
            <>
              <PopupBody>
                <p>Three auth methods, all link to the same player ID.</p>
                <p className="muted xsmall">No password — magic links for email; OAuth for the rest.</p>
              </PopupBody>
              <PopupActions right>
                <Btn variant="accent" size="sm">Continue →</Btn>
              </PopupActions>
            </>
          }>
            <Btn variant="accent" size="lg">Sign in &amp; play →</Btn>
          </Popup>

          <Popup title="github.com/open4x/vi" content={
            <>
              <PopupBody>
                <div className="kv" style={{ fontSize: "var(--fs-xs)" }}>
                  <span className="k">stars</span><span>1.2k</span>
                  <span className="k">license</span><span>AGPL-3.0</span>
                  <span className="k">stack</span><span>TypeScript · Rust core</span>
                  <span className="k">last commit</span><span>3h ago</span>
                </div>
              </PopupBody>
              <PopupActions right>
                <Btn variant="ghost" size="sm">↗ open repo</Btn>
              </PopupActions>
            </>
          }>
            <Btn variant="ghost" size="lg" className="has-popup">View source</Btn>
          </Popup>
        </div>

        <div className="muted" style={{ fontSize: "var(--fs-xs)", letterSpacing: "0.16em" }}>
          <Popup title="Email" content={<PopupBody><p>Magic-link login. We mail a one-time URL valid for 15 minutes.</p></PopupBody>}>
            <span className="trigger">EMAIL</span>
          </Popup>
          {" · "}
          <Popup title="OpenID Connect" content={<PopupBody><p>Sign in with Google, GitHub, GitLab, Microsoft, or any custom OIDC issuer URL.</p></PopupBody>}>
            <span className="trigger">OPENID</span>
          </Popup>
          {" · "}
          <Popup title="atproto" content={<PopupBody><p>Use your atproto handle (e.g. <code>alice.bsky.social</code>) or a DID. OAuth flow with your PDS.</p></PopupBody>}>
            <span className="trigger">ATPROTO</span>
          </Popup>
        </div>
      </div>

      <div style={{
        position: "absolute", bottom: 18, left: 0, right: 0,
        display: "flex", justifyContent: "space-between",
        padding: "0 32px",
        fontSize: "var(--fs-xs)", color: "var(--dim)",
      }}>
        <span>open4x.org</span>
        <span>// hover any underlined word</span>
        <span>self-hostable · API-driven</span>
      </div>
    </div>
  );
};

window.Landing = Landing;
