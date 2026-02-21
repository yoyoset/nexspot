import { create } from 'zustand';
import { HUDType } from '../components/Overlay/GlobalHUD';


export interface AIShortcut {
    id: string;
    name: string;
    prompt: string;
    shortcut?: string;
}

export interface CaptureAction {
    type: 'Selection' | 'Fullscreen' | 'Window' | 'Snapshot';
    config: {
        engine: string;
        width?: number;
        height?: number;
        allow_resize?: boolean;
    };
}

export interface CaptureOutput {
    save_to_file: boolean;
    save_to_clipboard: boolean;
    target_folder: string | null;
    naming_template: string;
    format: string;
}

export interface Workflow {
    id: string;
    label: string;
    shortcut: string;
    action: CaptureAction;
    output: CaptureOutput;
    enabled: boolean;
    is_system: boolean;
}

export interface AppConfig {
    workflows: Workflow[];
    save_path: string;
    language: string;
    font_family: string;
    vello_enabled: boolean;
    vello_advanced_effects: boolean;
    snapshot_enabled: boolean;
    snapshot_width: number;
    snapshot_height: number;
    selection_engine: string;
    snapshot_engine: string;

    // Appearance
    theme: string;
    accent_color: string;

    // AI Configuration
    ai_shortcuts: AIShortcut[];

    // Performance & Quality
    jpg_quality: number;
    concurrency: number;

    registration_errors: string[];
}

interface AppState {
    // UI Overlays
    showSettings: boolean;
    startupErrors: string[];

    // Config State
    config: AppConfig | null;

    // HUD State
    hud: {
        message: string;
        type: HUDType;
        visible: boolean;
    };
    hudTimeout: number | null;

    // Navigation State for Deep Linking
    settingsNavigation: {
        tab: string;
        workflowId: string | null;
    };

    // Dashboard Persistence
    dashboardCollapsible: {
        systemPresets: boolean;
        otherPresets: boolean;
        aiShortcuts: boolean;
    };

    // Workflow Editing (Modal)
    workflowEditing: {
        isOpen: boolean;
        workflow: Workflow | null;
    };

    // Actions
    setShowSettings: (show: boolean, tab?: string, workflowId?: string | null) => void;
    setSettingsNavigation: (tab: string, workflowId?: string | null) => void;
    setWorkflowEditing: (isOpen: boolean, workflow?: Workflow | null) => void;
    setDashboardCollapsible: (key: 'systemPresets' | 'otherPresets' | 'aiShortcuts', isOpen: boolean) => void;
    setStartupErrors: (errors: string[]) => void;

    // Config Actions
    setConfig: (config: AppConfig) => void;
    updateSavePath: (path: string) => void;

    // HUD Actions
    showHUD: (message: string, type?: HUDType, duration?: number) => void;
    hideHUD: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
    showSettings: false,
    startupErrors: [],
    config: null,

    hud: {
        message: '',
        type: 'success',
        visible: false,
    },
    hudTimeout: null,
    settingsNavigation: {
        tab: 'general',
        workflowId: null
    },
    workflowEditing: {
        isOpen: false,
        workflow: null
    },
    dashboardCollapsible: JSON.parse(localStorage.getItem('dashboard_collapsible') || '{"systemPresets":true,"otherPresets":true,"aiShortcuts":true}'),

    setShowSettings: (show, tab = 'general', workflowId = null) =>
        set({ showSettings: show, settingsNavigation: { tab, workflowId } }),

    setSettingsNavigation: (tab, workflowId = null) =>
        set({ settingsNavigation: { tab, workflowId } }),

    setWorkflowEditing: (isOpen, workflow = null) =>
        set({ workflowEditing: { isOpen, workflow } }),

    setDashboardCollapsible: (key, isOpen) => {
        const next = { ...get().dashboardCollapsible, [key]: isOpen };
        localStorage.setItem('dashboard_collapsible', JSON.stringify(next));
        set({ dashboardCollapsible: next });
    },

    setStartupErrors: (errors) => set({ startupErrors: errors }),

    setConfig: (config) => set({ config }),
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
