import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, Cpu, RefreshCcw, X, Terminal } from 'lucide-react';
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
                        className="absolute inset-0 bg-black/80 backdrop-blur-md"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: 10 }}
                        className="relative w-full max-w-md bg-bg-main border border-red-500/20 rounded-sm shadow-[0_0_80px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col"
                    >
                        {/* Header/Warning Area */}
                        <div className="bg-red-500/5 p-6 border-b border-red-500/10 flex items-start gap-4">
                            <div className="bg-red-500/10 p-3 rounded-sm border border-red-500/20 text-red-500 shadow-inner">
                                <AlertTriangle className="w-6 h-6 opacity-80" />
                            </div>
                            <div className="flex-1 space-y-1">
                                <h1 className="text-[14px] font-bold text-red-500 tech-text uppercase tracking-[0.2em] leading-tight">
                                    {t('engine_error.crash_title')}
                                </h1>
                                <p className="text-red-500/40 text-[9px] tech-text uppercase font-bold tracking-widest">
                                    {t('engine_error.crash_subtitle')}
                                </p>
                            </div>
                            <button
                                onClick={() => setVelloErrorModal(false)}
                                className="text-red-500/30 hover:text-red-500 p-1 transition-colors"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        {/* Details */}
                        <div className="p-6 space-y-5 bg-black/10">
                            <div className="bg-black/20 rounded-sm p-4 border border-white/5">
                                <div className="flex items-center gap-2 mb-3 text-text-muted opacity-60">
                                    <Terminal className="w-3.5 h-3.5" />
                                    <span className="text-[9px] font-bold tech-text uppercase tracking-widest">{t('engine_error.trace_label')}</span>
                                </div>
                                <code className="text-[10px] font-bold tech-text text-red-400 bg-red-500/5 p-3 rounded-sm border border-red-500/10 block break-words max-h-40 overflow-y-auto custom-scrollbar uppercase">
                                    {velloErrorModal.error || t('engine_error.initializing')}
                                </code>
                            </div>

                            <p className="text-[10px] tech-text text-text-muted leading-relaxed uppercase opacity-60 tracking-tight">
                                {t('engine_error.desc')}
                            </p>
                        </div>

                        {/* Actions */}
                        <div className="p-6 bg-black/40 border-t border-white/5 flex gap-3">
                            <button
                                onClick={handleSwitchToGdi}
                                className="flex-1 bg-red-500/10 hover:bg-red-500 text-red-500 hover:text-white border border-red-500/30 font-bold tech-text text-[10px] py-3.5 rounded-sm flex items-center justify-center gap-2 shadow-lg shadow-red-500/5 transition-all active:scale-95 uppercase tracking-widest"
                            >
                                <RefreshCcw className="w-4 h-4" />
                                {t('engine_error.action_fallback')}
                            </button>
                            <button
                                onClick={() => setVelloErrorModal(false)}
                                className="px-5 py-3.5 border border-white/10 text-text-muted hover:text-text-main hover:bg-white/5 rounded-sm font-bold tech-text text-[10px] transition-all uppercase tracking-widest"
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
