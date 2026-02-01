import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function SettingsModal() {
    const [savePath, setSavePath] = useState('Loading...');
    const [isLogEnabled, setIsLogEnabled] = useState(false);
    const [logStatus, setLogStatus] = useState('');

    useEffect(() => {
        // Initial load
        invoke<string>('get_save_path_setting').then(setSavePath).catch(() => setSavePath('.'));
        invoke<boolean>('is_log_enabled').then(setIsLogEnabled).catch(() => setIsLogEnabled(false));
    }, []);

    const handleUpdatePath = async () => {
        try {
            await invoke('set_save_path_setting', { path: savePath });
            setLogStatus('Save path updated!');
            setTimeout(() => setLogStatus(''), 3000);
        } catch (e) {
            setLogStatus('Error: ' + e);
        }
    };

    const handleToggleLogs = async (enabled: boolean) => {
        setIsLogEnabled(enabled);
        await invoke('set_log_enabled', { enabled });
    };

    const handleClearLogs = async () => {
        try {
            await invoke('clean_log');
            setLogStatus('Logs cleared');
            setTimeout(() => setLogStatus(''), 3000);
        } catch (e) {
            setLogStatus('Error clearing logs');
        }
    };

    const handleManualCapture = async () => {
        try {
            await invoke('trigger_capture');
        } catch (e) {
            setLogStatus('Capture trigger failed');
        }
    };

    return (
        <div className="p-6 bg-slate-900 text-slate-100 min-h-screen flex flex-col font-sans select-none">
            <h1 className="text-2xl font-bold mb-6 text-white border-b border-slate-700 pb-2">HyperLens Settings</h1>
            
            {/* Save Path Section */}
            <div className="mb-6">
                <label className="block text-sm font-medium text-slate-400 mb-2">Screenshot Save Path</label>
                <div className="flex gap-2">
                    <input 
                        className="flex-1 bg-slate-800 border border-slate-700 rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                        value={savePath}
                        onChange={(e) => setSavePath(e.target.value)}
                    />
                    <button 
                        onClick={handleUpdatePath}
                        className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-1.5 rounded text-sm font-medium transition"
                    >
                        Update
                    </button>
                </div>
                <p className="text-[10px] text-slate-500 mt-1 italic">Default "." creates a "Screenshots" folder next to the executable.</p>
            </div>

            {/* Diagnostics Section */}
            <div className="mb-6 bg-slate-800/50 p-4 rounded-lg border border-slate-700">
                <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wider">Diagnostics & Logs</h2>
                
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <span className="text-sm block">Enable Background Logging</span>
                        <span className="text-xs text-slate-500">Record system events for debugging</span>
                    </div>
                    <div 
                        onClick={() => handleToggleLogs(!isLogEnabled)}
                        className={`w-12 h-6 rounded-full cursor-pointer transition-colors relative ${isLogEnabled ? 'bg-green-600' : 'bg-slate-700'}`}
                    >
                        <div className={`absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform ${isLogEnabled ? 'translate-x-6' : ''}`}></div>
                    </div>
                </div>

                <div className="flex gap-2">
                    <button 
                        onClick={handleClearLogs}
                        className="flex-1 py-1.5 border border-slate-700 hover:bg-slate-700 text-slate-300 rounded text-xs transition"
                    >
                         Clear Logs
                    </button>
                    <button 
                        onClick={() => invoke('open_log_folder').catch(() => {})}
                        className="flex-1 py-1.5 border border-slate-700 hover:bg-slate-700 text-slate-300 rounded text-xs transition"
                    >
                         Show in Folder
                    </button>
                </div>
            </div>

            {/* Manual Actions */}
            <div className="mb-6">
                 <button 
                    onClick={handleManualCapture}
                    className="w-full py-2.5 bg-indigo-600 hover:bg-indigo-550 text-white rounded font-semibold text-sm shadow-lg transition transform active:scale-[0.98]"
                 >
                     Capture Screen Now
                 </button>
                 <p className="text-[11px] text-center text-slate-500 mt-2">Use this if the global hotkey (Ctrl + Shift + S) is blocked by another app.</p>
            </div>

            <div className="mt-auto pt-4 border-t border-slate-800 text-center">
                <div className="text-[10px] text-slate-600">HyperLens Engine v0.1.1 (Portable)</div>
                <div className="text-[10px] text-blue-500/50 mt-1">Status: {logStatus || 'Ready'}</div>
            </div>
        </div>
    );
}