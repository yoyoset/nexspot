import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { X, Save, Copy, Pin, Check, Download } from 'lucide-react';
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
        <div className="w-full h-full bg-black/60 backdrop-blur-xl border border-white/10 flex flex-col p-6 gap-6 select-none relative overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between shrink-0">
                <div className="flex flex-col">
                    <h2 className="text-[14px] font-bold text-text-main tech-text uppercase tracking-widest flex items-center gap-2">
                        <div className="w-1.5 h-1.5 bg-accent" />
                        {t('scrolling_preview.title')}
                    </h2>
                    <span className="text-[9px] text-text-muted tech-text opacity-40 uppercase mt-0.5">
                        {t('scrolling_preview.subtitle')}
                    </span>
                </div>
                <button 
                    onClick={close}
                    className="p-1.5 hover:bg-white/5 rounded-sm text-text-muted hover:text-text-main transition-colors"
                >
                    <X size={18} />
                </button>
            </div>

            {/* Content Area */}
            <div className="flex-1 min-h-0 bg-black/40 border border-white/5 rounded-sm overflow-hidden relative group">
                <div className="absolute inset-0 overflow-y-auto custom-scrollbar p-4 flex justify-center">
                    {imageSrc ? (
                        <motion.img 
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            src={imageSrc} 
                            className="max-w-full h-auto shadow-2xl border border-white/10"
                        />
                    ) : (
                        <div className="flex flex-col items-center justify-center h-full gap-3 text-text-muted/20">
                            <motion.div animate={{ rotate: 360 }} transition={{ repeat: Infinity, duration: 2, ease: "linear" }}>
                                <Download size={32} />
                            </motion.div>
                            <span className="tech-text text-[10px] uppercase font-bold tracking-widest">
                                {t('scrolling_preview.compiling')}
                            </span>
                        </div>
                    )}
                </div>
            </div>

            {/* Footer Actions */}
            <div className="flex items-center justify-end gap-3 shrink-0">
                <ActionBtn icon={<Copy size={14} />} label={t('scrolling_preview.copy')} onClick={handleCopy} />
                <ActionBtn 
                    icon={<Save size={14} />} 
                    label={isSaving ? t('scrolling_preview.saving') : t('scrolling_preview.save_as')} 
                    onClick={handleSave} 
                    variant="primary" 
                />
                <div className="w-[1px] h-4 bg-white/10 mx-1" />
                <ActionBtn icon={<Check size={14} />} label={t('scrolling_preview.finish')} onClick={close} />
            </div>
            
            {/* Background Branding */}
            <div className="absolute -bottom-10 -right-10 opacity-5 pointer-events-none rotate-12">
                <span className="text-[120px] font-black italic select-none">NEXSPOT</span>
            </div>
        </div>
    );
};

const ActionBtn: React.FC<{ icon: React.ReactNode, label: string, onClick: () => void, variant?: 'default' | 'primary' }> = ({ icon, label, onClick, variant = 'default' }) => (
    <button 
        onClick={onClick}
        className={`flex items-center gap-2 px-4 py-2 rounded-sm tech-text text-[10px] font-bold uppercase tracking-widest transition-all border ${
            variant === 'primary' 
            ? 'bg-accent text-white border-accent shadow-[0_0_15px_rgba(var(--color-accent-rgb),0.3)] hover:scale-[1.02]' 
            : 'bg-white/5 text-text-muted border-white/10 hover:bg-white/10 hover:text-text-main'
        }`}
    >
        {icon}
        {label}
    </button>
);

export default ScrollingPreview;
