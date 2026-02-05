import { AnimatePresence, motion } from "framer-motion";
import SettingsPanel from "./components/Settings/SettingsPanel";
import OCRResultView from "./components/Overlay/OCRResultView";
import Dashboard from "./components/Dashboard/Dashboard";
import StartupErrorToast from "./components/Overlay/StartupErrorToast";
import GlobalHUD from "./components/Overlay/GlobalHUD";
import { TauriEventListener } from "./components/Overlay/TauriEventListener";
import { useAppStore } from "./store/useAppStore";
import "./App.css";

function App() {
    const {
        showSettings, setShowSettings,
        ocrResult, setOcrResult,
        startupErrors, setStartupErrors,
        hud
    } = useAppStore();

    return (
        <main className="w-full h-full relative overflow-hidden bg-bg-main">
            <TauriEventListener />

            {/* Dashboard is always rendered to maintain context */}
            <Dashboard />

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
                            <SettingsPanel />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* OCR Result Overlay */}
            <AnimatePresence>
                {ocrResult && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
                        onClick={() => setOcrResult(null)}
                    >
                        <div onClick={e => e.stopPropagation()}>
                            <OCRResultView />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Startup Error Toast (Interactive) */}
            <AnimatePresence>
                {startupErrors.length > 0 && (
                    <StartupErrorToast />
                )}
            </AnimatePresence>

            {/* Global HUD Feedback */}
            <GlobalHUD message={hud.message} type={hud.type} isVisible={hud.visible} />
        </main>
    );
}

export default App;
