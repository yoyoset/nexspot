import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, Clock, FolderOpen, ChevronRight, FileText, Camera, Scroll } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore, Workflow, ActivityEntry } from '../../store/useAppStore';
import { getCurrentWebview } from '@tauri-apps/api/webview';

const ActivityHub: React.FC = () => {
    const { config, activity, fetchActivity } = useAppStore();
    const { t } = useTranslation();
    const workflows = config?.workflows || [];

    useEffect(() => {
        fetchActivity();
        
        // Listen for real-time updates
        let unlisten: (() => void) | undefined;
        const setup = async () => {
            const { listen } = await import('@tauri-apps/api/event');
            unlisten = await listen('activity://updated', () => {
                fetchActivity();
            });
        };
        setup();
        
        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    const getActivityIcon = (type: string) => {
        switch (type) {
            case 'ocr': return <FileText className="w-4 h-4 text-amber-400" />;
            case 'scrolling': return <Scroll className="w-4 h-4 text-blue-400" />;
            default: return <Camera className="w-4 h-4 text-emerald-400" />;
        }
    };

    return (
        <div className="w-full h-full bg-bg-main flex flex-col px-6 py-4 gap-4 overflow-hidden select-none relative">
            <div className="noise-bg" />
            <div className="flex flex-col gap-4 flex-1 relative z-10 overflow-hidden">
            {/* 1. Header with Technical Indicator */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2.5 pl-1">
                    <Activity className="w-4 h-4 text-accent" />
                    <h2 className="text-[12px] font-bold text-text-main tech-text uppercase tracking-widest">{t('activity.title')}</h2>
                </div>
                <div className="flex items-center gap-2 px-2 py-0.5 bg-accent/5 border border-accent/20 rounded-sm">
                    <div className="w-1.5 h-1.5 bg-green-500 rounded-full animate-breathing shadow-[0_0_8px_rgba(34,197,94,0.4)]" />
                    <span className="text-[9px] font-bold text-accent tech-text uppercase tracking-widest">{t('activity.live')}</span>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-5 flex-1 overflow-hidden">
                {/* 2. Left: Recent History */}
                <div className="flex flex-col gap-3 overflow-hidden">
                    <div className="flex items-center gap-2 px-1">
                        <Clock className="w-3 h-3 text-text-muted opacity-50" />
                        <span className="text-[9px] font-bold text-text-muted tech-text uppercase tracking-widest opacity-60">{t('activity.recent')}</span>
                    </div>

                    <div className="flex-1 bg-black/20 border border-white/5 rounded-sm flex flex-col overflow-y-auto custom-scrollbar p-1.5 space-y-1.5">
                        <AnimatePresence mode='popLayout'>
                            {activity.length > 0 ? (
                                activity.map((entry: ActivityEntry) => (
                                    <motion.div
                                        key={entry.id}
                                        layout
                                        initial={{ opacity: 0, x: -10 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        exit={{ opacity: 0, scale: 0.95 }}
                                        className="flex items-center gap-3 p-3 bg-white/[0.02] border border-white/5 rounded-sm hover:border-white/10 transition-colors shadow-sm group/row"
                                    >
                                        <div className="p-1.5 bg-black/40 rounded-sm border border-white/5">
                                            {getActivityIcon(entry.activity_type)}
                                        </div>
                                        <div className="flex flex-col min-w-0 flex-1">
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-2">
                                                    <span className="text-[10px] font-bold text-text-main tech-text uppercase tracking-wider">
                                                        {t(`activity.type.${entry.activity_type.toLowerCase()}` as any) || entry.activity_type}
                                                    </span>
                                                    <span className="tech-badge opacity-50 uppercase">{entry.activity_type}</span>
                                                </div>
                                                <span className="text-[9px] text-text-muted tech-text opacity-40 group-hover/row:opacity-100 transition-opacity">
                                                    [{new Date(entry.timestamp * 1000).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}]
                                                </span>
                                            </div>
                                            <span className="text-[9px] text-text-muted truncate opacity-30 font-mono mt-0.5">
                                                {entry.file_path || entry.metadata || "SYSTEM_EVENT"}
                                            </span>
                                        </div>
                                    </motion.div>
                                ))
                            ) : (
                                <div className="flex-1 flex flex-col items-center justify-center text-text-muted/10 p-10 text-center">
                                    <Activity className="w-8 h-8 mb-3 opacity-10" />
                                    <p className="text-[10px] tech-text font-bold uppercase tracking-widest">{t('activity.empty')}</p>
                                    <p className="text-[8px] tech-text opacity-30 uppercase mt-1">{t('activity.empty_desc')}</p>
                                </div>
                            )}
                        </AnimatePresence>
                    </div>
                </div>

                {/* 3. Right: Storage Pools & Health */}
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2 px-1">
                        <FolderOpen className="w-3 h-3 text-text-muted opacity-50" />
                        <span className="text-[9px] font-bold text-text-muted tech-text uppercase tracking-widest opacity-60">{t('activity.pools')}</span>
                    </div>

                    <div className="flex-1 overflow-y-auto custom-scrollbar pr-2 space-y-1.5">
                        {workflows.map((w: Workflow) => (
                            <motion.div
                                key={w.id}
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                className="group flex items-center justify-between p-2.5 rounded-sm bg-black/10 border border-white/5 hover:border-white/20 transition-all cursor-pointer shadow-sm"
                            >
                                <div className="flex flex-col min-w-0 pr-4">
                                    <div className="flex items-center gap-2">
                                        <span className="text-[11px] font-bold text-text-main tech-text uppercase group-hover:text-accent transition-colors truncate">
                                            {w.label}
                                        </span>
                                        <div className="w-1 h-1 bg-emerald-500 opacity-60" />
                                    </div>
                                    <span className="text-[9px] text-text-muted tech-text truncate opacity-40 uppercase tracking-tighter">
                                        REF: {w.output.target_folder || t('common.root_dir')}
                                    </span>
                                </div>
                                <div className="p-1 px-2 rounded-sm bg-bg-subtle text-text-muted group-hover:text-text-main group-hover:bg-white/5 border border-transparent group-hover:border-white/10 transition-all flex items-center gap-1 shrink-0">
                                    <span className="text-[8px] font-bold tech-text uppercase tracking-widest">{t('activity.enter')}</span>
                                    <ChevronRight className="w-3 h-3" />
                                </div>
                            </motion.div>
                        ))}
                    </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default ActivityHub;
