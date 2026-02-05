import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore, Shortcut } from "../store/useAppStore";

export function useShortcuts() {
    const { shortcuts, setShortcuts } = useAppStore();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchShortcuts = useCallback(async () => {
        try {
            setLoading(true);
            const data = await invoke<Shortcut[]>("get_shortcuts");
            setShortcuts(data);
            setError(null);
        } catch (err) {
            setError(String(err));
        } finally {
            setLoading(false);
        }
    }, [setShortcuts]);

    const updateShortcut = async (id: string, newKeys: string) => {
        try {
            await invoke("update_shortcut", { id, newKeys });
            // Optimistic update or refetch
            setShortcuts(shortcuts.map(s => s.id === id ? { ...s, shortcut: newKeys } : s));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    return { shortcuts, loading, error, fetchShortcuts, updateShortcut };
}
