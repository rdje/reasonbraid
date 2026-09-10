# Participant removal uses tenant-administration authority

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.8`; REPAIR-0050. Predecessor:
`576078dddcbd6d5821905123f9588a37ff7d55de`. Raw observations:
`target/participant-removal-controls`. The server target mapping, live HTTP controls and final lint/process/book
verification pass. The CLI companion remains pending.

## Root cause and historical correction

The affected invitation lifecycle returns 403 for its authorized administrator,
with the reason that tenant_admin does not support the target kind. The exact
original pre-fixture-migration test reproduces the same failure on a fresh owned
database; its receipt and stopped run-6o213qj7 remain under the previous fixture
qualification. Neither changing cleanup nor fresh database isolation fixes it.

Git blame identifies the evaluator's TenantAdmin→Tenant restriction in d7406e05
under .3.3.2, the removal operation's action mapping in d7b76733 and the handler's
unconditional Thread target in 35f395d9. The evaluator correction did not migrate
this older caller. Its prior focused authority/command tests did not run the
invitation lifecycle. The historical task now carries this correction; its
bounded original passes are not promoted to complete caller compatibility.

The explicit Rust operation/type census finds the HTTP action mapping, the domain
handler, the CLI envelope caller and the invitation tests. Domain preparation
locks aggregate_state with both tenant_id and aggregate_id before mutation. The
selected repair changes the administrative request target to Tenant while keeping
that locked lookup and all other thread actions unchanged. Tenant administration
still requires tenant-wide selection and live authority. This is not permission
to use a foreign tenant's thread or to grant administration through membership.

## Controls and limits

Two new live HTTP controls are private children of the existing invitations target;
the original four tests and runner registry remain unchanged. One control perturbs
thread-only selection, missing action, revoked/expired/future grant and suspended/
revoked parent, observes refusals and then restores valid authority for successful
removal. The second exercises a foreign administrator, a foreign thread, borrowed
caller authority and thread-only delegated scope before valid direct removal.

Each refusal compares complete aggregate, event, outbox/delivery, node-inbox,
budget-reservation and quota-event rows. New authorization rows are observed by
set difference from their prior IDs: authority refusals retain one tenant-target
denial; a missing-thread domain refusal rolls its provisional admission back.
Successful removal has one tenant-target allowance naming the actual grant and
an independently read revoked participant state. Existing lifecycle checks retain
accepted-participant removal and subsequent contribution refusal.

These controls do not establish concurrent revocation ordering, delegated consent
or depth enforcement. Those existing owners remain .3.3.4.4 and .3.4. The API's
development principal-header trust model is unchanged. No MCP transport, new
permission type or full-checkpoint qualification is implied.

## Runtime evidence

The two new controls run first against unchanged production and both fail because
the persisted target_kind is thread instead of tenant (zero pass/two fail/four
filtered, exit 101). Body: 18.43 seconds; command: 156.152901 seconds. Retain stopped
`target/pg-tests/run-mtrvlow8`. This independently complements the original
valid-administrator lifecycle failure; it is not a passing baseline.

The minimal handler change makes all six invitation tests pass (initial body
20.13 seconds, command 76.675273 seconds). Adjacent authority passes 22 tests
(0.37-second body, 35.943440-second command); command_api passes 33
(24.49-second body, 57.801897-second command). The evaluator/source, domain executor,
CLI code, schema, manifests and runner registry remain unchanged.

After that initial pass, the matrix adds an explicit historical-denial replay:
restore eligible authority, resend the denied key and verify the original 403 with
replayed=true, no new admission and unchanged domain rows; a fresh key then
succeeds. Initial test bytes are retained separately. All six final invitation
tests pass again, no skips/ignores, exit zero, in 13.72 seconds of test body.
Final command duration: 40.870236 seconds. Do not add that repeat to
the distinct selected live-test count: the final coverage is 6+22+33 = 61 tests.

The new controls exercise ten fresh authority refusals, one foreign-thread domain
refusal, one historical denial replay and two fresh direct allowances. The
remaining original lifecycle tests retain their separate assertions. The three
successful databases run-xhmph5yu, run-y3z4om17 and run-3iuxdxck under
target/pg-tests are removed after consumed shutdown; the failed baseline remains.
All original node-fixture failure evidence stays preserved.

## Final verification and reproducibility

Ten pure evaluator controls pass, 66 filtered, command 36.181750 seconds. Format
passes in 0.506758 seconds, strict all-target/all-feature server Clippy in
159.110521 seconds and book in 0.188000 seconds. All exit zero and their results
are consumed. These timings describe these runs only, not performance bounds.

The final native verifier exits zero: thirteen recorded groups are absent; the
three successful databases are removed, the new baseline and all four previous
fixture failures are stopped/preserved. Exact reconstruction allows only the
administrative target block and private test-module import; the original four
invitation tests and 337 other existing non-Markdown files remain unchanged.
Each live receipt binds all 340 then-present source files and retains 41 schema
FKs. Twelve rendered markers match across authority/CLI/deployment; README is
unchanged at 52 lines/2017 bytes and LIVE_STATUS categories remain unchanged.

Re-derive behavior with the tracked runner and tracked integration controls:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh invitations authority command_api
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-server --lib authority::evaluation_tests
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-server --all-targets --all-features --locked -- -D warnings
```

Falsification is the recorded unchanged-production baseline plus the independently
read audit/domain state and original historical lifecycle. Durable controls live
in the existing registered invitations target and adjacent suites. Raw timing,
process and exact-source receipts/verifier under target are local evidence, not a
portable regression gate; no future process-cleanliness or duration guarantee is
inferred from them. Full checkpoint, remaining fixtures and actual CLI delegation
remain required under their separate owners.

## Companion CLI scope

The main.rs removal command forwards to run_thread_verb in the CLI library, which
unconditionally uses a Threads selector for delegated existing-thread verbs.
That cannot cover the administrative Tenant target. The `.11.4.3.1.2.10` owner
will reproduce the actual CLI request and correct that one operation's scope
while retaining ordinary thread attenuation. No real CLI delegated-removal pass
is inferred from these HTTP tests; consent/depth and revocation ordering retain
their existing owners.
