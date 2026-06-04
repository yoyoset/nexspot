import { AnimatePresence, motion } from "framer-motion";
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import SettingsPanel from "./components/Settings/SettingsPanel";
import Dashboard from "./components/Dashboard/Dashboard";
import StartupErrorToast from "./components/Overlay/StartupErrorToast";
import GlobalHUD from "./components/Overlay/GlobalHUD";
import { TauriEventListener } from "./components/Overlay/TauriEventListener";
import { useAppStore } from "./store/useAppStore";
import "./App.css";
import PinCollectionWindow from "./components/Pin/PinCollectionWindow";
import Navigator from "./components/Navigation/Navigator";
import { AppTab } from "./types/navigation";
import ActivityHub from "./components/Dashboard/ActivityHub";
import WorkflowModal from "./components/Workflows/WorkflowModal";
import EngineErrorModal from "./components/Overlay/EngineErrorModal";
import ScrollingPreview from "./components/Overlay/ScrollingPreview";
import OCRResultWindow from "./components/Overlay/OCRResultWindow";
import { Workflow } from "./store/useAppStore";
import { useConfig } from "./hooks/useConfig";

function App() {
    const {
        startupErrors,
        hud,
        config,
        setSettingsNavigation,
        workflowEditing,
        setWorkflowEditing
    } = useAppStore();

    const { updateWorkflow, addWorkflow, removeWorkflow } = useConfig();

    const { t } = useTranslation();
    const [activeTab, setActiveTab] = React.useState<AppTab>('dashboard');

    const handleEditWorkflow = (id?: string) => {
        if (!id || id === 'new') {
            const newWorkflow: Workflow = {
                id: `user_${Date.now()}`,
                label: t('workflows.new_protocol'),
                shortcut: "Alt+F1",
                enabled: true,
                is_system: false,
                action: { type: 'Selection', config: { engine: 'gdi' } },
                output: {
                    save_to_file: true,
                    save_to_clipboard: true,
                    target_folder: null,
                    naming_template: "capture_%Y%m%d_%H%M%S",
                    format: "png"
                }
            };
            setWorkflowEditing(true, newWorkflow);
        } else {
            const w = config?.workflows.find(w => w.id === id);
            if (w) {
                setWorkflowEditing(true, w);
            }
        }
    };

    // Theme & Accent Color Application
    useEffect(() => {
        if (!config) return;

        const applyTheme = (theme: string) => {
            const root = document.documentElement;
            let effectiveTheme = theme;

            if (theme === 'system') {
                effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
            }

            root.setAttribute('data-theme', effectiveTheme);
            root.style.setProperty('--color-accent', config.accent_color);

            // Also update color-scheme for scrollbars/native inputs
            root.style.colorScheme = effectiveTheme;
        };

        applyTheme(config.theme);

        // Listen for system theme changes if set to system
        if (config.theme === 'system') {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            const handleChange = () => applyTheme('system');
            mediaQuery.addEventListener('change', handleChange);
            return () => mediaQuery.removeEventListener('change', handleChange);
        }
    }, [config?.theme, config?.accent_color]);

    const isPinCollection = window.location.hash.includes("pin-collection");
    const isScrollingPreview = window.location.hash.includes("scrolling-preview");
    const isOcrResult = window.location.hash.includes("ocr-result");

    if (isPinCollection) {
        return (
            <main className="w-full h-full relative overflow-hidden bg-transparent">
                <PinCollectionWindow />
                <GlobalHUD message={hud.message} type={hud.type} isVisible={hud.visible} />
            </main>
        );
    }

    if (isScrollingPreview) {
        return (
            <main className="w-full h-full relative overflow-hidden bg-transparent">
                <ScrollingPreview />
                <GlobalHUD message={hud.message} type={hud.type} isVisible={hud.visible} />
            </main>
        );
    }

    if (isOcrResult) {
        return (
            <main className="w-full h-full relative overflow-hidden bg-transparent">
                <OCRResultWindow />
                <GlobalHUD message={hud.message} type={hud.type} isVisible={hud.visible} />
            </main>
        );
    }

    return (
        <main className="w-full h-full relative overflow-hidden bg-bg-main flex">
            <TauriEventListener />

            <Navigator activeTab={activeTab} onTabChange={setActiveTab} />

            <div className="flex-1 h-full relative overflow-hidden pointer-events-auto">
                <AnimatePresence mode="wait">
                    {activeTab === 'dashboard' && (
                        <motion.div
                            key="dashboard"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            className="absolute inset-0"
                        >
                            <Dashboard
                                onNavigateToWorkflows={handleEditWorkflow}
                            />
                        </motion.div>
                    )}

                    {activeTab === 'activity' && (
                        <motion.div
                            key="activity"
                            initial={{ opacity: 0, x: 10 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -10 }}
                            className="absolute inset-0"
                        >
                            <ActivityHub />
                        </motion.div>
                    )}

                    {activeTab === 'settings' && (
                        <motion.div
                            key="settings"
                            initial={{ opacity: 0, x: 10 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -10 }}
                            className="absolute inset-0"
                        >
                            <SettingsPanel />
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>


            {/* Startup Error Toast (Interactive) */}
            <AnimatePresence>
                {startupErrors.length > 0 && (
                    <StartupErrorToast />
                )}
            </AnimatePresence>

            {/* Powerful Engine Error Blocking Modal */}
            <EngineErrorModal />

            {/* Global HUD Feedback */}
            <GlobalHUD message={hud.message} type={hud.type} isVisible={hud.visible} />

            {/* Global Workflow/Preset Modal */}
            <WorkflowModal
                isOpen={workflowEditing.isOpen}
                onClose={() => setWorkflowEditing(false)}
                workflow={workflowEditing.workflow}
                onSave={async (w) => {
                    const latestConfig = useAppStore.getState().config;
                    const currentWorkflows = latestConfig?.workflows || [];
                    const exists = currentWorkflows.some(ex => ex.id === w.id);

                    if (exists) {
                        await updateWorkflow(w.id, w);
                    } else {
                        await addWorkflow(w);
                    }
                    useAppStore.getState().showHUD(t('hud.saved'), 'success');
                }}
                onDelete={async (id) => {
                    await removeWorkflow(id);
                }}
                save_path={config?.save_path}
            />
        </main>
    );
}

export default App;
