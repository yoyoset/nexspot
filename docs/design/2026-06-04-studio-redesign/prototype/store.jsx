/* ===================== NexSpot · store + mock data =====================
   A tiny Zustand-style external store so the handoff maps 1:1 to the real
   Zustand store. Shapes here = the store slices Claude Code should build.
======================================================================= */
const { useState, useEffect, useRef, useLayoutEffect, useCallback } = React;

/* ---- icon helper (lucide UMD, imperative -> no React reconciliation fights) ---- */
function Icon({ name, size = 16, stroke = 1.9, className = "", style }) {
  const ref = useRef(null);
  useLayoutEffect(() => {
    const L = window.lucide;
    const host = ref.current;
    if (!L || !host) return;
    const pascal = String(name).split("-").map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join("");
    const node = (L.icons && (L.icons[pascal] || L.icons[name])) || L[pascal] || L[name];
    host.innerHTML = "";
    if (node && L.createElement) {
      const svg = L.createElement(node);
      svg.setAttribute("width", size);
      svg.setAttribute("height", size);
      svg.setAttribute("stroke-width", stroke);
      host.appendChild(svg);
    }
  }, [name, size, stroke]);
  return <span ref={ref} className={"icon " + className} style={style} />;
}

/* ---------- tiny store ---------- */
function createStore(initial) {
  let state = initial;
  const subs = new Set();
  const get = () => state;
  const set = (patch) => {
    state = { ...state, ...(typeof patch === "function" ? patch(state) : patch) };
    subs.forEach((f) => f());
  };
  const subscribe = (f) => (subs.add(f), () => subs.delete(f));
  return { get, set, subscribe };
}

function useStore(store, selector = (s) => s) {
  const [, force] = useState(0);
  useEffect(() => store.subscribe(() => force((n) => n + 1)), [store]);
  return selector(store.get());
}

/* ---------- capture-mode / engine vocab ---------- */
const MODES = {
  region: { label: "区域选取", icon: "scan" },
  full: { label: "全屏", icon: "monitor" },
  window: { label: "窗口捕获", icon: "app-window" },
  fixed: { label: "固定尺寸", icon: "crop" },
};
const ENGINES = { gdi: "GDI", vello: "Vello" };
const VELLO_STYLES = ["Default", "Neon", "PaperCut", "Sketch", "Glass"];

/* ---------- seed data ---------- */
const SEED_WORKFLOWS = [
  { id: "wf-1", name: "区域截图 · 快速", engine: "gdi", mode: "region", fmt: "PNG",
    folder: "~/Pictures/NexSpot/Region", hotkey: "Ctrl+Shift+A", toFile: true, toClip: true,
    preset: true, engineReady: true, conflict: false },
  { id: "wf-2", name: "全屏到剪贴板", engine: "gdi", mode: "full", fmt: "PNG",
    folder: "~/Pictures/NexSpot/Full", hotkey: "PrtSc", toFile: false, toClip: true,
    preset: true, engineReady: true, conflict: false },
  { id: "wf-3", name: "设计稿 · Neon 风格化", engine: "vello", mode: "region", fmt: "PNG",
    folder: "~/Design/Shots", hotkey: "Ctrl+Alt+D", toFile: true, toClip: false,
    preset: false, engineReady: true, conflict: false, style: "Neon" },
  { id: "wf-4", name: "窗口捕获 · 工单存档", engine: "vello", mode: "window", fmt: "JPG",
    folder: "~/Work/Tickets", hotkey: "Ctrl+Shift+A", toFile: true, toClip: false,
    preset: false, engineReady: false, conflict: true },
  { id: "wf-5", name: "固定 1080² 快照", engine: "gdi", mode: "fixed", fmt: "PNG",
    folder: "~/Pictures/NexSpot/Square", hotkey: "Ctrl+Shift+9", toFile: true, toClip: false,
    preset: false, engineReady: true, conflict: false, size: "1080×1080" },
];

const SEED_ACTIVITY = [
  { id: "a1", type: "screenshot", name: "区域截图 · 快速", t: "14:32:08", path: "~/Pictures/NexSpot/Region/shot_1432.png" },
  { id: "a2", type: "ocr", name: "OCR · 发票识别", t: "14:28:51", path: "剪贴板 · 412 字" },
  { id: "a3", type: "scroll", name: "滚动长截图 · 对话存档", t: "14:21:03", path: "~/Work/Tickets/thread_long.png" },
  { id: "a4", type: "screenshot", name: "设计稿 · Neon 风格化", t: "14:09:44", path: "~/Design/Shots/hero_neon.png" },
  { id: "a5", type: "screenshot", name: "全屏到剪贴板", t: "13:58:12", path: "剪贴板" },
  { id: "a6", type: "scroll", name: "滚动长截图 · 文档", t: "13:40:27", path: "~/Documents/spec_full.png" },
];

const SEED_PINS = [
  { id: "p1", title: "登录态 bug", w: 240, h: 150 },
  { id: "p2", title: "配色参考", w: 240, h: 178 },
  { id: "p3", title: "报错堆栈", w: 240, h: 132 },
  { id: "p4", title: "竞品布局", w: 240, h: 160 },
];

const OCR_TEXT = `NexSpot 截图工作流
采集模式：区域选取 / 全屏 / 窗口捕获 / 固定尺寸
渲染引擎：GDI（极速纯色）· Vello（GPU 加速）
导出：PNG / JPG · 存文件 / 存剪贴板
全局热键：Ctrl + Shift + A`;

const appStore = createStore({
  workflows: SEED_WORKFLOWS,
  activity: SEED_ACTIVITY,
  pins: SEED_PINS,
  alwaysOnTop: false,
  // settings
  theme: "dark", accent: "#7a6ff2", lang: "zh",
  savePath: "~/Pictures/NexSpot", annFont: "Manrope",
  jpgQuality: 88, defaultFmt: "PNG", concurrency: 3, defaultSize: "1920×1080",
  velloOn: true, velloStyle: "Default", advEffects: false,
});

/* ---- actions (mirror Zustand actions) ---- */
const actions = {
  toggleTop: () => appStore.set((s) => ({ alwaysOnTop: !s.alwaysOnTop })),
  deleteWorkflow: (id) => appStore.set((s) => ({ workflows: s.workflows.filter((w) => w.id !== id) })),
  set: (patch) => appStore.set(patch),
  pushActivity: (item) =>
    appStore.set((s) => ({ activity: [{ id: "a" + Date.now(), ...item }, ...s.activity].slice(0, 30) })),
  removePin: (id) => appStore.set((s) => ({ pins: s.pins.filter((p) => p.id !== id) })),
};

const ACCENTS = [
  ["#7a6ff2", "Periwinkle"], ["#4f8cff", "Blue"], ["#16b8a6", "Teal"],
  ["#f59e0b", "Amber"], ["#f4517b", "Rose"], ["#46b86a", "Green"],
];

Object.assign(window, {
  Icon, createStore, useStore, appStore, actions,
  MODES, ENGINES, VELLO_STYLES, ACCENTS, OCR_TEXT,
  useState, useEffect, useRef, useLayoutEffect, useCallback,
});
