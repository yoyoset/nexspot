// =============================================================================
// NexSpot · Zustand store contract  (reference — recreate in the real codebase)
// The prototype's `appStore` maps 1:1 to this. Split into slices if you prefer.
// =============================================================================

export type Engine = "gdi" | "vello";
export type CaptureMode = "region" | "full" | "window" | "fixed";
export type Fmt = "PNG" | "JPG";
export type VelloStyle = "Default" | "Neon" | "PaperCut" | "Sketch" | "Glass";
export type ThemeMode = "light" | "dark" | "system";
export type Lang = "zh" | "en";

export interface Workflow {
  id: string;
  name: string;
  engine: Engine;
  mode: CaptureMode;
  fmt: Fmt;
  folder: string;          // target save folder
  hotkey: string;          // e.g. "Ctrl+Shift+A"
  toFile: boolean;         // save to disk
  toClip: boolean;         // copy to clipboard
  preset: boolean;         // system preset → editable, NOT deletable
  style?: VelloStyle;      // only when engine === "vello"
  size?: string;           // only when mode === "fixed", e.g. "1080×1080"

  // --- two INDEPENDENT live status signals (drive the at-a-glance scan) ---
  engineReady: boolean;    // false → yellow "引擎未就绪" + warn dot
  conflict: boolean;       // true  → red "热键冲突" + red kbd  (hotkey clashes w/ another wf or OS)
}

export type ActivityType = "screenshot" | "ocr" | "scroll";
export interface ActivityItem {
  id: string;
  type: ActivityType;      // icon: camera / scan-text / gallery-vertical-end
  name: string;            // workflow or action name
  t: string;               // "14:32:08"
  path: string;            // file path OR "剪贴板 · 412 字"
}

export interface PinCard {
  id: string;
  title: string;
  w: number; h: number;    // card size (resizable; enforce a min size)
  // image: Blob / dataURL in the real app
}

export interface Settings {
  theme: ThemeMode;        // applied to <html data-theme>; "system" → matchMedia
  accent: string;          // hex; the ONLY accent source → derive --accent-press / --on-accent
  lang: Lang;
  savePath: string;
  annFont: string;         // annotation font for the native text tool
  jpgQuality: number;      // 40–100
  defaultFmt: Fmt;
  concurrency: number;     // 1–8
  defaultSize: string;     // default fixed-snapshot size, "1920×1080"
  velloOn: boolean;        // Vello engine master switch (independent of GDI)
  velloStyle: VelloStyle;
  advEffects: boolean;     // advanced effects (shadow/glow/glass)
}

export interface AppState extends Settings {
  workflows: Workflow[];
  activity: ActivityItem[];
  pins: PinCard[];
  alwaysOnTop: boolean;    // rail bottom toggle (置顶)

  // actions
  toggleTop: () => void;
  addWorkflow: (wf: Omit<Workflow, "id">) => void;
  updateWorkflow: (wf: Workflow) => void;
  deleteWorkflow: (id: string) => void;        // refuse when wf.preset === true
  triggerWorkflow: (id: string) => void;       // run capture → pushActivity(...)
  pushActivity: (a: Omit<ActivityItem, "id">) => void;
  removePin: (id: string) => void;
  set: (patch: Partial<Settings>) => void;     // settings writes
}

export const ACCENTS: [hex: string, name: string][] = [
  ["#7a6ff2", "Periwinkle"], ["#4f8cff", "Blue"], ["#16b8a6", "Teal"],
  ["#f59e0b", "Amber"], ["#f4517b", "Rose"], ["#46b86a", "Green"],
];

export const MODES: Record<CaptureMode, { label: string; icon: string }> = {
  region: { label: "区域选取", icon: "scan" },
  full:   { label: "全屏",     icon: "monitor" },
  window: { label: "窗口捕获", icon: "app-window" },
  fixed:  { label: "固定尺寸", icon: "crop" },
};
export const VELLO_STYLES: VelloStyle[] = ["Default", "Neon", "PaperCut", "Sketch", "Glass"];

/* --- theme/accent application (run in an effect on theme/accent change) ---
function hexLum(hex: string) {
  const m = hex.replace("#", "");
  const [r, g, b] = [0, 2, 4].map(i => parseInt(m.slice(i, i + 2), 16) / 255);
  const f = (c: number) => (c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}
function applyTheme(theme: ThemeMode, accent: string) {
  const t = theme === "system"
    ? (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
    : theme;
  const root = document.documentElement;
  root.setAttribute("data-theme", t);
  root.style.setProperty("--accent", accent);
  root.style.setProperty("--accent-press", `color-mix(in srgb, ${accent} 82%, #000)`);
  root.style.setProperty("--on-accent", hexLum(accent) > 0.55 ? "#1b1c1f" : "#ffffff");
}
*/
