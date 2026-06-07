import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Trash2, AlertCircle, Check, Workflow as WorkflowIcon } from "lucide-react";
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
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ opacity: 0, y: 8, scale: 0.99 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: 8, scale: 0.99 }}
                        transition={{ duration: 0.18 }}
                        className="relative w-full max-w-[660px] bg-bg-1 border border-line-2 rounded-lg shadow-float overflow-hidden flex flex-col max-h-[90vh]"
                    >
                        {/* Header */}
                        <div className="px-6 py-4 border-b border-line flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <div className="w-9 h-9 rounded-[9px] bg-accent-soft flex items-center justify-center text-accent">
                                    <WorkflowIcon className="w-[18px] h-[18px]" />
                                </div>
                                <h3 className="text-[15px] font-extrabold text-ink tracking-[-0.01em]">
                                    {workflow?.id?.startsWith('user_') ? t('workflows.new_protocol') : t('workflows.edit_protocol')}
                                </h3>
                            </div>
                            <button
                                onClick={onClose}
                                className="w-[30px] h-[30px] flex items-center justify-center text-muted hover:text-ink hover:bg-bg-2 rounded-btn transition-colors"
                            >
                                <X className="w-4 h-4" />
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

                        {/* Inline Error Message */}
                        <AnimatePresence>
                            {saveError && (
                                <motion.div
                                    initial={{ height: 0, opacity: 0 }}
                                    animate={{ height: "auto", opacity: 1 }}
                                    exit={{ height: 0, opacity: 0 }}
                                    className="overflow-hidden bg-bad-soft border-t border-bad/20"
                                >
                                    <div className="px-6 py-2.5 flex items-center gap-3">
                                        <AlertCircle className="w-4 h-4 text-bad flex-shrink-0" />
                                        <p className="flex-1 text-[12px] text-bad font-semibold">{saveError}</p>
                                        <button onClick={() => setSaveError(null)} className="text-bad/60 hover:text-bad transition-colors">
                                            <X className="w-4 h-4" />
                                        </button>
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>

                        {/* Read-only notice */}
                        {editForm.is_system && (
                            <div className="px-6 py-2 bg-warn-soft flex items-center gap-2 border-t border-warn/20">
                                <AlertCircle className="w-3.5 h-3.5 text-warn" />
                                <span className="text-[11.5px] text-warn font-semibold">{t('workflows.read_only_mode_active')}</span>
                            </div>
                        )}

                        {/* Footer */}
                        <div className="px-6 py-4 border-t border-line flex items-center justify-between">
                            <div>
                                {!editForm.is_system && onDelete && (
                                    <button
                                        onClick={async () => {
                                            if (window.confirm(t('workflows.delete_confirm'))) {
                                                try {
                                                    await onDelete(editForm.id);
                                                    onClose();
                                                } catch (err) {
                                                    const message = translateError(err, t);
                                                    useAppStore.getState().showHUD(message, 'error');
                                                }
                                            }
                                        }}
                                        className="flex items-center gap-1.5 px-2.5 py-2 rounded-btn text-[12.5px] font-semibold text-muted hover:text-bad hover:bg-bad-soft transition-colors"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                        {t('dashboard.delete')}
                                    </button>
                                )}
                            </div>
                            <div className="flex items-center gap-2">
                                <button
                                    onClick={onClose}
                                    className="px-4 py-2 rounded-btn text-[12.5px] font-semibold text-muted hover:text-ink hover:bg-bg-2 transition-colors"
                                >
                                    {t('common.cancel')}
                                </button>
                                <button
                                    onClick={handleSave}
                                    disabled={isSaving}
                                    className="px-5 py-2 rounded-btn bg-accent text-on-accent text-[12.5px] font-semibold hover:bg-[var(--accent-press)] transition-colors flex items-center gap-1.5 disabled:opacity-40 disabled:pointer-events-none"
                                >
                                    {isSaving ? (
                                        <div className="w-3.5 h-3.5 border-2 border-on-accent/30 border-t-on-accent rounded-full animate-spin" />
                                    ) : (
                                        <Check className="w-4 h-4" />
                                    )}
                                    {t('common.save')}
                                </button>
                            </div>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};

export default WorkflowModal;
