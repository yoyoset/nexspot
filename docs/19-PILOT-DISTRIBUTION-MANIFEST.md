# NexSpot Pilot Distribution Manifest

> Date: 2026-07-05  
> Version: NexSpot v0.3.0 Commercial Trial  
> Scope: internal trial and small commercial pilot package checklist.

## Purpose

This manifest prevents a pilot package from being assembled by memory. It verifies that the release artifacts, hashes, unsigned-build notice, release evidence, QA gate, signing readiness, and QA evidence pack all refer to the same current build.

## Command

```powershell
node scripts\pilot-distribution-manifest.mjs --write-manifest docs\19-PILOT-DISTRIBUTION-MANIFEST.json
```

## Required Pilot Materials

| Material | Purpose |
|---|---|
| `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe` | Primary pilot installer |
| `src-tauri/target/release/nexspot.exe` | Green/portable EXE for engineering trial |
| `docs/11-RELEASE-EVIDENCE.json` | Machine-readable release evidence |
| `docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md` | Required recipient warning |
| `docs/17-MANUAL-QA-GATE.json` | Manual gate and blocker state |
| `docs/18-SIGNING-READINESS.json` | Distribution trust/signing state |
| `docs/qa-runs/latest/qa-run.md` | Pilot QA instructions |
| `docs/qa-runs/latest/results-template.json` | Result return template |

## Current Decision

Internal trial and small commercial pilot are allowed only when `docs/19-PILOT-DISTRIBUTION-MANIFEST.json` has no blockers. Public stable release remains outside this manifest and requires strict manual QA plus public-stable signing readiness.
