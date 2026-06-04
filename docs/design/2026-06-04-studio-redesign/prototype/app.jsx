/* ===================== NexSpot · presentation shell + mount ===================== */

function hexLum(hex) {
  const m = hex.replace("#", "");
  const r = parseInt(m.slice(0, 2), 16) / 255, g = parseInt(m.slice(2, 4), 16) / 255, b = parseInt(m.slice(4, 6), 16) / 255;
  const f = (c) => (c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

function App() {
  const s = useStore(appStore);
  const [view, setView] = useState("main");

  // apply theme
  useEffect(() => {
    const apply = () => {
      let t = s.theme;
      if (t === "system") t = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
      document.documentElement.setAttribute("data-theme", t);
    };
    apply();
    if (s.theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", apply);
      return () => mq.removeEventListener("change", apply);
    }
  }, [s.theme]);

  // apply accent
  useEffect(() => {
    const root = document.documentElement.style;
    root.setProperty("--accent", s.accent);
    root.setProperty("--accent-press", `color-mix(in srgb, ${s.accent} 82%, #000)`);
    root.setProperty("--on-accent", hexLum(s.accent) > 0.55 ? "#1b1c1f" : "#ffffff");
  }, [s.accent]);

  const VIEWS = [
    { id: "main", icon: "app-window", label: "主窗口" },
    { id: "pin", icon: "pin", label: "PIN 合集" },
    { id: "scroll", icon: "gallery-vertical-end", label: "滚动预览" },
    { id: "ocr", icon: "scan-text", label: "OCR" },
    { id: "toolbar", icon: "wrench", label: "工具栏规格" },
  ];

  return (
    <div className="stage-root">
      <div className="pchrome">
        <div className="pbrand">
          <span className="logo"><Icon name="crop" size={14} /></span>
          NexSpot <span className="tag">Studio · 现代工作室</span>
        </div>
        <div className="pseg" style={{ marginLeft: 6 }}>
          {VIEWS.map((v) => (
            <button key={v.id} className={view === v.id ? "on" : ""} onClick={() => setView(v.id)}>
              <Icon name={v.icon} />{v.label}
            </button>
          ))}
        </div>
        <div className="pctl">
          <div className="theme-seg">
            {[["light", "sun"], ["dark", "moon"], ["system", "monitor"]].map(([t, ic]) => (
              <button key={t} className={s.theme === t ? "on" : ""} title={t} onClick={() => actions.set({ theme: t })}><Icon name={ic} /></button>
            ))}
          </div>
          <div className="accent-pick">
            {ACCENTS.map(([c]) => (
              <span key={c} className={"sw" + (s.accent === c ? " on" : "")} style={{ background: c }} onClick={() => actions.set({ accent: c })} title={c} />
            ))}
          </div>
        </div>
      </div>

      <div className="desk">
        <div className="deskpad">
          {view === "main" && <MainWindow />}
          {view === "pin" && <PinWindow />}
          {view === "scroll" && <ScrollWindow />}
          {view === "ocr" && <OcrWindow />}
          {view === "toolbar" && <ToolbarSpec />}
          {(view === "pin" || view === "scroll" || view === "ocr") && (
            <div style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--mut)", display: "flex", alignItems: "center", gap: 8, opacity: .8 }}>
              <Icon name="layers" size={13} />无边框 · 半透明玻璃 · 悬浮于桌面之上
            </div>
          )}
        </div>
      </div>

      <ToastHost />
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")).render(<App />);
