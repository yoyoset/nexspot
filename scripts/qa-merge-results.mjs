import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const apply = process.argv.includes('--apply');
const resultsIndex = process.argv.indexOf('--results');
const resultsRelativePath =
  resultsIndex >= 0 ? process.argv[resultsIndex + 1] : 'docs/qa-runs/latest/results-template.json';
const matrixPath = 'docs/12-MANUAL-QA-MATRIX.md';
const issueLogPath = 'docs/16-COMMERCIAL-QA-ISSUE-LOG.md';
const gatePath = 'docs/17-MANUAL-QA-GATE.json';

const allowedStatuses = new Set(['Pending', 'Passed', 'Failed', 'Blocked']);
const allowedIssueSeverities = new Set(['P0', 'P1', 'P2']);
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const readJson = (relativePath) => parseJsonText(fs.readFileSync(path.join(root, relativePath), 'utf8'));
const readText = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8');
const writeText = (relativePath, text) => fs.writeFileSync(path.join(root, relativePath), text, 'utf8');

const normalizePath = (value) => String(value || '').replaceAll('\\', '/');
const splitEvidencePaths = (value) =>
  normalizePath(value)
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

const parseMatrixRows = (text) => {
  const rows = new Map();
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
    if (/^(ENV|CAP|OCR|SCR|PIN|REL)-\d{2}$/.test(id)) {
      rows.set(id, {
        id,
        cells,
        required: cells[2] === 'Yes',
        status: cells[3] || '',
      });
    }
  }
  return rows;
};

const replaceMatrixRows = (text, updates) =>
  text
    .split(/\r?\n/)
    .map((line) => {
      const trimmed = line.trim();
      if (!trimmed.startsWith('|') || trimmed.includes('---')) {
        return line;
      }
      const cells = trimmed
        .split('|')
        .slice(1, -1)
        .map((cell) => cell.trim());
      const id = cells[0] || '';
      const update = updates.get(id);
      if (!update) {
        return line;
      }

      const nextCells = [...cells];
      nextCells[3] = update.status;
      nextCells[4] = update.evidence;
      nextCells[5] = update.notes;
      return `| ${nextCells.join(' | ')} |`;
    })
    .join('\n');

const safeCell = (value) => String(value || '').replaceAll('|', '/').trim();

const formatIssueRow = (issue) =>
  `| ${[
    issue.id,
    issue.severity,
    issue.area,
    issue.relatedMatrixIds,
    issue.status,
    issue.owner,
    issue.closeCondition,
  ].map(safeCell).join(' | ')} |`;

const parseIssueRows = (text) => {
  const rows = new Map();
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
    if (/^QA-[A-Z0-9-]+$/.test(id)) {
      rows.set(id, { id, line, cells });
    }
  }
  return rows;
};

const replaceOrInsertIssueRows = (text, issueUpdates) => {
  const remaining = new Map(issueUpdates);
  const replaced = text
    .split(/\r?\n/)
    .map((line) => {
      const trimmed = line.trim();
      if (!trimmed.startsWith('|') || trimmed.includes('---')) {
        return line;
      }
      const cells = trimmed
        .split('|')
        .slice(1, -1)
        .map((cell) => cell.trim());
      const id = cells[0] || '';
      const update = remaining.get(id);
      if (!update) {
        return line;
      }
      remaining.delete(id);
      return formatIssueRow(update);
    })
    .join('\n');

  if (remaining.size === 0) {
    return replaced;
  }

  const lines = replaced.split(/\r?\n/);
  const insertAfterTable = (sectionHeading, issues) => {
    if (issues.length === 0) {
      return;
    }

    const sectionIndex = lines.findIndex((line) => line.trim() === sectionHeading);
    if (sectionIndex < 0) {
      failures.push(`Unable to find issue-log section ${sectionHeading}.`);
      return;
    }

    let insertIndex = -1;
    for (let index = sectionIndex + 1; index < lines.length; index += 1) {
      const trimmed = lines[index].trim();
      if (trimmed.startsWith('## ') && index > sectionIndex + 1) {
        break;
      }
      if (trimmed.startsWith('|') && !trimmed.includes('---')) {
        insertIndex = index;
      }
    }
    if (insertIndex < 0) {
      failures.push(`Unable to find insertion point in ${sectionHeading}.`);
      return;
    }

    lines.splice(insertIndex + 1, 0, ...issues.map(formatIssueRow));
  };

  const p0Issues = [...remaining.values()].filter((issue) => issue.severity === 'P0');
  const nonP0Issues = [...remaining.values()].filter((issue) => issue.severity !== 'P0');
  insertAfterTable('## Public Stable Blockers', p0Issues);
  insertAfterTable('## Pilot Watch Items', nonP0Issues);

  return lines.join('\n');
};

const results = readJson(resultsRelativePath);
const gate = readJson(gatePath);
const matrixText = readText(matrixPath);
const issueLogText = readText(issueLogPath);
const matrixRows = parseMatrixRows(matrixText);
const issueLogRows = parseIssueRows(issueLogText);
const failures = [];
const warnings = [];
const updates = new Map();
const issueUpdates = new Map();
const failedOrBlockedResultIds = [];

if (results.schema !== 'nexspot.qa-results.v1') {
  failures.push('Results schema must be nexspot.qa-results.v1.');
}

if (results.artifacts?.exe?.sha256 !== gate.release?.hashes?.exe) {
  failures.push('Results EXE hash does not match current manual gate release hash.');
}

if (results.artifacts?.nsisInstaller?.sha256 !== gate.release?.hashes?.nsisInstaller) {
  failures.push('Results installer hash does not match current manual gate release hash.');
}

if (!Array.isArray(results.rows)) {
  failures.push('Results file must contain a rows array.');
}

for (const row of results.rows || []) {
  const id = String(row.id || '').trim();
  const status = String(row.status || '').trim();
  const evidence = normalizePath(row.evidence);
  const notes = String(row.notes || '').trim();
  const matrixRow = matrixRows.get(id);

  if (!matrixRow) {
    failures.push(`Unknown matrix row id: ${id || '(missing)'}.`);
    continue;
  }
  if (!allowedStatuses.has(status)) {
    failures.push(`Invalid status for ${id}: ${status || '(missing)'}. Use Pending, Passed, Failed, or Blocked.`);
    continue;
  }
  if (status === 'Passed' && matrixRow.required && !evidence) {
    failures.push(`Required row ${id} cannot be marked Passed without an evidence path.`);
  }
  if (status === 'Passed' && matrixRow.required) {
    for (const evidencePath of splitEvidencePaths(evidence)) {
      if (!evidencePathExists(evidencePath)) {
        failures.push(`Required row ${id} evidence file does not exist: ${evidencePath}`);
      }
    }
  }
  if ((status === 'Failed' || status === 'Blocked') && !notes) {
    failures.push(`${id} is ${status}; notes must include a short reproduction or blocker summary.`);
  }
  if ((status === 'Failed' || status === 'Blocked') && !Array.isArray(results.failedIssueRows)) {
    failures.push(`${id} is ${status}; results failedIssueRows must be present for issue-log follow-up.`);
  }
  if (status === 'Failed' || status === 'Blocked') {
    failedOrBlockedResultIds.push(id);
  }
  if (status !== 'Pending') {
    updates.set(id, {
      status,
      evidence,
      notes,
    });
  }
}

if (results.failedIssueRows !== undefined && !Array.isArray(results.failedIssueRows)) {
  failures.push('failedIssueRows must be an array when present.');
}

for (const issue of results.failedIssueRows || []) {
  const normalizedIssue = {
    id: String(issue.id || '').trim(),
    severity: String(issue.severity || '').trim(),
    area: String(issue.area || '').trim(),
    relatedMatrixIds: String(issue.relatedMatrixIds || '').trim(),
    status: String(issue.status || 'Open').trim(),
    owner: String(issue.owner || '').trim(),
    closeCondition: String(issue.closeCondition || '').trim(),
  };

  if (!/^QA-[A-Z0-9-]+$/.test(normalizedIssue.id)) {
    failures.push(`Invalid failedIssueRows id: ${normalizedIssue.id || '(missing)'}.`);
    continue;
  }
  if (!allowedIssueSeverities.has(normalizedIssue.severity)) {
    failures.push(`Invalid severity for ${normalizedIssue.id}: ${normalizedIssue.severity || '(missing)'}.`);
  }
  if (!normalizedIssue.area) {
    failures.push(`Missing area for ${normalizedIssue.id}.`);
  }
  if (!normalizedIssue.relatedMatrixIds) {
    failures.push(`Missing relatedMatrixIds for ${normalizedIssue.id}.`);
  }
  if (!normalizedIssue.owner) {
    failures.push(`Missing owner for ${normalizedIssue.id}.`);
  }
  if (!normalizedIssue.closeCondition) {
    failures.push(`Missing closeCondition for ${normalizedIssue.id}.`);
  }
  if (!/^Open$/i.test(normalizedIssue.status) && !/^(Closed|Done)$/i.test(normalizedIssue.status)) {
    failures.push(`Invalid status for ${normalizedIssue.id}: ${normalizedIssue.status}.`);
  }
  issueUpdates.set(normalizedIssue.id, normalizedIssue);
}

for (const resultId of failedOrBlockedResultIds) {
  const hasIssue = [...issueUpdates.values()].some((issue) =>
    issue.relatedMatrixIds
      .split(',')
      .map((id) => id.trim())
      .includes(resultId),
  );
  if (!hasIssue) {
    failures.push(`Failed/Blocked row ${resultId} must have a failedIssueRows entry referencing it.`);
  }
}

for (const id of gate.matrix?.pendingRequired || []) {
  if (!matrixRows.has(id)) {
    failures.push(`Gate references missing matrix row: ${id}.`);
  }
}

if (updates.size === 0) {
  warnings.push('No non-Pending rows found; matrix will not change.');
}

for (const id of issueUpdates.keys()) {
  if (issueLogRows.has(id)) {
    warnings.push(`Issue ${id} already exists and will be replaced if --apply is used.`);
  }
}

console.log('NexSpot QA result merge');
console.log('='.repeat(25));
console.log(`results: ${resultsRelativePath}`);
console.log(`mode: ${apply ? 'apply' : 'dry-run'}`);
console.log(`candidate updates: ${updates.size}`);
console.log(`candidate issue updates: ${issueUpdates.size}`);

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

if (!apply) {
  console.log('\nDry-run passed. Re-run with --apply to update docs/12-MANUAL-QA-MATRIX.md.');
  process.exit(0);
}

const nextMatrixText = replaceMatrixRows(matrixText, updates);
const nextIssueLogText = replaceOrInsertIssueRows(issueLogText, issueUpdates);
if (failures.length > 0) {
  console.error('\nFailures');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

writeText(matrixPath, nextMatrixText);
writeText(issueLogPath, nextIssueLogText);
console.log(`\nUpdated ${matrixPath}.`);
if (issueUpdates.size > 0) {
  console.log(`Updated ${issueLogPath}.`);
}
console.log('Next: run npm.cmd run qa:manual:gate, then npm.cmd run release:evidence.');
