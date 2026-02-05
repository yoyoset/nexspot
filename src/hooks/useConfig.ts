import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore, AppConfig } from "../store/useAppStore";

export function useConfig() {
    const { config, setConfig, updateSavePath } = useAppStore();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchConfig = useCallback(async () => {
        try {
            setLoading(true);
            const data = await invoke<AppConfig>("get_config");
            setConfig(data);
            setError(null);
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

    return { config, loading, error, fetchConfig, selectSavePath, setOcrEngine };
}
