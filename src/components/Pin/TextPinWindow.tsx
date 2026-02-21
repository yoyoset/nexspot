import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { X, GripHorizontal, Copy, Check, Move, Sparkles } from 'lucide-react';
import ResizeHandles from './ResizeHandles';

const TextPinWindow: React.FC = () => {
    const [content, setContent] = useState<string>('');
    const [loading, setLoading] = useState(true);
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        // Parse ID from URL hash: #/text-pin?id=...
        const hash = window.location.hash;
        const urlParams = new URLSearchParams(hash.split('?')[1]);
        const id = urlParams.get('id');

        if (id) {
            invoke<string>('get_pin_content', { id })
                .then(async (text) => {
                    setContent(text);
                    setLoading(false);

                    // Check if this is a pending AI Pin
                    const match = text.match(/> \*\*Prompt:\*\* (.*)\n/);
                    if (match && match[1] && text.includes("# AI Processing...")) {
                        const prompt = match[1].trim();
                        console.log("Found Pending AI Prompt:", prompt);

                        // Start Streaming
                        setContent(`> **${prompt}**\n\n`); // Keep prompt, clear rest

                        const unlistenChunk = await listen<any>('ai-stream://chunk', (event) => {
                            if (event.payload.id === id) {
                                setContent(prev => prev + event.payload.content);
                            }
                        });

                        const unlistenDone = await listen<any>('ai-stream://done', (event) => {
                            if (event.payload.id === id) {
                                // Maybe save the final content to backend?
                                // invoke('update_pin_content', { id, content: ... }) - Not impl yet
                            }
                        });

                        const unlistenError = await listen<any>('ai-stream://error', (event) => {
                            if (event.payload.id === id) {
                                setContent(prev => prev + `\n\n**Error:** ${event.payload.error}`);
                            }
                        });

                        // Trigger Backend Stream
                        invoke('stream_ai_response', { pinId: id, prompt })
                            .catch(err => setContent(prev => prev + `\n\n**Error launching AI:** ${err}`));

                        // Cleanup listeners on unmount (or when window closes)
                        // Note: ensure this doesn't leak if re-renders happen. 
                        // Implementation detail: For now, we trust the window closes shortly or 
                        // we can store unlisten fns in a ref/state cleanup.
                        // Ideally we should use a separate useEffect for the streaming part or useRef.
                    }
                })
                .catch(err => {
                    console.error("Failed to load pin content:", err);
                    setContent("# Error\nFailed to load content.");
                    setLoading(false);
                });
        }
    }, []);

    const copyToClipboard = async () => {
        try {
            await navigator.clipboard.writeText(content);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('Failed to copy', err);
        }
    };

    const closeWindow = async () => {
        await getCurrentWindow().close();
    };

    return (
        <div className="w-screen h-screen bg-neutral-900/90 backdrop-blur-md border border-white/10 flex flex-col overflow-hidden text-white select-none rounded-lg shadow-2xl relative">
            {/* Custom Titlebar / Drag Region */}
            <div
                data-tauri-drag-region
                className="h-8 bg-white/5 border-b border-white/5 flex items-center justify-between px-2 shrink-0 cursor-move group"
            >
                <div className="flex items-center gap-2 opacity-50 pointer-events-none">
                    <Sparkles className="w-4 h-4 text-purple-400" />
                    <span className="text-[10px] font-mono uppercase tracking-wider">AI Result</span>
                </div>

                <div className="flex items-center gap-1">
                    <div
                        draggable
                        onDragStart={(e) => {
                            e.dataTransfer.setData("text/plain", content);
                            e.dataTransfer.effectAllowed = "copy";
                        }}
                        className="p-1 hover:bg-white/10 rounded transition-colors text-white/60 hover:text-white cursor-grab active:cursor-grabbing"
                        title="Drag Content Out"
                    >
                        <Move className="w-3.5 h-3.5" />
                    </div>

                    <button
                        onClick={copyToClipboard}
                        className="p-1 hover:bg-white/10 rounded transition-colors text-white/60 hover:text-white"
                        title="Copy All"
                    >
                        {copied ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
                    </button>
                    <button
                        onClick={closeWindow}
                        className="p-1 hover:bg-red-500/20 rounded transition-colors text-white/60 hover:text-red-400"
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar select-text">
                {loading ? (
                    <div className="flex items-center justify-center h-full text-white/30 text-sm animate-pulse">
                        Loading...
                    </div>
                ) : (
                    <div className="prose prose-invert prose-sm max-w-none break-words">
                        <ReactMarkdown>{content}</ReactMarkdown>
                    </div>
                )}
            </div>

            {/* Custom Resize Handles */}
            <ResizeHandles />
        </div>
    );
};

export default TextPinWindow;
