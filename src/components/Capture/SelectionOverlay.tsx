import { useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export default function SelectionOverlay() {
    const [isReady, setIsReady] = useState(false);
    const [selection, setSelection] = useState<Rect | null>(null);
    const [isDragging, setIsDragging] = useState(false);
    const startPos = useRef({ x: 0, y: 0 });

    useEffect(() => {
        console.log('SelectionOverlay mounted, setting up listener...');

        // Listen for capture completion signal (native-mode)
        const unlisten = listen<string>('capture-ready', (event) => {
            console.log('capture-ready event received:', event.payload);
            setIsReady(true);
        });

        // Polling Fallback (in case event is missed)
        const pollInterval = setInterval(async () => {
            try {
                const ready = await invoke<boolean>('check_capture_status');
                if (ready) {
                    console.log('Poll: Capture Ready!');
                    setIsReady(true);
                    clearInterval(pollInterval);
                }
            } catch (e) {
                console.error('Poll failed:', e);
            }
        }, 100);

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') handleCancel();
            else if (e.key === 'Enter') handleSave().catch(console.error);
        };

        window.addEventListener('keydown', handleKeyDown);

        return () => {
            console.log('SelectionOverlay unmounting...');
            window.removeEventListener('keydown', handleKeyDown);
            unlisten.then(f => f());
        };
    }, []);

    const handlePointerDown = (e: React.PointerEvent) => {
        if (!isReady) return;
        if (e.button !== 0) return;
        e.preventDefault();
        (e.target as Element).setPointerCapture(e.pointerId);
        setIsDragging(true);
        startPos.current = { x: e.clientX, y: e.clientY };
        setSelection({ x: e.clientX, y: e.clientY, width: 0, height: 0 });
    };

    const handlePointerMove = (e: React.PointerEvent) => {
        if (!isDragging) return;
        const x = Math.min(startPos.current.x, e.clientX);
        const y = Math.min(startPos.current.y, e.clientY);
        const width = Math.abs(e.clientX - startPos.current.x);
        const height = Math.abs(e.clientY - startPos.current.y);
        setSelection({ x, y, width, height });
    };

    const handlePointerUp = (e: React.PointerEvent) => {
        (e.target as Element).releasePointerCapture(e.pointerId);
        setIsDragging(false);
    }

    const handleCancel = async () => {
        setSelection(null);
        setIsReady(false); // Reset readiness
        await invoke('hide_overlay');
    };

    const handleSave = async () => {
        try {
            const crop = selection && selection.width > 5
                ? [Math.round(selection.x), Math.round(selection.y), Math.round(selection.width), Math.round(selection.height)]
                : null;
            await invoke('save_capture', { path: null, crop });
            await invoke('hide_overlay');
            setSelection(null);
            setIsReady(false);
        } catch (e) {
            console.error('Save failed:', e);
        }
    };

    // Debug Mode: Always render to show status
    // if (!isReady) return null; 
    // Or return transparent div? But null is safer to avoid blocking events if any.
    // Actually we want to catch events. But without 'isReady', we shouldn't show dimming?
    // Wait, if !isReady, we shouldn't show dimming because Native Overlay might not be ready (though current logic says capture-ready AFTER native show).
    // So !isReady -> empty.

    // Calculate Dimming Rects
    // If no selection, full screen dim.
    // If selection, 4 rects.

    return (
        <div
            className="fixed inset-0 w-full h-full cursor-crosshair overflow-hidden select-none"
            style={{ background: 'transparent', touchAction: 'none' }}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
        >
            {/* DEBUG STATUS */}
            <div
                className="absolute top-10 left-1/2 -translate-x-1/2 z-[9999] pointer-events-none"
                style={{
                    backgroundColor: 'red',
                    color: 'white',
                    padding: '8px 16px',
                    borderRadius: '4px',
                    fontWeight: 'bold',
                    boxShadow: '0 4px 6px rgba(0,0,0,0.3)'
                }}
            >
                [DEBUG] Ready: {isReady.toString()} | Selection: {selection ? 'Active' : 'None'}
            </div>

            {/* Mask via Box Shadow Strategy */}
            {!selection ? (
                // Full screen dim if no selection
                <div className="absolute inset-0 pointer-events-none" style={{ background: 'rgba(0,0,0,0.4)' }} />
            ) : (
                // If selection exists, the selection box ITSELF casts a massive shadow to create the mask
                <div
                    className="absolute border-2 border-blue-500 pointer-events-none"
                    style={{
                        left: selection.x,
                        top: selection.y,
                        width: selection.width,
                        height: selection.height,
                        zIndex: 10
                    }}
                >
                    {/* Size indicator */}
                    {selection.width > 20 && (
                        <div className="absolute -top-6 left-0 bg-blue-600 text-white text-[10px] px-1.5 py-0.5 rounded font-bold">
                            {Math.round(selection.width)} × {Math.round(selection.height)}
                        </div>
                    )}

                    {/* Toolbar */}
                    {!isDragging && selection.width > 30 && (
                        <div className="absolute -bottom-10 right-0 flex gap-1 bg-slate-900/95 p-1 rounded border border-white/10 pointer-events-auto">
                            <button onClick={(e) => { e.stopPropagation(); handleCancel(); }} className="p-1 hover:bg-white/10 rounded text-white/60">✕</button>
                            <button onClick={(e) => { e.stopPropagation(); handleSave(); }} className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1 rounded text-[11px] font-bold">✔ OK</button>
                        </div>
                    )}
                </div>
            )}

            {/* Hint */}
            {!selection && (
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none">
                    <div className="px-6 py-3 bg-black/70 rounded-full border border-white/10 text-white text-sm font-medium">
                        Drag to Select • ESC to Cancel
                    </div>
                </div>
            )}
        </div>
    );
}