import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const now = new Date();
const stamp = now.toISOString().replace(/[:.]/g, '-');
const defaultOutDir = path.join('docs', 'qa-runs', stamp);
const outDirIndex = process.argv.indexOf('--out-dir');
const outDir = outDirIndex >= 0 ? process.argv[outDirIndex + 1] || defaultOutDir : defaultOutDir;
const outFullDir = path.join(root, outDir);
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => parseJsonText(fs.readFileSync(path.join(root, relativePath), 'utf8'));
const readOptionalJson = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return null;
  }
  return parseJsonText(fs.readFileSync(fullPath, 'utf8'));
};

const runPowerShellJson = (script) => {
  if (process.platform !== 'win32') {
    return { skipped: true, reason: 'not-win32' };
  }

  const command = [
    '[Console]::OutputEncoding = [System.Text.Encoding]::UTF8',
    '$ErrorActionPreference = "SilentlyContinue"',
    script,
  ].join('; ');
  const result = spawnSync(
    'powershell.exe',
    ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command],
    { encoding: 'utf8', windowsHide: true },
  );

  if (result.error || result.status !== 0) {
    return {
      error: result.error?.message || result.stderr?.trim() || `powershell exited ${result.status}`,
      stdout: result.stdout?.trim() || '',
    };
  }

  const output = result.stdout.trim();
  if (!output) {
    return null;
  }
  try {
    return JSON.parse(output);
  } catch {
    return { parseError: true, raw: output };
  }
};

const collectWindowsEnvironment = () => {
  const windows = runPowerShellJson(`
    $os = Get-CimInstance Win32_OperatingSystem
    $computer = Get-CimInstance Win32_ComputerSystem
    $video = @(Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion, CurrentHorizontalResolution, CurrentVerticalResolution, CurrentBitsPerPixel)
    $monitors = @(Get-CimInstance Win32_DesktopMonitor | Select-Object Name, ScreenWidth, ScreenHeight, PNPDeviceID)
    $dpi = (Get-ItemProperty -Path "HKCU:\\Control Panel\\Desktop\\WindowMetrics" -Name AppliedDPI).AppliedDPI
    [pscustomobject]@{
      osCaption = $os.Caption
      osVersion = $os.Version
      osBuildNumber = $os.BuildNumber
      architecture = $os.OSArchitecture
      manufacturer = $computer.Manufacturer
      model = $computer.Model
      totalPhysicalMemory = $computer.TotalPhysicalMemory
      logicalProcessors = $computer.NumberOfLogicalProcessors
      appliedDpi = $dpi
      videoControllers = $video
      desktopMonitors = $monitors
    } | ConvertTo-Json -Depth 6 -Compress
  `);

  return {
    node: {
      platform: os.platform(),
      release: os.release(),
      arch: os.arch(),
      cpuModels: [...new Set(os.cpus().map((cpu) => cpu.model))],
      logicalCpuCount: os.cpus().length,
      totalMemory: os.totalmem(),
      freeMemory: os.freemem(),
    },
    windows,
  };
};

const collectArtifactSignatures = () =>
  runPowerShellJson(`
    $paths = @(
      "src-tauri\\target\\release\\nexspot.exe",
      "src-tauri\\target\\release\\bundle\\nsis\\NexSpot_0.3.0_x64-setup.exe"
    )
    $items = foreach ($p in $paths) {
      if (Test-Path $p) {
        $sig = Get-AuthenticodeSignature -LiteralPath $p
        $item = Get-Item -LiteralPath $p
        [pscustomobject]@{
          path = $p
          bytes = $item.Length
          modifiedAt = $item.LastWriteTime.ToString("o")
          status = $sig.Status.ToString()
          subject = if ($sig.SignerCertificate) { $sig.SignerCertificate.Subject } else { "" }
        }
      }
    }
    @($items) | ConvertTo-Json -Depth 5 -Compress
  `);

const gate = readOptionalJson('docs/17-MANUAL-QA-GATE.json');
const releaseEvidence = readOptionalJson('docs/11-RELEASE-EVIDENCE.json');
const packageJson = readJson('package.json');
const environment = collectWindowsEnvironment();
const signatures = collectArtifactSignatures();

const pendingRequired = gate?.matrix?.pendingRequired || [];
const openP0 = gate?.issueLog?.openP0 || [];
const openP1 = gate?.issueLog?.openP1 || [];
const hashes = gate?.release?.hashes || releaseEvidence?.artifacts || {};

const report = {
  schema: 'nexspot.qa-evidence-pack.v1',
  generatedAt: now.toISOString(),
  productName: 'NexSpot',
  version: packageJson.version,
  outDir,
  artifacts: {
    exe: {
      path: 'src-tauri/target/release/nexspot.exe',
      sha256: hashes.exe || releaseEvidence?.artifacts?.exe?.sha256 || '',
    },
    nsisInstaller: {
      path: 'src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe',
      sha256: hashes.nsisInstaller || releaseEvidence?.artifacts?.nsisInstaller?.sha256 || '',
    },
  },
  decisions: gate?.decisions || {},
  pendingRequired,
  openP0,
  openP1,
  environment,
  signatures: Array.isArray(signatures)
    ? signatures.map((signature) => ({
        path: signature.path,
        bytes: signature.bytes,
        modifiedAt: signature.modifiedAt,
        status: signature.status,
        statusMessage: signature.status === 'NotSigned' ? 'Artifact is not Authenticode signed.' : '',
        subject: signature.subject,
      }))
    : signatures,
  requiredAttachments: [
    'Saved normal screenshot image',
    'Copied-image paste target screenshot or note',
    'OCR result text export when applicable',
    'Scrolling capture output image',
    'Scrolling preview metadata screenshot or copied metadata',
    'Windows warning prompt screenshot for unsigned installer',
  ],
};

const resultsTemplate = {
  schema: 'nexspot.qa-results.v1',
  generatedAt: now.toISOString(),
  evidencePack: path.join(outDir, 'qa-evidence-pack.json').replaceAll(path.sep, '/'),
  artifacts: report.artifacts,
  rows: pendingRequired.map((id) => ({
    id,
    status: 'Pending',
    evidence: '',
    notes: '',
  })),
  failedIssueRowsSchema: {
    id: 'QA-009',
    severity: 'P0',
    area: 'Short issue area',
    relatedMatrixIds: 'SCR-01',
    status: 'Open',
    owner: 'QA + Engineering',
    closeCondition: 'Describe the exact evidence required to close this issue.',
  },
  failedIssueRows: [],
};

const markdown = `# NexSpot QA Evidence Pack

Generated: ${report.generatedAt}

## Release Under Test

| Field | Value |
|---|---|
| Version | ${report.version} |
| EXE | \`${report.artifacts.exe.path}\` |
| EXE SHA256 | \`${report.artifacts.exe.sha256}\` |
| Installer | \`${report.artifacts.nsisInstaller.path}\` |
| Installer SHA256 | \`${report.artifacts.nsisInstaller.sha256}\` |
| Internal trial | ${report.decisions.internalTrial || 'Unknown'} |
| Small commercial pilot | ${report.decisions.smallCommercialPilot || 'Unknown'} |
| Public stable release | ${report.decisions.publicStableRelease || 'Unknown'} |

## Environment Snapshot

Review \`environment.json\` for OS, DPI, monitor, and GPU details. Fill any missing display/DPI details manually below.

| Field | Value |
|---|---|
| Tester |  |
| Machine label |  |
| Windows edition/build |  |
| Display count |  |
| DPI / scale per display |  |
| GPU / driver |  |

## Required Pending IDs

${pendingRequired.map((id) => `- [ ] ${id}`).join('\n') || '- None'}

## Open P0 Blockers

${openP0.map((id) => `- [ ] ${id}`).join('\n') || '- None'}

## Open P1 Polish Blockers

${openP1.map((id) => `- [ ] ${id}`).join('\n') || '- None'}

## Evidence Checklist

${report.requiredAttachments.map((item) => `- [ ] ${item}`).join('\n')}

Evidence paths must be repository-relative files that already exist when merged. Use semicolons to list multiple attachments for one row.

## Result Rows To Paste Back

| ID | Status | Evidence path | Notes |
|---|---|---|---|
${pendingRequired.map((id) => `| ${id} |  |  |  |`).join('\n')}

Update \`results-template.json\` with the helper, then run:

\`\`\`powershell
node scripts\\qa-result-update.mjs --results ${path.join(outDir, 'results-template.json')} --id SCR-01 --status Passed --evidence docs/qa-runs/latest/scr-01.png --notes "Chrome page captured"
node scripts\\qa-merge-results.mjs --results ${path.join(outDir, 'results-template.json')}
node scripts\\qa-merge-results.mjs --results ${path.join(outDir, 'results-template.json')} --apply
\`\`\`

## Failure Issue Row Template

| ID | Severity | Scenario ID | Environment ID | Summary | Owner | Status | Fixed in |
|---|---|---|---|---|---|---|---|
| QA-NEW | P0 |  |  |  | QA + Engineering | Open |  |

For any \`Failed\` or \`Blocked\` result row, pass \`--issue-id\`, \`--severity\`, \`--area\`, \`--owner\`, and \`--close-condition\` to \`qa-result-update.mjs\`. The merge script updates \`docs/16-COMMERCIAL-QA-ISSUE-LOG.md\` during \`--apply\`.
`;

fs.mkdirSync(outFullDir, { recursive: true });
fs.writeFileSync(path.join(outFullDir, 'qa-evidence-pack.json'), `${JSON.stringify(report, null, 2)}\n`, 'utf8');
fs.writeFileSync(path.join(outFullDir, 'environment.json'), `${JSON.stringify(environment, null, 2)}\n`, 'utf8');
fs.writeFileSync(path.join(outFullDir, 'results-template.json'), `${JSON.stringify(resultsTemplate, null, 2)}\n`, 'utf8');
fs.writeFileSync(path.join(outFullDir, 'qa-run.md'), markdown, 'utf8');

console.log('NexSpot QA evidence pack');
console.log('='.repeat(25));
console.log(`output: ${outDir}`);
console.log(`pending required IDs: ${pendingRequired.length}`);
console.log(`open P0 blockers: ${openP0.length}`);
console.log(`open P1 polish blockers: ${openP1.length}`);
console.log(`public stable release: ${report.decisions.publicStableRelease || 'Unknown'}`);
