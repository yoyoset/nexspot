import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const writeEvidenceIndex = process.argv.indexOf('--write-evidence');
const evidenceRelativePath =
  writeEvidenceIndex >= 0 ? process.argv[writeEvidenceIndex + 1] || 'docs/18-SIGNING-READINESS.json' : null;
const packageJson = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const artifacts = [
  {
    key: 'exe',
    label: 'Release EXE',
    path: 'src-tauri/target/release/nexspot.exe',
  },
  {
    key: 'nsisInstaller',
    label: 'NSIS installer',
    path: `src-tauri/target/release/bundle/nsis/NexSpot_${packageJson.version}_x64-setup.exe`,
  },
];

const quotePowerShellLiteral = (value) => `'${value.replaceAll("'", "''")}'`;

const hashFile = (fullPath) => {
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(fullPath));
  return hash.digest('hex');
};

const getAuthenticodeStatus = (fullPath) => {
  if (process.platform !== 'win32') {
    return {
      status: 'Skipped',
      subject: '',
      thumbprint: '',
      message: 'Authenticode check skipped because this is not Windows.',
    };
  }

  const command = [
    '[Console]::OutputEncoding = [System.Text.Encoding]::UTF8',
    `$sig = Get-AuthenticodeSignature -LiteralPath ${quotePowerShellLiteral(fullPath)}`,
    '[pscustomobject]@{Status=$sig.Status.ToString(); Subject=if($sig.SignerCertificate){$sig.SignerCertificate.Subject}else{""}; Thumbprint=if($sig.SignerCertificate){$sig.SignerCertificate.Thumbprint}else{""}} | ConvertTo-Json -Compress',
  ].join('; ');

  const result = spawnSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command], {
    encoding: 'utf8',
    windowsHide: true,
  });

  if (result.error || result.status !== 0) {
    return {
      status: 'Unknown',
      subject: '',
      thumbprint: '',
      message: result.error?.message || result.stderr?.trim() || `powershell exited ${result.status}`,
    };
  }

  try {
    const parsed = JSON.parse(result.stdout.trim());
    return {
      status: String(parsed.Status || 'Unknown'),
      subject: String(parsed.Subject || ''),
      thumbprint: String(parsed.Thumbprint || ''),
      message: '',
    };
  } catch {
    return {
      status: 'Unknown',
      subject: '',
      thumbprint: '',
      message: 'Unable to parse Authenticode output.',
    };
  }
};

const expectedSubject = process.env.NEXSPOT_SIGNING_SUBJECT || '';
const expectedThumbprint = process.env.NEXSPOT_SIGNING_THUMBPRINT || '';

const artifactEvidence = artifacts.map((artifact) => {
  const fullPath = path.join(root, artifact.path);
  const exists = fs.existsSync(fullPath);
  const stat = exists ? fs.statSync(fullPath) : null;
  const signature = exists ? getAuthenticodeStatus(fullPath) : null;
  return {
    ...artifact,
    exists,
    bytes: stat?.size || 0,
    modifiedAt: stat?.mtime.toISOString() || '',
    sha256: exists ? hashFile(fullPath) : '',
    signature,
    expectedSubjectMatches: expectedSubject ? signature?.subject.includes(expectedSubject) === true : null,
    expectedThumbprintMatches: expectedThumbprint
      ? signature?.thumbprint.toLowerCase() === expectedThumbprint.toLowerCase()
      : null,
  };
});

const allArtifactsPresent = artifactEvidence.every((artifact) => artifact.exists);
const allArtifactsSigned = artifactEvidence.every((artifact) => artifact.signature?.status === 'Valid');
const expectedSubjectMatches = expectedSubject
  ? artifactEvidence.every((artifact) => artifact.expectedSubjectMatches === true)
  : null;
const expectedThumbprintMatches = expectedThumbprint
  ? artifactEvidence.every((artifact) => artifact.expectedThumbprintMatches === true)
  : null;
const publicStableReady =
  allArtifactsPresent &&
  allArtifactsSigned &&
  (expectedSubjectMatches !== false) &&
  (expectedThumbprintMatches !== false);
const unsignedPilotReady =
  allArtifactsPresent && artifactEvidence.every((artifact) => artifact.signature?.status === 'NotSigned');

const blockers = [];
if (!allArtifactsPresent) {
  blockers.push('One or more release artifacts are missing.');
}
for (const artifact of artifactEvidence) {
  if (artifact.signature?.status !== 'Valid') {
    blockers.push(`${artifact.label} Authenticode status is ${artifact.signature?.status || 'Unknown'}.`);
  }
  if (artifact.expectedSubjectMatches === false) {
    blockers.push(`${artifact.label} signer subject does not include NEXSPOT_SIGNING_SUBJECT.`);
  }
  if (artifact.expectedThumbprintMatches === false) {
    blockers.push(`${artifact.label} signer thumbprint does not match NEXSPOT_SIGNING_THUMBPRINT.`);
  }
}
if (!expectedSubject) {
  blockers.push('NEXSPOT_SIGNING_SUBJECT is not set; signer identity is not pinned for public-stable verification.');
}
if (!expectedThumbprint) {
  blockers.push('NEXSPOT_SIGNING_THUMBPRINT is not set; certificate thumbprint is not pinned for public-stable verification.');
}

const evidence = {
  schema: 'nexspot.signing-readiness.v1',
  generatedAt: new Date().toISOString(),
  version: packageJson.version,
  expectedSubject: expectedSubject || null,
  expectedThumbprint: expectedThumbprint || null,
  decisions: {
    unsignedPilotReady,
    publicStableReady,
  },
  artifacts: artifactEvidence,
  blockers,
  nextActions: publicStableReady
    ? [
        'Run npm.cmd run release:verify.',
        'Run npm.cmd run qa:manual:gate:strict after real-device QA is complete.',
      ]
    : [
        'Acquire or select the production Authenticode code-signing certificate.',
        'Sign both the release EXE and NSIS installer.',
        'Set NEXSPOT_SIGNING_SUBJECT and NEXSPOT_SIGNING_THUMBPRINT for public-stable verification.',
        'Rerun node scripts\\signing-readiness.mjs --write-evidence docs\\18-SIGNING-READINESS.json.',
        'Rerun npm.cmd run qa:manual:gate and npm.cmd run release:evidence.',
      ],
};

console.log('NexSpot signing readiness');
console.log('='.repeat(28));
console.log(`unsigned pilot ready: ${unsignedPilotReady ? 'Yes' : 'No'}`);
console.log(`public stable ready: ${publicStableReady ? 'Yes' : 'No'}`);
for (const artifact of artifactEvidence) {
  console.log(`${artifact.label}: ${artifact.signature?.status || 'missing'} ${artifact.sha256 || ''}`);
}
if (blockers.length > 0) {
  console.log('\nBlockers');
  for (const blocker of blockers) {
    console.log(`- ${blocker}`);
  }
}

if (evidenceRelativePath) {
  const evidencePath = path.join(root, evidenceRelativePath);
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
  console.log(`\nSigning readiness evidence written: ${evidenceRelativePath}`);
}
