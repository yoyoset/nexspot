# NexSpot Commercial Release Review

> Date: 2026-07-07  
> Version: NexSpot v0.3.0 Commercial Trial  
> Scope: commercial readiness, release artifacts, screenshot/OCR/scrolling capture risk, QA gates, and remaining blockers.

## Executive Decision

| Release level | Decision | Evidence |
|---|---|---|
| Internal trial | Go | Current EXE smoke and installer smoke pass; release evidence is machine-readable. |
| Small commercial pilot | Go with unsigned notice | Pilot distribution is allowed only with `docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md`. |
| Public stable release | No-Go | Real-device QA is incomplete and both artifacts are unsigned. |

NexSpot is ready for controlled internal and small-pilot use, not for public stable distribution. The product has a current distributable EXE and NSIS installer, automated release verification, smoke evidence, manual QA gate evidence, signing readiness evidence, pilot package manifest, and a QA evidence workflow. Public stable remains blocked by Vello OCR validation, scrolling capture validation, multi-display/DPI validation, toolbar visual consistency sign-off, and code signing.

## Current Artifacts

| Artifact | Path | SHA256 | Signature |
|---|---|---|---|
| Release EXE | `src-tauri/target/release/nexspot.exe` | `3f271498d97efea76c4b6005aa238c0a238ea9a50a0686a52eb73e0e105b79cc` | `NotSigned` |
| NSIS installer | `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe` | `38ac84dde57680e5bfa54c16a45c4f27abd054f038cf5dc7836a9a541fa2a0e0` | `NotSigned` |

The installer name is `NexSpot_0.3.0_x64-setup.exe`. Current automated smoke evidence is recorded in `docs/13-SMOKE-EVIDENCE.json` and `docs/14-INSTALLER-SMOKE-EVIDENCE.json`.

## Commercial Improvements Since Initial Review

| Area | Improvement | Commercial value |
|---|---|---|
| Scrolling capture | Added conservative stitch safeguards for tiny jitter and ambiguous repeated content. | Reduces false seams in business screenshots. |
| Scrolling diagnostics | Preview and release source gate preserve target window, stop rule, runtime limits, capture stats, and per-frame stitch quality reports. | Makes failed or partial captures supportable. |
| Scrolling performance | Stitcher seeds each match from the previous successful scroll delta while still scanning every candidate, and records candidate count plus match duration in the preview report. | Improves long-page performance without weakening correctness checks. |
| Vello OCR diagnostics | OCR live capture now hides overlay windows, waits for visibility confirmation, and carries overlay-hide metadata into error/no-text/success states. | Makes Vello OCR failures diagnosable instead of silently ambiguous. |
| Toolbar icon consistency | GDI and Vello toolbar renderers now share commercial style tokens for color, radius, icon size, icon scale, stroke width, button size, spacing, and margin; release verification checks token usage. | Reduces visual drift between capture engines and keeps toolbar polish enforceable. |
| Pin performance | Pin cards are memoized, image pins avoid the Markdown renderer path, and `react-markdown` remains lazy-loaded only for text pins. | Keeps image pin windows responsive and prevents a heavy text-rendering dependency from delaying image workflows. |
| Failure reporting | Capture failures now emit structured `capture://error` events with area/code/title/message and the frontend shows a longer error HUD. | Prevents OCR/scrolling failure states from feeling silent and improves support triage. |
| QA tooling | Added result update, merge validation, manual gate evidence, and QA evidence pack flow. | Turns real-device QA into a repeatable process instead of ad hoc notes. |
| Release packaging | Rebuilt current EXE and NSIS installer after native changes. | Keeps trial artifacts aligned with source. |
| Evidence hygiene | Release evidence filters fragile localized version-info fields. | Keeps commercial reports clean and machine-readable. |

## Current Quality Position

| Area | Status | Notes |
|---|---|---|
| Core build | Passed | `npm.cmd run release` produced current EXE and NSIS installer. |
| Frontend type check | Passed | `npm.cmd run lint`. |
| Rust tests | Passed | `cargo test` passed 9 tests, including scrolling stitch safeguards and seeded stitch matching. |
| EXE smoke | Passed | Current EXE launches and shows main window. |
| Installer smoke | Passed | Current installer installs, overwrites, launches installed app, uninstalls, and cleans test directory. |
| Manual QA gate | No-Go for public stable | 23 required rows still pending. |
| Signing readiness | No-Go for public stable | EXE and installer are `NotSigned`. |

## Public Stable Blockers

| ID | Severity | Area | Close condition |
|---|---|---|---|
| QA-001 | P0 | Vello OCR | Pass real-device Vello OCR without overlay artifacts, with overlay-hide metadata visible, and with no-text/error/export states verified. |
| QA-002 | P0 | Scrolling capture | Pass Chrome, Edge, Electron container, native list, Esc stop, and continuity checks with attached output images. |
| QA-003 | P0 | Multi-display and DPI | Pass screenshot, OCR, save, and copy checks across required DPI/display configurations. |
| QA-004 | P0 | Distribution trust | Sign EXE and installer with an approved identity and refresh signing readiness. |
| QA-005 | P1 | Capture toolbar consistency | Confirm GDI and Vello toolbar icon size, active, hover, cancel, and color states are visually consistent. |

The authoritative blocker tracker is `docs/16-COMMERCIAL-QA-ISSUE-LOG.md`. The manual QA matrix is `docs/12-MANUAL-QA-MATRIX.md`.

## Required Commercial Gate

### Manual QA Matrix Gate

Before public stable, the following must all pass:

1. `npm.cmd run release:evidence`
2. `npm.cmd run qa:manual:gate:strict`
3. `node scripts\commercial-readiness.mjs --strict`
4. `node scripts\signing-readiness.mjs --write-evidence docs\18-SIGNING-READINESS.json`
5. Real-device QA evidence merged through `node scripts\qa-merge-results.mjs --results <run>\results-template.json --apply`

Until those pass with no P0/P1 blockers, label the product as `Commercial Trial`, not `Public Stable`.
