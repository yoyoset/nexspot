import React from "react";
import { FolderOpen, Scan, Monitor, AppWindow, Crop, HardDrive, Clipboard } from "lucide-react";
import ShortcutRecorder from "../Settings/ShortcutRecorder";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Workflow } from "../../store/useAppStore";
import { Segmented, Toggle, TextField } from "../Settings/atoms";

interface WorkflowFormProps {
    workflow: Workflow;
    onChange: (w: Workflow) => void;
    save_path?: string;
}

const MODES: { type: string; icon: React.ComponentType<{ className?: string }>; key: string }[] = [
    { type: 'Selection', icon: Scan, key: 'mode_selection' },
    { type: 'Fullscreen', icon: Monitor, key: 'mode_fullscreen' },
    { type: 'Window', icon: AppWindow, key: 'mode_window' },
    { type: 'Snapshot', icon: Crop, key: 'mode_snapshot' },
];

const FieldLabel: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <label className="text-[12px] font-semibold text-muted mb-1.5 block">{children}</label>
);

const WorkflowForm: React.FC<WorkflowFormProps> = ({ workflow, onChange, save_path }) => {
    const { t } = useTranslation();

    const updateAction = (updates: any) => onChange({ ...workflow, action: { ...workflow.action, ...updates } });
    const updateActionConfig = (updates: any) => onChange({ ...workflow, action: { ...workflow.action, config: { ...workflow.action.config, ...updates } } });
    const updateOutput = (updates: any) => onChange({ ...workflow, output: { ...workflow.output, ...updates } });

    const ToggleCard: React.FC<{ icon: React.ComponentType<{ className?: string }>; label: string; checked: boolean; onChange: (v: boolean) => void }> =
        ({ icon: Icon, label, checked, onChange }) => (
            <div className="flex-1 flex items-center gap-2.5 px-3 py-2.5 rounded-panel bg-bg-2 border border-line">
                <Icon className="w-4 h-4 text-muted shrink-0" />
                <span className="text-[12.5px] font-semibold text-ink flex-1">{label}</span>
                <Toggle checked={checked} onChange={onChange} />
            </div>
        );

    return (
        <div className="flex flex-col gap-5">
            {/* Name + enabled */}
            <div>
                <div className="flex items-center justify-between mb-1.5">
                    <FieldLabel>{t('workflows.label')}</FieldLabel>
                    {!workflow.is_system && (
                        <div className="flex items-center gap-2">
                            <span className="text-[11.5px] text-muted">{t('workflows.active')}</span>
                            <Toggle checked={workflow.enabled} onChange={(v) => onChange({ ...workflow, enabled: v })} />
                        </div>
                    )}
                </div>
                <TextField
                    value={workflow.label}
                    onChange={(e) => onChange({ ...workflow, label: e.target.value })}
                    placeholder={t('workflows.label')}
                    className="w-full"
                />
            </div>

            {/* Shortcut */}
            <div>
                <FieldLabel>{t('workflows.shortcut')}</FieldLabel>
                <ShortcutRecorder
                    value={workflow.shortcut}
                    onChange={(k) => onChange({ ...workflow, shortcut: k })}
                    placeholder={t('common.none')}
                    status={workflow.shortcut ? 'success' : 'neutral'}
                />
            </div>

            {/* Mode 4-card */}
            <div>
                <FieldLabel>{t('workflows.mode')}</FieldLabel>
                <div className="grid grid-cols-4 gap-2">
                    {MODES.map((m) => {
                        const active = workflow.action.type === m.type;
                        const Icon = m.icon;
                        return (
                            <button
                                key={m.type}
                                onClick={() => updateAction({ type: m.type })}
                                className={`flex flex-col items-center gap-1.5 py-3 rounded-panel border transition-colors ${active ? 'bg-accent-soft border-accent-line text-accent' : 'bg-bg-2 border-line text-muted hover:text-ink hover:border-line-2'}`}
                            >
                                <Icon className="w-5 h-5" />
                                <span className="text-[11px] font-semibold text-center px-1 leading-tight">{t(`workflows.${m.key}`)}</span>
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Snapshot size (only Snapshot) */}
            {workflow.action.type === 'Snapshot' && (
                <div className="flex gap-4">
                    <div className="flex-1">
                        <FieldLabel>{t('workflows.width')}</FieldLabel>
                        <TextField mono type="number" value={workflow.action.config.width ?? 800}
                            onChange={(e) => updateActionConfig({ width: parseInt(e.target.value) })} className="w-full" />
                    </div>
                    <div className="flex-1">
                        <FieldLabel>{t('workflows.height')}</FieldLabel>
                        <TextField mono type="number" value={workflow.action.config.height ?? 600}
                            onChange={(e) => updateActionConfig({ height: parseInt(e.target.value) })} className="w-full" />
                    </div>
                </div>
            )}

            {/* Engine + Format */}
            <div className="flex flex-wrap gap-6">
                <div>
                    <FieldLabel>{t('workflows.engine')}</FieldLabel>
                    <Segmented
                        value={workflow.action.config.engine}
                        onChange={(v) => updateActionConfig({ engine: v })}
                        options={[{ value: 'gdi', label: 'GDI' }, { value: 'vello', label: 'Vello' }]}
                    />
                </div>
                <div>
                    <FieldLabel>{t('workflows.format')}</FieldLabel>
                    <Segmented
                        value={workflow.output.format}
                        onChange={(v) => updateOutput({ format: v })}
                        options={[{ value: 'png', label: 'PNG' }, { value: 'jpg', label: 'JPG' }, { value: 'webp', label: 'WEBP' }]}
                    />
                </div>
            </div>

            {/* Save location */}
            <div>
                <FieldLabel>{t('workflows.target_folder')}</FieldLabel>
                <div className="flex gap-2">
                    <div className="flex-1 h-9 bg-bg-3 border border-line rounded-btn px-3 flex items-center mono text-[11.5px] text-muted truncate">
                        {workflow.output.target_folder || save_path || t('workflows.global_default')}
                    </div>
                    <button
                        onClick={async () => {
                            const path = await invoke('select_folder');
                            if (path) updateOutput({ target_folder: path as string });
                        }}
                        className="flex items-center gap-1.5 h-9 px-3 rounded-btn bg-bg-2 border border-line text-[12.5px] font-semibold text-ink hover:border-line-2 transition-colors shrink-0"
                    >
                        <FolderOpen className="w-4 h-4 text-muted" /> {t('common.select')}
                    </button>
                </div>
            </div>

            {/* Naming */}
            <div>
                <FieldLabel>{t('workflows.naming')}</FieldLabel>
                <TextField mono
                    value={workflow.output.naming_template}
                    onChange={(e) => updateOutput({ naming_template: e.target.value })}
                    placeholder={t('workflows.naming_placeholder')}
                    className="w-full"
                />
            </div>

            {/* Output toggles */}
            <div className="flex gap-3">
                <ToggleCard icon={HardDrive} label={t('workflows.save_to_file')} checked={workflow.output.save_to_file} onChange={(v) => updateOutput({ save_to_file: v })} />
                <ToggleCard icon={Clipboard} label={t('workflows.copy_to_clipboard')} checked={workflow.output.save_to_clipboard} onChange={(v) => updateOutput({ save_to_clipboard: v })} />
            </div>
        </div>
    );
};

export default WorkflowForm;
