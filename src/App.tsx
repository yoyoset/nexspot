import { useState, useEffect } from "react";
import { AnimatePresence, motion } from "framer-motion";
import SettingsPanel from "./components/Settings/SettingsPanel";
import OCRResultView from "./components/Overlay/OCRResultView";
import Dashboard from "./components/Dashboard/Dashboard";
import "./App.css";

function App() {
    const [showSettings, setShowSettings] = useState(false);
    const [ocrResult, setOcrResult] = useState<string | null>(null);

    return (
        <main className="w-full h-full relative overflow-hidden bg-bg-main">
            {/* Dashboard is always rendered to maintain context */}
            <Dashboard onOpenSettings={() => setShowSettings(true)} />

            {/* Settings Overlay */}
            <AnimatePresence>
                {showSettings && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
                        onClick={() => setShowSettings(false)} // Click outside to close
                    >
                        <div onClick={e => e.stopPropagation()}>
                            <SettingsPanel onClose={() => setShowSettings(false)} />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* OCR Result Overlay */}
            <AnimatePresence>
                {ocrResult && (
                    <OCRResultView text={ocrResult} onClose={() => setOcrResult(null)} />
                )}
            </AnimatePresence>
        </main>
    );
}

export default App;
