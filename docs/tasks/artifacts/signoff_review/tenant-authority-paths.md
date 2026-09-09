# Tenant authority/effect path census

- Owner: `SIGNOFF-REPAIR.3.3.4.1`.
- Status: source census and implementation contract complete; no new runtime qualification.
- Source baseline: `1ba6184`; this child changes documentation only.
- Resume: implement `.3.3.4.2` only after the design leaf commits and clean postconditions pass; subsequent children integrate the qualified guard into each named path.

## Method and limits

The first pass enumerates tracked crates/*/src Rust files, then matches direct calls
of twelve named authority functions, excluding matching function declarations and
whole comment lines. The enclosing-function label is the latest source function
heading, not a Rust AST result. This is a bounded lexical census; aliases, dynamic
SQL, separately implemented gates and transitive consumers require the second pass.
Do not treat it as a complete security boundary or a runtime race reproduction.

Command families: `git ls-files`; `rg -n` for authority entrypoints and tenant
boundary/grant INSERT/UPDATE plus revocation_epoch; Python line enumeration over
that tracked source set. All tools run through scripts/project_env.py. The
mechanical table is retained here so additions/omissions can be reviewed independently.

The direct-name pass read 101 tracked Rust source files and found 42 matching call lines, rc=0.

| Source | Enclosing function | Named call |
| --- | --- | --- |
| `crates/reasonbraid-server/src/api.rs:828` | `enroll` | `insert_boundary_in_tx` |
| `crates/reasonbraid-server/src/api.rs:872` | `enroll` | `create_grant_in_tx` |
| `crates/reasonbraid-server/src/api.rs:994` | `issue_node_enroll_token` | `authorize` |
| `crates/reasonbraid-server/src/api.rs:1063` | `authorize_tenant_admin` | `authorize` |
| `crates/reasonbraid-server/src/api.rs:1100` | `replay_command` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1187` | `quarantine_command` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1278` | `inspect_node_inbox` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1342` | `prune_node_inbox` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1412` | `revoke_node` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1453` | `revoke_node` | `bump_revocation_epoch` |
| `crates/reasonbraid-server/src/api.rs:1486` | `propose_federation_agreement` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1517` | `accept_federation_agreement` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1540` | `revoke_federation_agreement` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:1626` | `register_resolver` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:3513` | `create_thread_auto` | `authorize` |
| `crates/reasonbraid-server/src/api.rs:3691` | `open_recruitment_call` | `authorize` |
| `crates/reasonbraid-server/src/api.rs:3924` | `close_call` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4027` | `inspect_call` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4110` | `directory_match` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4224` | `directory_presence` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4372` | `authorize_profile_read` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4487` | `classify_reader` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4629` | `import_profile_card` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4668` | `import_profile_card` | `create_grant_in_tx` |
| `crates/reasonbraid-server/src/api.rs:4785` | `attest_capability_claim` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4832` | `revoke_grant` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4839` | `revoke_grant` | `revoke_grant` |
| `crates/reasonbraid-server/src/api.rs:4864` | `revoke_boundary` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:4871` | `revoke_boundary` | `revoke_boundary` |
| `crates/reasonbraid-server/src/api.rs:4905` | `inspect_tenant_admin` | `authorize_tenant_admin_inspection` |
| `crates/reasonbraid-server/src/api.rs:5205` | `arm_breaker` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:5227` | `reset_breaker` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/api.rs:5359` | `run_thread_command` | `authorize_in_tx` |
| `crates/reasonbraid-server/src/api.rs:5776` | `apply_node_result_in_tx` | `authorize_in_tx` |
| `crates/reasonbraid-server/src/api.rs:6166` | `inspect` | `authorize` |
| `crates/reasonbraid-server/src/api.rs:6631` | `list_cross_domain_receipts` | `authorize_tenant_admin` |
| `crates/reasonbraid-server/src/authority.rs:156` | `create_boundary` | `insert_boundary_in_tx` |
| `crates/reasonbraid-server/src/authority.rs:214` | `create_grant` | `create_grant_in_tx` |
| `crates/reasonbraid-server/src/authority.rs:607` | `authorize` | `authorize_in_tx` |
| `crates/reasonbraid-server/src/authority.rs:787` | `apply_authorized_command` | `authorize_in_tx` |
| `crates/reasonbraid-server/src/authority.rs:852` | `revoke_grant` | `bump_revocation_epoch` |
| `crates/reasonbraid-server/src/authority.rs:892` | `revoke_boundary` | `bump_revocation_epoch` |

## Source observations verified so far

- `authority::authorize` commits its admission before returning. `authorize_tenant_admin` returns only unit after that commit, so later administrative mutation transactions cannot retain its authority locks even if the admission gains them.
- `authorize_in_tx` selects caller/delegated sources and records admission on its caller executor. The current selection queries do not lock authority against status changes. Existing same-transaction domain writes therefore do not alone establish revocation ordering (prior source record R-36-39-4 remains the provenance; runtime race controls belong to implementation).
- `create_boundary` and `create_grant` acquire bare pooled connections; the helpers named in_tx also accept generic executors. Their naming does not prove multi-query atomicity or a transaction guard. Grant creation checks structural ceilings; liveness/issuance authority need explicit treatment.
- New-tenant `enroll` inserts tenant/default quotas/boundary/grant in one transaction. Existing-tenant enrollment loads its active boundary through the pool outside that transaction, then inserts the grant inside. Grant issuance is explicitly dev-trusted here, with a synthetic issuer for roles; caller/issuer binding remains its existing identity/administration owner. The ordering design must still fence grant creation against a frozen parent.
- Grant/boundary revocation locks the matching target and updates tenant epoch in one transaction (the completed .3.1 correction). HTTP caller admission precedes that transaction; submitted reason is validated but not passed to the mutation service. The target-row lock is not caller-authority ordering or final-effect evidence.
- Node revocation admits separately, checks node tenant outside the mutation transaction, then changes active certificates and epoch together. No-op with zero changed certificates returns conflict before epoch update. Node inbox replay/quarantine/prune also admit separately; their node-only SQL predicates and replayed cached-decision semantics remain separately owned .3.5/.3.4 defects, not fixed by an ordering guard alone.
- Federation propose/accept/revoke admit then call pool-based services. Resolver registration similarly admits against a tenant but writes the shared resolver registry; the existing .7.1 authority-scope owner must resolve that shared scope before any global ordering guarantee.

## Additional traced paths

- Thread HTTP create/command and MCP respond converge on run_thread_command. It begins a transaction, claims idempotency first, then samples application time and authorizes; prepared state, event/result, quota and inbox/budget writes follow in the same transaction. The guard must precede idempotency/domain locks, while a committed replay remains a replay rather than a new effect. Automatic creation also does an earlier standalone authorization, profile/spend checks and optional routing record before entering this core; the early checks/receipt must not be represented as final serialized authorization.
- node_channel::events verifies fencing outside the transaction, then locks node_leases, inserts the event receipt and invokes apply_node_result_in_tx. That callback claims idempotency, authorizes, writes an optional run, prepares/applies thread state and settles reported usage. A tenant guard inserted only inside the callback would come after the lease lock; outer events must acquire it before lease/event/idempotency/domain locks. The callback's error-catching/partial-result and unbound reservation behavior remain .4.2/.4.4/.4.5 owners, with ordering integration separately tracked here.
- Node token redemption already locks the token and commits host/node/key/certificate/incarnation/used-token/audit facts together. Rotation writes certificates separately after proof checks. Their issuing-authority and certificate/lease ordering repairs remain .4.1/.4.2 and must use the tenant guard before token/certificate/lease locks when integrated.
- Profile import admits, checks agreement/boundary through pool reads, commits identity/grant/enrollment/cross-domain receipt, then writes profile in another transaction. Attestation admits then reads a current profile outside the new-version transaction. Profile write takes an unlocked current version and updates its pointer; card atomicity/provenance and profile version races remain .5.1/.5.3 source findings. Self profile updates currently use identity equality rather than the tenant-admin helper; preserve that distinction for the authority-policy owner.
- Recruitment open admits then creates call and offers separately; close permits initiator or successful tenant-admin check, computes the panel and uses two independent service writes. The initiator alternative also treats storage errors from the admin check as false. Exact grant/tenant eligibility, offer/panel atomicity and bounds are already .5.2/.5.3 work; they need the guarded transaction interface at their eventual authorized effect boundary.
- Directory match/presence, profile-read/classification, call inspection, cross-domain receipt list and ordinary thread inspect use the normal boundary-checked helper or its direct authorize equivalent, sometimes with fallback visibility or initiator/self paths. They are not the eight frozen-tenant inspections. A shared authority guard can make each admission consistent; their subsequent queries/visibility fallbacks remain separate scope and failure-semantics work.
- Breaker arm/reset are standalone admission followed by pool writes. They can provide a small early administrative-effect integration control independent of node/recruitment complexity; current targeting is tenant-bound, while breaker-value/no-op semantics remain existing budget/administration scope.

## Guard design constraint from actual schema

Migration 0004 defines enrollment_boundaries, authority_grants and authorization_records
tenant_id as TEXT without a foreign key to tenants. The standalone authority tests
construct boundaries/grants for typed tenant IDs without inserting identity rows.
Using tenants as the mandatory lock anchor would silently change that API contract.
This is a schema/API compatibility observation, not a new FK-defect claim.

Choose a dedicated tenant_authority_guards table keyed by the complete tenant ID,
with no identity FK or implicit grant. Backfill known namespaces and provision a
missing lock row transactionally when the authority API sees a new typed tenant.
The guard row is coordination only: it neither enrolls a tenant nor grants authority.
Implementation must qualify concurrent first use and preserve standalone denials.
A shared guard covers ordinary authority evaluation through its protected local
commit; exclusive guard covers boundary/grant issuance/status and epoch-changing
operations. Mode is chosen before other locks; no shared-to-exclusive upgrade.
READ COMMITTED queries after the guard wait observe committed authority changes;
database decision time is sampled after acquiring the guard. Full behavior and
contention tests remain unimplemented.

Primary reference checked read-only: PostgreSQL 16 explicit-locking row-lock matrix
and transaction-isolation READ COMMITTED semantics. FOR SHARE is compatible with
itself and conflicts with FOR NO KEY UPDATE; locks last to transaction end (subject
to savepoint rollback). Consistent lock order and strongest needed mode first are
required. Sources: https://www.postgresql.org/docs/16/explicit-locking.html and
https://www.postgresql.org/docs/16/transaction-iso.html. These documented primitives
support a design inference, not runtime qualification of ReasonBraid.

## Integration ownership by path family

All owners below are in SIGNOFF-REPAIR. A guard integration is not evidence that
separately owned caller/target/consent policy already passes.

| Path family | Transaction integration owner | Other required policy/state owner |
| --- | --- | --- |
| create_boundary / insert_boundary_in_tx; create_grant / create_grant_in_tx; new/existing dev enroll | `.3.3.4.3` | `.3.5` issuer/enrollment policy |
| standalone authorize and apply_authorized_command | `.3.3.4.4` | `.3.4` authority/replay binding |
| run_thread_command; HTTP create/thread/auto core; MCP respond | `.3.3.4.4` | `.3.4`, `.5.2`, `.6.1`, `.8.1` respective caller/command policy |
| node_channel events → apply_node_result_in_tx | `.3.3.4.5` | `.4.1`–`.4.5` credential/fencing/receipt/budget mechanisms |
| authorize_tenant_admin and normal standalone inspect admissions | `.3.3.4.4`, `.3.3.4.6` | Caller families below retain their actual effect owner |
| inspect_tenant_admin → eight named administrative reads | `.3.3.4.6` | Existing frozen exception and receipt contract retained |
| API/service grant and boundary revoke; epoch bump | `.3.3.4.8` | `.3.1` completed foreign/no-op invariants retained |
| arm_breaker / reset_breaker | `.3.3.4.9` | `.3.5`, `.4.5` targeting/budget semantics |
| issue_node_enroll_token; revoke_node and its epoch bump | `.3.3.4.10` | `.3.5`, `.4.1` token/certificate use and target policy |
| replay_command / quarantine_command / prune_node_inbox | `.3.3.4.10` | `.3.5`, `.3.4`, `.4.3` ownership/cache/cursor repairs |
| inspect_node_inbox | `.3.3.4.6` admission | `.3.5` actual node ownership |
| import_profile_card → grant/identity/quota/receipt/profile | `.3.3.4.11` | `.5.1`, `.5.3` version/provenance/replay/remote-use policies |
| attest_capability_claim → attest_capability → write_profile | `.3.3.4.11` | `.5.1` version and claim policy |
| propose/accept/revoke federation agreement → federation service | `.3.3.4.12` | `.5.3` reciprocal-use/visibility/recruitment semantics |
| open_recruitment_call / close_call and offer/panel writes | `.5.2`, checked by `.3.3.4.13` | Repair actual thread/caller/initiator policy together with guarded effects |
| create_thread_auto preflight/spend/routing record | `.5.2`, checked by `.3.3.4.13` | Final core effect gains `.3.3.4.4`; early checks are not final evidence |
| directory_match / directory_presence / classify_reader | `.3.3.4.6` normal admission | `.5.1` candidate visibility and failure fallback |
| authorize_profile_read → profile versions; inspect_call; list_cross_domain_receipts | `.3.3.4.6` normal admission | `.5.1`, `.5.2`, `.5.3` self/initiator/visibility alternatives |
| put_profile self write | `.5.1` policy, `.3.3.4.11` transactional writer | Identity equality currently differs from grant authority |
| register_resolver → shared registry | `.7.1`, checked by `.3.3.4.13` | Shared operator policy must precede any claimed tenant guard protection |
| admin_metrics direct grant query | `.3.5`, checked by `.3.3.4.13` | Separate shared operational-read authority, not the frozen exception |
| policy, lifecycle, deployments, corrections direct grant predicates | `.9.1`–`.9.3`, checked by `.3.3.4.13` | Bind actual caller/action/tenant/parent before qualified effects |
| workflow/routing/global services with enrollment-only gates | `.8.1`/`.8.2`, checked by `.3.3.4.13` | Scope and approved state/evidence policy |
| MCP join_call / propose_policy_change and direct read tools | `.6.1` with `.5.2`/`.9.2` | Enrollment/quota gate alone is not authority |
| node token redemption, certificate rotation, lease/ack/heartbeat | `.4.1`/`.4.2` | Guard order at actual qualified credential/authority boundary |
| node journal/worker epoch cache and cached dispatch | `.3.4`, `.4.4` | Local invalidation input is not an authority-store mutation |
| schema upgrade, restore and test fixture authority writes | `.11.3` and owned test runners | Maintenance/owned-fixture scope; no claim that an app guard fences arbitrary DB-owner SQL |

## Independent mutation cross-check

A case-insensitive authority-table/epoch reference scan covered all tracked
non-Markdown files, not just the 101 Rust sources. Its literal-name mutation scan
(excluding tests for the production summary) found api.rs:821 tenant bootstrap
and authority.rs:170,242,737,812,845,885 boundary/grant/audit/epoch/revocation writes.
Node journal writes its local cache separately. The MCP source's table-name
inventory is in its cfg(test) fixture, not a hidden production authority writer.
Migrations 0004, 0013 and 0055 define the original authority/epoch/provenance schema;
they introduce no application status writer. Dynamic fixture/restore SQL is an
explicit maintenance exception, so this is not a universal SQL-language proof.

Independent reference hits also exposed the separately implemented policy,
lifecycle, deployment and correction grant predicates, plus metrics and auto
spend selection; their owners are in the table. This cross-check prevents treating
only the named authority helper as the complete application permission boundary.

## Verification and remaining implementation

The source/ownership census and selected design are complete; product guard and
final-effect implementation remains in the named children. The design is
`docs/decisions/2026-09-09_tenant-authority-transaction-order.md`.

- Source fingerprint: 101 tracked Rust source files, 1,749,975 bytes; SHA-256 `340c4af65db88e48496797c650bbce851bdfde47aaaefac5d93565cd38d26c2c` over sorted git-ls-files path bytes, NUL, raw file bytes, NUL. Baseline 1ba6184; documentation changes do not alter this corpus.
- Independently re-derived direct named-call locations: 42; exact set equality with the retained table passed, rc=0. Removing one location or adding an invented location is detected in memory, rc=0. The second derivation uses one callee alternation and excludes declarations/comments without the enclosing-function labeling pass. These controls verify this bounded lexical census, not dynamic dispatch or arbitrary SQL.
