import React, { useState } from "react";
import { Languages, Book } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useConfig } from "../../../hooks/useConfig";
import { invoke } from "@tauri-apps/api/core";

interface GeneralTabProps {
    config: any; // Ideally this should be strongly typed based on useConfig return type
    selectSavePath: () => void;
    setOcrEngine: (engine: string) => void;
    setFontFamily: (font: string) => void;
    fetchConfig: () => void;
}

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <section className="space-y-3">
        <h3 className="text-xs font-bold text-text-muted uppercase tracking-wider pl-1">{title}</h3>
        <div className="space-y-2">{children}</div>
    </section>
);

const GeneralTab: React.FC<GeneralTabProps> = ({ config, selectSavePath, setFontFamily, fetchConfig }) => {
    const { t, i18n } = useTranslation();
    const [showFontHelp, setShowFontHelp] = useState(false);

    const toggleLanguage = () => {
        const newLang = i18n.language === 'zh' ? 'en' : 'zh';
        i18n.changeLanguage(newLang);
    };

    return (
        <motion.div
            key="general"
            initial={{ opacity: 0, x: 5 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -5 }}
            className="space-y-6"
        >
            <Section title="Language">
                <div className="bg-bg-subtle rounded-lg p-4 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Languages className="w-5 h-5 text-text-muted" />
                        <div>
                            <div className="text-sm font-medium text-text-main">
                                {i18n.language === 'zh' ? '中文 (Chinese)' : 'English'}
                            </div>
                            <div className="text-xs text-text-muted">Current Language</div>
                        </div>
                    </div>
                    <button onClick={toggleLanguage} className="px-3 py-1.5 text-xs bg-white/5 hover:bg-white/10 border border-white/10 rounded transition-colors">
                        Switch
                    </button>
                </div>
            </Section>

            <Section title={t('settings.general.save_path') || "Save Path"}>
                <div className="bg-bg-subtle rounded-lg p-4 flex items-center justify-between">
                    <div className="flex-1 min-w-0 mr-4">
                        <div className="text-xs text-text-main font-mono truncate bg-white/5 px-2 py-1 rounded border border-white/5">
                            {config?.save_path || "Default (captures/)"}
                        </div>
                        <div className="text-[10px] text-text-muted mt-1">Select where to save your captures.</div>
                    </div>
                    <button onClick={selectSavePath} className="px-3 py-1.5 text-xs bg-accent text-white font-medium rounded hover:bg-accent/80 transition-all shadow-lg shadow-accent/20">
                        {t('settings.general.browse') || "Browse"}
                    </button>
                </div>
            </Section>

            <Section title="Custom Font">
                <div className="bg-bg-subtle rounded-lg p-4 space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="flex-1 mr-4">
                            <div className="flex items-center gap-2 mb-1">
                                <span className="text-sm font-medium text-text-main">Font Family</span>
                                <button onClick={() => setShowFontHelp(!showFontHelp)} className={`p-1 rounded hover:bg-white/10 transition-colors ${showFontHelp ? 'text-accent' : 'text-text-muted'}`}>
                                    <Book className="w-3.5 h-3.5" />
                                </button>
                            </div>
                            <input
                                type="text"
                                value={config?.font_family || "Segoe UI"}
                                onChange={(e) => setFontFamily(e.target.value)}
                                className="w-full bg-white/5 border border-white/10 rounded px-2 py-1 text-xs text-text-main focus:border-accent/50 outline-none transition-all"
                                placeholder="e.g. MiSans"
                            />
                        </div>
                    </div>
                    <AnimatePresence>
                        {showFontHelp && (
                            <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }} exit={{ height: 0, opacity: 0 }} className="overflow-hidden">
                                <div className="bg-accent/5 border border-accent/20 rounded-md p-2 text-[10px] text-text-muted space-y-1">
                                    <p className="font-medium text-accent">How to add fonts:</p>
                                    <ol className="list-decimal pl-4 space-y-0.5">
                                        <li>Place .ttf/.otf in App Data fonts/ folder.</li>
                                        <li>Restart NexSpot.</li>
                                        <li>Enter the Font Family name above.</li>
                                    </ol>
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </Section>

            <Section title="Engine Configuration (Dual Mode)">
                <div className="bg-bg-subtle rounded-lg p-4 space-y-4">
                    {/* Selection Mode Engine */}
                    <div className="flex items-center justify-between">
                        <div>
                            <div className="text-sm font-medium text-text-main">Selection Mode</div>
                            <div className="text-xs text-text-muted">Standard Capture (Alt+A) - Supports Multi-monitor</div>
                        </div>
                        <select
                            value={config?.selection_engine || "gdi"}
                            onChange={(e) => invoke("set_selection_engine", { engine: e.target.value }).then(fetchConfig)}
                            className="bg-white/5 border border-white/10 rounded px-2 py-1 text-xs text-text-main outline-none focus:border-accent/40"
                        >
                            <option value="gdi">GDI (Recommended)</option>
                            <option value="vello">Vello</option>
                        </select>
                    </div>

                    {/* Snapshot Mode Engine */}
                    <div className="flex items-center justify-between pt-4 border-t border-white/5">
                        <div>
                            <div className="text-sm font-medium text-text-main">Snapshot Mode</div>
                            <div className="text-xs text-text-muted">Fast Capture (Alt+Shift+A) - Single Monitor Only</div>
                        </div>
                        <select
                            value={config?.snapshot_engine || "vello"}
                            onChange={(e) => invoke("set_snapshot_engine", { engine: e.target.value }).then(fetchConfig)}
                            className="bg-white/5 border border-white/10 rounded px-2 py-1 text-xs text-text-main outline-none focus:border-accent/40"
                        >
                            <option value="gdi">GDI</option>
                            <option value="vello">Vello (Recommended)</option>
                        </select>
                    </div>
                </div>
            </Section>
        </motion.div>
    );
};

export default GeneralTab;
