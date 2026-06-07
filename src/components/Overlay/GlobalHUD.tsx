import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Copy, Save, AlertCircle } from 'lucide-react';

export type HUDType = 'success' | 'copy' | 'save' | 'error' | 'warning';

interface GlobalHUDProps {
    message: string;
    type?: HUDType;
    isVisible: boolean;
}

const GlobalHUD: React.FC<GlobalHUDProps> = ({ message, type = 'success', isVisible }) => {
    const getIcon = () => {
        switch (type) {
            case 'copy': return <Copy className="w-4 h-4 text-muted" />;
            case 'save': return <Save className="w-4 h-4 text-muted" />;
            case 'error': return <AlertCircle className="w-4 h-4 text-bad" />;
            case 'warning': return <AlertCircle className="w-4 h-4 text-warn" />;
            default: return <Check className="w-4 h-4 text-ok" />;
        }
    };

    return (
        <AnimatePresence>
            {isVisible && (
                <div className="fixed inset-0 pointer-events-none z-[100] flex items-end justify-center pb-10">
                    <motion.div
                        initial={{ opacity: 0, y: 8 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: 8 }}
                        transition={{ duration: 0.15, ease: "easeOut" }}
                        className="float-win px-4 py-2.5 rounded-panel flex items-center gap-2.5 min-w-[180px]"
                    >
                        <div className="flex-shrink-0 flex items-center justify-center">
                            {getIcon()}
                        </div>
                        <span className="text-ink font-semibold text-[12.5px]">
                            {message}
                        </span>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default GlobalHUD;
