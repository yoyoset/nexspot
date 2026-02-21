import React from "react";
import { motion } from "framer-motion";
import { Camera, Zap, Sparkles, Plus, Edit3, Layers, Settings, Trash2, AlertCircle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { useAppStore, Workflow } from "../../store/useAppStore";
import AISection from "./AISection";

interface DashboardProps {
    onNavigateToWorkflows: (id?: string) => void;
}

const Dashboard: React.FC<DashboardProps> = ({ onNavigateToWorkflows }) => {
    const { config, isVelloReady, removeWorkflow } = useConfig();
    const { t } = useTranslation();
    const [aiCreateTrigger, setAiCreateTrigger] = React.useState<(() => void) | null>(null);

    const workflows = config?.workflows || [];
    const aiShortcuts = config?.ai_shortcuts || [];

    // Helper to detect shortcut conflicts locally (Global: Workflows + AI)
    const hotkeyStatus = React.useMemo(() => {
        const counts: Record<string, number> = {};
        const errors = config?.registration_errors || [];

        [...workflows, ...aiShortcuts].forEach((w: any) => {
            if (w.shortcut) {
                counts[w.shortcut] = (counts[w.shortcut] || 0) + 1;
            }
        });

        return {
            counts,
            errors
        };
    }, [workflows, aiShortcuts, config?.registration_errors]);

    const { systemWorkflows, userWorkflows } = React.useMemo(() => {
        return {
            systemWorkflows: workflows.filter((w: Workflow) => w.is_system),
            userWorkflows: workflows.filter((w: Workflow) => !w.is_system)
        };
    }, [workflows]);

    const regionWorkflow = systemWorkflows.find(w => w.id === 'capture_default');
    const snapshotWorkflow = systemWorkflows.find(w => w.id === 'snapshot_default');

    const handleCreateNew = () => {
        onNavigateToWorkflows('new');
    };

    const handleEdit = (id: string) => {
        onNavigateToWorkflows(id);
    };

    const getLocalizedLabel = (label: string) => {
        const key = `workflows.presets.${label.toLowerCase().replace(/\s+/g, '_')}`;
        const translated = t(key);
        return translated === key ? label : translated;
    };

    const getModeInfo = (type: string) => {
        switch (type) {
            case 'Fullscreen': return { icon: <Layers className="w-5 h-5" />, label: t('workflows.mode_fullscreen') };
            case 'Snapshot': return { icon: <Zap className="w-5 h-5" />, label: t('workflows.mode_snapshot') };
            case 'Window': return { icon: <Camera className="w-5 h-5" />, label: t('workflows.mode_window') };
            default: return { icon: <Camera className="w-5 h-5" />, label: t('workflows.mode_selection') };
        }
    };

    return (
        <div className="w-full h-full bg-bg-main flex flex-col p-6 custom-scrollbar overflow-y-auto select-none relative gap-6">

            {/* 1. PRIMARY ACTIONS (ULTRA COMPACT HERO) */}
            <div className="grid grid-cols-2 gap-4">
                {regionWorkflow && (
                    <PresetCard
                        workflow={regionWorkflow}
                        isVelloReady={isVelloReady}
                        hasConflict={(hotkeyStatus.counts[regionWorkflow.shortcut] || 0) > 1}
                        hasError={hotkeyStatus.errors.some((e: string) => e.startsWith(regionWorkflow.id + ":"))}
                        onEdit={() => handleEdit(regionWorkflow.id)}
                        onTrigger={() => invoke('trigger_capture', { action: regionWorkflow.id })}
                        t={t}
                        getModeInfo={getModeInfo}
                        getLocalizedLabel={getLocalizedLabel}
                    />
                )}
                {snapshotWorkflow && (
                    <PresetCard
                        workflow={snapshotWorkflow}
                        isVelloReady={isVelloReady}
                        hasConflict={(hotkeyStatus.counts[snapshotWorkflow.shortcut] || 0) > 1}
                        hasError={hotkeyStatus.errors.some((e: string) => e.startsWith(snapshotWorkflow.id + ":"))}
                        onEdit={() => handleEdit(snapshotWorkflow.id)}
                        onTrigger={() => invoke('trigger_capture', { action: snapshotWorkflow.id })}
                        t={t}
                        getModeInfo={getModeInfo}
                        getLocalizedLabel={getLocalizedLabel}
                    />
                )}
            </div>

            {/* 2. SECONDARY COLLECTIONS (TIGHTER SPACING) */}
            <div className="flex flex-col gap-5">

                {/* PRESETS LIST */}
                <CollectionContainer
                    title={t('workflows.other_presets')}
                    icon={<Settings className="w-4 h-4" />}
                    action={
                        <button
                            onClick={handleCreateNew}
                            className="flex items-center gap-2 px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-[10px] font-black uppercase tracking-widest text-text-muted transition-all border border-white/5"
                        >
                            <Plus className="w-3.5 h-3.5" /> {t('dashboard.new')}
                        </button>
                    }
                >
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                        {userWorkflows.map((w: Workflow) => (
                            <ListTile
                                key={w.id}
                                icon={getModeInfo(w.action.type).icon}
                                label={getLocalizedLabel(w.label)}
                                shortcut={w.shortcut}
                                hasConflict={(hotkeyStatus.counts[w.shortcut] || 0) > 1}
                                hasError={hotkeyStatus.errors.some((e: string) => e.startsWith(w.id + ":"))}
                                onClick={() => invoke('trigger_capture', { action: w.id })}
                                onEdit={() => handleEdit(w.id)}
                                onRemove={() => removeWorkflow(w.id)}
                            />
                        ))}
                        {userWorkflows.length === 0 && (
                            <div className="col-span-full py-6 flex items-center justify-center border border-dashed border-white/5 rounded-2xl opacity-10">
                                <span className="text-[10px] font-black tracking-widest uppercase">{t('workflows.empty')}</span>
                            </div>
                        )}
                    </div>
                </CollectionContainer>

                {/* AI SHORTCUTS */}
                <CollectionContainer
                    title={t('ai.title')}
                    icon={<Sparkles className="w-4 h-4 text-purple-400" />}
                    action={
                        <button
                            onClick={() => aiCreateTrigger?.()}
                            className="flex items-center gap-2 px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-[10px] font-black uppercase tracking-widest text-text-muted transition-all border border-white/5"
                        >
                            <Plus className="w-3.5 h-3.5" /> {t('dashboard.new')}
                        </button>
                    }
                >
                    <AISection onCreateTrigger={setAiCreateTrigger} isCompact />
                </CollectionContainer>
            </div>
        </div>
    );
};

// --- Sub-Components ---

const CollectionContainer: React.FC<{
    title: string;
    icon: React.ReactNode;
    action?: React.ReactNode;
    children: React.ReactNode;
}> = ({ title, icon, action, children }) => {
    return (
        <div className="flex flex-col gap-3 px-1">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="text-text-muted opacity-30">
                        {icon}
                    </div>
                    <span className="text-[10px] font-black text-text-muted/60 uppercase tracking-[0.2em]">
                        {title}
                    </span>
                </div>
                {action}
            </div>

            <div className="bg-bg-subtle/20 border border-white/5 rounded-[20px] p-4">
                {children}
            </div>
        </div>
    );
};

const ListTile: React.FC<{
    icon: React.ReactNode;
    label: string;
    shortcut?: string;
    hasConflict?: boolean;
    hasError?: boolean;
    onClick: () => void;
    onEdit: () => void;
    onRemove?: () => void;
}> = ({ icon, label, shortcut, hasConflict, hasError, onClick, onEdit, onRemove }) => {
    return (
        <motion.div
            whileHover={{ x: 2 }}
            whileTap={{ scale: 0.98 }}
            onClick={onClick}
            className={`group/tile flex items-center justify-between p-3.5 rounded-xl bg-white/[0.02] border transition-all cursor-pointer overflow-hidden ${hasConflict || hasError ? 'border-red-500/20 bg-red-500/5' : 'border-white/5 hover:bg-white/[0.05] hover:border-white/10'}`}
        >
            <div className="flex items-center gap-3 overflow-hidden">
                <div className={`p-2 bg-white/5 rounded-lg transition-all shrink-0 ${hasConflict || hasError ? 'text-red-400' : 'text-text-muted'}`}>
                    <div className="w-3.5 h-3.5 flex items-center justify-center">
                        {icon}
                    </div>
                </div>
                <div className="flex flex-col overflow-hidden">
                    <span className="text-[13px] font-bold text-text-main/90 truncate tracking-tight leading-none mb-1">{label}</span>
                    <div className="flex items-center gap-2">
                        {shortcut && (
                            <span className={`text-[9px] font-mono font-black uppercase tracking-tighter ${hasConflict || hasError ? 'text-red-400/60' : 'text-text-muted/30'}`}>{shortcut}</span>
                        )}
                        {(hasConflict || hasError) && <AlertCircle className="w-2.5 h-2.5 text-red-400 anim-pulse" />}
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-1 opacity-0 group-hover/tile:opacity-100 transition-all ml-2 shrink-0">
                <button
                    onClick={(e) => { e.stopPropagation(); onEdit(); }}
                    className="p-1.5 rounded-lg hover:bg-white/10 text-text-muted hover:text-text-main transition-all"
                >
                    <Edit3 className="w-3.5 h-3.5" />
                </button>
                {onRemove && (
                    <button
                        onClick={(e) => { e.stopPropagation(); onRemove(); }}
                        className="p-1.5 rounded-lg hover:bg-red-500/10 text-red-500/20 hover:text-red-500 transition-all"
                    >
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                )}
            </div>
        </motion.div>
    );
};

const PresetCard: React.FC<{
    workflow: Workflow;
    isVelloReady: boolean;
    hasConflict?: boolean;
    hasError?: boolean;
    onEdit: () => void;
    onTrigger: () => void;
    t: any;
    getModeInfo: (type: string) => any;
    getLocalizedLabel: (label: string) => string;
}> = ({ workflow, isVelloReady, hasConflict, hasError, onEdit, onTrigger, t, getModeInfo, getLocalizedLabel }) => {
    const modeInfo = getModeInfo(workflow.action.type);
    const engine = workflow.action.config.engine.toLowerCase();

    // Status dot logic: 
    // GDI is always green (ready)
    // Vello is yellow if buffering (!isVelloReady), green when ready.
    const isEngineReady = engine === 'vello' ? isVelloReady : true;

    return (
        <motion.div
            whileHover={{ scale: 1.005 }}
            whileTap={{ scale: 0.98 }}
            onClick={onTrigger}
            className={`bg-bg-subtle/30 border rounded-[24px] p-5 flex flex-col gap-4 group transition-all relative overflow-hidden ${hasConflict || hasError ? 'border-red-500/30 shadow-[0_0_20px_rgba(239,68,68,0.1)]' : 'border-white/5 hover:bg-white/[0.02]'}`}
        >
            <div className="flex items-center justify-between relative z-10 w-full">
                <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg border border-white/5 shadow-inner ${hasConflict || hasError ? 'bg-red-500/10 text-red-400' : 'bg-accent/10 text-accent'}`}>
                        <div className="w-4 h-4 flex items-center justify-center">
                            {modeInfo.icon}
                        </div>
                    </div>
                    <div className="flex flex-col">
                        <div className="text-[14px] font-black text-text-main tracking-tight leading-none mb-1">
                            {getLocalizedLabel(workflow.label)}
                        </div>
                        <span className="text-[8px] text-text-muted/40 uppercase font-black tracking-widest">{modeInfo.label}</span>
                    </div>
                </div>

                <button
                    onClick={(e) => { e.stopPropagation(); onEdit(); }}
                    className="p-1.5 text-text-muted/30 hover:text-text-main transition-all"
                >
                    <Edit3 className="w-3.5 h-3.5" />
                </button>
            </div>

            <div className="mt-auto flex flex-col gap-1.5 relative z-10">
                <div className="flex items-center gap-2">
                    <div className={`text-[16px] font-black font-mono tracking-tighter leading-none ${hasConflict || hasError ? 'text-red-400' : 'text-text-accent'}`}>
                        {workflow.shortcut}
                    </div>
                    {workflow.enabled && !hasConflict && !hasError && (
                        <div className="flex items-center gap-1 text-[8px] text-emerald-400 font-black uppercase bg-emerald-500/5 px-1.5 py-0.5 rounded-full border border-emerald-500/10">
                            <div className="w-1 h-1 rounded-full bg-emerald-400" />
                            ACTIVE
                        </div>
                    )}
                    {(hasConflict || hasError) && (
                        <div className="flex items-center gap-1 text-[8px] text-red-400 font-black uppercase bg-red-500/5 px-1.5 py-0.5 rounded-full border border-red-500/10">
                            <AlertCircle className="w-2.5 h-2.5" />
                            {hasError ? 'ERROR' : 'CONFLICT'}
                        </div>
                    )}
                </div>

                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 text-[9px] font-black text-text-muted/20 uppercase tracking-[0.1em]">
                        <span className="text-text-main/20">{workflow.output.format}</span>
                        <div className="flex items-center gap-1.5">
                            <div className={`w-1 h-1 rounded-full animate-pulse ${isEngineReady ? 'bg-emerald-400' : 'bg-amber-400'}`} />
                            <span className={`transition-colors ${engine === 'vello' ? (isEngineReady ? 'text-amber-400/60' : 'text-amber-200/40') : 'text-blue-400/40'}`}>
                                {engine.toUpperCase()}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div className={`absolute top-0 left-0 w-full h-0.5 transition-all ${hasConflict ? 'bg-red-500/40' : 'bg-accent/0 group-hover:bg-accent/20'}`} />
        </motion.div>
    );
};

export default Dashboard;
