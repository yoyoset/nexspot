import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { X, Check, Copy, FileText, Info } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { save } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../../store/useAppStore';

interface OcrWord {
    text: string;
    x: number;
    y: number;
    width: number;
    height: number;
}

interface OcrLine {
    text: string;
    words: OcrWord[];
}

interface OcrResultData {
    lines: OcrLine[];
    full_text: string;
    engine?: string;
    status?: string;
    message?: string;
    metadata?: {
        captureSource: string;
        overlayHidden: boolean;
        overlayHideConfirmed: boolean;
        overlayWindowCount: number;
        overlayVisibleAfterHide: number;
        overlayHideWaitMs: number;
        captureEngine: string;
        physicalWidth: number;
        physicalHeight: number;
        logicalWidth: number;
        logicalHeight: number;
        scaleFactor: number;
        durationMs: number;
    };
}

const OCRResultWindow: React.FC = () => {
    const { t } = useTranslation();
    const { showHUD } = useAppStore();
    const [data, setData] = useState<OcrResultData | null>(null);
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        const unlistenPromise = (async () => {
            try {
                // 1. Try to fetch existing result immediately (fallback for race conditions)
                const existing = await invoke<OcrResultData | null>('get_last_ocr_result');
                if (existing) {
                    setData(existing);
                }

                // 2. Listen for future updates
                const { listen } = await import('@tauri-apps/api/event');
                return await listen<OcrResultData>('ocr://data', (event) => {
                    setData(event.payload);
                });
            } catch (e) {
                console.error("Failed to load OCR data", e);
                return () => {};
            }
        })();

        return () => {
            unlistenPromise.then(unlisten => unlisten()).catch(err => console.error("Failed to unlisten OCR data", err));
        };
    }, []);

    const close = async () => {
        await getCurrentWindow().close();
    };

    const hasText = !!data?.full_text.trim();
    const isError = data?.status === 'error';
    const wordCount = data?.lines.reduce((sum, line) => sum + line.words.length, 0) ?? 0;
    const charCount = data?.full_text.trim().length ?? 0;
    const lineCount = data?.lines.length ?? 0;
    const metadata = data?.metadata;
    const overlayRisk = !!metadata && metadata.overlayVisibleAfterHide > 0;

    const copyAll = async () => {
        if (!data || !hasText) return;
        try {
            await navigator.clipboard.writeText(data.full_text);
            showHUD(t('hud.copied'), 'copy');
        } catch (e) {
            console.error("Failed to copy OCR text", e);
            showHUD(t('ocr_result.copy_failed'), 'error');
        }
    };

    const exportText = async () => {
        if (!data || !hasText) return;
        setIsSaving(true);
        try {
            const dest = await save({
                filters: [{ name: 'Text', extensions: ['txt'] }],
                defaultPath: `ocr_result_${Date.now()}.txt`
            });
            if (dest) {
                await invoke('save_ocr_text_to', { destination: dest, text: data.full_text });
                showHUD(t('ocr_result.exported'), 'save');
            }
        } catch (e) {
            console.error("Failed to export OCR text", e);
            showHUD(t('ocr_result.export_failed'), 'error');
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div className="w-full h-full bg-bg-0/5 backdrop-blur-sm border border-line flex flex-col select-text relative overflow-hidden group rounded-lg">
            {/* Minimalist Header on Hover */}
            <div className="absolute top-2 left-2 right-2 flex items-center justify-between z-50 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-btn pointer-events-auto float-win">
                    <span className="w-1.5 h-1.5 rounded-full bg-accent animate-pulse" />
                    <span className="text-[12px] font-semibold text-ink">
                        {t('ocr_result.title')}
                    </span>
                </div>
                <div className="flex items-center gap-1.5 pointer-events-auto">
                    <button
                        onClick={exportText}
                        disabled={!data || !hasText || isSaving}
                        className="float-win px-2.5 py-1.5 rounded-btn text-[11.5px] font-semibold hover:text-accent transition-colors text-ink disabled:opacity-45 disabled:pointer-events-none inline-flex items-center gap-1.5"
                    >
                        <FileText size={13} />
                        {isSaving ? t('ocr_result.exporting') : t('ocr_result.export_txt')}
                    </button>
                    <button
                        onClick={copyAll}
                        disabled={!data || !hasText}
                        className="float-win px-2.5 py-1.5 rounded-btn text-[11.5px] font-semibold hover:text-accent transition-colors text-ink disabled:opacity-45 disabled:pointer-events-none inline-flex items-center gap-1.5"
                    >
                        <Copy size={13} />
                        {t('ocr_result.copy_all')}
                    </button>
                    <button onClick={close} className="float-win w-[28px] h-[28px] flex items-center justify-center rounded-btn text-muted hover:text-ink transition-colors">
                        <X size={14} />
                    </button>
                </div>
            </div>

            {/* Selectable Words Area (Absolute Positioned) */}
            <div className="flex-1 relative overflow-hidden">
                <AnimatePresence>
                    {data ? (
                        hasText && data.lines.length > 0 ? (
                            <motion.div
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                className="w-full h-full relative"
                            >
                                {(() => {
                                    const words = data.lines.flatMap(line => line.words);
                                    // 检测框高度天然抖动（±30%），逐词取高会大字小字乱跳。
                                    // 取中位高为正文基准统一字号；仅明显高于基准的(真标题)保留大字。
                                    const hs = words.map(w => w.height).sort((a, b) => a - b);
                                    const med = hs.length ? hs[Math.floor(hs.length / 2)] : 14;
                                    const fontFor = (h: number) => (h > med * 1.5 ? h * 0.68 : Math.min(h, med) * 0.68);
                                    return words.map((word, idx) => (
                                        <span
                                            key={idx}
                                            style={{
                                                position: 'absolute',
                                                left: `${word.x}px`,
                                                top: `${word.y}px`,
                                                minWidth: `${word.width}px`,
                                                height: `${word.height}px`,
                                                fontSize: `${fontFor(word.height)}px`,
                                                lineHeight: `${word.height}px`,
                                            }}
                                            className="inline-flex items-center px-1 bg-[rgba(12,13,16,0.88)] text-ink border border-line hover:bg-accent/30 hover:border-accent/60 rounded-[3px] cursor-text transition-colors leading-none whitespace-nowrap"
                                            title={word.text}
                                        >
                                            {word.text}
                                        </span>
                                    ));
                                })()}
                            </motion.div>
                        ) : (
                            <div className="w-full h-full flex items-center justify-center px-8">
                                <div className="float-win max-w-[320px] rounded-panel p-4 flex flex-col gap-2 text-center">
                                    <div className="mx-auto w-8 h-8 rounded-full bg-bg-2 border border-line flex items-center justify-center text-muted">
                                        <Info size={16} />
                                    </div>
                                    <span className="text-[13px] font-bold text-ink">
                                        {isError ? t('ocr_result.error_found') : t('ocr_result.no_text_found')}
                                    </span>
                                    <span className="text-[11.5px] leading-5 text-muted">
                                        {isError ? t('ocr_result.error_hint') : t('ocr_result.no_text_hint')}
                                    </span>
                                    {isError && data.message && (
                                        <span className="mono text-[10px] leading-4 text-muted break-words max-h-20 overflow-y-auto custom-scrollbar">
                                            {data.message}
                                        </span>
                                    )}
                                    {!isError && data.message && (
                                        <span className="mono text-[10px] leading-4 text-muted">
                                            {data.message}
                                        </span>
                                    )}
                                </div>
                            </div>
                        )
                    ) : (
                        <div className="w-full h-full flex items-center justify-center">
                            <div className="flex flex-col items-center gap-2 opacity-40">
                                <motion.div animate={{ scale: [1, 1.1, 1] }} transition={{ repeat: Infinity, duration: 1.5 }}>
                                    <Check size={24} className="text-accent" />
                                </motion.div>
                                <span className="text-[12px] font-semibold text-muted">
                                    {t('ocr_result.analyzing')}
                                </span>
                            </div>
                        </div>
                    )}
                </AnimatePresence>
            </div>
            
            {data && (
                <div className="absolute bottom-2 left-2 right-2 flex items-center justify-between gap-2 opacity-80 group-hover:opacity-100 transition-opacity pointer-events-none">
                    <div className="float-win px-2 py-1 rounded-btn flex items-center gap-1.5 min-w-0">
                        <Info size={12} className="text-muted shrink-0" />
                        <span className="mono text-[9.5px] text-muted truncate">
                            {data.engine || t('ocr_result.engine_unknown')}
                        </span>
                    </div>
                    {metadata && (
                        <div className={`float-win px-2 py-1 rounded-btn flex flex-col gap-0.5 min-w-0 ${overlayRisk ? 'border border-bad/45 bg-bad-soft' : ''}`}>
                            {overlayRisk && (
                                <span className="mono text-[9.5px] text-bad truncate">
                                    {t('ocr_result.overlay_warning', {
                                        visible: metadata.overlayVisibleAfterHide,
                                    })}
                                </span>
                            )}
                            <span className="mono text-[9.5px] text-muted truncate">
                                {t('ocr_result.capture_meta', {
                                    source: t(`ocr_result.capture_source.${metadata.captureSource}`, { defaultValue: metadata.captureSource }),
                                    engine: metadata.captureEngine,
                                    width: metadata.physicalWidth,
                                    height: metadata.physicalHeight,
                                    scale: Number(metadata.scaleFactor).toFixed(2),
                                    duration: Math.round(metadata.durationMs),
                                })}
                            </span>
                            <span className="mono text-[9.5px] text-muted truncate">
                                {t('ocr_result.capture_overlay_meta', {
                                    hidden: metadata.overlayHideConfirmed ? metadata.overlayWindowCount : Math.max(0, metadata.overlayWindowCount - metadata.overlayVisibleAfterHide),
                                    count: metadata.overlayWindowCount,
                                    wait: Math.round(metadata.overlayHideWaitMs),
                                })}
                            </span>
                        </div>
                    )}
                    <div className="float-win px-2 py-1 rounded-btn flex items-center gap-2">
                        {hasText ? (
                            <>
                                <span className="mono text-[9.5px] text-muted">
                                    {t('ocr_result.stats', { chars: charCount, lines: lineCount, words: wordCount })}
                                </span>
                                <span className="mono text-[9.5px] text-muted">
                                    {t('ocr_result.confidence_unavailable')}
                                </span>
                            </>
                        ) : (
                            <span className="mono text-[9.5px] text-muted">
                                {data.message || t('ocr_result.no_text_found')}
                            </span>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};

export default OCRResultWindow;
