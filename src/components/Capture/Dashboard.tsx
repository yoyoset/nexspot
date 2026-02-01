import { invoke } from '@tauri-apps/api/core';

export default function Dashboard() {
    const handleCapture = async () => {
        try {
            await invoke('trigger_capture');
        } catch (e) {
            console.error('Manual capture failed:', e);
        }
    };

    const handleOpenSettings = async () => {
        await invoke('open_settings');
    };

    return (
        <div className="min-h-screen bg-slate-950 flex flex-col items-center justify-center p-8 text-slate-200">
            {/* Logo/Header */}
            <div className="mb-12 text-center">
                <div className="w-20 h-20 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-3xl mx-auto mb-6 shadow-[0_0_40px_-10px_rgba(59,130,246,0.5)] flex items-center justify-center text-4xl">
                    📸
                </div>
                <h1 className="text-4xl font-black text-white tracking-tight mb-2">HyperLens</h1>
                <p className="text-slate-400 text-sm font-medium">Professional Screenshot Engine</p>
            </div>

            {/* Quick Action Button */}
            <button
                onClick={handleCapture}
                className="group relative bg-white text-slate-950 hover:bg-blue-50 transition-all duration-300 px-10 py-5 rounded-2xl font-black text-xl shadow-[0_20px_40px_-15px_rgba(255,255,255,0.3)] hover:shadow-[0_25px_50px_-12px_rgba(59,130,246,0.4)] active:scale-95 mb-8"
            >
                Start Capture
                <span className="block text-[10px] text-slate-500 font-bold opacity-60 mt-1 uppercase tracking-widest">Ctrl + Shift + S</span>
            </button>

            {/* Mini Actions */}
            <div className="flex gap-4">
                <button
                    onClick={handleOpenSettings}
                    className="flex items-center gap-2 bg-slate-900 border border-slate-800 hover:border-slate-600 px-5 py-2.5 rounded-xl text-sm font-semibold transition"
                >
                    ⚙️ Settings
                </button>
            </div>

            {/* Status Info */}
            <div className="mt-16 grid grid-cols-2 gap-8 max-w-sm w-full">
                <div className="bg-slate-900/30 p-4 rounded-2xl border border-slate-800/50">
                    <div className="text-[10px] text-slate-500 font-bold uppercase mb-1">Status</div>
                    <div className="text-xs text-green-400 flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 bg-green-400 rounded-full animate-pulse"></span>
                        Service Running
                    </div>
                </div>
                <div className="bg-slate-900/30 p-4 rounded-2xl border border-slate-800/50">
                    <div className="text-[10px] text-slate-500 font-bold uppercase mb-1">Build</div>
                    <div className="text-xs text-slate-400">v0.1.1 Production</div>
                </div>
            </div>

            <p className="mt-auto text-[10px] text-slate-600 uppercase tracking-widest font-bold">
                HyperLens Engine • High Performance Capture
            </p>
        </div>
    );
}
