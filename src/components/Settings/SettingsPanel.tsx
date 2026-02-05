import React, { useState, useEffect } from "react";
import { Settings, X, Cpu, Keyboard, Palette, FileText, Trash2, Languages } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import ShortcutRecorder from "./ShortcutRecorder";
import { useShortcuts } from "../../hooks/useShortcuts";
import { useConfig } from "../../hooks/useConfig";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { useAppStore } from "../../store/useAppStore";

const SettingsPanel: React.FC = () => {
    const setShowSettings = useAppStore(state => state.setShowSettings);
    // Default to shortcuts tab
    const [activeTab, setActiveTab] = useState("shortcuts");
    const { shortcuts, loading, fetchShortcuts, updateShortcut } = useShortcuts();
    const { config, selectSavePath, fetchConfig, setOcrEngine } = useConfig();
    const { t, i18n } = useTranslation();

    useEffect(() => {
        fetchShortcuts();
        fetchConfig();
    }, [fetchShortcuts, fetchConfig]);

    // Track status per shortcut for UI feedback
    const [statuses, setStatuses] = useState<Record<string, { type: 'success' | 'error' | 'neutral', msg?: string }>>({});

    // === Backend Event Listeners (Removed - Moved to App.tsx) ===
    // SettingsPanel only handles local validation feedback now.

    interface BackendError {
        code: 'Conflict' | 'RegistrationFailed' | 'NotFound' | 'InvalidFormat' | 'Empty' | 'Io';
        message: string;
    }

    // Parse backend error codes (now using structured JSON error objects)
    const translateError = (err: any) => {
        // Handle new structured error object
        if (typeof err === 'object' && err !== null && 'code' in err) {
            const bErr = err as BackendError;
            switch (bErr.code) {
                case 'Empty': return t('settings.shortcuts.errors.empty');
                case 'Conflict': {
                    // Message usually contains the label if we want to extract it, 
                    // or we rely on the backend message itself.
                    // For now, let's keep it simple.
                    return bErr.message;
                }
                case 'RegistrationFailed': return t('settings.shortcuts.errors.register_failed');
                case 'InvalidFormat': return t('settings.shortcuts.errors.invalid_format') || "Invalid Format";
                default: return bErr.message;
            }
        }

        // Fallback for simple strings (compatibility)
        if (typeof err === 'string') {
            if (err === "ERR_EMPTY") return t('settings.shortcuts.errors.empty');
            if (err.startsWith("ERR_CONFLICT|")) {
                const label = err.split('|')[1];
                return t('settings.shortcuts.errors.conflict', { label });
            }
            if (err.startsWith("ERR_REGISTER_FAILED")) {
                return t('settings.shortcuts.errors.register_failed');
            }
        }

        return String(err); // Fallback
    };

    const handleShortcutChange = async (id: string, newKeys: string) => {
        // Optimistically set to neutral/loading if needed, or straight to validation
        const result = await updateShortcut(id, newKeys);
        if (result.success) {
            setStatuses(prev => ({ ...prev, [id]: { type: 'success', msg: 'Valid' } }));
        } else {
            console.error(result.error);
            const msg = translateError(result.error || "Unknown Error");
            setStatuses(prev => ({ ...prev, [id]: { type: 'error', msg } }));

            // Show Toast (Frontend Driven)
            if (await isPermissionGranted() || await requestPermission()) {
                sendNotification({
                    title: t('notifications.register_error_title'),
                    body: msg
                });
            }
        }
    };

    const toggleLanguage = () => {
        const newLang = i18n.language === 'zh' ? 'en' : 'zh';
        i18n.changeLanguage(newLang);
    };

    const tabs = [
        { id: "shortcuts", icon: Keyboard, label: t('settings.tabs.shortcuts') },
        { id: "general", icon: Cpu, label: t('settings.tabs.general') },
        { id: "style", icon: Palette, label: t('settings.tabs.aesthetics') },
    ];

    return (
        <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            className="w-[600px] h-[450px] bg-bg-main/95 border border-border-subtle rounded-2xl shadow-2xl flex flex-col overflow-hidden backdrop-blur-3xl"
        >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-border-subtle bg-white/[0.02]">
                <div className="flex items-center gap-2">
                    <Settings className="w-4 h-4 text-accent" />
                    <h2 className="font-medium text-text-main text-sm tracking-wide">{t('settings.title')}</h2>
                </div>
                <button onClick={() => setShowSettings(false)} className="hover:bg-white/10 p-1.5 rounded-md transition-colors text-text-muted hover:text-text-main">
                    <X className="w-4 h-4" />
                </button>
            </div>

            <div className="flex flex-1 overflow-hidden">
                {/* Sidebar */}
                <div className="w-40 border-r border-border-subtle flex flex-col py-4 bg-black/20">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id)}
                            className={`flex items-center gap-3 px-4 py-3 mx-2 rounded-lg text-sm transition-all duration-200 ${activeTab === tab.id
                                ? "bg-accent/10 text-accent border border-accent/20"
                                : "text-text-muted hover:text-text-main hover:bg-white/5"
                                }`}
                        >
                            <tab.icon className="w-4 h-4" />
                            {tab.label}
                        </button>
                    ))}
                </div>

                {/* Content */}
                <div className="flex-1 p-6 overflow-y-auto bg-bg-main/50 relative">
                    <AnimatePresence mode="wait">
                        {activeTab === "general" && (
                            <motion.div
                                key="general"
                                initial={{ opacity: 0, x: 5 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -5 }}
                                className="space-y-6"
                            >
                                <Section title="Language">
                                    <div className="bg-black/20 rounded-lg p-4 flex items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <Languages className="w-5 h-5 text-text-muted" />
                                            <div>
                                                <div className="text-sm font-medium text-text-main">
                                                    {i18n.language === 'zh' ? '中文 (Chinese)' : 'English'}
                                                </div>
                                                <div className="text-xs text-text-muted">
                                                    Current Language
                                                </div>
                                            </div>
                                        </div>
                                        <button
                                            onClick={toggleLanguage}
                                            className="px-3 py-1.5 text-xs bg-white/5 hover:bg-white/10 border border-white/10 rounded transition-colors"
                                        >
                                            Switch
                                        </button>
                                    </div>
                                </Section>

                                <Section title={t('settings.general.save_path') || "Save Path"}>
                                    <div className="bg-black/20 rounded-lg p-4 flex items-center justify-between">
                                        <div className="flex-1 min-w-0 mr-4">
                                            <div className="text-xs text-text-main font-mono truncate bg-white/5 px-2 py-1 rounded border border-white/5">
                                                {config?.save_path || "Default (captures/)"}
                                            </div>
                                            <div className="text-[10px] text-text-muted mt-1">
                                                Select where to save your high-resolution captures.
                                            </div>
                                        </div>
                                        <button
                                            onClick={selectSavePath}
                                            className="px-3 py-1.5 text-xs bg-accent text-white font-medium rounded hover:bg-accent/80 transition-all shadow-lg shadow-accent/20"
                                        >
                                            {t('settings.general.browse') || "Browse"}
                                        </button>
                                    </div>
                                </Section>

                                <Section title="OCR Engine">
                                    <div className="bg-black/20 rounded-lg p-4">
                                        <div className="flex items-center justify-between">
                                            <div>
                                                <div className="text-sm font-medium text-text-main">
                                                    {config?.ocr_engine === "default" ? "Windows Native OCR" : config?.ocr_engine}
                                                </div>
                                                <div className="text-[10px] text-text-muted">
                                                    Choose the engine used for text recognition.
                                                </div>
                                            </div>
                                            <div className="flex gap-2">
                                                <button
                                                    onClick={() => setOcrEngine('default')}
                                                    className={`px-3 py-1.5 text-[10px] rounded border transition-all ${config?.ocr_engine === 'default'
                                                        ? "bg-accent/20 border-accent/40 text-accent"
                                                        : "bg-white/5 border-white/10 text-white/60 hover:text-white"
                                                        }`}
                                                >
                                                    Default
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </Section>

                                <Section title={t('settings.engine.title')}>
                                    <div className="bg-black/20 rounded-lg p-4 space-y-4">
                                        <div className="flex items-center justify-between">
                                            <div>
                                                <div className="text-sm font-medium text-text-main">{t('settings.engine.enable_logs')}</div>
                                                <div className="text-xs text-text-muted">{t('settings.engine.enable_logs_desc')}</div>
                                            </div>
                                            {/* Toggle Switch - Visual only for now as backend default is TRUE */}
                                            <div className="w-10 h-5 bg-accent rounded-full relative cursor-pointer opacity-80">
                                                <div className="absolute right-1 top-1 w-3 h-3 bg-white rounded-full shadow-sm" />
                                            </div>
                                        </div>

                                        <div className="flex gap-3 pt-2">
                                            <button
                                                onClick={() => invoke("reveal_logs")}
                                                className="px-3 py-1.5 text-xs bg-white/5 hover:bg-white/10 rounded border border-white/10 transition-colors flex items-center gap-2 text-text-main"
                                            >
                                                <FileText className="w-3 h-3" />
                                                {t('settings.engine.open_log')}
                                            </button>
                                            <button
                                                onClick={() => {
                                                    invoke("clear_logs");
                                                }}
                                                className="px-3 py-1.5 text-xs bg-red-500/10 hover:bg-red-500/20 text-red-400 rounded border border-red-500/20 transition-colors flex items-center gap-2"
                                            >
                                                <Trash2 className="w-3 h-3" />
                                                {t('settings.engine.clear_logs')}
                                            </button>
                                        </div>
                                    </div>
                                </Section>
                            </motion.div>
                        )}

                        {activeTab === "shortcuts" && (
                            <motion.div
                                key="shortcuts"
                                initial={{ opacity: 0, x: 5 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -5 }}
                                className="space-y-6"
                            >
                                <Section title={t('settings.shortcuts.title')}>
                                    {loading ? (
                                        <div className="text-sm text-text-muted">{t('settings.shortcuts.loading')}</div>
                                    ) : (
                                        shortcuts.map(s => (
                                            <ShortcutRow
                                                key={s.id}
                                                label={s.label}
                                                shortcut={s.shortcut}
                                                // Priority: Local validation status > Backend Registration Error
                                                status={statuses[s.id]?.type || (s.error ? 'error' : 'neutral')}
                                                statusMessage={statuses[s.id]?.msg || translateError(s.error || "")}
                                                onChange={(k) => handleShortcutChange(s.id, k)}
                                            />
                                        ))
                                    )}
                                </Section>

                                <div className="text-xs text-text-muted p-2 bg-white/5 border border-white/10 rounded flex items-center gap-2">
                                    <div className="w-1.5 h-1.5 rounded-full bg-accent animate-pulse" />
                                    {t('settings.shortcuts.hint')}
                                </div>
                            </motion.div>
                        )}

                        {activeTab === "style" && (
                            <motion.div
                                key="style"
                                initial={{ opacity: 0, x: 5 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -5 }}
                                className="space-y-6 pointer-events-none opacity-50 grayscale"
                            >
                                <div className="absolute top-0 right-0 bg-yellow-500 text-black text-[10px] font-bold px-2 py-1 rounded">{t('settings.aesthetics.coming_soon')}</div>
                                <Section title={t('settings.aesthetics.title')}>
                                    <div className="flex gap-2">
                                        <div className="w-8 h-8 rounded-full bg-blue-500 ring-2 ring-offset-2 ring-offset-bg-card ring-blue-500 cursor-pointer"></div>
                                        <div className="w-8 h-8 rounded-full bg-purple-500 cursor-pointer opacity-50 hover:opacity-100"></div>
                                        <div className="w-8 h-8 rounded-full bg-emerald-500 cursor-pointer opacity-50 hover:opacity-100"></div>
                                    </div>
                                </Section>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </div>

            {/* Footer */}
            <div className="p-3 border-t border-border-subtle bg-white/[0.02] flex justify-end">
                <span className="text-[10px] text-text-muted font-mono">
                    HYPER-LENS CORE v0.2.2
                </span>
            </div>
        </motion.div >
    );
};

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <section className="space-y-3">
        <h3 className="text-xs font-bold text-text-muted uppercase tracking-wider pl-1">{title}</h3>
        <div className="space-y-2">
            {children}
        </div>
    </section>
);

const SettingsRow = ({ label, enabled }: { label: string; enabled: boolean }) => (
    <div className="flex items-center justify-between p-3 rounded-lg bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
        <span className="text-sm text-text-main">{label}</span>
        <div className={`w-9 h-5 rounded-full p-1 transition-colors cursor-pointer ${enabled ? "bg-accent" : "bg-white/20"}`}>
            <div className={`w-3 h-3 bg-white rounded-full shadow-sm transition-transform ${enabled ? "translate-x-4" : "translate-x-0"}`} />
        </div>
    </div>
);

interface ShortcutRowProps {
    label: string;
    shortcut: string;
    onChange: (k: string) => void;
    status?: 'success' | 'error' | 'neutral';
    statusMessage?: string;
}

const ShortcutRow = ({ label, shortcut, onChange, status, statusMessage }: ShortcutRowProps) => (
    <div className="flex items-center justify-between p-2 rounded-lg hover:bg-white/5 transition-colors">
        <span className="text-sm text-text-main">{label}</span>
        <ShortcutRecorder
            value={shortcut}
            onChange={onChange}
            status={status}
            statusMessage={statusMessage}
            className="w-48"
        />
    </div>
);

export default SettingsPanel;
