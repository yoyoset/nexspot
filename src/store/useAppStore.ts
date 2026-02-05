import { create } from 'zustand';
import { HUDType } from '../components/Overlay/GlobalHUD';

export interface Shortcut {
    id: string;
    label: string;
    shortcut: string;
    icon: string;
    color: string;
    enabled: boolean;
    error?: string;
}

export interface AppConfig {
    shortcuts: Shortcut[];
    save_path: string;
    language: string;
    ocr_engine: string;
}

interface AppState {
    // UI Overlays
    showSettings: boolean;
    ocrResult: string | null;
    startupErrors: string[];

    // Config State
    config: AppConfig | null;
    shortcuts: Shortcut[];

    // HUD State
    hud: {
        message: string;
        type: HUDType;
        visible: boolean;
    };
    hudTimeout: number | null;

    // Actions
    setShowSettings: (show: boolean) => void;
    setOcrResult: (result: string | null) => void;
    setStartupErrors: (errors: string[]) => void;

    // Config Actions
    setConfig: (config: AppConfig) => void;
    setShortcuts: (shortcuts: Shortcut[]) => void;
    updateSavePath: (path: string) => void;

    // HUD Actions
    showHUD: (message: string, type?: HUDType, duration?: number) => void;
    hideHUD: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
    showSettings: false,
    ocrResult: null,
    startupErrors: [],
    config: null,
    shortcuts: [],

    hud: {
        message: '',
        type: 'success',
        visible: false,
    },
    hudTimeout: null,

    setShowSettings: (show) => set({ showSettings: show }),
    setOcrResult: (result) => set({ ocrResult: result }),
    setStartupErrors: (errors) => set({ startupErrors: errors }),

    setConfig: (config) => set({ config, shortcuts: config.shortcuts }),
    setShortcuts: (shortcuts) => set({ shortcuts }),
    updateSavePath: (path) => set((state) => ({
        config: state.config ? { ...state.config, save_path: path } : null
    })),

    showHUD: (message, type = 'success', duration = 2000) => {
        const { hudTimeout } = get();
        if (hudTimeout) {
            clearTimeout(hudTimeout);
        }

        set({
            hud: { message, type, visible: true },
        });

        const timeout = window.setTimeout(() => {
            set((state) => ({
                hud: { ...state.hud, visible: false },
                hudTimeout: null
            }));
        }, duration);

        set({ hudTimeout: timeout });
    },

    hideHUD: () => {
        const { hudTimeout } = get();
        if (hudTimeout) {
            clearTimeout(hudTimeout);
        }
        set((state) => ({
            hud: { ...state.hud, visible: false },
            hudTimeout: null
        }));
    }
}));
