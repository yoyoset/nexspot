import React, { useEffect } from "react";
import { translateError } from "../../utils/error";
import { Settings as SettingsIcon, Cpu, Layers, Palette } from "lucide-react";
import { AnimatePresence } from "framer-motion";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/useAppStore";
import GeneralTab from "./tabs/GeneralTab";
import AdvancedTab from "./tabs/AdvancedTab";
import StyleTab from "./tabs/StyleTab";
import WorkflowsTab from "./tabs/WorkflowsTab";
import DonateTab from "./tabs/DonateTab";
import { Coffee } from "lucide-react";

const SettingsPanel: React.FC = () => {
    const { settingsNavigation, setSettingsNavigation } = useAppStore();
    const activeTab = settingsNavigation.tab;
    const setActiveTab = (tab: string) => setSettingsNavigation(tab);

    const {
        config, selectSavePath, fetchConfig, setFontFamily,
        setVelloEnabled, setVelloAdvancedEffects, setVelloAestheticStyle, setSnapshotEnabled, setSnapshotSize,
        removeWorkflow, setJpgQuality, setConcurrency, setDefaultExportFormat
    } = useConfig();
    const { t } = useTranslation();

    useEffect(() => {
        fetchConfig();
    }, [fetchConfig]);

    const tabs = [
        { id: "general", icon: Cpu, label: t('settings.tabs.general') },
        { id: "workflows", icon: Layers, label: t('workflows.title') },
        { id: "advanced", icon: Cpu, label: t('settings.advanced.title') },
        { id: "style", icon: Palette, label: t('settings.tabs.aesthetics') },
        { id: "donate", icon: Coffee, label: t('settings.tabs.donate', 'Donate') },
    ];

    return (
        <div className="w-full h-full bg-bg-main flex flex-col overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-border-subtle bg-bg-subtle/30">
                <div className="flex items-center gap-2">
                    <SettingsIcon className="w-3.5 h-3.5 text-accent opacity-80" />
                    <h2 className="font-bold text-text-main text-sm tracking-tight tech-text uppercase">{t('settings.title')}</h2>
                </div>
            </div>

            <div className="flex flex-1 overflow-hidden">
                {/* Sidebar */}
                <div className="w-36 border-r border-border-subtle flex flex-col py-2 bg-bg-sidebar">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id)}
                            className={`flex items-center gap-2.5 px-3 py-2 mx-1.5 rounded-sm text-[11px] font-bold tech-text transition-all duration-150 border ${activeTab === tab.id
                                ? "bg-accent/5 text-accent border-accent/20"
                                : "text-text-muted border-transparent hover:text-text-main hover:bg-white/[0.03]"
                                }`}
                        >
                            <tab.icon className="w-3.5 h-3.5" />
                            {tab.label}
                        </button>
                    ))}
                </div>

                {/* Content */}
                <div className="flex-1 p-4 overflow-y-auto bg-bg-main/50 relative custom-scrollbar">
                    <AnimatePresence mode="wait">
                        {activeTab === "general" && (
                            <GeneralTab
                                config={config}
                                selectSavePath={selectSavePath}
                                setFontFamily={setFontFamily}
                                fetchConfig={fetchConfig}
                            />
                        )}

                        {activeTab === "workflows" && (
                            <WorkflowsTab
                                config={config}
                                removeWorkflow={async (id) => {
                                    try {
                                        await removeWorkflow(id);
                                    } catch (err) {
                                        console.error("Failed to remove workflow:", err);
                                        const message = translateError(err, t);
                                        useAppStore.getState().showHUD(message, 'error');
                                    }
                                }}
                                initialWorkflowId={settingsNavigation.workflowId}
                            />
                        )}

                        {activeTab === "advanced" && (
                            <AdvancedTab
                                config={config}
                                setVelloEnabled={setVelloEnabled}
                                setVelloAestheticStyle={setVelloAestheticStyle}
                                setVelloAdvancedEffects={setVelloAdvancedEffects}
                                setSnapshotEnabled={setSnapshotEnabled}
                                setSnapshotSize={setSnapshotSize}
                                setJpgQuality={setJpgQuality}
                                setConcurrency={setConcurrency}
                                setDefaultExportFormat={setDefaultExportFormat}
                            />
                        )}


                        {activeTab === "style" && (
                            <StyleTab />
                        )}

                        {activeTab === "donate" && (
                            <DonateTab />
                        )}
                    </AnimatePresence>
                </div>
            </div>

            {/* Footer */}
            <div className="px-3 py-1.5 border-t border-border-subtle bg-white/[0.01] flex justify-end">
                <span className="text-[9px] text-text-muted tech-text opacity-40 uppercase tracking-widest">
                    {t('common.version_brand')} <span className="mx-1">//</span> {t('common.industrial_grade')}
                </span>
            </div>
        </div>
    );
};

export default SettingsPanel;
