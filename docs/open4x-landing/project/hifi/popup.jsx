// Gwern-style popup — hover preview, pin on click, smart positioning

const PopupContext = React.createContext(null);

function PopupProvider({ children }) {
  const [popup, setPopup] = React.useState(null);
  // popup: { content, anchor: DOMRect, pinned, key }
  const hideTimer = React.useRef(null);
  const showTimer = React.useRef(null);

  const show = (content, anchor, opts = {}) => {
    clearTimeout(hideTimer.current);
    if (popup && popup.pinned) return;
    showTimer.current = setTimeout(() => {
      setPopup({ content, anchor, pinned: false, ...opts });
    }, opts.delay ?? 180);
  };
  const cancelShow = () => clearTimeout(showTimer.current);
  const scheduleHide = () => {
    clearTimeout(showTimer.current);
    if (popup && popup.pinned) return;
    hideTimer.current = setTimeout(() => setPopup(null), 140);
  };
  const cancelHide = () => clearTimeout(hideTimer.current);
  const pin = () => setPopup(p => p ? { ...p, pinned: true } : null);
  const close = () => { clearTimeout(showTimer.current); clearTimeout(hideTimer.current); setPopup(null); };

  // Esc to close pinned popups
  React.useEffect(() => {
    const onKey = e => { if (e.key === "Escape") close(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Click outside to dismiss pinned
  React.useEffect(() => {
    if (!popup?.pinned) return;
    const onClick = e => {
      if (e.target.closest(".popup")) return;
      if (e.target.closest("[data-popup-trigger]")) return;
      close();
    };
    setTimeout(() => window.addEventListener("click", onClick), 0);
    return () => window.removeEventListener("click", onClick);
  }, [popup?.pinned]);

  return (
    <PopupContext.Provider value={{ show, cancelShow, scheduleHide, cancelHide, pin, close, popup }}>
      {children}
      {popup && <PopupRender popup={popup} />}
    </PopupContext.Provider>
  );
}

function PopupRender({ popup }) {
  const { cancelHide, scheduleHide, pin, close } = React.useContext(PopupContext);
  const ref = React.useRef(null);
  const [pos, setPos] = React.useState({ top: -9999, left: -9999, placement: "bottom" });

  React.useLayoutEffect(() => {
    if (!ref.current) return;
    const a = popup.anchor;
    const pop = ref.current.getBoundingClientRect();
    const vw = window.innerWidth, vh = window.innerHeight;
    const margin = 8;
    // Default: below + aligned left
    let top = a.bottom + 6;
    let left = a.left;
    let placement = "bottom";
    if (top + pop.height + margin > vh && a.top - pop.height - 6 > margin) {
      top = a.top - pop.height - 6;
      placement = "top";
    }
    if (left + pop.width + margin > vw) left = vw - pop.width - margin;
    if (left < margin) left = margin;
    setPos({ top, left, placement });
  }, [popup]);

  return (
    <div ref={ref}
         className={`popup ${popup.size || ""}`}
         style={{ top: pos.top, left: pos.left }}
         onMouseEnter={cancelHide}
         onMouseLeave={scheduleHide}>
      {popup.title && (
        <div className="popup-head">
          <span className="title">{popup.title}</span>
          <button className="pin" onClick={popup.pinned ? close : pin}
                  title={popup.pinned ? "close (esc)" : "pin"}>
            {popup.pinned ? "×" : "⌶"}
          </button>
        </div>
      )}
      {popup.content}
    </div>
  );
}

// <Popup>: wraps a trigger element. On hover/click, opens with `content`.
function Popup({ children, content, title, size, asChild, trigger = "hover" }) {
  const ctx = React.useContext(PopupContext);
  const ref = React.useRef(null);

  const open = (e) => {
    if (!ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    ctx.show(content, rect, { title, size, delay: trigger === "click" ? 0 : 180 });
  };
  const onEnter = (e) => { if (trigger !== "click") open(e); };
  const onLeave = () => { if (trigger !== "click") ctx.scheduleHide(); };
  const onClick = (e) => {
    e.stopPropagation();
    open(e);
    setTimeout(() => ctx.pin(), 10);
  };

  // Wrap or pass-through. Simpler: render a <span> wrapper that we attach refs to.
  return (
    <span ref={ref}
          data-popup-trigger=""
          onMouseEnter={onEnter}
          onMouseLeave={onLeave}
          onClick={onClick}
          style={{ display: "inline-flex" }}>
      {children}
    </span>
  );
}

// Convenience renderers for common popup shapes
const PopupBody = ({ children }) => <div className="popup-body">{children}</div>;
const PopupActions = ({ children, right }) => <div className={`popup-actions ${right ? "right" : ""}`}>{children}</div>;
const PopupList = ({ items, onPick }) => (
  <div className="popup-list">
    {items.map((it, i) => it === "sep" ? <div key={i} className="sep" /> : (
      <button key={i} className="item" onClick={() => onPick?.(it)}>
        {it.icon && <span className="icon">{it.icon}</span>}
        <span>{it.label}</span>
        {it.desc && <span className="desc">{it.desc}</span>}
      </button>
    ))}
  </div>
);
const PopupKV = ({ rows }) => (
  <div className="popup-body">
    <div className="kv">
      {rows.map(([k, v], i) => (<React.Fragment key={i}><span className="k">{k}</span><span>{v}</span></React.Fragment>))}
    </div>
  </div>
);

Object.assign(window, { PopupProvider, Popup, PopupBody, PopupActions, PopupList, PopupKV });
