import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { X, Save, Copy, Download } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { save } from '@tauri-apps/plugin-dialog';

const ScrollingPreview: React.FC = () => {
    const { t } = useTranslation();
    const [imagePath, setImagePath] = useState<string | null>(null);
    const [imageSrc, setImageSrc] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        const load = async () => {
            try {
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
                        {t('scrolling_preview.subtitle')}
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
                <div className="absolute inset-0 overflow-y-auto custom-scrollbar p-4 flex justify-center">
                    {imageSrc ? (
                        <motion.img
                            initial={{ y: 7 }}
                            animate={{ y: 0 }}
                            src={imageSrc}
                            className="max-w-full h-auto rounded shadow-sm border border-line"
                        />
                    ) : (
                        <div className="flex flex-col items-center justify-center h-full gap-3 text-muted">
                            <motion.div animate={{ rotate: 360 }} transition={{ repeat: Infinity, duration: 2, ease: "linear" }}>
                                <Download size={30} className="text-faint" />
                            </motion.div>
                            <span className="text-[12px] font-semibold">{t('scrolling_preview.compiling')}</span>
                        </div>
                    )}
                </div>
            </div>

            {/* Footer Actions */}
            <div className="flex items-center justify-end gap-2 shrink-0">
                <ActionBtn icon={<Copy size={15} />} label={t('scrolling_preview.copy')} onClick={handleCopy} />
                <ActionBtn
                    icon={<Save size={15} />}
                    label={isSaving ? t('scrolling_preview.saving') : t('scrolling_preview.save_as')}
                    onClick={handleSave}
                    variant="primary"
                />
            </div>
        </div>
    );
};

const ActionBtn: React.FC<{ icon: React.ReactNode, label: string, onClick: () => void, variant?: 'default' | 'primary' }> = ({ icon, label, onClick, variant = 'default' }) => (
    <button
        onClick={onClick}
        className={`flex items-center gap-1.5 px-3.5 py-2 rounded-btn text-[12.5px] font-semibold transition-colors ${
            variant === 'primary'
            ? 'bg-accent text-on-accent hover:bg-[var(--accent-press)]'
            : 'bg-bg-2 text-ink border border-line hover:border-line-2'
        }`}
    >
        {icon}
        {label}
    </button>
);

export default ScrollingPreview;
