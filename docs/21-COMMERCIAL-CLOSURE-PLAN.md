# NexSpot Commercial Closure Plan

Generated: 2026-07-07T11:58:28.031Z

## Decision

| Level | Status |
|---|---|
| Internal trial | Go |
| Small commercial pilot | Go with unsigned notice |
| Public stable | No-Go |

## Remaining Counts

| Item | Count |
|---|---:|
| Pending required QA rows | 23 |
| Failed required QA rows | 0 |
| Open P0 blockers | 4 |
| Open P1 polish blockers | 1 |
| Unsigned artifacts | 2 |

## Open Issues

| ID | Severity | Area | Owner | Pending evidence rows |
|---|---|---|---|---|
| QA-001 | P0 | Vello OCR | QA + Engineering | OCR-02, OCR-03, OCR-04, OCR-05, CAP-06 |
| QA-002 | P0 | Scrolling capture | QA + Engineering | SCR-01, SCR-02, SCR-03, SCR-04, SCR-05, SCR-06 |
| QA-003 | P0 | Multi-display and DPI | QA | ENV-03, ENV-05, ENV-06, ENV-08, CAP-01, CAP-02, CAP-03, CAP-04, OCR-03 |
| QA-004 | P0 | Distribution trust | Release | None |
| QA-005 | P1 | Capture toolbar consistency | QA + Design | CAP-05, CAP-06 |

## Next Phases

### 1. Real-device QA

Exit criteria: All required ENV/CAP/OCR/SCR rows are Passed with repository evidence files and no failed required rows.

Commands:

- `node scripts\qa-evidence-pack.mjs --out-dir docs\qa-runs\<machine-or-run-label>`
- `node scripts\qa-merge-results.mjs --results docs\qa-runs\<machine-or-run-label>\results-template.json`
- `node scripts\qa-merge-results.mjs --results docs\qa-runs\<machine-or-run-label>\results-template.json --apply`
- `npm.cmd run qa:manual:gate`

### 2. Signing

Exit criteria: EXE and NSIS installer are Authenticode signed, signer subject/thumbprint are pinned, and signing readiness is public-stable ready.

Commands:

- `node scripts\signing-readiness.mjs --write-evidence docs\18-SIGNING-READINESS.json`
- `npm.cmd run qa:manual:gate:strict`

### 3. Final release evidence

Exit criteria: Release evidence, pilot manifest, and commercial readiness all match current artifact hashes with public stable Go.

Commands:

- `npm.cmd run release:evidence`
- `node scripts\pilot-distribution-manifest.mjs --write-manifest docs\19-PILOT-DISTRIBUTION-MANIFEST.json`
- `node scripts\commercial-readiness.mjs --strict --write-evidence docs\20-COMMERCIAL-READINESS.json`


## Public Stable Blockers

- manual matrix pending: ENV-01
- manual matrix pending: ENV-02
- manual matrix pending: ENV-03
- manual matrix pending: ENV-05
- manual matrix pending: ENV-06
- manual matrix pending: ENV-08
- manual matrix pending: CAP-01
- manual matrix pending: CAP-02
- manual matrix pending: CAP-03
- manual matrix pending: CAP-04
- manual matrix pending: CAP-05
- manual matrix pending: CAP-06
- manual matrix pending: OCR-01
- manual matrix pending: OCR-02
- manual matrix pending: OCR-03
- manual matrix pending: OCR-04
- manual matrix pending: OCR-05
- manual matrix pending: SCR-01
- manual matrix pending: SCR-02
- manual matrix pending: SCR-03
- manual matrix pending: SCR-04
- manual matrix pending: SCR-05
- manual matrix pending: SCR-06
- open P0 blocker: QA-001
- open P0 blocker: QA-002
- open P0 blocker: QA-003
- open P0 blocker: QA-004
- open P1 polish blocker: QA-005
- artifact not signed: exe (NotSigned)
- artifact not signed: nsisInstaller (NotSigned)
- signing readiness: signing readiness is not public-stable ready
