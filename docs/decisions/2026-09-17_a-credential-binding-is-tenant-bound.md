# A credential binding is tenant-bound, and a content row may not hold one

- **Date:** 2026-09-17
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.14.3.10`
- **Commit:** `REASONBRAID-REPAIR-0234`

## The question

`ResourceReference.credential_binding_ref` is a caller-supplied §12.1 field that
**selects a credential**: `api.rs`'s R5 arm reads it back out of the stored row
and hands it to `broker.resolve`, whose result attaches to the acquisition. It
was stored on `resource_references`, a row keyed
`UNIQUE (original_locator, expected_digest)`.

ROADMAP §16.3 invariant 5: *"target credentials are selected only after
authorization for the concrete target and action."*

## What was measured, before anything was decided

Reproduced end to end with the gate open (`start_gated`), one binding registered
with the broker, and two enrolled tenants over one locator:

| | before | after |
|---|---|---|
| owner resolves its own reference | `r5-credential-broker` → `destination_refused` | unchanged |
| **stranger replays the pair, names no binding** | **`r5-credential-broker` → `destination_refused`** | `r0-https-fetcher` → `destination_refused` |
| stranger names its own (unregistered) binding | impossible — the replay discarded it | `credential_unavailable`, quoting `cred_stranger_only` |
| owner resolves again afterwards | — | unchanged |

Row 2 is the defect: `destination_refused` is the **loopback pre-flight**, which
is reached only once a credential has resolved. The stranger drove an
authenticated acquisition with the owner's credential.

⭐ The RESOLVER is the discriminator in row 2, not the error kind — the
authenticated and unauthenticated paths end at the same refusal, and only
`resolvers` says which ran.

## The decision

**The selector moves off the shared row onto `reference_registrations`** — the
tenant-bound row `SIGNOFF-REPAIR.11.14.1`'s family had already created — and
`migrations/0069` **drops the old column**.

The shared row keeps the CONTENT; the tenant-bound row keeps the DECISION. That
is the same correction this family applied to citations, to assessments, to
snapshot writes and to reference reads; a credential selector is the fourth
instance and the sharpest, because the shared datum was a means of access.

Three consequences, all intended:

1. **No inheritance.** A replaying tenant gets its own registration, which names
   whatever that tenant named.
2. **Two tenants may now differ.** The pair key made one binding per pair; it
   does not govern the registration.
3. **The unbound read cannot produce a selector at all.** `resources::get` has
   no column to read one from, so the guarantee is structural rather than
   conventional — `get_for_tenant` is the only path that supplies one, and only
   the asking tenant's own.

## Rejected alternatives

- ⛔ **A tenant column on the broker.** The broker's deployment integration is
  the OS keychain (`broker.rs`), a machine-local operator store with no tenant
  dimension. Keying it by tenant pushes a server-side authorization fact into a
  component that cannot hold one, and the dev-profile `HashMap` would be the
  only place the change was ever real.
- ⛔ **Forbid the field on a reference and take the binding on the resolve
  request instead.** It puts the selection at the request boundary, which is
  where `broker.rs` says resolution belongs — but §12.1 lists the field on the
  reference, and a request-time selector is *more* caller-supplied, not less. It
  moves the defect rather than closing it.
- ⛔ **Keep it on the shared row and honour it only for the tenant that wrote
  it.** Closes the inheritance and leaves the pair-key limit standing: two
  tenants still could not hold two bindings. It also leaves a live credential
  selector on a shared row, which is the structure the defect is made of.
- ⛔ **Keep the dropped column, stop reading it.** A column that nothing writes
  and nothing reads is not inert when it is a *credential selector on a shared
  row*: it is the next author's mistake, already in the schema. This repository
  has repaired a dead column once (`.11.14.3.5`) and that one only held a risk
  class.
- ⛔ **Authorize the selection here, via `authority::authorize`.** The right
  long answer and out of scope for this leaf: `GrantAction` is thread-scoped
  (`thread_*` plus `TenantAdmin`) and `TargetSelector` is thread/tenant-shaped,
  so this needs a new action vocabulary and a new selector — a §16.4
  authorization surface, not a wiring change. `SIGNOFF-REPAIR.11.6` also forbids
  proposing a rule before its population is measured, and the population of
  operator-granted bindings is **zero** today. Routed to `.11.14.3.10.1`.

## The backfill, and why a credential may be moved at all

`reference_registrations.registered_by` and `resource_references.submitted_by`
are literally the same value — `api::submit_resource` writes
`actor_handle_for_subject(principal)` into one binding and clones it into the
other — so the reference's submitter joins **exactly** to its own registration.
The binding lands there and nowhere else; every other tenant's registration gets
`NULL`. §16.4: *"publication, secret access, grant changes, and irreversible
writes fail closed."*

## What is NOT closed, and is owned

The broker's binding namespace is global. A tenant may still *name* a binding an
operator created for another tenant — it must now name it in its own
registration rather than inherit it, but naming remains sufficient. ⚠️ Measured
**latent, not live**: the R5 pack is off by default, `grep -rn "\.register("`
over the broker returns three hits and all three are tests, and no binary
reaches `Broker` outside `ApiState::new`'s empty store. `SIGNOFF-REPAIR.11.14.3.10.1`
owns it.

## Falsification

The repair (both source and `migrations/0069`) was stashed with the control
left in place; the suite went RED at exactly the defect arm, reading
`Array [String("r5-credential-broker")]` where the repair gives
`Array [String("r0-https-fetcher")]`. The stash was verified to have landed
(`git status` showing only the control dirty and `0069` absent) before the run —
`docs/knowledge/an-injection-must-be-shown-to-land.md`.

## Related

- `docs/decisions/2026-09-17_the-reference-read-is-bound-to-its-registrants.md`
- `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
- `docs/decisions/2026-09-07_r5r3rx-contracts-opt-in.md`
- `docs/knowledge/a-shared-row-may-not-hold-a-tenant-decision.md`
