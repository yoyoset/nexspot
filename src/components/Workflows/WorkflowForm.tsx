import React from "react";
import { Keyboard, Cpu, FolderOpen, Check, Camera, Scan, Maximize, Target } from "lucide-react";
import ShortcutRecorder from "../Settings/ShortcutRecorder";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Workflow } from "../../store/useAppStore";

interface WorkflowFormProps {
    workflow: Workflow;
    onChange: (w: Workflow) => void;
    save_path?: string; // Global save path
}

const WorkflowForm: React.FC<WorkflowFormProps> = ({ workflow, onChange, save_path }) => {
    const { t } = useTranslation();

    const updateAction = (updates: any) => {
        onChange({
            ...workflow,
            action: {
                ...workflow.action,
                ...updates
            }
        });
    };

    const updateActionConfig = (updates: any) => {
        onChange({
            ...workflow,
            action: {
                ...workflow.action,
                config: { ...workflow.action.config, ...updates }
            }
        });
    };

    const updateOutput = (updates: any) => {
        onChange({
            ...workflow,
            output: {
                ...workflow.output,
                ...updates
            }
        });
    };

    return (
        <div className="space-y-6">
            {/* Section 1: Basic Identity */}
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold pl-1">{t('workflows.label')}</label>
                    <input
                        type="text"
                        value={workflow.label}
                        onChange={e => onChange({ ...workflow, label: e.target.value })}
                        className="w-full bg-bg-main border border-border-subtle rounded-xl px-3 py-2.5 text-xs text-text-main outline-none focus:border-accent/50 transition-colors shadow-inner"
                    />
                </div>
                <div className="space-y-1.5 flex flex-col">
                    <label className="text-[10px] text-text-muted uppercase font-bold pl-1 mb-1.5">{t('workflows.shortcut')}</label>
                    <ShortcutRecorder
                        value={workflow.shortcut}
                        onChange={(k) => onChange({ ...workflow, shortcut: k })}
                        placeholder={t('workflows.bind_hint') || "None"}
                        className="w-full h-9"
                    />
                </div>
            </div>

            {/* Section 2: Capture Engine & Type */}
            <div className="grid grid-cols-2 gap-4 pt-2">
                <div className="space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold pl-1">{t('workflows.engine')}</label>
                    <div className="relative group">
                        <Cpu className="absolute left-3 top-3 w-3.5 h-3.5 text-text-muted group-focus-within:text-accent transition-colors" />
                        <select
                            value={workflow.action.config.engine}
                            onChange={e => updateActionConfig({ engine: e.target.value })}
                            className="w-full bg-bg-main border border-border-subtle rounded-xl pl-9 pr-3 py-2.5 text-xs text-text-main outline-none focus:border-accent/50 appearance-none transition-colors"
                        >
                            <option value="gdi">{t('workflows.engine_gdi')}</option>
                            <option value="vello">{t('workflows.engine_vello')}</option>
                        </select>
                    </div>
                </div>
                <div className="space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold pl-1">{t('workflows.mode')}</label>
                    <div className="relative group">
                        {workflow.action.type === 'Selection' && <Scan className="absolute left-3 top-3 w-3.5 h-3.5 text-text-muted" />}
                        {workflow.action.type === 'Fullscreen' && <Maximize className="absolute left-3 top-3 w-3.5 h-3.5 text-text-muted" />}
                        {workflow.action.type === 'Window' && <Target className="absolute left-3 top-3 w-3.5 h-3.5 text-text-muted" />}
                        {workflow.action.type === 'Snapshot' && <Camera className="absolute left-3 top-3 w-3.5 h-3.5 text-text-muted" />}

                        <select
                            value={workflow.action.type}
                            onChange={e => updateAction({ type: e.target.value })}
                            className="w-full bg-bg-main border border-border-subtle rounded-xl pl-9 pr-3 py-2.5 text-xs text-text-main outline-none focus:border-accent/50 appearance-none transition-colors"
                        >
                            <option value="Selection">{t('workflows.mode_selection')}</option>
                            <option value="Fullscreen">{t('workflows.mode_fullscreen')}</option>
                            <option value="Window">{t('workflows.mode_window')}</option>
                            <option value="Snapshot">{t('workflows.mode_snapshot')}</option>
                        </select>
                    </div>
                </div>
            </div>

            {/* Snapshot Sub-settings */}
            {workflow.action.type === 'Snapshot' && (
                <motion.div
                    initial={{ opacity: 0, scale: 0.98 }}
                    animate={{ opacity: 1, scale: 1 }}
                    className="flex gap-4 p-4 bg-accent/5 rounded-2xl border border-accent/10"
                >
                    <div className="flex-1 space-y-1.5">
                        <label className="text-[9px] text-accent uppercase font-black tracking-widest">{t('workflows.width')}</label>
                        <input
                            type="number"
                            value={workflow.action.config.width || 800}
                            onChange={e => updateActionConfig({ width: parseInt(e.target.value) })}
                            className="w-full bg-bg-main/50 border border-accent/20 rounded-lg px-3 py-2 text-xs text-text-main outline-none focus:border-accent"
                        />
                    </div>
                    <div className="flex-1 space-y-1.5">
                        <label className="text-[9px] text-accent uppercase font-black tracking-widest">{t('workflows.height')}</label>
                        <input
                            type="number"
                            value={workflow.action.config.height || 600}
                            onChange={e => updateActionConfig({ height: parseInt(e.target.value) })}
                            className="w-full bg-bg-main/50 border border-accent/20 rounded-lg px-3 py-2 text-xs text-text-main outline-none focus:border-accent"
                        />
                    </div>
                </motion.div>
            )}

            {/* Section 3: Storage & Delivery */}
            <div className="space-y-4 pt-2 border-t border-border-subtle/50">
                <div className="flex items-center justify-between">
                    <label className="text-[10px] text-text-muted uppercase font-bold pl-1">{t('workflows.target_folder')}</label>
                    <button
                        onClick={async () => {
                            const path = await invoke('select_folder');
                            if (path) updateOutput({ target_folder: path as string });
                        }}
                        className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-bg-sidebar border border-border-subtle text-[10px] text-text-muted hover:text-text-main hover:bg-bg-subtle transition-all font-medium"
                    >
                        <FolderOpen className="w-3.5 h-3.5" />
                        {workflow.output.target_folder ? t('workflows.change_folder') : t('workflows.select_folder')}
                    </button>
                </div>

                <div className="px-3 py-2.5 bg-bg-main border border-border-subtle rounded-xl text-[10px] text-text-muted font-mono truncate shadow-inner">
                    {workflow.output.target_folder || save_path || t('workflows.global_default')}
                </div>

                <div className="grid grid-cols-2 gap-6 items-start">
                    {/* Toggle Group */}
                    <div className="space-y-2">
                        <label className="text-[10px] text-text-muted uppercase font-bold pl-1 opacity-50">{t('workflows.destination')}</label>
                        <div className="flex flex-col gap-2.5">
                            <ToggleItem
                                label={t('workflows.copy_to_clipboard')}
                                checked={workflow.output.save_to_clipboard}
                                onChange={(val) => updateOutput({ save_to_clipboard: val })}
                            />
                            <ToggleItem
                                label={t('workflows.save_to_file')}
                                checked={workflow.output.save_to_file}
                                onChange={(val) => updateOutput({ save_to_file: val })}
                            />
                        </div>
                    </div>

                    {/* Naming Policy */}
                    <div className="space-y-1.5">
                        <label className="text-[10px] text-text-muted uppercase font-bold pl-1 opacity-50">{t('workflows.naming')}</label>
                        <div className="space-y-2">
                            <input
                                type="text"
                                value={workflow.output.naming_template}
                                onChange={e => updateOutput({ naming_template: e.target.value })}
                                placeholder={t('workflows.naming_placeholder')}
                                className="w-full bg-bg-main border border-border-subtle rounded-xl px-3 py-2 text-[11px] text-text-main outline-none focus:border-accent/40 font-mono shadow-inner"
                            />
                            <div className="flex items-center gap-2">
                                <span className="text-[10px] text-text-muted whitespace-nowrap">{t('workflows.format')}</span>
                                <div className="flex gap-1 flex-1">
                                    {['png', 'jpg', 'webp'].map(fmt => (
                                        <button
                                            key={fmt}
                                            onClick={() => updateOutput({ format: fmt })}
                                            className={`flex-1 py-1 rounded-md text-[9px] font-black uppercase transition-all border ${workflow.output.format === fmt
                                                ? 'bg-accent/10 border-accent/30 text-accent'
                                                : 'bg-bg-main border-border-subtle text-text-muted hover:border-border-hover'
                                                }`}
                                        >
                                            {fmt}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

const ToggleItem: React.FC<{ label: string; checked: boolean; onChange: (val: boolean) => void }> = ({ label, checked, onChange }) => (
    <label className="flex items-center gap-2.5 cursor-pointer group">
        <div className={`w-4 h-4 rounded-md border transition-all flex items-center justify-center ${checked
            ? 'bg-accent border-accent text-white shadow-[0_0_10px_rgba(59,130,246,0.3)]'
            : 'bg-bg-main border-border-subtle group-hover:border-border-hover shadow-inner'
            }`}>
            {checked && <Check className="w-2.5 h-2.5 stroke-[3px]" />}
        </div>
        <input
            type="checkbox"
            className="hidden"
            checked={checked}
            onChange={e => onChange(e.target.checked)}
        />
        <span className={`text-xs transition-colors ${checked ? 'text-text-main font-medium' : 'text-text-muted group-hover:text-text-main'
            }`}>
            {label}
        </span>
    </label>
);

export default WorkflowForm;
