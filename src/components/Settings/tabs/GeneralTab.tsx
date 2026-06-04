import React, { useState } from "react";
import { Languages, Book } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useConfig } from "../../../hooks/useConfig";
import { invoke } from "@tauri-apps/api/core";

interface GeneralTabProps {
    config: any; // Ideally this should be strongly typed based on useConfig return type
    selectSavePath: () => void;
    setFontFamily: (font: string) => void;
    fetchConfig: () => void;
}

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <section className="space-y-2">
        <h3 className="text-[10px] tech-text font-bold text-text-muted uppercase tracking-wider pl-1">{title}</h3>
        <div className="space-y-1.5">{children}</div>
    </section>
);

const GeneralTab: React.FC<GeneralTabProps> = ({ config, selectSavePath, setFontFamily, fetchConfig }) => {
    const { t, i18n } = useTranslation();
    const [showFontHelp, setShowFontHelp] = useState(false);

    const toggleLanguage = async () => {
        const newLang = i18n.language === 'zh' ? 'en' : 'zh';
        try {
            await invoke('set_language', { lang: newLang });
            await i18n.changeLanguage(newLang);
            fetchConfig(); // Refresh local config
        } catch (e) {
            console.error("Failed to persist language:", e);
        }
    };

    return (
        <motion.div
            key="general"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-5"
        >
            <Section title={t('settings.general.language')}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Languages className="w-4 h-4 text-text-muted opacity-60" />
                        <div>
                            <div className="text-[12px] font-bold text-text-main tech-text">
                                {i18n.language === 'zh' ? t('settings.general.lang_zh') : t('settings.general.lang_en')}
                            </div>
                            <div className="text-[9px] tech-text text-text-muted opacity-50 uppercase">{t('settings.general.active_locale')}</div>
                        </div>
                    </div>
                    <button onClick={toggleLanguage} className="px-2 py-1 text-[10px] tech-text font-bold bg-white/5 hover:bg-white/10 border border-white/10 rounded-sm transition-colors text-text-muted hover:text-text-main">
                        {t('settings.general.switch_lang')}
                    </button>
                </div>
            </Section>

            <Section title={t('settings.general.save_path')}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 flex items-center justify-between gap-4">
                    <div className="flex-1 min-w-0">
                        <div className="text-[11px] text-text-main tech-text truncate bg-black/40 px-2 py-1.5 rounded-sm border border-white/5 font-bold">
                            {config?.save_path || `${t('common.root_dir')}/captures/`}
                        </div>
                        <div className="text-[9px] tech-text text-text-muted mt-1.5 opacity-50 uppercase tracking-tighter">{t('settings.general.save_path_desc')}</div>
                    </div>
                    <button onClick={selectSavePath} className="px-3 py-1.5 text-[10px] tech-text font-bold bg-accent text-white rounded-sm hover:bg-accent/80 transition-all shadow-md shadow-accent/10 whitespace-nowrap">
                        {t('settings.general.browse')}
                    </button>
                </div>
            </Section>

            <Section title={t('settings.general.tech_font')}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 space-y-3">
                    <div className="flex items-center justify-between gap-4">
                        <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1.5">
                                <span className="text-[11px] font-bold text-text-main tech-text uppercase">{t('settings.general.family_id')}</span>
                                <button onClick={() => setShowFontHelp(!showFontHelp)} className={`p-1 rounded-sm hover:bg-white/10 transition-colors ${showFontHelp ? 'text-accent' : 'text-text-muted'}`}>
                                    <Book className="w-3 h-3" />
                                </button>
                            </div>
                            <input
                                type="text"
                                value={config?.font_family || "Segoe UI"}
                                onChange={(e) => setFontFamily(e.target.value)}
                                className="w-full bg-black/40 border border-white/10 rounded-sm px-2 py-1.5 text-[11px] font-bold tech-text text-text-main focus:border-accent/40 outline-none transition-all placeholder:opacity-20"
                                placeholder={t('settings.general.family_id_placeholder') || "e.g. MiSans"}
                            />
                        </div>
                    </div>
                    <AnimatePresence>
                        {showFontHelp && (
                            <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }} exit={{ height: 0, opacity: 0 }} className="overflow-hidden">
                                <div className="bg-accent/5 border border-accent/20 rounded-sm p-2 text-[9px] tech-text text-text-muted space-y-1 opacity-80">
                                    <p className="font-bold text-accent uppercase tracking-widest">{t('settings.general.font_protocol')}</p>
                                    <ol className="list-decimal pl-4 space-y-0.5">
                                        <li>{t('settings.general.font_protocol_steps.step1')}</li>
                                        <li>{t('settings.general.font_protocol_steps.step2')}</li>
                                        <li>{t('settings.general.font_protocol_steps.step3')}</li>
                                    </ol>
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </Section>

            <Section title={t('settings.general.engine_config')}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 space-y-3">
                    {/* Selection Mode Engine */}
                    <div className="flex items-center justify-between border-b border-white/5 pb-3">
                        <div>
                            <div className="text-[12px] font-bold text-text-main tech-text uppercase">{t('settings.general.selection_interface')}</div>
                            <div className="text-[9px] tech-text text-text-muted opacity-50 uppercase">{t('settings.general.selection_interface_desc')}</div>
                        </div>
                        <select
                            value={config?.selection_engine || "gdi"}
                            onChange={(e) => invoke("set_selection_engine", { engine: e.target.value }).then(fetchConfig)}
                            className="bg-black/40 border border-white/10 rounded-sm px-2 py-1 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 cursor-pointer"
                        >
                            <option value="gdi">{t('settings.general.gdi_legacy')}</option>
                            <option value="vello">{t('settings.general.vello_compute')}</option>
                        </select>
                    </div>

                    {/* Snapshot Mode Engine */}
                    <div className="flex items-center justify-between pt-1">
                        <div>
                            <div className="text-[12px] font-bold text-text-main tech-text uppercase">{t('settings.general.fast_capture')}</div>
                            <div className="text-[9px] tech-text text-text-muted opacity-50 uppercase">{t('settings.general.fast_capture_desc')}</div>
                        </div>
                        <select
                            value={config?.snapshot_engine || "vello"}
                            onChange={(e) => invoke("set_snapshot_engine", { engine: e.target.value }).then(fetchConfig)}
                            className="bg-black/40 border border-white/10 rounded-sm px-2 py-1 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 cursor-pointer"
                        >
                            <option value="gdi">{t('settings.general.gdi_fallback')}</option>
                            <option value="vello">{t('settings.general.vello_accelerated')}</option>
                        </select>
                    </div>
                </div>
            </Section>

            <Section title={t('settings.general.quick_save')}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-3 flex items-center justify-between">
                    <div>
                        <div className="text-[12px] font-bold text-text-main tech-text uppercase">{t('settings.general.quick_save')}</div>
                        <div className="text-[9px] tech-text text-text-muted opacity-50 uppercase">{t('settings.general.quick_save_desc')}</div>
                    </div>
                    <div
                        onClick={() => invoke("set_quick_save", { enabled: !config?.quick_save }).then(fetchConfig)}
                        className={`w-10 h-5 rounded-sm relative cursor-pointer transition-all duration-200 border ${config?.quick_save ? "bg-accent/20 border-accent/40" : "bg-black/40 border-white/10"}`}
                    >
                        <motion.div
                            animate={{ x: config?.quick_save ? 22 : 4 }}
                            className={`absolute top-1 w-3 h-3 rounded-sm ${config?.quick_save ? "bg-accent" : "bg-text-muted/40"}`}
                        />
                    </div>
                </div>
            </Section>
        </motion.div>
    );
};

export default GeneralTab;
