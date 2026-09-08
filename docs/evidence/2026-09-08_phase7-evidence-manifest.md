# Phase 7 evidence manifest (`PHASE-7.5.2`)

The gate package's evidence: the guard of record + the record
pointers. Every number re-derives from the named log/artifact —
re-run the command, compare the output (the claim-verification
discipline).

## The guard of record

- `bash scripts/run_pg_tests.sh` → rc=0, **25 live suites** + the
  demo `ALL acceptance checks passed` (the two-host demonstration
  with the real kill points).
- `cargo test --all` → rc=0, **70 suites** (the offline sweep).
- `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make deny` →
  advisories/bans/licenses/sources ok; `make secret-scan` → no
  leaks; `make gate` → 13/13; `make book` builds.
- The load harness's measured run (`scripts/load_harness.sh`):
  200 commands at 8 workers → p50 0.0033s / p95 0.0079s,
  189.2 commands/s, 0 failures (`target/load_harness_run.log`).
- `make release` → rc=0 with the signed manifest + the verify
  (`target/release/release-manifest.json` + `.sig`).

## The per-lane evidence

| Lane | Evidence |
| --- | --- |
| `.1` the hardening | ADR-034; the mTLS roundtrip (the CA-issued client connects, the cert-less refuses); the RLS probe-role refusal; the quota storm/denial/slide legs; the quarantine retention-survival legs; the secret-store resolution + the undeclared refusal; the confidential-dispatch refusal (`classification_unqualified`) |
| `.2` the supply chain | ADR-027; SECURITY.md; the release-manifest roundtrip + the three refusals; the public-enrollment contract |
| `.3` the scaling | the extraction-criteria record (five seams, five trigger measurements, zero extractions) |
| `.4` the capacity/incident | the load harness + the measured run; the thirteen-family runbook catalogue; the game-day catalogue + the pen-test stance |
| `.5` the exit | the ten-line census (`.5.1`), the gate record, the subtraction record, this manifest |

## The named preconditions (the exposure's gate)

1. The externally reviewed threat model (the external review).
2. The prompt-injection action-boundary suite (the §19.4 surface).
3. The penetration test with the findings remediated (the `.4.3`
   findings-to-leaves stance).
