import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Copy, Save, AlertCircle, Cpu } from 'lucide-react';

export type HUDType = 'success' | 'copy' | 'save' | 'error' | 'warning' | 'ai';

interface GlobalHUDProps {
    message: string;
    type?: HUDType;
    isVisible: boolean;
}

const GlobalHUD: React.FC<GlobalHUDProps> = ({ message, type = 'success', isVisible }) => {
    const getIcon = () => {
        switch (type) {
            case 'copy': return <Copy className="w-4 h-4 opacity-70" />;
            case 'save': return <Save className="w-4 h-4 opacity-70" />;
            case 'error': return <AlertCircle className="w-4 h-4 text-red-500 opacity-80" />;
            case 'warning': return <AlertCircle className="w-4 h-4 text-amber-500 opacity-80" />;
            case 'ai': return <Cpu className="w-4 h-4 text-accent animate-pulse" />;
            default: return <Check className="w-4 h-4 text-emerald-500 opacity-80" />;
        }
    };

    return (
        <AnimatePresence>
            {isVisible && (
                <div className="fixed inset-0 pointer-events-none z-[100] flex items-center justify-center">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.95, y: -5 }}
                        transition={{ duration: 0.15, ease: "easeOut" }}
                        className="bg-black/80 backdrop-blur-md border border-white/10 px-4 py-2.5 rounded-sm shadow-[0_0_30px_rgba(0,0,0,0.5)] flex items-center gap-3 min-w-[180px]"
                    >
                        <div className="flex-shrink-0 flex items-center justify-center">
                            {getIcon()}
                        </div>
                        <span className="text-text-main font-bold tech-text uppercase tracking-widest text-[10px]">
                            {message}
                        </span>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default GlobalHUD;
