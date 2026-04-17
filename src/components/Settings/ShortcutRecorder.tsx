import React, { useState, useEffect, useRef } from "react";
import { Keyboard as KeyboardIcon, Check, X, AlertTriangle, RotateCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface ShortcutRecorderProps {
    value?: string;
    onChange: (shortcut: string) => void;
    placeholder?: string;
    className?: string;
    status?: 'success' | 'error' | 'neutral';
    statusMessage?: string;
}

const ShortcutRecorder: React.FC<ShortcutRecorderProps> = ({
    value = "",
    onChange,
    placeholder = "None",
    className = "",
    status = "neutral",
    statusMessage,
}) => {
    const { t } = useTranslation();
    const [isRecording, setIsRecording] = useState(false);
    const [capturedShortcut, setCapturedShortcut] = useState<string>("");
    const containerRef = useRef<HTMLDivElement>(null);
    const recordingState = useRef<{
        modifiers: Set<string>;
        mainKey: string;
    }>({ modifiers: new Set(), mainKey: "" });

    // Standardize key names for backend (global-hotkey compatibility)
    const getStandardKeyName = (key: string) => {
        // Modifiers
        if (key === "Control") return "Control";
        if (key === "Alt") return "Alt";
        if (key === "Shift") return "Shift";
        if (key === "Meta") return "Super"; // Win key

        // Function keys
        if (key.match(/^F\d+$/)) return key;

        // Special keys
        if (key === " ") return "Space";
        if (key === "Enter") return "Enter";
        if (key === "Escape") return "Escape";
        if (key === "Backspace") return "Backspace";
        if (key === "Delete") return "Delete";
        if (key === "Tab") return "Tab";
        if (key === "ArrowUp") return "Up";
        if (key === "ArrowDown") return "Down";
        if (key === "ArrowLeft") return "Left";
        if (key === "ArrowRight") return "Right";

        // Alphanumeric keys
        if (key.length === 1) {
            return key.toUpperCase();
        }

        return key;
    };

    const updateDisplay = () => {
        const { modifiers, mainKey } = recordingState.current;
        const parts = Array.from(modifiers);

        // Sorting modifiers for consistent display
        const order = ["Control", "Shift", "Alt", "Super"];
        parts.sort((a, b) => order.indexOf(a) - order.indexOf(b));

        if (mainKey && !parts.includes(mainKey)) {
            parts.push(mainKey);
        }
        setCapturedShortcut(parts.join("+"));
    };

    // Suspend/Resume Global Hotkeys
    useEffect(() => {
        if (isRecording) {
            invoke("suspend_hotkeys").catch(console.error);
        } else {
            invoke("resume_hotkeys").catch(console.error);
        }

        // Cleanup if component unmounts while recording
        return () => {
            if (isRecording) {
                invoke("resume_hotkeys").catch(console.error);
            }
        };
    }, [isRecording]);

    useEffect(() => {
        if (!isRecording) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();
            if (e.repeat) return;

            // Update modifiers based on the event state directly
            const nextModifiers = new Set<string>();
            if (e.ctrlKey) nextModifiers.add("Control");
            if (e.altKey) nextModifiers.add("Alt");
            if (e.shiftKey) nextModifiers.add("Shift");
            if (e.metaKey) nextModifiers.add("Super");

            recordingState.current.modifiers = nextModifiers;

            // Update main key if it's not a modifier
            if (!["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
                recordingState.current.mainKey = getStandardKeyName(e.key);
            }

            updateDisplay();
        };

        const handleKeyUp = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();

            const isModifier = ["Control", "Alt", "Shift", "Meta"].includes(e.key);

            // If a non-modifier key is released, we commit the CURRENT CHORD
            if (!isModifier && capturedShortcut) {
                onChange(capturedShortcut);
                setIsRecording(false);
            }
            // If it's a modifier release, we update the state
            else if (isModifier) {
                const nextModifiers = new Set<string>();
                if (e.ctrlKey) nextModifiers.add("Control");
                if (e.altKey) nextModifiers.add("Alt");
                if (e.shiftKey) nextModifiers.add("Shift");
                if (e.metaKey) nextModifiers.add("Super");
                recordingState.current.modifiers = nextModifiers;

                // If ALL keys are released and we have a shortcut (likely a single key or modifier-only), commit it
                if (nextModifiers.size === 0 && !recordingState.current.mainKey && capturedShortcut) {
                    onChange(capturedShortcut);
                    setIsRecording(false);
                }
            }
        };

        const handleClickOutside = (e: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
                setIsRecording(false);
                setCapturedShortcut("");
            }
        };

        window.addEventListener("keydown", handleKeyDown, true);
        window.addEventListener("keyup", handleKeyUp, true);
        window.addEventListener("mousedown", handleClickOutside, true);

        return () => {
            window.removeEventListener("keydown", handleKeyDown, true);
            window.removeEventListener("keyup", handleKeyUp, true);
            window.removeEventListener("mousedown", handleClickOutside, true);
        };
    }, [isRecording, capturedShortcut, onChange]);

    return (
        <div className={`flex flex-col gap-1 ${className}`} ref={containerRef}>
            <div className="flex items-center gap-2">
                <div
                    className={`relative flex-1 flex items-center gap-2 px-3 py-1.5 rounded-sm border text-xs transition-all h-8
                        ${isRecording
                            ? "bg-purple-500/10 border-purple-500 text-purple-400 ring-1 ring-purple-500/20"
                            : status === 'error'
                                ? "bg-red-500/10 border-red-500/30 text-white"
                                : "bg-white/5 border-white/10 text-text-main shadow-inner"
                        }
                    `}
                >
                    <KeyboardIcon className={`w-3.5 h-3.5 ${isRecording ? "animate-pulse text-purple-400" : "opacity-40"}`} />

                    <span className="font-mono flex-1 text-center font-bold tracking-tight tech-text">
                        {isRecording ? (
                            capturedShortcut || t('workflows.awaiting_input')
                        ) : (
                            value || <span className="opacity-30 italic font-medium uppercase tracking-widest text-[10px]">{t('workflows.unbound')}</span>
                        )}
                    </span>

                    {isRecording ? (
                        <div className="text-[8px] uppercase font-black text-purple-500 animate-pulse bg-purple-500/10 px-1 rounded-sm border border-purple-500/20">{t('workflows.rec_mode')}</div>
                    ) : (
                        <div className="flex gap-1 shrink-0">
                            {status === 'success' && <Check className="w-3.5 h-3.5 text-green-500" />}
                            {status === 'error' && <AlertTriangle className="w-3.5 h-3.5 text-red-500" />}
                        </div>
                    )}
                </div>

                <button
                    onClick={() => {
                        const next = !isRecording;
                        setIsRecording(next);
                        if (next) setCapturedShortcut("");
                    }}
                    className={`p-1.5 rounded-sm border border-white/10 hover:bg-white/10 transition-colors shrink-0
                         ${isRecording ? 'bg-red-500/20 border-red-500/40 text-red-400 hover:bg-red-500/30' : 'text-text-muted hover:text-white'}
                    `}
                >
                    {isRecording ? <X className="w-4 h-4" /> : <RotateCw className="w-4 h-4" />}
                </button>
            </div>

            {statusMessage && (
                <div className={`text-[9px] pl-1 font-bold tech-text uppercase tracking-widest ${status === 'error' ? 'text-red-400' :
                    status === 'success' ? 'text-green-400' : 'text-text-muted/40'
                    }`}>
                    {statusMessage === "READY" ? t('workflows.ready') : statusMessage}
                </div>
            )}
        </div>
    );
};

export default ShortcutRecorder;
