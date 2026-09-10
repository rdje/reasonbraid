# Complete the six partial fixture dependency plans

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.3`; REPAIR-0053. Predecessor:
`afbf7c49fceddce6e0f9d639098baa129f61a52d`. Raw diagnostics and exact plans:
`target/partial-fixture-controls`. All six original residue baselines are consumed; source migration is complete.
All paired/consecutive live runs and strict/final verification pass.

## Scope and root cause

The six existing partial plans are cards, quota, classification, mcp_listen,
federation and quarantine. They delete tenant/role parents without declaring all
of their incoming dependencies. Prior node_work→mcp_listen reproduction already
failed at incarnations_role_id_fkey. The new collection gives each unchanged
consumer its own owned cluster after the real eight-test node_work producer.
The producer leaves actual node, certificate, host, incarnation and role rows.
All six consumer baselines each fail with PostgreSQL 23503
at incarnations_role_id_fkey before their feature assertions. Preserve their
stopped databases; do not treat a clean initial database as a residue control.
Every producer passes eight tests, followed by one consumer failure. Those eight
producer tests repeat six times; they are not 48 distinct controls. All 341 source
hashes match the untouched baseline, and all twelve actual schema observations
match the selected 41-FK graph before any migration.

The selected source plans derive the minimum incoming-FK closure from a verified
41-FK schema and will require an identical schema observation in every new
baseline receipt before edits. Keep every original table and its relative order.
Add only required children, then topologically order against live constraints and
those original order edges. Each final static array remains explicit; the shared
checker validates it but never expands its deletion scope at runtime.

The additions cover node/certificate/key/lease/incarnation/run/recruitment/listener
relations, and missing federation/breaker relations where those original plans
need them. They do not add unrelated resource tables, deployment CA or audit
records simply to make fixtures look alike. Original plans have 24/20/24/25/23/22
tables respectively; selected closures have 37/35/37/37/37/37. These are plan sizes,
not counts of defects. All feature bodies, production constraints and registry
remain outside the selected code change.

## Qualification contract

Consume all unchanged producer/consumer failures and their schema/source binding
before applying imports/static arrays. Then re-run each real node_work→consumer
pair, observe the dependency rows cleared and the deployment CA preserved, and
run the six consumers consecutively. Independently reconstruct source outside
imports/cleanup blocks, reconcile the remaining explicit-loop census, run strict
focused lint/book and verify owned process/database disposition. Any newly exposed
feature failure gets a concrete owner and original-source comparison; never
rewrite an assertion to disguise it. Remaining complete-plan adoption and broader
coverage stay .2.7.4 before the still-incomplete full checkpoint and public push.

## Source migration and coverage

All six callers now import the private shared checker and replace only their
DELETE loops with explicit checked arrays. Exact reconstruction recovers each
original file after removing that import and restoring its loop. Every original
feature assertion and table relative order is unchanged, along with 335 other
existing non-Markdown files. Production/schema and runner registry are untouched.

The rederived original 25-plan population now contains twenty checked callers
and five legacy loops: regions, allowlist, rls, mcp_write and the
MCP crate's internal fixture. The actual source-pattern census matches that
remaining set in both directions. This population is literal-array DELETE loops,
not all direct or implicit fixture relationships. The five remaining callers and
broader affected qualification belong to .2.7.4; no full-checkpoint success follows
from the current source migration.

## Repaired producer/consumer runs

All six pairs pass: the existing eight-test node_work producer, then the original
single consumer test. For every pair, independently observed node, certificate,
host and incarnation counts fall from one to zero, while the deployment CA stays
at one with the same complete-row fingerprint. No feature assertion was changed.
All six successful pair databases are removed after consumed shutdown; preserve
the six stopped baseline failures.

| Consumer | Final tests passed | Test body seconds | Command seconds |
| --- | --- | --- | --- |
| cards | 1 | 16.67 | 39.629345 |
| quota | 1 | 17.80 | 40.714686 |
| classification | 1 | 18.31 | 41.189880 |
| mcp_listen | 1 | 0.03 | 22.589624 |
| federation | 1 | 18.32 | 41.143042 |
| quarantine | 1 | 18.09 | 40.960324 |

The command observations include compilation/startup; they are not performance
bounds. Repeated producer runs cover the same eight tests; do not count them as
new tests for each consumer. The consecutive consumer run also passes all six tests, including the internal
listener state observed present after mcp_listen and absent after federation.
Its seventh successful database is removed. Final coverage comprises fourteen
distinct live tests (eight producer plus six consumers), with sixty total final
test executions across the repeated pair and chain runs. Strict lint/book and
final source/process reconciliation pass as recorded below.

## Reproduction and durability

The unchanged failures are tied to afbf7c4 and exact per-command source hashes.
Re-derive each corrected pair with the tracked owned runner, replacing cards
with each named consumer, then run the consecutive collection:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh node_work cards
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh cards quota classification mcp_listen federation quarantine
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-server --all-targets --all-features --locked -- -D warnings
```

Falsification is PostgreSQL's original restrictive-FK failure with actual residue;
independent final row counts and CA fingerprints observe the correction. Durable
controls are the original registered producer/consumer tests plus the shared
pre-deletion graph checker and its prior adversarial controls. Local ignored
source/timing/process receipts and verifier are historical observations; they are
not a portable future-cleanliness or performance guarantee. Full checkpoint and
remaining coverage stay separately owned. No MCP wire/agent qualification is
inferred from these internal/database fixtures.

## Final verification

Format, strict all-target/all-feature server Clippy and book pass, all exit zero:
0.513257, 17.063604 and 0.186246 seconds respectively. The independent native
verifier exits zero: 46 recorded groups are absent, seven successful databases
removed and six original failures stopped/preserved. Exact source reconstruction
confirms only six imports/cleanup lists changed, all original table order retained,
and 335 other non-Markdown files unchanged. Every selected plan is precisely its
original scope's incoming-FK closure under the identical 41-FK observations.
The original 25-plan population reconciles to twenty checked/five legacy callers
in both directions. Nine rendered book markers match, README remains 52 lines/
2017 bytes and LIVE_STATUS categories remain unchanged. Full checkpoint/push
remain incomplete; .2.7.4 owns the remaining five plans and broader regression.
