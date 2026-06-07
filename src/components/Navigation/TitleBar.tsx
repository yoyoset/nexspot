import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface TitleBarProps {
    isAlwaysOnTop?: boolean;
}

/**
 * 自定义标题栏（无边框窗，decorations:false）。
 * 整条 data-tauri-drag-region 可拖动；右侧 Windows 三键。
 * 设计基准：docs/design/2026-06-04-studio-redesign/ §全局·主窗口外壳。
 */
const TitleBar: React.FC<TitleBarProps> = ({ isAlwaysOnTop }) => {
    const { t } = useTranslation();

    const onMinimize = () => getCurrentWindow().minimize();
    const onToggleMax = () => getCurrentWindow().toggleMaximize();
    const onClose = () => getCurrentWindow().close();

    return (
        <div
            data-tauri-drag-region
            className="h-[38px] shrink-0 flex items-center justify-between pl-4 pr-1 bg-bg-0 select-none z-30"
        >
            <div data-tauri-drag-region className="flex items-center gap-2 pointer-events-none">
                <div
                    className="w-4 h-4 rounded-[5px] shrink-0"
                    style={{ background: 'linear-gradient(135deg, var(--accent), #22d3ee)' }}
                />
                <span className="text-[12.5px] font-bold text-ink tracking-[-0.01em]">NexSpot</span>
                {isAlwaysOnTop && (
                    <span className="mono text-[10.5px] text-muted tracking-[0.04em]">
                        · {t('titlebar.pinned')}
                    </span>
                )}
            </div>

            <div className="flex items-center">
                <button
                    onClick={onMinimize}
                    aria-label={t('titlebar.minimize')}
                    className="w-[27px] h-[26px] flex items-center justify-center rounded-[6px] text-muted hover:text-ink hover:bg-bg-2 transition-colors"
                >
                    <Minus className="w-3.5 h-3.5" />
                </button>
                <button
                    onClick={onToggleMax}
                    aria-label={t('titlebar.maximize')}
                    className="w-[27px] h-[26px] flex items-center justify-center rounded-[6px] text-muted hover:text-ink hover:bg-bg-2 transition-colors"
                >
                    <Square className="w-3 h-3" />
                </button>
                <button
                    onClick={onClose}
                    aria-label={t('titlebar.close')}
                    className="w-[27px] h-[26px] flex items-center justify-center rounded-[6px] text-muted hover:text-white hover:bg-bad transition-colors"
                >
                    <X className="w-3.5 h-3.5" />
                </button>
            </div>
        </div>
    );
};

export default TitleBar;
