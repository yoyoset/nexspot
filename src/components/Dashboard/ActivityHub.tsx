import React from 'react';
import { motion } from 'framer-motion';
import { Activity, Clock, FolderOpen, ChevronRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore, Workflow } from '../../store/useAppStore';

const ActivityHub: React.FC = () => {
    const { config } = useAppStore();
    const { t } = useTranslation();
    const workflows = config?.workflows || [];

    return (
        <div className="w-full h-full bg-bg-main flex flex-col p-6 gap-6 overflow-hidden select-none">
            {/* 1. Header with Pulse */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Activity className="w-5 h-5 text-accent" />
                    <h2 className="text-lg font-bold text-text-main tracking-tight">{t('activity.title')}</h2>
                </div>
                <div className="flex items-center gap-2 px-3 py-1 bg-accent/5 border border-accent/10 rounded-full">
                    <div className="w-1.5 h-1.5 rounded-full bg-accent animate-pulse" />
                    <span className="text-[10px] font-bold text-accent uppercase tracking-widest">{t('activity.live')}</span>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1 overflow-hidden">
                {/* 2. Left: Recent History (Placeholder) */}
                <div className="flex flex-col gap-4">
                    <div className="flex items-center gap-2 px-1">
                        <Clock className="w-3.5 h-3.5 text-text-muted" />
                        <span className="text-[10px] font-bold text-text-muted uppercase tracking-[0.2em] opacity-60">{t('activity.recent')}</span>
                    </div>

                    <div className="flex-1 bg-bg-subtle/50 border border-border-subtle rounded-2xl flex flex-col items-center justify-center text-text-muted/30 border-dashed p-10 text-center">
                        <Activity className="w-12 h-12 mb-4 opacity-10" />
                        <p className="text-xs font-medium">{t('activity.empty')}</p>
                        <p className="text-[10px] opacity-50">{t('activity.empty_desc')}</p>
                    </div>
                </div>

                {/* 3. Right: Storage Pools & Health */}
                <div className="flex flex-col gap-4">
                    <div className="flex items-center gap-2 px-1">
                        <FolderOpen className="w-3.5 h-3.5 text-text-muted" />
                        <span className="text-[10px] font-bold text-text-muted uppercase tracking-[0.2em] opacity-60">{t('activity.pools')}</span>
                    </div>

                    <div className="flex-1 overflow-y-auto custom-scrollbar pr-2 space-y-2">
                        {workflows.map((w: Workflow) => (
                            <motion.div
                                key={w.id}
                                initial={{ opacity: 0, x: 20 }}
                                animate={{ opacity: 1, x: 0 }}
                                className="group flex items-center justify-between p-3 rounded-xl bg-bg-card border border-border-subtle hover:border-accent/30 transition-all cursor-pointer shadow-sm"
                            >
                                <div className="flex flex-col min-w-0 pr-4">
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs font-bold text-text-main group-hover:text-accent transition-colors truncate">
                                            {w.label} {t('activity.pool_suffix')}
                                        </span>
                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]" />
                                    </div>
                                    <span className="text-[10px] text-text-muted font-mono truncate opacity-60">
                                        {w.output.target_folder}
                                    </span>
                                </div>
                                <div className="p-1 px-2 rounded-lg bg-bg-subtle text-text-muted group-hover:text-text-main group-hover:bg-bg-card transition-all flex items-center gap-1 shrink-0">
                                    <span className="text-[10px] font-bold uppercase tracking-tighter">{t('activity.enter')}</span>
                                    <ChevronRight className="w-3 h-3" />
                                </div>
                            </motion.div>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
};

export default ActivityHub;
