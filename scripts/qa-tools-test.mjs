import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const tmpRelativeDir = 'docs/qa-runs/.tmp-qa-tools-test';
const tmpDir = path.join(root, tmpRelativeDir);
const templatePath = path.join(root, 'docs/qa-runs/latest/results-template.json');
const mergeScript = 'scripts/qa-merge-results.mjs';
const resultUpdateScript = 'scripts/qa-result-update.mjs';
const failures = [];
const parseJsonText = (text) => JSON.parse(text.replace(/^\uFEFF/, ''));

const loadTemplate = () => parseJsonText(fs.readFileSync(templatePath, 'utf8'));

const writeEvidence = (name, contents = 'synthetic QA evidence\n') => {
  fs.mkdirSync(tmpDir, { recursive: true });
  const relativePath = `${tmpRelativeDir}/${name}`;
  fs.writeFileSync(path.join(root, relativePath), contents, 'utf8');
  return relativePath;
};

const writeCase = (name, mutate) => {
  const data = loadTemplate();
  mutate(data);
  fs.mkdirSync(tmpDir, { recursive: true });
  const relativePath = `${tmpRelativeDir}/${name}.json`;
  fs.writeFileSync(path.join(root, relativePath), `${JSON.stringify(data, null, 2)}\n`, 'utf8');
  return relativePath;
};

const runMerge = (relativePath) =>
  spawnSync(process.execPath, [mergeScript, '--results', relativePath], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true,
  });

const runResultUpdate = (...args) =>
  spawnSync(process.execPath, [resultUpdateScript, ...args], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true,
  });

const expectExit = (name, result, expectedStatus, expectedText) => {
  if (result.status !== expectedStatus) {
    failures.push(`${name}: expected exit ${expectedStatus}, got ${result.status}`);
  }
  const combined = `${result.stdout || ''}\n${result.stderr || ''}`;
  if (expectedText && !combined.includes(expectedText)) {
    failures.push(`${name}: missing expected output text: ${expectedText}`);
  }
};

try {
  fs.rmSync(tmpDir, { recursive: true, force: true });

  const pendingOnly = writeCase('pending-only', () => {});
  expectExit('pending-only dry-run', runMerge(pendingOnly), 0, 'Dry-run passed');

  const passedMissingEvidence = writeCase('passed-missing-evidence', (data) => {
    data.rows[0].status = 'Passed';
    data.rows[0].evidence = '';
    data.rows[0].notes = 'No evidence should fail.';
  });
  expectExit(
    'passed missing evidence',
    runMerge(passedMissingEvidence),
    1,
    'cannot be marked Passed without an evidence path',
  );

  const passedMissingEvidenceFile = writeCase('passed-missing-evidence-file', (data) => {
    data.rows[0].status = 'Passed';
    data.rows[0].evidence = `${tmpRelativeDir}/missing-evidence.txt`;
    data.rows[0].notes = 'Missing evidence file should fail.';
  });
  expectExit(
    'passed missing evidence file',
    runMerge(passedMissingEvidenceFile),
    1,
    'evidence file does not exist',
  );

  const failedMissingIssue = writeCase('failed-missing-issue', (data) => {
    data.rows[0].status = 'Failed';
    data.rows[0].evidence = 'docs/qa-runs/latest/failure.png';
    data.rows[0].notes = 'Synthetic failure without issue row.';
    data.failedIssueRows = [];
  });
  expectExit(
    'failed missing issue',
    runMerge(failedMissingIssue),
    1,
    'must have a failedIssueRows entry referencing it',
  );

  const failedWithIssue = writeCase('failed-with-issue', (data) => {
    data.rows[0].status = 'Failed';
    data.rows[0].evidence = 'docs/qa-runs/latest/failure.png';
    data.rows[0].notes = 'Synthetic failure with issue row.';
    data.failedIssueRows = [
      {
        id: 'QA-TEST-MERGE',
        severity: 'P0',
        area: 'Synthetic QA merge test',
        relatedMatrixIds: data.rows[0].id,
        status: 'Open',
        owner: 'QA + Engineering',
        closeCondition: 'Synthetic close condition for qa-tools-test.',
      },
    ];
  });
  expectExit('failed with issue', runMerge(failedWithIssue), 0, 'candidate issue updates: 1');

  const passedWithEvidence = writeCase('passed-with-evidence', (data) => {
    data.rows[0].status = 'Passed';
    data.rows[0].evidence = writeEvidence('evidence.txt');
    data.rows[0].notes = 'Synthetic pass with evidence.';
  });
  expectExit('passed with evidence', runMerge(passedWithEvidence), 0, 'candidate updates: 1');

  const updateMissingEvidence = writeCase('update-missing-evidence', () => {});
  expectExit(
    'result update passed missing evidence',
    runResultUpdate('--results', updateMissingEvidence, '--id', 'ENV-01', '--status', 'Passed'),
    1,
    'Passed rows require --evidence',
  );

  const updateMissingEvidenceFile = writeCase('update-missing-evidence-file', () => {});
  expectExit(
    'result update passed missing evidence file',
    runResultUpdate(
      '--results',
      updateMissingEvidenceFile,
      '--id',
      'ENV-01',
      '--status',
      'Passed',
      '--evidence',
      `${tmpRelativeDir}/missing-update-evidence.txt`,
    ),
    1,
    'Evidence file does not exist',
  );

  const updateFailedMissingIssue = writeCase('update-failed-missing-issue', () => {});
  expectExit(
    'result update failed missing issue',
    runResultUpdate(
      '--results',
      updateFailedMissingIssue,
      '--id',
      'ENV-01',
      '--status',
      'Failed',
      '--notes',
      'Synthetic failure without issue details.',
    ),
    1,
    'Failed/Blocked rows require --issue-id',
  );

  const updatePassed = writeCase('update-passed', () => {});
  expectExit(
    'result update passed',
    runResultUpdate(
      '--results',
      updatePassed,
      '--id',
      'ENV-01',
      '--status',
      'Passed',
      '--evidence',
      writeEvidence('env-01.txt'),
      '--notes',
      'Synthetic pass through result update helper.',
    ),
    0,
    'NexSpot QA result updated',
  );
  expectExit('result update passed merge', runMerge(updatePassed), 0, 'candidate updates: 1');

  const updateFailed = writeCase('update-failed', () => {});
  expectExit(
    'result update failed',
    runResultUpdate(
      '--results',
      updateFailed,
      '--id',
      'ENV-01',
      '--status',
      'Failed',
      '--evidence',
      writeEvidence('env-01-failure.txt'),
      '--notes',
      'Synthetic failure through result update helper.',
      '--issue-id',
      'QA-TEST-UPDATE',
      '--severity',
      'P0',
      '--area',
      'Synthetic QA update test',
      '--owner',
      'QA + Engineering',
      '--close-condition',
      'Synthetic close condition for qa-result-update.',
    ),
    0,
    'issue: QA-TEST-UPDATE',
  );
  expectExit('result update failed merge', runMerge(updateFailed), 0, 'candidate issue updates: 1');

  expectExit(
    'result update clears issue link after pass',
    runResultUpdate(
      '--results',
      updateFailed,
      '--id',
      'ENV-01',
      '--status',
      'Passed',
      '--evidence',
      writeEvidence('env-01-fixed.txt'),
      '--notes',
      'Synthetic pass after failure.',
    ),
    0,
    'status: Passed',
  );
  const clearedIssueData = parseJsonText(fs.readFileSync(path.join(root, updateFailed), 'utf8'));
  if (clearedIssueData.failedIssueRows.some((item) => String(item.relatedMatrixIds || '').includes('ENV-01'))) {
    failures.push('result update clears issue link after pass: issue row still references ENV-01');
  }
} finally {
  fs.rmSync(tmpDir, { recursive: true, force: true });
}

if (failures.length > 0) {
  console.error('NexSpot QA tools test failed');
  console.error('='.repeat(29));
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log('NexSpot QA tools test passed');
console.log('='.repeat(29));
console.log('cases: merge validation, result-update validation, pass/fail issue linkage');
