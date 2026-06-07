import React from "react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useConfig } from "../../../hooks/useConfig";
import { Monitor, Sun, Moon, Check } from "lucide-react";
import { Row, Segmented } from "../atoms";

const ACCENTS = [
    { name: 'Periwinkle', value: '#7a6ff2' },
    { name: 'Blue', value: '#4f8cff' },
    { name: 'Teal', value: '#16b8a6' },
    { name: 'Amber', value: '#f59e0b' },
    { name: 'Rose', value: '#f4517b' },
    { name: 'Green', value: '#46b86a' },
];

const StyleTab: React.FC = () => {
    const { t } = useTranslation();
    const { config, setTheme, setAccentColor } = useConfig();

    return (
        <motion.div key="style" initial={{ y: 7 }} animate={{ y: 0 }} transition={{ duration: 0.18 }} className="max-w-[620px]">
            <Row title={t('settings.style.theme_protocol')}>
                <Segmented
                    value={config?.theme || 'system'}
                    onChange={setTheme}
                    options={[
                        { value: 'light', label: <span className="flex items-center gap-1.5"><Sun className="w-3.5 h-3.5" />{t('settings.style.theme_light')}</span> },
                        { value: 'dark', label: <span className="flex items-center gap-1.5"><Moon className="w-3.5 h-3.5" />{t('settings.style.theme_dark')}</span> },
                        { value: 'system', label: <span className="flex items-center gap-1.5"><Monitor className="w-3.5 h-3.5" />{t('settings.style.theme_system')}</span> },
                    ]}
                />
            </Row>

            <Row title={t('settings.style.accent_config')} last align="center">
                <div className="flex items-center gap-2.5">
                    {ACCENTS.map((c) => {
                        const active = (config?.accent_color || '').toLowerCase() === c.value.toLowerCase();
                        return (
                            <button
                                key={c.value}
                                onClick={() => setAccentColor(c.value)}
                                title={c.name}
                                className="relative w-9 h-9 rounded-[10px] transition-transform hover:scale-105"
                                style={{
                                    background: c.value,
                                    boxShadow: active ? `0 0 0 2px var(--bg1), 0 0 0 4px ${c.value}` : 'none',
                                }}
                            >
                                {active && <Check className="w-4 h-4 text-white absolute inset-0 m-auto drop-shadow" strokeWidth={3} />}
                            </button>
                        );
                    })}
                </div>
            </Row>
        </motion.div>
    );
};

export default StyleTab;
