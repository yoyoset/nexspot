import React from "react";
import { FileText, Trash2, Cpu, Image as ImageIcon, Zap, Layers } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface AdvancedTabProps {
    config: any;
    setVelloEnabled: (enabled: boolean) => void;
    setVelloAdvancedEffects: (enabled: boolean) => void;
    setSnapshotEnabled: (enabled: boolean) => void;
    setSnapshotSize: (width: number, height: number) => void;
    setJpgQuality: (quality: number) => void;
    setConcurrency: (concurrency: number) => void;
}

const Section = ({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) => (
    <section className="space-y-3">
        <div className="flex items-center gap-2 pl-1">
            <Icon className="w-3.5 h-3.5 text-accent" />
            <h3 className="text-xs font-bold text-text-muted uppercase tracking-wider">{title}</h3>
        </div>
        <div className="space-y-2">{children}</div>
    </section>
);

const AdvancedTab: React.FC<AdvancedTabProps> = ({
    config,
    setVelloEnabled,
    setVelloAdvancedEffects,
    setSnapshotEnabled,
    setSnapshotSize,
    setJpgQuality,
    setConcurrency
}) => {
    const { t } = useTranslation();

    return (
        <motion.div
            key="advanced"
            initial={{ opacity: 0, x: 5 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -5 }}
            className="space-y-8 pb-10"
        >
            {/* 1. Performance & Quality */}
            <Section title={t('settings.performance.title')} icon={Zap}>
                <div className="bg-bg-subtle rounded-xl p-5 space-y-6 border border-border-subtle shadow-lg shadow-black/5">
                    {/* JPG Quality */}
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-sm font-bold text-text-main flex items-center gap-2">
                                    <ImageIcon className="w-4 h-4 text-accent/60" />
                                    {t('settings.performance.jpg_quality')}
                                </div>
                                <div className="text-xs text-text-muted mt-0.5">{t('settings.performance.jpg_quality_desc')}</div>
                            </div>
                            <span className="text-xs font-mono font-bold text-accent">{config?.jpg_quality}%</span>
                        </div>
                        <input
                            type="range"
                            min="10"
                            max="100"
                            value={config?.jpg_quality || 90}
                            onChange={(e) => setJpgQuality(parseInt(e.target.value))}
                            className="w-full accent-accent bg-bg-main h-1.5 rounded-lg appearance-none cursor-pointer"
                        />
                    </div>

                    {/* Concurrency */}
                    <div className="flex items-center justify-between pt-4 border-t border-border-subtle/50">
                        <div>
                            <div className="text-sm font-bold text-text-main flex items-center gap-2">
                                <Layers className="w-4 h-4 text-accent/60" />
                                {t('settings.performance.concurrency')}
                            </div>
                            <div className="text-xs text-text-muted mt-0.5">{t('settings.performance.concurrency_desc')}</div>
                        </div>
                        <div className="flex items-center gap-2 bg-bg-main border border-border-subtle rounded-lg p-1 px-2">
                            <button
                                onClick={() => setConcurrency(Math.max(1, (config?.concurrency || 1) - 1))}
                                className="w-6 h-6 flex items-center justify-center hover:bg-bg-card rounded-md text-text-muted transition-colors font-bold"
                            >
                                -
                            </button>
                            <span className="w-8 text-center text-xs font-mono font-bold text-text-main">{config?.concurrency || 4}</span>
                            <button
                                onClick={() => setConcurrency(Math.min(32, (config?.concurrency || 4) + 1))}
                                className="w-6 h-6 flex items-center justify-center hover:bg-bg-card rounded-md text-text-muted transition-colors font-bold"
                            >
                                +
                            </button>
                        </div>
                    </div>

                    {/* Default Snapshot Size */}
                    <div className="space-y-3 pt-4 border-t border-border-subtle/50">
                        <div>
                            <div className="text-sm font-bold text-text-main">{t('settings.performance.snapshot_size')}</div>
                            <div className="text-xs text-text-muted mt-0.5">{t('settings.performance.snapshot_size_desc')}</div>
                        </div>
                        <div className="flex gap-4">
                            <div className="flex-1">
                                <label className="text-[10px] text-text-muted uppercase mb-1.5 block font-black">{t('settings.snapshot.width')}</label>
                                <input
                                    type="number"
                                    value={config?.snapshot_width}
                                    onChange={(e) => setSnapshotSize(parseInt(e.target.value), config?.snapshot_height)}
                                    className="w-full bg-bg-main border border-border-subtle rounded-lg px-3 py-2 text-xs text-text-main outline-none focus:border-accent/40 font-mono"
                                />
                            </div>
                            <div className="flex-1">
                                <label className="text-[10px] text-text-muted uppercase mb-1.5 block font-black">{t('settings.snapshot.height')}</label>
                                <input
                                    type="number"
                                    value={config?.snapshot_height}
                                    onChange={(e) => setSnapshotSize(config?.snapshot_width, parseInt(e.target.value))}
                                    className="w-full bg-bg-main border border-border-subtle rounded-lg px-3 py-2 text-xs text-text-main outline-none focus:border-accent/40 font-mono"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </Section>

            {/* 2. Rendering Engine */}
            <Section title={t('settings.advanced.title')} icon={Cpu}>
                <div className="bg-bg-subtle rounded-xl p-5 space-y-4 border border-border-subtle">
                    <div className="flex items-center justify-between">
                        <div>
                            <div className="text-sm font-bold text-text-main">{t('settings.advanced.vello')}</div>
                            <div className="text-xs text-text-muted mt-0.5">{t('settings.advanced.vello_desc')}</div>
                        </div>
                        <div
                            onClick={() => setVelloEnabled(!config?.vello_enabled)}
                            className={`w-10 h-5 rounded-full relative cursor-pointer transition-all duration-300 ${config?.vello_enabled ? "bg-accent shadow-[0_0_10px_rgba(59,130,246,0.3)]" : "bg-bg-card border border-border-subtle"}`}
                        >
                            <motion.div
                                animate={{ x: config?.vello_enabled ? 22 : 4 }}
                                className={`absolute top-1 w-3 h-3 rounded-full shadow-sm ${config?.vello_enabled ? "bg-white" : "bg-text-muted"}`}
                            />
                        </div>
                    </div>
                    {config?.vello_enabled && (
                        <motion.div
                            initial={{ opacity: 0, height: 0 }}
                            animate={{ opacity: 1, height: 'auto' }}
                            className="flex items-center justify-between pt-4 border-t border-border-subtle/30"
                        >
                            <div>
                                <div className="text-sm font-bold text-text-main">{t('settings.advanced.advanced_effects')}</div>
                                <div className="text-xs text-text-muted mt-0.5">{t('settings.advanced.advanced_effects_desc')}</div>
                            </div>
                            <div
                                onClick={() => setVelloAdvancedEffects(!config?.vello_advanced_effects)}
                                className={`w-10 h-5 rounded-full relative cursor-pointer transition-all duration-300 ${config?.vello_advanced_effects ? "bg-accent shadow-[0_0_10px_rgba(59,130,246,0.3)]" : "bg-bg-card border border-border-subtle"}`}
                            >
                                <motion.div
                                    animate={{ x: config?.vello_advanced_effects ? 22 : 4 }}
                                    className={`absolute top-1 w-3 h-3 rounded-full shadow-sm ${config?.vello_advanced_effects ? "bg-white" : "bg-text-muted"}`}
                                />
                            </div>
                        </motion.div>
                    )}
                </div>
            </Section>

            {/* 3. System Actions */}
            <Section title={t('settings.engine.title')} icon={Cpu}>
                <div className="bg-bg-subtle rounded-xl p-5 border border-border-subtle shadow-sm flex gap-3">
                    <button onClick={() => invoke("reveal_logs")} className="flex-1 px-4 py-2.5 text-xs bg-bg-card hover:bg-bg-sidebar rounded-xl border border-border-subtle transition-all flex items-center justify-center gap-2 text-text-main font-bold">
                        <FileText className="w-4 h-4 text-accent/70" />
                        {t('settings.engine.open_log')}
                    </button>
                    <button onClick={() => invoke("clear_logs")} className="flex-1 px-4 py-2.5 text-xs bg-red-500/5 hover:bg-red-500/10 text-red-400 rounded-xl border border-red-500/20 transition-all flex items-center justify-center gap-2 font-bold">
                        <Trash2 className="w-4 h-4" />
                        {t('settings.engine.clear_logs')}
                    </button>
                </div>
            </Section>
        </motion.div>
    );
};

export default AdvancedTab;
