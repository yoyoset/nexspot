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
        <div className={`flex flex-col gap-1.5 ${className}`} ref={containerRef}>
            <div className="flex items-center gap-2">
                <button
                    onClick={() => {
                        const next = !isRecording;
                        setIsRecording(next);
                        if (next) setCapturedShortcut("");
                    }}
                    className={`relative flex-1 flex items-center gap-2 px-3 h-9 rounded-btn border transition-colors
                        ${isRecording
                            ? "bg-accent-soft border-accent text-accent border-dashed"
                            : status === 'error'
                                ? "bg-bad-soft border-bad/40 text-ink"
                                : "bg-bg-3 border-line text-ink hover:border-line-2"
                        }
                    `}
                >
                    <KeyboardIcon className={`w-4 h-4 ${isRecording ? "animate-pulse text-accent" : "text-muted"}`} />

                    <span className="mono flex-1 text-center font-semibold text-[12.5px]">
                        {isRecording ? (
                            capturedShortcut || t('workflows.awaiting_input')
                        ) : (
                            value || <span className="text-faint italic">{t('workflows.unbound')}</span>
                        )}
                    </span>

                    {isRecording ? (
                        <span className="mono text-[9.5px] font-semibold text-accent bg-accent-soft px-1.5 py-0.5 rounded">{t('workflows.rec_mode')}</span>
                    ) : (
                        <span className="flex gap-1 shrink-0">
                            {status === 'success' && <Check className="w-4 h-4 text-ok" />}
                            {status === 'error' && <AlertTriangle className="w-4 h-4 text-bad" />}
                        </span>
                    )}
                </button>

                {isRecording && (
                    <button
                        onClick={() => { setIsRecording(false); setCapturedShortcut(""); }}
                        className="w-9 h-9 flex items-center justify-center rounded-btn bg-bad-soft border border-bad/30 text-bad hover:bg-bad/15 transition-colors shrink-0"
                    >
                        <X className="w-4 h-4" />
                    </button>
                )}
                {!isRecording && value && (
                    <button
                        onClick={() => { setIsRecording(true); setCapturedShortcut(""); }}
                        title={t('workflows.rec_mode')}
                        className="w-9 h-9 flex items-center justify-center rounded-btn bg-bg-2 border border-line text-muted hover:text-ink hover:border-line-2 transition-colors shrink-0"
                    >
                        <RotateCw className="w-4 h-4" />
                    </button>
                )}
            </div>

            {statusMessage && (
                <div className={`text-[11px] pl-0.5 font-semibold ${status === 'error' ? 'text-bad' : status === 'success' ? 'text-ok' : 'text-muted'}`}>
                    {statusMessage === "READY" ? t('workflows.ready') : statusMessage}
                </div>
            )}
        </div>
    );
};

export default ShortcutRecorder;
