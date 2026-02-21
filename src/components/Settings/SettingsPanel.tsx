import React, { useState, useEffect } from "react";
import { Settings, X, Cpu, Keyboard, Palette, Layers } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { useAppStore } from "../../store/useAppStore";
import GeneralTab from "./tabs/GeneralTab";
import AdvancedTab from "./tabs/AdvancedTab";
import StyleTab from "./tabs/StyleTab";
import WorkflowsTab from "./tabs/WorkflowsTab";

const SettingsPanel: React.FC = () => {
    const setShowSettings = useAppStore(state => state.setShowSettings);
    const { settingsNavigation, setSettingsNavigation } = useAppStore();
    const activeTab = settingsNavigation.tab;
    const setActiveTab = (tab: string) => setSettingsNavigation(tab);

    const {
        config, selectSavePath, fetchConfig, setOcrEngine, setFontFamily,
        setVelloEnabled, setVelloAdvancedEffects, setSnapshotEnabled, setSnapshotSize,
        addWorkflow, updateWorkflow, removeWorkflow,
        setJpgQuality, setConcurrency
    } = useConfig();
    const { t } = useTranslation();

    useEffect(() => {
        fetchConfig();
    }, [fetchConfig]);


    interface BackendError {
        code: 'Conflict' | 'RegistrationFailed' | 'NotFound' | 'InvalidFormat' | 'Empty' | 'Io';
        message: string;
    }

    const translateError = (err: any) => {
        if (typeof err === 'object' && err !== null && 'code' in err) {
            const bErr = err as BackendError;
            switch (bErr.code) {
                case 'Empty': return t('settings.shortcuts.errors.empty');
                case 'Conflict': return bErr.message;
                case 'RegistrationFailed': return t('settings.shortcuts.errors.register_failed');
                case 'InvalidFormat': return t('settings.shortcuts.errors.invalid_format') || "Invalid Format";
                default: return bErr.message;
            }
        }
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
        return String(err);
    };


    const tabs = [
        { id: "general", icon: Cpu, label: t('settings.tabs.general') },
        { id: "advanced", icon: Cpu, label: t('settings.advanced.title') },
        { id: "style", icon: Palette, label: t('settings.tabs.aesthetics') },
    ];

    return (
        <div className="w-full h-full bg-bg-main flex flex-col overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-border-subtle bg-bg-subtle">
                <div className="flex items-center gap-2">
                    <Settings className="w-4 h-4 text-accent" />
                    <h2 className="font-bold text-text-main text-lg tracking-tight">{t('settings.title')}</h2>
                </div>
            </div>

            <div className="flex flex-1 overflow-hidden">
                {/* Sidebar */}
                <div className="w-40 border-r border-border-subtle flex flex-col py-4 bg-bg-sidebar">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id)}
                            className={`flex items-center gap-3 px-4 py-3 mx-2 rounded-lg text-xs font-semibold transition-all duration-200 ${activeTab === tab.id
                                ? "bg-accent/10 text-accent border border-accent/20"
                                : "text-text-muted hover:text-text-main hover:bg-bg-subtle"
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
                            <GeneralTab
                                config={config}
                                selectSavePath={selectSavePath}
                                setOcrEngine={setOcrEngine}
                                setFontFamily={setFontFamily}
                                fetchConfig={fetchConfig}
                            />
                        )}

                        {activeTab === "advanced" && (
                            <AdvancedTab
                                config={config}
                                setVelloEnabled={setVelloEnabled}
                                setVelloAdvancedEffects={setVelloAdvancedEffects}
                                setSnapshotEnabled={setSnapshotEnabled}
                                setSnapshotSize={setSnapshotSize}
                                setJpgQuality={setJpgQuality}
                                setConcurrency={setConcurrency}
                            />
                        )}

                        {activeTab === "style" && (
                            <StyleTab />
                        )}
                    </AnimatePresence>
                </div>
            </div>

            {/* Footer */}
            <div className="p-3 border-t border-border-subtle bg-white/[0.02] flex justify-end">
                <span className="text-[10px] text-text-muted font-mono">
                    HYPER-LENS CORE v0.2.1
                </span>
            </div>
        </div>
    );
};

export default SettingsPanel;
