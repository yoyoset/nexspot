import React, { useState } from "react";
import { Globe, Cpu, Key, CheckCircle2, AlertCircle, Eye, EyeOff, Terminal, Info, ChevronDown, Activity } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";

interface AITabProps {
    config: any;
    setAiApiUrl: (url: string) => Promise<any>;
    setAiModel: (model: string) => Promise<any>;
    setAiApiKey: (key: string) => Promise<any>;
    setAiProvider: (provider: string) => Promise<any>;
    setAiProviderKey: (providerId: string, key: string) => Promise<any>;
    verifyAiConnection: (url: string, key: string, model: string) => Promise<any>;
}

const Section = ({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) => (
    <motion.section
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="space-y-3"
    >
        <div className="flex items-center gap-2.5 pl-1">
            <Icon className="w-3.5 h-3.5 text-accent opacity-70" />
            <h3 className="text-[10px] font-bold tech-text text-text-muted uppercase tracking-[0.15em]">{title}</h3>
        </div>
        <div className="space-y-2">{children}</div>
    </motion.section>
);

const AgentCoreTab: React.FC<AITabProps> = ({ config, setAiApiUrl, setAiModel, setAiApiKey, setAiProvider, setAiProviderKey, verifyAiConnection }) => {
    const { t } = useTranslation();
    const [showKey, setShowKey] = useState(false);
    const [verifying, setVerifying] = useState(false);
    const [saving, setSaving] = useState(false);
    const [saveStatus, setSaveStatus] = useState<'idle' | 'success' | 'error'>('idle');
    const [verifyStatus, setVerifyStatus] = useState<'idle' | 'success' | 'error'>('idle');
    const [verifyMsg, setVerifyMsg] = useState("");
    const [inputKey, setInputKey] = useState(config?.ai_api_key || "");

    const providers = [
        { id: "openai", name: "OpenAI", url: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini" },
        { id: "gemini", name: "Google Gemini", url: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", model: "gemini-2.0-flash" },
        { id: "deepseek", name: "DeepSeek", url: "https://api.deepseek.com/chat/completions", model: "deepseek-chat" },
        { id: "groq", name: "Groq", url: "https://api.groq.com/openai/v1/chat/completions", model: "llama-3.2-11b-vision-preview" },
        { id: "ollama", name: "Ollama (Local)", url: "http://localhost:11434/v1/chat/completions", model: "llama3.2-vision" },
        { id: "siliconflow", name: "SiliconFlow", url: "https://api.siliconflow.cn/v1/chat/completions", model: "deepseek-ai/DeepSeek-V3" },
        { id: "openrouter", name: "OpenRouter", url: "https://openrouter.ai/api/v1/chat/completions", model: "google/gemini-2.0-flash-001" },
        { id: "custom", name: "Custom Provider", url: "", model: "" }
    ];

    const currentProvider = config?.ai_provider || "openai";

    React.useEffect(() => {
        if (config?.ai_api_key !== undefined) {
            setInputKey(config.ai_api_key);
        }
    }, [config?.ai_api_key]);

    const handleProviderChange = async (providerId: string) => {
        await setAiProvider(providerId);
        if (providerId !== "custom") {
            const preset = providers.find(p => p.id === providerId);
            if (preset) {
                await setAiApiUrl(preset.url);
                await setAiModel(preset.model);
            }
        }
        setSaveStatus('idle');
        setVerifyStatus('idle');
    };

    const handleSave = async () => {
        setSaving(true);
        setSaveStatus('idle');
        try {
            await setAiApiKey(inputKey);
            setSaveStatus('success');
            setTimeout(() => setSaveStatus('idle'), 2000);
        } catch (err) {
            setSaveStatus('error');
        }
        setSaving(false);
    };

    const handleVerify = async () => {
        setVerifying(true);
        setVerifyStatus('idle');
        setVerifyMsg("");

        const result = await verifyAiConnection(config.ai_api_url, config.ai_api_key, config.ai_model);

        if (result.success) {
            setVerifyStatus('success');
            setVerifyMsg(t('ai.verify_success'));
        } else {
            setVerifyStatus('error');
            setVerifyMsg(result.error || t('ai.verify_fail'));
        }
        setVerifying(false);
    };

    return (
        <motion.div
            key="agent-settings"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-6 pb-6 max-w-2xl"
        >
            {/* 1. Core API Config */}
            <Section title={t('ai.api_orchestration')} icon={Terminal}>
                <div className="bg-bg-subtle border border-white/5 rounded-sm p-4 space-y-5">

                    {/* Vision Warning */}
                    <div className="bg-accent/5 border border-accent/20 rounded-sm p-3 flex gap-3 items-center">
                        <div className="p-1.5 bg-accent/10 rounded-sm text-accent">
                            <Info className="w-3.5 h-3.5" />
                        </div>
                        <div className="space-y-0.5">
                            <div className="text-[11px] font-bold text-accent tech-text uppercase">{t('ai.vision_warning')}</div>
                            <p className="text-[9px] tech-text text-text-muted opacity-70 uppercase leading-tight">
                                {t('ai.vision_desc')}
                                Recommended: [GPT-4O], [GEMINI-2.0-FLASH], [CLAUDE-3.5-SONNET].
                            </p>
                        </div>
                    </div>

                    <div className="grid gap-4">
                        {/* Provider Selection */}
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold tech-text text-text-muted uppercase tracking-widest flex items-center gap-2 opacity-50">
                                <Terminal className="w-3 h-3" /> {t('ai.provider_uplink')}
                            </label>
                            <div className="relative">
                                <select
                                    value={currentProvider}
                                    onChange={(e) => handleProviderChange(e.target.value)}
                                    className="w-full bg-black/40 border border-white/10 rounded-sm px-3 py-2 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 appearance-none cursor-pointer pr-10 hover:bg-black/60 transition-all"
                                >
                                    {providers.map((p) => (
                                        <option key={p.id} value={p.id} className="bg-bg-subtle text-text-main">
                                            {p.name.toUpperCase()}
                                        </option>
                                    ))}
                                </select>
                                <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-text-muted opacity-30">
                                    <ChevronDown className="w-4 h-4" />
                                </div>
                            </div>
                        </div>

                        {/* API URL */}
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold tech-text text-text-muted uppercase tracking-widest flex items-center gap-2 opacity-50">
                                <Globe className="w-3 h-3" /> {t('ai.endpoint_uri')}
                            </label>
                            <input
                                type="text"
                                value={config?.ai_api_url}
                                onChange={(e) => setAiApiUrl(e.target.value)}
                                disabled={currentProvider !== "custom"}
                                className={`w-full bg-black/40 border border-white/10 rounded-sm px-3 py-2 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 transition-all ${currentProvider !== "custom" ? "opacity-30 cursor-not-allowed" : ""}`}
                            />
                        </div>

                        {/* API Model */}
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold tech-text text-text-muted uppercase tracking-widest flex items-center gap-2 opacity-50">
                                <Cpu className="w-3 h-3" /> {t('ai.kernel_id')}
                            </label>
                            <input
                                type="text"
                                value={config?.ai_model}
                                onChange={(e) => setAiModel(e.target.value)}
                                disabled={currentProvider !== "custom"}
                                className={`w-full bg-black/40 border border-white/10 rounded-sm px-3 py-2 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 transition-all ${currentProvider !== "custom" ? "opacity-30 cursor-not-allowed" : ""}`}
                            />
                        </div>

                        {/* API Key */}
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold tech-text text-text-muted uppercase tracking-widest flex items-center gap-2 opacity-50">
                                <Key className="w-3 h-3" /> {t('ai.access_key')}
                            </label>
                            <div className="relative">
                                <input
                                    type={showKey ? "text" : "password"}
                                    value={inputKey}
                                    onChange={(e) => setInputKey(e.target.value)}
                                    className="w-full bg-black/40 border border-white/10 rounded-sm px-3 py-2 text-[11px] font-bold tech-text text-text-main outline-none focus:border-accent/40 transition-all pr-12"
                                />
                                <button
                                    onClick={() => setShowKey(!showKey)}
                                    className="absolute right-3 top-1/2 -translate-y-1/2 p-1.5 text-text-muted opacity-40 hover:opacity-100 transition-opacity"
                                >
                                    {showKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Verification & Save Footer */}
                    <div className="pt-4 border-t border-white/5 flex gap-2">
                        <button
                            onClick={handleSave}
                            disabled={saving}
                            className={`flex-[1.5] py-2.5 rounded-sm text-[10px] font-bold tech-text uppercase tracking-widest transition-all flex items-center justify-center gap-2 border ${saveStatus === 'success'
                                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-500"
                                : saving
                                    ? "bg-bg-subtle text-text-muted border-white/5 cursor-not-allowed"
                                    : "bg-white/5 border-white/10 text-text-muted hover:text-text-main hover:bg-white/10"
                                }`}
                        >
                            <Key className={`w-3.5 h-3.5 ${saving ? "animate-spin" : ""}`} />
                            {saving ? t('ai.syncing') : saveStatus === 'success' ? t('ai.synced') : t('ai.commit_config')}
                        </button>

                        <button
                            onClick={handleVerify}
                            disabled={verifying}
                            className={`flex-1 py-2.5 rounded-sm text-[10px] font-bold tech-text uppercase tracking-widest transition-all flex items-center justify-center gap-2 border ${verifying
                                ? "bg-bg-subtle text-text-muted border-white/5 cursor-not-allowed"
                                : "bg-accent/10 border-accent/20 text-accent hover:bg-accent hover:text-white"
                                }`}
                        >
                            <Activity className={`w-3.5 h-3.5 ${verifying ? "animate-pulse" : ""}`} />
                            {verifying ? "..." : t('ai.ping')}
                        </button>
                    </div>

                    <AnimatePresence>
                        {verifyStatus !== 'idle' && (
                            <motion.div
                                initial={{ opacity: 0, height: 0 }}
                                animate={{ opacity: 1, height: 'auto' }}
                                exit={{ opacity: 0, height: 0 }}
                                className={`p-3 rounded-sm border flex gap-3 items-center ${verifyStatus === 'success'
                                    ? "bg-emerald-500/5 border-emerald-500/10 text-emerald-500/80"
                                    : "bg-red-500/5 border-red-500/10 text-red-500/80"
                                    }`}
                            >
                                {verifyStatus === 'success' ? <CheckCircle2 className="w-3.5 h-3.5" /> : <AlertCircle className="w-3.5 h-3.5" />}
                                <div className="text-[10px] tech-text font-bold uppercase truncate">
                                    {verifyMsg}
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </Section>
        </motion.div>
    );
};

export default AgentCoreTab;
