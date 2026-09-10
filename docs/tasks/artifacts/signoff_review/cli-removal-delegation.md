# CLI participant-removal delegation scope

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.10`; REPAIR-0051. Predecessor:
`f63795556467ca26cec85eb290ff8a1499bfd218`. Raw evidence:
`target/cli-removal-controls`. Final focused qualification passes; full checkpoint remains incomplete.

## Reproduction and history

The server's administrative target correction preserves TenantAdmin requiring a
Tenant target and tenant-wide grant coverage. The CLI's generic run_thread_verb
helper still sends Threads for every delegated existing-thread operation. Git
blame binds this scope and the removal delegation flags to b62f6a88 under
PHASE-2.1.4.2; the original removal dispatch is d7b76733. The historical delegation
leaf now carries the compatibility correction without erasing prior scoped results.

Two new private controls join the existing cli_end_to_end target, leaving its
three original tests and runner registry unchanged. All user requests invoke the
actual rb binary against the real in-process API and owned PostgreSQL. Fixture
setup explicitly permits delegation in its source grant/boundary; this is not
qualification of grant issuance or consent enforcement. Independent SQL reads
observe actual authorization and domain effects, alongside real CLI inspection.

Against unchanged production, the ordinary one-thread invitation control passes;
the authorized delegated removal fails with a 403 delegation-scope refusal.
The test target exits 101: one pass/one fail/three filtered, no ignores or skips.
Body time 17.51 seconds; command 40.639553 seconds. The stopped failure database
is target/pg-tests/run-t4xbqo5e. This is actual CLI evidence, not inference from
an HTTP-only test.

Two earlier setup failures are separate: the local driver's unregistered cli key
fails before any Cargo command (empty stopped run-hkq8ugfz), then a new test's
JsonValue/&String comparison fails to compile (stopped run-8qt01f28). The registry
key is corrected to cli_end_to_end and preflighted; the comparison uses &str.
Both receipts are preserved. Compilation/startup without test results cannot
count as a product reproduction. The post-compile snapshot's missing tables
confirmed migrations had not run; no runtime baseline is claimed for that attempt.

## Selected correction and controls

Match the requested authorization target in the CLI: removal uses TenantWide,
other existing-thread verbs retain exactly the requested Threads selector. No
server/evaluator/schema or protocol representation changes. Thread creation's
existing TenantWide scope remains untouched.

The live removal matrix includes successful delegated and direct removal,
insufficient caller permission, missing source action, thread-limited source,
revoked source, a foreign delegated source and a foreign thread. Each refusal
compares complete aggregate, event, delivery/outbox, node inbox, budget-reservation
and quota-event rows; CLI state is also preserved. Admissions are read by record-ID
set difference. Successful delegation names the actual source grant and subject;
direct success has no delegated subject. The foreign-thread domain refusal rolls
its provisional admission back.

The separate invitation control uses a grant confined to one thread, proves
invitation there and refusal on another. A deliberate all-TenantWide mutation
must fail that actual CLI control before exact restoration of the intended source
and final checks. Broader delegation consent/depth and concurrent command/revocation
ordering retain .3.4/.3.3.4.4 ownership. The development principal-header model,
transport bounds and full-checkpoint status are unchanged.

## Final results and durability

The deliberate all-TenantWide mutation fails the ordinary one-thread invitation
control with 403 delegation scope: zero pass/one fail/four filtered, exit 101;
16.90-second body and 46.601166-second command. The source restoration receipt
binds the exact selected library bytes before final tests. Retain stopped
run-8psce4a0. This separates the removal-specific correction from accidentally
broadening every delegated thread verb.

All five final real CLI tests pass, including the two new matrices and three
unchanged original controls: 30.52-second body, 74.196326-second command. All six
adjacent server invitation tests pass: 17.67-second body, 28.297274-second command.
No skips/ignores. The eleven distinct live tests are separate from eleven CLI
library tests (0.63-second body, 31.276452-second command). Format, strict all-target/
all-feature CLI Clippy and book pass, respectively 0.828131, 49.382412 and 0.246369
seconds. These are observed command durations, not performance bounds.

The final independent verifier exits zero: fifteen recorded groups are absent;
successful run-3y2nedv_ and run-tf_pspcq are removed; the two expected runtime
failures and two preliminary setup failures remain stopped. All five prior
server/fixture failure databases remain stopped/preserved. The only production
difference is the selected CLI scope mapping/comments; 338 other existing
non-Markdown files and the three original CLI tests are unchanged. Server,
evaluator, schema, manifests and runner registry are unchanged. All final/live
receipts bind 341 source files, and 41 public FKs remain. Eleven rendered book
markers match, README stays 52 lines/2017 bytes and progress categories do not
advance.

Re-derive with tracked test code and the tracked runner:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh cli_end_to_end invitations
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-cli --all-targets --all-features --locked -- -D warnings
```

Falsification includes the unchanged-CLI baseline, overbroad-scope mutation and
independent actual database effects/CLI inspection. Durable regression controls
are private children of the already registered CLI target. Raw timings, source
hashes and process/residue verifier under target are local observations, not a
portable future-cleanliness or performance gate. This leaf does not turn the
previously failed full checkpoint into a passing run; retention and remaining
fixture prerequisites still precede a new complete checkpoint and push.
