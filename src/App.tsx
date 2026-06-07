import { AnimatePresence } from "framer-motion";
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import SettingsPanel from "./components/Settings/SettingsPanel";
import Dashboard from "./components/Dashboard/Dashboard";
import StartupErrorToast from "./components/Overlay/StartupErrorToast";
import GlobalHUD from "./components/Overlay/GlobalHUD";
import { TauriEventListener } from "./components/Overlay/TauriEventListener";
import TitleBar from "./components/Navigation/TitleBar";
import { invoke } from "@tauri-apps/api/core";
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
    const [isAlwaysOnTop, setIsAlwaysOnTop] = React.useState(false);

    useEffect(() => {
        invoke<boolean>('is_pin_always_on_top').then(setIsAlwaysOnTop).catch(() => { });
    }, []);

    const toggleAlwaysOnTop = async () => {
        try {
            setIsAlwaysOnTop(await invoke<boolean>('toggle_pin_always_on_top'));
        } catch (e) {
            console.error(e);
        }
    };

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

            // Accent: single source --accent; derive press/on-accent (Studio tokens)
            const accent = config.accent_color || '#7a6ff2';
            const hexLum = (hex: string) => {
                const m = hex.replace('#', '');
                if (m.length < 6) return 0;
                const ch = [0, 2, 4].map((i) => parseInt(m.slice(i, i + 2), 16) / 255);
                const f = (c: number) => (c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));
                return 0.2126 * f(ch[0]) + 0.7152 * f(ch[1]) + 0.0722 * f(ch[2]);
            };
            root.style.setProperty('--accent', accent);
            root.style.setProperty('--accent-press', `color-mix(in srgb, ${accent} 82%, #000)`);
            root.style.setProperty('--on-accent', hexLum(accent) > 0.55 ? '#1b1c1f' : '#ffffff');

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
        <main className="w-full h-full relative overflow-hidden bg-bg-0 flex flex-col">
            <TauriEventListener />

            <TitleBar isAlwaysOnTop={isAlwaysOnTop} />

            <div className="flex-1 min-h-0 flex relative overflow-hidden">
            <Navigator
                activeTab={activeTab}
                onTabChange={setActiveTab}
                isAlwaysOnTop={isAlwaysOnTop}
                onToggleAlwaysOnTop={toggleAlwaysOnTop}
            />

            <div className="flex-1 h-full relative overflow-hidden pointer-events-auto">
                {activeTab === 'dashboard' && (
                    <div className="absolute inset-0">
                        <Dashboard onNavigateToWorkflows={handleEditWorkflow} />
                    </div>
                )}
                {activeTab === 'activity' && (
                    <div className="absolute inset-0">
                        <ActivityHub />
                    </div>
                )}
                {activeTab === 'settings' && (
                    <div className="absolute inset-0">
                        <SettingsPanel />
                    </div>
                )}
            </div>
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
