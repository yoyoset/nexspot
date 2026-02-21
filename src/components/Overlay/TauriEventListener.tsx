import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { useTranslation } from 'react-i18next';
import { useAppStore, AppConfig, Shortcut } from '../../store/useAppStore';

export const TauriEventListener = () => {
    const { t } = useTranslation();
    const { showHUD, setStartupErrors, setConfig, setShortcuts } = useAppStore();

    useEffect(() => {
        let unlistenStartup: () => void;
        let unlistenDpi: () => void;
        let unlistenSaved: () => void;
        let unlistenCopied: () => void;

        const setupListeners = async () => {
            // 1. Initial State Sync
            try {
                const initialConfig = await invoke<AppConfig>('get_config');
                if (initialConfig) {
                    setConfig(initialConfig);
                }
                const initialShortcuts = await invoke<Shortcut[]>('get_shortcuts');
                setShortcuts(initialShortcuts);
                const initialErrors = await invoke<string[]>('get_startup_errors');
                if (initialErrors && initialErrors.length > 0) {
                    setStartupErrors(initialErrors);
                }
            } catch (e) {
                console.error("Failed to fetch initial state on mount", e);
            }

            unlistenStartup = await listen<string[]>('shortcut-startup-error', (event) => {
                setStartupErrors(event.payload || []);
            });

            unlistenDpi = await listen('mixed-dpi-detected', async () => {
                if (await isPermissionGranted() || await requestPermission()) {
                    sendNotification({
                        title: t('notifications.dpi_title'),
                        body: t('notifications.dpi_body')
                    });
                }
            });

            unlistenSaved = await listen('screenshot-saved', () => {
                showHUD(t('hud.saved'), 'save');
            });

            unlistenCopied = await listen('screenshot-copied', () => {
                showHUD(t('hud.copied'), 'copy');
            });
        };

        setupListeners();

        return () => {
            if (unlistenStartup) unlistenStartup();
            if (unlistenDpi) unlistenDpi();
            if (unlistenSaved) unlistenSaved();
            if (unlistenCopied) unlistenCopied();
        };
    }, [t, showHUD, setStartupErrors]);

    return null; // Headless component
};
