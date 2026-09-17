# §16.11's two unwired scopes bind the acquisition path, and fail-closed becomes a default row

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.14`
- **Delegated by the director on 2026-09-17**: *"Regarding the call, it is yours
  to make."*
- Related: ADR-034 / `migrations/0047` (the machinery and its fail-closed
  contract), `2026-09-17_what-a-citation-list-costs.md` (`.11.14.3.7`, which
  established that a quota is *not* the instrument for per-request work).

## The measurement, first

```bash
grep -rn "quota::check_in_tx" crates/ --include=*.rs        # -> 2
grep -c  "pub const SCOPE_"   …/quota.rs                    # -> 5, see below
grep -c  "^pub const OP_"     …/threads.rs                  # -> 12
```

- **Two** producers: `thread.invite` on the tenant scope, the MCP write gate on
  the principal scope.
- **Four** scope kinds — the array's own type is `[&str; 4]`. ⚠️ The raw grep
  returns 5 because it matches the `SCOPE_KINDS` array itself; publishing the
  grep count would have been the wrong number.
- `resolver` and `destination` exist in the vocabulary, in `0047`'s CHECK
  constraint, and in **nothing else**.
- `POST /v1/resources/{id}/resolve` carries **no quota, no storm control and no
  breaker**, and performs a real network acquisition per call.

## Part 1 — the gap §16.11 names is the ACQUISITION path, not the thread verbs

§16.11 asks quotas to bound "notification floods, invitation storms, expensive
loops, and **scraping**", and names "**resolver abuse**" among the circuit
breakers. Invitation storms are bound. Scraping and resolver abuse are the
acquisition path, and it is unbounded.

⛔ **The thread verbs do not gain quotas here**, and the reason is measured
rather than preferred: `.11.14.3.7` established that the citation amplification
is inside ONE request, so a per-hour call ceiling bounds arrivals and not
per-request work. Adding quotas to the other eleven thread operations would be a
rule proposed without a population, which `SIGNOFF-REPAIR.11.6` forbids.

## Part 2 — why the two scopes sat unwired, which is the load-bearing finding

`check_in_tx` is **fail-closed**: a scope with no configured row is the typed
`quota_unconfigured` refusal. `0047` states the reason — *the bound must exist
before the surface is usable*.

⭐ That contract is correct, and it works for the two wired scopes because of a
property they have and these two do not: **their members are created by a path
the server controls, so the bound can be seeded at creation.** A tenant gets its
row from the enroll transaction; a principal gets its row from enrolment and card
import.

Neither unwired scope has such a path:

| Scope | Why it cannot be seeded per member |
| --- | --- |
| `resolver` | the space **grows at runtime** — `POST /v1/resolvers` registers new packs — so a per-member seeding rule closes every newly registered pack for every tenant until someone notices |
| `destination` | the space is **the open internet**, and cannot be enumerated in advance at all. A fail-closed per-host bound would refuse every acquisition to every host nobody pre-configured |

⛔ **A fail-closed bound over a space you cannot enumerate is not a bound; it is
an outage.** That is why these two were deferred rather than wired, and it is the
thing the deferral never said out loud.

## Part 3 — the decision: the fail-closed CONTRACT is kept, the MECHANISM changes

**Each tenant gets a DEFAULT row at the wildcard scope id `*`; a specific row
overrides it; the absence of BOTH is still the typed refusal.**

- `insert_defaults_in_tx` seeds `(tenant, 'resolver', '*')` and
  `(tenant, 'destination', '*')` at tenant creation, beside the invite row it
  already writes.
- `migrations/0068` backfills the same two rows for existing tenants, exactly as
  `0047` backfilled the invite bound.
- `check_open_scope_in_tx` resolves most-specific-wins with one existence probe
  and then reaches `check_in_tx` unchanged — so the window arithmetic, the
  recorded `use`/`denial` and the transaction semantics are the ones already
  qualified.

⭐ The property fail-closed exists for — *there is always a bound* — holds
exactly as before. What changed is that the default is a **row** rather than an
**absence**.

⚠️ **A specific row starts its own count**, because `quota_events` is keyed by
`quota_id`. That is the correct semantic — a narrowed bound is a new bound, not a
continuation — and the control asserts it.

## Part 4 — where the check sits, and what it counts

After the ranking and **before** the pack executes, for two scopes: the ranked
resolver id, and the locator's host.

- ⚠️ **It counts ATTEMPTS, not successes.** An attempt is what a caller can
  repeat, and it is what reaches the network. The control shows the bound being
  consumed by an attempt that the destination policy then refuses.
- ⚠️ **A locator with no host takes the resolver bound only.** There is no
  destination to bound, and no pack can fetch such a locator.
- ⚠️ **An attempt against a gated-off pack is counted too**, because the check
  precedes the match rather than sitting in each arm. Deliberate: the caller's
  repeatable act is the resolution request.
- The denial row **commits with the refusal**, which is `0047`'s contract and
  `.3.5.1`'s shape: a refusal is a recorded fact, never silent.

## The alternatives, and why each was rejected

1. **Retire the two scope kinds from the vocabulary** — ❌ §16.11 names both
   explicitly and `ROADMAP.md` is frozen; removing a named control narrows a
   stated capability on the strength of it being inconvenient to wire.
2. **Seed `resolver` per member from `resolver_capabilities`** — ❌ measured to
   break: `POST /v1/resolvers` adds members at runtime, so the next registered
   pack is closed for every tenant, and the failure appears as
   `quota_unconfigured` on a pack an operator just installed.
3. **Make the two scopes fail-OPEN** — ❌ it discards the property `0047` was
   built for and leaves the section's named surface unbounded while appearing
   bounded, which is worse than today's honest absence.
4. **Bound it with a new mechanism (a rate limiter beside the quota)** — ❌ the
   leaf's own acceptance forbids a second mechanism where the existing one
   reaches, and it does reach: only the *seeding rule* needed changing.
5. **Pick a measured ceiling** — ❌ impossible today and `.11.6` forbids
   pretending otherwise. No acquisition volume has ever been measured. The
   ceilings are dev-profile defaults in the shape `0047` already established, and
   they are labelled as such at the constant. **What this leaf decided is the
   shape of the bound, not its number.**

## What this does NOT do

- ⚠️ It does not close `SIGNOFF-REPAIR.11.14.3.7`. A call ceiling bounds
  arrivals; that leaf's amplification is inside one request and is bounded by the
  request body.
- ⚠️ It does not bound the R2/R3 worker's own resource use, which is `.7.3.4`'s.
- ⚠️ The ceilings are not production figures, and no gate should read them as
  evidence that abuse is bounded at any particular level.

## Verification

- `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **54 passed / 0 failed**.
- The control asserts: the enrol transaction seeds **one default row per open
  scope, not one per member**; one resolution records **one use per scope**; a
  host-specific ceiling **overrides the default** and refuses `429` with a
  **recorded denial**; and removing **both** rows is still the typed fail-closed
  `503`.
