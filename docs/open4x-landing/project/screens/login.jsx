// Login — 3 variants

const LoginA = ({ onSubmit, onBack }) => {
  // Variant A: Stacked, all three methods equal weight
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
      <div className="row between" style={{ padding: "12px 20px", borderBottom: "1.5px dashed var(--ink)" }}>
        <Btn size="sm" variant="ghost" onClick={onBack}>← back</Btn>
        <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>open4x·vi / sign in</span>
      </div>
      <div className="login-card">
        <div className="h2" style={{ marginBottom: 4 }}>Sign in</div>
        <div className="muted" style={{ fontSize: "var(--fs-sm)", marginBottom: 20 }}>
          // any method below — all link to the same player ID
        </div>

        <Box className="fill" style={{ marginBottom: 12, padding: 14 }}>
          <div className="h3" style={{ marginBottom: 8 }}>EMAIL</div>
          <div className="field">
            <label>email</label>
            <input className="sk-input" placeholder="you@example.com" />
          </div>
          <Btn variant="primary" style={{ width: "100%", justifyContent: "center" }} onClick={onSubmit}>
            Send magic link →
          </Btn>
        </Box>

        <Box style={{ marginBottom: 12, padding: 14 }}>
          <div className="h3" style={{ marginBottom: 8 }}>OPENID</div>
          <div className="row" style={{ gap: 6, flexWrap: "wrap" }}>
            <Btn size="sm" onClick={onSubmit}>Google</Btn>
            <Btn size="sm" onClick={onSubmit}>GitHub</Btn>
            <Btn size="sm" onClick={onSubmit}>GitLab</Btn>
            <Btn size="sm" onClick={onSubmit}>Custom OIDC…</Btn>
          </div>
        </Box>

        <Box style={{ padding: 14 }}>
          <div className="h3" style={{ marginBottom: 8 }}>ATPROTO</div>
          <div className="field">
            <label>handle or DID</label>
            <input className="sk-input" placeholder="alice.bsky.social" />
          </div>
          <Btn style={{ width: "100%", justifyContent: "center" }} onClick={onSubmit}>
            Continue with atproto
          </Btn>
        </Box>

        <div className="muted" style={{ fontSize: "var(--fs-sm)", textAlign: "center", marginTop: 18 }}>
          new here? a player ID is created automatically.
        </div>
      </div>
    </div>
  );
};

const LoginB = ({ onSubmit, onBack }) => {
  // Variant B: Email primary, others as secondary "continue with…"
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
      <div className="row between" style={{ padding: "12px 20px", borderBottom: "1.5px dashed var(--ink)" }}>
        <Btn size="sm" variant="ghost" onClick={onBack}>← back</Btn>
        <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>open4x·vi / sign in</span>
      </div>
      <div className="login-card">
        <div className="h2" style={{ marginBottom: 4 }}>Sign in</div>
        <div className="muted" style={{ fontSize: "var(--fs-sm)", marginBottom: 22 }}>
          // we'll email you a one-time link — no password.
        </div>

        <div className="field">
          <label>email</label>
          <input className="sk-input" placeholder="you@example.com" />
        </div>
        <Btn variant="accent" style={{ width: "100%", justifyContent: "center", marginTop: 6 }} onClick={onSubmit}>
          Send magic link
        </Btn>

        <div className="row center-y" style={{ margin: "24px 0", gap: 10 }}>
          <hr className="sk-divider" style={{ flex: 1, margin: 0 }} />
          <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>or continue with</span>
          <hr className="sk-divider" style={{ flex: 1, margin: 0 }} />
        </div>

        <div className="col" style={{ gap: 8 }}>
          <Btn style={{ justifyContent: "flex-start", width: "100%" }} onClick={onSubmit}>
            <span style={{ width: 20, color: "var(--dim)" }}>○</span> Continue with Google
          </Btn>
          <Btn style={{ justifyContent: "flex-start", width: "100%" }} onClick={onSubmit}>
            <span style={{ width: 20, color: "var(--dim)" }}>○</span> Continue with GitHub
          </Btn>
          <Btn style={{ justifyContent: "flex-start", width: "100%" }} onClick={onSubmit}>
            <span style={{ width: 20, color: "var(--dim)" }}>@</span> Continue with atproto
          </Btn>
          <Btn variant="ghost" style={{ justifyContent: "flex-start", width: "100%" }} onClick={onSubmit}>
            <span style={{ width: 20, color: "var(--dim)" }}>+</span> Custom OIDC provider…
          </Btn>
        </div>

        <div className="muted" style={{ fontSize: "var(--fs-sm)", textAlign: "center", marginTop: 22 }}>
          all methods link to the same player ID.
        </div>
      </div>
    </div>
  );
};

const LoginC = ({ onSubmit, onBack }) => {
  // Variant C: Tabbed
  const [tab, setTab] = React.useState("email");
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
      <div className="row between" style={{ padding: "12px 20px", borderBottom: "1.5px dashed var(--ink)" }}>
        <Btn size="sm" variant="ghost" onClick={onBack}>← back</Btn>
        <span className="muted" style={{ fontSize: "var(--fs-sm)" }}>open4x·vi / sign in</span>
      </div>
      <div className="login-card">
        <div className="h2" style={{ marginBottom: 14 }}>Sign in</div>

        <div className="login-tabs">
          {["email", "openid", "atproto"].map(t => (
            <button key={t} aria-current={tab === t} onClick={() => setTab(t)}>{t}</button>
          ))}
        </div>

        {tab === "email" && (
          <>
            <div className="field">
              <label>email</label>
              <input className="sk-input" placeholder="you@example.com" />
            </div>
            <Btn variant="accent" style={{ width: "100%", justifyContent: "center" }} onClick={onSubmit}>
              Send magic link
            </Btn>
            <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 14 }}>
              we'll email you a one-time link valid for 15 min.
            </div>
          </>
        )}

        {tab === "openid" && (
          <>
            <div className="muted" style={{ fontSize: "var(--fs-sm)", marginBottom: 12 }}>
              choose a provider:
            </div>
            <div className="col" style={{ gap: 6 }}>
              <Btn onClick={onSubmit} style={{ justifyContent: "flex-start" }}>● Google</Btn>
              <Btn onClick={onSubmit} style={{ justifyContent: "flex-start" }}>● GitHub</Btn>
              <Btn onClick={onSubmit} style={{ justifyContent: "flex-start" }}>● GitLab</Btn>
              <Btn onClick={onSubmit} style={{ justifyContent: "flex-start" }}>● Microsoft</Btn>
            </div>
            <div className="field" style={{ marginTop: 16 }}>
              <label>or custom OIDC issuer URL</label>
              <input className="sk-input" placeholder="https://auth.example.com" />
            </div>
            <Btn style={{ width: "100%", justifyContent: "center" }} onClick={onSubmit}>Connect</Btn>
          </>
        )}

        {tab === "atproto" && (
          <>
            <div className="field">
              <label>handle or DID</label>
              <input className="sk-input" placeholder="alice.bsky.social" />
            </div>
            <div className="field">
              <label>PDS (optional)</label>
              <input className="sk-input" placeholder="https://bsky.social" />
            </div>
            <Btn variant="accent" style={{ width: "100%", justifyContent: "center" }} onClick={onSubmit}>
              Continue
            </Btn>
            <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 14 }}>
              opens an oauth flow with your PDS.
            </div>
          </>
        )}
      </div>
    </div>
  );
};

window.LoginVariants = { A: LoginA, B: LoginB, C: LoginC };
