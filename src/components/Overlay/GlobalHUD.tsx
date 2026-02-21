import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Copy, Save, AlertCircle, Sparkles } from 'lucide-react';

export type HUDType = 'success' | 'copy' | 'save' | 'error' | 'ai';

interface GlobalHUDProps {
    message: string;
    type?: HUDType;
    isVisible: boolean;
}

const GlobalHUD: React.FC<GlobalHUDProps> = ({ message, type = 'success', isVisible }) => {
    const getIcon = () => {
        switch (type) {
            case 'copy': return <Copy className="w-5 h-5" />;
            case 'save': return <Save className="w-5 h-5" />;
            case 'error': return <AlertCircle className="w-5 h-5 text-red-400" />;
            case 'ai': return <Sparkles className="w-5 h-5 text-blue-400 animate-pulse" />;
            default: return <Check className="w-5 h-5 text-emerald-400" />;
        }
    };

    return (
        <AnimatePresence>
            {isVisible && (
                <div className="fixed inset-0 pointer-events-none z-[100] flex items-center justify-center">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.8, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.8, y: -10 }}
                        transition={{ type: 'spring', damping: 20, stiffness: 300 }}
                        className="bg-bg-card/90 backdrop-blur-2xl border border-border-subtle px-6 py-4 rounded-2xl shadow-2xl flex items-center gap-4 min-w-[200px]"
                    >
                        <div className="flex-shrink-0">
                            {getIcon()}
                        </div>
                        <span className="text-text-main font-semibold tracking-tight text-sm">
                            {message}
                        </span>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default GlobalHUD;
