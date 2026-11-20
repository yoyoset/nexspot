import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const allowedStatuses = new Set(['Pending', 'Passed', 'Failed', 'Blocked']);
const allowedIssueSeverities = new Set(['P0', 'P1', 'P2']);
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));
const splitRelatedIds = (value) =>
  String(value || '')
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
const splitEvidencePaths = (value) =>
  String(value || '')
    .split(';')
    .map((item) => item.replaceAll('\\', '/').trim().replace(/^`+|`+$/g, ''))
    .filter(Boolean);

const argValue = (name, fallback = '') => {
  const index = process.argv.indexOf(name);
  if (index < 0) {
    return fallback;
  }
  return process.argv[index + 1] || fallback;
};

const usage = () => {
  console.error(`Usage:
node scripts\\qa-result-update.mjs --results docs\\qa-runs\\latest\\results-template.json --id SCR-01 --status Passed --evidence docs/qa-runs/latest/scr-01.png --notes "Chrome page captured"

For Failed or Blocked rows, also pass:
  --issue-id QA-009 --severity P0 --area "Scrolling capture" --owner "QA + Engineering" --close-condition "Attach fixed output image and rerun SCR-01"
`);
};

const resultsRelativePath = argValue('--results', 'docs/qa-runs/latest/results-template.json');
const id = argValue('--id').trim();
const status = argValue('--status').trim();
const evidence = argValue('--evidence').replaceAll('\\', '/').trim();
const notes = argValue('--notes').trim();
const issueId = argValue('--issue-id').trim();
const severity = argValue('--severity').trim();
const area = argValue('--area').trim();
const owner = argValue('--owner').trim();
const closeCondition = argValue('--close-condition').trim();

const failures = [];

if (!id) {
  failures.push('Missing --id.');
}
if (!allowedStatuses.has(status)) {
  failures.push(`Invalid --status: ${status || '(missing)'}. Use Pending, Passed, Failed, or Blocked.`);
}
if (status === 'Passed' && !evidence) {
  failures.push('Passed rows require --evidence.');
}
if ((status === 'Failed' || status === 'Blocked') && !notes) {
  failures.push(`${status} rows require --notes.`);
}
if (status === 'Failed' || status === 'Blocked') {
  if (!/^QA-[A-Z0-9-]+$/.test(issueId)) {
    failures.push('Failed/Blocked rows require --issue-id like QA-009.');
  }
  if (!allowedIssueSeverities.has(severity)) {
    failures.push('Failed/Blocked rows require --severity P0, P1, or P2.');
  }
  if (!area) {
    failures.push('Failed/Blocked rows require --area.');
  }
  if (!owner) {
    failures.push('Failed/Blocked rows require --owner.');
  }
  if (!closeCondition) {
    failures.push('Failed/Blocked rows require --close-condition.');
  }
}

const resultsPath = path.resolve(root, resultsRelativePath);
if (!resultsPath.toLowerCase().startsWith(`${root.toLowerCase()}${path.sep}`)) {
  failures.push(`Results file must be inside the repository: ${resultsRelativePath}`);
}
if (!fs.existsSync(resultsPath)) {
  failures.push(`Results file does not exist: ${resultsRelativePath}`);
}

for (const evidencePath of splitEvidencePaths(evidence)) {
  const resolvedEvidencePath = path.resolve(root, evidencePath);
  if (!resolvedEvidencePath.toLowerCase().startsWith(`${root.toLowerCase()}${path.sep}`)) {
    failures.push(`Evidence file must be inside the repository: ${evidencePath}`);
    continue;
  }
  if (!fs.existsSync(resolvedEvidencePath) || !fs.statSync(resolvedEvidencePath).isFile()) {
    failures.push(`Evidence file does not exist: ${evidencePath}`);
  }
}

if (failures.length > 0) {
  usage();
  console.error('Failures');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

const results = parseJsonText(fs.readFileSync(resultsPath, 'utf8'));
if (results.schema !== 'nexspot.qa-results.v1') {
  console.error('Results schema must be nexspot.qa-results.v1.');
  process.exit(1);
}
if (!Array.isArray(results.rows)) {
  console.error('Results file must contain a rows array.');
  process.exit(1);
}

const row = results.rows.find((item) => item.id === id);
if (!row) {
  console.error(`Unknown results row id: ${id}`);
  process.exit(1);
}

row.status = status;
row.evidence = status === 'Pending' ? '' : evidence;
row.notes = status === 'Pending' ? '' : notes;
results.generatedAt = new Date().toISOString();

if (!Array.isArray(results.failedIssueRows)) {
  results.failedIssueRows = [];
}

results.failedIssueRows = results.failedIssueRows.flatMap((issue) => {
  const remainingIds = splitRelatedIds(issue.relatedMatrixIds).filter((relatedId) => relatedId !== id);
  if (remainingIds.length === 0) {
    return [];
  }
  return [{ ...issue, relatedMatrixIds: remainingIds.join(', ') }];
});

if (status === 'Failed' || status === 'Blocked') {
  const issue = {
    id: issueId,
    severity,
    area,
    relatedMatrixIds: id,
    status: 'Open',
    owner,
    closeCondition,
  };
  const existingIndex = results.failedIssueRows.findIndex((item) => item.id === issueId);
  if (existingIndex >= 0) {
    results.failedIssueRows[existingIndex] = issue;
  } else {
    results.failedIssueRows.push(issue);
  }
}

fs.writeFileSync(resultsPath, `${JSON.stringify(results, null, 2)}\n`, 'utf8');

console.log('NexSpot QA result updated');
console.log('=========================');
console.log(`results: ${resultsRelativePath}`);
console.log(`row: ${id}`);
console.log(`status: ${status}`);
if (evidence) {
  console.log(`evidence: ${evidence}`);
}
if (status === 'Failed' || status === 'Blocked') {
  console.log(`issue: ${issueId}`);
}
