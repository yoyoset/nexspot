import React from "react";
import { motion } from "framer-motion";

/**
 * Studio 设置页共享原子。设计基准：docs/design/2026-06-04-studio-redesign/ §3 Settings。
 * SectionTitle / Row / Toggle / Segmented / Stepper / TextField / Panel。
 */

export const SectionTitle: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <h3 className="mono text-[10.5px] font-semibold tracking-[0.12em] uppercase text-faint mb-2">
        {children}
    </h3>
);

export const Row: React.FC<{
    title: string;
    hint?: string;
    children: React.ReactNode;
    last?: boolean;
    align?: 'center' | 'start';
}> = ({ title, hint, children, last, align = 'center' }) => (
    <div className={`flex ${align === 'center' ? 'items-center' : 'items-start'} justify-between gap-4 py-3.5 ${last ? '' : 'border-b border-line'}`}>
        <div className="flex flex-col gap-0.5 min-w-0">
            <span className="text-[13px] font-semibold text-ink">{title}</span>
            {hint && <span className="text-[11.5px] text-muted leading-snug">{hint}</span>}
        </div>
        <div className="shrink-0">{children}</div>
    </div>
);

export const Toggle: React.FC<{ checked: boolean; onChange: (v: boolean) => void; disabled?: boolean }> = ({ checked, onChange, disabled }) => (
    <button
        type="button"
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={`relative w-10 h-[22px] rounded-full transition-colors shrink-0 ${disabled ? 'opacity-40 cursor-not-allowed' : ''} ${checked ? 'bg-accent' : 'bg-bg-3 border border-line-2'}`}
        aria-pressed={checked}
    >
        <motion.span
            layout
            transition={{ type: 'spring', stiffness: 500, damping: 34 }}
            className={`absolute top-1/2 -translate-y-1/2 w-[16px] h-[16px] rounded-full ${checked ? 'right-[3px] bg-on-accent' : 'left-[3px] bg-muted'}`}
        />
    </button>
);

export function Segmented<T extends string>({ value, options, onChange }: {
    value: T;
    options: { value: T; label: React.ReactNode }[];
    onChange: (v: T) => void;
}) {
    return (
        <div className="inline-flex items-center gap-0.5 p-0.5 rounded-btn bg-bg-2 border border-line">
            {options.map((opt) => {
                const active = opt.value === value;
                return (
                    <button
                        key={opt.value}
                        type="button"
                        onClick={() => onChange(opt.value)}
                        className={`px-3 py-1 rounded-[7px] text-[12px] font-semibold transition-colors ${active ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'}`}
                    >
                        {opt.label}
                    </button>
                );
            })}
        </div>
    );
}

export const Stepper: React.FC<{ value: number; min?: number; max?: number; onChange: (v: number) => void }> = ({ value, min = 1, max = 8, onChange }) => (
    <div className="inline-flex items-center gap-1 p-0.5 rounded-btn bg-bg-2 border border-line">
        <button
            type="button"
            onClick={() => onChange(Math.max(min, value - 1))}
            className="w-7 h-7 flex items-center justify-center rounded-[7px] text-muted hover:text-ink hover:bg-bg-3 transition-colors text-[15px] leading-none"
        >−</button>
        <span className="w-8 text-center mono text-[12.5px] font-semibold text-ink">{value}</span>
        <button
            type="button"
            onClick={() => onChange(Math.min(max, value + 1))}
            className="w-7 h-7 flex items-center justify-center rounded-[7px] text-muted hover:text-ink hover:bg-bg-3 transition-colors text-[15px] leading-none"
        >+</button>
    </div>
);

export const TextField = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement> & { mono?: boolean }>(
    ({ mono, className = "", ...props }, ref) => (
        <input
            ref={ref}
            {...props}
            className={`h-9 bg-bg-3 border border-line rounded-btn px-3 text-[12.5px] text-ink outline-none focus:border-accent transition-colors placeholder:text-faint ${mono ? 'mono' : ''} ${className}`}
        />
    )
);
TextField.displayName = 'TextField';

export const Panel: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className = "" }) => (
    <div className={`bg-bg-1 border border-line rounded-panel shadow-sm ${className}`}>{children}</div>
);
