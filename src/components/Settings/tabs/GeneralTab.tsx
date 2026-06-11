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

interface PaddleStatus {
    installed: boolean;
    dir: string;
    version: string | null;
    languages: string[];
}

interface PaddleUpdateInfo {
    current: string | null;
    latest: string | null;
    has_update: boolean;
}

interface PaddleProgress {
    phase: 'download' | 'extract' | 'done' | 'error';
    downloaded: number;
    total: number;
}

const PADDLE_LANG_NAMES: Record<string, string> = {
    chinese: '中文（简体）',
    chinese_cht: '中文（繁體）',
    en: 'English',
    japan: '日本語',
    korean: '한국어',
    latin: 'Latin',
    cyrillic: 'Кириллица',
    devanagari: 'देवनागरी',
};

interface GeneralTabProps {
    config: any;
    selectSavePath: () => void;
    setFontFamily: (font: string) => void;
    fetchConfig: () => void;
}

const GeneralTab: React.FC<GeneralTabProps> = ({ config, selectSavePath, setFontFamily, fetchConfig }) => {
    const { t, i18n } = useTranslation();
    const [ocrLangs, setOcrLangs] = useState<OcrLanguage[]>([]);
    const [paddle, setPaddle] = useState<PaddleStatus | null>(null);
    const [paddleUpdate, setPaddleUpdate] = useState<PaddleUpdateInfo | null>(null);
    const [paddleProgress, setPaddleProgress] = useState<PaddleProgress | null>(null);
    const [paddleError, setPaddleError] = useState<string | null>(null);

    const refreshPaddle = () => invoke<PaddleStatus>('get_paddle_status').then((s) => {
        setPaddle(s);
        if (s.installed) {
            invoke<PaddleUpdateInfo>('check_paddle_update').then(setPaddleUpdate).catch(() => {});
        }
    }).catch(() => setPaddle(null));

    useEffect(() => {
        invoke<OcrLanguage[]>('get_ocr_languages').then(setOcrLangs).catch(() => setOcrLangs([]));
        refreshPaddle();

        let unlisten: (() => void) | undefined;
        (async () => {
            const { listen } = await import('@tauri-apps/api/event');
            unlisten = await listen<PaddleProgress>('paddle://progress', (e) => {
                setPaddleProgress(e.payload.phase === 'done' || e.payload.phase === 'error' ? null : e.payload);
            });
        })();
        return () => { if (unlisten) unlisten(); };
    }, []);

    const installPaddle = async () => {
        setPaddleError(null);
        setPaddleProgress({ phase: 'download', downloaded: 0, total: 0 });
        try {
            await invoke('download_paddle_component');
            setPaddleUpdate(null);
        } catch (e) {
            setPaddleError(String(e));
        } finally {
            setPaddleProgress(null);
            refreshPaddle();
        }
    };

    const ocrEngine = config?.ocr_engine || 'winrt';

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

            <Row title={t('settings.general.ocr_engine')}>
                <Segmented
                    value={ocrEngine}
                    onChange={(v) => { invoke('set_ocr_engine', { engine: v }).then(fetchConfig); refreshPaddle(); }}
                    options={[
                        { value: 'winrt', label: t('settings.general.ocr_engine_winrt') },
                        { value: 'paddle', label: 'PaddleOCR' },
                    ]}
                />
            </Row>

            {ocrEngine === 'winrt' ? (
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
            ) : (
                <>
                    <Row
                        title={t('settings.general.paddle_component')}
                        hint={paddleError
                            ? `${t('settings.general.paddle_error')}: ${paddleError}`
                            : paddle?.installed
                                ? t('settings.general.paddle_installed', { version: paddle.version || '?', count: paddle.languages.length })
                                : t('settings.general.paddle_not_installed')}
                    >
                        <div className="flex items-center gap-2">
                            {paddleProgress ? (
                                <button disabled className="h-9 px-3.5 rounded-btn bg-accent-soft text-accent text-[12.5px] font-semibold cursor-wait min-w-[130px]">
                                    {paddleProgress.phase === 'extract'
                                        ? t('settings.general.paddle_extracting')
                                        : t('settings.general.paddle_downloading', {
                                            pct: paddleProgress.total > 0
                                                ? Math.round((paddleProgress.downloaded / paddleProgress.total) * 100)
                                                : Math.round(paddleProgress.downloaded / 1048576),
                                        })}
                                </button>
                            ) : !paddle?.installed ? (
                                <button
                                    onClick={installPaddle}
                                    className="h-9 px-3.5 rounded-btn bg-accent text-on-accent text-[12.5px] font-semibold hover:bg-[var(--accent-press)] transition-colors"
                                >
                                    {t('settings.general.paddle_download')}
                                </button>
                            ) : paddleUpdate?.has_update ? (
                                <button
                                    onClick={installPaddle}
                                    className="h-9 px-3.5 rounded-btn bg-accent text-on-accent text-[12.5px] font-semibold hover:bg-[var(--accent-press)] transition-colors"
                                >
                                    {t('settings.general.paddle_update_to', { ver: paddleUpdate.latest })}
                                </button>
                            ) : (
                                <span className="text-[12px] font-semibold text-ok px-1">{t('settings.general.paddle_latest')}</span>
                            )}
                            <button
                                onClick={() => paddle && invoke('open_folder', { path: paddle.dir })}
                                title={t('settings.general.paddle_open_dir')}
                                className="h-9 px-2.5 rounded-btn text-[12.5px] text-muted hover:text-ink hover:bg-bg-2 transition-colors"
                            >
                                <FolderOpen className="w-4 h-4" />
                            </button>
                        </div>
                    </Row>
                    <Row title={t('settings.general.ocr_language')} last>
                        <select
                            value={config?.ocr_paddle_language || 'chinese'}
                            onChange={(e) => invoke('set_paddle_language', { lang: e.target.value }).then(fetchConfig)}
                            disabled={!paddle?.installed}
                            className="h-9 bg-bg-3 border border-line rounded-btn px-3 text-[12.5px] text-ink outline-none focus:border-accent transition-colors cursor-pointer max-w-[220px] disabled:opacity-40"
                        >
                            {(paddle?.languages.length ? paddle.languages : ['chinese']).map((l) => (
                                <option key={l} value={l}>{PADDLE_LANG_NAMES[l] || l}</option>
                            ))}
                        </select>
                    </Row>
                </>
            )}
        </motion.div>
    );
};

export default GeneralTab;
