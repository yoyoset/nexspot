import React, { useState, useEffect, useRef } from "react";
import { Keyboard as KeyboardIcon, Check, X, AlertTriangle, RotateCw } from "lucide-react";

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
    const [isRecording, setIsRecording] = useState(false);
    const [currentKeys, setCurrentKeys] = useState<Set<string>>(new Set());
    // Ref to handle focus/blur
    const containerRef = useRef<HTMLDivElement>(null);

    // Normalize key names
    const normalizeKey = (key: string) => {
        if (key === " ") return "Space";
        if (key.length === 1) return key.toUpperCase();
        return key;
    };

    // Format keys for display
    const formatKeys = (keys: Set<string>) => {
        const sorted = Array.from(keys).sort((a, b) => {
            const modifiers = ["Ctrl", "Alt", "Shift", "Meta"];
            const aIsMod = modifiers.includes(a);
            const bIsMod = modifiers.includes(b);
            if (aIsMod && !bIsMod) return -1;
            if (!aIsMod && bIsMod) return 1;
            return a.localeCompare(b);
        });
        // Windows/Linux uses +, macOS usually represents these with symbols but + is fine for cross-platform
        return sorted.join(" + ");
    };

    useEffect(() => {
        if (!isRecording) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();

            const newKeys = new Set(currentKeys);

            // Map modifiers
            if (e.ctrlKey) newKeys.add("Ctrl");
            if (e.altKey) newKeys.add("Alt");
            if (e.shiftKey) newKeys.add("Shift");
            if (e.metaKey) newKeys.add("Meta");

            // Add main key if it's not a modifier
            if (!["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
                newKeys.add(normalizeKey(e.key));
            }

            setCurrentKeys(newKeys);
        };

        const handleKeyUp = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();

            // Heuristic for "completion": if a non-modifier key is released
            if (currentKeys.size > 0 && !["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
                onChange(formatKeys(currentKeys));
                setIsRecording(false);
            }
        };

        // If clicking outside, cancel recording
        const handleClickOutside = (e: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
                setIsRecording(false);
                setCurrentKeys(new Set());
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("keyup", handleKeyUp);
        window.addEventListener("mousedown", handleClickOutside);

        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            window.removeEventListener("keyup", handleKeyUp);
            window.removeEventListener("mousedown", handleClickOutside);
        };
    }, [isRecording, currentKeys, onChange]);

    return (
        <div className={`flex flex-col gap-1 ${className}`} ref={containerRef}>
            <div className="flex items-center gap-2">
                <div
                    className={`relative flex-1 flex items-center gap-2 px-3 py-2 rounded-lg border text-sm transition-all
                        ${isRecording
                            ? "bg-accent/10 border-accent text-accent ring-2 ring-accent/20"
                            : status === 'error'
                                ? "bg-red-500/10 border-red-500/30 text-white"
                                : "bg-white/5 border-white/10 text-text-main"
                        }
                    `}
                >
                    <KeyboardIcon className={`w-4 h-4 ${isRecording ? "animate-pulse" : "opacity-40"}`} />

                    <span className="font-mono flex-1 text-center font-bold tracking-wide">
                        {isRecording ? (
                            currentKeys.size > 0 ? formatKeys(currentKeys) : "Type shortcut..."
                        ) : (
                            value || <span className="opacity-30 italic">{placeholder}</span>
                        )}
                    </span>

                    {/* Status Indicator / Actions */}
                    {isRecording ? (
                        <div className="text-[10px] uppercase font-bold text-accent animate-pulse">REC</div>
                    ) : (
                        <div className="flex gap-1">
                            {status === 'success' && <Check className="w-4 h-4 text-green-500" />}
                            {status === 'error' && <AlertTriangle className="w-4 h-4 text-red-500" />}
                        </div>
                    )}
                </div>

                {/* Re-record / Action Button */}
                <button
                    onClick={() => {
                        setIsRecording(!isRecording);
                        if (!isRecording) setCurrentKeys(new Set());
                    }}
                    className={`p-2 rounded-lg border border-white/10 hover:bg-white/10 transition-colors
                         ${isRecording ? 'bg-red-500/20 border-red-500/50 text-red-500 hover:bg-red-500/30' : 'text-text-muted hover:text-white'}
                    `}
                    title={isRecording ? "Cancel Recording" : "Record New Shortcut"}
                >
                    {isRecording ? <X className="w-4 h-4" /> : <RotateCw className="w-4 h-4" />}
                </button>
            </div>

            {/* Context feedback message */}
            {statusMessage && (
                <div className={`text-[10px] pl-1 ${status === 'error' ? 'text-red-400' :
                        status === 'success' ? 'text-green-400' : 'text-text-muted'
                    }`}>
                    {statusMessage}
                </div>
            )}
        </div>
    );
};

export default ShortcutRecorder;
