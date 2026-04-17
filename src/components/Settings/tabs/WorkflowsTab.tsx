import React, { useEffect, useMemo } from "react";
import { Plus, Trash2, Settings2, ShieldCheck, Keyboard, Terminal, Layers, Check } from "lucide-react";
import { motion } from "framer-motion";
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

    const getLocalizedMode = (mode: string) => {
        const mapping: Record<string, string> = {
            'Selection': t('workflows.mode_selection'),
            'Fullscreen': t('workflows.mode_fullscreen'),
            'Window': t('workflows.mode_window'),
            'Snapshot': t('workflows.mode_snapshot')
        };
        return mapping[mode] || mode;
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
            }
        }
    }, [initialWorkflowId]);

    return (
        <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="space-y-6 pb-6"
        >
            {/* System Section */}
            <div className="space-y-2.5">
                <div className="flex items-center gap-2 px-1">
                    <ShieldCheck className="w-3.5 h-3.5 text-accent opacity-70" />
                    <span className="text-[10px] font-bold tracking-widest uppercase text-text-muted tech-text">
                        {t('workflows.system_section')}
                    </span>
                </div>

                <div className="grid grid-cols-2 gap-2">
                    {systemWorkflows.map((w: Workflow) => (
                        <div
                            key={w.id}
                            className="bg-black/20 border border-white/5 rounded-sm p-2 flex items-center justify-between group hover:border-white/20 transition-all"
                        >
                            <div className="flex items-center gap-3 overflow-hidden">
                                <div className="p-1.5 bg-accent/10 rounded-sm text-accent">
                                    <Layers className="w-3.5 h-3.5" />
                                </div>
                                <div className="overflow-hidden">
                                    <div className="text-[11px] font-bold text-text-main tech-text uppercase truncate">
                                        {getLocalizedLabel(w.label)}
                                    </div>
                                    <div className="text-[9px] text-text-muted tech-text opacity-40 uppercase tracking-tighter">
                                        ID_REF: {w.shortcut}
                                    </div>
                                </div>
                            </div>
                            <button
                                onClick={() => handleEdit(w)}
                                className="p-1.5 text-text-muted hover:text-text-main hover:bg-white/5 rounded-sm transition-all"
                            >
                                <Settings2 className="w-3 h-3" />
                            </button>
                        </div>
                    ))}
                </div>
            </div>

            {/* User Section */}
            <div className="space-y-2.5">
                <div className="flex items-center justify-between px-1">
                    <div className="flex items-center gap-2">
                        <Terminal className="w-3.5 h-3.5 text-accent opacity-70" />
                        <span className="text-[10px] font-bold tracking-widest uppercase text-text-muted tech-text">
                            {t('workflows.user_section')}
                        </span>
                    </div>
                    <button
                        onClick={handleAdd}
                        className="px-2 py-1 rounded-sm bg-accent/10 hover:bg-accent/20 text-accent text-[9px] tech-text font-bold border border-accent/20 transition-all flex items-center gap-1"
                    >
                        <Plus className="w-3 h-3" /> {t('workflows.new_workflow')}
                    </button>
                </div>

                <div className="grid grid-cols-1 gap-1.5">
                    {userWorkflows.length > 0 ? (
                        userWorkflows.map((w: Workflow) => (
                            <motion.div
                                key={w.id}
                                id={`workflow-${w.id}`}
                                layout
                                className="bg-black/20 border border-white/5 rounded-sm flex items-center justify-between group hover:border-white/20 transition-all pr-2"
                            >
                                <div className="px-3 py-2.5 flex items-center gap-4 flex-1 min-w-0">
                                    <div className="p-1.5 bg-white/5 rounded-sm text-text-muted group-hover:text-text-main transition-colors shrink-0">
                                        <Keyboard className="w-4 h-4" />
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-2 mb-0.5">
                                            <span className="text-[11px] font-bold text-text-main tech-text uppercase truncate tracking-tight">
                                                {getLocalizedLabel(w.label)}
                                            </span>
                                            {!w.enabled && (
                                                <span className="text-[8px] px-1 bg-black/40 border border-white/10 text-text-muted rounded-sm uppercase font-bold tech-text shrink-0">
                                                    {t('workflows.status_deactivated')}
                                                </span>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-3">
                                            <div className="text-[9px] text-accent font-bold tech-text bg-accent/5 px-1 rounded-sm border border-accent/10">
                                                {w.shortcut}
                                            </div>
                                            <div className="text-[9px] text-text-muted tech-text uppercase font-bold tracking-widest opacity-30 truncate">
                                                {t('workflows.field_type')} {getLocalizedMode(w.action.type)}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all pr-1">
                                    <button
                                        onClick={() => handleEdit(w)}
                                        className="p-1.5 text-text-muted hover:text-text-main hover:bg-white/5 rounded-sm transition-all"
                                    >
                                        <Settings2 className="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                        onClick={() => {
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                removeWorkflow(w.id);
                                            }
                                        }}
                                        className="p-1.5 text-red-500/50 hover:text-red-500 hover:bg-red-500/10 rounded-sm transition-all"
                                    >
                                        <Trash2 className="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            </motion.div>
                        ))
                    ) : (
                        <div className="h-20 flex flex-col items-center justify-center text-text-muted/20 gap-2 border border-dashed border-white/5 rounded-sm tech-text font-bold text-[9px] uppercase tracking-widest">
                            {t('workflows.no_protocols')}
                        </div>
                    )}
                </div>
            </div>
        </motion.div>
    );
};

export default WorkflowsTab;
