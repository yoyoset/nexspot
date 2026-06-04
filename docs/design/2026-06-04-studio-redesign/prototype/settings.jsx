/* ===================== NexSpot · Settings (sub-tabs) ===================== */

const SUBTABS = [
  { id: "general", icon: "sliders-horizontal", label: "通用" },
  { id: "workflows", icon: "workflow", label: "工作流" },
  { id: "advanced", icon: "cpu", label: "高级" },
  { id: "appearance", icon: "palette", label: "外观" },
  { id: "donate", icon: "heart", label: "捐赠" },
];

function SettingsPage({ tab, setTab, editTarget, setEditTarget }) {
  return (
    <div className="page">
      <div className="win-body" style={{ flex: 1, minHeight: 0 }}>
        {/* sub-tab nav */}
        <nav style={{ width: 158, flex: "none", borderRight: "1px solid var(--bd)", padding: "16px 10px", background: "var(--bg1)" }}>
          <div className="sect-title" style={{ padding: "0 8px 12px" }}>设置</div>
          {SUBTABS.map((t) => (
            <button key={t.id} onClick={() => { setTab(t.id); setEditTarget(null); }}
              style={{ width: "100%", display: "flex", alignItems: "center", gap: 10, padding: "9px 10px", marginBottom: 2, borderRadius: "var(--r-btn)", border: 0, fontSize: 12.5, fontWeight: 600, background: tab === t.id ? "var(--accent-soft)" : "transparent", color: tab === t.id ? "var(--accent)" : "var(--mut)" }}>
              <Icon name={t.icon} size={15} />{t.label}
            </button>
          ))}
        </nav>

        <div className="page-scroll" style={{ flex: 1, padding: "22px 26px" }}>
          {tab === "general" && <GeneralTab />}
          {tab === "workflows" && <WorkflowsTab editTarget={editTarget} setEditTarget={setEditTarget} />}
          {tab === "advanced" && <AdvancedTab />}
          {tab === "appearance" && <AppearanceTab />}
          {tab === "donate" && <DonateTab />}
        </div>
      </div>
    </div>
  );
}

function TabHead({ title, desc }) {
  return (
    <div style={{ marginBottom: 8 }}>
      <div style={{ fontSize: 18, fontWeight: 800, letterSpacing: "-.02em" }}>{title}</div>
      {desc && <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 4 }}>{desc}</div>}
    </div>
  );
}

/* -------- General -------- */
function GeneralTab() {
  const s = useStore(appStore);
  return (
    <div className="fade" style={{ maxWidth: 560 }}>
      <TabHead title="通用" desc="基础偏好设置" />
      <Row label="默认保存路径" hint="未单独配置的工作流将落到这里">
        <div style={{ display: "flex", gap: 8, width: 280 }}>
          <input className="input mono" value={s.savePath} onChange={(e) => actions.set({ savePath: e.target.value })} />
          <button className="btn" onClick={() => toast("选择文件夹…", "folder-open")}><Icon name="folder-open" /></button>
        </div>
      </Row>
      <Row label="标注字体" hint="原生标注工具栏文字工具使用">
        <select className="select" style={{ width: 180 }} value={s.annFont} onChange={(e) => actions.set({ annFont: e.target.value })}>
          {["Manrope", "JetBrains Mono", "Microsoft YaHei", "Source Han Sans", "Segoe UI"].map((f) => <option key={f}>{f}</option>)}
        </select>
      </Row>
      <Row label="界面语言" hint="Interface language">
        <Segmented value={s.lang} onChange={(v) => actions.set({ lang: v })} options={[{ value: "zh", label: "中文" }, { value: "en", label: "English" }]} />
      </Row>
    </div>
  );
}

/* -------- Workflows (list + form) -------- */
function HotkeyRecorder({ value, onChange }) {
  const [rec, setRec] = useState(false);
  useEffect(() => {
    if (!rec) return;
    const h = (e) => {
      e.preventDefault();
      const parts = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");
      const k = e.key.length === 1 ? e.key.toUpperCase() : e.key;
      if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) { parts.push(k); onChange(parts.join("+")); setRec(false); }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [rec]);
  return (
    <button className="btn" style={{ minWidth: 150, justifyContent: "center", borderStyle: rec ? "dashed" : "solid", borderColor: rec ? "var(--accent)" : "var(--bd)", color: rec ? "var(--accent)" : "var(--tx)" }} onClick={() => setRec((v) => !v)}>
      <Icon name={rec ? "circle-dot" : "keyboard"} />
      <span className="mono" style={{ fontSize: 11.5 }}>{rec ? "按下组合键…" : value}</span>
    </button>
  );
}

function WorkflowForm({ initial, onSave, onCancel }) {
  const [f, setF] = useState(initial);
  const up = (patch) => setF((x) => ({ ...x, ...patch }));
  return (
    <div className="fade card" style={{ padding: 22, maxWidth: 620 }}>
      <div style={{ fontSize: 15, fontWeight: 800, marginBottom: 18, display: "flex", alignItems: "center", gap: 9 }}>
        <Icon name={initial.id ? "pencil" : "plus"} size={17} />{initial.id ? "编辑工作流" : "新建工作流"}
      </div>
      <div style={{ display: "grid", gap: 18 }}>
        <Field label="名称">
          <input className="input" value={f.name} placeholder="例如：区域截图 · 快速" onChange={(e) => up({ name: e.target.value })} />
        </Field>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 18 }}>
          <Field label="全局热键">
            <HotkeyRecorder value={f.hotkey} onChange={(v) => up({ hotkey: v })} />
          </Field>
          <Field label="输出格式">
            <Segmented value={f.fmt} onChange={(v) => up({ fmt: v })} options={[{ value: "PNG", label: "PNG" }, { value: "JPG", label: "JPG" }]} />
          </Field>
        </div>

        <Field label="采集模式">
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4,1fr)", gap: 8 }}>
            {Object.entries(MODES).map(([k, m]) => (
              <button key={k} onClick={() => up({ mode: k })} className="card" style={{ padding: "12px 8px", display: "flex", flexDirection: "column", alignItems: "center", gap: 7, borderColor: f.mode === k ? "var(--accent)" : "var(--bd)", background: f.mode === k ? "var(--accent-soft)" : "var(--bg2)", color: f.mode === k ? "var(--accent)" : "var(--mut)" }}>
                <Icon name={m.icon} size={18} /><span style={{ fontSize: 11, fontWeight: 600 }}>{m.label}</span>
              </button>
            ))}
          </div>
        </Field>

        <div style={{ display: "grid", gridTemplateColumns: f.engine === "vello" ? "1fr 1fr" : "1fr", gap: 18 }}>
          <Field label="渲染引擎" hint="GDI 极速纯色 · Vello GPU 加速风格化">
            <Segmented value={f.engine} onChange={(v) => up({ engine: v })} options={[{ value: "gdi", label: "GDI" }, { value: "vello", label: "Vello" }]} />
          </Field>
          {f.engine === "vello" && (
            <Field label="风格">
              <select className="select" value={f.style || "Default"} onChange={(e) => up({ style: e.target.value })}>
                {VELLO_STYLES.map((v) => <option key={v}>{v}</option>)}
              </select>
            </Field>
          )}
        </div>

        <Field label="保存位置">
          <div style={{ display: "flex", gap: 8 }}>
            <input className="input mono" value={f.folder} onChange={(e) => up({ folder: e.target.value })} />
            <button className="btn" onClick={() => toast("选择文件夹…", "folder-open")}><Icon name="folder-open" /></button>
          </div>
        </Field>

        <div style={{ display: "flex", gap: 10 }}>
          <button className="card" onClick={() => up({ toFile: !f.toFile })} style={{ flex: 1, display: "flex", alignItems: "center", gap: 11, padding: "13px 14px", cursor: "pointer", borderColor: f.toFile ? "var(--accent-line)" : "var(--bd)" }}>
            <Icon name="hard-drive" size={17} style={{ color: f.toFile ? "var(--accent)" : "var(--mut)" }} />
            <div style={{ flex: 1, textAlign: "left" }}><div style={{ fontSize: 12.5, fontWeight: 600 }}>保存到文件</div></div>
            <Toggle on={f.toFile} onChange={(v) => up({ toFile: v })} />
          </button>
          <button className="card" onClick={() => up({ toClip: !f.toClip })} style={{ flex: 1, display: "flex", alignItems: "center", gap: 11, padding: "13px 14px", cursor: "pointer", borderColor: f.toClip ? "var(--accent-line)" : "var(--bd)" }}>
            <Icon name="clipboard" size={17} style={{ color: f.toClip ? "var(--accent)" : "var(--mut)" }} />
            <div style={{ flex: 1, textAlign: "left" }}><div style={{ fontSize: 12.5, fontWeight: 600 }}>复制到剪贴板</div></div>
            <Toggle on={f.toClip} onChange={(v) => up({ toClip: v })} />
          </button>
        </div>
      </div>

      <div style={{ display: "flex", gap: 9, marginTop: 22, justifyContent: "flex-end" }}>
        <button className="btn" onClick={onCancel}>取消</button>
        <button className="btn btn-accent" onClick={() => onSave(f)}><Icon name="check" />保存工作流</button>
      </div>
    </div>
  );
}

function WorkflowsTab({ editTarget, setEditTarget }) {
  const workflows = useStore(appStore, (s) => s.workflows);
  const blank = { name: "", engine: "gdi", mode: "region", fmt: "PNG", folder: "~/Pictures/NexSpot", hotkey: "Ctrl+Shift+N", toFile: true, toClip: false, preset: false, engineReady: true, conflict: false };

  if (editTarget) {
    const initial = editTarget === "new" ? blank : workflows.find((w) => w.id === editTarget) || blank;
    const save = (f) => {
      if (f.id) actions.set((s) => ({ workflows: s.workflows.map((w) => (w.id === f.id ? f : w)) }));
      else actions.set((s) => ({ workflows: [...s.workflows, { ...f, id: "wf-" + Date.now() }] }));
      toast("工作流已保存", "check"); setEditTarget(null);
    };
    return (
      <div>
        <button className="btn btn-ghost btn-sm" style={{ marginBottom: 14 }} onClick={() => setEditTarget(null)}><Icon name="arrow-left" />返回列表</button>
        <WorkflowForm initial={initial} onSave={save} onCancel={() => setEditTarget(null)} />
      </div>
    );
  }

  return (
    <div className="fade" style={{ maxWidth: 640 }}>
      <div style={{ display: "flex", alignItems: "center", marginBottom: 16 }}>
        <TabHead title="工作流" desc="增 / 删 / 改你的截图流水线" />
        <button className="btn btn-accent" style={{ marginLeft: "auto" }} onClick={() => setEditTarget("new")}><Icon name="plus" />新建</button>
      </div>
      {workflows.map((w) => (
        <div key={w.id} className="card" style={{ display: "flex", alignItems: "center", gap: 12, padding: "11px 13px", marginBottom: 8 }}>
          <div style={{ width: 32, height: 32, borderRadius: "var(--r-btn)", display: "grid", placeItems: "center", background: "var(--accent-soft)", color: "var(--accent)", flex: "none" }}><Icon name={MODES[w.mode].icon} size={16} /></div>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontSize: 12.5, fontWeight: 600 }}>{w.name}</div>
            <div style={{ display: "flex", gap: 7, marginTop: 4 }}><ModeBadge engine={w.engine} mode={w.mode} /><span className="kbd" style={{ fontSize: 10 }}>{w.hotkey}</span></div>
          </div>
          <button className="icon-btn" title="编辑" onClick={() => setEditTarget(w.id)}><Icon name="pencil" /></button>
          {w.preset ? <button className="icon-btn" disabled title="系统预设 · 不可删除"><Icon name="lock" /></button>
            : <button className="icon-btn danger" title="删除" onClick={() => { actions.deleteWorkflow(w.id); toast("已删除", "trash-2"); }}><Icon name="trash-2" /></button>}
        </div>
      ))}
    </div>
  );
}

/* -------- Advanced -------- */
function AdvancedTab() {
  const s = useStore(appStore);
  return (
    <div className="fade" style={{ maxWidth: 580 }}>
      <TabHead title="高级" desc="导出、引擎与日志" />

      <div className="sect-title" style={{ margin: "22px 0 4px" }}>导出</div>
      <Row label="JPG 质量" hint={"有损压缩等级 · 当前 " + s.jpgQuality}>
        <div style={{ display: "flex", alignItems: "center", gap: 12, width: 220 }}>
          <input className="slider" type="range" min="40" max="100" value={s.jpgQuality} onChange={(e) => actions.set({ jpgQuality: +e.target.value })} />
          <span className="mono" style={{ fontSize: 12, fontWeight: 600, width: 30 }}>{s.jpgQuality}</span>
        </div>
      </Row>
      <Row label="默认导出格式">
        <Segmented value={s.defaultFmt} onChange={(v) => actions.set({ defaultFmt: v })} options={[{ value: "PNG", label: "PNG" }, { value: "JPG", label: "JPG" }]} />
      </Row>
      <Row label="并发度" hint="同时处理的截图任务数">
        <Stepper value={s.concurrency} onChange={(v) => actions.set({ concurrency: v })} min={1} max={8} />
      </Row>
      <Row label="默认快照尺寸" hint="固定尺寸采集模式的默认值">
        <input className="input mono" style={{ width: 130, textAlign: "center" }} value={s.defaultSize} onChange={(e) => actions.set({ defaultSize: e.target.value })} />
      </Row>

      <div className="sect-title" style={{ margin: "26px 0 4px" }}>Vello 渲染引擎</div>
      <Row label="启用 Vello 引擎" hint="GPU 加速 · 支持风格化效果（与 GDI 互相独立）">
        <Toggle on={s.velloOn} onChange={(v) => actions.set({ velloOn: v })} />
      </Row>
      {s.velloOn && (
        <div style={{ padding: "16px 0", borderBottom: "1px solid var(--bd)" }}>
          <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 10 }}>风格</div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            {VELLO_STYLES.map((v) => (
              <button key={v} className={"chip" + (s.velloStyle === v ? " on" : "")} onClick={() => actions.set({ velloStyle: v })}>{v}</button>
            ))}
          </div>
        </div>
      )}
      {s.velloOn && (
        <Row label="高级效果" hint="阴影 / 发光 / 玻璃材质等增强效果">
          <Toggle on={s.advEffects} onChange={(v) => actions.set({ advEffects: v })} />
        </Row>
      )}

      <div className="sect-title" style={{ margin: "26px 0 4px" }}>日志维护</div>
      <Row label="运行日志" hint="排查问题时使用">
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn" onClick={() => toast("打开日志文件", "scroll-text")}><Icon name="file-text" />查看</button>
          <button className="btn" onClick={() => toast("日志已清空", "trash-2")}><Icon name="trash-2" />清空</button>
        </div>
      </Row>
    </div>
  );
}

/* -------- Appearance -------- */
function AppearanceTab() {
  const s = useStore(appStore);
  return (
    <div className="fade" style={{ maxWidth: 560 }}>
      <TabHead title="外观" desc="主题与强调色 · 实时生效" />
      <Row label="主题" hint="跟随系统会随 OS 切换亮 / 暗">
        <Segmented value={s.theme} onChange={(v) => actions.set({ theme: v })} options={[{ value: "light", label: "亮色", icon: "sun" }, { value: "dark", label: "暗色", icon: "moon" }, { value: "system", label: "跟随系统", icon: "monitor" }]} />
      </Row>
      <div style={{ padding: "18px 0" }}>
        <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>强调色</div>
        <div style={{ fontSize: 11.5, color: "var(--mut)", marginBottom: 14 }}>由单一 <span className="mono">--accent</span> 变量驱动全局</div>
        <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
          {ACCENTS.map(([c, name]) => (
            <button key={c} onClick={() => actions.set({ accent: c })} style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 7, background: "none", border: 0 }}>
              <span style={{ width: 40, height: 40, borderRadius: 12, background: c, boxShadow: s.accent === c ? "0 0 0 2px var(--bg1),0 0 0 4px " + c : "var(--shadow-sm)", transition: ".14s" }} />
              <span style={{ fontSize: 10.5, fontWeight: 600, color: s.accent === c ? "var(--tx)" : "var(--mut)" }}>{name}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

/* -------- Donate -------- */
function DonateTab() {
  return (
    <div className="fade" style={{ maxWidth: 520 }}>
      <TabHead title="支持 NexSpot" desc="独立开发 · 你的支持让它走得更远" />
      <div className="card" style={{ padding: 28, marginTop: 16, textAlign: "center", background: "linear-gradient(160deg,var(--accent-soft),transparent 70%)" }}>
        <div style={{ width: 60, height: 60, borderRadius: 18, margin: "0 auto 16px", display: "grid", placeItems: "center", background: "var(--accent)", color: "var(--on-accent)", boxShadow: "0 10px 28px -10px var(--accent)" }}><Icon name="heart" size={28} /></div>
        <div style={{ fontSize: 16, fontWeight: 800 }}>请我喝杯咖啡</div>
        <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 8, lineHeight: 1.6 }}>NexSpot 永久免费、无广告、无追踪。<br />如果它帮你省了时间，欢迎小额赞助。</div>
        <div style={{ display: "flex", gap: 10, justifyContent: "center", marginTop: 22 }}>
          {["¥6", "¥18", "¥66"].map((v, i) => (
            <button key={v} className={i === 1 ? "btn btn-accent" : "btn"} style={{ minWidth: 64, justifyContent: "center", fontSize: 14, fontWeight: 700 }} onClick={() => toast("感谢支持 " + v + " ❤", "heart")}>{v}</button>
          ))}
        </div>
        <div style={{ display: "flex", gap: 8, justifyContent: "center", marginTop: 18 }}>
          <button className="btn btn-sm" onClick={() => toast("已复制 GitHub 链接", "github")}><Icon name="github" />Star on GitHub</button>
          <button className="btn btn-sm" onClick={() => toast("已复制收款码", "qr-code")}><Icon name="qr-code" />微信 / 支付宝</button>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { SettingsPage });
