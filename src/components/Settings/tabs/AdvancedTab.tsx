import React from "react";
import { FileText, Trash2, Cpu, Image as ImageIcon, Zap, Layers, Palette } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface AdvancedTabProps {
    config: any;
    setVelloEnabled: (enabled: boolean) => void;
    setVelloAestheticStyle: (style: any) => void;
    setVelloAdvancedEffects: (enabled: boolean) => void;
    setSnapshotEnabled: (enabled: boolean) => void;
    setSnapshotSize: (width: number, height: number) => void;
    setJpgQuality: (quality: number) => void;
    setConcurrency: (concurrency: number) => void;
    setDefaultExportFormat: (format: string) => void;
}

const styles = [
    { id: 'Default', color: 'bg-blue-400' },
    { id: 'Neon', color: 'bg-pink-500 shadow-[0_0_8px_rgba(236,72,153,0.5)]' },
    { id: 'PaperCut', color: 'bg-white shadow-[2px_2px_4px_rgba(0,0,0,0.3)]' },
    { id: 'Sketch', color: 'bg-amber-100 border border-amber-800/30' },
    { id: 'Glass', color: 'bg-white/30 backdrop-blur-sm border border-white/20' },
];

const Section = ({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) => (
    <section className="space-y-2.5">
        <div className="flex items-center gap-2 pl-1">
            <Icon className="w-3.5 h-3.5 text-accent opacity-70" />
            <h3 className="text-[10px] tech-text font-bold text-text-muted uppercase tracking-wider">{title}</h3>
        </div>
        <div className="space-y-2">{children}</div>
    </section>
);

const AdvancedTab: React.FC<AdvancedTabProps> = ({
    config,
    setVelloEnabled,
    setVelloAestheticStyle,
    setVelloAdvancedEffects,
    setSnapshotEnabled,
    setSnapshotSize,
    setJpgQuality,
    setConcurrency,
    setDefaultExportFormat
}) => {
    const { t } = useTranslation();

    return (
        <motion.div
            key="advanced"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-6 pb-6"
        >
            {/* 1. Performance & Quality */}
            <Section title={t('settings.performance.title')} icon={Zap}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-4 space-y-5">
                    {/* JPG Quality */}
                    <div className="space-y-2.5">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-[11px] font-bold text-text-main tech-text uppercase flex items-center gap-2">
                                    <ImageIcon className="w-3.5 h-3.5 text-accent opacity-50" />
                                    {t('settings.performance.jpg_quality')}
                                </div>
                                <div className="text-[9px] tech-text text-text-muted mt-1 opacity-50 uppercase tracking-tight">{t('settings.performance.jpg_quality_desc')}</div>
                            </div>
                            <span className="text-[11px] font-bold tech-text text-accent">{t('settings.performance.compression')}: {config?.jpg_quality}%</span>
                        </div>
                        <input
                            type="range"
                            min="10"
                            max="100"
                            value={config?.jpg_quality || 90}
                            onChange={(e) => setJpgQuality(parseInt(e.target.value))}
                            className="w-full accent-accent bg-black/40 h-1 rounded-sm appearance-none cursor-pointer border border-white/5"
                        />
                    </div>

                    {/* Default Export Format */}
                    <div className="flex items-center justify-between pt-4 border-t border-white/5">
                        <div>
                            <div className="text-[11px] font-bold text-text-main tech-text uppercase flex items-center gap-2">
                                <FileText className="w-3.5 h-3.5 text-accent opacity-50" />
                                {t('settings.performance.export_format')}
                            </div>
                            <div className="text-[9px] tech-text text-text-muted mt-1 opacity-50 uppercase tracking-tight">{t('settings.performance.export_format_desc')}</div>
                        </div>
                        <div className="flex bg-black/40 border border-white/10 rounded-sm p-0.5">
                            {['png', 'jpg'].map((f) => (
                                <button
                                    key={f}
                                    onClick={() => setDefaultExportFormat(f)}
                                    className={`px-3 py-1 text-[9px] tech-text font-bold uppercase transition-all rounded-sm ${
                                        (config?.default_export_format || 'png') === f
                                            ? 'bg-accent text-white shadow-sm'
                                            : 'text-text-muted hover:text-text-main hover:bg-white/5'
                                    }`}
                                >
                                    {f}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Concurrency */}
                    <div className="flex items-center justify-between pt-4 border-t border-white/5">
                        <div>
                            <div className="text-[11px] font-bold text-text-main tech-text uppercase flex items-center gap-2">
                                <Layers className="w-3.5 h-3.5 text-accent opacity-50" />
                                {t('settings.performance.concurrency')}
                            </div>
                            <div className="text-[9px] tech-text text-text-muted mt-1 opacity-50 uppercase tracking-tight">{t('settings.performance.concurrency_desc')}</div>
                        </div>
                        <div className="flex items-center gap-2 bg-black/40 border border-white/10 rounded-sm p-1">
                            <button
                                onClick={() => setConcurrency(Math.max(1, (config?.concurrency || 1) - 1))}
                                className="w-5 h-5 flex items-center justify-center hover:bg-white/5 rounded-sm text-text-muted hover:text-text-main transition-colors font-bold text-[12px]"
                            >
                                -
                            </button>
                            <span className="w-6 text-center text-[11px] font-bold tech-text text-text-main">{config?.concurrency || 4}</span>
                            <button
                                onClick={() => setConcurrency(Math.min(32, (config?.concurrency || 4) + 1))}
                                className="w-5 h-5 flex items-center justify-center hover:bg-white/5 rounded-sm text-text-muted hover:text-text-main transition-colors font-bold text-[12px]"
                            >
                                +
                            </button>
                        </div>
                    </div>

                    {/* Default Snapshot Size */}
                    <div className="space-y-2.5 pt-4 border-t border-white/5">
                        <div>
                            <div className="text-[11px] font-bold text-text-main tech-text uppercase">{t('settings.performance.snapshot_size')}</div>
                            <div className="text-[9px] tech-text text-text-muted mt-1 opacity-50 uppercase tracking-tight">{t('settings.performance.snapshot_size_desc')}</div>
                        </div>
                        <div className="flex gap-3">
                            <div className="flex-1">
                                <label className="text-[9px] tech-text text-text-muted uppercase mb-1.5 block font-bold opacity-40">{t('settings.performance.dim_x')}</label>
                                <input
                                    type="number"
                                    value={config?.snapshot_width}
                                    onChange={(e) => setSnapshotSize(parseInt(e.target.value), config?.snapshot_height)}
                                    className="w-full bg-black/40 border border-white/10 rounded-sm px-2 py-1.5 text-[11px] font-bold text-text-main outline-none focus:border-accent/40 tech-text"
                                />
                            </div>
                            <div className="flex-1">
                                <label className="text-[9px] tech-text text-text-muted uppercase mb-1.5 block font-bold opacity-40">{t('settings.performance.dim_y')}</label>
                                <input
                                    type="number"
                                    value={config?.snapshot_height}
                                    onChange={(e) => setSnapshotSize(config?.snapshot_width, parseInt(e.target.value))}
                                    className="w-full bg-black/40 border border-white/10 rounded-sm px-2 py-1.5 text-[11px] font-bold text-text-main outline-none focus:border-accent/40 tech-text"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </Section>

            {/* 2. Rendering Engine & Aesthetic Styles */}
            <Section title={t('settings.advanced.aesthetic_protocol')} icon={Palette}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-4 space-y-5">
                    <div className="flex items-center justify-between">
                        <div>
                            <div className="text-[11px] font-bold text-text-main tech-text uppercase">{t('settings.advanced.vello')}</div>
                            <div className="text-[9px] tech-text text-text-muted mt-1 opacity-50 uppercase tracking-tight leading-relaxed">{t('settings.advanced.vello_desc')}</div>
                        </div>
                        <div
                            onClick={() => setVelloEnabled(!config?.vello_enabled)}
                            className={`w-10 h-5 rounded-sm relative cursor-pointer transition-all duration-200 border ${config?.vello_enabled ? "bg-accent/20 border-accent/40" : "bg-black/40 border-white/10"}`}
                        >
                            <motion.div
                                animate={{ x: config?.vello_enabled ? 22 : 4 }}
                                className={`absolute top-1 w-3 h-3 rounded-sm ${config?.vello_enabled ? "bg-accent" : "bg-text-muted/40"}`}
                            />
                        </div>
                    </div>

                    <div className="space-y-3 pt-2">
                        <label className="text-[9px] tech-text text-text-muted uppercase font-bold tracking-widest pl-1 opacity-40">{t('settings.advanced.visual_presets')}</label>
                        <div className="grid grid-cols-2 gap-2">
                            {styles.map((style) => (
                                <div
                                    key={style.id}
                                    onClick={() => setVelloAestheticStyle(style.id)}
                                    className={`p-2.5 rounded-sm border transition-all cursor-pointer flex items-center gap-3 group ${config?.vello_aesthetic_style === style.id
                                        ? "bg-accent/5 border-accent/40"
                                        : "bg-black/20 border-white/5 hover:border-white/20"
                                        }`}
                                >
                                    <div className={`w-5 h-5 rounded-sm ${style.color} shrink-0 border border-white/5`} />
                                    <div className="flex flex-col min-w-0">
                                        <span className={`text-[10px] font-bold tech-text uppercase ${config?.vello_aesthetic_style === style.id ? 'text-accent' : 'text-text-main'}`}>
                                            {t(`settings.aesthetics.styles.${style.id.toLowerCase()}`)}
                                        </span>
                                        <span className="text-[8px] tech-text text-text-muted mt-0.5 opacity-40 uppercase truncate">
                                            {t(`settings.aesthetics.styles.${style.id.toLowerCase()}_desc`)}
                                        </span>
                                    </div>
                                    {config?.vello_aesthetic_style === style.id && (
                                        <div className="ml-auto w-1 h-3 bg-accent" />
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="flex items-center justify-between pt-4 border-t border-white/5">
                        <div>
                            <div className="text-[10px] font-bold text-text-main tech-text uppercase tracking-wider opacity-60">{t('settings.advanced.advanced_effects')}</div>
                        </div>
                        <div
                            onClick={() => setVelloAdvancedEffects(!config?.vello_advanced_effects)}
                            className={`w-10 h-5 rounded-sm relative cursor-pointer transition-all duration-200 border ${config?.vello_advanced_effects ? "bg-accent/20 border-accent/40" : "bg-black/40 border-white/10"}`}
                        >
                            <motion.div
                                animate={{ x: config?.vello_advanced_effects ? 22 : 4 }}
                                className={`absolute top-1 w-3 h-3 rounded-sm ${config?.vello_advanced_effects ? "bg-accent" : "bg-text-muted/40"}`}
                            />
                        </div>
                    </div>
                </div>
            </Section>

            {/* 3. System Actions */}
            <Section title={t('settings.advanced.maintenance')} icon={Cpu}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 flex gap-2">
                    <button onClick={() => invoke("reveal_logs")} className="flex-1 px-3 py-2 text-[10px] tech-text font-bold bg-white/5 hover:bg-white/10 border border-white/10 rounded-sm transition-all flex items-center justify-center gap-2 text-text-muted hover:text-text-main">
                        <FileText className="w-3.5 h-3.5 opacity-50" />
                        {t('settings.advanced.reveal_logs')}
                    </button>
                    <button onClick={() => invoke("clear_logs")} className="flex-1 px-3 py-2 text-[10px] tech-text font-bold bg-red-500/5 hover:bg-red-500/10 text-red-500 border border-red-500/20 rounded-sm transition-all flex items-center justify-center gap-2">
                        <Trash2 className="w-3.5 h-3.5" />
                        {t('settings.advanced.clear_logs')}
                    </button>
                </div>
            </Section>
        </motion.div>
    );
};

export default AdvancedTab;
