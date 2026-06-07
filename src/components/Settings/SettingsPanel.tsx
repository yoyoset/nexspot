import React, { useEffect } from "react";
import { translateError } from "../../utils/error";
import { SlidersHorizontal, Workflow as WorkflowIcon, Cpu, Palette, Heart } from "lucide-react";
import { AnimatePresence } from "framer-motion";
import { useConfig } from "../../hooks/useConfig";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/useAppStore";
import GeneralTab from "./tabs/GeneralTab";
import AdvancedTab from "./tabs/AdvancedTab";
import StyleTab from "./tabs/StyleTab";
import WorkflowsTab from "./tabs/WorkflowsTab";
import DonateTab from "./tabs/DonateTab";

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
        { id: "general", icon: SlidersHorizontal, label: t('settings.tabs.general') },
        { id: "workflows", icon: WorkflowIcon, label: t('workflows.title') },
        { id: "advanced", icon: Cpu, label: t('settings.advanced.title') },
        { id: "style", icon: Palette, label: t('settings.tabs.aesthetics') },
        { id: "donate", icon: Heart, label: t('settings.tabs.donate', 'Donate') },
    ];

    return (
        <div className="w-full h-full bg-bg-main flex overflow-hidden">
            {/* Sub-tab navigation */}
            <div className="w-[158px] shrink-0 border-r border-line flex flex-col py-3 px-2 gap-0.5 bg-bg-1">
                {tabs.map((tab) => (
                    <button
                        key={tab.id}
                        onClick={() => setActiveTab(tab.id)}
                        className={`flex items-center gap-2.5 px-2.5 py-2 rounded-btn text-[12.5px] font-semibold transition-colors ${activeTab === tab.id
                            ? "bg-accent-soft text-accent"
                            : "text-muted hover:text-ink hover:bg-bg-2"
                            }`}
                    >
                        <tab.icon className="w-4 h-4 shrink-0" />
                        {tab.label}
                    </button>
                ))}
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto custom-scrollbar px-7 py-6">
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
    );
};

export default SettingsPanel;
