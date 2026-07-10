import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const strict = process.argv.includes('--strict');
const writeEvidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  writeEvidenceIndex >= 0
    ? process.argv[writeEvidenceIndex + 1] || 'docs/20-COMMERCIAL-READINESS.json'
    : null;

const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return { missing: true, path: relativePath };
  }
  return parseJsonText(fs.readFileSync(fullPath, 'utf8'));
};

const allTrue = (value) =>
  value && Object.values(value).every((item) => item === true);

const releaseEvidence = readJson('docs/11-RELEASE-EVIDENCE.json');
const manualGate = readJson('docs/17-MANUAL-QA-GATE.json');
const pilotManifest = readJson('docs/19-PILOT-DISTRIBUTION-MANIFEST.json');
const signingReadiness = readJson('docs/18-SIGNING-READINESS.json');
const qaPack = readJson('docs/qa-runs/latest/qa-evidence-pack.json');

const missingInputs = [
  releaseEvidence,
  manualGate,
  pilotManifest,
  signingReadiness,
  qaPack,
]
  .filter((item) => item?.missing)
  .map((item) => item.path);

const releaseGatesPassed = allTrue(releaseEvidence.automatedGates);
const commercialSourceGatePassed = allTrue(releaseEvidence.commercialSourceGate);
const noPilotBlockers = (pilotManifest.blockers || []).length === 0;
const pendingRequired = manualGate.matrix?.pendingRequired || [];
const failedRequired = manualGate.matrix?.failedRequired || [];
const openP0 = manualGate.issueLog?.openP0 || [];
const openP1 = manualGate.issueLog?.openP1 || [];
const unsignedArtifacts = manualGate.release?.unsignedArtifacts || [];
const signingBlockers = signingReadiness.blockers || [];
const qaPackMatchesCurrent =
  manualGate.qaEvidencePack?.matchesCurrentArtifacts === true ||
  (
    qaPack.artifacts?.exe?.sha256 === releaseEvidence.artifacts?.exe?.sha256 &&
    qaPack.artifacts?.nsisInstaller?.sha256 === releaseEvidence.artifacts?.nsisInstaller?.sha256
  );

const internalTrialReady =
  missingInputs.length === 0 &&
  releaseGatesPassed &&
  commercialSourceGatePassed &&
  manualGate.decisions?.internalTrial === 'Go' &&
  pilotManifest.decisions?.internalTrial === 'Go' &&
  qaPackMatchesCurrent &&
  noPilotBlockers;

const smallCommercialPilotReady =
  internalTrialReady &&
  manualGate.decisions?.smallCommercialPilot === 'Go with unsigned notice' &&
  pilotManifest.decisions?.smallCommercialPilot === 'Go with unsigned notice' &&
  signingReadiness.decisions?.unsignedPilotReady === true;

const publicStableReady =
  internalTrialReady &&
  pendingRequired.length === 0 &&
  failedRequired.length === 0 &&
  openP0.length === 0 &&
  openP1.length === 0 &&
  unsignedArtifacts.length === 0 &&
  signingReadiness.decisions?.publicStableReady === true &&
  manualGate.decisions?.publicStableRelease === 'Go' &&
  pilotManifest.decisions?.publicStableRelease === 'Go';

const summary = {
  schema: 'nexspot.commercial-readiness.v1',
  generatedAt: new Date().toISOString(),
  version: releaseEvidence.version || null,
  artifacts: {
    exe: releaseEvidence.artifacts?.exe || null,
    nsisInstaller: releaseEvidence.artifacts?.nsisInstaller || null,
  },
  decisions: {
    internalTrialReady,
    smallCommercialPilotReady,
    publicStableReady,
  },
  release: {
    releaseGatesPassed,
    commercialSourceGatePassed,
    releaseVerifyPassed: releaseEvidence.automatedGates?.releaseVerifyPassed === true,
    warnings: releaseEvidence.warnings || [],
  },
  manualQa: {
    pendingRequiredCount: pendingRequired.length,
    pendingRequired,
    failedRequired,
    openP0,
    openP1,
    qaPackMatchesCurrent,
  },
  signing: {
    unsignedPilotReady: signingReadiness.decisions?.unsignedPilotReady === true,
    publicStableReady: signingReadiness.decisions?.publicStableReady === true,
    unsignedArtifacts,
    blockers: signingBlockers,
  },
  pilotDistribution: {
    decisions: pilotManifest.decisions || {},
    blockers: pilotManifest.blockers || [],
    materialCount: Array.isArray(pilotManifest.materials) ? pilotManifest.materials.length : 0,
  },
  missingInputs,
};

if (evidenceRelativePath) {
  const evidencePath = path.join(root, evidenceRelativePath);
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(summary, null, 2)}\n`, 'utf8');
}

console.log('NexSpot commercial readiness');
console.log('============================');
console.log(`version: ${summary.version || 'unknown'}`);
console.log(`internal trial: ${internalTrialReady ? 'Go' : 'No-Go'}`);
console.log(`small commercial pilot: ${smallCommercialPilotReady ? 'Go with unsigned notice' : 'No-Go'}`);
console.log(`public stable release: ${publicStableReady ? 'Go' : 'No-Go'}`);

if (summary.artifacts.exe) {
  console.log(`release exe: ${summary.artifacts.exe.path}`);
  console.log(`release exe sha256: ${summary.artifacts.exe.sha256}`);
}
if (summary.artifacts.nsisInstaller) {
  console.log(`nsis installer: ${summary.artifacts.nsisInstaller.path}`);
  console.log(`nsis installer sha256: ${summary.artifacts.nsisInstaller.sha256}`);
}

const blockers = [
  ...missingInputs.map((item) => `missing input: ${item}`),
  ...pendingRequired.map((item) => `manual QA pending: ${item}`),
  ...failedRequired.map((item) => `manual QA failed: ${item}`),
  ...openP0.map((item) => `open P0: ${item}`),
  ...openP1.map((item) => `open P1 polish: ${item}`),
  ...unsignedArtifacts.map((item) => `unsigned artifact: ${item.name || item.path}`),
  ...signingBlockers.map((item) => `signing: ${item}`),
  ...(pilotManifest.blockers || []).map((item) => `pilot manifest: ${item}`),
];

if (blockers.length > 0) {
  console.log('');
  console.log('Public-stable blockers');
  for (const blocker of blockers) {
    console.log(`- ${blocker}`);
  }
}

if (evidenceRelativePath) {
  console.log('');
  console.log(`commercial readiness evidence written: ${evidenceRelativePath}`);
}

if (missingInputs.length > 0 || !internalTrialReady || !smallCommercialPilotReady) {
  process.exit(1);
}

if (strict && !publicStableReady) {
  process.exit(1);
}
