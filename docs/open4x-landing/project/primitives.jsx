// Wireframe primitives — sketchy boxes, scribble labels, etc.

const Box = ({ children, className = "", style, ...rest }) => (
  <div className={`sk-box ${className}`} style={style} {...rest}>{children}</div>
);

const Btn = ({ children, variant = "", size = "", ...rest }) => (
  <button className={`sk-btn ${variant} ${size}`} {...rest}>{children}</button>
);

const Tag = ({ children, variant = "" }) => (
  <span className={`sk-tag ${variant}`}>{children}</span>
);

const Scribble = ({ children, sm }) => (
  <span className={`scribble ${sm ? "scribble-sm" : ""}`}>{children}</span>
);

const Annotation = ({ children }) => (
  <span className="annotation">{children}</span>
);

// Stripe placeholder for images / map regions / hero art
const Stripe = ({ children, style, className = "" }) => (
  <div className={`sk-stripe ${className}`} style={style}>{children}</div>
);

// Simulated terminal cursor
const Caret = () => (
  <span style={{
    display: "inline-block",
    width: "0.55em",
    height: "1em",
    background: "currentColor",
    verticalAlign: "text-bottom",
    animation: "blink 1.1s steps(2) infinite",
    marginLeft: 2,
  }} />
);

// Tiny terminal-ish line
const CmdLine = ({ children }) => (
  <div style={{ fontSize: "var(--fs-sm)" }}>
    <span className="cmd-prefix"></span>{children}
  </div>
);

// Sketchy "map thumbnail" — pseudo-hex pattern with some land blobs
const MapThumb = ({ seed = 1, style }) => {
  const blobs = [];
  // deterministic-ish pseudo-random
  let s = seed * 9301 + 49297;
  const rnd = () => {
    s = (s * 9301 + 49297) % 233280;
    return s / 233280;
  };
  for (let i = 0; i < 6 + (seed % 4); i++) {
    blobs.push({
      x: rnd() * 90 + 5,
      y: rnd() * 80 + 10,
      r: rnd() * 12 + 6,
    });
  }
  return (
    <div className="map-thumb-svg" style={{
      width: "100%",
      height: "100%",
      border: "1.5px dashed var(--ink)",
      background: "var(--paper-2)",
      position: "relative",
      overflow: "hidden",
      ...style,
    }}>
      <svg viewBox="0 0 100 60" preserveAspectRatio="none" style={{ width: "100%", height: "100%" }}>
        {/* faint hex grid lines */}
        <defs>
          <pattern id={`hex-${seed}`} x="0" y="0" width="8" height="7" patternUnits="userSpaceOnUse">
            <path d="M0 3.5 L2 0 L6 0 L8 3.5 L6 7 L2 7 Z" fill="none" stroke="var(--dim-2)" strokeWidth="0.2" />
          </pattern>
        </defs>
        <rect width="100" height="60" fill={`url(#hex-${seed})`} />
        {blobs.map((b, i) => (
          <circle key={i} cx={b.x} cy={b.y} r={b.r}
                  fill="var(--paper)" stroke="var(--ink)" strokeWidth="0.4"
                  strokeDasharray="0.8 0.6" />
        ))}
      </svg>
    </div>
  );
};

// Slider primitive used in advanced params
const Slider = ({ value, min = 0, max = 100, onChange, format }) => (
  <>
    <input type="range" className="range" min={min} max={max} value={value}
           onChange={e => onChange?.(+e.target.value)} />
    <span className="value">{format ? format(value) : value}</span>
  </>
);

// Segmented control
const Segmented = ({ options, value, onChange }) => (
  <div className="seg">
    {options.map(o => {
      const v = typeof o === "string" ? o : o.value;
      const label = typeof o === "string" ? o : o.label;
      return (
        <button key={v} aria-pressed={value === v} onClick={() => onChange?.(v)}>{label}</button>
      );
    })}
  </div>
);

// Simple toggle
const Toggle = ({ on, onChange }) => (
  <button className={`toggle ${on ? "on" : ""}`} onClick={() => onChange?.(!on)}
          aria-pressed={on}></button>
);

Object.assign(window, {
  Box, Btn, Tag, Scribble, Annotation, Stripe, Caret, CmdLine,
  MapThumb, Slider, Segmented, Toggle,
});
