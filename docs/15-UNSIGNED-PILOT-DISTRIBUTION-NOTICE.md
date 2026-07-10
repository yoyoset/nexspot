# NexSpot Unsigned Pilot Distribution Notice

> Date: 2026-07-10
> Version: NexSpot v0.3.0 RC1 · Commercial Trial
> Scope: Internal trial and small commercial pilot only

## Decision

The current NexSpot pilot build is intentionally documented as unsigned:

| Artifact | Path | Authenticode status | SHA256 |
|---|---|---|---|
| Release EXE | `src-tauri/target/release/nexspot.exe` | `NotSigned` | `86964f7b8466926b032a07c27c13f4faa1e4b752ac72cafdf62a93a98efecf4e` |
| NSIS installer | `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe` | `NotSigned` | `e91da429727c923f028435900642211e3fadcc0c6af304d0e15fc049b18bacfc` |

REL-06 is acceptable for pilot scope only when this notice is attached to the release evidence and the pilot recipient is told that the installer is unsigned.

## Pilot Runtime Prerequisite

This pilot installer is built for repeatable current-user installation and does not download WebView2 during silent installation. Pilot machines must already have Microsoft Edge WebView2 Runtime installed. The release smoke verifies the application can launch on the packaging machine; public stable distribution must either keep a documented WebView2 prerequisite or switch to an offline/fixed WebView2 runtime strategy.

## Pilot User Message

Use this exact meaning when sending the pilot package:

> NexSpot v0.3.0 Commercial Trial is an unsigned pilot build. Windows may show an unknown publisher or security warning before launch or installation. Only install it if you received the package directly from the NexSpot pilot channel. Verify the SHA256 hash before testing.

## Required Handling

1. Do not describe this build as signed, store-ready, enterprise-ready, or public stable.
2. Attach `docs/11-RELEASE-EVIDENCE.json` and this notice to internal/pilot release notes.
3. Ask pilot users to verify the installer hash before running it.
4. Treat unexpected publisher prompts, SmartScreen blocks, or antivirus quarantine as pilot feedback, not as user error.
5. Confirm WebView2 Runtime is present on pilot machines before using the installer.
6. Code signing is still required before a public stable release.

## Go / No-Go

| Release level | Decision | Reason |
|---|---|---|
| Internal trial | Go | Authenticode status is explicit, hashes are recorded, and the installer smoke passed. |
| Small commercial pilot | Go with notice | Pilot users must receive the unsigned-build warning and SHA256 hash. |
| Public stable release | No-Go | Requires code signing and Windows warning prompt review. |

## Verification

This notice is checked by `npm.cmd run release:verify` when REL-06 in `docs/12-MANUAL-QA-MATRIX.md` claims the unsigned pilot notice has passed.
