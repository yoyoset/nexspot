import { useState, useCallback, useRef } from 'react';
import { HUDType } from '../components/Overlay/GlobalHUD';

export const useHUD = () => {
    const [hud, setHud] = useState<{ message: string; type: HUDType; visible: boolean }>({
        message: '',
        type: 'success',
        visible: false,
    });

    const timeoutRef = useRef<number | null>(null);

    const showHUD = useCallback((message: string, type: HUDType = 'success', duration = 2000) => {
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }

        setHud({ message, type, visible: true });

        timeoutRef.current = window.setTimeout(() => {
            setHud(prev => ({ ...prev, visible: false }));
            timeoutRef.current = null;
        }, duration);
    }, []);

    return { hud, showHUD };
};
