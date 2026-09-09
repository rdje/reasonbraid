# Grant creation error qualification

Owner: `SIGNOFF-REPAIR.3.3.4.3.1`; baseline implementation `a74ca85`.

Final status: qualified for this child. All 56 live controls and focused strict
lint pass; results and shutdown consumed. All four owned clusters are absent.
The intermediate entries below preserve baseline and verification provenance.

## Contract and scope

Grant creation must distinguish an absent named parent, a structural ceiling
refusal and unavailable or malformed storage. The public Rust return type becomes
non-exhaustive `GrantCreateError`; `GrantRefused.violations` continues to describe
only actual structural failures. Storage keeps the original SQLx error as its
source. Enrollment and card import preserve their structural HTTP 400 context,
while storage faults use the existing generic HTTP 500 response. Checked decoding
must replace the active-boundary loader's panic.

This child does not integrate tenant guards or change development issuer policy,
parent liveness, import transaction scope, or automatic retries. Those have
separate owners in the parent and repair tree.

## Baseline controls in progress

Commands run through `python3 -B scripts/project_env.py`, with repository-local
stores and the owned PostgreSQL runner:

- `python3 -B scripts/run_pg_tests.py authority command_api cards`;
  runner stops at the first failing suite, so separate HTTP baselines follow.
- `python3 -B scripts/run_pg_tests.py command_api`.
- `python3 -B scripts/run_pg_tests.py cards` (sequential after command_api).

Local logs and exit receipts are in `target/grant-error-controls/`. Results are
pending; compilation is not runtime evidence. The product implementation has not
changed during baseline verification.

The repository controls exercise duplicate insertion, malformed parent data and
closed-pool acquisition independently, preserving the original row or proving no
new grant and recovery as applicable. Enrollment controls snapshot all seven
provisional tables, inject grant insertion failure for both bootstrap and existing
tenant enrollment, and separately corrupt the selected active boundary. Card
import snapshots seven local effect tables and injects grant insertion failure
after the valid agreement/card rungs. Fault DDL/data is restored before outcome
assertions; the successful controls retain their original request/response path.

## Consumed repository baseline

`run-ox5zuquk`: rc=101, 18 existing controls passed and all three new
error-preservation controls failed (0.32s execution; 5m27s build). Unique insertion
lost SQLSTATE 23505, closed-pool acquisition lost PoolClosed, and malformed parent
storage lost Protocol while claiming the existing parent did not exist. Both row
preservation/no-new-grant checks and repaired-parent recovery completed before
those outcome assertions. The runner stopped and retained its owned failure
cluster; census/removal remains pending.

A one-second sample of the delayed compiler (PID 16482) completed and was consumed:
731 samples follow rustc proc-macro loading through dlopen to dyld fcntl. The
read-only diagnostic itself completed slowly. This matches the previously owned
`.11.2` loader-wait observation, but does not establish the underlying operating
system/security-service cause. No host settings were changed. Repository volume
free space was 3.1 TiB (17% used). Local sample:
`target/grant-error-controls/baseline-rustc.sample`.

## Consumed enrollment baseline

`run-sdfkvhhi`: rc=101, 30 existing controls passed / two new controls failed
(19.68s execution; 5m20s build including the Cargo lock wait). Corrupt stored
actions panic at the active-boundary loader's expect; the request receives
IncompleteMessage instead of a safe HTTP error. Injected grant insertion failure
returns invalid_command 400 with false “exceeds its boundary” prose. Exact
seven-table snapshots are unchanged, fixtures are restored and both bootstrap and
role recovery succeed before the failed status assertions. The response loop's
first assertion reports bootstrap; the role response is independently checked in
the corrected run. The runner stopped and retained the owned failure cluster;
census/removal remains pending.

## Consumed card baseline and residue census

`run-zj8frm7r`: rc=101, the card ladder reaches the injected INSERT fault and
returns invalid_command 400 with false boundary-excess prose (12.38s execution;
10.70s build). Fault DDL was restored and all seven effect tables were unchanged
before the failed status assertion. No card import success is inferred from this
failed run; the final corrected run owns recovery and the full ladder.

All three baseline command outcomes and runner shutdowns were consumed before
cleanup. The exact retained clusters were inspected with an escalated process
census; each command log is byte-for-byte included in its retained outer log:

- `target/pg-tests/run-ox5zuquk`: stopped rc=101; 1611 files / 51593256 bytes; all entries same-volume and no symlinks, postmaster PID absent and no matching live PG process. command-1.log: 3084 bytes, SHA-256 e691dcbd355fede9d9a7faf0d1e893904c3333dad8d645ce4041834fdf9cb322; postgres.log: 3000 bytes, SHA-256 5524489ccd7cd6f899d508a18b53deef18323bc34c22f573898f46e0319fc469.
- `target/pg-tests/run-sdfkvhhi`: stopped rc=101; 1613 files / 52241016 bytes; all entries same-volume and no symlinks, postmaster PID absent and no matching live PG process. command-1.log: 7066 bytes, SHA-256 4e88ffd570f54bc5cba3eb0aba6ecb199ed9d7223db016429c5e02e3dcab0106; postgres.log: 15122 bytes, SHA-256 40e043021098c0efcb5cc1f5ca83ce74c62fba39263078553110c859bb821343.
- `target/pg-tests/run-zj8frm7r`: stopped rc=101; 1608 files / 51375317 bytes; all entries same-volume and no symlinks, postmaster PID absent and no matching live PG process. command-1.log: 1090 bytes, SHA-256 19bc144b4376c4b703df3ce95de72733c8c79f5abc6cdc2109d0c5646a792f1e; postgres.log: 2578 bytes, SHA-256 4a57e18df61845488906c9fed80ccda19c9c80fbebfb55f0d12b48451b08a1a5.

All three exact cluster roots were then removed and proven absent. No shared cache or ambiguous global data was deleted.

## Implementation and final qualification in progress

`GrantCreateError` is public, exported and non-exhaustive. MissingBoundary maps
only the named-parent fetch's RowNotFound; Refused retains the actual structural
violations; Storage retains the original connection/read/insert/decode error as
Error::source. The two HTTP consumers share a contextual mapping that preserves
structural 400 prose and sends storage failures through the existing safe 500
mapper. Active-boundary decoding uses checked Option/Result transposition.

`cargo check --locked -p reasonbraid-server --test authority --test command_api
--test cards` passed, rc=0 (1m06s). Final live suites and focused strict Clippy are
running; no runtime repair claim is made until their results are consumed. The
final controls also add typed missing-parent/no-row/recovery and real HTTP
structural-refusal cases for enrollment and card import. The card control keeps
the caller's administrator grant structurally valid while narrowing the parent,
so its 400 must come from the imported role's default grant rather than an earlier
administrator denial. Authorization admissions remain separate, intentionally
excluded from the import-effect snapshots; complete import effect integration is
still `.3.3.4.11`.

Focused strict lint completed, rc=0 (1m01s including a Cargo lock wait):
`cargo clippy --locked -p reasonbraid-server --lib --test authority --test
command_api --test cards -- -D warnings`. `cargo fmt --all --check` and
`git diff --check` pass. The book builds; its generated authority page was parsed
and checked for five error-contract, Rust example and scope-limit markers.
The final runner's authority suite now passes all 22 controls (0.32s); HTTP and
card results/shutdown are still pending, so this is not leaf closure.

## Final consumed result

`python3 -B scripts/run_pg_tests.py authority command_api cards`, through
project_env: `run-7_vfc8bu`, rc=0. All 56 live controls pass: authority 22 (0.32s),
command_api 33 (21.86s), cards one composite ladder/failure/recovery control
(16.25s). Build durations were 24.34s / 19.25s / 8.71s. Both bootstrap and role
insertion faults return exactly the generic 500 JSON. Corrupt active authority
returns 500 without a handler panic. Real structural enrollment/import failures
retain their specific 400 context. Snapshots remain unchanged on the injected
failures and the valid enrollment/import recovery succeeds. Existing admission,
receipt, revocation, delegation, audit-fault and card-ladder controls still pass.

The runner's success/shutdown output was consumed and run-7_vfc8bu is absent.
The earlier three failed clusters were already censused and removed. Focused
strict lint, formatting, whitespace and the generated-book checks pass. No full
CI or push was run. There is no outstanding verification job.

### Retained local evidence digests

- `target/grant-error-controls/baseline.log`: 3346 bytes; SHA-256 `7678f17abb03382701d8cf2e8c74b78bfc8ac741d718a95ef4f8693f353fd11d`.
- `target/grant-error-controls/baseline_command_api.log`: 7330 bytes; SHA-256 `4f58dd03bc59336e3caf8b7da8b352d1e2957e21e571c43a7c6462d2c90956cb`.
- `target/grant-error-controls/baseline_cards.log`: 1348 bytes; SHA-256 `cb7e1469cf5c0eedb8824713e2d2f64f3ea6d253a8d7bea6c199aa7bf3a6f178`.
- `target/grant-error-controls/baseline-rustc.sample`: 80012 bytes; SHA-256 `3b69a94a4d6eed23cac7580acdcbdd2dbeadf68841b5fd700390f83e4dba455f`.
- `target/grant-error-controls/final.log`: 8871 bytes; SHA-256 `af7641e10e394197d743b540a3d920cf66d635d093a128719a7c25459fe1c50f`.
- `target/grant-error-controls/lint.log`: 236 bytes; SHA-256 `fd03f72898bb598e5ae85034e0122da6a3429541501a1c779485ab5041723b14`.
