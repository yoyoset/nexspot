# NexSpot Signing Readiness

> Date: 2026-07-10
> Version: NexSpot v0.3.0 RC1 · Commercial Trial
> Scope: public-stable release signing gate.

## Purpose

This gate turns distribution trust from a prose warning into executable evidence. Internal trial and small commercial pilot can proceed with the unsigned notice, but public stable release requires signed artifacts and a pinned signer identity.

## Command

```powershell
node scripts\signing-readiness.mjs --write-evidence docs\18-SIGNING-READINESS.json
```

Optional public-stable identity checks:

```powershell
$env:NEXSPOT_SIGNING_SUBJECT="expected certificate subject fragment"
$env:NEXSPOT_SIGNING_THUMBPRINT="expected certificate thumbprint"
```

## Current Decision

| Release level | Decision | Reason |
|---|---|---|
| Internal trial | Go | Unsigned artifacts are documented and hashed. |
| Small commercial pilot | Go with notice | Pilot users must receive `docs/15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md`. |
| Public stable release | No-Go | Requires valid Authenticode signatures on both EXE and NSIS installer plus pinned signer identity. |

## Close Condition

QA-004 can close only when:

1. `docs/18-SIGNING-READINESS.json` reports `publicStableReady: true`.
2. Both release artifacts have Authenticode status `Valid`.
3. The signer subject and thumbprint match the approved production certificate.
4. `npm.cmd run release:verify` passes.
5. `npm.cmd run qa:manual:gate:strict` passes after real-device QA is complete.
