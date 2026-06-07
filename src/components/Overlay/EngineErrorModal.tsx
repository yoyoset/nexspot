import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, RefreshCcw, X, Terminal } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { useConfig } from '../../hooks/useConfig';
import { useTranslation } from 'react-i18next';

const EngineErrorModal: React.FC = () => {
    const { velloErrorModal, setVelloErrorModal } = useAppStore();
    const { emergencyResetToGdi } = useConfig();
    const { t } = useTranslation();

    const handleSwitchToGdi = async () => {
        try {
            const result = await emergencyResetToGdi();
            if (result.success) {
                setVelloErrorModal(false);
                useAppStore.getState().showHUD(t('hud.engine_switched'), 'success');
            } else {
                console.error("Emergency reset failed:", result.error);
            }
        } catch (err) {
            console.error("Failed to switch engine:", err);
        }
    };

    return (
        <AnimatePresence>
            {velloErrorModal.isOpen && (
                <div className="fixed inset-0 z-[110] flex items-center justify-center p-8 sm:p-24">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={() => setVelloErrorModal(false)}
                        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ opacity: 0, y: 8, scale: 0.99 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: 8, scale: 0.99 }}
                        transition={{ duration: 0.18 }}
                        className="relative w-full max-w-md bg-bg-1 border border-line-2 rounded-lg shadow-float overflow-hidden flex flex-col"
                    >
                        {/* Header/Warning Area */}
                        <div className="p-6 border-b border-line flex items-start gap-4">
                            <div className="w-11 h-11 bg-bad-soft rounded-[10px] flex items-center justify-center text-bad shrink-0">
                                <AlertTriangle className="w-6 h-6" />
                            </div>
                            <div className="flex-1 space-y-0.5">
                                <h1 className="text-[15px] font-extrabold text-ink tracking-[-0.01em] leading-tight">
                                    {t('engine_error.crash_title')}
                                </h1>
                                <p className="text-muted text-[11.5px]">
                                    {t('engine_error.crash_subtitle')}
                                </p>
                            </div>
                            <button
                                onClick={() => setVelloErrorModal(false)}
                                className="text-muted hover:text-ink p-1 transition-colors"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        {/* Details */}
                        <div className="p-6 space-y-4">
                            <div className="bg-bg-2 rounded-panel p-4 border border-line">
                                <div className="flex items-center gap-2 mb-2.5 text-muted">
                                    <Terminal className="w-3.5 h-3.5" />
                                    <span className="mono text-[10.5px] font-semibold tracking-[0.08em] uppercase">{t('engine_error.trace_label')}</span>
                                </div>
                                <code className="mono text-[11px] text-bad bg-bad-soft p-3 rounded-md block break-words max-h-40 overflow-y-auto custom-scrollbar">
                                    {velloErrorModal.error || t('engine_error.initializing')}
                                </code>
                            </div>

                            <p className="text-[12px] text-muted leading-relaxed">
                                {t('engine_error.desc')}
                            </p>
                        </div>

                        {/* Actions */}
                        <div className="p-6 pt-0 flex gap-2">
                            <button
                                onClick={handleSwitchToGdi}
                                className="flex-1 bg-accent text-on-accent font-semibold text-[12.5px] py-2.5 rounded-btn flex items-center justify-center gap-1.5 hover:bg-[var(--accent-press)] transition-colors"
                            >
                                <RefreshCcw className="w-4 h-4" />
                                {t('engine_error.action_fallback')}
                            </button>
                            <button
                                onClick={() => setVelloErrorModal(false)}
                                className="px-5 py-2.5 text-muted hover:text-ink hover:bg-bg-2 rounded-btn font-semibold text-[12.5px] transition-colors"
                            >
                                {t('engine_error.action_close')}
                            </button>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default EngineErrorModal;
