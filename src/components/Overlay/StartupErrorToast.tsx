import React from 'react';
import { motion } from 'framer-motion';
import { AlertTriangle, X, Settings } from 'lucide-react';
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
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 12 }}
            className="fixed bottom-6 right-6 z-50 w-80 float-win overflow-hidden"
        >
            <div className="p-4">
                <div className="flex items-start gap-3">
                    <div className="w-9 h-9 bg-bad-soft rounded-[9px] flex items-center justify-center shrink-0">
                        <AlertTriangle className="w-4 h-4 text-bad" />
                    </div>
                    <div className="flex-1 min-w-0">
                        <h4 className="text-[13px] font-extrabold text-ink mb-1.5 leading-tight">
                            {t('notifications.conflict_title')}
                        </h4>
                        <div className="space-y-1.5">
                            {startupErrors.map((err, i) => (
                                <div key={i} className="mono text-[10.5px] text-bad bg-bad-soft px-2 py-1.5 rounded-md truncate">
                                    {err}
                                </div>
                            ))}
                        </div>
                    </div>
                    <button
                        onClick={onDismiss}
                        className="text-muted hover:text-ink transition-colors -mt-1 -mr-1 p-1"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <div className="mt-4 flex gap-2">
                    <button
                        onClick={onOpenSettings}
                        className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 bg-accent text-on-accent text-[12px] font-semibold rounded-btn hover:bg-[var(--accent-press)] transition-colors"
                    >
                        <Settings className="w-3.5 h-3.5" />
                        {t('nav.settings')}
                    </button>
                    <button
                        onClick={onDismiss}
                        className="px-4 py-2 text-muted hover:text-ink hover:bg-bg-2 text-[12px] font-semibold rounded-btn transition-colors"
                    >
                        {t('common.cancel')}
                    </button>
                </div>
            </div>
        </motion.div>
    );
};

export default StartupErrorToast;
