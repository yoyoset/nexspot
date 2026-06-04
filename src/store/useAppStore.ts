import { create } from 'zustand';
import { HUDType } from '../components/Overlay/GlobalHUD';



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

export type AestheticStyle = 'Default' | 'Neon' | 'PaperCut' | 'Sketch' | 'Glass';

export interface AppConfig {
    workflows: Workflow[];
    save_path: string;
    language: string;
    font_family: string;
    vello_enabled: boolean;
    vello_advanced_effects: boolean;
    vello_aesthetic_style: AestheticStyle;
    snapshot_enabled: boolean;
    snapshot_width: number;
    snapshot_height: number;
    selection_engine: string;
    snapshot_engine: string;

    // Appearance
    theme: string;
    accent_color: string;


    // Performance & Quality
    jpg_quality: number;
    concurrency: number;
    default_export_format: string;
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
    };

    // Workflow Editing (Modal)
    workflowEditing: {
        isOpen: boolean;
        workflow: Workflow | null;
    };

    // Vello Engine State
    velloStatus: 'pending' | 'ready' | 'failed';
    velloError: string | null;
    velloErrorModal: {
        isOpen: boolean;
        error: string | null;
    };
    setVelloErrorModal: (isOpen: boolean, error?: string | null) => void;
    
    // Activity State
    activity: ActivityEntry[];
    fetchActivity: () => Promise<void>;


    // Actions
    setShowSettings: (show: boolean, tab?: string, workflowId?: string | null) => void;
    setSettingsNavigation: (tab: string, workflowId?: string | null) => void;
    setWorkflowEditing: (isOpen: boolean, workflow?: Workflow | null) => void;
    setDashboardCollapsible: (key: 'systemPresets' | 'otherPresets', isOpen: boolean) => void;
    setStartupErrors: (errors: string[]) => void;

    // Config Actions
    setConfig: (config: AppConfig) => void;
    updateConfig: (patch: Partial<AppConfig>) => void;
    setVelloStatus: (status: 'pending' | 'ready' | 'failed', error?: string | null) => void;
    updateSavePath: (path: string) => void;

    // HUD Actions
    showHUD: (message: string, type?: HUDType, duration?: number) => void;
    hideHUD: () => void;
}

export interface ActivityEntry {
    id: string;
    timestamp: number;
    activity_type: string;
    file_path: string | null;
    metadata: string | null;
}

export const useAppStore = create<AppState>((set, get) => ({
    showSettings: false,
    startupErrors: [],
    config: null,
    activity: [],

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
    velloStatus: 'pending',
    velloError: null,
    velloErrorModal: {
        isOpen: false,
        error: null
    },
    dashboardCollapsible: (() => {
        try {
            const val = localStorage.getItem('dashboard_collapsible');
            if (!val) return { systemPresets: true, otherPresets: true };
            return JSON.parse(val);
        } catch (e) {
            console.error("Failed to parse dashboardCollapsible", e);
            return { systemPresets: true, otherPresets: true };
        }
    })(),

    setVelloErrorModal: (isOpen, error = null) => set({ velloErrorModal: { isOpen, error: error || get().velloError } }),


    setShowSettings: (show, tab = 'general', workflowId = null) =>
        set({ showSettings: show, settingsNavigation: { tab, workflowId } }),

    setSettingsNavigation: (tab, workflowId = null) =>
        set({ settingsNavigation: { tab, workflowId } }),

    fetchActivity: async () => {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            const data = await invoke<ActivityEntry[]>('get_activity');
            set({ activity: data });
        } catch (e) {
            console.error("Failed to fetch activity", e);
        }
    },

    setWorkflowEditing: (isOpen, workflow = null) =>
        set({ workflowEditing: { isOpen, workflow } }),

    setDashboardCollapsible: (key, isOpen) => {
        const next = { ...get().dashboardCollapsible, [key]: isOpen };
        localStorage.setItem('dashboard_collapsible', JSON.stringify(next));
        set({ dashboardCollapsible: next });
    },

    setStartupErrors: (errors) => set({ startupErrors: errors }),

    setConfig: (config) => set({ config }),
    updateConfig: (patch) => set((state) => ({
        config: state.config ? { ...state.config, ...patch } : null
    })),
    setVelloStatus: (status, error = null) => set({ velloStatus: status, velloError: error }),
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
