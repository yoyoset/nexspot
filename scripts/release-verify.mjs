import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const failures = [];
const warnings = [];
const writeEvidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  writeEvidenceIndex >= 0 ? process.argv[writeEvidenceIndex + 1] || 'docs/11-RELEASE-EVIDENCE.json' : null;
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  return parseJsonText(fs.readFileSync(fullPath, 'utf8'));
};

const readOptionalJson = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return null;
  }
  return parseJsonText(fs.readFileSync(fullPath, 'utf8'));
};

const readOptionalText = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return '';
  }
  return fs.readFileSync(fullPath, 'utf8');
};

const collectCommercialSourceGate = () => {
  const scrollingRust = readOptionalText('src-tauri/src/service/native_overlay/scrolling.rs');
  const stitchRust = readOptionalText('src-tauri/src/service/stitch.rs');
  const ocrRust = readOptionalText('src-tauri/src/service/ocr.rs');
  const notificationRust = readOptionalText('src-tauri/src/service/notification.rs');
  const overlayCommands = readOptionalText('src-tauri/src/service/native_overlay/commands.rs');
  const toolbarStyle = readOptionalText('src-tauri/src/service/native_overlay/render/toolbar/style.rs');
  const toolbarConstants = readOptionalText('src-tauri/src/service/native_overlay/render/toolbar/constants.rs');
  const toolbarMod = readOptionalText('src-tauri/src/service/native_overlay/render/toolbar/mod.rs');
  const toolbarRender = readOptionalText('src-tauri/src/service/native_overlay/render/toolbar/render.rs');
  const velloToolbar = readOptionalText('src-tauri/src/service/native_overlay/render/vello_engine/renderer/ui/toolbar.rs');
  const velloToolIcons = readOptionalText('src-tauri/src/service/native_overlay/render/vello_engine/renderer/ui/icons/tools.rs');
  const velloActionIcons = readOptionalText('src-tauri/src/service/native_overlay/render/vello_engine/renderer/ui/icons/actions.rs');
  const eventListener = readOptionalText('src/components/Overlay/TauriEventListener.tsx');
  const preview = readOptionalText('src/components/Overlay/ScrollingPreview.tsx');
  const ocrResultWindow = readOptionalText('src/components/Overlay/OCRResultWindow.tsx');
  const appTsx = readOptionalText('src/App.tsx');
  const pinCollectionWindow = readOptionalText('src/components/Pin/PinCollectionWindow.tsx');
  const enLocale = readOptionalText('src/locales/en.json');
  const zhLocale = readOptionalText('src/locales/zh.json');

  return {
    scrollingStartedReportsTargetAndSettings:
      scrollingRust.includes('"targetWindow"') &&
      scrollingRust.includes('"selection"') &&
      scrollingRust.includes('scrolling_runtime_settings()'),
    scrollingProgressReportsStopChecks:
      scrollingRust.includes('"phase": "checking_stop"') &&
      scrollingRust.includes('"stagnantFrames"') &&
      scrollingRust.includes('NO_GROWTH_LIMIT'),
    scrollingResultCarriesDiagnosticSettings:
      scrollingRust.includes('max_capture_frames') &&
      scrollingRust.includes('no_growth_limit') &&
      scrollingRust.includes('wheel_step') &&
      scrollingRust.includes('stitch_reports') &&
      scrollingRust.includes('"stitchReport"'),
    scrollingStitcherRejectsTinyOrAmbiguousMatches:
      stitchRust.includes('MIN_APPEND_PER_FRAME_PX') &&
      stitchRust.includes('AMBIGUOUS_MATCH_PERCENT') &&
      stitchRust.includes('nearby_duplicate') &&
      stitchRust.includes('add_frame_rejects_ambiguous_repeating_content'),
    scrollingStitcherUsesSeededPerformancePath:
      stitchRust.includes('last_added_px') &&
      stitchRust.includes('seeded_match_y') &&
      stitchRust.includes('candidate_count') &&
      stitchRust.includes('match_duration_ms') &&
      stitchRust.includes('add_frame_uses_previous_scroll_delta_as_seed'),
    ocrLiveCaptureConfirmsOverlayHide:
      ocrRust.includes('overlay_hide_confirmed') &&
      ocrRust.includes('overlay_hide_status') &&
      ocrRust.includes('is_window_visible') &&
      ocrRust.includes('capture_live_screen_rect(selection_rect)'),
    ocrFailureKeepsDiagnosticMetadata:
      ocrRust.includes('status: "error"') &&
      ocrRust.includes('status: "no_text"') &&
      ocrRust.includes('metadata: Some(metadata())') &&
      ocrRust.includes('open_ocr_result_window'),
    captureFailuresEmitStructuredFrontendErrors:
      notificationRust.includes('CaptureErrorEvent') &&
      notificationRust.includes('"capture://error"') &&
      overlayCommands.includes('emit_capture_error') &&
      overlayCommands.includes('"scrolling"') &&
      overlayCommands.includes('"ocr"') &&
      eventListener.includes("'capture://error'") &&
      eventListener.includes('capture_failed_detail'),
    toolbarUsesSharedCommercialStyleTokens:
      toolbarStyle.includes('COLOR_ACCENT') &&
      toolbarStyle.includes('ICON_STROKE_VELLO') &&
      toolbarStyle.includes('ICON_FONT_SIZE_GDI') &&
      toolbarStyle.includes('BUTTON_SIZE') &&
      toolbarConstants.includes('style::rgba_to_gdi') &&
      toolbarMod.includes('style::BUTTON_SIZE') &&
      toolbarRender.includes('style::ICON_FONT_SIZE_GDI'),
    velloToolbarUsesSharedStyleTokens:
      velloToolbar.includes('style::rgba_to_vello(style::COLOR_BG_MAIN)') &&
      velloToolbar.includes('style::RADIUS_CONTAINER') &&
      velloToolbar.includes('style::RADIUS_WIDGET') &&
      velloToolIcons.includes('style::ICON_STROKE_VELLO') &&
      velloToolIcons.includes('style::ICON_SCALE_VELLO') &&
      velloActionIcons.includes('style::ICON_STROKE_VELLO') &&
      velloActionIcons.includes('style::ICON_SCALE_VELLO'),
    pinWindowKeepsMarkdownOutOfImagePath:
      appTsx.includes('React.lazy(() => import("./components/Pin/PinCollectionWindow"))') &&
      pinCollectionWindow.includes("React.lazy(() => import('react-markdown'))") &&
      pinCollectionWindow.includes('const PinCard = React.memo') &&
      pinCollectionWindow.includes("pin.data.type === 'ImageBase64'") &&
      pinCollectionWindow.includes('decoding="async"') &&
      pinCollectionWindow.includes('<ReactMarkdown>{pin.data.payload}</ReactMarkdown>'),
    frontendShowsStartedTargetAndStopRule:
      eventListener.includes('started_detail') &&
      eventListener.includes('targetWindow') &&
      eventListener.includes('maxCaptureFrames') &&
      eventListener.includes('noGrowthLimit'),
    frontendShowsNoGrowthProgress:
      eventListener.includes('progress_checking') &&
      eventListener.includes("phase === 'checking_stop'"),
    previewShowsStitchingDiagnostics:
      preview.includes('logic_stats') &&
      preview.includes('stitch_quality') &&
      preview.includes('stitch_perf') &&
      preview.includes('seededMatchY') &&
      preview.includes('stitchReports') &&
      preview.includes('noGrowthLimit') &&
      preview.includes('maxCaptureFrames') &&
      preview.includes('wheelStep'),
    scrollingLoopErrorsStayVisibleInPreview:
      scrollingRust.includes('error_message') &&
      scrollingRust.includes('loop_error') &&
      scrollingRust.includes('emit_capture_error') &&
      preview.includes('errorMessage') &&
      preview.includes('scrolling_preview.error_message') &&
      enLocale.includes('"error_message"') &&
      zhLocale.includes('"error_message"'),
    frontendShowsOcrCaptureDiagnostics:
      ocrResultWindow.includes('overlayHideConfirmed') &&
      ocrResultWindow.includes('overlayVisibleAfterHide') &&
      ocrResultWindow.includes('capture_overlay_meta') &&
      ocrResultWindow.includes('overlayRisk') &&
      ocrResultWindow.includes('overlay_warning'),
    localesCoverScrollingDiagnostics:
      enLocale.includes('"started_detail"') &&
      enLocale.includes('"progress_checking"') &&
      enLocale.includes('"logic_stats"') &&
      enLocale.includes('"stitch_quality"') &&
      enLocale.includes('"stitch_perf"') &&
      zhLocale.includes('"started_detail"') &&
      zhLocale.includes('"progress_checking"') &&
      zhLocale.includes('"logic_stats"') &&
      zhLocale.includes('"stitch_quality"') &&
      zhLocale.includes('"stitch_perf"'),
    localesCoverOcrCaptureDiagnostics:
      enLocale.includes('"capture_overlay_meta"') &&
      enLocale.includes('"overlay_warning"') &&
      zhLocale.includes('"capture_overlay_meta"') &&
      zhLocale.includes('"overlay_warning"'),
    localesCoverCaptureFailureHud:
      enLocale.includes('"capture_failed_detail"') &&
      zhLocale.includes('"capture_failed_detail"'),
  };
};

const statFile = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    failures.push(`Missing required file: ${relativePath}`);
    return null;
  }

  const stat = fs.statSync(fullPath);
  if (!stat.isFile()) {
    failures.push(`Expected a file: ${relativePath}`);
    return null;
  }

  return { fullPath, relativePath, stat };
};

const optionalPathStat = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return null;
  }
  return { fullPath, relativePath, stat: fs.statSync(fullPath) };
};

const collectLatestFile = (relativePath) => {
  const start = optionalPathStat(relativePath);
  if (!start) {
    return { root: relativePath, present: false, fileCount: 0, latestFile: null };
  }

  let fileCount = 0;
  let latestFile = null;
  const visit = (entry) => {
    const stat = fs.statSync(entry.fullPath);
    if (stat.isDirectory()) {
      for (const child of fs.readdirSync(entry.fullPath, { withFileTypes: true })) {
        if (child.name === 'node_modules' || child.name === 'target' || child.name === 'dist') {
          continue;
        }
        visit({
          fullPath: path.join(entry.fullPath, child.name),
          relativePath: path.join(entry.relativePath, child.name).replaceAll(path.sep, '/'),
        });
      }
      return;
    }
    if (!stat.isFile()) {
      return;
    }
    fileCount += 1;
    if (!latestFile || stat.mtimeMs > latestFile.modifiedMs) {
      latestFile = {
        path: entry.relativePath.replaceAll(path.sep, '/'),
        modifiedAt: stat.mtime.toISOString(),
        modifiedAtLocal: formatTime(stat.mtime),
        modifiedMs: stat.mtimeMs,
      };
    }
  };

  visit(start);

  return {
    root: relativePath,
    present: true,
    fileCount,
    latestFile,
  };
};

const collectReleaseSourceSnapshot = () => {
  const rootsToScan = [
    'package.json',
    'index.html',
    'tsconfig.json',
    'tsconfig.app.json',
    'tsconfig.node.json',
    'vite.config.ts',
    'src',
    'src-tauri/Cargo.toml',
    'src-tauri/Cargo.lock',
    'src-tauri/tauri.conf.json',
    'src-tauri/src',
  ];
  const roots = rootsToScan.map(collectLatestFile);
  const presentRoots = roots.filter((item) => item.present);
  const latestFile = presentRoots
    .map((item) => item.latestFile)
    .filter(Boolean)
    .sort((a, b) => b.modifiedMs - a.modifiedMs)[0] || null;

  return {
    roots,
    fileCount: presentRoots.reduce((total, item) => total + item.fileCount, 0),
    latestFile,
  };
};

const releaseTimeZone = process.env.NEXSPOT_RELEASE_TZ || 'Asia/Taipei';
const timeFormatter = new Intl.DateTimeFormat('sv-SE', {
  timeZone: releaseTimeZone,
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
});

const formatBytes = (bytes) => `${(bytes / 1024 / 1024).toFixed(2)} MB`;
const formatTime = (date) => `${timeFormatter.format(date)} ${releaseTimeZone}`;

const hashFile = (fullPath) => {
  const hash = crypto.createHash('sha256');
  const bytes = fs.readFileSync(fullPath);
  hash.update(bytes);
  return hash.digest('hex');
};

const quotePowerShellLiteral = (value) => `'${value.replaceAll("'", "''")}'`;

const getAuthenticodeStatus = (artifact) => {
  if (!artifact || process.platform !== 'win32') {
    return null;
  }

  const command = [
    '[Console]::OutputEncoding = [System.Text.Encoding]::UTF8',
    `$sig = Get-AuthenticodeSignature -LiteralPath ${quotePowerShellLiteral(artifact.fullPath)}`,
    '[pscustomobject]@{Status=$sig.Status.ToString(); StatusMessage=$sig.StatusMessage; Subject=$sig.SignerCertificate.Subject} | ConvertTo-Json -Compress',
  ].join('; ');

  const result = spawnSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command], {
    encoding: 'utf8',
    windowsHide: true,
  });

  if (result.error || result.status !== 0) {
    warnings.push(`Unable to read Authenticode status for ${artifact.relativePath}.`);
    return null;
  }

  try {
    const parsed = JSON.parse(result.stdout.trim());
    return {
      status: String(parsed.Status || 'Unknown'),
      message: String(parsed.StatusMessage || ''),
      subject: parsed.Subject ? String(parsed.Subject) : '',
    };
  } catch {
    warnings.push(`Unable to parse Authenticode status for ${artifact.relativePath}.`);
    return null;
  }
};

const getVersionInfo = (artifact) => {
  if (!artifact || process.platform !== 'win32') {
    return null;
  }

  const command = [
    '[Console]::OutputEncoding = [System.Text.Encoding]::UTF8',
    `$v = (Get-Item -LiteralPath ${quotePowerShellLiteral(artifact.fullPath)}).VersionInfo`,
    '[pscustomobject]@{FileVersion=$v.FileVersion; ProductVersion=$v.ProductVersion; ProductName=$v.ProductName; FileDescription=$v.FileDescription; OriginalFilename=$v.OriginalFilename} | ConvertTo-Json -Compress',
  ].join('; ');

  const result = spawnSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command], {
    encoding: 'utf8',
    windowsHide: true,
  });

  if (result.error || result.status !== 0) {
    warnings.push(`Unable to read Windows version info for ${artifact.relativePath}.`);
    return null;
  }

  try {
    const parsed = JSON.parse(result.stdout.trim());
    return {
      fileVersion: String(parsed.FileVersion || ''),
      productVersion: String(parsed.ProductVersion || ''),
      productName: String(parsed.ProductName || ''),
      fileDescription: String(parsed.FileDescription || ''),
      originalFilename: String(parsed.OriginalFilename || ''),
    };
  } catch {
    warnings.push(`Unable to parse Windows version info for ${artifact.relativePath}.`);
    return null;
  }
};

const signatureEvidence = (signature) =>
  signature
    ? {
        status: signature.status,
        subject: signature.subject,
      }
    : null;

const versionInfoEvidence = (versionInfo) =>
  versionInfo
    ? {
        fileVersion: String(versionInfo.fileVersion || ''),
        productVersion: String(versionInfo.productVersion || ''),
        productName: String(versionInfo.productName || ''),
        fileDescription: String(versionInfo.fileDescription || ''),
        originalFilename: String(versionInfo.originalFilename || ''),
      }
    : null;

const summarizeManualQaMatrix = (fullPath) => {
  const text = fs.readFileSync(fullPath, 'utf8');
  const rows = [];

  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('|') || trimmed.includes('---')) {
      continue;
    }

    const cells = trimmed
      .split('|')
      .slice(1, -1)
      .map((cell) => cell.trim());
    const id = cells[0] || '';
    if (!/^(ENV|CAP|OCR|SCR|PIN|REL)-\d{2}$/.test(id)) {
      continue;
    }

    rows.push({
      id,
      required: cells[2] || '',
      status: cells[3] || 'Missing',
    });
  }

  const byStatus = rows.reduce((acc, row) => {
    acc[row.status] = (acc[row.status] || 0) + 1;
    return acc;
  }, {});
  const statusById = rows.reduce((acc, row) => {
    acc[row.id] = row.status;
    return acc;
  }, {});
  const requiredRows = rows.filter((row) => row.required === 'Yes');
  const pendingRequired = requiredRows.filter((row) => row.status === 'Pending').map((row) => row.id);
  const failedRequired = requiredRows
    .filter((row) => /failed|blocked/i.test(row.status))
    .map((row) => row.id);

  return {
    totalRows: rows.length,
    requiredRows: requiredRows.length,
    byStatus,
    statusById,
    pendingRequired,
    failedRequired,
  };
};

const packageJson = readJson('package.json');
const tauriConfig = readJson('src-tauri/tauri.conf.json');

if (packageJson.version !== tauriConfig.version) {
  failures.push(
    `Version mismatch: package.json=${packageJson.version}, tauri.conf.json=${tauriConfig.version}`,
  );
}

if (tauriConfig.productName !== 'NexSpot') {
  failures.push(`Unexpected product name: ${tauriConfig.productName}`);
}

const bundleTargets = tauriConfig.bundle?.targets;
if (!Array.isArray(bundleTargets) || !bundleTargets.includes('nsis')) {
  failures.push('Tauri bundle targets must include nsis for commercial trial distribution.');
}

const windowsBundle = tauriConfig.bundle?.windows || {};
const webviewInstallMode = windowsBundle.webviewInstallMode?.type || '';
const nsisInstallMode = windowsBundle.nsis?.installMode || '';
if (webviewInstallMode !== 'skip') {
  failures.push('Commercial trial installer must explicitly set bundle.windows.webviewInstallMode.type to skip; document WebView2 as a pilot prerequisite or switch to offline/fixed runtime before public stable.');
}
if (nsisInstallMode !== 'currentUser') {
  failures.push('NSIS installer must explicitly use currentUser installMode for repeatable pilot smoke tests.');
}

if (typeof packageJson.scripts?.release !== 'string' || !packageJson.scripts.release.includes('tauri build')) {
  failures.push('package.json must expose a release script that runs tauri build.');
}

const version = packageJson.version;
const exe = statFile('src-tauri/target/release/nexspot.exe');
const installer = statFile(`src-tauri/target/release/bundle/nsis/NexSpot_${version}_x64-setup.exe`);
const distIndex = statFile('dist/index.html');
const report = statFile('docs/10-COMMERCIAL-RELEASE-REVIEW.md');
const manualQaMatrix = statFile('docs/12-MANUAL-QA-MATRIX.md');
const commercialIssueLog = statFile('docs/16-COMMERCIAL-QA-ISSUE-LOG.md');
const unsignedPilotNotice = statFile('docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md');
const manualQaSummary = manualQaMatrix ? summarizeManualQaMatrix(manualQaMatrix.fullPath) : null;
const releaseSourceSnapshot = collectReleaseSourceSnapshot();
const exeHash = exe ? hashFile(exe.fullPath) : null;
const installerHash = installer ? hashFile(installer.fullPath) : null;
const smokeEvidence = readOptionalJson('docs/13-SMOKE-EVIDENCE.json');
const installerSmokeEvidence = readOptionalJson('docs/14-INSTALLER-SMOKE-EVIDENCE.json');
const manualQaGateEvidence = readOptionalJson('docs/17-MANUAL-QA-GATE.json');
const pilotDistributionManifest = readOptionalJson('docs/19-PILOT-DISTRIBUTION-MANIFEST.json');
const exeSignature = getAuthenticodeStatus(exe);
const installerSignature = getAuthenticodeStatus(installer);
const exeVersionInfo = getVersionInfo(exe);
const installerVersionInfo = getVersionInfo(installer);
const commercialSourceGate = collectCommercialSourceGate();

for (const [name, passed] of Object.entries(commercialSourceGate)) {
  if (!passed) {
    failures.push(`Commercial source gate failed: ${name}`);
  }
}

if (exe && exe.stat.size < 10 * 1024 * 1024) {
  failures.push(`Release EXE is suspiciously small: ${formatBytes(exe.stat.size)}`);
}

if (installer && installer.stat.size < 5 * 1024 * 1024) {
  failures.push(`NSIS installer is suspiciously small: ${formatBytes(installer.stat.size)}`);
}

if (exe && distIndex && exe.stat.mtimeMs + 1_000 < distIndex.stat.mtimeMs) {
  failures.push('Release EXE is older than the frontend dist output. Run npm run release again.');
}

if (installer && exe && installer.stat.mtimeMs + 1_000 < exe.stat.mtimeMs) {
  failures.push('NSIS installer is older than the release EXE. Run npm run release again.');
}

if (exe && releaseSourceSnapshot.latestFile && exe.stat.mtimeMs + 1_000 < releaseSourceSnapshot.latestFile.modifiedMs) {
  failures.push(
    `Release EXE is older than release source input ${releaseSourceSnapshot.latestFile.path}. Run npm run release again.`,
  );
}

for (const [label, signature] of [
  ['release EXE', exeSignature],
  ['NSIS installer', installerSignature],
]) {
  if (signature && signature.status !== 'Valid') {
    warnings.push(`${label} Authenticode status: ${signature.status}.`);
  }
}

for (const [label, versionInfo] of [
  ['release EXE', exeVersionInfo],
  ['NSIS installer', installerVersionInfo],
]) {
  if (!versionInfo) {
    continue;
  }
  if (versionInfo.fileVersion !== version) {
    failures.push(`${label} FileVersion mismatch: ${versionInfo.fileVersion || 'missing'} != ${version}`);
  }
  if (versionInfo.productVersion !== version) {
    failures.push(`${label} ProductVersion mismatch: ${versionInfo.productVersion || 'missing'} != ${version}`);
  }
  if (versionInfo.productName !== tauriConfig.productName) {
    failures.push(`${label} ProductName mismatch: ${versionInfo.productName || 'missing'} != ${tauriConfig.productName}`);
  }
  if (versionInfo.fileDescription !== tauriConfig.productName) {
    warnings.push(`${label} FileDescription is ${versionInfo.fileDescription || 'missing'}; expected ${tauriConfig.productName}.`);
  }
}

const assetsDir = path.join(root, 'dist', 'assets');
let jsAssets = [];
if (!fs.existsSync(assetsDir)) {
  failures.push('Missing dist/assets. Run npm run build or npm run release.');
} else {
  jsAssets = fs
    .readdirSync(assetsDir)
    .filter((file) => file.endsWith('.js'))
    .map((file) => {
      const filePath = path.join(assetsDir, file);
      return { file, size: fs.statSync(filePath).size };
    })
    .sort((a, b) => b.size - a.size);

  if (jsAssets.length === 0) {
    failures.push('No JavaScript assets found in dist/assets.');
  }

  const oversizedAssets = jsAssets.filter((asset) => asset.size > 500 * 1024);
  for (const asset of oversizedAssets) {
    failures.push(`JavaScript chunk exceeds 500 KiB: ${asset.file} (${formatBytes(asset.size)})`);
  }
}

if (report) {
  const reportText = fs.readFileSync(report.fullPath, 'utf8');
  const installerName = `NexSpot_${version}_x64-setup.exe`;
  if (!reportText.includes(installerName)) {
    failures.push(`Commercial release report does not mention ${installerName}.`);
  }
  if (exeHash && !reportText.includes(exeHash)) {
    failures.push('Commercial release report does not contain the current release EXE SHA256.');
  }
  if (installerHash && !reportText.includes(installerHash)) {
    failures.push('Commercial release report does not contain the current NSIS installer SHA256.');
  }
  if (!reportText.includes('Manual QA Matrix Gate')) {
    failures.push('Commercial release report does not mention the manual QA matrix gate.');
  }
  if (!reportText.includes('docs/16-COMMERCIAL-QA-ISSUE-LOG.md')) {
    failures.push('Commercial release report does not mention the commercial QA issue log.');
  }
  if (!reportText.includes('docs/12-MANUAL-QA-MATRIX.md')) {
    failures.push('Commercial release report does not mention the manual QA matrix file.');
  }
}

if (manualQaMatrix) {
  const matrixText = fs.readFileSync(manualQaMatrix.fullPath, 'utf8');
  for (const requiredText of ['real-device completion', 'Vello OCR', 'Scrolling Capture Acceptance']) {
    if (!matrixText.includes(requiredText)) {
      failures.push(`Manual QA matrix is missing required section text: ${requiredText}`);
    }
  }
  if (!manualQaSummary || manualQaSummary.totalRows === 0) {
    failures.push('Manual QA matrix does not contain any recognized test rows.');
  }
  if (manualQaSummary?.failedRequired.length > 0) {
    failures.push(`Manual QA matrix has failed required rows: ${manualQaSummary.failedRequired.join(', ')}`);
  }
  if (manualQaSummary?.pendingRequired.length > 0) {
    warnings.push(`Manual QA matrix required rows still pending: ${manualQaSummary.pendingRequired.length}.`);
  }
}

if (!manualQaGateEvidence) {
  failures.push('Missing manual QA gate evidence: docs/17-MANUAL-QA-GATE.json. Run npm.cmd run qa:manual:gate.');
} else {
  if (manualQaGateEvidence.schema !== 'nexspot.manual-qa-gate.v1') {
    failures.push('Manual QA gate evidence schema is invalid.');
  }
  if (manualQaGateEvidence.release?.hashes?.exe !== exeHash) {
    failures.push('Manual QA gate evidence EXE hash does not match the current release EXE.');
  }
  if (manualQaGateEvidence.release?.hashes?.nsisInstaller !== installerHash) {
    failures.push('Manual QA gate evidence installer hash does not match the current NSIS installer.');
  }
  if (manualQaSummary) {
    const gatePending = manualQaGateEvidence.matrix?.pendingRequired || [];
    const gateFailed = manualQaGateEvidence.matrix?.failedRequired || [];
    if (gatePending.length !== manualQaSummary.pendingRequired.length) {
      failures.push('Manual QA gate evidence pending count is stale. Run npm.cmd run qa:manual:gate.');
    }
    if (gateFailed.length !== manualQaSummary.failedRequired.length) {
      failures.push('Manual QA gate evidence failed count is stale. Run npm.cmd run qa:manual:gate.');
    }
  }
  if (manualQaGateEvidence.decisions?.internalTrial !== 'Go') {
    failures.push('Manual QA gate evidence does not allow internal trial.');
  }
  if (!String(manualQaGateEvidence.decisions?.publicStableRelease || '').includes('No-Go')) {
    warnings.push('Manual QA gate says public stable is Go. Reconfirm all P0 blockers, matrix evidence, and signing are actually closed.');
  }
}

if (!pilotDistributionManifest) {
  failures.push(
    'Missing pilot distribution manifest: docs/19-PILOT-DISTRIBUTION-MANIFEST.json. Run node scripts\\pilot-distribution-manifest.mjs --write-manifest docs\\19-PILOT-DISTRIBUTION-MANIFEST.json.',
  );
} else {
  if (pilotDistributionManifest.schema !== 'nexspot.pilot-distribution-manifest.v1') {
    failures.push('Pilot distribution manifest schema is invalid.');
  }
  if (pilotDistributionManifest.artifacts?.exe?.sha256 !== exeHash) {
    failures.push('Pilot distribution manifest EXE hash does not match the current release EXE.');
  }
  if (pilotDistributionManifest.artifacts?.nsisInstaller?.sha256 !== installerHash) {
    failures.push('Pilot distribution manifest installer hash does not match the current NSIS installer.');
  }
  if (pilotDistributionManifest.decisions?.internalTrial !== 'Go') {
    warnings.push('Pilot distribution manifest does not currently allow internal trial. Refresh it after release evidence if artifact hashes just changed.');
  }
  if (pilotDistributionManifest.decisions?.smallCommercialPilot !== 'Go with unsigned notice') {
    warnings.push('Pilot distribution manifest does not currently allow small commercial pilot with unsigned notice. Refresh it after release evidence if artifact hashes just changed.');
  }
  if ((pilotDistributionManifest.blockers || []).length > 0) {
    warnings.push(`Pilot distribution manifest currently has blockers: ${pilotDistributionManifest.blockers.join('; ')}`);
  }
}

if (commercialIssueLog) {
  const issueLogText = fs.readFileSync(commercialIssueLog.fullPath, 'utf8');
  for (const requiredText of [
    'Public Stable Blockers',
    'QA-001',
    'QA-002',
    'QA-003',
    'QA-004',
    'QA-005',
    'Release evidence',
    exeHash || 'release-exe-hash-unavailable',
    installerHash || 'installer-hash-unavailable',
  ]) {
    if (!issueLogText.includes(requiredText)) {
      failures.push(`Commercial QA issue log is missing required text: ${requiredText}`);
    }
  }
  if (!issueLogText.includes('Public stable release requires all P0/P1 items closed')) {
    failures.push('Commercial QA issue log must state the public stable close condition.');
  }
}

const rel01Status = manualQaSummary?.statusById?.['REL-01'] || '';
const rel01ClaimsAutomatedSmoke = /automated smoke/i.test(rel01Status);
if (rel01ClaimsAutomatedSmoke) {
  if (!smokeEvidence) {
    failures.push('REL-01 claims automated smoke passed, but docs/13-SMOKE-EVIDENCE.json is missing.');
  } else {
    if (smokeEvidence.schema !== 'nexspot.release-smoke-evidence.v1') {
      failures.push('Smoke evidence schema is invalid.');
    }
    if (smokeEvidence.passed !== true) {
      failures.push('Smoke evidence does not show a passing launch.');
    }
    if (exeHash && smokeEvidence.exe?.sha256 !== exeHash) {
      failures.push('Smoke evidence EXE hash does not match the current release EXE.');
    }
    if (Number(smokeEvidence.durationMs || 0) < 6000) {
      failures.push('Smoke evidence duration must be at least 6000 ms.');
    }
    if (smokeEvidence.launch?.mainWindowSeen !== true) {
      failures.push('Smoke evidence does not show a main window handle.');
    }
    if (smokeEvidence.launch?.mainWindowTitle && smokeEvidence.launch.mainWindowTitle !== tauriConfig.productName) {
      failures.push(
        `Smoke evidence main window title mismatch: ${smokeEvidence.launch.mainWindowTitle} != ${tauriConfig.productName}`,
      );
    } else if (!smokeEvidence.launch?.mainWindowTitle) {
      warnings.push('Smoke evidence main window title is not exposed; using main window handle as launch proof.');
    }
  }
}
if (!rel01ClaimsAutomatedSmoke && smokeEvidence && exeHash && smokeEvidence.exe?.sha256 !== exeHash) {
  warnings.push('Release smoke evidence exists but does not match the current release EXE hash.');
}

const installerSmokeIds = ['REL-02', 'REL-03', 'REL-04', 'REL-05'];
const installerSmokeClaims = installerSmokeIds.some((id) =>
  /automated installer smoke/i.test(manualQaSummary?.statusById?.[id] || ''),
);
if (installerSmokeClaims) {
  if (!installerSmokeEvidence) {
    failures.push(
      `${installerSmokeIds.join(', ')} claim automated installer smoke passed, but docs/14-INSTALLER-SMOKE-EVIDENCE.json is missing.`,
    );
  } else {
    const result = installerSmokeEvidence.result || {};
    if (installerSmokeEvidence.schema !== 'nexspot.installer-smoke-evidence.v1') {
      failures.push('Installer smoke evidence schema is invalid.');
    }
    if (installerSmokeEvidence.executed !== true || installerSmokeEvidence.passed !== true || result.ok !== true) {
      failures.push('Installer smoke evidence does not show a passing executed run.');
    }
    if (installerHash && installerSmokeEvidence.installer?.sha256 !== installerHash) {
      failures.push('Installer smoke evidence installer hash does not match the current NSIS installer.');
    }
    if (result.freshInstall?.exitCode !== 0) {
      failures.push('Installer smoke evidence does not show a successful fresh install.');
    }
    if (result.overwriteInstall?.exitCode !== 0) {
      failures.push('Installer smoke evidence does not show a successful overwrite install.');
    }
    if (result.launch?.ok !== true || result.launch?.mainWindowSeen !== true) {
      failures.push('Installer smoke evidence does not show a successful installed-app launch.');
    }
    if (result.launch?.mainWindowTitle && result.launch.mainWindowTitle !== tauriConfig.productName) {
      failures.push(
        `Installer smoke main window title mismatch: ${result.launch.mainWindowTitle} != ${tauriConfig.productName}`,
      );
    } else if (!result.launch?.mainWindowTitle) {
      warnings.push('Installer smoke main window title is not exposed; using main window handle as launch proof.');
    }
    if (!result.installedExeSha256) {
      failures.push('Installer smoke evidence does not include the installed EXE SHA256.');
    }
    if (!result.freshInstalledExeSha256) {
      failures.push('Installer smoke evidence does not include the fresh-installed EXE SHA256.');
    }
    if (
      result.freshInstalledExeSha256 &&
      result.installedExeSha256 &&
      result.freshInstalledExeSha256 !== result.installedExeSha256
    ) {
      failures.push('Installer smoke overwrite install changed the installed EXE hash unexpectedly.');
    }
    if (Number(result.installedExeBytes || 0) < 10 * 1024 * 1024) {
      failures.push(`Installer smoke installed EXE is suspiciously small: ${formatBytes(Number(result.installedExeBytes || 0))}`);
    }
    if (result.installedExeVersionInfo?.fileVersion !== version) {
      failures.push(
        `Installer smoke installed EXE FileVersion mismatch: ${result.installedExeVersionInfo?.fileVersion || 'missing'} != ${version}`,
      );
    }
    if (result.installedExeVersionInfo?.productVersion !== version) {
      failures.push(
        `Installer smoke installed EXE ProductVersion mismatch: ${result.installedExeVersionInfo?.productVersion || 'missing'} != ${version}`,
      );
    }
    if (result.installedExeVersionInfo?.productName !== tauriConfig.productName) {
      failures.push(
        `Installer smoke installed EXE ProductName mismatch: ${result.installedExeVersionInfo?.productName || 'missing'} != ${tauriConfig.productName}`,
      );
    }
    if (result.uninstall?.exitCode !== 0 || result.finalAppExeRemoved !== true || result.installDirRemoved !== true) {
      failures.push('Installer smoke evidence does not show a clean uninstall.');
    }
  }
}
if (!installerSmokeClaims && installerSmokeEvidence && installerHash && installerSmokeEvidence.installer?.sha256 !== installerHash) {
  warnings.push('Installer smoke evidence exists but does not match the current NSIS installer hash.');
}

const rel06Status = manualQaSummary?.statusById?.['REL-06'] || '';
const rel06ClaimsUnsignedPilotNotice = /unsigned pilot notice/i.test(rel06Status);
if (rel06ClaimsUnsignedPilotNotice) {
  if (!unsignedPilotNotice) {
    failures.push('REL-06 claims unsigned pilot notice passed, but docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md is missing.');
  } else {
    const noticeText = fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8');
    const requiredNoticeText = [
      'REL-06',
      'NotSigned',
      'Commercial Trial',
      'Internal trial',
      'Small commercial pilot',
      'Public stable release',
      'No-Go',
      'Windows may show an unknown publisher or security warning',
      'Verify the SHA256 hash before testing',
    ];
    for (const text of requiredNoticeText) {
      if (!noticeText.includes(text)) {
        failures.push(`Unsigned pilot distribution notice is missing required text: ${text}`);
      }
    }
    if (exeHash && !noticeText.includes(exeHash)) {
      failures.push('Unsigned pilot distribution notice does not contain the current release EXE SHA256.');
    }
    if (installerHash && !noticeText.includes(installerHash)) {
      failures.push('Unsigned pilot distribution notice does not contain the current NSIS installer SHA256.');
    }
    if (exeSignature?.status !== 'NotSigned' || installerSignature?.status !== 'NotSigned') {
      warnings.push('REL-06 unsigned pilot notice is present, but one or more artifacts are no longer NotSigned. Revisit signing guidance.');
    }
  }
}

warnings.push('Manual gate still required: GUI matrix for Vello OCR, scrolling capture, multi-display/DPI.');
warnings.push('Manual gate still required: code-signing and Windows warning prompt review.');

const summaryRows = [
  ['version', version],
  ['bundle targets', Array.isArray(bundleTargets) ? bundleTargets.join(', ') : String(bundleTargets)],
  ['release exe', exe ? `${exe.relativePath} (${formatBytes(exe.stat.size)}, ${formatTime(exe.stat.mtime)})` : 'missing'],
  ['release exe sha256', exeHash || 'missing'],
  ['release exe signature', exeSignature ? exeSignature.status : process.platform === 'win32' ? 'unknown' : 'skipped'],
  [
    'nsis installer',
    installer ? `${installer.relativePath} (${formatBytes(installer.stat.size)}, ${formatTime(installer.stat.mtime)})` : 'missing',
  ],
  ['nsis installer sha256', installerHash || 'missing'],
  ['nsis installer signature', installerSignature ? installerSignature.status : process.platform === 'win32' ? 'unknown' : 'skipped'],
];

console.log('NexSpot release verification');
console.log('='.repeat(30));
for (const [label, value] of summaryRows) {
  console.log(`${label}: ${value}`);
}

if (warnings.length > 0) {
  console.log('\nWarnings');
  for (const warning of warnings) {
    console.log(`- ${warning}`);
  }
}

if (failures.length > 0) {
  console.error('\nFailures');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

if (evidenceRelativePath) {
  const evidencePath = path.join(root, evidenceRelativePath);
  const evidence = {
    schema: 'nexspot.release-evidence.v1',
    generatedAt: new Date().toISOString(),
    productName: tauriConfig.productName,
    version,
    bundleTargets,
    timezone: releaseTimeZone,
    artifacts: {
      exe: exe
        ? {
            path: exe.relativePath,
            bytes: exe.stat.size,
            modifiedAt: exe.stat.mtime.toISOString(),
            modifiedAtLocal: formatTime(exe.stat.mtime),
            sha256: exeHash,
            authenticode: signatureEvidence(exeSignature),
            versionInfo: versionInfoEvidence(exeVersionInfo),
          }
        : null,
      nsisInstaller: installer
        ? {
            path: installer.relativePath,
            bytes: installer.stat.size,
            modifiedAt: installer.stat.mtime.toISOString(),
            modifiedAtLocal: formatTime(installer.stat.mtime),
            sha256: installerHash,
            authenticode: signatureEvidence(installerSignature),
            versionInfo: versionInfoEvidence(installerVersionInfo),
          }
        : null,
    },
    frontend: {
      largestJsAssets: jsAssets.slice(0, 8).map((asset) => ({
        file: asset.file,
        bytes: asset.size,
      })),
      maxChunkBytes: 500 * 1024,
    },
    releaseSourceSnapshot: {
      fileCount: releaseSourceSnapshot.fileCount,
      latestFile: releaseSourceSnapshot.latestFile
        ? {
            path: releaseSourceSnapshot.latestFile.path,
            modifiedAt: releaseSourceSnapshot.latestFile.modifiedAt,
            modifiedAtLocal: releaseSourceSnapshot.latestFile.modifiedAtLocal,
          }
        : null,
      roots: releaseSourceSnapshot.roots.map((item) => ({
        root: item.root,
        present: item.present,
        fileCount: item.fileCount,
        latestFile: item.latestFile
          ? {
              path: item.latestFile.path,
              modifiedAt: item.latestFile.modifiedAt,
              modifiedAtLocal: item.latestFile.modifiedAtLocal,
            }
          : null,
      })),
    },
    automatedGates: {
      versionMatchesTauriConfig: packageJson.version === tauriConfig.version,
      windowsVersionInfoMatches: [exeVersionInfo, installerVersionInfo].every(
        (info) => info && info.fileVersion === version && info.productVersion === version && info.productName === tauriConfig.productName,
      ),
      nsisTargetConfigured: Array.isArray(bundleTargets) && bundleTargets.includes('nsis'),
      windowsInstallerModeExplicit: webviewInstallMode === 'skip' && nsisInstallMode === 'currentUser',
      reportMentionsInstaller: report ? fs.readFileSync(report.fullPath, 'utf8').includes(`NexSpot_${version}_x64-setup.exe`) : false,
      reportContainsCurrentHashes: report
        ? fs.readFileSync(report.fullPath, 'utf8').includes(exeHash || '') &&
          fs.readFileSync(report.fullPath, 'utf8').includes(installerHash || '')
        : false,
      manualQaMatrixPresent: Boolean(manualQaMatrix),
      manualQaMatrixHasRows: Boolean(manualQaSummary?.totalRows),
      commercialIssueLogPresent: Boolean(commercialIssueLog),
      releaseArtifactsNewerThanSource:
        Boolean(exe && releaseSourceSnapshot.latestFile) &&
        exe.stat.mtimeMs + 1_000 >= releaseSourceSnapshot.latestFile.modifiedMs,
      smokeEvidenceMatchesRelease: !rel01ClaimsAutomatedSmoke || Boolean(smokeEvidence?.passed && smokeEvidence.exe?.sha256 === exeHash),
      smokeMainWindowProofPresent:
        !rel01ClaimsAutomatedSmoke ||
        Boolean(
          smokeEvidence?.launch?.mainWindowSeen === true &&
            (!smokeEvidence.launch?.mainWindowTitle || smokeEvidence.launch.mainWindowTitle === tauriConfig.productName),
        ),
      installerSmokeMatchesRelease:
        !installerSmokeClaims || Boolean(installerSmokeEvidence?.passed && installerSmokeEvidence.installer?.sha256 === installerHash),
      installerSmokeEndToEndPassed:
        !installerSmokeClaims ||
        Boolean(
          installerSmokeEvidence?.result?.freshInstall?.exitCode === 0 &&
            installerSmokeEvidence?.result?.overwriteInstall?.exitCode === 0 &&
            installerSmokeEvidence?.result?.launch?.ok === true &&
            installerSmokeEvidence?.result?.uninstall?.exitCode === 0 &&
            installerSmokeEvidence?.result?.finalAppExeRemoved === true &&
            installerSmokeEvidence?.result?.installDirRemoved === true,
        ),
      installerInstalledExeVersionMatches:
        !installerSmokeClaims ||
        Boolean(
          installerSmokeEvidence?.result?.installedExeSha256 &&
            installerSmokeEvidence?.result?.freshInstalledExeSha256 &&
            installerSmokeEvidence?.result?.freshInstalledExeSha256 === installerSmokeEvidence?.result?.installedExeSha256 &&
            Number(installerSmokeEvidence?.result?.installedExeBytes || 0) >= 10 * 1024 * 1024 &&
            installerSmokeEvidence?.result?.installedExeVersionInfo?.fileVersion === version &&
            installerSmokeEvidence?.result?.installedExeVersionInfo?.productVersion === version &&
            installerSmokeEvidence?.result?.installedExeVersionInfo?.productName === tauriConfig.productName,
        ),
      unsignedPilotNoticeMatchesRelease:
        !rel06ClaimsUnsignedPilotNotice ||
        Boolean(
          unsignedPilotNotice &&
            fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes(exeHash || '') &&
            fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes(installerHash || ''),
        ),
      manualQaGateEvidenceMatchesRelease:
        Boolean(manualQaGateEvidence) &&
        manualQaGateEvidence.release?.hashes?.exe === exeHash &&
        manualQaGateEvidence.release?.hashes?.nsisInstaller === installerHash,
      pilotDistributionManifestMatchesRelease:
        Boolean(pilotDistributionManifest) &&
        pilotDistributionManifest.artifacts?.exe?.sha256 === exeHash &&
        pilotDistributionManifest.artifacts?.nsisInstaller?.sha256 === installerHash,
      commercialSourceGatePassed: Object.values(commercialSourceGate).every(Boolean),
      noOversizedJsChunks: jsAssets.every((asset) => asset.size <= 500 * 1024),
      releaseVerifyPassed: true,
    },
    commercialSourceGate,
    releaseSmoke: smokeEvidence
      ? {
          path: 'docs/13-SMOKE-EVIDENCE.json',
          generatedAt: smokeEvidence.generatedAt,
          passed: smokeEvidence.passed,
          durationMs: smokeEvidence.durationMs,
          exeSha256: smokeEvidence.exe?.sha256 || null,
          matchesCurrentRelease: Boolean(exeHash && smokeEvidence.exe?.sha256 === exeHash),
          launchReason: smokeEvidence.launch?.reason || null,
          mainWindowSeen: smokeEvidence.launch?.mainWindowSeen || false,
          mainWindowTitle: smokeEvidence.launch?.mainWindowTitle || '',
        }
      : null,
    installerSmoke: installerSmokeEvidence
      ? {
          path: 'docs/14-INSTALLER-SMOKE-EVIDENCE.json',
          generatedAt: installerSmokeEvidence.generatedAt,
          passed: installerSmokeEvidence.passed,
          installerSha256: installerSmokeEvidence.installer?.sha256 || null,
          matchesCurrentRelease: Boolean(installerHash && installerSmokeEvidence.installer?.sha256 === installerHash),
          installDir: installerSmokeEvidence.installDir || '',
          freshInstallExitCode: installerSmokeEvidence.result?.freshInstall?.exitCode ?? null,
          overwriteInstallExitCode: installerSmokeEvidence.result?.overwriteInstall?.exitCode ?? null,
          freshInstalledExeSha256: installerSmokeEvidence.result?.freshInstalledExeSha256 || null,
          installedExeSha256: installerSmokeEvidence.result?.installedExeSha256 || null,
          installedExeHashStableAfterOverwrite:
            Boolean(installerSmokeEvidence.result?.freshInstalledExeSha256) &&
            installerSmokeEvidence.result?.freshInstalledExeSha256 === installerSmokeEvidence.result?.installedExeSha256,
          installedExeBytes: installerSmokeEvidence.result?.installedExeBytes || null,
          installedExeVersionInfo: versionInfoEvidence(installerSmokeEvidence.result?.installedExeVersionInfo),
          mainWindowSeen: installerSmokeEvidence.result?.launch?.mainWindowSeen || false,
          mainWindowTitle: installerSmokeEvidence.result?.launch?.mainWindowTitle || '',
          uninstallExitCode: installerSmokeEvidence.result?.uninstall?.exitCode ?? null,
          finalAppExeRemoved: installerSmokeEvidence.result?.finalAppExeRemoved || false,
          installDirRemoved: installerSmokeEvidence.result?.installDirRemoved || false,
        }
      : null,
    unsignedPilotNotice: unsignedPilotNotice
      ? {
          path: unsignedPilotNotice.relativePath,
          rel06Status,
          appliesToUnsignedArtifacts: exeSignature?.status === 'NotSigned' && installerSignature?.status === 'NotSigned',
          containsCurrentExeHash: fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes(exeHash || ''),
          containsCurrentInstallerHash: fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes(installerHash || ''),
          pilotScopeOnly: fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes('Small commercial pilot'),
          publicStableNoGo: fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes('Public stable release') &&
            fs.readFileSync(unsignedPilotNotice.fullPath, 'utf8').includes('No-Go'),
        }
      : null,
    manualQaMatrix: manualQaMatrix
      ? {
          path: manualQaMatrix.relativePath,
          status: 'pending-real-device-completion',
          summary: manualQaSummary,
          requiredSuites: [
            'capture-toolbar',
            'vello-ocr',
            'scrolling-capture',
            'multi-display-dpi',
            'installer-uninstaller-overwrite',
            'code-signing-review',
          ],
        }
      : null,
    manualQaGate: manualQaGateEvidence
      ? {
          path: 'docs/17-MANUAL-QA-GATE.json',
          generatedAt: manualQaGateEvidence.generatedAt || '',
          decisions: manualQaGateEvidence.decisions || {},
          pendingRequired: manualQaGateEvidence.matrix?.pendingRequired || [],
          failedRequired: manualQaGateEvidence.matrix?.failedRequired || [],
          openP0: manualQaGateEvidence.issueLog?.openP0 || [],
          openP1: manualQaGateEvidence.issueLog?.openP1 || [],
          unsignedArtifacts: manualQaGateEvidence.release?.unsignedArtifacts || [],
          signingReadiness: manualQaGateEvidence.signingReadiness || null,
          publicStableBlockers: manualQaGateEvidence.publicStableBlockers || [],
        }
      : null,
    pilotDistributionManifest: pilotDistributionManifest
      ? {
          path: 'docs/19-PILOT-DISTRIBUTION-MANIFEST.json',
          generatedAt: pilotDistributionManifest.generatedAt || '',
          decisions: pilotDistributionManifest.decisions || {},
          artifacts: pilotDistributionManifest.artifacts || {},
          materialCount: Array.isArray(pilotDistributionManifest.materials)
            ? pilotDistributionManifest.materials.length
            : 0,
          blockers: pilotDistributionManifest.blockers || [],
        }
      : null,
    commercialIssueLog: commercialIssueLog
      ? {
          path: commercialIssueLog.relativePath,
          publicStableBlockersTracked: true,
          requiredBlockers: ['QA-001', 'QA-002', 'QA-003', 'QA-004', 'QA-005'],
        }
      : null,
    manualGatesRemaining: [
      'GUI matrix for Vello OCR, scrolling capture, multi-display/DPI',
      'Code-signing and Windows warning prompt review',
    ],
    warnings,
  };

  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
  console.log(`\nRelease evidence written: ${evidenceRelativePath}`);
}

console.log('\nRelease verification passed.');
