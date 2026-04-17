import React from "react";
import { ChevronRight } from "lucide-react";

interface IndustrialPanelProps {
    title?: string;
    description?: string;
    icon?: React.ReactNode;
    children?: React.ReactNode;
    className?: string;
    colSpan?: number;
    onClick?: () => void;
    active?: boolean;
}

const IndustrialPanel: React.FC<IndustrialPanelProps> = ({
    title,
    description,
    icon,
    children,
    className = "",
    colSpan = 1,
    onClick,
    active = false,
}) => {
    const colSpanClass = {
        1: "col-span-1", 2: "col-span-2", 3: "col-span-3", 4: "col-span-4",
        5: "col-span-5", 6: "col-span-6", 7: "col-span-7", 8: "col-span-8",
        9: "col-span-9", 10: "col-span-10", 11: "col-span-11", 12: "col-span-12",
    }[colSpan] || "col-span-1";

    return (
        <div
            className={`industrial-panel relative overflow-hidden rounded-sm p-3 flex flex-col group ${colSpanClass} ${onClick ? "cursor-pointer hover:bg-white/[0.02]" : ""
                } ${active ? "border-accent/40 bg-accent/[0.02]" : ""
                } ${className}`}
            onClick={onClick}
        >
            <div className="flex items-center justify-between mb-3 border-b border-white/5 pb-2">
                <div className="flex items-center gap-2">
                    {icon && (
                        <div className="p-1.5 rounded-sm bg-white/5 text-text-muted group-hover:text-accent transition-colors">
                            {icon}
                        </div>
                    )}
                    {title && <h3 className="text-[12px] font-bold text-text-main tracking-tight tech-text">{title}</h3>}
                </div>
                {onClick && (
                    <ChevronRight className="w-3 h-3 text-text-muted opacity-20 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all" />
                )}
            </div>

            {description && <p className="text-[10px] text-text-muted mb-1 tech-text opacity-60 leading-tight">{description}</p>}

            <div className="flex-1">{children}</div>

            {/* Sharp Selection Indicator */}
            {active && (
                <div className="absolute top-0 left-0 w-1 h-full bg-accent/40" />
            )}
        </div>
    );
};

export default IndustrialPanel;
