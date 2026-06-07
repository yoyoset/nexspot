import React from 'react';
import { LayoutDashboard, Activity, Settings, Pin, PinOff } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { AppTab } from '../../types/navigation';

interface NavigatorProps {
    activeTab: AppTab;
    onTabChange: (tab: AppTab) => void;
    isAlwaysOnTop: boolean;
    onToggleAlwaysOnTop: () => void;
}

/**
 * 活动栏 Rail（宽 48，底 bg0）。纯图标 34×34，圆角 9。
 * 激活态：accent-soft 底 + accent 图标 + 左缘竖条。底部窗口置顶开关。
 * 设计基准：docs/design/2026-06-04-studio-redesign/ §全局·活动栏 Rail。
 */
const Navigator: React.FC<NavigatorProps> = ({ activeTab, onTabChange, isAlwaysOnTop, onToggleAlwaysOnTop }) => {
    const { t } = useTranslation();

    const navItems = [
        { id: 'dashboard' as AppTab, icon: LayoutDashboard, label: t('nav.dashboard') },
        { id: 'activity' as AppTab, icon: Activity, label: t('nav.activity') },
        { id: 'settings' as AppTab, icon: Settings, label: t('nav.settings') },
    ];

    const RailButton: React.FC<{
        active: boolean;
        onClick: () => void;
        tip: string;
        children: React.ReactNode;
        accent?: boolean;
    }> = ({ active, onClick, tip, children, accent }) => (
        <button
            onClick={onClick}
            className="relative group w-[34px] h-[34px] flex items-center justify-center rounded-[9px] transition-colors hover:bg-bg-2"
            aria-label={tip}
        >
            {active && (
                <>
                    <span className="absolute inset-0 rounded-[9px] bg-accent-soft" />
                    <span className="absolute -left-[7px] top-1/2 -translate-y-1/2 w-[3px] h-[18px] rounded-full bg-accent" />
                </>
            )}
            <span className={`relative z-10 ${active || accent ? 'text-accent' : 'text-muted group-hover:text-ink'} transition-colors`}>
                {children}
            </span>

            {/* Tooltip 右侧 */}
            <span className="absolute left-[42px] px-2 py-1 bg-bg-3 border border-line-2 rounded-[7px] text-[11px] text-ink opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap shadow-float z-50">
                {tip}
            </span>
        </button>
    );

    return (
        <nav className="w-12 h-full bg-bg-0 flex flex-col items-center pt-3 pb-3 gap-2 shrink-0 z-20">
            {navItems.map((item) => {
                const Icon = item.icon;
                return (
                    <RailButton
                        key={item.id}
                        active={activeTab === item.id}
                        onClick={() => onTabChange(item.id)}
                        tip={item.label}
                    >
                        <Icon className="w-[18px] h-[18px]" />
                    </RailButton>
                );
            })}

            <div className="mt-auto">
                <RailButton
                    active={false}
                    accent={isAlwaysOnTop}
                    onClick={onToggleAlwaysOnTop}
                    tip={isAlwaysOnTop ? t('pin.tooltip_pin_off') : t('pin.tooltip_pin_on')}
                >
                    {isAlwaysOnTop ? <Pin className="w-[18px] h-[18px]" /> : <PinOff className="w-[18px] h-[18px]" />}
                </RailButton>
            </div>
        </nav>
    );
};

export default Navigator;
