import React from 'react';
import { LayoutDashboard, Activity, Settings, Lock, Unlock } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { AppTab } from '../../types/navigation';

interface NavigatorProps {
    activeTab: AppTab;
    onTabChange: (tab: AppTab) => void;
}

const Navigator: React.FC<NavigatorProps> = ({ activeTab, onTabChange }) => {
    const { t } = useTranslation();
    const [isAlwaysOnTop, setIsAlwaysOnTop] = React.useState(false);

    React.useEffect(() => {
        // Get initial status
        invoke<boolean>('is_pin_always_on_top').then(setIsAlwaysOnTop);
    }, []);

    const toggleAlwaysOnTop = async () => {
        try {
            const next = await invoke<boolean>('toggle_pin_always_on_top');
            setIsAlwaysOnTop(next);
        } catch (e) {
            console.error(e);
        }
    };

    const navItems = [
        { id: 'dashboard' as AppTab, icon: LayoutDashboard, label: t('nav.dashboard') },
        { id: 'activity' as AppTab, icon: Activity, label: t('nav.activity') },
        { id: 'settings' as AppTab, icon: Settings, label: t('nav.settings') },
    ];

    return (
        <nav className="w-10 h-full bg-bg-sidebar border-r border-border-subtle flex flex-col items-center py-2 gap-2 shrink-0 z-20">
            {navItems.map((item) => {
                const isActive = activeTab === item.id;
                const Icon = item.icon;

                return (
                    <button
                        key={item.id}
                        onClick={() => onTabChange(item.id)}
                        className="relative group p-2 rounded-sm transition-all"
                        title={item.label}
                    >
                        {isActive && (
                            <motion.div
                                layoutId="nav-active"
                                className="absolute inset-0 bg-accent/5 border border-accent/20 rounded-sm"
                                transition={{ duration: 0.1 }}
                            />
                        )}
                        <Icon
                            className={`w-3.5 h-3.5 transition-colors relative z-10 ${isActive ? 'text-accent' : 'text-text-muted group-hover:text-text-main'
                                }`}
                        />

                        {/* Tooltip (Industrial Style) */}
                        <div className="absolute left-full ml-1.5 px-1.5 py-0.5 bg-bg-card border border-border-subtle rounded-sm text-[9px] tech-text text-text-main opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap shadow-2xl z-50">
                            {item.label}
                        </div>
                    </button>
                );
            })}
            <div className="mt-auto flex flex-col items-center gap-1.5 pb-2">
                <button
                    onClick={toggleAlwaysOnTop}
                    className={`p-2 rounded-sm transition-all border ${isAlwaysOnTop ? 'text-accent bg-accent/5 border-accent/20' : 'text-text-muted border-transparent hover:text-text-main hover:bg-white/5'}`}
                    title={isAlwaysOnTop ? t('pin.tooltip_pin_off') : t('pin.tooltip_pin_on')} 
                >
                    {isAlwaysOnTop ? <Lock className="w-3.5 h-3.5" /> : <Unlock className="w-3.5 h-3.5" />}
                </button>
            </div>
        </nav>
    );
};

export default Navigator;
