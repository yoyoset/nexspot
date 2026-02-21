import React, { useState } from "react";
import { Plus, Trash2, Edit2, Sparkles, X, Check } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import ShortcutRecorder from "../Settings/ShortcutRecorder";
import { useConfig } from "../../hooks/useConfig";
import { useAppStore, AIShortcut } from "../../store/useAppStore";

interface AISectionProps {
    onCreateTrigger?: (fn: () => void) => void;
    isCompact?: boolean;
}

const AISection: React.FC<AISectionProps> = ({ onCreateTrigger, isCompact }) => {
    const { config, addAIShortcut, removeAIShortcut, updateAIShortcut } = useConfig();
    const { t } = useTranslation();
    const shortcuts = config?.ai_shortcuts || [];

    // ... same state ...
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editForm, setEditForm] = useState<Partial<AIShortcut>>({});
    const [isCreating, setIsCreating] = useState(false);

    const startCreating = () => {
        setIsCreating(true);
        setEditingId(null);
        setEditForm({
            name: t('ai.default_name'),
            prompt: "",
            shortcut: ""
        });
    };

    React.useEffect(() => {
        if (onCreateTrigger) {
            onCreateTrigger(startCreating);
        }
    }, [onCreateTrigger]);

    const startEditing = (s: AIShortcut) => {
        setEditingId(s.id);
        setIsCreating(false);
        setEditForm({ ...s });
    };

    const cancelEdit = () => {
        setEditingId(null);
        setIsCreating(false);
        setEditForm({});
    };

    const saveEdit = async () => {
        if (!editForm.name || !editForm.prompt) return;

        if (isCreating) {
            const newShortcut: AIShortcut = {
                id: crypto.randomUUID(),
                name: editForm.name,
                prompt: editForm.prompt,
                shortcut: editForm.shortcut || undefined
            };
            await addAIShortcut(newShortcut);
        } else if (editingId) {
            const existing = shortcuts.find(s => s.id === editingId);
            if (existing) {
                await updateAIShortcut(editingId, {
                    ...existing,
                    ...editForm as AIShortcut
                });
            }
        }
        cancelEdit();
    };

    const allWorkflows = config?.workflows || [];
    const shortcutConflicts = React.useMemo(() => {
        const counts: Record<string, number> = {};
        [...allWorkflows, ...shortcuts].forEach(item => {
            if (item.shortcut) {
                // For workflows, check if enabled. AI shortcuts are always considered candidate.
                const isEnabled = (item as any).enabled !== false;
                if (isEnabled) {
                    counts[item.shortcut] = (counts[item.shortcut] || 0) + 1;
                }
            }
        });
        return counts;
    }, [allWorkflows, shortcuts]);

    if (isCompact) {
        return (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 w-full">
                <AnimatePresence>
                    {isCreating && (
                        <div className="col-span-full border-b border-white/5 pb-4 mb-2">
                            <EditRow form={editForm} setForm={setEditForm} onSave={saveEdit} onCancel={cancelEdit} />
                        </div>
                    )}
                    {shortcuts.map(s => {
                        const hasConflict = s.shortcut ? (shortcutConflicts[s.shortcut] || 0) > 1 : false;
                        return editingId === s.id ? (
                            <div key={s.id} className="col-span-full border border-purple-500/20 rounded-2xl p-4 bg-purple-500/5 mb-2">
                                <EditRow form={editForm} setForm={setEditForm} onSave={saveEdit} onCancel={cancelEdit} />
                            </div>
                        ) : (
                            <motion.div
                                key={s.id}
                                layout
                                whileHover={{ x: 2 }}
                                whileTap={{ scale: 0.98 }}
                                className={`group/tile flex items-center justify-between p-3.5 rounded-xl bg-white/[0.02] border transition-all cursor-pointer overflow-hidden ${hasConflict ? 'border-red-500/20 bg-red-500/5' : 'border-white/5 hover:bg-white/[0.05] hover:border-purple-500/20'}`}
                            >
                                <div className="flex items-center gap-3.5 overflow-hidden">
                                    <div className={`p-2 rounded-lg transition-all shrink-0 ${hasConflict ? 'bg-red-500/10 text-red-400' : 'bg-white/5 text-purple-400/60 group-hover/tile:text-purple-400'}`}>
                                        <div className="w-3.5 h-3.5 flex items-center justify-center">
                                            <Sparkles className="w-4 h-4" />
                                        </div>
                                    </div>
                                    <div className="flex flex-col overflow-hidden">
                                        <span className="text-[13px] font-bold text-text-main/90 truncate tracking-tight leading-none mb-1">{s.name}</span>
                                        <div className="flex items-center gap-2">
                                            {s.shortcut && (
                                                <span className={`text-[9px] font-mono font-black uppercase tracking-tighter ${hasConflict ? 'text-red-400/60' : 'text-text-muted/30'}`}>{s.shortcut}</span>
                                            )}
                                        </div>
                                    </div>
                                </div>
                                <div className="flex items-center gap-1 opacity-0 group-hover/tile:opacity-100 transition-all ml-2 shrink-0">
                                    <button
                                        onClick={(e) => { e.stopPropagation(); startEditing(s); }}
                                        className="p-1.5 rounded-lg hover:bg-white/10 text-text-muted hover:text-text-main transition-all"
                                    >
                                        <Edit2 className="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                        onClick={(e) => { e.stopPropagation(); removeAIShortcut(s.id); }}
                                        className="p-1.5 rounded-lg hover:bg-red-500/10 text-red-500/20 hover:text-red-500 transition-all"
                                    >
                                        <Trash2 className="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            </motion.div>
                        );
                    })}
                    {!isCreating && shortcuts.length === 0 && (
                        <div className="col-span-full py-8 flex items-center justify-center border border-dashed border-purple-500/10 rounded-2xl opacity-10">
                            <span className="text-[10px] font-black tracking-widest uppercase text-text-muted">{t('ai.empty')}</span>
                        </div>
                    )}
                </AnimatePresence>
            </div>
        );
    }

    return (
        <>
            <AnimatePresence>
                {isCreating && (
                    <div className="col-span-2">
                        <EditRow
                            form={editForm}
                            setForm={setEditForm}
                            onSave={saveEdit}
                            onCancel={cancelEdit}
                        />
                    </div>
                )}

                {shortcuts.map(s => (
                    editingId === s.id ? (
                        <div key={s.id} className="col-span-2">
                            <EditRow
                                form={editForm}
                                setForm={setEditForm}
                                onSave={saveEdit}
                                onCancel={cancelEdit}
                            />
                        </div>
                    ) : (
                        <motion.div
                            key={s.id}
                            layout
                            whileHover={{ y: -4, scale: 1.005 }}
                            whileTap={{ scale: 0.98 }}
                            className="bg-bg-subtle/30 border border-border-subtle/60 rounded-[32px] p-5 flex flex-col gap-5 group hover:bg-bg-subtle/60 hover:border-purple-500/40 cursor-pointer shadow-xl transition-all relative overflow-hidden h-full"
                        >
                            <div className="absolute top-0 left-0 w-full h-[3px] bg-purple-500/0 group-hover:bg-purple-500/20 transition-all duration-700" />

                            <div className="flex items-start justify-between relative z-10">
                                <div className="flex items-center gap-5 overflow-hidden">
                                    <div className="p-3 bg-purple-500/10 border border-white/[0.05] rounded-2xl text-purple-400 shrink-0 shadow-inner">
                                        <Sparkles className="w-5 h-5" />
                                    </div>
                                    <div className="overflow-hidden">
                                        <div className="text-[15px] font-black text-text-main truncate leading-none tracking-tight mb-2">
                                            {s.name}
                                        </div>
                                        <div className="text-[10px] text-text-muted/50 uppercase font-black tracking-widest">
                                            {t('ai.title')}
                                        </div>
                                    </div>
                                </div>

                                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-3 group-hover:translate-x-0">
                                    <button
                                        onClick={(e) => { e.stopPropagation(); startEditing(s); }}
                                        className="p-2.5 text-text-muted/60 hover:text-text-main hover:bg-bg-card rounded-2xl transition-all border border-transparent hover:border-border-subtle"
                                    >
                                        <Edit2 className="w-5 h-5" />
                                    </button>
                                    <button
                                        onClick={(e) => { e.stopPropagation(); removeAIShortcut(s.id); }}
                                        className="p-2.5 text-red-500/30 hover:text-red-400 hover:bg-red-400/10 rounded-2xl transition-all border border-transparent hover:border-red-400/20"
                                    >
                                        <Trash2 className="w-5 h-5" />
                                    </button>
                                </div>
                            </div>

                            <div className="flex items-end justify-between mt-auto pt-3 relative z-10 border-t border-border-subtle/20">
                                <div className="flex flex-col gap-2.5 flex-1 min-w-0">
                                    <div className="text-[20px] font-black text-text-accent font-mono tracking-tighter leading-none group-hover:scale-105 origin-left transition-transform duration-500">
                                        {s.shortcut || '---'}
                                    </div>
                                    <div className="text-[10px] text-text-muted/30 font-mono truncate uppercase tracking-tighter w-full group-hover:opacity-60 transition-opacity">
                                        {s.prompt}
                                    </div>
                                </div>
                            </div>
                        </motion.div>
                    )
                ))}

                {!isCreating && shortcuts.length === 0 && (
                    <div className="col-span-2 py-6 flex items-center justify-center text-text-muted opacity-20 border border-dashed border-border-subtle rounded-xl bg-bg-main/5">
                        <span className="text-[10px] font-bold tracking-widest uppercase">{t('ai.empty')}</span>
                    </div>
                )}
            </AnimatePresence>
        </>
    );
};

const EditRow: React.FC<{
    form: Partial<AIShortcut>;
    setForm: React.Dispatch<React.SetStateAction<Partial<AIShortcut>>>;
    onSave: () => void;
    onCancel: () => void;
}> = ({ form, setForm, onSave, onCancel }) => {
    const { t } = useTranslation();

    return (
        <motion.div
            layout
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="bg-purple-900/20 border border-purple-500/30 rounded-xl p-3 space-y-3"
        >
            <div className="flex gap-2">
                <input
                    type="text"
                    value={form.name || ""}
                    onChange={e => setForm({ ...form, name: e.target.value })}
                    placeholder={t('ai.field_name')}
                    className="flex-1 bg-bg-main border border-border-subtle rounded-lg px-2 py-1.5 text-xs text-text-main placeholder-text-muted/40 focus:border-purple-500/50 outline-none"
                    autoFocus
                />
                <ShortcutRecorder
                    value={form.shortcut || ""}
                    onChange={k => setForm({ ...form, shortcut: k })}
                    className="w-24 h-[30px] text-[10px]"
                    placeholder={t('ai.bind')}
                />
            </div>
            <textarea
                value={form.prompt || ""}
                onChange={e => setForm({ ...form, prompt: e.target.value })}
                placeholder={t('ai.field_prompt')}
                className="w-full bg-bg-main border border-border-subtle rounded-lg px-2 py-2 text-xs text-text-main placeholder-text-muted/40 focus:border-purple-500/50 outline-none resize-none h-16 font-mono"
            />
            <div className="flex justify-end gap-2">
                <button onClick={onCancel} className="px-3 py-1.5 rounded-lg bg-bg-subtle hover:bg-bg-card text-text-muted text-xs">
                    {t('ai.cancel')}
                </button>
                <button onClick={onSave} className="px-3 py-1.5 rounded-lg bg-purple-500 hover:bg-purple-600 text-white text-xs font-medium flex items-center gap-1">
                    <Check className="w-3 h-3" /> {t('ai.save')}
                </button>
            </div>
        </motion.div>
    );
};

export default AISection;
