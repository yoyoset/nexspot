/* ===================== NexSpot · main window (rail + Dashboard + Activity) ===================== */

function Rail({ page, setPage }) {
  const top = useStore(appStore, (s) => s.alwaysOnTop);
  const items = [
    { id: "dashboard", icon: "layout-dashboard", tip: "工作流" },
    { id: "activity", icon: "activity", tip: "活动中心" },
    { id: "settings", icon: "settings", tip: "设置" },
  ];
  return (
    <div className="rail">
      {items.map((it) => (
        <button key={it.id} className={"rail-btn" + (page === it.id ? " on" : "")} data-tip={it.tip} onClick={() => setPage(it.id)}>
          <Icon name={it.icon} />
        </button>
      ))}
      <div className="sp" />
      <button className={"rail-btn pin" + (top ? " act" : "")} data-tip={top ? "已置顶窗口" : "窗口置顶"} onClick={() => { actions.toggleTop(); toast(top ? "已取消置顶" : "窗口已置顶", "pin"); }}>
        <Icon name={top ? "pin" : "pin-off"} />
      </button>
    </div>
  );
}

/* ---------------- Dashboard ---------------- */
function WorkflowRow({ wf, onTrigger, onEdit, onDelete, onFolder }) {
  return (
    <div className="card fade" style={{ display: "flex", alignItems: "center", gap: 13, padding: "13px 14px", marginBottom: 9, borderColor: wf.conflict ? "color-mix(in srgb,var(--bad) 36%,var(--bd))" : "var(--bd)" }}>
      <div style={{ width: 40, height: 40, flex: "none", borderRadius: "var(--r-btn)", display: "grid", placeItems: "center", background: "var(--accent-soft)", color: "var(--accent)" }}>
        <Icon name={MODES[wf.mode].icon} size={19} />
      </div>

      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 13.5, fontWeight: 600, display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{wf.name}</span>
          {wf.preset && <span className="tagm" style={{ fontSize: 9.5, padding: "1px 6px", color: "var(--faint)" }}>系统预设</span>}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 7, marginTop: 6, flexWrap: "wrap" }}>
          <ModeBadge engine={wf.engine} mode={wf.mode} />
          <span className={"fmt" + (wf.fmt === "JPG" ? " jpg" : "")}>{wf.fmt}</span>
          <button className="tagm" style={{ display: "inline-flex", alignItems: "center", gap: 5, cursor: "pointer" }} onClick={() => onFolder(wf)} title="打开文件夹">
            <Icon name="folder" size={11} />
            <span style={{ maxWidth: 150, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{wf.folder}</span>
          </button>
        </div>
      </div>

      {/* two independent status indicators */}
      <div style={{ width: 118, flex: "none", display: "flex", flexDirection: "column", gap: 5, alignItems: "flex-start" }}>
        <span className={"statline" + (wf.engineReady ? "" : " warn-t")}>
          <span className={"dot " + (wf.engineReady ? "ok" : "warn")} />
          {wf.engineReady ? "引擎就绪" : "引擎未就绪"}
        </span>
        {wf.conflict
          ? <span className="statline bad-t" style={{ fontWeight: 600 }}><Icon name="alert-triangle" />热键冲突</span>
          : <span className="statline"><Icon name="keyboard" size={12} />热键正常</span>}
      </div>

      <span className={"kbd" + (wf.conflict ? " bad" : "")} style={{ flex: "none" }}>{wf.hotkey}</span>

      <div style={{ display: "flex", gap: 3, flex: "none" }}>
        <button className="icon-btn go" title="立即触发" onClick={() => onTrigger(wf)}><Icon name="zap" /></button>
        <button className="icon-btn" title="编辑" onClick={() => onEdit(wf)}><Icon name="pencil" /></button>
        {wf.preset
          ? <button className="icon-btn" disabled title="系统预设 · 不可删除"><Icon name="lock" /></button>
          : <button className="icon-btn danger" title="删除" onClick={() => onDelete(wf)}><Icon name="trash-2" /></button>}
      </div>
    </div>
  );
}

function DashboardPage({ goSettings }) {
  const workflows = useStore(appStore, (s) => s.workflows);
  const [emptyDemo, setEmptyDemo] = useState(false);
  const list = emptyDemo ? [] : workflows;
  const conflicts = workflows.filter((w) => w.conflict).length;
  const notReady = workflows.filter((w) => !w.engineReady).length;

  const trigger = (wf) => { actions.pushActivity({ type: "screenshot", name: wf.name, t: new Date().toTimeString().slice(0, 8), path: wf.toClip && !wf.toFile ? "剪贴板" : wf.folder + "/shot.png" }); toast("已触发：" + wf.name, "zap"); };
  const del = (wf) => { actions.deleteWorkflow(wf.id); toast("已删除工作流", "trash-2"); };
  const folder = (wf) => toast("打开 " + wf.folder, "folder-open");

  return (
    <div className="page">
      <header style={{ display: "flex", alignItems: "center", gap: 12, padding: "16px 22px", borderBottom: "1px solid var(--bd)" }}>
        <div>
          <div style={{ fontSize: 17, fontWeight: 800, letterSpacing: "-.02em" }}>工作流</div>
          <div style={{ fontSize: 11.5, color: "var(--mut)", marginTop: 2, display: "flex", gap: 12 }}>
            <span>{workflows.length} 条 · 每个热键就是一条流水线</span>
          </div>
        </div>
        {/* scan-at-a-glance summary */}
        <div style={{ display: "flex", gap: 7, marginLeft: 8 }}>
          {conflicts > 0 && <span className="statline bad-t" style={{ fontWeight: 600, background: "var(--bad-soft)", padding: "5px 10px", borderRadius: "var(--r-pill)" }}><Icon name="alert-triangle" />{conflicts} 个热键冲突</span>}
          {notReady > 0 && <span className="statline warn-t" style={{ fontWeight: 600, background: "var(--warn-soft)", padding: "5px 10px", borderRadius: "var(--r-pill)" }}><span className="dot warn" />{notReady} 个引擎未就绪</span>}
        </div>
        <div style={{ marginLeft: "auto", display: "flex", gap: 8 }}>
          <button className="icon-btn" title="预览空态" onClick={() => setEmptyDemo((v) => !v)}><Icon name={emptyDemo ? "eye-off" : "eye"} /></button>
          <button className="btn btn-accent" onClick={() => goSettings("workflows", "new")}><Icon name="plus" />新建工作流</button>
        </div>
      </header>

      <div className="page-scroll" style={{ padding: 16 }}>
        {list.length === 0
          ? <EmptyState icon="workflow" title="还没有工作流" body="每个全局热键 = 一条可配置流水线：采集模式 → 渲染引擎 → 输出格式 → 保存位置。新建一条开始吧。" action="新建第一条工作流" onAction={() => { setEmptyDemo(false); goSettings("workflows", "new"); }} />
          : list.map((wf) => <WorkflowRow key={wf.id} wf={wf} onTrigger={trigger} onEdit={(w) => goSettings("workflows", w.id)} onDelete={del} onFolder={folder} />)}
      </div>
    </div>
  );
}

/* ---------------- Activity ---------------- */
const ACT_ICON = { screenshot: "camera", ocr: "scan-text", scroll: "gallery-vertical-end" };
const ACT_TINT = { screenshot: "var(--accent)", ocr: "#22d3ee", scroll: "#f59e0b" };

function ActivityPage() {
  const activity = useStore(appStore, (s) => s.activity);
  const workflows = useStore(appStore, (s) => s.workflows);
  // storage pools grouped by folder
  const pools = Object.values(workflows.reduce((acc, w) => {
    (acc[w.folder] = acc[w.folder] || { folder: w.folder, names: [], count: 0 }).names.push(w.name);
    acc[w.folder].count += Math.floor(Math.random() * 18) + 3;
    return acc;
  }, {}));

  return (
    <div className="page">
      <header style={{ display: "flex", alignItems: "center", gap: 12, padding: "16px 22px", borderBottom: "1px solid var(--bd)" }}>
        <div style={{ fontSize: 17, fontWeight: 800, letterSpacing: "-.02em", whiteSpace: "nowrap", flex: "none" }}>活动中心</div>
        <span style={{ display: "inline-flex", alignItems: "center", gap: 7, fontFamily: "var(--mono)", fontSize: 10.5, fontWeight: 600, letterSpacing: ".12em", color: "var(--bad)", border: "1px solid color-mix(in srgb,var(--bad) 40%,var(--bd))", padding: "4px 9px", borderRadius: "var(--r-pill)" }}>
          <span className="live-dot" />LIVE
        </span>
        <div style={{ marginLeft: "auto", fontSize: 11.5, color: "var(--mut)" }}>实时活动流 · 按工作流分组的存储池</div>
      </header>

      <div className="page-scroll" style={{ padding: 16, display: "grid", gridTemplateColumns: "1.5fr 1fr", gap: 16, alignItems: "start" }}>
        {/* activity stream */}
        <div>
          <div className="sect-title" style={{ marginBottom: 10 }}>实时活动流</div>
          {activity.map((a) => (
            <div key={a.id} className="card fade" style={{ display: "flex", alignItems: "center", gap: 12, padding: "11px 13px", marginBottom: 8 }}>
              <div style={{ width: 34, height: 34, flex: "none", borderRadius: "var(--r-btn)", display: "grid", placeItems: "center", background: "color-mix(in srgb," + ACT_TINT[a.type] + " 16%,transparent)", color: ACT_TINT[a.type] }}>
                <Icon name={ACT_ICON[a.type]} size={16} />
              </div>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 12.5, fontWeight: 600 }}>{a.name}</div>
                <div className="mono" style={{ fontSize: 10.5, color: "var(--mut)", marginTop: 3, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{a.path}</div>
              </div>
              <span className="mono" style={{ fontSize: 10.5, color: "var(--faint)", flex: "none" }}>{a.t}</span>
              <button className="icon-btn" title="打开" onClick={() => toast("打开 " + a.path, "folder-open")}><Icon name="external-link" /></button>
            </div>
          ))}
        </div>

        {/* storage pools */}
        <div>
          <div className="sect-title" style={{ marginBottom: 10 }}>存储池</div>
          {pools.map((p, i) => (
            <button key={i} className="card fade" style={{ width: "100%", textAlign: "left", display: "flex", alignItems: "center", gap: 12, padding: "13px", marginBottom: 8, cursor: "pointer" }} onClick={() => toast("打开 " + p.folder, "folder-open")}>
              <div style={{ width: 36, height: 36, flex: "none", borderRadius: "var(--r-btn)", display: "grid", placeItems: "center", background: "var(--bg0)", color: "var(--mut)", border: "1px solid var(--bd)" }}>
                <Icon name="folder" size={17} />
              </div>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="mono" style={{ fontSize: 11, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.folder}</div>
                <div style={{ fontSize: 10.5, color: "var(--mut)", marginTop: 3, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.names.join(" · ")}</div>
              </div>
              <span className="fmt" style={{ flex: "none" }}>{p.count}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

/* ---------------- Main window shell ---------------- */
function MainWindow() {
  const [page, setPage] = useState("dashboard");
  const [stab, setStab] = useState("general");
  const [editTarget, setEditTarget] = useState(null);
  const top = useStore(appStore, (s) => s.alwaysOnTop);

  const goSettings = (tab, edit = null) => { setStab(tab); setEditTarget(edit); setPage("settings"); };

  return (
    <div className="win" style={{ height: "min(620px,82vh)" }}>
      <div className="titlebar">
        <div className="tlogo"><Icon name="crop" /></div>
        <div className="tname">NexSpot <b className="mono">{top ? "· 置顶" : ""}</b></div>
        <div className="wbtns">
          <button title="最小化"><Icon name="minus" /></button>
          <button title="最大化"><Icon name="square" size={11} /></button>
          <button className="close" title="关闭"><Icon name="x" /></button>
        </div>
      </div>
      <div className="win-body">
        <Rail page={page} setPage={setPage} />
        {page === "dashboard" && <DashboardPage goSettings={goSettings} />}
        {page === "activity" && <ActivityPage />}
        {page === "settings" && <SettingsPage tab={stab} setTab={setStab} editTarget={editTarget} setEditTarget={setEditTarget} />}
      </div>
    </div>
  );
}

Object.assign(window, { MainWindow });
