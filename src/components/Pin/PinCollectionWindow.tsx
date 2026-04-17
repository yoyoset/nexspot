import React, { useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { X, Trash2, Maximize2, Copy, Download, Loader2, Lock, Unlock, ChevronUp, ChevronDown, Terminal, Cpu, LayoutGrid, Grid2X2, StretchHorizontal } from 'lucide-react';
import { save } from '@tauri-apps/plugin-dialog';
import ReactMarkdown from 'react-markdown';
import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';

type PinData =
    | { type: 'Text'; payload: string }
    | { type: 'ImageBase64'; payload: string };

interface PinItem {
    id: string;
    data: PinData;
    timestamp: number;
    isStreaming?: boolean;
}

const PinCollectionWindow: React.FC = () => {
    const { t } = useTranslation();
    const [pins, setPins] = useState<PinItem[]>([]);
    const [isPinned, setIsPinned] = useState(true);
    const [isCollapsed, setIsCollapsed] = useState(false);
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number, pin: PinItem } | null>(null);
    const [config, setConfig] = useState<any>(null);
    const [layoutMode, setLayoutMode] = useState<'small' | 'medium' | 'large'>('medium');
    const scrollRef = useRef<HTMLDivElement>(null);
    const lastSize = useRef({ width: 320, height: 600 });

    const fetchPins = async () => {
        try {
            const list: PinItem[] = await invoke('get_all_pins');
            list.sort((a, b) => b.timestamp - a.timestamp);
            setPins(list);
        } catch (e) {
            console.error("Failed to fetch pins:", e);
        }
    };

    useEffect(() => {
        fetchPins();
        invoke<any>('get_config').then(setConfig);
        invoke<boolean>('is_pin_always_on_top').then(setIsPinned);

        const unlistenUpdated = listen('pin-collection-updated', () => {
            fetchPins();
        });

        const unlistenChunk = listen<{ id: string, content: string }>('ai-stream://chunk', (event) => {
            setPins(prev => prev.map(p => {
                if (p.id === event.payload.id && p.data.type === 'Text') {
                    return {
                        ...p,
                        isStreaming: true,
                        data: { ...p.data, payload: p.data.payload + event.payload.content }
                    };
                }
                return p;
            }));
        });

        const unlistenError = listen<{ id: string, error: string }>('ai-stream://error', (event) => {
            setPins(prev => prev.map(p => {
                if (p.id === event.payload.id && p.data.type === 'Text') {
                    return {
                        ...p,
                        isStreaming: false,
                        data: { ...p.data, payload: p.data.payload + `\n\n**Error:** ${event.payload.error}` }
                    };
                }
                return p;
            }));
        });

        const unlistenDone = listen<{ id: string }>('ai-stream://done', (event) => {
            setPins(prev => prev.map(p => {
                if (p.id === event.payload.id) {
                    return { ...p, isStreaming: false };
                }
                return p;
            }));
        });

        const handleGlobalClick = () => setContextMenu(null);
        window.addEventListener('click', handleGlobalClick);

        return () => {
            unlistenUpdated.then(f => f());
            unlistenChunk.then(f => f());
            unlistenError.then(f => f());
            unlistenDone.then(f => f());
            window.removeEventListener('click', handleGlobalClick);
        };
    }, []);

    const closeWindow = async () => {
        await getCurrentWindow().close();
    };

    const clearAll = async () => {
        try {
            await invoke('clear_all_pins');
            fetchPins();
        } catch (e) {
            console.error(e);
        }
    };

    const savePinAs = async (pin: PinItem, ext?: 'png' | 'jpg') => {
        const defaultExt = ext || (config?.default_export_format as 'png' | 'jpg') || 'png';
        try {
            const path = await save({
                filters: [
                    { name: 'PNG Image', extensions: ['png'] },
                    { name: 'JPEG Image', extensions: ['jpg', 'jpeg'] },
                ],
                defaultPath: `capture_pin_${pin.id}.${defaultExt}`
            });

            if (path) {
                await invoke('save_pin_as', { id: pin.id, path });
            }
        } catch (e) {
            console.error("Failed to save pin:", e);
        }
        setContextMenu(null);
    };

    const handleContextMenu = (e: React.MouseEvent, pin: PinItem) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY, pin });
    };

    const togglePinned = async () => {
        try {
            const next = await invoke<boolean>('toggle_pin_always_on_top');
            setIsPinned(next);
        } catch (e) {
            console.error(e);
        }
    };

    const toggleCollapse = async () => {
        const next = !isCollapsed;
        setIsCollapsed(next);
        try {
            const window = getCurrentWindow();
            const physicalSize = await window.innerSize();
            const factor = await window.scaleFactor();
            const logicalWidth = physicalSize.width / factor;
            const logicalHeight = physicalSize.height / factor;

            if (next) {
                lastSize.current = { width: logicalWidth, height: logicalHeight };
                await invoke('set_pin_min_size', { width: 320, height: 30 });
                await invoke('set_pin_window_size', { width: logicalWidth, height: 30 });
            } else {
                await invoke('set_pin_min_size', { width: 320, height: 300 });
                await invoke('set_pin_window_size', { width: Math.max(logicalWidth, 640), height: Math.max(lastSize.current.height, 400) });
            }
        } catch (e) {
            console.error(e);
        }
    };

    const removePin = async (id: string) => {
        try {
            await invoke('remove_pin', { id });
            fetchPins();
        } catch (e) {
            console.error(e);
        }
    };

    const copyPinData = async (pin: PinItem) => {
        try {
            if (pin.data.type === 'Text') {
                await navigator.clipboard.writeText(pin.data.payload);
            } else if (pin.data.type === 'ImageBase64') {
                const res = await fetch(pin.data.payload);
                const blob = await res.blob();
                await navigator.clipboard.write([
                    new ClipboardItem({ [blob.type]: blob })
                ]);
            }
        } catch (err) {
            console.error("Failed to copy:", err);
        }
    };

    return (
        <div className="w-screen h-screen bg-black/80 backdrop-blur-md border border-white/10 flex flex-col overflow-hidden text-text-main select-none rounded-sm shadow-2xl relative">
            {/* Custom Titlebar / Drag Region */}
            <div
                data-tauri-drag-region
                className="h-10 w-full flex items-center justify-between px-3 bg-black/40 border-b border-white/5 shrink-0"
            >
                <div data-tauri-drag-region className="flex items-center gap-2.5 text-text-muted">
                    <div className="p-1 rounded-sm bg-accent/10 border border-accent/20 text-accent">
                        <Cpu className="w-3.5 h-3.5" />
                    </div>
                    <span data-tauri-drag-region className="text-[10px] font-bold tech-text uppercase tracking-[0.2em] text-accent opacity-80">{t('pin.title')}</span>
                    <span className="text-[9px] tech-text bg-white/5 border border-white/10 px-1.5 py-0.5 rounded-sm opacity-40 uppercase tracking-tighter">{t('pin.nodes')}: {pins.length}</span>
                    
                    <div className="flex items-center gap-0.5 bg-black/40 p-0.5 rounded-sm border border-white/5 ml-2">
                        <button
                            onClick={() => setLayoutMode('small')}
                            className={`p-1 rounded-[2px] transition-all ${layoutMode === 'small' ? 'bg-accent/20 text-accent' : 'text-text-muted hover:text-text-main'}`}
                            title={t('pin.layout_small')}
                        >
                            <LayoutGrid size={10} />
                        </button>
                        <button
                            onClick={() => setLayoutMode('medium')}
                            className={`p-1 rounded-[2px] transition-all ${layoutMode === 'medium' ? 'bg-accent/20 text-accent' : 'text-text-muted hover:text-text-main'}`}
                            title={t('pin.layout_medium')}
                        >
                            <Grid2X2 size={10} />
                        </button>
                        <button
                            onClick={() => setLayoutMode('large')}
                            className={`p-1 rounded-[2px] transition-all ${layoutMode === 'large' ? 'bg-accent/20 text-accent' : 'text-text-muted hover:text-text-main'}`}
                            title={t('pin.layout_large')}
                        >
                            <StretchHorizontal size={10} />
                        </button>
                    </div>
                </div>

                <div className="flex items-center gap-1.5 z-10">
                    <button
                        onClick={togglePinned}
                        className={`p-1.5 flex items-center justify-center rounded-sm transition-all border ${isPinned ? 'text-accent bg-accent/10 border-accent/20' : 'text-text-muted hover:text-text-main hover:bg-white/5 border-transparent'}`}
                        title={isPinned ? t('pin.tooltip_pin_on') : t('pin.tooltip_pin_off')}
                    >
                        {isPinned ? <Lock size={12} className="stroke-[2.5px]" /> : <Unlock size={12} />}
                    </button>
                    <button
                        onClick={toggleCollapse}
                        className="p-1.5 flex items-center justify-center rounded-sm text-text-muted hover:text-text-main hover:bg-white/5 transition-all"
                        title={isCollapsed ? t('pin.tooltip_expand') : t('pin.tooltip_collapse')}
                    >
                        {isCollapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
                    </button>
                    <button
                        onClick={clearAll}
                        className="p-1.5 flex items-center justify-center rounded-sm text-text-muted hover:text-red-500 hover:bg-red-500/5 transition-all"
                        title={t('pin.tooltip_purge')}
                    >
                        <Trash2 size={13} />
                    </button>
                    <button
                        onClick={closeWindow}
                        className="p-1.5 flex items-center justify-center rounded-sm text-text-muted hover:text-text-main hover:bg-white/5 transition-all"
                        title={t('pin.tooltip_close')}
                    >
                        <X size={14} />
                    </button>
                </div>
            </div>

            {/* Masonry or Grid Layout */}
            <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 bg-black/20 custom-scrollbar">
                <div className={`grid gap-3 ${
                    layoutMode === 'small' ? 'grid-cols-3' : 
                    layoutMode === 'medium' ? 'grid-cols-2' : 
                    'grid-cols-1'
                }`}>
                    <AnimatePresence>
                    {pins.length === 0 && (
                        <div className="h-full w-full flex flex-col items-center justify-center text-text-muted/20 gap-3 opacity-40">
                            <Terminal size={40} className="stroke-[1px]" />
                            <p className="text-[10px] tech-text uppercase tracking-widest font-bold">{t('pin.no_active_data')}</p>
                        </div>
                    )}
                    {pins.map(pin => (
                        <motion.div
                            key={pin.id}
                            initial={{ opacity: 0, scale: 0.98 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0 }}
                            className="w-full bg-black/30 border border-white/5 rounded-sm p-3.5 flex flex-col gap-3 relative group shadow-lg hover:border-white/10 transition-all"
                        >
                            <div className="absolute top-2.5 right-2.5 flex gap-1.5 z-10">
                                {pin.isStreaming && (
                                    <div className="flex items-center gap-1.5 px-2 py-1 bg-accent/10 border border-accent/20 text-accent rounded-sm text-[8px] uppercase font-bold tech-text tracking-widest animate-pulse">
                                        <Loader2 size={10} className="animate-spin" />
                                        {t('pin.streaming_data')}
                                    </div>
                                )}
                                <div className="opacity-0 group-hover:opacity-100 flex gap-1 transition-all">
                                    {pin.data.type === 'ImageBase64' && (
                                        <button
                                            onClick={() => savePinAs(pin)}
                                            className="p-1.5 bg-black/60 border border-white/10 rounded-sm text-text-muted hover:text-text-main transition-all"
                                            title={t('pin.tooltip_export')}
                                        >
                                            <Download size={12} />
                                        </button>
                                    )}
                                    <button
                                        onClick={() => copyPinData(pin)}
                                        className="p-1.5 bg-black/60 border border-white/10 rounded-sm text-text-muted hover:text-text-main transition-all"
                                        title={t('pin.tooltip_copy')}
                                    >
                                        <Copy size={12} />
                                    </button>
                                    <button
                                        onClick={() => removePin(pin.id)}
                                        className="p-1.5 bg-black/60 border border-red-500/20 rounded-sm text-text-muted hover:text-red-500 transition-all"
                                        title={t('pin.delete_segment')}
                                    >
                                        <X size={12} />
                                    </button>
                                </div>
                            </div>

                            {pin.data.type === 'ImageBase64' ? (
                                <div className="rounded-sm overflow-hidden border border-white/5 bg-black/40">
                                    <img
                                        src={pin.data.payload}
                                        className="w-full h-auto object-contain max-h-[500px] cursor-context-menu"
                                        alt="Pin Asset"
                                        onContextMenu={(e) => handleContextMenu(e, pin)}
                                    />
                                </div>
                            ) : (
                                <div className="text-[12px] font-bold tech-text text-text-main/80 prose prose-invert prose-sm max-w-none pt-1 uppercase tracking-tight leading-tight">
                                    <ReactMarkdown>{pin.data.payload}</ReactMarkdown>
                                </div>
                            )}
                            
                            <div className="flex justify-between items-center mt-1 pt-2 border-t border-white/5 opacity-20">
                                <span className="text-[8px] tech-text font-bold">{t('pin.node_id')}: {pin.id.split('-')[0].toUpperCase()}</span>
                                <span className="text-[8px] tech-text">{new Date(pin.timestamp).toISOString().split('T')[1].split('.')[0]}</span>
                            </div>
                        </motion.div>
                    ))}
                    </AnimatePresence>
                </div>
            </div>
            
            <div className="h-1.5 bg-accent/5 w-full shrink-0 border-t border-white/5" />

            {/* Custom Context Menu */}
            <AnimatePresence>
                {contextMenu && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.95 }}
                        className="fixed z-50 bg-bg-sidebar/95 backdrop-blur-xl border border-white/10 rounded-sm shadow-2xl p-1.5 min-w-[160px] flex flex-col gap-1"
                        style={{ left: Math.min(contextMenu.x, window.innerWidth - 170), top: Math.min(contextMenu.y, window.innerHeight - 150) }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        <ContextItem
                            icon={<Download size={12} />}
                            label={t('pin.export_png')}
                            onClick={() => savePinAs(contextMenu.pin, 'png')}
                        />
                        <ContextItem
                            icon={<Download size={12} />}
                            label={t('pin.export_jpg')}
                            onClick={() => savePinAs(contextMenu.pin, 'jpg')}
                        />
                        <div className="h-[1px] bg-white/5 my-0.5 mx-1" />
                        <ContextItem
                            icon={<Copy size={12} />}
                            label={t('pin.copy_to_clipboard')}
                            onClick={() => {
                                copyPinData(contextMenu.pin);
                                setContextMenu(null);
                            }}
                        />
                        <ContextItem
                            icon={<Trash2 size={12} />}
                            label={t('pin.delete_segment')}
                            variant="danger"
                            onClick={() => {
                                removePin(contextMenu.pin.id);
                                setContextMenu(null);
                            }}
                        />
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
};

const ContextItem: React.FC<{ icon: React.ReactNode, label: string, onClick: () => void, variant?: 'default' | 'danger' }> = ({ icon, label, onClick, variant = 'default' }) => (
    <button
        onClick={onClick}
        className={`flex items-center gap-3 px-3 py-2 rounded-sm text-[9px] tech-text font-bold transition-all uppercase tracking-widest ${variant === 'danger'
            ? 'text-red-400 hover:bg-red-500/10'
            : 'text-text-muted hover:text-text-main hover:bg-white/5'
            }`}
    >
        <span className="opacity-60">{icon}</span>
        {label}
    </button>
);

export default PinCollectionWindow;
