import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const writeManifestIndex = process.argv.indexOf('--write-manifest');
const manifestRelativePath =
  writeManifestIndex >= 0
    ? process.argv[writeManifestIndex + 1] || 'docs/19-PILOT-DISTRIBUTION-MANIFEST.json'
    : null;
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => parseJsonText(fs.readFileSync(path.join(root, relativePath), 'utf8'));
const readText = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8');

const hashFile = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(fullPath));
  return hash.digest('hex');
};

const statFile = (relativePath) => {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return {
      path: relativePath,
      exists: false,
      bytes: 0,
      sha256: '',
    };
  }
  const stat = fs.statSync(fullPath);
  return {
    path: relativePath,
    exists: true,
    bytes: stat.size,
    modifiedAt: stat.mtime.toISOString(),
    sha256: hashFile(relativePath),
  };
};

const packageJson = readJson('package.json');
const releaseEvidence = readJson('docs/11-RELEASE-EVIDENCE.json');
const manualGate = readJson('docs/17-MANUAL-QA-GATE.json');
const signingReadiness = readJson('docs/18-SIGNING-READINESS.json');
const unsignedNoticeText = readText('docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md');

const exePath = 'src-tauri/target/release/nexspot.exe';
const installerPath = `src-tauri/target/release/bundle/nsis/NexSpot_${packageJson.version}_x64-setup.exe`;

const artifacts = {
  exe: statFile(exePath),
  nsisInstaller: statFile(installerPath),
};
const releaseEvidenceMatchesCurrent =
  artifacts.exe.sha256 === releaseEvidence.artifacts?.exe?.sha256 &&
  artifacts.nsisInstaller.sha256 === releaseEvidence.artifacts?.nsisInstaller?.sha256;

const materials = [
  'docs/11-RELEASE-EVIDENCE.json',
  'docs/12-MANUAL-QA-MATRIX.md',
  'docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md',
  'docs/16-COMMERCIAL-QA-ISSUE-LOG.md',
  'docs/17-MANUAL-QA-GATE.json',
  'docs/18-SIGNING-READINESS.json',
  'docs/18-SIGNING-READINESS.md',
  'docs/qa-runs/latest/qa-run.md',
  'docs/qa-runs/latest/environment.json',
  'docs/qa-runs/latest/qa-evidence-pack.json',
  'docs/qa-runs/latest/results-template.json',
].map(statFile);

const blockers = [];

if (!unsignedNoticeText.includes(artifacts.exe.sha256)) {
  blockers.push('Unsigned pilot notice does not include current EXE hash.');
}
if (!unsignedNoticeText.includes(artifacts.nsisInstaller.sha256)) {
  blockers.push('Unsigned pilot notice does not include current installer hash.');
}
if (manualGate.decisions?.internalTrial !== 'Go') {
  blockers.push('Manual gate does not allow internal trial.');
}
if (manualGate.decisions?.smallCommercialPilot !== 'Go with unsigned notice') {
  blockers.push('Manual gate does not allow small commercial pilot with unsigned notice.');
}
if (manualGate.signingReadiness?.unsignedPilotReady !== true) {
  blockers.push('Manual gate does not confirm unsigned pilot signing readiness.');
}
if (signingReadiness.decisions?.unsignedPilotReady !== true) {
  blockers.push('Signing readiness does not confirm unsigned pilot readiness.');
}
for (const material of materials) {
  if (!material.exists) {
    blockers.push(`Missing distribution material: ${material.path}`);
  }
}

const manifest = {
  schema: 'nexspot.pilot-distribution-manifest.v1',
  generatedAt: new Date().toISOString(),
  productName: 'NexSpot',
  version: packageJson.version,
  channel: 'internal-trial-and-small-commercial-pilot',
  decisions: {
    internalTrial: blockers.length === 0 ? 'Go' : 'No-Go',
    smallCommercialPilot: blockers.length === 0 ? 'Go with unsigned notice' : 'No-Go',
    publicStableRelease: 'No-Go',
  },
  artifacts,
  releaseEvidenceMatchesCurrent,
  materials,
  requiredRecipientNotice: {
    path: 'docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md',
    mustMentionUnknownPublisher: true,
    mustMentionSha256Verification: true,
  },
  blockers,
};

console.log('NexSpot pilot distribution manifest');
console.log('='.repeat(37));
console.log(`internal trial: ${manifest.decisions.internalTrial}`);
console.log(`small commercial pilot: ${manifest.decisions.smallCommercialPilot}`);
console.log(`public stable release: ${manifest.decisions.publicStableRelease}`);
console.log(`materials: ${materials.length}`);
if (blockers.length > 0) {
  console.log('\nBlockers');
  for (const blocker of blockers) {
    console.log(`- ${blocker}`);
  }
}

if (manifestRelativePath) {
  const manifestPath = path.join(root, manifestRelativePath);
  fs.mkdirSync(path.dirname(manifestPath), { recursive: true });
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  console.log(`\nPilot distribution manifest written: ${manifestRelativePath}`);
}
