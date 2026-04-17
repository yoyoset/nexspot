import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Save, Trash2, AlertCircle, Check, Terminal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Workflow, useAppStore } from "../../store/useAppStore";
import WorkflowForm from "./WorkflowForm";
import { translateError } from "../../utils/error";

interface WorkflowModalProps {
    isOpen: boolean;
    onClose: () => void;
    workflow: Workflow | null;
    onSave: (w: Workflow) => Promise<void>;
    onDelete?: (id: string) => Promise<void>;
    save_path?: string;
}

const WorkflowModal: React.FC<WorkflowModalProps> = ({ isOpen, onClose, workflow, onSave, onDelete, save_path }) => {
    const { t } = useTranslation();
    const [editForm, setEditForm] = useState<Workflow | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [saveError, setSaveError] = useState<string | null>(null);

    useEffect(() => {
        if (workflow) {
            setEditForm({ ...workflow });
        } else {
            setEditForm(null);
        }
    }, [workflow]);

    const handleSave = async () => {
        if (!editForm) return;
        setIsSaving(true);
        setSaveError(null);
        try {
            await onSave(editForm);
            onClose();
        } catch (err) {
            console.error("Failed to save workflow:", err);
            const message = translateError(err, t);
            setSaveError(message);
            useAppStore.getState().showHUD(message, 'error');
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <AnimatePresence>
            {isOpen && editForm && (
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-2 sm:p-4">
                    {/* Backdrop (Darker Industrial) */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/80 backdrop-blur-sm"
                    />

                    {/* Modal Content (Industrial Sharp Box) */}
                    <motion.div
                        initial={{ opacity: 0, scale: 0.98 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.98 }}
                        className="relative w-full max-w-[95%] xl:max-w-[1400px] bg-bg-main border border-white/10 rounded-sm shadow-[0_0_100px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col max-h-[92vh]"
                    >
                        {/* Header */}
                        <div className="px-5 py-4 border-b border-white/5 flex items-center justify-between bg-black/40">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-accent/5 rounded-sm border border-accent/20 text-accent">
                                    <Terminal className="w-4 h-4" />
                                </div>
                                <div className="space-y-0.5">
                                    <h3 className="text-[12px] font-bold text-text-main tech-text uppercase tracking-widest leading-tight">
                                        {workflow?.id?.startsWith('user_') ? t('workflows.new_protocol') : t('workflows.edit_protocol')}
                                    </h3>
                                    <p className="text-[9px] text-text-muted tech-text uppercase tracking-widest opacity-40">
                                        {t('workflows.kernel_param_id')}: {editForm.id.toUpperCase()}
                                    </p>
                                </div>
                            </div>
                            <button
                                onClick={onClose}
                                className="p-2 text-text-muted hover:text-text-main hover:bg-white/5 rounded-sm transition-all"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        {/* Body - Scrollable */}
                        <div className="flex-1 overflow-y-auto p-5 custom-scrollbar bg-black/10">
                            <WorkflowForm
                                workflow={editForm}
                                onChange={setEditForm}
                                save_path={save_path}
                            />
                        </div>

                        {/* Inline Error Message */}
                        <AnimatePresence>
                            {saveError && (
                                <motion.div
                                    initial={{ height: 0, opacity: 0 }}
                                    animate={{ height: "auto", opacity: 1 }}
                                    exit={{ height: 0, opacity: 0 }}
                                    className="overflow-hidden bg-red-500/5 border-t border-red-500/10"
                                >
                                    <div className="px-5 py-2.5 flex items-center gap-3">
                                        <AlertCircle className="w-3.5 h-3.5 text-red-500 opacity-60 flex-shrink-0" />
                                        <div className="flex-1">
                                            <p className="text-[10px] text-red-500/80 tech-text uppercase font-bold tracking-widest">{saveError}</p>
                                        </div>
                                        <button
                                            onClick={() => setSaveError(null)}
                                            className="text-red-500/40 hover:text-red-500 transition-colors"
                                        >
                                            <X className="w-3.5 h-3.5" />
                                        </button>
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>

                        {/* Footer */}
                        <div className="px-5 py-4 bg-black/40 border-t border-white/5 flex items-center justify-between">
                            <div>
                                {!editForm.is_system && onDelete && (
                                    <button
                                        onClick={async () => {
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                try {
                                                    await onDelete(editForm.id);
                                                    onClose();
                                                } catch (err) {
                                                    console.error("Failed to delete workflow:", err);
                                                    const message = translateError(err, t);
                                                    useAppStore.getState().showHUD(message, 'error');
                                                }
                                            }
                                        }}
                                        className="flex items-center gap-2 px-3 py-2 rounded-sm text-[10px] tech-text font-bold text-red-500/50 hover:text-red-500 hover:bg-red-500/5 border border-transparent transition-all uppercase tracking-widest"
                                    >
                                        <Trash2 className="w-3.5 h-3.5" />
                                        {t('workflows.terminate_id')}
                                    </button>
                                )}
                            </div>
                            <div className="flex items-center gap-3">
                                <button
                                    onClick={onClose}
                                    className="px-4 py-2 rounded-sm text-[10px] tech-text font-bold text-text-muted hover:text-text-main hover:bg-white/5 border border-white/5 transition-all uppercase tracking-widest"
                                >
                                    {t('ai.cancel')}
                                </button>
                                <button
                                    onClick={handleSave}
                                    disabled={isSaving}
                                    className="px-6 py-2 rounded-sm bg-accent/20 hover:bg-accent text-accent hover:text-white text-[10px] tech-text font-bold border border-accent/40 shadow-lg shadow-accent/5 active:translate-y-0 transition-all flex items-center gap-2 disabled:opacity-30 disabled:pointer-events-none uppercase tracking-widest"
                                >
                                    {isSaving ? (
                                        <div className="w-3.5 h-3.5 border border-accent/20 border-t-accent rounded-sm animate-spin" />
                                    ) : (
                                        <Check className="w-3.5 h-3.5" />
                                    )}
                                    {t('workflows.commit_changes')}
                                </button>
                            </div>
                        </div>

                        {/* Warning Info */}
                        {editForm.is_system && (
                            <div className="px-5 py-2 bg-amber-500/5 flex items-center gap-2 border-t border-amber-500/10">
                                <AlertCircle className="w-3 h-3 text-amber-500 opacity-60" />
                                <span className="text-[9px] tech-text text-amber-500/60 font-bold uppercase tracking-widest">{t('workflows.read_only_mode_active')}</span>
                            </div>
                        )}
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default WorkflowModal;
