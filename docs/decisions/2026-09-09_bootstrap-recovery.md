---
answers:
  - How can a client recover a bootstrap whose success response was lost?
  - How does a bootstrap request identity remain separate from tenant authority?
  - How will bootstrap replay preserve tenant guard order and the total deadline?
  - What must the CLI persist before a new-tenant bootstrap request?
---
# Persist the bootstrap request identity before sending it

- Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3`; runtime/contract child `.1`, server protocol `.2`, CLI persistence `.3`.
- Status: selected contract. Actual unconfirmed-commit/readback behavior is qualified by 25 selected controls and focused strict lint; all results/shutdown consumed, cluster absent. Server protocol in child .2 passes 73 selected controls, its final eleven-control fixture rerun and strict lint; all results/shutdown are consumed and four owned clusters are absent. CLI persistence/recovery remains unimplemented in child .3.
- Evidence: `docs/tasks/artifacts/signoff_review/bootstrap-recovery.md` at product baseline `01bd473`.

## Request and outcome

Add an optional `bootstrap_request_id` to development enrollment. It identifies
one logical new-human bootstrap across transport attempts. Use the existing
RequestId family with a bounded canonical req_-prefixed UUIDv7 representation;
reject malformed, noncanonical and wrong-kind values before database effects.
It is valid only for kind=human with tenant_id absent. Requests with no key retain
the established behavior: each is a distinct new-tenant request. Existing-tenant
(tenant, kind, name) replay is unchanged. Server-generated TenantId remains a
different identity; a request key cannot nominate an existing tenant for minting.

Bind a key to the effective creation request. Currently kind/new-tenant mode is
fixed, and name is the effectful caller value. Human actions are deliberately
ignored by this development endpoint, so changing an ignored action list does
not turn the same logical request into another effect. Any future effectful field
must join the binding explicitly. Compare bound text exactly; do not normalize
names into a global identity or infer replay from equal names alone.

Persist the request binding and complete successful response in the same guarded
transaction as the tenant, boundary, grant, identity, quotas and enrollment. Use
an immutable global request-ID key and a unique tenant mapping with appropriate
identity constraints. A new receipt contains real generated IDs and the echoed
request ID; it is never synthesized after an uncertain commit. The stored result
must have a strict versioned decoder that checks its key, tenant, human identity,
name, response shape and source references. Malformed/conflicting storage refuses
safely instead of starting another bootstrap or inventing a response.

A matching keyed replay returns the original tenant/principal/boundary/grant IDs
and request ID, with replayed=true. It creates no replacement authority and does
not re-evaluate the original grant as if replay were fresh issuance. Original
outcomes remain recoverable after revocation. A conflicting bound request returns
409 without changing existing state. Missing known authority or storage faults
retain their real classes. Commit failures remain unconfirmed; resubmitting the
same keyed request to the same authoritative store uses the durable outcome
mechanism rather than assuming rollback. Never expire/reassign a key while its
original tenant/outcome remains recoverable; retention must preserve that binding.

## Keep coordination in the existing transaction owner

The caller knows a request ID before it knows a tenant ID. Keep server tenant
allocation; use the existing exclusive full-tenant guard and typed rollback owner
rather than converting the request UUID into a tenant UUID.

A provisional attempt declares a new server-generated candidate tenant. It may
read only the minimal request-ID-to-tenant routing metadata. If the immutable key
already maps elsewhere, abort the provisional attempt and its new anchor, then
reacquire the recorded tenant guard before decoding the stored result or checking
its request binding. That limited routing read is not foreign policy evaluation
or authority to mutate the recorded tenant.

For a new key, create all provisional enrollment rows and insert the outcome
binding before commit. An INSERT ON CONFLICT DO NOTHING collision with a concurrent
winner obtains only its recorded routing tenant; abort every losing provisional
row before replay under the winner's guard. Never change guards within a live
callback, upgrade modes or issue callback transaction-control SQL. All reads of
the full outcome are tenant-scoped on the guarded connection.

The key-to-tenant mapping is immutable, so allow at most one such redirect. Share
one fifteen-second operation budget across both attempts, shortening the existing
bounded lock/statement/whole limits to the remaining whole-millisecond budget.
Preserve commit-error phase; no outer timeout may collapse a potentially sent
COMMIT into a proved pre-commit rollback. Unexpected remapping or malformed stored
routing is a storage invariant failure, not a retry loop. Qualify winner commit,
winner rollback, lock/total limits and both concurrent arrival orders.

## Durable CLI recovery

For a new-tenant human enrollment, persist one bounded pending request before
sending HTTP. It includes the canonical request ID, exact request and configured
server identity. Reuse it after transport uncertainty, a malformed reply or local
state-write interruption. A different request/server must refuse while recovery
is pending; do not overwrite the unresolved request with a fresh key.

Validate the keyed reply against the pending request before updating local
principal state. Publish state atomically and durably, then clear the pending
request durably. A restart between those steps must recover the same original
server outcome and complete cleanup. A normally completed new invocation gets a
fresh key, preserving intentional distinct tenant creation.

The current in-place state.json writer is insufficient for this contract. The CLI
child owns atomic replacement and synchronization, interruption/restart controls,
strict recovery-state decoding, concurrency exclusion and conflict handling for
the existing enrollment/thread state-writer paths. It must preserve unrelated
principal/thread state and refuse ambiguous or off-volume paths. Pending files,
locks and temporary replacements are repository-derived on the repository volume;
no home/tmp fallback. A live process lock must release on crash; stale lock files
must not be treated as live ownership or silently deleted to bypass exclusion.
Choose and qualify the concrete lock/publication mechanism in the CLI child.

## Qualification boundary

This decision does not add bootstrap caller authentication or a site-operator
credential; `.3.5` retains that policy work. The endpoint stays a development trust
surface and cannot mint site-operator grants. The existing no-key endpoint cannot
promise response-loss recovery; operator reconciliation may still be needed for
callers that omit the key. The CLI will use keyed recovery by default once its
implementation is qualified. No completed-client-recovery or zero-defect claim
follows from the truthful unconfirmed error or this design alone.

## Server implementation details

Migration 0057 creates tenant_bootstrap_requests with a canonical UUIDv7 request
key, unique tenant mapping, identity foreign key, exact C-collated name, positive
smallint outcome version and JSON object outcome. No legacy key inference occurs.
Immutability is the service/retention contract: no update/delete/expiry endpoint is
added; database-owner mutation is corruption, not a supported recovery operation.
The database enforces key/tenant uniqueness, identity existence and basic shape;
the version-one decoder additionally requires every response field, canonical
source IDs and the actual tenant/human/enrollment/grant/parent bindings. Current
grant status and time are not issuance checks on this historical replay.

The bootstrap conflict wire code is idempotency_conflict (409). Ordinary command
envelope idempotency_mismatch remains unchanged. Optional null keys retain the
existing no-key behavior; malformed strings/modes produce semantic 400, while
non-string JSON values fail extraction with 422. The private coordinator in
api/bootstrap.rs uses the existing typed transaction owner, one abort/reacquire
redirect and the remaining whole-millisecond budget. No outer timeout can erase
an already-sent COMMIT phase. Exact evidence is recorded in
docs/tasks/artifacts/signoff_review/bootstrap-server.md.
