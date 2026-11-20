import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const writeJsonIndex = process.argv.indexOf('--write-json');
const writeMdIndex = process.argv.indexOf('--write-md');
const jsonRelativePath =
  writeJsonIndex >= 0 ? process.argv[writeJsonIndex + 1] || 'docs/21-COMMERCIAL-CLOSURE-PLAN.json' : null;
const mdRelativePath =
  writeMdIndex >= 0 ? process.argv[writeMdIndex + 1] || 'docs/21-COMMERCIAL-CLOSURE-PLAN.md' : null;
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => parseJsonText(fs.readFileSync(path.join(root, relativePath), 'utf8'));
const readText = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8');
const writeText = (relativePath, text) => {
  const fullPath = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(fullPath), { recursive: true });
  fs.writeFileSync(fullPath, text, 'utf8');
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

const expandRelatedIds = (value) =>
  String(value || '')
    .split(',')
    .flatMap((chunk) => {
      const trimmed = chunk.trim();
      const rangeMatch = /^([A-Z]+)-(\d{2})\.\.([A-Z]+)-(\d{2})$/.exec(trimmed);
      if (!rangeMatch || rangeMatch[1] !== rangeMatch[3]) {
        return trimmed ? [trimmed] : [];
      }
      const [, prefix, startText, , endText] = rangeMatch;
      const ids = [];
      for (let id = Number(startText); id <= Number(endText); id += 1) {
        ids.push(`${prefix}-${String(id).padStart(2, '0')}`);
      }
      return ids;
    });

const matrixText = readText('docs/12-MANUAL-QA-MATRIX.md');
const issueLogText = readText('docs/16-COMMERCIAL-QA-ISSUE-LOG.md');
const gate = readJson('docs/17-MANUAL-QA-GATE.json');
const readiness = readJson('docs/20-COMMERCIAL-READINESS.json');
const signing = readJson('docs/18-SIGNING-READINESS.json');

const matrixItems = parseTableRows(matrixText, /^(ENV|CAP|OCR|SCR|PIN|REL)-\d{2}$/).map((cells) => ({
  id: cells[0],
  title: cells[1] || '',
  required: cells[2] || '',
  status: cells[3] || '',
  evidence: cells[4] || '',
  notes: cells[5] || '',
}));
const matrixById = new Map(matrixItems.map((item) => [item.id, item]));

const issueItems = parseTableRows(issueLogText, /^QA-\d{3}$/).map((cells) => ({
  id: cells[0],
  severity: cells[1] || '',
  area: cells[2] || '',
  relatedMatrixIds: cells[3] || '',
  status: cells[4] || '',
  owner: cells[5] || '',
  closeCondition: cells[6] || '',
}));
const openIssueIds = new Set([...(gate.issueLog?.openP0 || []), ...(gate.issueLog?.openP1 || [])]);
const openIssues = issueItems.filter((item) => openIssueIds.has(item.id));

const issuePlans = openIssues.map((issue) => {
  const relatedIds = expandRelatedIds(issue.relatedMatrixIds);
  const rows = relatedIds.map((id) => matrixById.get(id)).filter(Boolean);
  return {
    id: issue.id,
    severity: issue.severity,
    area: issue.area,
    status: issue.status,
    owner: issue.owner,
    closeCondition: issue.closeCondition,
    relatedMatrixIds: relatedIds,
    pendingRows: rows
      .filter((row) => row.status === 'Pending')
      .map((row) => ({ id: row.id, title: row.title })),
    evidenceNeeded: rows.map((row) => `${row.id}: ${row.title}`),
  };
});

const decisions = readiness.decisions || {};
const closurePlan = {
  schema: 'nexspot.commercial-closure-plan.v1',
  generatedAt: new Date().toISOString(),
  version: readiness.version || '',
  decisions,
  artifactHashes: {
    exe: readiness.artifacts?.exe?.sha256 || gate.release?.hashes?.exe || '',
    nsisInstaller:
      readiness.artifacts?.nsisInstaller?.sha256 || gate.release?.hashes?.nsisInstaller || '',
  },
  counts: {
    pendingRequired: gate.matrix?.pendingRequired?.length || 0,
    failedRequired: gate.matrix?.failedRequired?.length || 0,
    openP0: gate.issueLog?.openP0?.length || 0,
    openP1: gate.issueLog?.openP1?.length || 0,
    unsignedArtifacts: gate.release?.unsignedArtifacts?.length || 0,
  },
  nextPhases: [
    {
      phase: 'Real-device QA',
      exitCriteria:
        'All required ENV/CAP/OCR/SCR rows are Passed with repository evidence files and no failed required rows.',
      commands: [
        'node scripts\\qa-evidence-pack.mjs --out-dir docs\\qa-runs\\<machine-or-run-label>',
        'node scripts\\qa-merge-results.mjs --results docs\\qa-runs\\<machine-or-run-label>\\results-template.json',
        'node scripts\\qa-merge-results.mjs --results docs\\qa-runs\\<machine-or-run-label>\\results-template.json --apply',
        'npm.cmd run qa:manual:gate',
      ],
    },
    {
      phase: 'Signing',
      exitCriteria:
        'EXE and NSIS installer are Authenticode signed, signer subject/thumbprint are pinned, and signing readiness is public-stable ready.',
      commands: [
        'node scripts\\signing-readiness.mjs --write-evidence docs\\18-SIGNING-READINESS.json',
        'npm.cmd run qa:manual:gate:strict',
      ],
    },
    {
      phase: 'Final release evidence',
      exitCriteria:
        'Release evidence, pilot manifest, and commercial readiness all match current artifact hashes with public stable Go.',
      commands: [
        'npm.cmd run release:evidence',
        'node scripts\\pilot-distribution-manifest.mjs --write-manifest docs\\19-PILOT-DISTRIBUTION-MANIFEST.json',
        'node scripts\\commercial-readiness.mjs --strict --write-evidence docs\\20-COMMERCIAL-READINESS.json',
      ],
    },
  ],
  openIssues: issuePlans,
  signingBlockers: signing.blockers || [],
  publicStableBlockers: gate.publicStableBlockers || [],
};

const status = decisions.publicStableReady ? 'Go' : 'No-Go';
const markdown = `# NexSpot Commercial Closure Plan

Generated: ${closurePlan.generatedAt}

## Decision

| Level | Status |
|---|---|
| Internal trial | ${decisions.internalTrialReady ? 'Go' : 'No-Go'} |
| Small commercial pilot | ${decisions.smallCommercialPilotReady ? 'Go with unsigned notice' : 'No-Go'} |
| Public stable | ${status} |

## Remaining Counts

| Item | Count |
|---|---:|
| Pending required QA rows | ${closurePlan.counts.pendingRequired} |
| Failed required QA rows | ${closurePlan.counts.failedRequired} |
| Open P0 blockers | ${closurePlan.counts.openP0} |
| Open P1 polish blockers | ${closurePlan.counts.openP1} |
| Unsigned artifacts | ${closurePlan.counts.unsignedArtifacts} |

## Open Issues

| ID | Severity | Area | Owner | Pending evidence rows |
|---|---|---|---|---|
${issuePlans
  .map(
    (issue) =>
      `| ${issue.id} | ${issue.severity} | ${issue.area} | ${issue.owner} | ${
        issue.pendingRows.map((row) => row.id).join(', ') || 'None'
      } |`,
  )
  .join('\n')}

## Next Phases

${closurePlan.nextPhases
  .map(
    (phase, index) => `### ${index + 1}. ${phase.phase}

Exit criteria: ${phase.exitCriteria}

Commands:

${phase.commands.map((command) => `- \`${command}\``).join('\n')}
`,
  )
  .join('\n')}

## Public Stable Blockers

${closurePlan.publicStableBlockers.map((blocker) => `- ${blocker}`).join('\n') || '- None'}
`;

console.log('NexSpot commercial closure plan');
console.log('='.repeat(32));
console.log(`public stable: ${status}`);
console.log(`pending required QA rows: ${closurePlan.counts.pendingRequired}`);
console.log(`open P0 blockers: ${closurePlan.counts.openP0}`);
console.log(`open P1 polish blockers: ${closurePlan.counts.openP1}`);
console.log(`unsigned artifacts: ${closurePlan.counts.unsignedArtifacts}`);

if (jsonRelativePath) {
  writeText(jsonRelativePath, `${JSON.stringify(closurePlan, null, 2)}\n`);
  console.log(`closure plan JSON written: ${jsonRelativePath}`);
}

if (mdRelativePath) {
  writeText(mdRelativePath, markdown);
  console.log(`closure plan Markdown written: ${mdRelativePath}`);
}
