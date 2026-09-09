---
answers:
  - What must the scheduled pre-push CI checkpoint actually execute?
  - Why are registered Rust targets and successful offline returns not live test passes?
  - Which workflow and fixture gaps must be repaired before the full checkpoint?
  - Does the periodic artifact census authorize deleting old logs or caches?
---
# Qualify a checkpoint from executed coverage and owned lifetimes

- Owner: `SIGNOFF-REPAIR.11.4.3.1.1`; REPAIR-0033.
- Evidence: docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md.
- Status: source/tool/metadata inventory; full checkpoint and remote CI pending.

Census the actual workflows, Make targets, Cargo targets and PostgreSQL registration
before executing the scheduled broad checkpoint. The 6bc76c6 source has 12 packages,
86 test-enabled targets and 40 registered PG runner commands (38 server plus MCP
and CLI). These counts are not executed test counts. Offline database early
returns, unavailable worker/browser skips and deliberately ignored provider tests
must remain distinct in the final evidence. Build required worker binaries and
force --demo for the complete PG collection.

Require format, all-target/all-feature strict lint, workspace tests, the full owned
PG collection/demo, all four Python control modules, doctrines, fresh dependency
checks, redacted Git-history scanning and rendered book verification before the
authorized push. Inspect warnings and exact scope. Then consume remote advancement
and triggered CI outcomes. Full CI is for pushes/important checkpoints; routine
slices keep focused checks and individual commits.

The Rust check job currently bypasses the project-local environment, Python
controls are absent from workflows, and the supply-chain action's stores need
explicit review. Checkpoint child .3 owns these repairs. Predictable publisher
fixture deletion is .4; shared browser profile and unconsumed lifecycle boundaries
are .5, coordinated with existing .7.2/.11.2. These findings are source evidence;
reproduce actual effects before claiming runtime root cause or repair. Keep each
bounded repair committed before proceeding to broad execution .2.

The metadata artifact census records substantial old compiler data but no storage
emergency. Age alone does not authorize deletion. Child .6 must prove exact
same-volume generated-cache identity, inactivity and evidence independence,
then verify residue and regeneration. Historical failed-run logs, ambiguous
fixtures and shared global caches remain preserved. A reasoned retention result
is valid where cleanup cannot be proved safe.

Tool version probes and source-reviewed cache paths establish prerequisites only.
Installed tools are necessary read-only exceptions; all newly owned data must
remain repository-derived on its volume. Locality enforcement is not an arbitrary
filesystem sandbox. This inventory changes no product behavior or gate policy and
does not advance release qualification. After the completed checkpoint, resume
CLI transport/reply bounds under .3.3.4.3.3.3.3.2.3.2.
