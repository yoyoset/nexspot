/* ===================== NexSpot · floating windows (PIN / Scroll / OCR) ===================== */

function FwBar({ icon, title, count, onClose, children }) {
  return (
    <div className="fw-bar" title="标题栏可拖动">
      <span className="grip"><i /><i /><i /><i /><i /><i /></span>
      <span className="fw-title"><Icon name={icon} />{title}</span>
      {count && <span className="fw-count">{count}</span>}
      <div style={{ marginLeft: "auto", display: "flex", gap: 2 }}>
        {children}
        <button className="icon-btn" style={{ width: 26, height: 24 }} onClick={onClose} title="关闭"><Icon name="x" size={14} /></button>
      </div>
    </div>
  );
}

/* ---------------- PIN collection ---------------- */
function PinCard({ pin, onSave, onCopy, onDelete }) {
  const [hover, setHover] = useState(false);
  return (
    <div className="card" onMouseEnter={() => setHover(true)} onMouseLeave={() => setHover(false)}
      style={{ overflow: "hidden", background: "var(--bg1)", position: "relative", cursor: "grab" }}>
      <div style={{ height: 30, display: "flex", alignItems: "center", gap: 7, padding: "0 6px 0 11px", borderBottom: "1px solid var(--bd)" }}>
        <Icon name="grip-vertical" size={12} style={{ color: "var(--faint)" }} />
        <span style={{ fontSize: 11.5, fontWeight: 600, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{pin.title}</span>
        <div style={{ display: "flex", gap: 1, opacity: hover ? 1 : 0, transition: ".14s" }}>
          <button className="icon-btn" style={{ width: 24, height: 22 }} title="保存" onClick={onSave}><Icon name="download" size={13} /></button>
          <button className="icon-btn" style={{ width: 24, height: 22 }} title="复制" onClick={onCopy}><Icon name="copy" size={13} /></button>
          <button className="icon-btn danger" style={{ width: 24, height: 22 }} title="删除" onClick={onDelete}><Icon name="x" size={13} /></button>
        </div>
      </div>
      <Ph label="screenshot" style={{ height: pin.h ? pin.h - 30 : 120, borderRadius: 0 }} />
      <div style={{ position: "absolute", bottom: 6, right: 6, display: "flex", alignItems: "center", gap: 4, opacity: hover ? 1 : 0, transition: ".14s", background: "var(--glass)", backdropFilter: "blur(6px)", padding: "3px 7px", borderRadius: 6, fontSize: 9.5, color: "var(--mut)", fontFamily: "var(--mono)" }}>
        <Icon name="move" size={10} />拖出粘贴
      </div>
    </div>
  );
}

function PinWindow() {
  const pins = useStore(appStore, (s) => s.pins);
  const MAX = 24;
  return (
    <div className="float-win" style={{ width: "min(580px,94vw)", height: 470 }}>
      <FwBar icon="pin" title="PIN 合集" count={pins.length + " / " + MAX}>
        <button className="icon-btn" style={{ width: 26, height: 24 }} title="全部保存" onClick={() => toast("已保存 " + pins.length + " 张", "download")}><Icon name="download" size={14} /></button>
      </FwBar>
      <div className="page-scroll" style={{ flex: 1, padding: 12 }}>
        {pins.length === 0
          ? <div style={{ display: "grid", placeItems: "center", height: "100%", color: "var(--mut)", fontSize: 12.5, gap: 10, textAlign: "center" }}><Icon name="pin-off" size={26} /><div>合集已空<br /><span style={{ fontSize: 11, color: "var(--faint)" }}>截图时点 PIN 把它钉到这里</span></div></div>
          : <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              {pins.map((p) => (
                <PinCard key={p.id} pin={p}
                  onSave={() => toast("已保存：" + p.title, "download")}
                  onCopy={() => toast("已复制：" + p.title, "copy")}
                  onDelete={() => { actions.removePin(p.id); toast("已移除卡片", "x"); }} />
              ))}
            </div>}
      </div>
      <div style={{ height: 34, flex: "none", borderTop: "1px solid var(--bd)", display: "flex", alignItems: "center", gap: 10, padding: "0 12px" }}>
        <span style={{ fontFamily: "var(--mono)", fontSize: 10, color: "var(--faint)" }}>可视化的临时剪贴板合集</span>
        <div style={{ marginLeft: "auto", display: "flex", gap: 5, alignItems: "center" }}>
          {[0, 1, 2].map((i) => <span key={i} style={{ width: i === 0 ? 16 : 6, height: 6, borderRadius: 99, background: i === 0 ? "var(--accent)" : "var(--bd2)" }} />)}
        </div>
      </div>
      <Icon name="grip" className="resize-grip" size={14} />
    </div>
  );
}

/* ---------------- Scroll long-screenshot preview ---------------- */
function ScrollWindow() {
  const [zoom, setZoom] = useState(100);
  return (
    <div className="float-win" style={{ width: "min(430px,92vw)", height: 540 }}>
      <FwBar icon="gallery-vertical-end" title="滚动长截图" count="1080 × 5240">
        <button className="icon-btn" style={{ width: 26, height: 24 }} title="缩小" onClick={() => setZoom((z) => Math.max(40, z - 20))}><Icon name="zoom-out" size={14} /></button>
        <button className="icon-btn" style={{ width: 26, height: 24 }} title="放大" onClick={() => setZoom((z) => Math.min(140, z + 20))}><Icon name="zoom-in" size={14} /></button>
      </FwBar>
      <div className="page-scroll" style={{ flex: 1, padding: 14, background: "var(--bg0)" }}>
        <div style={{ width: zoom + "%", margin: "0 auto", transition: ".18s" }}>
          {[0, 1, 2, 3].map((i) => (
            <div key={i} style={{ position: "relative" }}>
              <Ph label={i === 0 ? "拼接长图 · 段 1" : "段 " + (i + 1)} style={{ height: 150, borderRadius: i === 0 ? "8px 8px 0 0" : i === 3 ? "0 0 8px 8px" : 0, borderBottom: i < 3 ? "1px dashed var(--accent-line)" : "none" }} />
              {i < 3 && <div style={{ position: "absolute", left: "50%", bottom: -1, transform: "translate(-50%,50%)", fontFamily: "var(--mono)", fontSize: 8.5, color: "var(--accent)", background: "var(--bg0)", padding: "1px 6px", borderRadius: 99, border: "1px solid var(--accent-line)" }}>接缝</div>}
            </div>
          ))}
        </div>
      </div>
      <div style={{ height: 40, flex: "none", borderTop: "1px solid var(--bd)", display: "flex", alignItems: "center", gap: 8, padding: "0 12px" }}>
        <span className="mono" style={{ fontSize: 10.5, color: "var(--mut)" }}>缩放 {zoom}%</span>
        <div style={{ marginLeft: "auto", display: "flex", gap: 7 }}>
          <button className="btn btn-sm" onClick={() => toast("已保存长图", "download")}><Icon name="download" />保存</button>
          <button className="btn btn-sm btn-accent" onClick={() => toast("已复制到剪贴板", "copy")}><Icon name="copy" />复制</button>
        </div>
      </div>
      <Icon name="grip" className="resize-grip" size={14} />
    </div>
  );
}

/* ---------------- OCR result ---------------- */
function OcrWindow() {
  return (
    <div className="float-win" style={{ width: "min(460px,92vw)", height: 420 }}>
      <FwBar icon="scan-text" title="OCR 结果" count="412 字 · 98%">
        <button className="icon-btn" style={{ width: 26, height: 24 }} title="复制全部" onClick={() => toast("已复制全部文字", "copy")}><Icon name="copy" size={14} /></button>
      </FwBar>
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", borderBottom: "1px solid var(--bd)" }}>
        <Ph label="源图" style={{ width: 52, height: 38, flex: "none", borderRadius: 7 }} />
        <span style={{ fontSize: 11.5, color: "var(--mut)" }}>识别自 <span className="mono" style={{ color: "var(--tx)" }}>shot_1428.png</span></span>
        <span className="chip on" style={{ marginLeft: "auto", fontSize: 10.5 }}><Icon name="badge-check" size={12} />置信度 98%</span>
      </div>
      <div className="page-scroll" style={{ flex: 1, padding: 16 }}>
        <div style={{ fontSize: 13, lineHeight: 1.85, whiteSpace: "pre-wrap", userSelect: "text", color: "var(--tx)" }}>{OCR_TEXT}</div>
      </div>
      <div style={{ height: 42, flex: "none", borderTop: "1px solid var(--bd)", display: "flex", alignItems: "center", gap: 8, padding: "0 12px" }}>
        <span style={{ fontSize: 10.5, color: "var(--faint)" }}>可选中复制任意片段</span>
        <div style={{ marginLeft: "auto", display: "flex", gap: 7 }}>
          <button className="btn btn-sm" onClick={() => toast("已导出 result.txt", "file-text")}><Icon name="file-text" />导出 .txt</button>
          <button className="btn btn-sm btn-accent" onClick={() => toast("已复制全部文字", "copy")}><Icon name="copy" />复制全部</button>
        </div>
      </div>
      <Icon name="grip" className="resize-grip" size={14} />
    </div>
  );
}

Object.assign(window, { PinWindow, ScrollWindow, OcrWindow });
