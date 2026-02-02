import React from "react";
import { motion } from "framer-motion";
import { Camera, Settings, History, Sparkles, Command } from "lucide-react";
import BentoCard from "../UI/BentoCard";
import { invoke } from "@tauri-apps/api/core";
import { useShortcuts } from "../../hooks/useShortcuts";

interface DashboardProps {
    onOpenSettings: () => void;
}

const Dashboard: React.FC<DashboardProps> = ({ onOpenSettings }) => {
    const { shortcuts } = useShortcuts();
    const getKey = (id: string) => shortcuts.find(s => s.id === id)?.shortcut || "...";

    const handleCapture = async () => {
        try {
            await invoke("start_capture");
        } catch (error) {
            console.error("Capture trigger failed:", error);
        }
    };

    return (
        <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="w-full h-full bg-bg-main/90 p-6 grid grid-cols-3 gap-4"
        >
            {/* Left Column: Main Capture - Spans 2 cols, full height */}
            <BentoCard
                colSpan={2}
                title="HyperLens Capture"
                description="Instant, zero-latency screenshot engine."
                icon={<Camera className="w-6 h-6" />}
                onClick={handleCapture}
                className="bg-gradient-to-br from-white/5 to-transparent h-full flex flex-col justify-between group active:scale-[0.99] transition-transform"
            >
                <div className="flex-1 flex flex-col justify-center items-center">
                    <div className="w-32 h-32 rounded-full bg-accent/5 border border-accent/10 flex items-center justify-center mb-6 group-hover:bg-accent/10 transition-colors shadow-[0_0_40px_-10px_rgba(59,130,246,0.3)]">
                        <div className="w-24 h-24 rounded-full bg-accent/10 flex items-center justify-center animate-pulse-slow">
                            <Camera className="w-10 h-10 text-accent" />
                        </div>
                    </div>

                    <div className="text-5xl font-bold text-white tracking-tight mb-4 group-hover:glow-text transition-all">
                        CAPTURE
                    </div>

                    <div className="flex items-center gap-2 text-sm font-mono text-accent bg-accent/10 px-4 py-2 rounded-lg border border-accent/20">
                        <Command className="w-4 h-4" />
                        <span>{getKey("capture")}</span>
                    </div>
                </div>
            </BentoCard>

            {/* Right Column: Tools Stack */}
            <div className="flex flex-col gap-4 h-full">
                {/* Settings Card */}
                <BentoCard
                    title="Settings"
                    icon={<Settings className="w-5 h-5" />}
                    onClick={onOpenSettings}
                    className="flex-1"
                >
                    <div className="text-text-muted text-xs mt-1">
                        Configure shortcuts & UI
                    </div>
                </BentoCard>

                {/* Library/History Card */}
                <BentoCard
                    title="Library"
                    icon={<History className="w-5 h-5" />}
                    description="Recent Snaps"
                    className="flex-1"
                >
                    <div className="flex-1 flex items-center justify-center opacity-50">
                        <span className="text-[10px] font-mono">NO HISTORY</span>
                    </div>
                </BentoCard>

                {/* Hyper AI Feature */}
                <div className="h-16 rounded-2xl bg-gradient-to-r from-accent/10 to-purple-500/10 border border-accent/20 flex items-center justify-between px-4 cursor-pointer hover:border-accent/40 transition-colors group">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-lg bg-accent/20 text-accent group-hover:bg-accent group-hover:text-white transition-colors">
                            <Sparkles className="w-4 h-4" />
                        </div>
                        <span className="text-sm font-bold text-text-main group-hover:text-white transition-colors">Hyper AI</span>
                    </div>
                    <span className="text-[10px] text-accent/50 group-hover:text-accent font-mono">BETA</span>
                </div>
            </div>
        </motion.div>
    );
};

export default Dashboard;
