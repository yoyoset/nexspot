import { TFunction } from "i18next";

export interface BackendError {
    code: 'Conflict' | 'RegistrationFailed' | 'NotFound' | 'InvalidFormat' | 'Empty' | 'Io';
    message: string;
}

export const translateError = (err: any, t: TFunction): string => {
    if (typeof err === 'object' && err !== null && 'code' in err) {
        const bErr = err as BackendError;
        switch (bErr.code) {
            case 'Empty': return t('settings.shortcuts.errors.empty');
            case 'Conflict':
                // If it's a conflict, the message often contains the label from backend
                // But we check for specialized formatting if needed
                if (bErr.message.includes("Conflict with")) {
                    // Extract label if possible or use generic
                    return bErr.message;
                }
                return bErr.message;
            case 'RegistrationFailed': return t('settings.shortcuts.errors.register_failed');
            case 'InvalidFormat': return t('settings.shortcuts.errors.invalid_format') || "Invalid Format";
            default: return bErr.message;
        }
    }

    if (typeof err === 'string') {
        if (err === "ERR_EMPTY") return t('settings.shortcuts.errors.empty');
        if (err.startsWith("ERR_CONFLICT|")) {
            const label = err.split('|')[1];
            return t('settings.shortcuts.errors.conflict', { label });
        }
        if (err.startsWith("ERR_REGISTER_FAILED")) {
            return t('settings.shortcuts.errors.register_failed');
        }
    }

    return String(err);
};
