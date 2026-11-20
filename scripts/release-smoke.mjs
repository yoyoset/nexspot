import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const durationMs = Number(process.env.NEXSPOT_SMOKE_DURATION_MS || 6000);
const evidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  evidenceIndex >= 0 ? process.argv[evidenceIndex + 1] || 'docs/13-SMOKE-EVIDENCE.json' : null;
const exeRelativePath = process.argv.includes('--exe')
  ? process.argv[process.argv.indexOf('--exe') + 1]
  : 'src-tauri/target/release/nexspot.exe';
const exePath = path.resolve(root, exeRelativePath);

const hashFile = (fullPath) => {
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(fullPath));
  return hash.digest('hex');
};

const quotePowerShellLiteral = (value) => `'${value.replaceAll("'", "''")}'`;

const fail = (message, evidence = {}) => {
  const payload = {
    schema: 'nexspot.release-smoke-evidence.v1',
    generatedAt: new Date().toISOString(),
    exe: {
      path: path.relative(root, exePath).replaceAll(path.sep, '/'),
      sha256: fs.existsSync(exePath) ? hashFile(exePath) : null,
    },
    durationMs,
    passed: false,
    message,
    ...evidence,
  };
  writeEvidence(payload);
  console.error(message);
  process.exit(1);
};

const writeEvidence = (payload) => {
  if (!evidenceRelativePath) {
    return;
  }
  const evidencePath = path.resolve(root, evidenceRelativePath);
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
};

if (!Number.isFinite(durationMs) || durationMs < 1000) {
  fail('NEXSPOT_SMOKE_DURATION_MS must be at least 1000.');
}

if (!fs.existsSync(exePath)) {
  fail(`Release EXE does not exist: ${exeRelativePath}`);
}

if (process.platform !== 'win32') {
  fail('NexSpot release smoke is only supported on Windows.');
}

const powershellScript = `
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class NexSpotSmokeWin32 {
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
}
"@
function Get-WindowTitle([IntPtr]$handle) {
  if ($handle -eq [IntPtr]::Zero) { return "" }
  $buffer = New-Object System.Text.StringBuilder 512
  [void][NexSpotSmokeWin32]::GetWindowText($handle, $buffer, $buffer.Capacity)
  return $buffer.ToString()
}
$exe = ${quotePowerShellLiteral(exePath)}
$durationMs = ${Math.trunc(durationMs)}
$existing = @(Get-Process -Name 'nexspot' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe } | ForEach-Object {
  [pscustomobject]@{
    id = $_.Id
    path = $_.Path
    startTime = $_.StartTime.ToString('o')
  }
})
if ($existing.Count -gt 0) {
  [pscustomobject]@{
    ok = $false
    reason = 'preexisting-process'
    existing = $existing
  } | ConvertTo-Json -Depth 5 -Compress
  exit 2
}

$process = Start-Process -FilePath $exe -PassThru
$deadline = [DateTime]::UtcNow.AddMilliseconds($durationMs)
$mainWindowSeen = $false
$mainWindowHandle = 0
$mainWindowTitle = ''
do {
  Start-Sleep -Milliseconds 250
  $process.Refresh()
  if ($process.HasExited) {
    break
  }
  if ($process.MainWindowHandle -ne 0) {
    $mainWindowSeen = $true
    $mainWindowHandle = [int64]$process.MainWindowHandle
    $mainWindowTitle = Get-WindowTitle $process.MainWindowHandle
    if (-not $mainWindowTitle) {
      $mainWindowTitle = $process.MainWindowTitle
    }
  }
} while ([DateTime]::UtcNow -lt $deadline)

$remainingMs = [int]([Math]::Max(0, ($deadline - [DateTime]::UtcNow).TotalMilliseconds))
if ($remainingMs -gt 0) {
  Start-Sleep -Milliseconds $remainingMs
}
$process.Refresh()
$mainWindowTitle = if ($mainWindowHandle -ne 0) { Get-WindowTitle ([IntPtr]$mainWindowHandle) } else { $mainWindowTitle }
if (-not $mainWindowTitle) {
  $mainWindowTitle = $process.MainWindowTitle
}
$alive = -not $process.HasExited
$exitCode = $null
$reason = 'exited-early'
if ($alive -and $mainWindowSeen) {
  $reason = 'alive-after-duration'
} elseif ($alive) {
  $reason = 'no-main-window'
}
if ($alive) {
  Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  Wait-Process -Id $process.Id -Timeout 5 -ErrorAction SilentlyContinue
} else {
  $exitCode = $process.ExitCode
}
[pscustomobject]@{
  ok = $alive
  reason = $reason
  pid = $process.Id
  durationMs = $durationMs
  mainWindowSeen = $mainWindowSeen
  mainWindowHandle = $mainWindowHandle
  mainWindowTitle = $mainWindowTitle
  exitCode = $exitCode
} | ConvertTo-Json -Depth 5 -Compress
if ($alive -and $mainWindowSeen) { exit 0 } else { exit 1 }
`;

const result = spawnSync(
  'powershell.exe',
  ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', powershellScript],
  {
    encoding: 'utf8',
    windowsHide: true,
  },
);

if (result.error) {
  fail(`Unable to run smoke launch: ${result.error.message}`);
}

let parsed = null;
try {
  parsed = JSON.parse(result.stdout.trim());
} catch {
  fail('Unable to parse smoke launch output.', {
    stdout: result.stdout.trim(),
    stderr: result.stderr.trim(),
  });
}

const payload = {
  schema: 'nexspot.release-smoke-evidence.v1',
  generatedAt: new Date().toISOString(),
  exe: {
    path: path.relative(root, exePath).replaceAll(path.sep, '/'),
    sha256: hashFile(exePath),
  },
  durationMs,
  passed: Boolean(parsed.ok),
  launch: parsed,
};

writeEvidence(payload);

if (!parsed.ok || result.status !== 0) {
  console.error(`Release smoke failed: ${parsed.reason || 'unknown'}`);
  process.exit(result.status || 1);
}

console.log('NexSpot release smoke passed');
console.log(`exe: ${payload.exe.path}`);
console.log(`sha256: ${payload.exe.sha256}`);
console.log(`durationMs: ${durationMs}`);
if (evidenceRelativePath) {
  console.log(`smoke evidence written: ${evidenceRelativePath}`);
}
