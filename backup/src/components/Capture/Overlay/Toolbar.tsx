import { invoke } from '@tauri-apps/api/core';

interface ToolbarProps {
    x: number;
    y: number;
    width: number; // Used for positioning logic
    onConfirm: () => void;
    onCancel: () => void;
}

export default function Toolbar({ x, y, width, onConfirm, onCancel }: ToolbarProps) {
    if (width <= 30) return null;

    // Check distance to bottom
    const isNearBottom = (y + 100) > window.innerHeight; // Heuristic

    return (
        <div
            className={`absolute right-0 flex gap-1 bg-slate-900/95 p-1 rounded border border-white/10 pointer-events-auto cursor-default shadow-lg z-[99999] transition-all`}
            style={{
                pointerEvents: 'auto',
                bottom: isNearBottom ? '100%' : '-3rem', // Flip to top if near bottom
                marginBottom: isNearBottom ? '0.5rem' : '0'
            }}
        >
            <button
                onClick={(e) => {
                    e.stopPropagation();
                    invoke('frontend_log', { msg: "Cancel Clicked" });
                    onCancel();
                }}
                className="p-1 hover:bg-white/10 rounded text-white/60 transition-colors"
                title="Cancel (Esc)"
            >
                ✕
            </button>
            <button
                onClick={(e) => {
                    e.stopPropagation();
                    invoke('frontend_log', { msg: "Save Clicked" });
                    onConfirm();
                }}
                className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1 rounded text-[11px] font-bold transition-colors shadow-sm"
                title="Save (Enter)"
            >
                ✔ OK
            </button>
        </div>
    );
}
