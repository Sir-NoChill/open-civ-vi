// Hi-fi components shared across screens

const Btn = ({ children, variant = "", size = "", className = "", ...rest }) => (
  <button className={`btn ${variant} ${size} ${className}`} {...rest}>{children}</button>
);

const Tag = ({ children, variant = "" }) => <span className={`tag ${variant}`}>{children}</span>;

// Mini SVG map — deterministic blobby continents
const MiniMap = ({ seed = 1, className = "", style }) => {
  let s = seed * 9301 + 49297;
  const rnd = () => { s = (s * 9301 + 49297) % 233280; return s / 233280; };
  const blobs = [];
  for (let i = 0; i < 5 + (seed % 4); i++) {
    blobs.push({ x: rnd() * 90 + 5, y: rnd() * 60 + 8, r: rnd() * 10 + 5, self: i === 0 });
  }
  return (
    <svg viewBox="0 0 100 64" preserveAspectRatio="none" className={`svg-map ${className}`} style={style}>
      <rect className="water" width="100" height="64" />
      {/* faint grid */}
      <g className="grid">
        {Array.from({length: 8}, (_, i) => <line key={"v"+i} x1={i*12.5} y1="0" x2={i*12.5} y2="64" />)}
        {Array.from({length: 6}, (_, i) => <line key={"h"+i} x1="0" y1={i*10.6} x2="100" y2={i*10.6} />)}
      </g>
      {blobs.map((b, i) => (
        <ellipse key={i} cx={b.x} cy={b.y} rx={b.r} ry={b.r * 0.7}
                 className={b.self ? "land-self" : "land"} />
      ))}
    </svg>
  );
};

// Slider primitive
const Slider = ({ value, min = 0, max = 100, format }) => (
  <>
    <input type="range" className="range" min={min} max={max} defaultValue={value} />
    <span className="value">{format ? format(value) : value}</span>
  </>
);

const Segmented = ({ options, value }) => (
  <div className="seg">
    {options.map(o => {
      const v = typeof o === "string" ? o : o.value;
      const label = typeof o === "string" ? o : o.label;
      return <button key={v} aria-pressed={value === v}>{label}</button>;
    })}
  </div>
);

const Toggle = ({ on }) => (
  <button className={`toggle ${on ? "on" : ""}`} aria-pressed={on}></button>
);

Object.assign(window, { Btn, Tag, MiniMap, Slider, Segmented, Toggle });
