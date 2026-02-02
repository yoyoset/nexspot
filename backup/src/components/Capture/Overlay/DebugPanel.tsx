interface DebugPanelProps {
    isReady: boolean;
    hasSelection: boolean;
    selectionInfo?: string; // Optional detailed info
}

export default function DebugPanel({ isReady, hasSelection, selectionInfo }: DebugPanelProps) {
    return (
        <>
            {/* Green Overlay for Visibility Check */}
            <div className="absolute inset-0 bg-green-500/30 pointer-events-none" />

            {/* Info Box */}
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 p-6 bg-blue-600/90 text-white z-[9999] pointer-events-none text-2xl font-bold border-4 border-white shadow-2xl">
                [WEB VIEW ALIVE]
                <br />
                Ready: {isReady.toString()}
                <br />
                Sel: {hasSelection ? 'YES' : 'NO'}
                <br />
                <span className="text-sm font-normal opacity-80">{selectionInfo}</span>
            </div>
        </>
    );
}
