// Hi-fi Login A — stacked panels, equal weight: email / openid / atproto

const Login = ({ onBack }) => {
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
      <div className="row between center-y" style={{ padding: "10px 20px", borderBottom: "1px solid var(--hairline)" }}>
        <Btn variant="bare" size="sm" onClick={onBack}>← back</Btn>
        <span className="muted xsmall" style={{ letterSpacing: "0.08em" }}>OPEN4X·VI / SIGN IN</span>
      </div>

      <div style={{ width: 460, maxWidth: "100%", margin: "32px auto", padding: 4 }}>
        <h1 className="h1" style={{ marginBottom: 4 }}>Sign in</h1>
        <p className="muted small" style={{ marginBottom: 24 }}>
          Any method below — they all link to the same{" "}
          <Popup title="player ID" content={
            <PopupBody>
              <p>An opaque, unique identifier (e.g. <code>0xA9C3·7F12·EE04</code>) created the first time you sign in.</p>
              <p>All three login methods can be linked to the same player ID, so you can reach your games from any device.</p>
              <p className="muted xsmall">There are no usernames. Friends find you by your linked email, OpenID URL, or atproto handle.</p>
            </PopupBody>
          }>
            <span className="trigger">player ID</span>
          </Popup>.
        </p>

        {/* Email */}
        <div className="panel" style={{ marginBottom: 12 }}>
          <div className="row between center-y" style={{ marginBottom: 10 }}>
            <span className="h3">Email</span>
            <Popup title="How it works" content={
              <PopupBody>
                <p>1. Enter your email.</p>
                <p>2. Receive a one-time magic link (valid 15 min).</p>
                <p>3. Click it — you're signed in.</p>
              </PopupBody>
            }>
              <span className="trigger xsmall muted">how it works</span>
            </Popup>
          </div>
          <div className="field" style={{ marginBottom: 10 }}>
            <input className="input mono" placeholder="you@example.com" />
          </div>
          <Btn variant="primary" className="block">Send magic link →</Btn>
        </div>

        {/* OpenID */}
        <div className="panel" style={{ marginBottom: 12 }}>
          <div className="row between center-y" style={{ marginBottom: 10 }}>
            <span className="h3">OpenID</span>
            <Popup title="OIDC" content={
              <PopupBody>
                <p>Standard OpenID Connect flow. We support common providers and any custom issuer URL.</p>
                <p className="muted xsmall">Tokens are stored client-side; the server only sees signed claims.</p>
              </PopupBody>
            }>
              <span className="trigger xsmall muted">about OIDC</span>
            </Popup>
          </div>
          <div className="row wrap" style={{ gap: 6 }}>
            {["Google", "GitHub", "GitLab", "Microsoft"].map(p => (
              <Popup key={p} title={p} content={
                <>
                  <PopupBody>
                    <p>Continue with <strong>{p}</strong>. Opens a new tab for OAuth, returns when complete.</p>
                  </PopupBody>
                  <PopupActions right>
                    <Btn variant="accent" size="sm">Continue →</Btn>
                  </PopupActions>
                </>
              }>
                <Btn size="sm">{p}</Btn>
              </Popup>
            ))}
            <Popup title="Custom OIDC" content={
              <>
                <PopupBody>
                  <p>Paste any OIDC issuer URL — your own auth server, Keycloak, Auth0, Authentik, etc.</p>
                  <input className="input mono" placeholder="https://auth.example.com" style={{ marginTop: 6 }} />
                </PopupBody>
                <PopupActions right>
                  <Btn variant="primary" size="sm">Connect</Btn>
                </PopupActions>
              </>
            }>
              <Btn variant="ghost" size="sm" className="has-popup">Custom OIDC…</Btn>
            </Popup>
          </div>
        </div>

        {/* atproto */}
        <div className="panel" style={{ marginBottom: 16 }}>
          <div className="row between center-y" style={{ marginBottom: 10 }}>
            <span className="h3">atproto</span>
            <Popup title="atproto" content={
              <PopupBody>
                <p>Use your handle (e.g. <code>alice.bsky.social</code>) or DID. We resolve your PDS and start an OAuth flow.</p>
                <p className="muted xsmall">Works with self-hosted PDSes too.</p>
              </PopupBody>
            }>
              <span className="trigger xsmall muted">about atproto</span>
            </Popup>
          </div>
          <div className="field" style={{ marginBottom: 10 }}>
            <input className="input mono" placeholder="alice.bsky.social  or  did:plc:…" />
          </div>
          <Btn className="block">Continue with atproto →</Btn>
        </div>

        <p className="muted xsmall" style={{ textAlign: "center", marginTop: 14 }}>
          New here? A player ID is created automatically on first sign-in.
        </p>
      </div>
    </div>
  );
};

window.Login = Login;
