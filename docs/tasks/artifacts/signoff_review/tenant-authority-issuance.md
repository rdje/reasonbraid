# Standalone authority writer integration

Owner: `SIGNOFF-REPAIR.3.3.4.3.2`. Baseline implementation: `eb1f080`.

Final status: qualified for this leaf. All 85 selected controls (84 live / one
pure) and final focused strict lint pass. Every result and shutdown is consumed;
all three owned clusters are absent. The historical entries below retain the
matched failures and the compatibility control's observed lock-graph correction.

## Baseline setup (historical)

The production writer code is unchanged. Nine new live controls in
`crates/reasonbraid-server/tests/authority_issuance.rs` are running through
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py authority_issuance`.
The owned runner is `target/pg-tests/run-_48kbvf2`; outer log/exit evidence is in
`target/authority-issuance-controls/`. Results and shutdown are pending.
The initial test-only strict Clippy check passed (17.27s); final strict lint must
include the subsequent join/fixture-restoration refinement and implementation.

The tests observe actual PostgreSQL blockers and query fragments, consume writers
after releasing their blockers, and abort/consume a timed-out join instead of
detaching it. Fault DDL/status restoration precedes result assertions. No result
from compilation alone is treated as runtime reproduction.

## Owned contract

- Standalone boundary/grant creation and grant/boundary status writers acquire
  the exclusive full-tenant guard before authority reads or target locks and keep
  it through commit. Active-boundary lookup uses the shared mode. Coordination
  anchors create no identity or permission; unrelated tenants still progress.
- Issuance first preserves the existing structural checks, including status and
  exact tenant/parent binding. It then requires a currently live named parent at
  fresh database time after preceding guard/authority waits. Scheduled grants
  under a currently live parent remain supported; no new grant-own-now check.
- Scope the full issuance policy-row load to the guarded tenant. An absent own
  row may use a minimal foreign-ID existence check solely to preserve the existing
  tenant-binding refusal; it must not decode or evaluate foreign policy.
- Public AuthorityTransactionError preserves storage, pre-commit deadline and
  unconfirmed commit distinctions. create_boundary's Rust return type migrates;
  GrantCreateError retains original SQLx Storage sources and gains transaction
  and current-time parent-refusal outcomes. A commit acknowledgment error must
  not become a rollback receipt or trigger an automatic retry.
- Historical `.3.2` adapter: actual missing/structural/liveness refusals were
  ordinary callback results. `.3.3.1` supersedes that adapter with typed errors
  that abort provisional anchors; see docs/tasks/artifacts/signoff_review/typed-rollback-errors.md.
  SQL and transaction faults always leave as errors, preserving their original cause.
- Status decoding must preserve malformed target evidence and leave the epoch
  unchanged. Migration 0004 has unchecked TEXT status; both baseline services
  use parse().ok() and treat an unknown value as eligible for revocation. Matched
  HTTP controls keep the caller valid and make only a separate target malformed.

## Qualification matrix

1. Existing shared guard blocks standalone boundary creation; another tenant
   progresses and neither standalone namespace gains an identity.
2. Existing exclusive guard blocks the active-boundary lookup used by enrollment;
   a committed freeze is then observed. This qualifies that lookup only.
3. Expired and future active parents cannot issue; scheduled grants under a live
   parent remain structurally supported.
4. A parent expiring during an observed guard wait cannot issue after that wait.
5. Real boundary revocation obtains its target lock first; later issuance must
   wait on its tenant guard and then refuse.
6. Real issuance is paused inside its INSERT; boundary revocation waits on its
   tenant guard until issuance commits, then revokes the still-attributed parent.
7. Both grant and boundary status services wait on the guard and bump one epoch.
8. Unknown status on a separate grant/boundary target returns safe 500 with exact
   target/epoch preservation; restored valid status permits normal revocation.
9. Deferred boundary constraint failure retains its commit phase, rolls back
   provisional boundary/anchor rows, and permits recovery after fixture removal.

Existing authority, primitive transaction, HTTP/card and selected upgrade controls
will qualify compatibility. Public grant transaction mapping needs a matched
commit-failure case as implementation is wired. The existing primitive control
retains actual committed data after COMMIT acknowledgment timeout, so no blanket
rollback-on-error claim is allowed.

Enrollment and import still have temporary unordered insertion executors; their
complete transactions migrate in `.3.3.4.3.3` and `.3.3.4.11`. Node-certificate
epoch writes retain `.3.3.4.10`/`.4.1`. HTTP administrative admission is separate
until `.3.3.4.8`; this leaf does not qualify caller-admission/effect coupling.

## Consumed baseline result

run-_48kbvf2 returned rc=101: zero passed / all nine matched controls failed
(41.51s execution; 17.66s build). Both boundary and status writers bypassed held
tenant guards. Existing-tenant lookup created a role before the guarded freeze.
Revocation ordered at its target first still permitted a later grant INSERT;
issuance paused in its INSERT allowed boundary revocation to finish first.
Expired and future parents both issued a grant, and queued issuance bypassed the
guard. Both malformed target kinds returned 200 and changed target/epoch; restored
valid status permitted recovery. Deferred boundary failure rolled back its row,
but the standalone autocommit API did not preserve a distinct commit phase.
All writer handles and runner shutdown were consumed; the exact stopped failure
cluster remains pending census/removal. No product implementation changed during
this baseline.

The new private issuance module is now explicitly owned by the leaf. Liveness is
the guarded evaluation after guard and parent lookup, immediately before INSERT;
it is not a claim that time stops during commit or that the parent remains live
at response delivery. Later grant use still evaluates both live windows normally.

## Baseline residue census

`target/pg-tests/run-_48kbvf2`: 1608 files / 51383611 bytes, all same-volume and no symlinks; stopped rc=101, absent postmaster PID/file and no matching PG executable.
- command-1.log: 5714 bytes; SHA-256 `401f3d6d56a4faba760928afaf344b254d48c4f50f534b01e234cdae52d02f03`.
- postgres.log: 2377 bytes; SHA-256 `bdf2dcf163cd0db8e19be46cc045359681d276c91e4cd2ecc414159f9f8fd764`.

The command log is byte-for-byte present in the retained outer baseline log.
The initial conservative process matcher refused its own shell command before any mutation; the corrected check identifies the actual executable and exact old postmaster PID.
After evidence preservation the exact stopped cluster was removed and proven absent. The independent corrected run remains untouched.

## Initial integrated verification in progress

The first integrated eleven-control issuance suite passes (41.30s), including
the added grant deferred-commit/source-chain control and foreign-policy refusal
under an occupied foreign guard. The primitive transaction suite passes all
fourteen controls (4.21s), and all twenty-two existing authority controls pass
(0.27s). Focused strict Clippy for the library and all six selected test targets
passed, rc=0 (1m07s including the Cargo lock wait). The runner remains in flight
on HTTP compatibility; these are consumed component results, not final closure.

Public HTTP commit-outcome differentiation and its deferred-update controls are
owned refinements still pending implementation. A read-only graph observer is
checking the exact live runner identity before collecting the older contention
fixture's new target→tenant-guard dependency; no policy/state writes are performed
by that observer.

## Consumed compatibility observation

The initial integrated run run-4g9ckujd returned rc=101 after passing issuance
11, primitive 14 and authority 22; command_api passed 32 and failed the one legacy
contention wait (51.82s). Cards and upgrade were not reached. The old test required
two waiting queries containing authority_grants and timed out before releasing its
held target. The independent read-only observer verified this runner's exact live
directory, database and owner token, and captured the actual graph:

- Grant-row revoker PID 29880 waits on held-target PID 29881.
- Tenant-guard waiter PID 29891 waits on revoker PID 29880.

Both use transactionid waits; the second query is SELECT tenant_id FROM
tenant_authority_guards ... FOR NO KEY UPDATE. This is the required serialization
chain, not two direct target-row waits. Graph: target/authority-issuance-controls/legacy-wait-graph.json.
Observer and runner results/shutdown are consumed; exact failed-cluster cleanup
remains pending. The corrected control requires two transitive dependents rooted
at its actual held target, one target query and one guard query. It releases and
consumes all requests before assertions, retaining one-200/one-409/one-epoch checks.

The HTTP error mapper now exposes Commit/CommitDeadline as safe 500
commit_outcome_unconfirmed with an inspect-before-retry message. Ordinary storage
errors keep dependency_unavailable. A twelfth control injects deferred UPDATE
failure in each real status route, restores DDL before response assertions, checks
the exact target/epoch snapshot and valid recovery. Final verification is pending.

The post-refinement strict Clippy command for server lib, authority_issuance and
command_api passed, rc=0 (42.73s). The final selected run uses delivery.log / 
delivery.exit; final lint uses delivery_lint.log / delivery_lint.exit. It includes
all six selected suites and the twelve-control issuance suite. The primitive's
module comment now explicitly limits foreign-ID identification to a binding
refusal, matching the scoped policy loader and its held-foreign-guard control.
No foreign policy evaluation or mutation is authorized by that probe.

## Initial integrated residue census

`target/pg-tests/run-4g9ckujd`: 1633 files / 52473337 bytes, all same-volume and no symlinks; stopped rc=101, absent old postmaster PID/file and no matching PG executable.
- command-1.log: 1714 bytes; SHA-256 `c06fd3af20764dfcd03556750f0fbe06f7e8ecfb6fcd403a365a03ab77249812`.
- command-2.log: 1496 bytes; SHA-256 `c9009815ef03d9fb8dbe082cabc9d4d2eba822f70c4c8596fb2ebd8be7bd0be5`.
- command-3.log: 1833 bytes; SHA-256 `2b600ef51c9dfe6e968ff23c38ae69613337ce38a741c76a78dd24170045f01c`.
- command-4.log: 6590 bytes; SHA-256 `21f6df3df9bae566678ea7e285a7a73e2f9c55a60acb72e9383ddcea05f5d565`.
- postgres.log: 19131 bytes; SHA-256 `ef893c45814d45c76a984101463f7fee0c1e1a7a4918148befc5fae08d34cb2b`.
- `target/authority-issuance-controls/legacy-wait-graph.json`: 496 bytes; SHA-256 `ebc998345828909954c437e2a83f57d1e560b87f1808150a13c4b5a13850b771`.

All four command logs are byte-for-byte present in the retained outer final.log.
After evidence preservation the exact stopped cluster was removed and proven absent. The final delivery run remains untouched.

Final delivery progress: all twelve issuance controls pass (18.48s), all fourteen
primitive controls pass (4.25s), all twenty-two authority controls pass (0.27s) and
all thirty-three HTTP controls pass (49.16s), including the corrected contention
control. Cards, upgrade and runner shutdown are pending. Final strict lint for
all six test targets and library passes, rc=0 (1m19s). The generated authority
page contains the five checked service/error/uncertainty/timing/scope markers.

## Final consumed delivery result

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py
 authority_issuance authority_transaction authority command_api cards migration_upgrade`
returned rc=0. run-8myeaect passed all 85 controls (84 live / one pure):

| Suite | Passed | Execution | Build |
| --- | --- | --- | --- |
| authority_issuance | 12 live | 18.48s | 42.11s |
| authority_transaction | 13 live + one pure | 4.25s | 6.58s |
| authority | 22 live | 0.27s | 22.88s |
| command_api | 33 live | 49.16s | 19.79s |
| cards | one live composite | 30.10s | 20.92s |
| migration_upgrade | three live | 30.51s | 18.01s |

The HTTP deferred-commit control covers both status routes: exact safe
commit_outcome_unconfirmed response, unchanged target/epoch for the injected
abort, and successful recovery. The corrected contention control observes the
actual target→first revoker→second guard waiter dependency and retains one 200,
one 409 and one epoch increment. Existing original SQL causes, structural
refusals, audits/receipts, card ladder and all upgrade snapshots remain correct.

Final strict Clippy for the library and these six targets passes, rc=0 (1m19s).
Formatting, whitespace and book build/generated-page markers pass. The delivery
runner's completion/shutdown is consumed and its exact root is absent; both
failed roots had already been censused and removed. No background result remains.
No full workspace CI or push was run. Complete enrollment/import, caller policy,
node epoch and HTTP admission/final-effect integration retain their named owners.

### Retained local evidence digests

- `target/authority-issuance-controls/baseline.log`: 5985 bytes; SHA-256 `f1afdb24cf0bb0316a78e306ae57e9010d9e38280fc2fd7e5d9fb3f9d643c4a5`.
- `target/authority-issuance-controls/final.log`: 12149 bytes; SHA-256 `de6c04654b9ce40f60ae46c830aaf83a996d8d3b5b50a36efa3990c1544e8de4`.
- `target/authority-issuance-controls/lint.log`: 236 bytes; SHA-256 `1636f7762f518ece929ba89a07d744326e52827130ba90bc20febd46ffe08631`.
- `target/authority-issuance-controls/legacy-wait-graph.json`: 496 bytes; SHA-256 `ebc998345828909954c437e2a83f57d1e560b87f1808150a13c4b5a13850b771`.
- `target/authority-issuance-controls/delivery.log`: 13326 bytes; SHA-256 `5d3de927b5f5dac7e17ea141dd7a135bc0c293a44004df58285e4f4ab00744a9`.
- `target/authority-issuance-controls/delivery_lint.log`: 236 bytes; SHA-256 `f4431fc90a30020008c6408c4689ef7911dc0c05042ba3e599a0bffe1731be4b`.
