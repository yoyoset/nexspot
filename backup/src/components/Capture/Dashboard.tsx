import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Shortcut {
    id: String;
    label: string;
    icon: string;
    color: string;
    enabled: boolean;
}

export default function Dashboard() {
    const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);

    useEffect(() => {
        const load = async () => {
            const list = await invoke<Shortcut[]>('get_shortcuts');
            setShortcuts(list);
        };
        load();
    }, []);

    const handleCapture = async () => {
        try {
            await invoke('trigger_capture');
        } catch (e) {
            console.error('Manual capture failed:', e);
        }
    };

    const handleOpenSettings = async () => {
        await invoke('open_settings');
    };

    const handleOpenGallery = async () => {
        await invoke('open_gallery');
    };

    return (
        <div className="min-h-screen w-full bg-slate-950 flex flex-col items-center justify-center p-8 text-slate-200">
            <div className="bento-grid">
                {/* Hero Tile: Start Capture */}
                <div className="bento-card hero" onClick={handleCapture}>
                    <div className="fluent-icon"></div> {/* Camera icon in Segoe Fluent */}
                    <h3 className="text-2xl font-black">Start Capture</h3>
                    <p className="font-bold border border-white/20 px-3 py-1 rounded-full bg-white/10 uppercase tracking-widest text-[10px] mt-4">
                        Ctrl + Shift + S
                    </p>
                </div>

                {/* System Tiles */}
                <div className="bento-card small" onClick={handleOpenGallery}>
                    <div className="fluent-icon text-amber-400"></div> {/* Folder */}
                    <h3>Gallery</h3>
                </div>

                <div className="bento-card small" onClick={handleOpenSettings}>
                    <div className="fluent-icon text-slate-400"></div> {/* Settings */}
                    <h3>Settings</h3>
                </div>

                {/* Shortcuts Grid (Apple Shortcuts style) */}
                {shortcuts.map((plugin) => (
                    <div
                        key={plugin.id as string}
                        className="bento-card small"
                        style={{ borderLeft: `4px solid ${plugin.color}` }}
                    >
                        <div className="fluent-icon" style={{ color: plugin.color }}>{plugin.icon}</div>
                        <h3>{plugin.label}</h3>
                    </div>
                ))}

                {/* Status Tile */}
                <div className="bento-card small cursor-default">
                    <div className="text-xs text-green-400 flex items-center gap-1.5 font-bold uppercase tracking-wider">
                        <span className="w-1.5 h-1.5 bg-green-400 rounded-full animate-pulse"></span>
                        Ready
                    </div>
                    <p className="text-[10px] opacity-40 uppercase font-black mt-2">v0.2.1 Production</p>
                </div>
            </div>

            <footer className="mt-12 text-[10px] text-slate-600 uppercase tracking-[0.2em] font-black">
                HyperLens Engine • High Performance Capture
            </footer>
        </div>
    );
}
