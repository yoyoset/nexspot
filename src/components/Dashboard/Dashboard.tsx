import React from "react";
import { motion } from "framer-motion";
import { Camera, Zap, Cpu, Plus, Edit3, Layers, Settings, Trash2, AlertCircle, Keyboard, Folder, ExternalLink, Scissors, CheckCircle2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { translateError } from "../../utils/error";
import { useAppStore, Workflow } from "../../store/useAppStore";

interface DashboardProps {
    onNavigateToWorkflows: (id?: string) => void;
}

const Dashboard: React.FC<DashboardProps> = ({ onNavigateToWorkflows }) => {
    const { config, velloStatus, velloError, removeWorkflow, openFolder } = useConfig();
    const { t } = useTranslation();

    const workflows = config?.workflows || [];

    const getLocalizedLabel = React.useCallback((label: string, id?: string) => {
        if (id) {
            const idKey = `workflows.presets.${id.replace('_default', '')}`;
            const idTranslated = t(idKey);
            if (idTranslated !== idKey) return idTranslated;
        }
        const key = `workflows.presets.${label.toLowerCase().replace(/\s+/g, '_')}`;
        const translated = t(key);
        return translated === key ? label : translated;
    }, [t]);

    const hotkeyStatus = React.useMemo(() => {
        const counts: Record<string, number> = {};
        const registrations = config?.registration_errors || [];
        workflows.forEach((w: Workflow) => {
            if (w.shortcut) {
                counts[w.shortcut] = (counts[w.shortcut] || 0) + 1;
            }
        });
        return { counts, registrations };
    }, [workflows, config?.registration_errors]);

    const handleCreateNew = () => onNavigateToWorkflows('new');
    const handleEdit = (id: string) => onNavigateToWorkflows(id);

    return (
        <div className="w-full h-full bg-bg-main custom-scrollbar overflow-y-auto select-none relative">
            <div className="noise-bg" />
            <div className="max-w-screen-lg mx-auto px-6 py-4 flex flex-col gap-2 min-h-full relative z-10">
            
            {/* Header: Industrial Console Header */}
            <div className="flex items-center justify-between border-b border-border-subtle pb-2 mb-2">
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-accent rounded-full animate-breathing shadow-[0_0_8px_var(--color-accent)]" />
                    <h1 className="tech-text text-[11px] font-bold uppercase tracking-[0.2em] text-text-main opacity-90">
                        {t('dashboard.title')} <span className="opacity-20 mx-1">//</span> <span className="text-accent">{t('common.status_active')}</span>
                    </h1>
                </div>
                <button
                    onClick={handleCreateNew}
                    className="flex items-center gap-1.5 px-2.5 py-1 rounded-sm bg-accent/10 hover:bg-accent/20 border border-accent/20 tech-text text-[9px] font-bold uppercase tracking-wider text-accent transition-all"
                >
                    <Plus className="w-3 h-3" /> {t('dashboard.new')}
                </button>
            </div>

            {/* List Header */}
            <div className="grid grid-cols-[1.2fr_1.8fr_60px_80px_40px_100px_80px] gap-2 px-3 py-1.5 bg-bg-subtle/30 border border-border-subtle rounded-sm tech-text text-[9px] font-bold uppercase text-text-muted/50 tracking-widest">
                <div className="pl-4">{t('dashboard.column_identifier')}</div>
                <div>{t('dashboard.column_spec')}</div>
                <div>{t('dashboard.column_ext')}</div>
                <div className="text-center">STAT</div>
                <div className="text-center">DIR</div>
                <div>{t('dashboard.column_bind')}</div>
                <div className="text-right">OP</div>
            </div>

            {/* List Content */}
            <div className="flex flex-col gap-1 mt-1 transition-all">
                {workflows.map((w: Workflow) => {
                    const engine = w.action.config.engine.toLowerCase();
                    const engineStatus = engine === 'vello' ? velloStatus : 'ready';
                    const hasHotkeyConflict = (hotkeyStatus.counts[w.shortcut] || 0) > 1;
                    const hasRegError = hotkeyStatus.registrations.some((e: string) => e === w.id || e.startsWith(w.id + ":"));
                    const isSystem = w.is_system;

                    return (
                        <div 
                            key={w.id}
                            className={`grid grid-cols-[1.2fr_1.8fr_60px_80px_40px_100px_80px] gap-2 items-center px-4 py-2 border transition-all group relative overflow-hidden ${
                                !w.enabled ? 'opacity-40 grayscale-[0.5] bg-black/20 border-white/5' :
                                hasHotkeyConflict || hasRegError 
                                ? 'bg-red-500/[0.04] border-red-500/20 shadow-[inset_0_0_20px_rgba(239,68,68,0.05)]' 
                                : 'bg-bg-card/40 border-border-subtle hover:border-border-hover hover:bg-white/[0.015] shadow-sm'
                            }`}
                        >
                            {/* Status Rail */}
                            <div className={`absolute left-0 top-0 bottom-0 w-0.5 transition-all ${
                                hasHotkeyConflict || hasRegError ? 'bg-red-500' : 
                                engine === 'vello' ? 'bg-amber-500/40' : 'bg-blue-500/40'
                            }`} />
                            {/* Identifier */}
                            <div className="flex items-center gap-2 overflow-hidden">
                                {/* Status Dot */}
                                <div className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                                    !w.enabled ? 'bg-text-muted/30' :
                                    engine === 'vello' ? 'bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.4)]' : 'bg-blue-500 shadow-[0_0_6px_rgba(59,130,246,0.4)]'
                                }`} />
                                
                                {isSystem ? <Layers className="w-3 h-3 opacity-30 text-accent" /> : <Scissors className="w-3 h-3 opacity-30" />}
                                <span className={`text-[12px] font-bold truncate ${!w.enabled ? 'text-text-muted' : hasHotkeyConflict || hasRegError ? 'text-red-400' : 'text-text-main'}`}>
                                    {getLocalizedLabel(w.label, w.id)}
                                </span>
                            </div>

                            {/* Spec */}
                            <div className="flex items-center gap-1.5 tech-text text-[10px] uppercase font-bold text-text-muted/40">
                                <span className={engine === 'vello' ? 'text-amber-500/50' : 'text-blue-500/50'}>{engine}</span>
                                <span className="opacity-20">/</span>
                                <span className="whitespace-nowrap">{t(`workflows.mode_${w.action.type.toLowerCase()}`)}</span>
                            </div>

                            {/* Ext */}
                            <div className="tech-text text-[10px] font-bold text-text-muted/40">
                                <span className="tech-badge">{w.output.format.toUpperCase()}</span>
                            </div>

                            {/* Stat (Split Hotkey / Engine) */}
                            <div className="flex items-center justify-center gap-2">
                                {/* Hotkey Stat */}
                                <div 
                                    className={`p-1 rounded-sm border ${
                                        hasHotkeyConflict || hasRegError 
                                        ? 'border-red-500/30 text-red-500' 
                                        : 'border-white/5 text-text-muted/20 group-hover:text-text-muted/40'
                                    }`}
                                    title={hasHotkeyConflict ? t('dashboard.conflict') : hasRegError ? t('dashboard.error') : 'Hotkey Ready'}
                                >
                                    <Keyboard className="w-3 h-3" />
                                </div>
                                {/* Engine Stat */}
                                <div 
                                    className={`p-1 rounded-sm border ${
                                        engineStatus === 'ready' 
                                        ? 'border-white/5 text-emerald-500/30 group-hover:text-emerald-500/60' 
                                        : engineStatus === 'pending'
                                        ? 'border-amber-500/20 text-amber-500 animate-pulse'
                                        : 'border-red-500/30 text-red-500'
                                    }`}
                                    title={engineStatus === 'failed' ? velloError || 'Engine Error' : 'Engine Ready'}
                                >
                                    <Cpu className="w-3 h-3" />
                                </div>
                            </div>

                            {/* Dir */}
                            <div className="flex justify-center">
                                <button
                                    onClick={(e) => { e.stopPropagation(); if(w.output.target_folder) openFolder(w.output.target_folder); }}
                                    className="p-1 rounded-sm hover:bg-white/5 border border-transparent hover:border-white/10 text-text-muted/20 hover:text-text-main transition-all"
                                    title={w.output.target_folder || t('dashboard.no_folder')}
                                >
                                    <Folder className="w-3 h-3" />
                                </button>
                            </div>

                            {/* Bind (Hotkey) */}
                            <div className="tech-text text-[11px] font-bold tracking-tight">
                                {w.shortcut ? (
                                    <span className={hasHotkeyConflict || hasRegError ? 'text-red-400 opacity-80' : 'text-accent opacity-80'}>
                                        {w.shortcut}
                                    </span>
                                ) : (
                                    <span className="text-text-muted/20 italic font-normal text-[9px]">unbound</span>
                                )}
                            </div>

                            {/* Op (Actions) */}
                            <div className="flex items-center justify-end gap-1">
                                <button
                                    onClick={async (e) => { 
                                        e.stopPropagation(); 
                                        try {
                                            await invoke('trigger_capture', { action: w.id });
                                        } catch (err) {
                                            console.error("Manual trigger failed:", err);
                                            const message = translateError(err, t);
                                            useAppStore.getState().showHUD(message, 'error');
                                        }
                                    }}
                                    className="px-2 py-1 rounded-sm bg-accent/10 hover:bg-accent hover:text-white border border-accent/20 hover:border-accent transition-all duration-200 group/zap flex items-center justify-center gap-1.5"
                                    title={t('dashboard.capture_btn')}
                                >
                                    <Zap className="w-3 h-3 transition-transform group-hover/zap:scale-110" />
                                    <span className="text-[8px] tech-text font-black tracking-tighter hidden group-hover/zap:inline-block">ZAP</span>
                                </button>
                                <button
                                    onClick={(e) => { e.stopPropagation(); handleEdit(w.id); }}
                                    className="p-1 rounded-sm hover:bg-white/5 text-text-muted/20 hover:text-text-main transition-all"
                                >
                                    <Edit3 className="w-3 h-3" />
                                </button>
                                {!isSystem && (
                                    <button
                                        onClick={async (e) => { 
                                            e.stopPropagation(); 
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                try {
                                                    await removeWorkflow(w.id);
                                                    useAppStore.getState().showHUD(t('hud.saved'), 'success');
                                                } catch (err) {
                                                    console.error("Failed to remove workflow:", err);
                                                    const message = translateError(err, t);
                                                    useAppStore.getState().showHUD(message, 'error');
                                                }
                                            }
                                        }}
                                        className="p-1 rounded-sm hover:bg-red-500/10 text-text-muted/10 hover:text-red-500 transition-all"
                                    >
                                        <Trash2 className="w-3 h-3" />
                                    </button>
                                )}
                            </div>
                        </div>
                    );
                })}
                {workflows.length === 0 && (
                    <div className="py-20 flex flex-col items-center justify-center border border-dashed border-border-subtle rounded-sm bg-white/[0.01]">
                        <Zap className="w-6 h-6 text-text-muted/10 mb-2" />
                        <span className="tech-text text-[9px] font-bold tracking-widest uppercase text-text-muted/30">{t('workflows.empty')}</span>
                    </div>
                )}
            </div>

            {/* System Status Footer */}
            <div className="mt-auto pt-4 flex items-center justify-between opacity-30 grayscale hover:grayscale-0 transition-all border-t border-white/5 mx-1">
                <div className="flex items-center gap-4 tech-text text-[9px] font-bold uppercase tracking-wider text-text-muted">
                    <div className="flex items-center gap-1.5">
                        <CheckCircle2 className="w-3 h-3 text-emerald-500" />
                        <span>Core_Kernel: Ready</span>
                    </div>
                    <div className="flex items-center gap-1.5 border-l border-white/10 pl-4">
                        <div className={`w-2 h-2 rounded-full ${velloStatus === 'ready' ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]' : 'bg-text-muted/20'}`} />
                        <span>{t('common.kernel_ready')} // VELLO: {velloStatus.toUpperCase()}</span>
                    </div>
                </div>
                <div className="tech-text text-[9px] font-bold text-text-muted/40 italic">
                    {t('common.version_brand')}
                </div>
            </div>
        </div>
    </div>
);
};

export default Dashboard;
