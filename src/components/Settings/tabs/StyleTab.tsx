import React from "react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useConfig } from "../../../hooks/useConfig";
import { Monitor, Sun, Moon, Check } from "lucide-react";

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <section className="space-y-3">
        <h3 className="text-xs font-bold text-text-muted uppercase tracking-wider pl-1">{title}</h3>
        <div className="space-y-2">{children}</div>
    </section>
);

const StyleTab: React.FC = () => {
    const { t } = useTranslation();
    const { config, setTheme, setAccentColor } = useConfig();

    const themes = [
        { id: 'system', icon: Monitor, label: 'System' },
        { id: 'light', icon: Sun, label: 'Light' },
        { id: 'dark', icon: Moon, label: 'Dark' },
    ];

    const colors = [
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Purple', value: '#a855f7' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Emerald', value: '#10b981' },
        { name: 'Sky', value: '#0ea5e9' },
    ];

    return (
        <motion.div
            key="style"
            initial={{ opacity: 0, x: 5 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -5 }}
            className="space-y-8"
        >
            <Section title="Interface Theme">
                <div className="grid grid-cols-3 gap-3">
                    {themes.map((theme) => (
                        <button
                            key={theme.id}
                            onClick={() => setTheme(theme.id)}
                            className={`flex flex-col items-center gap-2 p-3 rounded-xl border transition-all ${config?.theme === theme.id
                                ? "bg-accent/10 border-accent/40 text-accent shadow-lg shadow-accent/10"
                                : "bg-bg-subtle border-white/5 text-text-muted hover:border-white/10 hover:text-text-main"
                                }`}
                        >
                            <theme.icon className="w-5 h-5" />
                            <span className="text-[10px] font-medium tracking-wide">{theme.label}</span>
                        </button>
                    ))}
                </div>
            </Section>

            <Section title="Accent Color">
                <div className="flex flex-wrap gap-4 px-2">
                    {colors.map((color) => (
                        <button
                            key={color.name}
                            onClick={() => setAccentColor(color.value)}
                            className="relative group"
                        >
                            <div
                                className={`w-8 h-8 rounded-full shadow-lg transition-transform duration-200 group-hover:scale-110 active:scale-95`}
                                style={{ backgroundColor: color.value }}
                            />
                            {config?.accent_color === color.value && (
                                <motion.div
                                    layoutId="color-check"
                                    className="absolute inset-0 flex items-center justify-center text-white"
                                >
                                    <Check className="w-4 h-4" />
                                </motion.div>
                            )}
                            <div className="absolute -bottom-6 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap text-[8px] text-text-muted">
                                {color.name}
                            </div>
                        </button>
                    ))}
                </div>
            </Section>
        </motion.div>
    );
};

export default StyleTab;
