import React from "react";
import { FileText, Trash2 } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Row, Segmented, Toggle, Stepper, TextField, SectionTitle } from "../atoms";

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

const STYLES = ['Default', 'Neon', 'PaperCut', 'Sketch', 'Glass'];

const AdvancedTab: React.FC<AdvancedTabProps> = ({
    config, setVelloEnabled, setVelloAestheticStyle, setVelloAdvancedEffects,
    setSnapshotSize, setJpgQuality, setConcurrency, setDefaultExportFormat,
}) => {
    const { t } = useTranslation();
    const velloOn = !!config?.vello_enabled;

    return (
        <motion.div key="advanced" initial={{ y: 7 }} animate={{ y: 0 }} transition={{ duration: 0.18 }} className="max-w-[620px] flex flex-col gap-7">
            {/* Export */}
            <section>
                <SectionTitle>{t('settings.performance.title')}</SectionTitle>
                <Row title={t('settings.performance.jpg_quality')} hint={t('settings.performance.jpg_quality_desc')}>
                    <div className="flex items-center gap-3">
                        <input
                            type="range" min={40} max={100}
                            value={config?.jpg_quality || 90}
                            onChange={(e) => setJpgQuality(parseInt(e.target.value))}
                            className="w-[140px] accent-[var(--accent)] cursor-pointer"
                        />
                        <span className="mono text-[12.5px] font-semibold text-ink w-9 text-right">{config?.jpg_quality || 90}</span>
                    </div>
                </Row>
                <Row title={t('settings.performance.export_format')} hint={t('settings.performance.export_format_desc')}>
                    <Segmented
                        value={config?.default_export_format || 'png'}
                        onChange={setDefaultExportFormat}
                        options={[{ value: 'png', label: 'PNG' }, { value: 'jpg', label: 'JPG' }]}
                    />
                </Row>
                <Row title={t('settings.performance.concurrency')} hint={t('settings.performance.concurrency_desc')}>
                    <Stepper value={config?.concurrency || 4} min={1} max={8} onChange={setConcurrency} />
                </Row>
                <Row title={t('settings.performance.snapshot_size')} hint={t('settings.performance.snapshot_size_desc')} last>
                    <div className="flex items-center gap-2">
                        <TextField mono type="number" value={config?.snapshot_width ?? 800}
                            onChange={(e) => setSnapshotSize(parseInt(e.target.value), config?.snapshot_height)} className="w-[72px]" />
                        <span className="text-faint">×</span>
                        <TextField mono type="number" value={config?.snapshot_height ?? 600}
                            onChange={(e) => setSnapshotSize(config?.snapshot_width, parseInt(e.target.value))} className="w-[72px]" />
                    </div>
                </Row>
            </section>

            {/* Vello engine */}
            <section>
                <SectionTitle>{t('settings.advanced.aesthetic_protocol')}</SectionTitle>
                <Row title={t('settings.advanced.vello')} hint={t('settings.advanced.vello_desc')} last={!velloOn}>
                    <Toggle checked={velloOn} onChange={setVelloEnabled} />
                </Row>
                {velloOn && (
                    <>
                        <Row title={t('settings.advanced.visual_presets')} align="start">
                            <div className="flex flex-wrap gap-1.5 justify-end max-w-[320px]">
                                {STYLES.map((s) => {
                                    const active = config?.vello_aesthetic_style === s;
                                    return (
                                        <button
                                            key={s}
                                            onClick={() => setVelloAestheticStyle(s)}
                                            className={`px-2.5 py-1 rounded-full text-[11.5px] font-semibold border transition-colors ${active ? 'bg-accent-soft border-accent-line text-accent' : 'bg-bg-2 border-line text-muted hover:text-ink'}`}
                                        >
                                            {t(`settings.aesthetics.styles.${s.toLowerCase()}`)}
                                        </button>
                                    );
                                })}
                            </div>
                        </Row>
                        <Row title={t('settings.advanced.advanced_effects')} last>
                            <Toggle checked={!!config?.vello_advanced_effects} onChange={setVelloAdvancedEffects} />
                        </Row>
                    </>
                )}
            </section>

            {/* Logs */}
            <section>
                <SectionTitle>{t('settings.advanced.maintenance')}</SectionTitle>
                <div className="flex gap-2 pt-1">
                    <button onClick={() => invoke('reveal_logs')}
                        className="flex items-center gap-1.5 h-9 px-3.5 rounded-btn bg-bg-2 border border-line text-[12.5px] font-semibold text-ink hover:border-line-2 transition-colors">
                        <FileText className="w-4 h-4 text-muted" /> {t('settings.advanced.reveal_logs')}
                    </button>
                    <button onClick={() => invoke('clear_logs')}
                        className="flex items-center gap-1.5 h-9 px-3.5 rounded-btn bg-bad-soft border border-bad/30 text-[12.5px] font-semibold text-bad hover:bg-bad/15 transition-colors">
                        <Trash2 className="w-4 h-4" /> {t('settings.advanced.clear_logs')}
                    </button>
                </div>
            </section>
        </motion.div>
    );
};

export default AdvancedTab;
