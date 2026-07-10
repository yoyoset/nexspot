# NexSpot Commercial Manual QA Matrix

> Date: 2026-07-10
> Version: NexSpot v0.3.0 RC1 · Commercial Trial
> Artifacts:
> - EXE: `src-tauri/target/release/nexspot.exe`
> - Installer: `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe`
>
> Status: RC1 artifacts rebuilt at 2026-07-10 13:17 Asia/Taipei and automated EXE/installer smoke has passed. Public-stable readiness still depends on real-device completion for GUI flows, toolbar visual consistency, and code-signing/Windows-warning review.

## Decision

| Release level | Current decision | Required before upgrade |
|---|---|---|
| Internal trial | Go | Use the 2026-07-07 19:49 artifacts and attach release evidence plus the unsigned pilot notice. |
| Small commercial pilot | Go with notice | Tell pilot users scrolling capture and Vello OCR are under device-matrix validation and that the build is unsigned. |
| Public stable release | No-Go until matrix passed | Finish Vello OCR, scrolling capture, multi-display/DPI, and code-signing warning review. |

## Test Owner Instructions

1. Use the exact artifact paths listed above.
2. Record every failed case with app name, Windows version, DPI scale, display count, GPU, reproduction steps, expected result, actual result, and attachment path.
3. Do not mark a scenario passed if the result is only visually plausible. Open the saved file or OCR text and verify the output.
4. For scrolling capture, verify top, middle, and bottom continuity, not just whether a preview window opened.
5. For OCR, verify the result contains real screen text and does not include overlay borders, toolbar labels, or mask artifacts.
6. Before each real-device run, run `node scripts\qa-evidence-pack.mjs --out-dir docs\qa-runs\<machine-or-run-label>` and attach the generated `qa-run.md`, `environment.json`, `qa-evidence-pack.json`, and `results-template.json`.
7. Fill `results-template.json`, including repository-relative evidence file paths for every `Passed` required row. Use semicolons for multiple attachments, and make sure every listed file exists before merging.
8. Include `failedIssueRows` for every `Failed` or `Blocked` result, then run `node scripts\qa-merge-results.mjs --results docs\qa-runs\<machine-or-run-label>\results-template.json` for dry-run validation.
9. After the dry-run passes, run `node scripts\qa-merge-results.mjs --results docs\qa-runs\<machine-or-run-label>\results-template.json --apply` to update this matrix and `docs/16-COMMERCIAL-QA-ISSUE-LOG.md`.
10. After editing this matrix or the issue log, run `npm.cmd run qa:manual:gate` to refresh `docs/17-MANUAL-QA-GATE.json`.
11. Run `node scripts\qa-tools-test.mjs` before public-stable sign-off to confirm QA merge tooling still rejects missing evidence, missing evidence files, and missing issue rows.
12. Run `node scripts\signing-readiness.mjs --write-evidence docs\18-SIGNING-READINESS.json` before public-stable sign-off.
13. Public stable release requires `npm.cmd run qa:manual:gate:strict` to pass with no pending required rows, no open P0/P1 blockers, and public-stable signing readiness.

## Environment Matrix

| ID | Environment | Required | Status | Evidence | Notes |
|---|---|---:|---|---|---|
| ENV-01 | Windows 10, single display, 100% DPI | Yes | Pending |  |  |
| ENV-02 | Windows 11, single display, 100% DPI | Yes | Pending |  |  |
| ENV-03 | Windows 11, 150% DPI | Yes | Pending |  |  |
| ENV-04 | Windows 11, 200% DPI | Recommended | Pending |  |  |
| ENV-05 | Dual display, same DPI | Yes | Pending |  |  |
| ENV-06 | Dual display, mixed DPI | Yes | Pending |  |  |
| ENV-07 | Negative-coordinate secondary display | Recommended | Pending |  |  |
| ENV-08 | Integrated GPU | Yes | Pending |  |  |
| ENV-09 | Discrete GPU | Recommended | Pending |  |  |

## Functional Matrix

| ID | Scenario | Required | Status | Evidence | Notes |
|---|---|---:|---|---|---|
| CAP-01 | GDI selection screenshot, copy to clipboard | Yes | Pending |  |  |
| CAP-02 | GDI selection screenshot, save to file | Yes | Pending |  |  |
| CAP-03 | Vello selection screenshot, copy to clipboard | Yes | Pending |  |  |
| CAP-04 | Vello selection screenshot, save to file | Yes | Pending |  |  |
| CAP-05 | Toolbar icon size, color, hover, active, cancel states in GDI | Yes | Pending |  |  |
| CAP-06 | Toolbar icon size, color, hover, active, cancel states in Vello | Yes | Pending |  |  |
| OCR-01 | GDI OCR on plain browser text | Yes | Pending |  |  |
| OCR-02 | Vello OCR on plain browser text | Yes | Pending |  |  |
| OCR-03 | OCR on document text at 150% DPI | Yes | Pending |  |  |
| OCR-04 | OCR no-text area shows no-text guidance and disables copy/export | Yes | Pending |  |  |
| OCR-05 | OCR export creates a readable `.txt` file when text exists | Yes | Pending |  |  |
| SCR-01 | Chrome page scrolling capture auto-scrolls and opens preview | Yes | Pending |  |  |
| SCR-02 | Edge page scrolling capture auto-scrolls and opens preview | Yes | Pending |  |  |
| SCR-03 | Electron app scrolling container can be captured or fails gracefully | Yes | Pending |  |  |
| SCR-04 | Native Windows list can be captured or stops gracefully when unsupported | Yes | Pending |  |  |
| SCR-05 | `Esc` stops active scrolling capture and reports manual-stop reason | Yes | Pending |  |  |
| SCR-06 | Scrolling result has no large repeated blocks, blank regions, or obvious seams | Yes | Pending |  |  |
| PIN-01 | Pin window opens image pin without delayed Markdown bundle | Recommended | Pending |  |  |
| PIN-02 | Text pin lazy-loads Markdown rendering only when needed | Recommended | Pending |  |  |
| REL-01 | EXE launches, creates the main window, and remains alive for at least 6 seconds | Yes | Passed automated smoke | `docs/13-SMOKE-EVIDENCE.json` | Current EXE hash passed 6-second launch smoke with main window title `NexSpot`. |
| REL-02 | NSIS installer launches | Yes | Passed automated installer smoke | `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Current installer hash passed executed NSIS smoke. |
| REL-03 | Fresh install creates app entry and launches installed app | Yes | Passed automated installer smoke | `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Fresh install exited 0, installed EXE version metadata matched v0.3.0/NexSpot, and installed app launched with main window title `NexSpot`. |
| REL-04 | Overwrite install preserves expected config and replaces binaries | Yes | Passed automated installer smoke | `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Overwrite install exited 0; final installed EXE hash matched the fresh-installed hash and version metadata were recorded. |
| REL-05 | Uninstall removes app entry and installed files | Yes | Passed automated installer smoke | `docs/14-INSTALLER-SMOKE-EVIDENCE.json` | Uninstall exited 0 and temporary install directory was removed. |
| REL-06 | Unsigned installer warning is documented and acceptable for pilot scope | Yes | Passed unsigned pilot notice | `docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md`; `docs/11-RELEASE-EVIDENCE.json` | Pilot scope only; public stable still requires code signing and Windows warning prompt review. |

## Toolbar Visual Acceptance

| Check | Pass criteria |
|---|---|
| Cross-engine consistency | GDI and Vello toolbars use the same button size, spacing, radius, primary/accent/bad colors, active fill, and disabled/hover visual language. |
| Icon rhythm | Tool and action icons appear centered on the same optical grid; no icon looks obviously larger, lower, heavier, or from a different visual family. |
| State clarity | Normal, hover, pressed, active, and cancel states remain distinguishable without changing toolbar layout or causing text/tooltip overlap. |
| Release gate | `npm.cmd run release:verify` passes the shared toolbar style-token source gate before sign-off. |

## Scrolling Capture Acceptance

| Check | Pass criteria |
|---|---|
| Target focus | The selected area receives wheel input or fails with a clear no-growth/unsupported outcome. |
| Target evidence | The preview records the target window title/class/PID so failed or partial scrolls can be traced to the actual recipient. |
| Auto-stop | Capture stops after content stops growing or when `Esc` is pressed. |
| Preview | Preview shows final dimensions, zoom controls, workflow feedback, and stitch quality diagnostics. |
| Performance evidence | Preview shows stitch match performance, candidate count, and seeded-match information so slow long captures can be diagnosed from the QA attachment. |
| Image quality | Top, middle, and bottom content are continuous enough for business use; repeated or ambiguous content must stop as no-growth/unsupported instead of producing a visibly false seam. |
| Failure behavior | No infinite capture loop, frozen overlay, stale preview action, or silent disappearance. Initial-capture failure must reset scrolling state and show a user-visible failure. |
| Partial result truthfulness | If capture stops because of an internal error, the preview must label the image as a partial result and show the raw error message. |

## Vello OCR Acceptance

| Check | Pass criteria |
|---|---|
| Real content capture | OCR result uses selected screen content after overlay is hidden. |
| Overlay hide diagnostics | Result metadata shows overlay hidden count, total overlay window count, and hide wait time; Vello OCR cannot pass if any overlay window remains visible at capture time. |
| Overlay risk warning | If any overlay window remains visible after hide, the result window must show a visible warning instead of burying the issue in metadata only. |
| Overlay exclusion | Result does not include selection border, dim mask, or toolbar artifacts. |
| Empty content | Empty/no-text result opens the result window with a clear retry hint. |
| Capture failure | Screen capture failures before OCR keep the result window open with an error state, raw message, and capture metadata instead of silently disappearing. |
| Engine failure | OCR engine or language-pack failures keep the result window open with an error state, raw message, and capture metadata. |
| Export | Non-empty text can be copied and exported to `.txt`. |
| Positioning | Result window appears near the selection without obvious DPI offset. |

## Failure Reporting Acceptance

| Check | Pass criteria |
|---|---|
| Visible feedback | OCR, scrolling, and capture command failures emit a user-visible in-app error HUD, not only a system notification or log entry. |
| Diagnostic payload | Failure events preserve area, code, title, and raw message for support triage. |
| State reset | Initial scrolling-capture failures reset transient scrolling state and prevent stale preview actions. |
| Release gate | `npm.cmd run release:verify` passes the structured capture-failure source gate before sign-off. |

## Pin Performance Acceptance

| Check | Pass criteria |
|---|---|
| Image pin path | Image pins render without loading or waiting for the Markdown renderer bundle. |
| Text pin path | Text pins lazy-load Markdown rendering only when text content is actually displayed. |
| Re-render control | Existing pin cards do not all remount or visibly flicker when a pin is copied, removed, or when a new pin arrives. |
| Release gate | `npm.cmd run release:verify` passes the Pin performance source gate before sign-off. |

## Screenshot Copy/Save Acceptance

| Check | Pass criteria |
|---|---|
| Clipboard truthfulness | Copy success notification is shown only after Windows accepts the bitmap into the clipboard, including both normal screenshot copy and scrolling-preview image copy. |
| Clipboard failure | If the clipboard is locked or rejects the bitmap, the overlay shows a copy failure notification instead of a false success. |
| Output verification | Saved files must be opened and visually checked; do not rely only on the saved notification. |

## Issue Log

Canonical blocker and pilot watch-item tracker: `docs/16-COMMERCIAL-QA-ISSUE-LOG.md`.
Machine-readable manual gate evidence: `docs/17-MANUAL-QA-GATE.json`.
Machine-readable signing readiness evidence: `docs/18-SIGNING-READINESS.json`.
Pilot distribution manifest: `docs/19-PILOT-DISTRIBUTION-MANIFEST.json`.
Current QA evidence pack example: `docs/qa-runs/latest/qa-run.md`.
Current QA result template example: `docs/qa-runs/latest/results-template.json`.

| ID | Severity | Scenario ID | Environment ID | Summary | Owner | Status | Fixed in |
|---|---|---|---|---|---|---|---|
|  |  |  |  |  |  |  |  |

## Sign-Off

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Product |  |  |  |  |
| QA |  |  |  |  |
| Engineering |  |  |  |  |
| Release |  |  |  |  |
