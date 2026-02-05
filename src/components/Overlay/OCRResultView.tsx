import React from "react";
import { Copy, X, Type } from "lucide-react";
import { motion } from "framer-motion";
import { useAppStore } from "../../store/useAppStore";

const OCRResultView: React.FC = () => {
    const { ocrResult: text, setOcrResult } = useAppStore();
    const onClose = () => setOcrResult(null);
    const copyToClipboard = () => {
        if (text) {
            navigator.clipboard.writeText(text);
        }
    };

    return (
        <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="glass-panel max-w-lg min-w-[300px] rounded-xl overflow-hidden shadow-2xl"
        >
            <div className="flex items-center justify-between p-3 border-b border-white/10 bg-white/5">
                <div className="flex items-center gap-2">
                    <Type className="w-4 h-4 text-accent" />
                    <span className="text-[10px] font-bold text-white/50 uppercase tracking-tighter">Recognized Text</span>
                </div>
                <div className="flex items-center gap-1">
                    <button
                        onClick={copyToClipboard}
                        className="hover:bg-white/10 p-1.5 rounded transition-colors group"
                        title="Copy to Clipboard"
                    >
                        <Copy className="w-3.5 h-3.5 text-white/60 group-hover:text-accent" />
                    </button>
                    <button
                        onClick={onClose}
                        className="hover:bg-white/10 p-1.5 rounded transition-colors"
                    >
                        <X className="w-3.5 h-3.5 text-white/60" />
                    </button>
                </div>
            </div>

            <div className="p-4 max-h-[300px] overflow-y-auto">
                <pre className="text-sm text-white/90 whitespace-pre-wrap font-sans leading-relaxed">
                    {text}
                </pre>
            </div>

            <div className="px-4 py-2 bg-accent/5 flex justify-between items-center">
                <span className="text-[9px] text-accent/40 font-mono italic">NexSpot OCR Core // Fast Inference</span>
                <button
                    onClick={copyToClipboard}
                    className="text-[9px] font-bold bg-accent/10 text-accent px-2 py-0.5 rounded border border-accent/20 hover:bg-accent/20 transition-all"
                >
                    COPY ALL
                </button>
            </div>
        </motion.div>
    );
};

export default OCRResultView;
