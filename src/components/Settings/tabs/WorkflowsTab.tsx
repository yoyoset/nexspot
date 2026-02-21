import React, { useEffect, useMemo } from "react";
import { Plus, Trash2, Settings2, ShieldCheck, Keyboard, Cpu, X, Check, Sparkles, Layers } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useAppStore, Workflow } from "../../../store/useAppStore";

interface WorkflowsTabProps {
    config: any;
    removeWorkflow: (id: string) => Promise<any>;
    initialWorkflowId?: string | null;
}

const WorkflowsTab: React.FC<WorkflowsTabProps> = ({ config, removeWorkflow, initialWorkflowId }) => {
    const { t } = useTranslation();
    const setWorkflowEditing = useAppStore(state => state.setWorkflowEditing);
    const workflows = config?.workflows || [];

    // Split workflows into system and user categories
    const { systemWorkflows, userWorkflows } = useMemo(() => {
        return {
            systemWorkflows: workflows.filter((w: Workflow) => w.is_system),
            userWorkflows: workflows.filter((w: Workflow) => !w.is_system)
        };
    }, [workflows]);

    const handleEdit = (w: Workflow) => {
        setWorkflowEditing(true, w);
    };

    const getLocalizedLabel = (label: string) => {
        const key = `workflows.presets.${label.toLowerCase().replace(/\s+/g, '_')}`;
        const translated = t(key);
        return translated === key ? label : translated;
    };

    const handleAdd = () => {
        const newWorkflow: Workflow = {
            id: `user_${Date.now()}`,
            label: t('workflows.new'),
            shortcut: "Alt+F1",
            enabled: true,
            is_system: false,
            action: { type: 'Selection', config: { engine: 'gdi' } },
            output: {
                save_to_file: true,
                save_to_clipboard: true,
                target_folder: null,
                naming_template: "capture_%Y%m%d_%H%M%S",
                format: "png"
            }
        };
        setWorkflowEditing(true, newWorkflow);
    };

    useEffect(() => {
        if (initialWorkflowId === 'new') {
            handleAdd();
        } else if (initialWorkflowId && initialWorkflowId !== 'none') {
            const workflow = workflows.find((w: Workflow) => w.id === initialWorkflowId);
            if (workflow) {
                handleEdit(workflow);
                setTimeout(() => {
                    document.getElementById(`workflow-${initialWorkflowId}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' });
                }, 100);
            }
        }
    }, [initialWorkflowId]);

    return (
        <motion.div
            initial={{ opacity: 0, x: 5 }}
            animate={{ opacity: 1, x: 0 }}
            className="space-y-6 pb-6"
        >
            {/* System Section */}
            <div className="space-y-3">
                <div className="flex items-center gap-2 px-1">
                    <ShieldCheck className="w-4 h-4 text-accent" />
                    <span className="text-xs font-bold tracking-wide uppercase opacity-60 text-text-main">
                        {t('workflows.system_presets') || "System Presets"}
                    </span>
                </div>

                <div className="grid grid-cols-2 gap-3">
                    {systemWorkflows.map((w: Workflow) => (
                        <div
                            key={w.id}
                            className="bg-bg-subtle/50 border border-border-subtle rounded-xl p-3 flex items-center justify-between group hover:bg-bg-subtle hover:border-accent/30 transition-all"
                        >
                            <div className="flex items-center gap-3 overflow-hidden">
                                <div className="p-2 bg-accent/10 rounded-lg text-accent">
                                    <Layers className="w-4 h-4" />
                                </div>
                                <div className="overflow-hidden">
                                    <div className="text-xs font-bold text-text-main truncate">
                                        {getLocalizedLabel(w.label)}
                                    </div>
                                    <div className="text-[10px] text-text-muted font-mono opacity-60">
                                        {w.shortcut}
                                    </div>
                                </div>
                            </div>
                            <button
                                onClick={() => handleEdit(w)}
                                className="p-1.5 text-text-muted hover:text-accent hover:bg-accent/10 rounded-lg transition-all"
                            >
                                <Settings2 className="w-3.5 h-3.5" />
                            </button>
                        </div>
                    ))}
                </div>
            </div>

            {/* User Section */}
            <div className="space-y-3">
                <div className="flex items-center justify-between px-1">
                    <div className="flex items-center gap-2">
                        <Sparkles className="w-4 h-4 text-purple-400" />
                        <span className="text-xs font-bold tracking-wide uppercase opacity-60 text-text-main">
                            {t('workflows.other_presets') || "Other Presets"}
                        </span>
                    </div>
                    <button
                        onClick={handleAdd}
                        className="px-2 py-1 rounded bg-purple-500/10 hover:bg-purple-500/20 text-purple-400 text-[10px] font-mono border border-purple-500/20 transition-colors flex items-center gap-1"
                    >
                        <Plus className="w-3 h-3" /> {t('workflows.new')}
                    </button>
                </div>

                <div className="grid grid-cols-1 gap-2.5">
                    {userWorkflows.length > 0 ? (
                        userWorkflows.map((w: Workflow) => (
                            <motion.div
                                key={w.id}
                                id={`workflow-${w.id}`}
                                layout
                                className="bg-bg-subtle border border-border-subtle rounded-xl flex items-center justify-between group hover:border-border-hover transition-all pr-2"
                            >
                                <div className="px-4 py-3 flex items-center gap-4 flex-1 min-w-0">
                                    <div className="p-2 bg-bg-sidebar rounded-lg text-text-muted group-hover:bg-bg-sidebar/80 group-hover:text-text-main transition-colors shrink-0">
                                        <Keyboard className="w-4.5 h-4.5" />
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-2 mb-0.5">
                                            <span className="text-sm font-bold text-text-main truncate tracking-tight">
                                                {getLocalizedLabel(w.label)}
                                            </span>
                                            {!w.enabled && (
                                                <span className="text-[8px] px-1 bg-bg-main border border-border-subtle text-text-muted rounded-md uppercase font-black tracking-tighter shrink-0">
                                                    {t('workflows.disabled')}
                                                </span>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-3">
                                            <div className="text-[10px] text-text-accent font-mono bg-accent/5 px-1.5 py-0.5 rounded border border-accent/10">
                                                {w.shortcut}
                                            </div>
                                            <div className="text-[9px] text-text-muted uppercase font-bold tracking-widest opacity-40 truncate">
                                                {w.action.type}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity pr-1">
                                    <button
                                        onClick={() => handleEdit(w)}
                                        className="p-2 text-text-muted hover:text-text-main hover:bg-bg-sidebar rounded-lg transition-all"
                                        title={t('workflows.edit')}
                                    >
                                        <Settings2 className="w-4 h-4" />
                                    </button>
                                    <button
                                        onClick={() => {
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                removeWorkflow(w.id);
                                            }
                                        }}
                                        className="p-2 text-red-400/50 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                                        title={t('workflows.delete')}
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </button>
                                </div>
                            </motion.div>
                        ))
                    ) : (
                        <div className="h-24 flex flex-col items-center justify-center text-text-muted opacity-30 gap-2 border-2 border-dashed border-border-subtle rounded-xl italic text-xs">
                            {t('workflows.empty') || "No custom presets yet"}
                        </div>
                    )}
                </div>
            </div>
        </motion.div>
    );
};

export default WorkflowsTab;
