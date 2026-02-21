import React from 'react';
import { LayoutDashboard, Activity, Settings } from 'lucide-react';
import { motion } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { AppTab } from '../../types/navigation';

interface NavigatorProps {
    activeTab: AppTab;
    onTabChange: (tab: AppTab) => void;
}

const Navigator: React.FC<NavigatorProps> = ({ activeTab, onTabChange }) => {
    const { t } = useTranslation();

    const navItems = [
        { id: 'dashboard' as AppTab, icon: LayoutDashboard, label: t('nav.dashboard') },
        { id: 'activity' as AppTab, icon: Activity, label: t('nav.activity') },
        { id: 'settings' as AppTab, icon: Settings, label: t('nav.settings') },
    ];

    return (
        <nav className="w-14 h-full bg-bg-sidebar border-r border-border-subtle flex flex-col items-center py-6 gap-6 shrink-0 z-20">
            {navItems.map((item) => {
                const isActive = activeTab === item.id;
                const Icon = item.icon;

                return (
                    <button
                        key={item.id}
                        onClick={() => onTabChange(item.id)}
                        className="relative group p-2 rounded-xl transition-all"
                        title={item.label}
                    >
                        {isActive && (
                            <motion.div
                                layoutId="nav-active"
                                className="absolute inset-0 bg-accent/10 border border-accent/20 rounded-xl"
                                transition={{ type: 'spring', bounce: 0.2, duration: 0.6 }}
                            />
                        )}
                        <Icon
                            className={`w-5 h-5 transition-colors relative z-10 ${isActive ? 'text-accent' : 'text-text-muted group-hover:text-text-main'
                                }`}
                        />

                        {/* Tooltip (Optional, for better UX) */}
                        <div className="absolute left-full ml-3 px-2 py-1 bg-bg-card border border-border-subtle rounded text-[10px] font-bold text-text-main opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap shadow-xl">
                            {item.label}
                        </div>
                    </button>
                );
            })}
        </nav>
    );
};

export default Navigator;
