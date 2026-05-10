// User profile / settings

const Profile = () => {
  return (
    <div style={{ flex: 1, overflow: "auto" }}>
      <div className="content-header">
        <div className="title">Profile &amp; settings</div>
        <span className="crumbs">player_id: 0xA9C3·7F12·EE04</span>
      </div>

      <div className="profile-grid">
        <Box>
          <div className="col center-x" style={{ alignItems: "center", gap: 10 }}>
            <div className="avatar">A</div>
            <Btn size="sm" variant="ghost">change avatar</Btn>
            <div className="muted" style={{ fontSize: "var(--fs-sm)", textAlign: "center" }}>
              upload, gravatar,<br/>or initials fallback
            </div>
          </div>
          <hr className="sk-divider" />
          <div className="h3" style={{ marginBottom: 8 }}>QUICK ACTIONS</div>
          <div className="col" style={{ gap: 6 }}>
            <Btn size="sm" variant="ghost" style={{ justifyContent: "flex-start" }}>copy player ID</Btn>
            <Btn size="sm" variant="ghost" style={{ justifyContent: "flex-start" }}>show QR for invite</Btn>
            <Btn size="sm" variant="ghost" style={{ justifyContent: "flex-start" }}>export save data</Btn>
            <Btn size="sm" variant="ghost" style={{ justifyContent: "flex-start", color: "var(--accent)" }}>sign out</Btn>
          </div>
        </Box>

        <div className="col">
          <Box>
            <div className="h3" style={{ marginBottom: 12 }}>PROFILE</div>
            <div className="field">
              <label>preferred name</label>
              <input className="sk-input" defaultValue="Alice" />
            </div>
            <div className="field">
              <label>pronouns (optional)</label>
              <input className="sk-input" defaultValue="she/her" />
            </div>
            <div className="field">
              <label>bio (shown on invite cards)</label>
              <textarea className="sk-input" rows={2} defaultValue="Plays slow. Reads everything." />
            </div>
            <div className="muted" style={{ fontSize: "var(--fs-sm)" }}>
              ⓘ no usernames — others find you by your linked IDs below.
            </div>
          </Box>

          <Box>
            <div className="row between center-y" style={{ marginBottom: 12 }}>
              <div className="h3">LINKED IDENTITIES</div>
              <Btn size="sm" variant="ghost">+ link another</Btn>
            </div>

            <div className="id-row primary">
              <div className="id-type">EMAIL <span style={{ color: "var(--accent)", fontWeight: 400 }}>· primary</span></div>
              <div className="id-val">alice@example.com</div>
              <Btn size="sm" variant="ghost">manage</Btn>
            </div>
            <div className="id-row">
              <div className="id-type">OPENID</div>
              <div className="id-val">google.com/110293·a73f</div>
              <Btn size="sm" variant="ghost">unlink</Btn>
            </div>
            <div className="id-row">
              <div className="id-type">OPENID</div>
              <div className="id-val">github.com/alice</div>
              <Btn size="sm" variant="ghost">unlink</Btn>
            </div>
            <div className="id-row">
              <div className="id-type">ATPROTO</div>
              <div className="id-val">did:plc:abcd1234efgh5678 · alice.bsky.social</div>
              <Btn size="sm" variant="ghost">unlink</Btn>
            </div>

            <div className="muted" style={{ fontSize: "var(--fs-sm)", marginTop: 10 }}>
              all four IDs map to the same player. friends can find you by any of them.
            </div>
          </Box>

          <Box>
            <div className="h3" style={{ marginBottom: 12 }}>PREFERENCES</div>
            <div className="param-row">
              <div className="label">interface density</div>
              <div className="control">
                <Segmented options={["compact", "comfortable", "spacious"]} value="comfortable" onChange={() => {}} />
              </div>
              <div className="value muted">→ tweaks panel</div>
            </div>
            <div className="param-row">
              <div className="label">color scheme</div>
              <div className="control">
                <Segmented options={["paper", "ink", "auto"]} value="paper" onChange={() => {}} />
              </div>
              <div className="value"></div>
            </div>
            <div className="param-row">
              <div className="label">keyboard nav</div>
              <div className="control"><Toggle on={true} /></div>
              <div className="value muted">vim bindings</div>
            </div>
            <div className="param-row">
              <div className="label">turn notifications</div>
              <div className="control"><Toggle on={true} /></div>
              <div className="value muted">email + push</div>
            </div>
            <div className="param-row">
              <div className="label">discoverable by ID</div>
              <div className="control"><Toggle on={true} /></div>
              <div className="value muted">others can invite</div>
            </div>
          </Box>
        </div>
      </div>
    </div>
  );
};

window.Profile = Profile;
