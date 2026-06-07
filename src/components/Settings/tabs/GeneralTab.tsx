import React from "react";
import { FolderOpen } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Row, Segmented, Toggle, TextField } from "../atoms";

interface GeneralTabProps {
    config: any;
    selectSavePath: () => void;
    setFontFamily: (font: string) => void;
    fetchConfig: () => void;
}

const GeneralTab: React.FC<GeneralTabProps> = ({ config, selectSavePath, setFontFamily, fetchConfig }) => {
    const { t, i18n } = useTranslation();

    const setLanguage = async (lang: string) => {
        if (lang === i18n.language) return;
        try {
            await invoke('set_language', { lang });
            await i18n.changeLanguage(lang);
            fetchConfig();
        } catch (e) {
            console.error("Failed to persist language:", e);
        }
    };

    const setEngine = (cmd: string, engine: string) => invoke(cmd, { engine }).then(fetchConfig);

    return (
        <motion.div key="general" initial={{ y: 7 }} animate={{ y: 0 }} transition={{ duration: 0.18 }} className="max-w-[620px]">
            <Row title={t('settings.general.save_path')} hint={config?.save_path || `${t('common.root_dir')}/captures/`}>
                <button
                    onClick={selectSavePath}
                    className="flex items-center gap-1.5 h-9 px-3 rounded-btn bg-bg-3 border border-line text-[12.5px] font-semibold text-ink hover:border-line-2 transition-colors"
                >
                    <FolderOpen className="w-4 h-4 text-muted" /> {t('settings.general.browse')}
                </button>
            </Row>

            <Row title={t('settings.general.tech_font')} hint={t('settings.general.family_id')}>
                <TextField
                    value={config?.font_family || "Segoe UI"}
                    onChange={(e) => setFontFamily(e.target.value)}
                    placeholder="e.g. MiSans"
                    className="w-[200px]"
                />
            </Row>

            <Row title={t('settings.general.language')}>
                <Segmented
                    value={i18n.language === 'zh' ? 'zh' : 'en'}
                    onChange={setLanguage}
                    options={[
                        { value: 'zh', label: t('settings.general.lang_zh') },
                        { value: 'en', label: t('settings.general.lang_en') },
                    ]}
                />
            </Row>

            <Row title={t('settings.general.selection_interface')} hint={t('settings.general.selection_interface_desc')}>
                <Segmented
                    value={config?.selection_engine || 'gdi'}
                    onChange={(v) => setEngine('set_selection_engine', v)}
                    options={[
                        { value: 'gdi', label: 'GDI' },
                        { value: 'vello', label: 'Vello' },
                    ]}
                />
            </Row>

            <Row title={t('settings.general.fast_capture')} hint={t('settings.general.fast_capture_desc')}>
                <Segmented
                    value={config?.snapshot_engine || 'vello'}
                    onChange={(v) => setEngine('set_snapshot_engine', v)}
                    options={[
                        { value: 'gdi', label: 'GDI' },
                        { value: 'vello', label: 'Vello' },
                    ]}
                />
            </Row>

            <Row title={t('settings.general.quick_save')} hint={t('settings.general.quick_save_desc')} last>
                <Toggle checked={!!config?.quick_save} onChange={(v) => invoke('set_quick_save', { enabled: v }).then(fetchConfig)} />
            </Row>
        </motion.div>
    );
};

export default GeneralTab;
