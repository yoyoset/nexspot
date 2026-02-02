interface SizeTagProps {
    width: number;
    height: number;
}

export default function SizeTag({ width, height }: SizeTagProps) {
    if (width <= 20) return null;

    return (
        <div className="absolute -top-6 left-0 bg-blue-600 text-white text-[10px] px-1.5 py-0.5 rounded font-bold shadow-sm">
            {Math.round(width)} × {Math.round(height)}
        </div>
    );
}
