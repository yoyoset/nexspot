import React from "react";
import { MoveRight } from "lucide-react";

interface BentoCardProps {
    title?: string;
    description?: string;
    icon?: React.ReactNode;
    children?: React.ReactNode;
    className?: string;
    colSpan?: number;
    onClick?: () => void;
    active?: boolean;
}

const BentoCard: React.FC<BentoCardProps> = ({
    title,
    description,
    icon,
    children,
    className = "",
    colSpan = 1,
    onClick,
    active = false,
}) => {
    // Map numerical colSpan to Tailwind classes
    const colSpanClass = {
        1: "col-span-1",
        2: "col-span-2",
        3: "col-span-3",
        4: "col-span-4",
        5: "col-span-5",
        6: "col-span-6",
        7: "col-span-7",
        8: "col-span-8",
        9: "col-span-9",
        10: "col-span-10",
        11: "col-span-11",
        12: "col-span-12",
    }[colSpan] || "col-span-1";

    return (
        <div
            className={`bento-card relative overflow-hidden rounded-2xl p-5 flex flex-col group ${colSpanClass} ${onClick ? "cursor-pointer hover:bg-white/[0.02]" : ""
                } ${active ? "border-accent/40 shadow-[0_0_20px_rgba(59,130,246,0.1)]" : ""
                } ${className}`}
            onClick={onClick}
        >
            <div className="flex items-start justify-between mb-2">
                {icon && (
                    <div className="p-2 rounded-lg bg-white/5 text-white/70 group-hover:text-accent group-hover:bg-accent/10 transition-colors">
                        {icon}
                    </div>
                )}
                {onClick && (
                    <MoveRight className="w-4 h-4 text-white/20 group-hover:text-white/50 group-hover:translate-x-1 transition-all" />
                )}
            </div>

            {title && <h3 className="text-lg font-medium text-text-main mb-1">{title}</h3>}
            {description && <p className="text-xs text-text-muted">{description}</p>}

            <div className="flex-1 mt-4">{children}</div>

            {/* Hover Glow Effect */}
            <div className="absolute -inset-px opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none">
                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent blur-sm" />
            </div>
        </div>
    );
};

export default BentoCard;
