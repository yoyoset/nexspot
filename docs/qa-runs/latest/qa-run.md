# NexSpot QA Evidence Pack

Generated: 2026-07-10T05:20:28.452Z

## Release Under Test

| Field | Value |
|---|---|
| Version | 0.3.0 |
| EXE | `src-tauri/target/release/nexspot.exe` |
| EXE SHA256 | `86964f7b8466926b032a07c27c13f4faa1e4b752ac72cafdf62a93a98efecf4e` |
| Installer | `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe` |
| Installer SHA256 | `e91da429727c923f028435900642211e3fadcc0c6af304d0e15fc049b18bacfc` |
| Internal trial | Go |
| Small commercial pilot | Go with unsigned notice |
| Public stable release | No-Go |

## Environment Snapshot

Review `environment.json` for OS, DPI, monitor, and GPU details. Fill any missing display/DPI details manually below.

| Field | Value |
|---|---|
| Tester |  |
| Machine label |  |
| Windows edition/build |  |
| Display count |  |
| DPI / scale per display |  |
| GPU / driver |  |

## Required Pending IDs

- [ ] ENV-01
- [ ] ENV-02
- [ ] ENV-03
- [ ] ENV-05
- [ ] ENV-06
- [ ] ENV-08
- [ ] CAP-01
- [ ] CAP-02
- [ ] CAP-03
- [ ] CAP-04
- [ ] CAP-05
- [ ] CAP-06
- [ ] OCR-01
- [ ] OCR-02
- [ ] OCR-03
- [ ] OCR-04
- [ ] OCR-05
- [ ] SCR-01
- [ ] SCR-02
- [ ] SCR-03
- [ ] SCR-04
- [ ] SCR-05
- [ ] SCR-06

## Open P0 Blockers

- [ ] QA-001
- [ ] QA-002
- [ ] QA-003
- [ ] QA-004

## Open P1 Polish Blockers

- [ ] QA-005

## Evidence Checklist

- [ ] Saved normal screenshot image
- [ ] Copied-image paste target screenshot or note
- [ ] OCR result text export when applicable
- [ ] Scrolling capture output image
- [ ] Scrolling preview metadata screenshot or copied metadata
- [ ] Windows warning prompt screenshot for unsigned installer

Evidence paths must be repository-relative files that already exist when merged. Use semicolons to list multiple attachments for one row.

## Result Rows To Paste Back

| ID | Status | Evidence path | Notes |
|---|---|---|---|
| ENV-01 |  |  |  |
| ENV-02 |  |  |  |
| ENV-03 |  |  |  |
| ENV-05 |  |  |  |
| ENV-06 |  |  |  |
| ENV-08 |  |  |  |
| CAP-01 |  |  |  |
| CAP-02 |  |  |  |
| CAP-03 |  |  |  |
| CAP-04 |  |  |  |
| CAP-05 |  |  |  |
| CAP-06 |  |  |  |
| OCR-01 |  |  |  |
| OCR-02 |  |  |  |
| OCR-03 |  |  |  |
| OCR-04 |  |  |  |
| OCR-05 |  |  |  |
| SCR-01 |  |  |  |
| SCR-02 |  |  |  |
| SCR-03 |  |  |  |
| SCR-04 |  |  |  |
| SCR-05 |  |  |  |
| SCR-06 |  |  |  |

Update `results-template.json` with the helper, then run:

```powershell
node scripts\qa-result-update.mjs --results docs\qa-runs\latest\results-template.json --id SCR-01 --status Passed --evidence docs/qa-runs/latest/scr-01.png --notes "Chrome page captured"
node scripts\qa-merge-results.mjs --results docs\qa-runs\latest\results-template.json
node scripts\qa-merge-results.mjs --results docs\qa-runs\latest\results-template.json --apply
```

## Failure Issue Row Template

| ID | Severity | Scenario ID | Environment ID | Summary | Owner | Status | Fixed in |
|---|---|---|---|---|---|---|---|
| QA-NEW | P0 |  |  |  | QA + Engineering | Open |  |

For any `Failed` or `Blocked` result row, pass `--issue-id`, `--severity`, `--area`, `--owner`, and `--close-condition` to `qa-result-update.mjs`. The merge script updates `docs/16-COMMERCIAL-QA-ISSUE-LOG.md` during `--apply`.
