import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Save, Trash2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Workflow } from "../../store/useAppStore";
import WorkflowForm from "./WorkflowForm";

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
        try {
            await onSave(editForm);
            onClose();
        } catch (err) {
            console.error("Failed to save workflow:", err);
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <AnimatePresence>
            {isOpen && editForm && (
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-6 sm:p-12">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/60 backdrop-blur-md"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.9, y: 20 }}
                        className="relative w-full max-w-2xl bg-bg-card border border-border-hover rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-full"
                    >
                        {/* Header */}
                        <div className="px-6 py-5 border-b border-border-subtle flex items-center justify-between bg-bg-sidebar/50">
                            <div className="flex items-center gap-3">
                                <div className="p-2.5 bg-accent/10 rounded-xl text-accent">
                                    <Save className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="text-base font-bold text-text-main leading-tight">
                                        {workflow?.id?.startsWith('user_') ? t('workflows.new') : t('workflows.edit')}
                                    </h3>
                                    <p className="text-[10px] text-text-muted uppercase tracking-widest font-bold opacity-60">
                                        Preset Configuration
                                    </p>
                                </div>
                            </div>
                            <button
                                onClick={onClose}
                                className="p-2 text-text-muted hover:text-text-main hover:bg-bg-sidebar rounded-xl transition-all"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        {/* Body - Scrollable */}
                        <div className="flex-1 overflow-y-auto p-6 custom-scrollbar">
                            <WorkflowForm
                                workflow={editForm}
                                onChange={setEditForm}
                                save_path={save_path}
                            />
                        </div>

                        {/* Footer */}
                        <div className="px-6 py-4 bg-bg-sidebar/80 border-t border-border-subtle flex items-center justify-between">
                            <div>
                                {!editForm.is_system && onDelete && (
                                    <button
                                        onClick={() => {
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                onDelete(editForm.id);
                                                onClose();
                                            }
                                        }}
                                        className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold text-red-400 hover:bg-red-400/10 transition-all"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                        {t('workflows.delete')}
                                    </button>
                                )}
                            </div>
                            <div className="flex items-center gap-3">
                                <button
                                    onClick={onClose}
                                    className="px-5 py-2 rounded-xl text-xs font-bold text-text-muted hover:text-text-main hover:bg-bg-card transition-all"
                                >
                                    {t('ai.cancel')}
                                </button>
                                <button
                                    onClick={handleSave}
                                    disabled={isSaving}
                                    className="px-8 py-2.5 rounded-xl bg-accent text-white text-xs font-black shadow-lg shadow-accent/25 hover:shadow-accent/40 hover:-translate-y-0.5 active:translate-y-0 transition-all flex items-center gap-2 disabled:opacity-50 disabled:pointer-events-none"
                                >
                                    {isSaving ? (
                                        <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                                    ) : (
                                        <Check className="w-4 h-4 stroke-[3px]" />
                                    )}
                                    {t('ai.save')}
                                </button>
                            </div>
                        </div>

                        {/* Warning Info */}
                        {editForm.is_system && (
                            <div className="px-6 py-2 bg-amber-500/10 flex items-center gap-2 border-t border-amber-500/20">
                                <AlertCircle className="w-3.5 h-3.5 text-amber-500" />
                                <span className="text-[10px] text-amber-500/80 font-medium">System presets have restricted modifications.</span>
                            </div>
                        )}
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

const Check: React.FC<{ className?: string }> = ({ className }) => (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
    </svg>
);

export default WorkflowModal;
