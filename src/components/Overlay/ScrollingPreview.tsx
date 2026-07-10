import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { X, Save, Copy, Download, ZoomIn, ZoomOut } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { save } from '@tauri-apps/plugin-dialog';

type ScrollingResult = {
    path: string;
    width: number;
    height: number;
    reason: string;
    errorMessage?: string | null;
    capturedFrames: number;
    appendedFrames: number;
    stagnantFrames: number;
    totalAddedPx: number;
    durationMs: number;
    maxCaptureFrames: number;
    noGrowthLimit: number;
    wheelStep: number;
    stitchReports?: {
        addedPx: number;
        matchY?: number | null;
        seededMatchY?: number | null;
        templateHeight: number;
        candidateCount?: number;
        matchDurationMs?: number;
        bestDiffPerPixel?: number | null;
        secondBestDiffPerPixel?: number | null;
        rejectedReason?: string | null;
    }[];
    targetWindow?: {
        hwnd: number;
        processId: number;
        className: string;
        title: string;
    } | null;
};

const ScrollingPreview: React.FC = () => {
    const { t } = useTranslation();
    const [imagePath, setImagePath] = useState<string | null>(null);
    const [imageSrc, setImageSrc] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [zoom, setZoom] = useState(100);
    const [dimensions, setDimensions] = useState<{ width: number; height: number } | null>(null);
    const [result, setResult] = useState<ScrollingResult | null>(null);
    const stitchReports = result?.stitchReports ?? [];
    const appendedReports = stitchReports.filter((item) => item.addedPx > 0);
    const lastStitchReport = stitchReports[stitchReports.length - 1];
    const averageBestDiff = appendedReports.length > 0
        ? Math.round(appendedReports.reduce((sum, item) => sum + (item.bestDiffPerPixel ?? 0), 0) / appendedReports.length)
        : null;
    const averageMatchMs = stitchReports.length > 0
        ? Math.round(stitchReports.reduce((sum, item) => sum + (item.matchDurationMs ?? 0), 0) / stitchReports.length)
        : null;
    const seededReports = stitchReports.filter((item) => item.seededMatchY != null).length;

    useEffect(() => {
        const load = async () => {
            try {
                const metadata = await invoke<ScrollingResult | null>('get_last_scrolling_result');
                if (metadata) {
                    setResult(metadata);
                    setImagePath(metadata.path);
                    setImageSrc(convertFileSrc(metadata.path));
                    setDimensions({ width: metadata.width, height: metadata.height });
                    return;
                }
                const path = await invoke<string | null>('get_last_scrolled_image');
                if (path) {
                    setImagePath(path);
                    setImageSrc(convertFileSrc(path));
                }
            } catch (e) {
                console.error("Failed to load scrolled image", e);
            }
        };
        load();
    }, []);

    const close = async () => {
        await getCurrentWindow().close();
    };

    const handleSave = async () => {
        if (!imagePath) return;
        setIsSaving(true);
        try {
            const dest = await save({
                filters: [{ name: 'Images', extensions: ['png', 'jpg'] }],
                defaultPath: `scrolling_capture_${Date.now()}.png`
            });
            if (dest) {
                await invoke('save_scrolled_image_to', { source: imagePath, destination: dest });
            }
        } catch (e) {
            console.error("Save failed", e);
        } finally {
            setIsSaving(false);
        }
    };

    const handleCopy = async () => {
        if (!imagePath) return;
        try {
            await invoke('copy_image_to_clipboard', { path: imagePath });
        } catch (e) {
            console.error("Copy failed", e);
        }
    };

    return (
        <div className="float-win w-full h-full flex flex-col p-5 gap-4 select-none relative overflow-hidden">
            {/* Header */}
            <div data-tauri-drag-region className="flex items-center justify-between shrink-0">
                <div data-tauri-drag-region className="flex flex-col gap-0.5 pointer-events-none">
                    <h2 className="text-[14px] font-extrabold text-ink tracking-[-0.01em]">
                        {t('scrolling_preview.title')}
                    </h2>
                    <span className="mono text-[10.5px] text-muted">
                        {dimensions
                            ? `${dimensions.width} x ${dimensions.height} px`
                            : t('scrolling_preview.subtitle')}
                    </span>
                </div>
                <button
                    onClick={close}
                    className="w-[30px] h-[30px] flex items-center justify-center rounded-btn text-muted hover:text-ink hover:bg-bg-2 transition-colors"
                >
                    <X size={18} />
                </button>
            </div>

            {/* Content Area */}
            <div className="flex-1 min-h-0 bg-bg-0/40 border border-line rounded-panel overflow-hidden relative">
                <div className="absolute top-0 left-0 right-0 z-10 px-4 py-2 bg-bg-1/92 border-b border-line">
                    <p className="text-[11px] leading-4 text-muted">
                        {t('scrolling_preview.how_it_works')}
                    </p>
                    {result && (
                        <div className="mono mt-1 text-[10.5px] leading-4 text-ink">
                            <p>
                                {t('scrolling_preview.result_stats', {
                                    reason: t(`scrolling_preview.finish_reason.${result.reason}`, { defaultValue: t('scrolling_preview.finish') }),
                                    captured: result.capturedFrames,
                                    appended: result.appendedFrames,
                                    added: result.totalAddedPx,
                                    duration: Math.round(result.durationMs / 100) / 10,
                                })}
                            </p>
                            <p className="text-muted">
                                {t('scrolling_preview.logic_stats', {
                                    stagnant: result.stagnantFrames,
                                    limit: result.noGrowthLimit,
                                    max: result.maxCaptureFrames,
                                    wheel: result.wheelStep,
                                })}
                            </p>
                            {result.errorMessage && (
                                <p className="text-bad">
                                    {t('scrolling_preview.error_message', {
                                        message: result.errorMessage,
                                    })}
                                </p>
                            )}
                            {stitchReports.length > 0 && (
                                <>
                                    <p className="text-muted">
                                        {t('scrolling_preview.stitch_quality', {
                                            reports: stitchReports.length,
                                            appended: appendedReports.length,
                                            best: averageBestDiff ?? '-',
                                            last: lastStitchReport?.rejectedReason ?? t('scrolling_preview.stitch_ok'),
                                        })}
                                    </p>
                                    <p className="text-muted">
                                        {t('scrolling_preview.stitch_perf', {
                                            avgMs: averageMatchMs ?? '-',
                                            candidates: lastStitchReport?.candidateCount ?? '-',
                                            seeded: seededReports,
                                            seedY: lastStitchReport?.seededMatchY ?? '-',
                                        })}
                                    </p>
                                </>
                            )}
                            {result.targetWindow && (
                                <p className="truncate text-muted">
                                    {t('scrolling_preview.target_window', {
                                        title: result.targetWindow.title || t('scrolling_preview.untitled_window'),
                                        className: result.targetWindow.className || t('scrolling_preview.unknown_window_class'),
                                        pid: result.targetWindow.processId,
                                    })}
                                </p>
                            )}
                        </div>
                    )}
                </div>
                <div className="absolute inset-0 overflow-y-auto custom-scrollbar p-4 flex justify-center">
                    {imageSrc ? (
                        <motion.img
                            initial={{ y: 7 }}
                            animate={{ y: 0 }}
                            src={imageSrc}
                            className="h-auto rounded shadow-sm border border-line origin-top mt-16"
                            style={{ width: `${zoom}%`, maxWidth: zoom <= 100 ? '100%' : 'none' }}
                            onLoad={(event) => {
                                const img = event.currentTarget;
                                setDimensions({ width: img.naturalWidth, height: img.naturalHeight });
                            }}
                        />
                    ) : (
                        <div className="flex flex-col items-center justify-center h-full gap-3 text-muted pt-8">
                            <motion.div animate={{ rotate: 360 }} transition={{ repeat: Infinity, duration: 2, ease: "linear" }}>
                                <Download size={30} className="text-faint" />
                            </motion.div>
                            <span className="text-[12px] font-semibold">{t('scrolling_preview.compiling')}</span>
                        </div>
                    )}
                </div>
            </div>

            {/* Footer Actions */}
            <div className="flex items-center justify-between gap-2 shrink-0">
                <div className="flex items-center gap-1.5">
                    <IconBtn
                        icon={<ZoomOut size={14} />}
                        onClick={() => setZoom((z) => Math.max(35, z - 15))}
                        disabled={!imageSrc || zoom <= 35}
                    />
                    <span className="mono text-[11px] text-muted w-12 text-center">{zoom}%</span>
                    <IconBtn
                        icon={<ZoomIn size={14} />}
                        onClick={() => setZoom((z) => Math.min(200, z + 15))}
                        disabled={!imageSrc || zoom >= 200}
                    />
                </div>
                <div className="flex items-center justify-end gap-2">
                    <ActionBtn icon={<Copy size={15} />} label={t('scrolling_preview.copy')} onClick={handleCopy} disabled={!imagePath} />
                    <ActionBtn
                        icon={<Save size={15} />}
                        label={isSaving ? t('scrolling_preview.saving') : t('scrolling_preview.save_as')}
                        onClick={handleSave}
                        variant="primary"
                        disabled={!imagePath || isSaving}
                    />
                </div>
            </div>
        </div>
    );
};

const IconBtn: React.FC<{ icon: React.ReactNode, onClick: () => void, disabled?: boolean }> = ({ icon, onClick, disabled = false }) => (
    <button
        onClick={onClick}
        disabled={disabled}
        className="w-8 h-8 flex items-center justify-center rounded-btn bg-bg-2 text-muted border border-line hover:text-ink hover:border-line-2 disabled:opacity-40 disabled:pointer-events-none transition-colors"
    >
        {icon}
    </button>
);

const ActionBtn: React.FC<{ icon: React.ReactNode, label: string, onClick: () => void, variant?: 'default' | 'primary', disabled?: boolean }> = ({ icon, label, onClick, variant = 'default', disabled = false }) => (
    <button
        onClick={onClick}
        disabled={disabled}
        className={`flex items-center gap-1.5 px-3.5 py-2 rounded-btn text-[12.5px] font-semibold transition-colors ${
            variant === 'primary'
            ? 'bg-accent text-on-accent hover:bg-[var(--accent-press)]'
            : 'bg-bg-2 text-ink border border-line hover:border-line-2'
        } disabled:opacity-45 disabled:pointer-events-none`}
    >
        {icon}
        {label}
    </button>
);

export default ScrollingPreview;
