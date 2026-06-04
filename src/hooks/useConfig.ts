import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { useAppStore, AppConfig, Workflow, AestheticStyle } from "../store/useAppStore";
import { translateError } from "../utils/error";

export function useConfig() {
    const { t } = useTranslation();
    const { config, setConfig, updateConfig, updateSavePath, velloStatus, velloError } = useAppStore();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchConfig = useCallback(async () => {
        try {
            setLoading(true);
            const data = await invoke<AppConfig>("get_config");
            setConfig(data);
            setError(null);

            // Vello Status Sync is now handled here but also on mount
            const vStatus = await invoke<any>("get_vello_status");
            if (vStatus === 'ready') {
                useAppStore.getState().setVelloStatus('ready');
            } else if (typeof vStatus === 'object' && vStatus.failed) {
                useAppStore.getState().setVelloStatus('failed', vStatus.failed);
            } else if (vStatus === 'pending') {
                useAppStore.getState().setVelloStatus('pending');
            } else if (!data.vello_enabled) {
                // If Vello is NOT enabled, we can consider it "ready" (fallback to GDI)
                useAppStore.getState().setVelloStatus('ready');
            }
        } catch (err) {
            setError(String(err));
        } finally {
            setLoading(false);
        }
    }, [setConfig]);

    useEffect(() => {
        // Active Sync on Mount as a fallback to global listeners
        if (velloStatus === 'pending') {
            fetchConfig();
        }
    }, [fetchConfig, velloStatus]);

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

    const setVelloAestheticStyle = async (style: AestheticStyle) => {
        try {
            await invoke('set_vello_aesthetic_style', { style });
            useAppStore.getState().updateConfig({ vello_aesthetic_style: style });
        } catch (err) {
            console.error("Failed to set vello aesthetic style:", err);
            const message = translateError(err, t);
            useAppStore.getState().showHUD(message, 'error');
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

    const setDefaultExportFormat = async (format: string) => {
        try {
            await invoke("set_default_export_format", { format });
            useAppStore.setState((state) => ({
                config: state.config ? { ...state.config, default_export_format: format } : null
            }));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };




    const addWorkflow = async (workflow: Workflow) => {
        await invoke("add_workflow", { workflow });
        useAppStore.setState((state) => {
            const currentWorkflows = state.config?.workflows || [];
            return {
                config: state.config ? { ...state.config, workflows: [...currentWorkflows, workflow] } : null
            };
        });
        return { success: true };
    };

    const removeWorkflow = async (id: string) => {
        await invoke("remove_workflow", { id });
        useAppStore.setState((state) => {
            if (!state.config) return state;
            return {
                config: { ...state.config, workflows: (state.config.workflows || []).filter(w => w.id !== id) }
            };
        });
        return { success: true };
    };

    const updateWorkflow = async (id: string, workflow: Workflow) => {
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
    };

    const openFolder = async (path: string) => {
        try {
            await invoke("open_folder", { path });
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    const emergencyResetToGdi = async () => {
        try {
            await invoke("emergency_reset_to_gdi");
            // Re-fetch everything to sync UI
            await fetchConfig();
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    return {
        config, loading, error, fetchConfig, selectSavePath,
        setFontFamily,
        setVelloEnabled, setVelloAdvancedEffects, setVelloAestheticStyle, setSnapshotEnabled, setSnapshotSize,
        setSelectionEngine, setSnapshotEngine,
        setTheme, setAccentColor,
        addWorkflow, removeWorkflow, updateWorkflow,
        openFolder, emergencyResetToGdi,
        setJpgQuality, setConcurrency, setDefaultExportFormat,
        velloStatus, velloError
    };
}
