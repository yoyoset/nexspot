import React from "react";
import { motion } from "framer-motion";
import { Camera, Settings, FolderOpen, Command } from "lucide-react";
import BentoCard from "../UI/BentoCard";
import { invoke } from "@tauri-apps/api/core";
import { useShortcuts } from "../../hooks/useShortcuts";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/useAppStore";

const Dashboard: React.FC = () => {
    const setShowSettings = useAppStore(state => state.setShowSettings);
    const { shortcuts } = useShortcuts();
    const { t } = useTranslation();
    const getKey = (id: string) => shortcuts.find(s => s.id === id)?.shortcut || "...";

    const handleCapture = async () => {
        try {
            await invoke("start_capture");
        } catch (error) {
            console.error("Capture trigger failed:", error);
        }
    };

    return (
        <div className="w-full h-full bg-bg-main/95 flex flex-col p-6 gap-6 overflow-hidden">
            {/* Top Cards Area - Occupies less vertical space */}
            <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="grid grid-cols-12 gap-4 items-stretch"
            >
                {/* 1. Main Capture (6/12) */}
                <BentoCard
                    colSpan={6}
                    title={t('dashboard.capture_title')}
                    description={t('dashboard.capture_desc')}
                    icon={<Camera className="w-5 h-5" />}
                    onClick={handleCapture}
                    className="bg-gradient-to-br from-white/5 to-transparent flex flex-col justify-center group active:scale-[0.99] transition-transform py-4"
                >
                    <div className="flex flex-col items-center justify-center">
                        <div className="text-2xl font-black text-white tracking-[0.2em] mb-4 group-hover:glow-text transition-all uppercase">
                            {t('dashboard.capture_btn')}
                        </div>

                        <div className="flex items-center gap-2 text-xs font-mono text-accent bg-accent/10 px-3 py-1.5 rounded-lg border border-accent/20">
                            <Command className="w-3 h-3" />
                            <span>{getKey("capture")}</span>
                        </div>
                    </div>
                </BentoCard>

                {/* 2. Folder (3/12) */}
                <BentoCard
                    colSpan={3}
                    title={t('dashboard.library_title')}
                    icon={<FolderOpen className="w-5 h-5" />}
                    description={t('dashboard.library_desc')}
                    className="h-full py-4"
                >
                    <div className="flex flex-col items-center justify-center opacity-30 mt-2">
                        <span className="text-[10px] font-mono tracking-widest">{t('dashboard.library_empty')}</span>
                    </div>
                </BentoCard>

                {/* 3. Settings (3/12) */}
                <BentoCard
                    colSpan={3}
                    title={t('dashboard.settings_title')}
                    icon={<Settings className="w-5 h-5" />}
                    onClick={() => setShowSettings(true)}
                    className="h-full py-4"
                >
                    <div className="text-text-muted text-[10px] mt-1 line-clamp-2">
                        {t('dashboard.settings_desc')}
                    </div>
                </BentoCard>
            </motion.div>

            {/* Bottom Toolbar Area - Expanded for custom components */}
            <div className="flex-grow border-t border-white/5 bg-black/5 rounded-3xl flex items-center justify-center transition-all">
                <span className="text-xs text-text-muted/10 font-mono tracking-[0.5em] uppercase">
                    Toolbar Zone (Reserved)
                </span>
            </div>
        </div>
    );
};

export default Dashboard;
