import React from "react";
import { Cpu, FolderOpen, Check, Camera, Scan, Maximize, Target, Hash } from "lucide-react";
import ShortcutRecorder from "../Settings/ShortcutRecorder";
import { motion, AnimatePresence } from "framer-motion";
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
        <div className="space-y-2.5">
            {/* Row 1: Active + Identity + Shortcut */}
            <div className="grid grid-cols-[auto_1fr_180px] gap-3 items-end p-2 bg-accent/5 rounded-sm border border-accent/10 shadow-inner">
                <div className="flex flex-col gap-1.5 pb-0.5">
                    <label className="text-[9px] text-accent uppercase font-black tech-text tracking-widest">{t('workflows.active')}</label>
                    <ToggleItem
                        label=""
                        checked={workflow.enabled}
                        onChange={(val) => !workflow.is_system && onChange({ ...workflow, enabled: val })}
                        disabled={workflow.is_system}
                    />
                </div>
                <div className="space-y-1.5 px-2 border-l border-white/5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1">{t('workflows.label')}</label>
                    <input
                        type="text"
                        value={workflow.label}
                        onChange={e => onChange({ ...workflow, label: e.target.value })}
                        className="w-full h-8 bg-bg-main/50 border border-border-subtle rounded-sm px-3 text-[11px] text-text-main tech-text outline-none focus:border-accent/40 transition-colors"
                        placeholder="Protocol Name"
                    />
                </div>
                <div className="space-y-1.5 px-2 border-l border-white/5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1">{t('workflows.shortcut')}</label>
                    <ShortcutRecorder
                        value={workflow.shortcut}
                        onChange={(k) => onChange({ ...workflow, shortcut: k })}
                        placeholder={t('common.none')}
                        status={workflow.shortcut ? 'success' : 'neutral'}
                        className="w-full h-8"
                    />
                </div>
            </div>

            {/* Row 2: Engine + Mode + Snapshot Settings */}
            <div className="grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] gap-3 p-2 bg-black/20 border border-white/5 rounded-sm">
                <div className="space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1">{t('workflows.engine')}</label>
                    <div className="relative group">
                        <Cpu className="absolute left-2.5 top-2 w-3.5 h-3.5 text-text-muted opacity-40 group-focus-within:text-accent group-focus-within:opacity-100 transition-all" />
                        <select
                            value={workflow.action.config.engine}
                            onChange={e => updateActionConfig({ engine: e.target.value })}
                            className="w-full h-8 bg-bg-main/30 border border-border-subtle rounded-sm pl-8 pr-2 text-[11px] text-text-main tech-text outline-none focus:border-accent/40 transition-colors appearance-none"
                        >
                            <option value="gdi">{t('workflows.engine_gdi')}</option>
                            <option value="vello">{t('workflows.engine_vello')}</option>
                        </select>
                    </div>
                </div>

                <div className="space-y-1.5 border-l border-white/5 pl-3">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1">{t('workflows.mode')}</label>
                    <div className="relative group">
                        {workflow.action.type === 'Selection' && <Scan className="absolute left-2.5 top-2 w-3.5 h-3.5 text-text-muted opacity-40" />}
                        {workflow.action.type === 'Fullscreen' && <Maximize className="absolute left-2.5 top-2 w-3.5 h-3.5 text-text-muted opacity-40" />}
                        {workflow.action.type === 'Window' && <Target className="absolute left-2.5 top-2 w-3.5 h-3.5 text-text-muted opacity-40" />}
                        {workflow.action.type === 'Snapshot' && <Camera className="absolute left-2.5 top-2 w-3.5 h-3.5 text-text-muted opacity-40" />}

                        <select
                            value={workflow.action.type}
                            onChange={e => updateAction({ type: e.target.value })}
                            className="w-full h-8 bg-bg-main/30 border border-border-subtle rounded-sm pl-8 pr-2 text-[11px] text-text-main tech-text outline-none focus:border-accent/40 transition-colors appearance-none"
                        >
                            <option value="Selection">{t('workflows.mode_selection')}</option>
                            <option value="Fullscreen">{t('workflows.mode_fullscreen')}</option>
                            <option value="Window">{t('workflows.mode_window')}</option>
                            <option value="Snapshot">{t('workflows.mode_snapshot')}</option>
                        </select>
                    </div>
                </div>

                {workflow.action.type === 'Snapshot' && (
                    <div className="col-span-1 lg:col-span-1 flex gap-2 border-l border-white/5 pl-3">
                        <div className="flex-1 space-y-1.5">
                            <label className="text-[10px] text-accent/60 uppercase font-black tech-text tracking-widest flex items-center gap-1">
                                <Hash className="w-2.5 h-2.5" /> {t('workflows.width')}
                            </label>
                            <input
                                type="number"
                                value={workflow.action.config.width || 800}
                                onChange={e => updateActionConfig({ width: parseInt(e.target.value) })}
                                className="w-full h-8 bg-accent/5 border border-accent/20 rounded-sm px-2 text-[11px] text-text-main tech-text outline-none focus:border-accent"
                            />
                        </div>
                        <div className="flex-1 space-y-1.5">
                            <label className="text-[10px] text-accent/60 uppercase font-black tech-text tracking-widest flex items-center gap-1">
                                <Hash className="w-2.5 h-2.5" /> {t('workflows.height')}
                            </label>
                            <input
                                type="number"
                                value={workflow.action.config.height || 600}
                                onChange={e => updateActionConfig({ height: parseInt(e.target.value) })}
                                className="w-full h-8 bg-accent/5 border border-accent/20 rounded-sm px-2 text-[11px] text-text-main tech-text outline-none focus:border-accent"
                            />
                        </div>
                    </div>
                )}
            </div>

            {/* Row 3: Target Folder (Compact) */}
            <div className="flex items-center gap-4 p-2 bg-black/20 border border-white/5 rounded-sm">
                <div className="flex-1 space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1 opacity-50">{t('workflows.target_folder')}</label>
                    <div className="flex gap-2">
                        <div className="flex-1 h-8 bg-black/40 border border-border-subtle rounded-sm px-3 flex items-center text-[10px] text-text-muted tech-text font-mono truncate shadow-inner">
                            {workflow.output.target_folder || save_path || t('workflows.global_default')}
                        </div>
                        <button
                            onClick={async () => {
                                const path = await invoke('select_folder');
                                if (path) updateOutput({ target_folder: path as string });
                            }}
                            className="h-8 px-3 rounded-sm bg-bg-sidebar border border-border-subtle text-[10px] text-text-muted tech-text hover:text-white hover:bg-accent/40 hover:border-accent/50 transition-all font-bold uppercase tracking-widest flex items-center gap-2"
                        >
                            <FolderOpen className="w-3.5 h-3.5" />
                            {t('common.select')}
                        </button>
                    </div>
                </div>
            </div>

            {/* Row 4: Naming + Format + Distribution */}
            <div className="grid grid-cols-[1fr_auto_auto] gap-4 p-2 bg-black/20 border border-white/5 rounded-sm items-end">
                {/* Naming */}
                <div className="space-y-1.5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest pl-1 opacity-50">{t('workflows.naming')}</label>
                    <input
                        type="text"
                        value={workflow.output.naming_template}
                        onChange={e => updateOutput({ naming_template: e.target.value })}
                        placeholder={t('workflows.naming_placeholder')}
                        className="w-full h-8 bg-bg-main/30 border border-border-subtle rounded-sm px-3 text-[11px] text-text-main tech-text outline-none focus:border-accent/40 font-mono shadow-inner uppercase"
                    />
                </div>

                {/* Format */}
                <div className="space-y-1.5 px-3 border-l border-white/5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest opacity-50 text-center block mb-1">{t('workflows.format')}</label>
                    <div className="flex gap-1">
                        {['png', 'jpg', 'webp'].map(fmt => (
                            <button
                                key={fmt}
                                onClick={() => updateOutput({ format: fmt })}
                                className={`w-12 h-7 rounded-sm text-[9px] tech-text font-black uppercase transition-all border ${workflow.output.format === fmt
                                    ? 'bg-accent/20 border-accent/50 text-accent shadow-[0_0_10px_rgba(59,130,246,0.2)]'
                                    : 'bg-bg-main/30 border-border-subtle text-text-muted hover:border-border-hover'
                                    }`}
                            >
                                {fmt}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Distribution */}
                <div className="space-y-1.5 px-3 border-l border-white/5">
                    <label className="text-[10px] text-text-muted uppercase font-bold tech-text tracking-widest opacity-50 text-center block mb-1">{t('workflows.destination')}</label>
                    <div className="flex gap-2">
                        <ToggleItemHighlight
                            label={t('workflows.copy_to_clipboard')}
                            checked={workflow.output.save_to_clipboard}
                            onChange={(val) => updateOutput({ save_to_clipboard: val })}
                        />
                        <ToggleItemHighlight
                            label={t('workflows.save_to_file')}
                            checked={workflow.output.save_to_file}
                            onChange={(val) => updateOutput({ save_to_file: val })}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};

const ToggleItem: React.FC<{ label: string; checked: boolean; onChange: (val: boolean) => void; disabled?: boolean }> = ({ label, checked, onChange, disabled }) => (
    <label className={`flex items-center gap-2 transition-opacity ${disabled ? 'cursor-not-allowed opacity-40' : 'cursor-pointer group'}`}>
        <div className={`w-6 h-3.5 rounded-full border transition-all relative ${checked
            ? 'bg-accent border-accent shadow-[0_0_10px_rgba(59,130,246,0.3)]'
            : 'bg-bg-main border-border-subtle group-hover:border-border-hover shadow-inner'
            }`}>
            <div className={`absolute top-0.5 w-2 h-2 rounded-full transition-all ${checked ? 'left-3 bg-white' : 'left-0.5 bg-text-muted'}`} />
        </div>
        <input
            type="checkbox"
            className="hidden"
            checked={checked}
            onChange={e => !disabled && onChange(e.target.checked)}
            disabled={disabled}
        />
        {label && (
           <span className={`text-[10px] tech-text uppercase tracking-widest transition-colors ${checked ? 'text-text-main font-bold' : 'text-text-muted group-hover:text-text-main'}`}>
               {label}
           </span>
        )}
    </label>
);

const ToggleItemHighlight: React.FC<{ label: string; checked: boolean; onChange: (val: boolean) => void }> = ({ label, checked, onChange }) => (
    <button
        onClick={() => onChange(!checked)}
        className={`h-7 px-3 rounded-sm text-[9px] tech-text font-black uppercase transition-all border flex items-center gap-2 ${checked
            ? 'bg-accent/10 border-accent/40 text-accent shadow-[inset_0_0_10px_rgba(59,130,246,0.1)]'
            : 'bg-bg-main/30 border-border-subtle text-text-muted opacity-60 grayscale hover:grayscale-0 hover:opacity-100 hover:border-border-hover'
            }`}
    >
        <div className={`w-1.5 h-1.5 rounded-full ${checked ? 'bg-accent shadow-[0_0_5px_rgba(59,130,246,1)]' : 'bg-text-muted/30'}`} />
        {label}
    </button>
);

export default WorkflowForm;
