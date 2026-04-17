import React from 'react';
import { motion } from 'framer-motion';
import { AlertTriangle, X, Settings, Terminal } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/useAppStore';

const StartupErrorToast: React.FC = () => {
    const { startupErrors, setStartupErrors, setShowSettings } = useAppStore();
    const { t } = useTranslation();

    const onDismiss = () => setStartupErrors([]);
    const onOpenSettings = () => {
        setStartupErrors([]);
        setShowSettings(true);
    };

    return (
        <motion.div
            initial={{ opacity: 0, y: 50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="fixed bottom-6 right-6 z-50 w-80 bg-black/60 border border-red-500/30 backdrop-blur-md rounded-sm shadow-[0_0_40px_rgba(239,68,68,0.15)] overflow-hidden"
        >
            <div className="p-4">
                <div className="flex items-start gap-3">
                    <div className="p-2 bg-red-500/10 rounded-sm border border-red-500/20 shrink-0">
                        <AlertTriangle className="w-4 h-4 text-red-500 opacity-80" />
                    </div>
                    <div className="flex-1 min-w-0">
                        <h4 className="text-[11px] font-bold text-red-500 tech-text uppercase tracking-widest mb-1.5 leading-none">
                            {t('notifications.conflict_title')}
                        </h4>
                        <div className="space-y-1.5">
                            {startupErrors.map((err, i) => (
                                <div key={i} className="text-[9px] font-bold tech-text text-red-400 bg-red-500/5 px-2 py-1.5 rounded-sm border border-red-500/10 truncate uppercase tracking-tight">
                                    ERR_CODE_{i}: {err}
                                </div>
                            ))}
                        </div>
                    </div>
                    <button
                        onClick={onDismiss}
                        className="text-red-500/40 hover:text-red-500 transition-colors -mt-1 -mr-1 p-1"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <div className="mt-4 flex gap-2">
                    <button
                        onClick={onOpenSettings}
                        className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-red-500/10 hover:bg-red-500 text-red-500 hover:text-white text-[9px] tech-text font-bold rounded-sm border border-red-500/30 transition-all uppercase tracking-widest shadow-lg shadow-red-500/10"
                    >
                        <Settings className="w-3 h-3" />
                        RECONFIGURE
                    </button>
                    <button
                        onClick={onDismiss}
                        className="px-4 py-2 bg-transparent hover:bg-white/5 text-text-muted hover:text-text-main text-[9px] tech-text font-bold rounded-sm border border-white/5 transition-all uppercase tracking-widest"
                    >
                        IGNORE
                    </button>
                </div>
            </div>
        </motion.div>
    );
};

export default StartupErrorToast;
