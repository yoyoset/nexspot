import React, { useEffect, useState } from "react";
import { FolderOpen } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Row, Segmented, Toggle, TextField } from "../atoms";

interface OcrLanguage {
    tag: string;
    display_name: string;
}

interface GeneralTabProps {
    config: any;
    selectSavePath: () => void;
    setFontFamily: (font: string) => void;
    fetchConfig: () => void;
}

const GeneralTab: React.FC<GeneralTabProps> = ({ config, selectSavePath, setFontFamily, fetchConfig }) => {
    const { t, i18n } = useTranslation();
    const [ocrLangs, setOcrLangs] = useState<OcrLanguage[]>([]);

    useEffect(() => {
        invoke<OcrLanguage[]>('get_ocr_languages').then(setOcrLangs).catch(() => setOcrLangs([]));
    }, []);

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

            <Row title={t('settings.general.quick_save')} hint={t('settings.general.quick_save_desc')}>
                <Toggle checked={!!config?.quick_save} onChange={(v) => invoke('set_quick_save', { enabled: v }).then(fetchConfig)} />
            </Row>

            <Row title={t('settings.general.ocr_language')} hint={t('settings.general.ocr_language_desc')} last>
                <select
                    value={config?.ocr_language || 'auto'}
                    onChange={(e) => invoke('set_ocr_language', { lang: e.target.value }).then(fetchConfig)}
                    className="h-9 bg-bg-3 border border-line rounded-btn px-3 text-[12.5px] text-ink outline-none focus:border-accent transition-colors cursor-pointer max-w-[220px]"
                >
                    <option value="auto">{t('settings.general.ocr_language_auto')}</option>
                    {ocrLangs.map((l) => (
                        <option key={l.tag} value={l.tag}>{l.display_name}</option>
                    ))}
                </select>
            </Row>
        </motion.div>
    );
};

export default GeneralTab;
