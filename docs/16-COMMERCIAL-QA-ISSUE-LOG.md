# NexSpot Commercial QA Issue Log

> Date: 2026-07-07  
> Version: NexSpot v0.3.0 Commercial Trial  
> Scope: public-stable release blockers and pilot watch items.

## Release Position

NexSpot v0.3.0 now has refreshed 2026-07-07 19:49 Asia/Taipei release artifacts that include the OCR capture-failure fix, the manual QA gate script, conservative scrolling stitch safeguards in `src-tauri/src/service/stitch.rs`, scrolling partial-result error visibility, and QA tooling that rejects missing evidence files. The current EXE smoke and installer smoke pass. It is suitable for internal trial and small commercial pilot with the unsigned-build notice attached. It is not yet a public stable commercial release because the required real-device matrix, toolbar visual consistency sign-off, and code signing are still open.

Current artifacts:

- EXE: `src-tauri/target/release/nexspot.exe`
- EXE SHA256: `3f271498d97efea76c4b6005aa238c0a238ea9a50a0686a52eb73e0e105b79cc`
- Installer: `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe`
- Installer SHA256: `38ac84dde57680e5bfa54c16a45c4f27abd054f038cf5dc7836a9a541fa2a0e0`
- Release evidence: `docs/11-RELEASE-EVIDENCE.json`
- Manual QA matrix: `docs/12-MANUAL-QA-MATRIX.md`
- Manual QA gate evidence: `docs/17-MANUAL-QA-GATE.json`
- Signing readiness evidence: `docs/18-SIGNING-READINESS.json`
- Pilot distribution manifest: `docs/19-PILOT-DISTRIBUTION-MANIFEST.json`
- Current QA evidence pack example: `docs/qa-runs/latest/qa-run.md`
- Current QA result template example: `docs/qa-runs/latest/results-template.json`

## Public Stable Blockers

| ID | Severity | Area | Related matrix IDs | Status | Owner | Close condition |
|---|---|---|---|---|---|---|
| QA-001 | P0 | Vello OCR | OCR-02, OCR-03, OCR-04, OCR-05, CAP-06 | Open | QA + Engineering | Pass Vello OCR on real Windows devices without overlay artifacts, with no-text/error/export states verified. |
| QA-002 | P0 | Scrolling capture | SCR-01, SCR-02, SCR-03, SCR-04, SCR-05, SCR-06 | Open | QA + Engineering | Pass Chrome, Edge, Electron container, native list, Esc stop, and image-continuity checks with attached output images. |
| QA-003 | P0 | Multi-display and DPI | ENV-03, ENV-05, ENV-06, ENV-08, CAP-01..CAP-04, OCR-03 | Open | QA | Pass screenshot, OCR, save, and copy checks at required DPI/display configurations. |
| QA-004 | P0 | Distribution trust | REL-06 | Open | Release | `docs/18-SIGNING-READINESS.json` reports public-stable ready with valid signatures, approved signer identity, and matching current artifact hashes. |
| QA-005 | P1 | Capture toolbar consistency | CAP-05, CAP-06 | Open | QA + Design | Confirm GDI and Vello toolbar icon size, active, hover, cancel, and color states are visually consistent. |

## Pilot Watch Items

| ID | Severity | Area | Related matrix IDs | Status | Owner | Close condition |
|---|---|---|---|---|---|---|

## Closed Evidence

| ID | Area | Evidence | Result |
|---|---|---|---|
| REL-01 | EXE launch smoke | `docs/13-SMOKE-EVIDENCE.json` | Current EXE hash launches and shows main window `NexSpot`. |
| REL-02..REL-05 | Installer smoke | `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Current NSIS hash installs, overwrites, launches installed app, uninstalls, and cleans the test directory. |
| REL-06 pilot notice | Unsigned pilot distribution notice | `docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md` | Pilot-only warning and hash-verification guidance documented. |
| QA-006 | Pin/performance polish | `src/components/Pin/PinCollectionWindow.tsx`; `scripts/release-verify.mjs`; `npm.cmd run release:verify` | Pin cards are memoized, image pins stay on the non-Markdown render path with async image decoding, `react-markdown` is lazy-loaded only for text pins, and release verification enforces the source gate. |
| QA-007 | Failure reporting quality | `src-tauri/src/service/notification.rs`; `src-tauri/src/service/native_overlay/commands.rs`; `src/components/Overlay/TauriEventListener.tsx`; `scripts/release-verify.mjs` | OCR and scrolling command failures emit structured `capture://error` events with area/code/title/message, the frontend shows an error HUD, OCR result windows preserve error/no-text metadata, and initial scrolling failures reset transient state. |
| QA-008 | Release artifact freshness | `npm.cmd run release`, `docs/13-SMOKE-EVIDENCE.json`, `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Current artifacts include scrolling stitch quality reports, seeded stitch performance diagnostics, explicit partial-result error messages for scrolling loop failures, OCR overlay-hide diagnostics with visible overlay-risk warnings, shared toolbar style tokens, Pin performance gating, structured failure-reporting events, P1 toolbar polish gating, and QA tooling that rejects missing evidence files; EXE SHA256 `3f271498d97efea76c4b6005aa238c0a238ea9a50a0686a52eb73e0e105b79cc`; installer SHA256 `38ac84dde57680e5bfa54c16a45c4f27abd054f038cf5dc7836a9a541fa2a0e0`. |

## Update Rules

1. Do not close a P0 item from code review alone. Attach real-device evidence paths generated from `node scripts\qa-evidence-pack.mjs --out-dir docs\qa-runs\<machine-or-run-label>`.
2. Fill the run's `results-template.json`, including `failedIssueRows` for every `Failed` or `Blocked` row, validate with `node scripts\qa-merge-results.mjs --results docs\qa-runs\<machine-or-run-label>\results-template.json`, then apply it with `--apply` to update `docs/12-MANUAL-QA-MATRIX.md` and this issue log.
3. If a required matrix row fails, the matching `failedIssueRows` entry must include reproduction evidence, affected artifact hash context, owner, and close condition.
4. After matrix or issue-log changes, run `npm.cmd run qa:manual:gate` and commit the refreshed `docs/17-MANUAL-QA-GATE.json`.
5. Run `node scripts\qa-tools-test.mjs` before public-stable sign-off so result merge validation remains covered.
6. Public stable release requires all P0/P1 items closed, required matrix rows passed, `node scripts\signing-readiness.mjs --write-evidence docs\18-SIGNING-READINESS.json` reporting public-stable ready, and `npm.cmd run qa:manual:gate:strict` passing.
