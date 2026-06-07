import React from "react";
import { motion } from "framer-motion";
import {
    Zap, Plus, Pencil, Trash2, Lock, Folder, Scan, Monitor, AppWindow, Crop,
    AlertTriangle, CircleDot,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { translateError } from "../../utils/error";
import { useAppStore, Workflow } from "../../store/useAppStore";

interface DashboardProps {
    onNavigateToWorkflows: (id?: string) => void;
}

const MODE_ICON: Record<string, React.ComponentType<{ className?: string }>> = {
    selection: Scan,
    fullscreen: Monitor,
    window: AppWindow,
    snapshot: Crop,
};

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

    const { counts, registrations } = React.useMemo(() => {
        const counts: Record<string, number> = {};
        workflows.forEach((w: Workflow) => {
            if (w.shortcut) counts[w.shortcut] = (counts[w.shortcut] || 0) + 1;
        });
        return { counts, registrations: config?.registration_errors || [] };
    }, [workflows, config?.registration_errors]);

    const isVello = (w: Workflow) => w.action.config.engine.toLowerCase() === 'vello';
    const conflictCount = Object.values(counts).filter((n) => n > 1).length;
    const enginesNotReady = workflows.filter((w) => w.enabled && isVello(w) && velloStatus !== 'ready').length;

    const handleCreateNew = () => onNavigateToWorkflows('new');
    const handleEdit = (id: string) => onNavigateToWorkflows(id);

    const handleTrigger = async (id: string) => {
        try {
            await invoke('trigger_capture', { action: id });
        } catch (err) {
            useAppStore.getState().showHUD(translateError(err, t), 'error');
        }
    };

    const handleDelete = async (id: string) => {
        if (!window.confirm(t('workflows.delete_confirm'))) return;
        try {
            await removeWorkflow(id);
            useAppStore.getState().showHUD(t('hud.saved'), 'success');
        } catch (err) {
            useAppStore.getState().showHUD(translateError(err, t), 'error');
        }
    };

    return (
        <div className="w-full h-full bg-bg-main custom-scrollbar overflow-y-auto select-none">
            {/* Header */}
            <div className="flex items-center gap-3 px-6 py-[18px] border-b border-line">
                <div className="flex flex-col gap-0.5 min-w-0">
                    <h1 className="text-[17px] font-extrabold tracking-[-0.02em] text-ink leading-none">
                        {t('dashboard.page_title')}
                    </h1>
                    <span className="text-[11.5px] text-muted">
                        {t('dashboard.subtitle', { count: workflows.length })}
                    </span>
                </div>

                {/* At-a-glance summary pills */}
                <div className="flex items-center gap-2">
                    {conflictCount > 0 && (
                        <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-bad-soft text-bad text-[11px] font-semibold">
                            <AlertTriangle className="w-3.5 h-3.5" />
                            {t('dashboard.summary_conflicts', { count: conflictCount })}
                        </span>
                    )}
                    {enginesNotReady > 0 && (
                        <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-warn-soft text-warn text-[11px] font-semibold">
                            <CircleDot className="w-3.5 h-3.5" />
                            {t('dashboard.summary_engines', { count: enginesNotReady })}
                        </span>
                    )}
                </div>

                <button
                    onClick={handleCreateNew}
                    className="ml-auto flex items-center gap-1.5 px-3.5 py-2 rounded-btn bg-accent text-on-accent text-[12.5px] font-semibold hover:bg-[var(--accent-press)] transition-colors shrink-0"
                >
                    <Plus className="w-4 h-4" /> {t('dashboard.new_workflow')}
                </button>
            </div>

            {/* List */}
            <div className="px-6 py-5">
                {workflows.length === 0 ? (
                    <EmptyState t={t} onCreate={handleCreateNew} />
                ) : (
                    <div className="flex flex-col gap-[9px]">
                        {workflows.map((w: Workflow, i: number) => {
                            const mode = w.action.type.toLowerCase();
                            const Icon = MODE_ICON[mode] || Scan;
                            const vello = isVello(w);
                            const engineReady = vello ? velloStatus === 'ready' : true;
                            const hasConflict =
                                (counts[w.shortcut] || 0) > 1 ||
                                registrations.some((e: string) => e === w.id || e.startsWith(w.id + ":"));
                            const fmt = w.output.format.toUpperCase();
                            const isJpg = fmt === 'JPG' || fmt === 'JPEG';

                            return (
                                <motion.div
                                    key={w.id}
                                    initial={{ y: 7 }}
                                    animate={{ y: 0 }}
                                    transition={{ duration: 0.18, delay: Math.min(i * 0.02, 0.12) }}
                                    className={`group flex items-center gap-3.5 px-3.5 py-3.5 rounded-panel border bg-bg-2 transition-colors ${
                                        hasConflict ? 'border-[color-mix(in_srgb,var(--bad)_36%,var(--bd))]' : 'border-line hover:border-line-2'
                                    } ${!w.enabled ? 'opacity-50' : ''}`}
                                >
                                    {/* Mode icon brick */}
                                    <div className="w-10 h-10 shrink-0 rounded-[9px] bg-accent-soft flex items-center justify-center">
                                        <Icon className="w-[19px] h-[19px] text-accent" />
                                    </div>

                                    {/* Body */}
                                    <div className="flex flex-col gap-1 min-w-0 flex-1">
                                        <div className="flex items-center gap-2 min-w-0">
                                            <span className="text-[13.5px] font-semibold text-ink truncate">
                                                {getLocalizedLabel(w.label, w.id)}
                                            </span>
                                            {w.is_system && (
                                                <span className="mono text-[10px] text-faint border border-line rounded px-1.5 py-0.5 shrink-0">
                                                    {t('dashboard.system_preset')}
                                                </span>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-2 min-w-0">
                                            <span className="mono text-[10.5px] tracking-[0.02em] text-muted">
                                                <span className={vello ? 'text-accent' : ''}>{w.action.config.engine.toLowerCase()}</span>
                                                <span className="text-faint"> · </span>
                                                {t(`workflows.mode_${mode}`)}
                                            </span>
                                            <span
                                                className="mono text-[10px] font-semibold px-1.5 py-0.5 rounded"
                                                style={isJpg
                                                    ? { background: 'color-mix(in srgb, #22d3ee 16%, transparent)', color: '#22d3ee' }
                                                    : { background: 'var(--accent-soft)', color: 'var(--accent)' }}
                                            >
                                                {fmt}
                                            </span>
                                            {w.output.target_folder && (
                                                <button
                                                    onClick={(e) => { e.stopPropagation(); openFolder(w.output.target_folder!); }}
                                                    className="flex items-center gap-1 mono text-[10.5px] text-muted hover:text-ink transition-colors min-w-0"
                                                    title={w.output.target_folder}
                                                >
                                                    <Folder className="w-3 h-3 shrink-0" />
                                                    <span className="truncate max-w-[140px]">{w.output.target_folder}</span>
                                                </button>
                                            )}
                                        </div>
                                    </div>

                                    {/* Two independent status indicators */}
                                    <div className="w-[118px] shrink-0 flex flex-col gap-1.5 text-[11px]">
                                        <span className="flex items-center gap-1.5">
                                            <span className={`w-1.5 h-1.5 rounded-full ${engineReady ? 'bg-ok' : 'bg-warn'}`} />
                                            <span className={engineReady ? 'text-muted' : 'text-warn font-semibold'}>
                                                {engineReady ? t('dashboard.engine_ready') : t('dashboard.engine_not_ready')}
                                            </span>
                                        </span>
                                        <span className="flex items-center gap-1.5">
                                            {hasConflict ? (
                                                <>
                                                    <AlertTriangle className="w-3 h-3 text-bad" />
                                                    <span className="text-bad font-semibold">{t('dashboard.hotkey_conflict')}</span>
                                                </>
                                            ) : (
                                                <>
                                                    <span className="w-1.5 h-1.5 rounded-full bg-line-2" />
                                                    <span className="text-muted">{t('dashboard.hotkey_ok')}</span>
                                                </>
                                            )}
                                        </span>
                                    </div>

                                    {/* Hotkey kbd */}
                                    <div className="w-[110px] shrink-0 flex justify-center">
                                        {w.shortcut ? (
                                            <kbd className={`mono text-[11px] font-semibold px-2 py-1 rounded-[7px] bg-bg-0 border ${
                                                hasConflict ? 'text-bad border-bad' : 'text-ink border-line-2'
                                            }`} style={{ borderBottomWidth: 2 }}>
                                                {w.shortcut}
                                            </kbd>
                                        ) : (
                                            <span className="mono text-[10px] text-faint italic">{t('workflows.unbound')}</span>
                                        )}
                                    </div>

                                    {/* Actions */}
                                    <div className="flex items-center gap-0.5 shrink-0">
                                        <IconBtn title={t('dashboard.capture_btn')} onClick={() => handleTrigger(w.id)} accent>
                                            <Zap className="w-4 h-4" />
                                        </IconBtn>
                                        <IconBtn title={t('dashboard.edit')} onClick={() => handleEdit(w.id)}>
                                            <Pencil className="w-4 h-4" />
                                        </IconBtn>
                                        {w.is_system ? (
                                            <span className="w-[30px] h-[30px] flex items-center justify-center text-faint" title={t('dashboard.system_preset')}>
                                                <Lock className="w-4 h-4" />
                                            </span>
                                        ) : (
                                            <IconBtn title={t('dashboard.delete')} onClick={() => handleDelete(w.id)} danger>
                                                <Trash2 className="w-4 h-4" />
                                            </IconBtn>
                                        )}
                                    </div>
                                </motion.div>
                            );
                        })}
                    </div>
                )}
            </div>
            {velloError && velloStatus === 'failed' && (
                <div className="px-6 pb-4 mono text-[10.5px] text-warn">{velloError}</div>
            )}
        </div>
    );
};

const IconBtn: React.FC<{
    title: string;
    onClick: () => void;
    accent?: boolean;
    danger?: boolean;
    children: React.ReactNode;
}> = ({ title, onClick, accent, danger, children }) => (
    <button
        title={title}
        onClick={(e) => { e.stopPropagation(); onClick(); }}
        className={`w-[30px] h-[30px] flex items-center justify-center rounded-btn transition-colors ${
            accent ? 'text-accent hover:bg-accent-soft'
                : danger ? 'text-muted hover:text-bad hover:bg-bad-soft'
                    : 'text-muted hover:text-ink hover:bg-bg-3'
        }`}
    >
        {children}
    </button>
);

const EmptyState: React.FC<{ t: (k: string) => string; onCreate: () => void }> = ({ t, onCreate }) => (
    <div className="flex flex-col items-center justify-center text-center py-20 gap-4">
        <div className="w-16 h-16 rounded-lg bg-accent-soft flex items-center justify-center">
            <Zap className="w-7 h-7 text-accent" />
        </div>
        <div className="flex flex-col gap-1.5">
            <h2 className="text-[15px] font-extrabold text-ink">{t('dashboard.empty_title')}</h2>
            <p className="text-[12.5px] text-muted max-w-[360px]">{t('dashboard.empty_desc')}</p>
        </div>
        <button
            onClick={onCreate}
            className="flex items-center gap-1.5 px-3.5 py-2 rounded-btn bg-accent text-on-accent text-[12.5px] font-semibold hover:bg-[var(--accent-press)] transition-colors"
        >
            <Plus className="w-4 h-4" /> {t('dashboard.empty_cta')}
        </button>
    </div>
);

export default Dashboard;
