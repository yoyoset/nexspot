// P9: Pure Native Logic means this React component is now ONLY for Post-Capture Results.
// The Selection/Drawing/Toolbar is all Native Window (GDI+).

export default function SelectionOverlay() {
    // Left empty for now, or could show "Processing..." if we decide to show this window during AI analysis.
    // For P9, the Native Overlay handles everything until "Save".
    // "Save" triggers `capture-save` event which might open a Result Window.

    // We keep this component mounted as part of the React Tree but transparent/inert during capture.

    return (
        <div
            className="fixed inset-0 w-full h-full pointer-events-none"
            style={{ background: 'transparent' }}
        >
            {/* P9: Native Overlay is Active. This layer is passive. */}
        </div>
    );
}
