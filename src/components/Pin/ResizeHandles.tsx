import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

// Map directions to Tauri expected values (usually specific Enum or string)
type ResizeDirection =
    | "East"
    | "North"
    | "NorthEast"
    | "NorthWest"
    | "South"
    | "SouthEast"
    | "SouthWest"
    | "West";

const ResizeHandle: React.FC<{
    direction: ResizeDirection;
    className: string;
    cursor: string;
}> = ({ direction, className, cursor }) => {
    const handleMouseDown = async (e: React.MouseEvent) => {
        // e.preventDefault(); // Don't prevent default, might block tauri?
        try {
            // @ts-ignore - startResize might not be in definition check
            await (getCurrentWindow() as any).startResize(direction);
        } catch (err) {
            console.error("Failed to start resize:", err);
        }
    };

    return (
        <div
            className={`absolute ${className} z-50 hover:bg-white/10 transition-colors`}
            style={{ cursor }}
            onMouseDown={handleMouseDown}
        />
    );
};

const ResizeHandles: React.FC = () => {
    // Thickness of the resize areas
    const T = "w-1.5 h-full top-0";
    const H = "h-1.5 w-full left-0";
    const C = "w-3 h-3";

    return (
        <>
            {/* Edges */}
            <ResizeHandle direction="North" className={`${H} top-0`} cursor="n-resize" />
            <ResizeHandle direction="South" className={`${H} bottom-0`} cursor="s-resize" />
            <ResizeHandle direction="West" className={`${T} left-0`} cursor="w-resize" />
            <ResizeHandle direction="East" className={`${T} right-0`} cursor="e-resize" />

            {/* Corners */}
            <ResizeHandle direction="NorthWest" className={`${C} top-0 left-0`} cursor="nw-resize" />
            <ResizeHandle direction="NorthEast" className={`${C} top-0 right-0`} cursor="ne-resize" />
            <ResizeHandle direction="SouthWest" className={`${C} bottom-0 left-0`} cursor="sw-resize" />
            <ResizeHandle direction="SouthEast" className={`${C} bottom-0 right-0`} cursor="se-resize" />
        </>
    );
};

export default ResizeHandles;
