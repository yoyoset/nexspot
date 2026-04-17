import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { useTranslation } from 'react-i18next';
import { useAppStore, AppConfig } from '../../store/useAppStore';

export const TauriEventListener = () => {
    const { t } = useTranslation();
    const { showHUD, setStartupErrors, setConfig } = useAppStore();

    useEffect(() => {
        let unlistenStartup: () => void;
        let unlistenDpi: () => void;
        let unlistenSaved: () => void;
        let unlistenCopied: () => void;
        let unlistenVelloReady: () => void;
        let unlistenVelloError: () => void;

        const setupListeners = async () => {
            // 1. Initial State Sync
            try {
                const initialConfig = await invoke<AppConfig>('get_config');
                if (initialConfig) {
                    setConfig(initialConfig);

                    // Initial Vello Status Sync
                    const vStatus = await invoke<any>('get_vello_status');
                    console.log("[Vello Sync] Initial Status:", vStatus, "Enabled:", initialConfig.vello_enabled);

                    if (vStatus === 'ready') {
                        useAppStore.getState().setVelloStatus('ready');
                    } else if (typeof vStatus === 'object' && vStatus.failed) {
                        useAppStore.getState().setVelloStatus('failed', vStatus.failed);
                    } else if (vStatus === 'pending') {
                        useAppStore.getState().setVelloStatus('pending');
                    } else if (!initialConfig.vello_enabled) {
                        useAppStore.getState().setVelloStatus('ready');
                    }
                }
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
                showHUD(t('notifications.dpi_title'), 'warning');
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

            unlistenVelloReady = await listen('vello://ready', () => {
                console.log("Vello Engine Ready (Global)");
                useAppStore.getState().setVelloStatus('ready');
            });

            unlistenVelloError = await listen<string>('vello://error', (event) => {
                console.error("Vello Engine Error (Global):", event.payload);
                useAppStore.getState().setVelloStatus('failed', event.payload);
                // HUD is too weak for hardware failures, use the Blocking Modal
                useAppStore.getState().setVelloErrorModal(true, event.payload);
            });
        };

        setupListeners();

        return () => {
            if (unlistenStartup) unlistenStartup();
            if (unlistenDpi) unlistenDpi();
            if (unlistenSaved) unlistenSaved();
            if (unlistenCopied) unlistenCopied();
            if (unlistenVelloReady) unlistenVelloReady();
            if (unlistenVelloError) unlistenVelloError();
        };
    }, [t, showHUD, setStartupErrors]);

    return null; // Headless component
};
