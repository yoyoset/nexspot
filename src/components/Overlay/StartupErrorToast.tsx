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
            initial={{ opacity: 0, y: 50, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 20, scale: 0.95 }}
            className="fixed bottom-6 right-6 z-50 w-80 bg-red-500/10 border border-red-500/20 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden"
        >
            <div className="p-4">
                <div className="flex items-start gap-3">
                    <div className="p-2 bg-red-500/20 rounded-lg shrink-0">
                        <AlertTriangle className="w-5 h-5 text-red-500" />
                    </div>
                    <div className="flex-1 min-w-0">
                        <h4 className="text-sm font-semibold text-red-100 mb-1">
                            {t('notifications.conflict_title')}
                        </h4>
                        <p className="text-xs text-red-200/80 leading-relaxed mb-3">
                            {t('settings.shortcuts.errors.startup_error', { errors: '' })}
                        </p>
                        <div className="space-y-1">
                            {startupErrors.map((err, i) => (
                                <div key={i} className="text-xs font-mono text-red-300 bg-red-500/10 px-2 py-1 rounded border border-red-500/10 truncate">
                                    {err}
                                </div>
                            ))}
                        </div>
                    </div>
                    <button
                        onClick={onDismiss}
                        className="text-red-300 hover:text-red-100 transition-colors -mt-1 -mr-1 p-1"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <div className="mt-4 flex gap-2">
                    <button
                        onClick={onOpenSettings}
                        className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-red-500 hover:bg-red-400 text-white text-xs font-medium rounded-lg transition-colors shadow-lg shadow-red-500/20"
                    >
                        <Settings className="w-3 h-3" />
                        {t('settings.tabs.shortcuts')}
                    </button>
                    <button
                        onClick={onDismiss}
                        className="px-3 py-2 bg-transparent hover:bg-red-500/10 text-red-200 text-xs font-medium rounded-lg transition-colors border border-red-500/20"
                    >
                        Ignore
                    </button>
                </div>
            </div>

            {/* Progress/Timeout bar could go here if we wanted auto-dismiss, but critical errors should probably stick */}
        </motion.div>
    );
};

export default StartupErrorToast;
