import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { X, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';

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
}

const OCRResultWindow: React.FC = () => {
    const { t } = useTranslation();
    const [data, setData] = useState<OcrResultData | null>(null);

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

    const copyAll = async () => {
        if (!data) return;
        await navigator.clipboard.writeText(data.full_text);
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
                    <button onClick={copyAll} className="float-win px-2.5 py-1.5 rounded-btn text-[11.5px] font-semibold hover:text-accent transition-colors text-ink">
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
                        data.lines.length > 0 ? (
                            <motion.div 
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                className="w-full h-full relative"
                            >
                                {data.lines.flatMap(line => line.words).map((word, idx) => (
                                    <span 
                                        key={idx}
                                        style={{
                                            position: 'absolute',
                                            left: `${word.x}px`,
                                            top: `${word.y}px`,
                                            width: `${word.width}px`,
                                            height: `${word.height}px`,
                                            fontSize: `${word.height * 0.8}px`,
                                            lineHeight: `${word.height}px`
                                        }}
                                        className="inline-flex items-center justify-center bg-accent/5 hover:bg-accent/40 border border-transparent hover:border-accent/50 rounded-[1px] cursor-text transition-all text-ink/0 hover:text-ink leading-none whitespace-nowrap overflow-hidden"
                                        title={word.text}
                                    >
                                        {word.text}
                                    </span>
                                ))}
                            </motion.div>
                        ) : (
                            <div className="w-full h-full flex items-center justify-center">
                                <span className="text-[12px] font-semibold text-muted">
                                    {t('ocr_result.no_text_found')}
                                </span>
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
            
            {/* Branding Indicator (Subtle) */}
            <div className="absolute bottom-1 right-2 opacity-0 group-hover:opacity-20 transition-opacity pointer-events-none">
                <span className="mono text-[9px] text-muted">
                    {t('ocr_result.engine_info')}
                </span>
            </div>
        </div>
    );
};

export default OCRResultWindow;
