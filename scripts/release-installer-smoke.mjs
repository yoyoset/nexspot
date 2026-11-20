import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const execute = process.argv.includes('--execute');
const evidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  evidenceIndex >= 0 ? process.argv[evidenceIndex + 1] || 'docs/14-INSTALLER-SMOKE-EVIDENCE.json' : null;
const packageJson = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const version = packageJson.version;
const installerRelativePath = `src-tauri/target/release/bundle/nsis/NexSpot_${version}_x64-setup.exe`;
const installerPath = path.resolve(root, installerRelativePath);
const installDir = process.env.NEXSPOT_INSTALLER_SMOKE_DIR || 'C:\\tmp\\nexspot-installer-smoke';
const installTimeoutSec = Number(process.env.NEXSPOT_INSTALLER_TIMEOUT_SEC || 120);
const launchDurationMs = Number(process.env.NEXSPOT_INSTALLER_LAUNCH_DURATION_MS || 6000);

const hashFile = (fullPath) => {
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(fullPath));
  return hash.digest('hex');
};

const quotePowerShellLiteral = (value) => `'${value.replaceAll("'", "''")}'`;

const writeEvidence = (payload) => {
  if (!evidenceRelativePath) {
    return;
  }
  const evidencePath = path.resolve(root, evidenceRelativePath);
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
};

const fail = (message, evidence = {}) => {
  const payload = {
    schema: 'nexspot.installer-smoke-evidence.v1',
    generatedAt: new Date().toISOString(),
    installer: {
      path: installerRelativePath,
      sha256: fs.existsSync(installerPath) ? hashFile(installerPath) : null,
    },
    installDir,
    executed: execute,
    passed: false,
    message,
    ...evidence,
  };
  writeEvidence(payload);
  console.error(message);
  process.exit(1);
};

if (!fs.existsSync(installerPath)) {
  fail(`NSIS installer does not exist: ${installerRelativePath}`);
}

if (process.platform !== 'win32') {
  fail('NexSpot installer smoke is only supported on Windows.');
}

if (!execute) {
  fail('Refusing to run installer smoke without --execute. This test installs and uninstalls NexSpot in a temporary directory.', {
    dryRun: true,
  });
}

if (!Number.isFinite(installTimeoutSec) || installTimeoutSec < 30) {
  fail('NEXSPOT_INSTALLER_TIMEOUT_SEC must be at least 30.');
}

if (!Number.isFinite(launchDurationMs) || launchDurationMs < 1000) {
  fail('NEXSPOT_INSTALLER_LAUNCH_DURATION_MS must be at least 1000.');
}

const powershellScript = `
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$installer = ${quotePowerShellLiteral(installerPath)}
$installDir = ${quotePowerShellLiteral(installDir)}
$safeRoot = [System.IO.Path]::GetFullPath('C:\\tmp\\nexspot-installer-smoke')
$resolvedInstallDir = [System.IO.Path]::GetFullPath($installDir)
if (-not $resolvedInstallDir.StartsWith($safeRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Unsafe install dir: $resolvedInstallDir"
}

function Run-ProcessChecked([string]$file, [string[]]$arguments, [int]$timeoutSec, [string]$stage) {
  $p = Start-Process -FilePath $file -ArgumentList $arguments -WindowStyle Hidden -PassThru
  $deadline = [DateTime]::UtcNow.AddSeconds($timeoutSec)
  do {
    Start-Sleep -Milliseconds 250
    $p.Refresh()
    if ($p.HasExited) {
      break
    }
  } while ([DateTime]::UtcNow -lt $deadline)
  $p.Refresh()
  if (-not $p.HasExited) {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    throw "$stage timed out after $timeoutSec seconds"
  }
  $p.Refresh()
  if ($p.ExitCode -ne 0) {
    throw "$stage exited with code $($p.ExitCode)"
  }
  return [pscustomobject]@{
    stage = $stage
    pid = $p.Id
    exitCode = $p.ExitCode
  }
}

function Find-InstalledExe([string]$dir) {
  $candidates = @(
    (Join-Path $dir 'NexSpot.exe'),
    (Join-Path $dir 'nexspot.exe')
  )
  foreach ($candidate in $candidates) {
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
      return $candidate
    }
  }
  return $null
}

function Wait-InstalledExe([string]$dir, [int]$timeoutSec, [string]$stage) {
  $deadline = [DateTime]::UtcNow.AddSeconds($timeoutSec)
  do {
    $candidate = Find-InstalledExe $dir
    if ($candidate) {
      return $candidate
    }
    Start-Sleep -Milliseconds 250
  } while ([DateTime]::UtcNow -lt $deadline)
  throw "Installed EXE not found after $stage in $dir"
}

function Test-MainWindow([string]$exe, [int]$durationMs) {
  $p = Start-Process -FilePath $exe -PassThru
  $deadline = [DateTime]::UtcNow.AddMilliseconds($durationMs)
  $mainWindowSeen = $false
  $mainWindowHandle = 0
  $mainWindowTitle = ''
  do {
    Start-Sleep -Milliseconds 250
    $p.Refresh()
    if ($p.HasExited) {
      break
    }
    if ($p.MainWindowHandle -ne 0) {
      $mainWindowSeen = $true
      $mainWindowHandle = [int64]$p.MainWindowHandle
      $mainWindowTitle = $p.MainWindowTitle
    }
  } while ([DateTime]::UtcNow -lt $deadline)
  $remainingMs = [int]([Math]::Max(0, ($deadline - [DateTime]::UtcNow).TotalMilliseconds))
  if ($remainingMs -gt 0) {
    Start-Sleep -Milliseconds $remainingMs
  }
  $p.Refresh()
  $alive = -not $p.HasExited
  $exitCode = $null
  if ($alive) {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Wait-Process -Id $p.Id -Timeout 5 -ErrorAction SilentlyContinue
  } else {
    $exitCode = $p.ExitCode
  }
  return [pscustomobject]@{
    ok = ($alive -and $mainWindowSeen -and $mainWindowTitle -eq 'NexSpot')
    pid = $p.Id
    aliveAfterDuration = $alive
    mainWindowSeen = $mainWindowSeen
    mainWindowHandle = $mainWindowHandle
    mainWindowTitle = $mainWindowTitle
    exitCode = $exitCode
  }
}

function Get-InstalledExeEvidence([string]$exe) {
  $item = Get-Item -LiteralPath $exe
  $v = $item.VersionInfo
  return [pscustomobject]@{
    path = $exe
    bytes = [int64]$item.Length
    sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $exe).Hash.ToLowerInvariant()
    versionInfo = [pscustomobject]@{
      fileVersion = [string]$v.FileVersion
      productVersion = [string]$v.ProductVersion
      productName = [string]$v.ProductName
      fileDescription = [string]$v.FileDescription
      originalFilename = [string]$v.OriginalFilename
    }
  }
}

function Invoke-Cleanup([string]$dir) {
  $uninstaller = Join-Path $dir 'uninstall.exe'
  $result = $null
  if (Test-Path -LiteralPath $uninstaller -PathType Leaf) {
    $result = Run-ProcessChecked $uninstaller @('/S') ${Math.trunc(installTimeoutSec)} 'uninstall'
  }
  if (Test-Path -LiteralPath $dir) {
    Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue
  }
  return $result
}

$cleanupBefore = $false
if (Test-Path -LiteralPath $resolvedInstallDir) {
  Remove-Item -LiteralPath $resolvedInstallDir -Recurse -Force
  $cleanupBefore = $true
}

$install = $null
$overwrite = $null
$uninstall = $null
$launch = $null
$installedExe = $null
$freshInstalledExeEvidence = $null
$installedExeEvidence = $null
$finalAppExeRemoved = $false
$installDirRemoved = $false

try {
  $install = Run-ProcessChecked $installer @('/S', "/D=$resolvedInstallDir") ${Math.trunc(installTimeoutSec)} 'fresh-install'
  $installedExe = Wait-InstalledExe $resolvedInstallDir ${Math.min(Math.trunc(installTimeoutSec), 30)} 'fresh-install'
  $freshInstalledExeEvidence = Get-InstalledExeEvidence $installedExe

  $overwrite = Run-ProcessChecked $installer @('/S', "/D=$resolvedInstallDir") ${Math.trunc(installTimeoutSec)} 'overwrite-install'
  $installedExeAfterOverwrite = Wait-InstalledExe $resolvedInstallDir ${Math.min(Math.trunc(installTimeoutSec), 30)} 'overwrite-install'
  $installedExeEvidence = Get-InstalledExeEvidence $installedExeAfterOverwrite

  $launch = Test-MainWindow $installedExeAfterOverwrite ${Math.trunc(launchDurationMs)}
  if (-not $launch.ok) {
    throw "Installed EXE launch failed main-window smoke"
  }

  $uninstall = Invoke-Cleanup $resolvedInstallDir
  $finalAppExeRemoved = -not (Test-Path -LiteralPath $installedExeAfterOverwrite)
  $installDirRemoved = -not (Test-Path -LiteralPath $resolvedInstallDir)

  [pscustomobject]@{
    ok = $true
    installDir = $resolvedInstallDir
    cleanupBefore = $cleanupBefore
    freshInstall = $install
    overwriteInstall = $overwrite
    installedExe = $installedExeAfterOverwrite
    freshInstalledExeSha256 = $freshInstalledExeEvidence.sha256
    installedExeSha256 = $installedExeEvidence.sha256
    installedExeBytes = $installedExeEvidence.bytes
    installedExeVersionInfo = $installedExeEvidence.versionInfo
    launch = $launch
    uninstall = $uninstall
    finalAppExeRemoved = $finalAppExeRemoved
    installDirRemoved = $installDirRemoved
  } | ConvertTo-Json -Depth 8 -Compress
  exit 0
} catch {
  $message = $_.Exception.Message
  try {
    $uninstall = Invoke-Cleanup $resolvedInstallDir
  } catch {}
  [pscustomobject]@{
    ok = $false
    message = $message
    installDir = $resolvedInstallDir
    cleanupBefore = $cleanupBefore
    freshInstall = $install
    overwriteInstall = $overwrite
    installedExe = $installedExe
    freshInstalledExeSha256 = if ($freshInstalledExeEvidence) { $freshInstalledExeEvidence.sha256 } else { $null }
    installedExeSha256 = if ($installedExeEvidence) { $installedExeEvidence.sha256 } else { $null }
    installedExeBytes = if ($installedExeEvidence) { $installedExeEvidence.bytes } else { $null }
    installedExeVersionInfo = if ($installedExeEvidence) { $installedExeEvidence.versionInfo } else { $null }
    launch = $launch
    uninstall = $uninstall
    finalAppExeRemoved = if ($installedExe) { -not (Test-Path -LiteralPath $installedExe) } else { $false }
    installDirRemoved = -not (Test-Path -LiteralPath $resolvedInstallDir)
  } | ConvertTo-Json -Depth 8 -Compress
  exit 1
}
`;

const result = spawnSync(
  'powershell.exe',
  ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', powershellScript],
  {
    encoding: 'utf8',
    windowsHide: true,
    timeout: Math.max(installTimeoutSec * 3 * 1000, 180000),
  },
);

if (result.error) {
  fail(`Unable to run installer smoke: ${result.error.message}`, {
    stdout: result.stdout?.trim() || '',
    stderr: result.stderr?.trim() || '',
  });
}

let parsed = null;
try {
  parsed = JSON.parse(result.stdout.trim());
} catch {
  fail('Unable to parse installer smoke output.', {
    stdout: result.stdout.trim(),
    stderr: result.stderr.trim(),
  });
}

const payload = {
  schema: 'nexspot.installer-smoke-evidence.v1',
  generatedAt: new Date().toISOString(),
  installer: {
    path: installerRelativePath,
    sha256: hashFile(installerPath),
  },
  installDir,
  executed: true,
  passed: Boolean(parsed.ok),
  result: parsed,
};

writeEvidence(payload);

if (!parsed.ok || result.status !== 0) {
  console.error(`Installer smoke failed: ${parsed.message || 'unknown'}`);
  process.exit(result.status || 1);
}

console.log('NexSpot installer smoke passed');
console.log(`installer: ${payload.installer.path}`);
console.log(`sha256: ${payload.installer.sha256}`);
console.log(`installDir: ${installDir}`);
if (evidenceRelativePath) {
  console.log(`installer smoke evidence written: ${evidenceRelativePath}`);
}
