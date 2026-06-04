/* ===================== NexSpot · shared UI primitives ===================== */

function Toggle({ on, onChange }) {
  return <button className={"toggle" + (on ? " on" : "")} onClick={() => onChange(!on)} aria-pressed={on} />;
}

function Segmented({ value, onChange, options }) {
  // options: [{value,label,icon?}]
  return (
    <div className="seg">
      {options.map((o) => (
        <button key={o.value} className={value === o.value ? "on" : ""} onClick={() => onChange(o.value)}>
          {o.icon && <Icon name={o.icon} />}
          {o.label}
        </button>
      ))}
    </div>
  );
}

function Stepper({ value, onChange, min = 1, max = 16 }) {
  const clamp = (n) => Math.max(min, Math.min(max, n));
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 0, border: "1px solid var(--bd)", borderRadius: "var(--r-btn)", background: "var(--bg2)", overflow: "hidden", width: "fit-content" }}>
      <button className="icon-btn" style={{ borderRadius: 0, width: 32 }} onClick={() => onChange(clamp(value - 1))}><Icon name="minus" /></button>
      <span className="mono" style={{ minWidth: 30, textAlign: "center", fontSize: 12.5, fontWeight: 600 }}>{value}</span>
      <button className="icon-btn" style={{ borderRadius: 0, width: 32 }} onClick={() => onChange(clamp(value + 1))}><Icon name="plus" /></button>
    </div>
  );
}

function Field({ label, hint, children }) {
  return (
    <div className="field">
      {label && <label>{label}</label>}
      {children}
      {hint && <div className="hint">{hint}</div>}
    </div>
  );
}

function Row({ label, hint, children }) {
  // settings row: label left, control right
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 18, padding: "16px 0", borderBottom: "1px solid var(--bd)" }}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 13, fontWeight: 600 }}>{label}</div>
        {hint && <div style={{ fontSize: 11.5, color: "var(--mut)", marginTop: 3 }}>{hint}</div>}
      </div>
      <div style={{ flex: "none" }}>{children}</div>
    </div>
  );
}

function ModeBadge({ engine, mode }) {
  return (
    <span className="tagm" style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
      <span style={{ color: engine === "vello" ? "var(--accent)" : "var(--mut)", fontWeight: 600 }}>{ENGINES[engine]}</span>
      <span style={{ opacity: .4 }}>·</span>
      {MODES[mode].label}
    </span>
  );
}

/* ---- toast host (subscribes to a module-level emitter) ---- */
const toastBus = (() => {
  const subs = new Set();
  return { emit: (msg, icon) => subs.forEach((f) => f(msg, icon)), sub: (f) => (subs.add(f), () => subs.delete(f)) };
})();
const toast = (msg, icon = "check") => toastBus.emit(msg, icon);

function ToastHost() {
  const [items, setItems] = useState([]);
  useEffect(() =>
    toastBus.sub((msg, icon) => {
      const id = Date.now() + Math.random();
      setItems((x) => [...x, { id, msg, icon }]);
      setTimeout(() => setItems((x) => x.filter((i) => i.id !== id)), 2200);
    }), []);
  return (
    <div className="toast-wrap">
      {items.map((i) => (
        <div className="toast" key={i.id}><Icon name={i.icon} />{i.msg}</div>
      ))}
    </div>
  );
}

function EmptyState({ icon, title, body, action, onAction }) {
  return (
    <div className="fade" style={{ display: "grid", placeItems: "center", padding: "64px 24px", textAlign: "center" }}>
      <div style={{ maxWidth: 340 }}>
        <div style={{ width: 64, height: 64, borderRadius: 18, margin: "0 auto 18px", display: "grid", placeItems: "center", background: "var(--accent-soft)", color: "var(--accent)" }}>
          <Icon name={icon} size={28} />
        </div>
        <div style={{ fontSize: 17, fontWeight: 800, letterSpacing: "-.02em" }}>{title}</div>
        <div style={{ fontSize: 13, color: "var(--mut)", marginTop: 8, lineHeight: 1.6 }}>{body}</div>
        {action && <button className="btn btn-accent" style={{ marginTop: 20 }} onClick={onAction}><Icon name="plus" />{action}</button>}
      </div>
    </div>
  );
}

function Ph({ label, style, className = "" }) {
  return (
    <div className={"ph " + className} style={{ display: "grid", placeItems: "center", borderRadius: "var(--r-panel)", ...style }}>
      {label && <span className="ph-label">{label}</span>}
    </div>
  );
}

Object.assign(window, { Toggle, Segmented, Stepper, Field, Row, ModeBadge, ToastHost, toast, EmptyState, Ph });
