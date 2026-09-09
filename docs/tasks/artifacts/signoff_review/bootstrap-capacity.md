# Bootstrap completion capacity before dispatch

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1. Product baseline: a3fca5e.
Status: repaired; thirty-one selected controls, final three-scenario capacity matrix and strict CLI lint pass. All results consumed and unique fixtures absent.

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_writers bootstrap_checks_completion_capacity_before_dispatch_at_the_exact_bound -- --nocapture --test-threads=1`
returns rc=101, 0/1 matrix control passed (build 12.11s; execution 3.70s).
The real CLI sends one HTTP request when completion would exceed 8,388,608 bytes
by exactly one byte. Pending needs only 8,387,814 bytes and is published first;
completion then returns the actual 8 MiB encoding error. Original bytes change,
old principal/thread maps remain intact, and exclusion releases. The fitting
positive control has a completion of exactly 8,388,608 bytes and succeeds with
one request; its pending is 8,387,813 bytes. The matched desired refusal fails.

Both real child/server results are consumed and both unique owned fixtures are
removed before the desired matrix assertion. This measures dispatch, local
publication/error and recovery capacity, not authoritative PostgreSQL creation.
The counterexample now uses actual typed Rust records, actual JSON encoding and
actual StateFile/CLI publication, beyond the preceding Python source model.

Root cause: bootstrap_flow persists only pending before HTTP and validates the
larger principal/completed-receipt snapshot after the response. state_store's
8 MiB codec limit is correct; pre-dispatch admission does not yet account for
that completion shape. Retaining a key does not make an oversized snapshot fit.

- capacity-baseline.log: 1557 bytes, SHA-256 `88cc2fbaf97dabe2f41ea5c1e3d5c20898f3c40dbd53d573f6cf08fb8bb892c9`.
- capacity-baseline.exit: 4 bytes, SHA-256 `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c`.

## Implemented preflight contract

A private completion installer is shared by the prospective snapshot and actual
post-response publication. Before changing writer state or dispatching HTTP,
clone the current snapshot, install the exact saved request and prospective
principal/completion (with pending retained), and call the real nonpublishing
state codec. Preserve every unrelated map. The initial pending publication still
checks its own bound before HTTP, including a possibly larger earlier receipt.

For an unknown outcome, the private sample uses HumanPrincipalId/TenantId from
nil UUIDs through the actual typed formatter. Source inspection of
crates/reasonbraid-core/src/id.rs shows prefix plus underscore plus the UUID
Display representation; canonical admission requires round-trip equality.
Hyphenated UUIDs and fixed family prefixes have invariant ASCII JSON widths.
The outcome validator fixes kind/name/request and derives the bnd_/grt_ strings;
false is the longer JSON boolean. A known local receipt uses its exact outcome.
Thus the full real codec measures escaping/metadata and preserved maps without
an invented constant allowance. The sizing sample and encoded bytes are dropped
in memory, never published, returned or sent to a server. Only actual validated
outcomes enter publication. This is an encoded-size check, not physical disk
reservation or protection from later synchronization/process failures.

## Verification results

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib --test bootstrap_state --test state_publication --test state_writers -- --nocapture --test-threads=1`
returns rc=0: 11 library + four schema + four publication + twelve writer controls
pass (build 22.23s; execution 0.98s / 0.62s / 0.10s / 15.69s). The matched
one-byte-over case sends zero HTTP, preserves exact original bytes and releases
exclusion; the exact-limit case sends one request and completes. Both preserve
old maps. Original-key failure recovery, strict replies, local historical output,
process interruption, all writer orders and storage compatibility remain green.
Initial all-target CLI strict lint returns rc=0 in 16.44s. Results consumed.

The final matrix adds a previously saved matching pending snapshot at the same
one-byte-over bound and the explicit preflight error phase. Run
`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_writers bootstrap_checks_completion_capacity_before_dispatch_at_the_exact_bound -- --nocapture --test-threads=1`:
rc=0, 1/1 matrix test (build 15.25s; execution 4.77s). Both fresh and restored
pending refusals send zero HTTP, preserve exact bytes/maps and release exclusion.
Both report "bootstrap completion preflight refused before HTTP" for this
invocation; no inference is made about earlier attempts. The exact-limit fresh
control succeeds with one request and preserved old maps. All three unique
fixtures are removed and child/server results consumed. Final product changes
after the broad run only add the preflight error context; the capacity algorithm
and publication behavior are unchanged.

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
finally returns rc=0 in 17.40s; result consumed. Unique writer/e2e/record/state/
unit/sync-probe fixtures are absent. This bounded client-capacity slice runs no
PostgreSQL/full CI/push; real server restore/fresh-intent evidence remains the
prior REPAIR-0031 control. The scheduled pre-push gate follows the clean commit.

Retained evidence under target/cli-bootstrap-controls:

- capacity-baseline.log: 1557 bytes; SHA-256 `88cc2fbaf97dabe2f41ea5c1e3d5c20898f3c40dbd53d573f6cf08fb8bb892c9`.
- capacity-baseline.exit: 4 bytes; SHA-256 `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c`.
- capacity-candidate.log: 7364 bytes; SHA-256 `fc3869023d75e4dcde26dbce583f858d113833408f1572725bd1dcd98c07cfc5`.
- capacity-candidate.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- capacity-lint.log: 176 bytes; SHA-256 `f37da89d0654ad6823cb2ef62bf9717e020d0dc7e9c51fec7789d4ed0408e8bf`.
- capacity-lint.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- capacity-final.log: 1408 bytes; SHA-256 `5b8ed894820eebe854a3dce939235cc389f08ff831242bbe4029a858abd8a769`.
- capacity-final.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- capacity-lint-final.log: 176 bytes; SHA-256 `353d74617bd103ac833dcfc9631d90fb2e382a926eae3c959ef8b3ae6e1d616f`.
- capacity-lint-final.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.

Final formatting/diff/mdBook build pass, rc=0. Ten rendered capacity/recovery/limit
markers and three consistent JSON examples pass. README remains 52 lines/2054
bytes; LIVE_STATUS category values unchanged. Scheduled pre-push full CI is owned
by .11.4.3.1 after the clean capacity commit, before transport bounds resume.

The final escalated `python3 -B scripts/project_env.py bash scripts/check_no_background_jobs.sh`
census returns `handoff: OK`, rc=0; result consumed before REPAIR-0032 staging.
