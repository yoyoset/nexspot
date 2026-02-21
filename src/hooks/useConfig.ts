import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore, AppConfig, AIShortcut, Workflow } from "../store/useAppStore";

export function useConfig() {
    const { config, setConfig, updateSavePath } = useAppStore();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [isVelloReady, setIsVelloReady] = useState(false);

    useEffect(() => {
        // Listen for Vello Ready Event
        import("@tauri-apps/api/event").then(({ listen }) => {
            const unlisten = listen("vello://ready", () => {
                console.log("Vello Engine Ready");
                setIsVelloReady(true);
            });
            return unlisten;
        }).then((unlistenPromise) => {
            // Cleanup if needed, though this hook is likely global
        });
    }, []);

    const fetchConfig = useCallback(async () => {
        try {
            setLoading(true);
            const data = await invoke<AppConfig>("get_config");
            setConfig(data);
            setError(null);

            // 1. Sync Vello Status from Backend
            const vReady = await invoke<boolean>("is_vello_ready");
            if (vReady) {
                setIsVelloReady(true);
            } else if (!data.vello_enabled) {
                // If Vello is NOT enabled, we can consider it "ready" (fallback to GDI)
                setIsVelloReady(true);
            }
        } catch (err) {
            setError(String(err));
        } finally {
            setLoading(false);
        }
    }, [setConfig]);

    const selectSavePath = async () => {
        try {
            const path = await invoke<string | null>("select_folder");
            if (path) {
                await invoke("set_save_path", { path });
                updateSavePath(path);
                return { success: true, path };
            }
            return { success: false, cancelled: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setOcrEngine = async (engine: string) => {
        try {
            await invoke("set_ocr_engine", { engine });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, ocr_engine: engine } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setFontFamily = async (font: string) => {
        try {
            await invoke("set_font_family", { font });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, font_family: font } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setVelloEnabled = async (enabled: boolean) => {
        try {
            await invoke("set_vello_enabled", { enabled });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, vello_enabled: enabled } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setSnapshotEnabled = async (enabled: boolean) => {
        try {
            await invoke("set_snapshot_enabled", { enabled });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, snapshot_enabled: enabled } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setSnapshotSize = async (width: number, height: number) => {
        try {
            await invoke("set_snapshot_size", { width, height });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, snapshot_width: width, snapshot_height: height } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setVelloAdvancedEffects = async (enabled: boolean) => {
        try {
            await invoke("set_vello_advanced_effects", { enabled });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, vello_advanced_effects: enabled } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };
    const setSelectionEngine = async (engine: string) => {
        try {
            await invoke("set_selection_engine", { engine });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, selection_engine: engine } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setSnapshotEngine = async (engine: string) => {
        try {
            await invoke("set_snapshot_engine", { engine });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, snapshot_engine: engine } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setTheme = async (theme: string) => {
        try {
            await invoke("set_theme", { theme });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, theme } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setAccentColor = async (color: string) => {
        try {
            await invoke("set_accent_color", { color });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, accent_color: color } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setJpgQuality = async (quality: number) => {
        try {
            await invoke("set_jpg_quality", { quality });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, jpg_quality: quality } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const setConcurrency = async (concurrency: number) => {
        try {
            await invoke("set_concurrency", { concurrency });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, concurrency } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const addAIShortcut = async (shortcut: AIShortcut) => {
        try {
            await invoke("add_ai_shortcut", { shortcut });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: { ...state.config, ai_shortcuts: [...(state.config.ai_shortcuts || []), shortcut] }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const removeAIShortcut = async (id: string) => {
        try {
            await invoke("remove_ai_shortcut", { id });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: { ...state.config, ai_shortcuts: (state.config.ai_shortcuts || []).filter(s => s.id !== id) }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const updateAIShortcut = async (id: string, shortcut: AIShortcut) => {
        try {
            await invoke("update_ai_shortcut", { id, shortcut });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: {
                        ...state.config,
                        ai_shortcuts: (state.config.ai_shortcuts || []).map(s => s.id === id ? shortcut : s)
                    }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const addWorkflow = async (workflow: Workflow) => {
        try {
            await invoke("add_workflow", { workflow });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: { ...state.config, workflows: [...(state.config.workflows || []), workflow] }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const removeWorkflow = async (id: string) => {
        try {
            await invoke("remove_workflow", { id });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: { ...state.config, workflows: (state.config.workflows || []).filter(w => w.id !== id) }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const updateWorkflow = async (id: string, workflow: Workflow) => {
        try {
            await invoke("update_workflow", { id, workflow });
            useAppStore.setState((state) => {
                if (!state.config) return state;
                return {
                    config: {
                        ...state.config,
                        workflows: (state.config.workflows || []).map(w => w.id === id ? workflow : w)
                    }
                };
            });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    return {
        config, loading, error, fetchConfig, selectSavePath,
        setOcrEngine, setFontFamily,
        setVelloEnabled, setVelloAdvancedEffects, setSnapshotEnabled, setSnapshotSize,
        setSelectionEngine, setSnapshotEngine,
        setTheme, setAccentColor,
        addAIShortcut, removeAIShortcut, updateAIShortcut,
        addWorkflow, removeWorkflow, updateWorkflow,
        setJpgQuality, setConcurrency,
        isVelloReady
    };
}
