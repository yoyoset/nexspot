import React, { useEffect, useMemo } from "react";
import { Plus, Pencil, Trash2, Lock, Scan, Monitor, AppWindow, Crop } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useAppStore, Workflow } from "../../../store/useAppStore";
import { SectionTitle } from "../atoms";

interface WorkflowsTabProps {
    config: any;
    removeWorkflow: (id: string) => Promise<any>;
    initialWorkflowId?: string | null;
}

const MODE_ICON: Record<string, React.ComponentType<{ className?: string }>> = {
    Selection: Scan, Fullscreen: Monitor, Window: AppWindow, Snapshot: Crop,
};

const WorkflowsTab: React.FC<WorkflowsTabProps> = ({ config, removeWorkflow, initialWorkflowId }) => {
    const { t } = useTranslation();
    const setWorkflowEditing = useAppStore(state => state.setWorkflowEditing);
    const workflows = config?.workflows || [];

    const { systemWorkflows, userWorkflows } = useMemo(() => ({
        systemWorkflows: workflows.filter((w: Workflow) => w.is_system),
        userWorkflows: workflows.filter((w: Workflow) => !w.is_system),
    }), [workflows]);

    const handleEdit = (w: Workflow) => setWorkflowEditing(true, w);

    const getLocalizedLabel = (label: string) => {
        const key = `workflows.presets.${label.toLowerCase().replace(/\s+/g, '_')}`;
        const tr = t(key);
        return tr === key ? label : tr;
    };

    const handleAdd = () => {
        const newWorkflow: Workflow = {
            id: `user_${Date.now()}`,
            label: t('workflows.new'),
            shortcut: "Alt+F1",
            enabled: true,
            is_system: false,
            action: { type: 'Selection', config: { engine: 'gdi' } },
            output: { save_to_file: true, save_to_clipboard: true, target_folder: null, naming_template: "capture_%Y%m%d_%H%M%S", format: "png" },
        };
        setWorkflowEditing(true, newWorkflow);
    };

    useEffect(() => {
        if (initialWorkflowId === 'new') handleAdd();
        else if (initialWorkflowId && initialWorkflowId !== 'none') {
            const w = workflows.find((w: Workflow) => w.id === initialWorkflowId);
            if (w) handleEdit(w);
        }
    }, [initialWorkflowId]);

    const Card: React.FC<{ w: Workflow }> = ({ w }) => {
        const Icon = MODE_ICON[w.action.type] || Scan;
        return (
            <motion.div
                layout
                className="group flex items-center gap-3 px-3.5 py-3 rounded-panel bg-bg-2 border border-line hover:border-line-2 transition-colors"
            >
                <div className="w-9 h-9 shrink-0 rounded-[9px] bg-accent-soft flex items-center justify-center">
                    <Icon className="w-[18px] h-[18px] text-accent" />
                </div>
                <div className="flex flex-col min-w-0 flex-1">
                    <div className="flex items-center gap-2 min-w-0">
                        <span className="text-[13px] font-semibold text-ink truncate">{getLocalizedLabel(w.label)}</span>
                        {!w.enabled && (
                            <span className="mono text-[10px] text-faint border border-line rounded px-1.5 py-0.5 shrink-0">
                                {t('workflows.status_deactivated')}
                            </span>
                        )}
                    </div>
                    <span className="mono text-[10.5px] text-muted truncate">{w.shortcut} · {t(`workflows.mode_${w.action.type.toLowerCase()}`)}</span>
                </div>
                <div className="flex items-center gap-0.5 shrink-0">
                    <button onClick={() => handleEdit(w)} title={t('dashboard.edit')}
                        className="w-[30px] h-[30px] flex items-center justify-center rounded-btn text-muted hover:text-ink hover:bg-bg-3 transition-colors">
                        <Pencil className="w-4 h-4" />
                    </button>
                    {w.is_system ? (
                        <span className="w-[30px] h-[30px] flex items-center justify-center text-faint"><Lock className="w-4 h-4" /></span>
                    ) : (
                        <button
                            onClick={() => { if (window.confirm(t('workflows.delete_confirm'))) removeWorkflow(w.id); }}
                            title={t('dashboard.delete')}
                            className="w-[30px] h-[30px] flex items-center justify-center rounded-btn text-muted hover:text-bad hover:bg-bad-soft transition-colors"
                        >
                            <Trash2 className="w-4 h-4" />
                        </button>
                    )}
                </div>
            </motion.div>
        );
    };

    return (
        <motion.div initial={{ y: 7 }} animate={{ y: 0 }} transition={{ duration: 0.18 }} className="max-w-[620px] flex flex-col gap-6">
            <section>
                <div className="flex items-center justify-between mb-2">
                    <SectionTitle>{t('workflows.user_section')}</SectionTitle>
                    <button
                        onClick={handleAdd}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-btn bg-accent text-on-accent text-[12px] font-semibold hover:bg-[var(--accent-press)] transition-colors"
                    >
                        <Plus className="w-3.5 h-3.5" /> {t('workflows.new_workflow')}
                    </button>
                </div>
                <div className="flex flex-col gap-2">
                    {userWorkflows.length > 0 ? userWorkflows.map((w: Workflow) => <Card key={w.id} w={w} />) : (
                        <div className="py-10 flex items-center justify-center rounded-panel border border-dashed border-line text-[12.5px] text-muted">
                            {t('workflows.no_protocols')}
                        </div>
                    )}
                </div>
            </section>

            <section>
                <SectionTitle>{t('workflows.system_section')}</SectionTitle>
                <div className="flex flex-col gap-2">
                    {systemWorkflows.map((w: Workflow) => <Card key={w.id} w={w} />)}
                </div>
            </section>
        </motion.div>
    );
};

export default WorkflowsTab;
