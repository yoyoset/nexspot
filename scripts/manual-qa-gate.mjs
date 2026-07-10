import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const strict = process.argv.includes('--strict');
const writeEvidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  writeEvidenceIndex >= 0 ? process.argv[writeEvidenceIndex + 1] || 'docs/17-MANUAL-QA-GATE.json' : null;
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const matrixPath = 'docs/12-MANUAL-QA-MATRIX.md';
const issueLogPath = 'docs/16-COMMERCIAL-QA-ISSUE-LOG.md';
const releaseEvidencePath = 'docs/11-RELEASE-EVIDENCE.json';
const qaEvidencePackPath = 'docs/qa-runs/latest/qa-evidence-pack.json';
const signingReadinessPath = 'docs/18-SIGNING-READINESS.json';

const readText = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8');
const readOptionalJson = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return null;
  }
  return parseJsonText(fs.readFileSync(fullPath, 'utf8'));
};
const readRequiredJson = (relativePath) => JSON.parse(readText(relativePath));
const hashFile = (fullPath) => {
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(fullPath));
  return hash.digest('hex');
};

const parseTableRows = (text, idPattern) => {
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
    if (idPattern.test(cells[0] || '')) {
      rows.push(cells);
    }
  }
  return rows;
};

const isPassedStatus = (status) => /^passed\b/i.test(status);
const isFailedStatus = (status) => /failed|blocked/i.test(status);
const isClosedStatus = (status) => /^(closed|done|passed)$/i.test(status);
const splitEvidencePaths = (value) =>
  String(value || '')
    .replaceAll('\\', '/')
    .split(';')
    .map((item) => item.trim().replace(/^`+|`+$/g, ''))
    .filter(Boolean);
const evidencePathExists = (relativePath) => {
  const resolvedPath = path.resolve(root, relativePath);
  if (!resolvedPath.toLowerCase().startsWith(`${root.toLowerCase()}${path.sep}`)) {
    return false;
  }
  return fs.existsSync(resolvedPath) && fs.statSync(resolvedPath).isFile();
};

const summarizeMatrix = () => {
  const text = readText(matrixPath);
  const rows = parseTableRows(text, /^(ENV|CAP|OCR|SCR|PIN|REL)-\d{2}$/);
  const items = rows.map((cells) => ({
    id: cells[0],
    title: cells[1] || '',
    required: cells[2] === 'Yes',
    status: cells[3] || 'Missing',
    evidence: cells[4] || '',
    notes: cells[5] || '',
  }));

  const required = items.filter((item) => item.required);
  const pendingRequired = required.filter((item) => item.status === 'Pending');
  const failedRequired = required.filter((item) => isFailedStatus(item.status));
  const passedRequired = required.filter((item) => isPassedStatus(item.status));
  const requiredWithoutEvidence = required.filter(
    (item) => isPassedStatus(item.status) && !item.evidence && !/^REL-\d{2}$/.test(item.id),
  );
  const requiredWithMissingEvidenceFiles = required
    .filter((item) => isPassedStatus(item.status))
    .flatMap((item) =>
      splitEvidencePaths(item.evidence)
        .filter((evidencePath) => !evidencePathExists(evidencePath))
        .map((evidencePath) => `${item.id}: ${evidencePath}`),
    );
  const byStatus = items.reduce((acc, item) => {
    acc[item.status] = (acc[item.status] || 0) + 1;
    return acc;
  }, {});

  return {
    totalRows: items.length,
    requiredRows: required.length,
    passedRequired: passedRequired.map((item) => item.id),
    pendingRequired: pendingRequired.map((item) => item.id),
    failedRequired: failedRequired.map((item) => item.id),
    requiredWithoutEvidence: requiredWithoutEvidence.map((item) => item.id),
    requiredWithMissingEvidenceFiles,
    byStatus,
    items,
  };
};

const summarizeIssueLog = () => {
  const text = readText(issueLogPath);
  const rows = parseTableRows(text, /^QA-\d{3}$/);
  const items = rows.map((cells) => ({
    id: cells[0],
    severity: cells[1] || '',
    area: cells[2] || '',
    relatedMatrixIds: cells[3] || '',
    status: cells[4] || '',
    owner: cells[5] || '',
    closeCondition: cells[6] || '',
  }));
  const openP0 = items.filter((item) => item.severity === 'P0' && !isClosedStatus(item.status));
  const openP1 = items.filter((item) => item.severity === 'P1' && !isClosedStatus(item.status));

  return {
    totalItems: items.length,
    openP0: openP0.map((item) => item.id),
    openP1: openP1.map((item) => item.id),
    items,
  };
};

const summarizeReleaseEvidence = () => {
  const packageJson = readRequiredJson('package.json');
  const exeFullPath = path.join(root, 'src-tauri/target/release/nexspot.exe');
  const installerRelativePath = `src-tauri/target/release/bundle/nsis/NexSpot_${packageJson.version}_x64-setup.exe`;
  const installerFullPath = path.join(root, installerRelativePath);
  const exeExists = fs.existsSync(exeFullPath);
  const installerExists = fs.existsSync(installerFullPath);
  const currentHashes = {
    exe: exeExists ? hashFile(exeFullPath) : '',
    nsisInstaller: installerExists ? hashFile(installerFullPath) : '',
  };
  const evidence = readOptionalJson(releaseEvidencePath);
  const evidenceHashes = {
    exe: evidence?.artifacts?.exe?.sha256 || '',
    nsisInstaller: evidence?.artifacts?.nsisInstaller?.sha256 || '',
  };
  const evidenceMatchesCurrent =
    Boolean(evidence) &&
    evidenceHashes.exe === currentHashes.exe &&
    evidenceHashes.nsisInstaller === currentHashes.nsisInstaller;

  if (!evidence) {
    return {
      present: false,
      currentArtifactsPresent: exeExists && installerExists,
      releaseVerifyPassed: false,
      evidenceMatchesCurrent: false,
      unsignedArtifacts: [],
      hashes: currentHashes,
      evidenceHashes,
      warnings: [`Missing ${releaseEvidencePath}`],
    };
  }

  const artifacts = evidence.artifacts || {};
  const signedEntries = [
    ['exe', artifacts.exe],
    ['nsisInstaller', artifacts.nsisInstaller],
  ];
  const unsignedArtifacts = signedEntries
    .filter(([, artifact]) => artifact?.authenticode?.status !== 'Valid')
    .map(([name, artifact]) => ({
      name,
      status: artifact?.authenticode?.status || 'Unknown',
      path: artifact?.path || '',
      sha256: artifact?.sha256 || '',
    }));

  return {
    present: true,
    currentArtifactsPresent: exeExists && installerExists,
    generatedAt: evidence.generatedAt || '',
    version: evidence.version || '',
    releaseVerifyPassed: evidence.automatedGates?.releaseVerifyPassed === true && evidenceMatchesCurrent,
    evidenceMatchesCurrent,
    releaseArtifactsNewerThanSource: evidence.automatedGates?.releaseArtifactsNewerThanSource === true,
    unsignedArtifacts,
    hashes: currentHashes,
    evidenceHashes,
    warnings: Array.isArray(evidence.warnings) ? evidence.warnings : [],
  };
};

const summarizeQaEvidencePack = (matrix, issueLog, release) => {
  const pack = readOptionalJson(qaEvidencePackPath);
  if (!pack) {
    return {
      present: false,
      matchesCurrentArtifacts: false,
      pendingRequiredMatches: false,
      openP0Matches: false,
      blockers: [`missing QA evidence pack: ${qaEvidencePackPath}`],
    };
  }

  const packPending = pack.pendingRequired || [];
  const packOpenP0 = pack.openP0 || [];
  const packOpenP1 = pack.openP1 || [];
  const blockers = [];
  const matchesCurrentArtifacts =
    pack.artifacts?.exe?.sha256 === release.hashes.exe &&
    pack.artifacts?.nsisInstaller?.sha256 === release.hashes.nsisInstaller;
  const pendingRequiredMatches =
    JSON.stringify([...packPending].sort()) === JSON.stringify([...matrix.pendingRequired].sort());
  const openP0Matches =
    JSON.stringify([...packOpenP0].sort()) === JSON.stringify([...issueLog.openP0].sort());
  const openP1Matches =
    JSON.stringify([...packOpenP1].sort()) === JSON.stringify([...issueLog.openP1].sort());

  if (pack.schema !== 'nexspot.qa-evidence-pack.v1') {
    blockers.push('QA evidence pack schema is invalid');
  }
  if (!matchesCurrentArtifacts) {
    blockers.push('QA evidence pack artifact hashes do not match current release');
  }
  if (!pendingRequiredMatches) {
    blockers.push('QA evidence pack pending-required list is stale');
  }
  if (!openP0Matches) {
    blockers.push('QA evidence pack open-P0 list is stale');
  }
  if (!openP1Matches) {
    blockers.push('QA evidence pack open-P1 list is stale');
  }
  if (!pack.environment?.node?.platform) {
    blockers.push('QA evidence pack is missing node environment snapshot');
  }
  if (!Array.isArray(pack.signatures) || pack.signatures.length < 2) {
    blockers.push('QA evidence pack is missing artifact signature snapshots');
  }
  if (!Array.isArray(pack.requiredAttachments) || pack.requiredAttachments.length === 0) {
    blockers.push('QA evidence pack is missing required attachment checklist');
  }

  return {
    present: true,
    path: qaEvidencePackPath,
    generatedAt: pack.generatedAt || '',
    matchesCurrentArtifacts,
    pendingRequiredMatches,
    openP0Matches,
    openP1Matches,
    publicStableDecision: pack.decisions?.publicStableRelease || '',
    pendingRequiredCount: packPending.length,
    openP0Count: packOpenP0.length,
    openP1Count: packOpenP1.length,
    blockers,
  };
};

const summarizeSigningReadiness = (release) => {
  const readiness = readOptionalJson(signingReadinessPath);
  if (!readiness) {
    return {
      present: false,
      path: signingReadinessPath,
      publicStableReady: false,
      unsignedPilotReady: false,
      matchesCurrentArtifacts: false,
      blockers: [`missing signing readiness evidence: ${signingReadinessPath}`],
    };
  }

  const exe = readiness.artifacts?.find((artifact) => artifact.key === 'exe');
  const nsisInstaller = readiness.artifacts?.find((artifact) => artifact.key === 'nsisInstaller');
  const matchesCurrentArtifacts =
    exe?.sha256 === release.hashes.exe &&
    nsisInstaller?.sha256 === release.hashes.nsisInstaller;
  const blockers = [];

  if (readiness.schema !== 'nexspot.signing-readiness.v1') {
    blockers.push('signing readiness schema is invalid');
  }
  if (!matchesCurrentArtifacts) {
    blockers.push('signing readiness artifact hashes do not match current release');
  }
  if (readiness.decisions?.publicStableReady !== true) {
    blockers.push('signing readiness is not public-stable ready');
  }
  if (readiness.decisions?.unsignedPilotReady !== true) {
    blockers.push('signing readiness does not confirm unsigned pilot readiness');
  }

  return {
    present: true,
    path: signingReadinessPath,
    generatedAt: readiness.generatedAt || '',
    publicStableReady: readiness.decisions?.publicStableReady === true,
    unsignedPilotReady: readiness.decisions?.unsignedPilotReady === true,
    matchesCurrentArtifacts,
    artifactStatuses: {
      exe: exe?.signature?.status || 'Unknown',
      nsisInstaller: nsisInstaller?.signature?.status || 'Unknown',
    },
    blockers,
  };
};

const matrix = summarizeMatrix();
const issueLog = summarizeIssueLog();
const release = summarizeReleaseEvidence();
const qaEvidencePack = summarizeQaEvidencePack(matrix, issueLog, release);
const signingReadiness = summarizeSigningReadiness(release);

const publicStableBlockers = [
  ...matrix.pendingRequired.map((id) => `manual matrix pending: ${id}`),
  ...matrix.failedRequired.map((id) => `manual matrix failed/blocked: ${id}`),
  ...matrix.requiredWithoutEvidence.map((id) => `passed required row lacks evidence: ${id}`),
  ...matrix.requiredWithMissingEvidenceFiles.map((item) => `passed required evidence file missing: ${item}`),
  ...issueLog.openP0.map((id) => `open P0 blocker: ${id}`),
  ...issueLog.openP1.map((id) => `open P1 polish blocker: ${id}`),
  ...release.unsignedArtifacts.map((artifact) => `artifact not signed: ${artifact.name} (${artifact.status})`),
  ...qaEvidencePack.blockers.map((blocker) => `QA evidence pack: ${blocker}`),
  ...signingReadiness.blockers.map((blocker) => `signing readiness: ${blocker}`),
];

if (!release.present) {
  publicStableBlockers.push(`missing release evidence: ${releaseEvidencePath}`);
}
if (!release.evidenceMatchesCurrent) {
  publicStableBlockers.push('release evidence does not match current artifacts');
}
if (!release.releaseVerifyPassed) {
  publicStableBlockers.push('release verification has not passed');
}
if (!release.releaseArtifactsNewerThanSource) {
  publicStableBlockers.push('release artifacts are not proven newer than source');
}

const decisions = {
  internalTrial: release.currentArtifactsPresent && matrix.failedRequired.length === 0 ? 'Go' : 'No-Go',
  smallCommercialPilot:
    release.currentArtifactsPresent && matrix.failedRequired.length === 0 ? 'Go with unsigned notice' : 'No-Go',
  publicStableRelease: publicStableBlockers.length === 0 ? 'Go' : 'No-Go',
};

const evidence = {
  schema: 'nexspot.manual-qa-gate.v1',
  generatedAt: new Date().toISOString(),
  inputs: {
    matrix: matrixPath,
    issueLog: issueLogPath,
    releaseEvidence: releaseEvidencePath,
    qaEvidencePack: qaEvidencePackPath,
    signingReadiness: signingReadinessPath,
  },
  decisions,
  matrix: {
    totalRows: matrix.totalRows,
    requiredRows: matrix.requiredRows,
    passedRequired: matrix.passedRequired,
    pendingRequired: matrix.pendingRequired,
    failedRequired: matrix.failedRequired,
    requiredWithoutEvidence: matrix.requiredWithoutEvidence,
    requiredWithMissingEvidenceFiles: matrix.requiredWithMissingEvidenceFiles,
    byStatus: matrix.byStatus,
  },
  issueLog: {
    totalItems: issueLog.totalItems,
    openP0: issueLog.openP0,
    openP1: issueLog.openP1,
  },
  release,
  qaEvidencePack,
  signingReadiness,
  publicStableBlockers,
};

console.log('NexSpot manual QA gate');
console.log('='.repeat(26));
console.log(`internal trial: ${decisions.internalTrial}`);
console.log(`small commercial pilot: ${decisions.smallCommercialPilot}`);
console.log(`public stable release: ${decisions.publicStableRelease}`);
console.log(`required matrix rows: ${matrix.requiredRows}`);
console.log(`pending required rows: ${matrix.pendingRequired.length}`);
console.log(`failed required rows: ${matrix.failedRequired.length}`);
console.log(`open P0 blockers: ${issueLog.openP0.length}`);
console.log(`open P1 polish blockers: ${issueLog.openP1.length}`);
console.log(`unsigned artifacts: ${release.unsignedArtifacts.length}`);
console.log(`QA evidence pack: ${qaEvidencePack.blockers.length === 0 ? 'current' : 'needs refresh'}`);
console.log(`signing readiness: ${signingReadiness.blockers.length === 0 ? 'public-stable ready' : 'not ready'}`);

if (publicStableBlockers.length > 0) {
  console.log('\nPublic stable blockers');
  for (const blocker of publicStableBlockers) {
    console.log(`- ${blocker}`);
  }
}

if (evidenceRelativePath) {
  const evidencePath = path.join(root, evidenceRelativePath);
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
  console.log(`\nManual QA gate evidence written: ${evidenceRelativePath}`);
}

if (strict && publicStableBlockers.length > 0) {
  process.exit(1);
}
