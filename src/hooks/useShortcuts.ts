import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Shortcut {
    id: string;
    label: string;
    shortcut: string;
    icon: string;
    color: string;
    enabled: boolean;
}

export function useShortcuts() {
    const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchShortcuts = async () => {
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
    };

    const updateShortcut = async (id: string, newKeys: string) => {
        try {
            await invoke("update_shortcut", { id, newKeys });
            // Optimistic update or refetch
            setShortcuts(prev => prev.map(s => s.id === id ? { ...s, shortcut: newKeys } : s));
            return { success: true };
        } catch (err) {
            return { success: false, error: String(err) };
        }
    };

    useEffect(() => {
        fetchShortcuts();
    }, []);

    return { shortcuts, loading, error, updateShortcut };
}
