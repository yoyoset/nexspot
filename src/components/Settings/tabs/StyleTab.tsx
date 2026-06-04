import React from "react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useConfig } from "../../../hooks/useConfig";
import { Monitor, Sun, Moon } from "lucide-react";

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <section className="space-y-2.5">
        <h3 className="text-[10px] font-bold tech-text text-text-muted uppercase tracking-widest pl-1">{title}</h3>
        <div className="space-y-2">{children}</div>
    </section>
);

const StyleTab: React.FC = () => {
    const { t } = useTranslation();
    const { config, setTheme, setAccentColor, setVelloAestheticStyle } = useConfig();

    const themes = [
        { id: 'system', icon: Monitor, label: t('settings.style.theme_system') },
        { id: 'light', icon: Sun, label: t('settings.style.theme_light') },
        { id: 'dark', icon: Moon, label: t('settings.style.theme_dark') },
    ];

    const colors = [
        { name: t('settings.style.colors.blue_print'), value: '#3b82f6' },
        { name: t('settings.style.colors.void_purple'), value: '#a855f7' },
        { name: t('settings.style.colors.crimson_data'), value: '#f43f5e' },
        { name: t('settings.style.colors.amber_link'), value: '#f97316' },
        { name: t('settings.style.colors.matrix_emerald'), value: '#10b981' },
        { name: t('settings.style.colors.cyber_sky'), value: '#0ea5e9' },
        { name: t('settings.style.colors.tiffany_code'), value: '#0ABAB5' },
    ];

    return (
        <motion.div
            key="style"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-6 pb-6"
        >
            <Section title={t('settings.style.theme_protocol')}>
                <div className="grid grid-cols-3 gap-2">
                    {themes.map((theme) => (
                        <button
                            key={theme.id}
                            onClick={() => setTheme(theme.id)}
                            className={`flex flex-col items-center gap-2 p-3 rounded-sm border transition-all ${config?.theme === theme.id
                                ? "bg-accent/10 border-accent/40 text-accent font-bold"
                                : "bg-black/20 border-white/5 text-text-muted hover:border-white/20 hover:text-text-main"
                                }`}
                        >
                            <theme.icon className="w-4 h-4 opacity-70" />
                            <span className="text-[9px] font-bold tech-text tracking-widest uppercase">{theme.label}</span>
                        </button>
                    ))}
                </div>
            </Section>

            <Section title={t('settings.style.accent_config')}>
                <div className="flex flex-wrap gap-2 px-1">
                    {colors.map((color) => (
                        <button
                            key={color.name}
                            onClick={() => setAccentColor(color.value)}
                            className="relative group p-0.5 border border-transparent hover:border-white/20 rounded-sm transition-all"
                        >
                            <div
                                className={`w-6 h-6 rounded-sm shadow-inner transition-transform`}
                                style={{ backgroundColor: color.value }}
                            />
                            {config?.accent_color === color.value && (
                                <motion.div
                                    layoutId="color-check"
                                    className="absolute inset-0 flex items-center justify-center text-white"
                                >
                                    <div className="w-1 h-3 bg-white" />
                                </motion.div>
                            )}
                        </button>
                    ))}
                </div>
            </Section>

            <Section title={t('settings.style.render_preset')}>
                <div className="grid grid-cols-2 lg:grid-cols-5 gap-2">
                    {(['Default', 'Neon', 'PaperCut', 'Sketch', 'Glass'] as any[]).map((style) => (
                        <button
                            key={style}
                            onClick={() => setVelloAestheticStyle(style)}
                            className={`flex flex-col items-center gap-2 py-2 px-1 rounded-sm border transition-all ${config?.vello_aesthetic_style === style
                                ? "bg-accent/10 border-accent/40 text-accent font-bold"
                                : "bg-black/20 border-white/5 text-text-muted hover:border-white/20 hover:text-text-main"
                                }`}
                        >
                            <span className="text-[9px] font-bold tracking-widest uppercase tech-text">
                                {t(`settings.aesthetics.styles.${style.toLowerCase()}`)}
                            </span>
                        </button>
                    ))}
                </div>
            </Section>
        </motion.div>
    );
};

export default StyleTab;
