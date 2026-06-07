import React, { useEffect, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { Camera, FileText, Scroll, Folder, ExternalLink, Activity as ActivityIcon, LayoutList, LayoutGrid } from 'lucide-react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useAppStore, Workflow, ActivityEntry } from '../../store/useAppStore';
import { useConfig } from '../../hooks/useConfig';

const TYPE_META: Record<string, { color: string; Icon: React.ComponentType<{ className?: string }> }> = {
    ocr: { color: '#22d3ee', Icon: FileText },
    scrolling: { color: '#f59e0b', Icon: Scroll },
};
const typeMeta = (type: string) => TYPE_META[type.toLowerCase()] || { color: 'var(--accent)', Icon: Camera };

const dirOf = (p: string) => p.replace(/[/\\][^/\\]*$/, '');
const baseName = (p: string) => p.split(/[/\\]/).pop() || p;
const isImage = (p: string) => /\.(png|jpe?g|webp|bmp|gif)$/i.test(p);

/** 缩略图：走 asset 协议加载真实位图，加载失败回退到类型图标。 */
const Thumb: React.FC<{ path: string; color: string; Icon: React.ComponentType<{ className?: string }>; className?: string }> = ({ path, color, Icon, className }) => {
    const [failed, setFailed] = useState(false);
    if (isImage(path) && !failed) {
        return (
            <img
                src={convertFileSrc(path)}
                onError={() => setFailed(true)}
                className={className}
                loading="lazy"
                alt=""
            />
        );
    }
    return (
        <div className={`flex items-center justify-center ${className}`} style={{ background: `color-mix(in srgb, ${color} 16%, transparent)` }}>
            <Icon className="w-4 h-4" />
        </div>
    );
};

const ActivityHub: React.FC = () => {
    const { config, activity, fetchActivity } = useAppStore();
    const { openFolder } = useConfig();
    const { t } = useTranslation();
    const workflows = config?.workflows || [];
    const [view, setView] = useState<'list' | 'grid'>('grid');

    useEffect(() => {
        fetchActivity();
        let unlisten: (() => void) | undefined;
        (async () => {
            const { listen } = await import('@tauri-apps/api/event');
            unlisten = await listen('activity://updated', () => fetchActivity());
        })();
        return () => { if (unlisten) unlisten(); };
    }, []);

    const defaultPath = config?.save_path || '';
    const pools = useMemo(() => {
        // 按解析后的实际文件夹分组（target_folder 为空时归入全局默认路径）
        const map = new Map<string, Workflow[]>();
        workflows.forEach((w: Workflow) => {
            const folder = w.output.target_folder || defaultPath || t('common.root_dir');
            if (!map.has(folder)) map.set(folder, []);
            map.get(folder)!.push(w);
        });
        return Array.from(map.entries());
    }, [workflows, defaultPath, t]);

    const typeLabel = (type: string) => {
        const key = `activity.type.${type.toLowerCase()}`;
        const tr = t(key);
        return tr === key ? type : tr;
    };

    return (
        <div className="w-full h-full bg-bg-0 flex flex-col overflow-hidden select-none">
            {/* Header */}
            <div className="flex items-center gap-3 px-6 py-[18px] border-b border-line shrink-0">
                <h1 className="text-[17px] font-extrabold tracking-[-0.02em] text-ink leading-none whitespace-nowrap">
                    {t('activity.title')}
                </h1>
                <span className="flex items-center gap-1.5 px-2 py-0.5 rounded-full border border-bad/60">
                    <span className="relative flex items-center justify-center w-2 h-2">
                        <span className="absolute w-2 h-2 rounded-full bg-bad-soft animate-ping" />
                        <span className="w-2 h-2 rounded-full bg-bad" />
                    </span>
                    <span className="mono text-[10.5px] font-semibold tracking-[0.08em] text-bad">LIVE</span>
                </span>
                <span className="ml-auto text-[11.5px] text-muted truncate">{t('activity.subtitle')}</span>
            </div>

            {/* Body: 2-col */}
            <div className="flex-1 min-h-0 grid grid-cols-1 lg:grid-cols-[1.5fr_1fr] gap-4 px-6 py-5 overflow-hidden">
                {/* Left · activity stream */}
                <section className="flex flex-col min-h-0">
                    <div className="flex items-center justify-between mb-3">
                        <h2 className="mono text-[10.5px] font-semibold tracking-[0.12em] uppercase text-faint">
                            {t('activity.section_stream')}
                        </h2>
                        <div className="flex items-center gap-0.5 p-0.5 rounded-btn bg-bg-2 border border-line">
                            <button
                                onClick={() => setView('list')}
                                title={t('activity.view_list')}
                                className={`w-7 h-7 flex items-center justify-center rounded-[7px] transition-colors ${view === 'list' ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'}`}
                            >
                                <LayoutList className="w-4 h-4" />
                            </button>
                            <button
                                onClick={() => setView('grid')}
                                title={t('activity.view_grid')}
                                className={`w-7 h-7 flex items-center justify-center rounded-[7px] transition-colors ${view === 'grid' ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'}`}
                            >
                                <LayoutGrid className="w-4 h-4" />
                            </button>
                        </div>
                    </div>

                    {activity.length === 0 ? (
                        <div className="flex-1 flex flex-col items-center justify-center text-center py-16 gap-2">
                            <ActivityIcon className="w-8 h-8 text-faint opacity-40 mb-1" />
                            <p className="text-[12.5px] font-semibold text-muted">{t('activity.empty')}</p>
                            <p className="text-[11.5px] text-faint">{t('activity.empty_desc')}</p>
                        </div>
                    ) : view === 'grid' ? (
                        <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar pr-1">
                            <div className="grid grid-cols-2 gap-3">
                                {activity.map((entry: ActivityEntry) => {
                                    const { color, Icon } = typeMeta(entry.activity_type);
                                    const path = entry.file_path || entry.metadata || '';
                                    return (
                                        <motion.button
                                            key={entry.id}
                                            initial={{ y: 7 }}
                                            animate={{ y: 0 }}
                                            transition={{ duration: 0.18 }}
                                            onClick={() => path && openFolder(dirOf(path))}
                                            title={path}
                                            className="group flex flex-col rounded-panel bg-bg-2 border border-line hover:border-line-2 transition-colors overflow-hidden text-left"
                                        >
                                            <div className="relative w-full aspect-video bg-bg-0 overflow-hidden">
                                                <Thumb path={path} color={color} Icon={Icon} className="w-full h-full object-cover" />
                                                <span className="absolute top-1.5 right-1.5 mono text-[9.5px] text-faint bg-bg-0/80 backdrop-blur px-1.5 py-0.5 rounded">
                                                    {new Date(entry.timestamp * 1000).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit' })}
                                                </span>
                                            </div>
                                            <div className="px-2.5 py-2 min-w-0">
                                                <div className="text-[12px] font-semibold text-ink truncate">{typeLabel(entry.activity_type)}</div>
                                                {path && <div className="mono text-[10px] text-muted truncate">{baseName(path)}</div>}
                                            </div>
                                        </motion.button>
                                    );
                                })}
                            </div>
                        </div>
                    ) : (
                        <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar flex flex-col gap-2 pr-1">
                            {activity.map((entry: ActivityEntry) => {
                                const { color, Icon } = typeMeta(entry.activity_type);
                                const path = entry.file_path || entry.metadata || '';
                                return (
                                    <motion.div
                                        key={entry.id}
                                        initial={{ y: 7 }}
                                        animate={{ y: 0 }}
                                        transition={{ duration: 0.18 }}
                                        className="group flex items-center gap-3 px-3 py-2.5 rounded-panel bg-bg-2 border border-line hover:border-line-2 transition-colors"
                                    >
                                        <Thumb path={path} color={color} Icon={Icon} className="w-[44px] h-[34px] shrink-0 rounded-[7px] object-cover border border-line" />
                                        <div className="flex flex-col min-w-0 flex-1">
                                            <span className="text-[12.5px] font-semibold text-ink truncate">{typeLabel(entry.activity_type)}</span>
                                            {path && <span className="mono text-[10.5px] text-muted truncate">{path}</span>}
                                        </div>
                                        <span className="mono text-[10.5px] text-faint shrink-0">
                                            {new Date(entry.timestamp * 1000).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                                        </span>
                                        {path && (
                                            <button
                                                onClick={() => openFolder(dirOf(path))}
                                                title={t('activity.open')}
                                                className="w-[30px] h-[30px] shrink-0 flex items-center justify-center rounded-btn text-muted hover:text-ink hover:bg-bg-3 transition-colors opacity-0 group-hover:opacity-100"
                                            >
                                                <ExternalLink className="w-4 h-4" />
                                            </button>
                                        )}
                                    </motion.div>
                                );
                            })}
                        </div>
                    )}
                </section>

                {/* Right · storage pools */}
                <section className="flex flex-col min-h-0">
                    <h2 className="mono text-[10.5px] font-semibold tracking-[0.12em] uppercase text-faint mb-3">
                        {t('activity.section_pools')}
                    </h2>
                    <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar flex flex-col gap-2 pr-1">
                        {pools.map(([folder, ws]) => {
                            const openable = folder && folder !== t('common.root_dir');
                            return (
                            <button
                                key={folder}
                                onClick={() => openable && openFolder(folder)}
                                disabled={!openable}
                                className="group flex items-center gap-3 px-3 py-2.5 rounded-panel bg-bg-2 border border-line hover:border-line-2 transition-colors text-left disabled:cursor-default"
                            >
                                <div className="w-9 h-9 shrink-0 rounded-[9px] bg-bg-0 border border-line flex items-center justify-center">
                                    <Folder className="w-[18px] h-[18px] text-muted group-hover:text-accent transition-colors" />
                                </div>
                                <div className="flex flex-col min-w-0 flex-1">
                                    <span className="mono text-[11px] font-semibold text-ink truncate">{folder}</span>
                                    <span className="text-[10.5px] text-muted truncate">
                                        {ws.map((w) => w.label).join(' · ')}
                                    </span>
                                </div>
                                <span className="mono text-[10px] font-semibold px-1.5 py-0.5 rounded shrink-0 bg-accent-soft text-accent">
                                    {ws.length}
                                </span>
                            </button>
                            );
                        })}
                    </div>
                </section>
            </div>
        </div>
    );
};

export default ActivityHub;
