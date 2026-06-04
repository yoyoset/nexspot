/* ===================== NexSpot · native annotation toolbar — visual spec ===================== */

const TOOLS = [
  { icon: "square", name: "矩形" }, { icon: "circle", name: "椭圆" },
  { icon: "minus", name: "直线" }, { icon: "move-up-right", name: "箭头" },
  { icon: "pen-line", name: "画笔" }, { icon: "type", name: "文字" },
  { icon: "hash", name: "序号" }, { icon: "grid-3x3", name: "马赛克" },
];
const ACTIONS = [
  { icon: "undo-2", name: "撤销" }, { icon: "pin", name: "PIN 贴图" },
  { icon: "download", name: "保存" }, { icon: "copy", name: "复制" }, { icon: "x", name: "关闭", danger: true },
];

function Marker({ n, style }) {
  return <span style={{ position: "absolute", width: 22, height: 22, borderRadius: 99, background: "var(--accent)", color: "#fff", fontSize: 11.5, fontWeight: 800, display: "grid", placeItems: "center", boxShadow: "0 4px 12px -3px var(--accent)", zIndex: 6, ...style }}>{n}</span>;
}

/* the floating toolbar exactly as it should render natively */
function NativeToolbar({ sel = 0 }) {
  return (
    <div className="toolbar" style={{ display: "flex", alignItems: "center", gap: 2, padding: 6, borderRadius: 13, background: "var(--bg1)", border: "1px solid var(--bd2)", boxShadow: "var(--shadow-float)" }}>
      {TOOLS.map((t, i) => (
        <button key={t.name} title={t.name} style={btnStyle(i === sel)}>
          <Icon name={t.icon} size={16} />
        </button>
      ))}
      <span style={sepStyle} />
      {ACTIONS.slice(0, 2).map((t) => <button key={t.name} title={t.name} style={btnStyle(false)}><Icon name={t.icon} size={16} /></button>)}
      <span style={sepStyle} />
      {ACTIONS.slice(2).map((t) => <button key={t.name} title={t.name} style={btnStyle(false, t.danger)}><Icon name={t.icon} size={16} /></button>)}
    </div>
  );
}
function btnStyle(on, danger) {
  return { width: 32, height: 32, border: 0, borderRadius: 8, display: "grid", placeItems: "center", background: on ? "var(--accent)" : "transparent", color: on ? "var(--on-accent)" : danger ? "var(--bad)" : "var(--tx)" };
}
const sepStyle = { width: 1, height: 20, background: "var(--bd)", margin: "0 4px", display: "block" };

function PropBar() {
  return (
    <div className="propbar" style={{ display: "flex", alignItems: "center", gap: 11, padding: "7px 11px", borderRadius: 11, background: "var(--bg1)", border: "1px solid var(--bd2)", boxShadow: "var(--shadow)" }}>
      {/* size */}
      <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
        <Icon name="pen-line" size={12} style={{ color: "var(--mut)" }} />
        {[4, 7, 11].map((d, i) => <span key={d} style={{ width: d, height: d, borderRadius: 99, background: "var(--tx)", outline: i === 2 ? "2px solid var(--accent)" : "none", outlineOffset: 2 }} />)}
      </div>
      <span style={{ width: 1, height: 16, background: "var(--bd)" }} />
      {/* color */}
      <div style={{ display: "flex", gap: 5 }}>
        {["#f76d6d", "#f5c451", "#57d9a3", "#4f8cff", "#7a6ff2", "#f3f3f3"].map((c, i) => (
          <span key={c} style={{ width: 16, height: 16, borderRadius: 5, background: c, cursor: "pointer", border: "1.5px solid rgba(255,255,255,.12)", boxShadow: i === 4 ? "0 0 0 2px var(--bg1),0 0 0 3.5px var(--accent)" : "none" }} />
        ))}
      </div>
      <span style={{ width: 1, height: 16, background: "var(--bd)" }} />
      {/* opacity (vello) */}
      <div style={{ display: "flex", alignItems: "center", gap: 7 }} title="仅 Vello 引擎">
        <span style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--accent)" }}>透明度</span>
        <span style={{ width: 54, height: 4, borderRadius: 99, background: "var(--bg3)", position: "relative" }}><span style={{ position: "absolute", left: 0, top: 0, height: "100%", width: "78%", borderRadius: 99, background: "var(--accent)" }} /></span>
        <span style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--mut)" }}>78%</span>
      </div>
      <span style={{ width: 1, height: 16, background: "var(--bd)" }} />
      {/* fill (rect/ellipse) */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }} title="矩形 / 椭圆">
        <span style={{ fontSize: 11, color: "var(--mut)" }}>填充</span>
        <span className="toggle on" style={{ width: 30, height: 17 }}><span style={{ position: "absolute", top: 2, left: 14, width: 11, height: 11, borderRadius: 99, background: "#fff" }} /></span>
      </div>
    </div>
  );
}

/* selected object with 8 direction handles + connection + rotate */
function SelectedObject() {
  const H = (x, y) => <span style={{ position: "absolute", left: x, top: y, transform: "translate(-50%,-50%)", width: 10, height: 10, background: "var(--bg1)", border: "2px solid var(--accent)", borderRadius: 2, zIndex: 3 }} />;
  return (
    <div style={{ position: "absolute", left: 60, top: 78, width: 150, height: 96, zIndex: 2 }}>
      <div style={{ position: "absolute", inset: 0, border: "2px solid var(--accent)", borderRadius: 4, background: "color-mix(in srgb,var(--accent) 10%,transparent)" }} />
      {/* connection line to rotate handle */}
      <span style={{ position: "absolute", left: "50%", top: -26, width: 2, height: 26, background: "var(--accent)", transform: "translateX(-50%)" }} />
      <span style={{ position: "absolute", left: "50%", top: -30, transform: "translate(-50%,-50%)", width: 12, height: 12, borderRadius: 99, background: "var(--bg1)", border: "2px solid var(--accent)", display: "grid", placeItems: "center", zIndex: 3 }}><Icon name="rotate-cw" size={7} style={{ color: "var(--accent)" }} /></span>
      {[["0", "0"], ["50%", "0"], ["100%", "0"], ["100%", "50%"], ["100%", "100%"], ["50%", "100%"], ["0", "100%"], ["0", "50%"]].map(([x, y], i) => <span key={i}>{H(x, y)}</span>)}
    </div>
  );
}

function Magnifier({ style }) {
  return (
    <div style={{ position: "absolute", width: 116, ...style, zIndex: 6 }}>
      <div style={{ width: 96, height: 96, borderRadius: "50%", overflow: "hidden", border: "3px solid var(--bg1)", boxShadow: "var(--shadow-float)", position: "relative", background: "#1a2740", backgroundImage: "repeating-linear-gradient(0deg,rgba(255,255,255,.08) 0 11px,transparent 11px 12px),repeating-linear-gradient(90deg,rgba(255,255,255,.08) 0 11px,transparent 11px 12px)" }}>
        <span style={{ position: "absolute", left: "50%", top: 0, width: 1, height: "100%", background: "var(--accent)", transform: "translateX(-50%)" }} />
        <span style={{ position: "absolute", top: "50%", left: 0, height: 1, width: "100%", background: "var(--accent)", transform: "translateY(-50%)" }} />
        <span style={{ position: "absolute", left: "50%", top: "50%", width: 13, height: 13, transform: "translate(-50%,-50%)", border: "1.5px solid var(--accent)", boxShadow: "0 0 0 1px rgba(0,0,0,.5)" }} />
      </div>
      <div style={{ marginTop: 6, background: "var(--bg1)", border: "1px solid var(--bd2)", borderRadius: 8, padding: "6px 8px", fontFamily: "var(--mono)", fontSize: 9.5, lineHeight: 1.65, boxShadow: "var(--shadow)" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 5 }}><span style={{ width: 9, height: 9, borderRadius: 2, background: "#4f8cff" }} />#4F8CFF</div>
        <div style={{ color: "var(--mut)" }}>RGB 79,140,255</div>
        <div style={{ color: "var(--mut)" }}>XY 412,288</div>
      </div>
    </div>
  );
}

function SpecRow({ k, v }) {
  return <div style={{ display: "flex", justifyContent: "space-between", gap: 12, padding: "7px 0", borderBottom: "1px dashed var(--bd)", fontSize: 11.5 }}><span style={{ color: "var(--mut)" }}>{k}</span><span className="mono" style={{ color: "var(--tx)", fontWeight: 600, textAlign: "right" }}>{v}</span></div>;
}

function ToolbarSpec() {
  return (
    <div className="win" style={{ width: "min(1040px,94vw)", height: "min(640px,84vh)" }}>
      <div className="titlebar">
        <div className="tlogo"><Icon name="crop" /></div>
        <div className="tname">原生截图工具栏 <b className="mono">视觉规格 · 由 Rust 原生层还原</b></div>
      </div>
      <div className="win-body" style={{ flex: 1, minHeight: 0 }}>
        {/* live spec stage */}
        <div style={{ flex: 1, minWidth: 0, position: "relative", background: "var(--bg0)", overflow: "hidden" }}>
          {/* faux desktop */}
          <Ph label="" style={{ position: "absolute", inset: 0, borderRadius: 0, opacity: .5 }} />
          {/* dim overlay outside selection */}
          <div style={{ position: "absolute", inset: 0, background: "rgba(0,0,0,.42)" }} />
          {/* selection region (un-dimmed) */}
          <div style={{ position: "absolute", left: 36, top: 54, width: 320, height: 188, boxShadow: "0 0 0 9999px rgba(0,0,0,.42)", border: "1.5px solid var(--accent)", borderRadius: 2 }}>
            {/* selection corner/edge handles */}
            {[["0", "0"], ["50%", "0"], ["100%", "0"], ["100%", "50%"], ["100%", "100%"], ["50%", "100%"], ["0", "100%"], ["0", "50%"]].map(([x, y], i) => (
              <span key={i} style={{ position: "absolute", left: x, top: y, transform: "translate(-50%,-50%)", width: 9, height: 9, background: "var(--accent)", border: "1.5px solid #fff", borderRadius: 2 }} />
            ))}
            {/* size readout */}
            <span style={{ position: "absolute", left: 0, top: -22, fontFamily: "var(--mono)", fontSize: 10, fontWeight: 600, color: "#fff", background: "var(--accent)", padding: "2px 7px", borderRadius: 5 }}>320 × 188</span>
            <Marker n="1" style={{ right: -11, top: -11 }} />
            <Marker n="4" style={{ left: 36, top: 30 }} />
            <SelectedObject />
          </div>

          {/* magnifier near cursor */}
          <Magnifier style={{ left: 300, top: 150 }} />
          <Marker n="5" style={{ left: 396, top: 150 }} />

          {/* toolbar + propbar below selection */}
          <div style={{ position: "absolute", left: 36, top: 254, display: "flex", flexDirection: "column", gap: 6 }}>
            <div style={{ position: "relative" }}><NativeToolbar sel={0} /><Marker n="2" style={{ right: -11, top: -11 }} /></div>
            <div style={{ position: "relative" }}><PropBar /><Marker n="3" style={{ right: -11, top: -11 }} /></div>
          </div>
        </div>

        {/* spec sheet */}
        <aside style={{ width: 320, flex: "none", borderLeft: "1px solid var(--bd)", background: "var(--bg1)", overflowY: "auto", padding: "18px 18px 28px" }}>
          <div style={{ fontSize: 15, fontWeight: 800, letterSpacing: "-.02em", marginBottom: 3 }}>规格说明</div>
          <div style={{ fontSize: 11, color: "var(--mut)", marginBottom: 16 }}>颜色与暗/亮主题共用同一组 token</div>

          {[["1", "选区 + 手柄", "1.5px 强调色描边 · 8 个 9px 手柄（白描边）· 外部 42% 黑遮罩 · 顶部尺寸读数"],
            ["2", "主工具栏", "浮于选区下方 8px；贴底时翻转到上方。高 44px、按钮 32px、圆角 13px、阴影替代边框"],
            ["3", "二级属性条", "选中绘图工具时在工具栏下方 6px 弹出：大小 / 颜色 / 透明度(仅 Vello) / 填充(矩形·椭圆)"],
            ["4", "选中对象手柄", "8 向方向手柄 + 顶部连接线 + 旋转手柄（圆形）"],
            ["5", "放大镜", "96px 圆形 · 像素网格 + 十字准星 · 取色/定位读数（HEX·RGB·XY）"]].map(([n, t, d]) => (
            <div key={n} style={{ display: "flex", gap: 10, marginBottom: 13 }}>
              <span style={{ flex: "none", width: 20, height: 20, borderRadius: 99, background: "var(--accent)", color: "#fff", fontSize: 11, fontWeight: 800, display: "grid", placeItems: "center" }}>{n}</span>
              <div><div style={{ fontSize: 12.5, fontWeight: 700 }}>{t}</div><div style={{ fontSize: 11, color: "var(--mut)", marginTop: 3, lineHeight: 1.55 }}>{d}</div></div>
            </div>
          ))}

          <div className="sect-title" style={{ margin: "18px 0 8px" }}>尺寸 token</div>
          <SpecRow k="工具栏高 / 按钮" v="44 / 32px" />
          <SpecRow k="工具栏圆角" v="13px" />
          <SpecRow k="按钮圆角" v="8px" />
          <SpecRow k="选区描边" v="1.5px accent" />
          <SpecRow k="手柄 / 旋转点" v="9 / 12px" />
          <SpecRow k="放大镜直径" v="96px" />
          <SpecRow k="与选区间距" v="8px (翻转感知)" />

          <div className="sect-title" style={{ margin: "18px 0 8px" }}>工具清单（左→右）</div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {[...TOOLS, ...ACTIONS].map((t) => (
              <span key={t.name} className="tagm" style={{ display: "inline-flex", alignItems: "center", gap: 5, color: t.danger ? "var(--bad)" : "var(--mut)" }}>
                <Icon name={t.icon} size={12} />{t.name}
              </span>
            ))}
          </div>
        </aside>
      </div>
    </div>
  );
}

Object.assign(window, { ToolbarSpec });
