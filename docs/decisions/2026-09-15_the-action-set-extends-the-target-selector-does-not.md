answers: may the grant action vocabulary gain administrative members; is GrantAction frozen by the roadmap; why can a grant not name the publication it is authority over; should TargetSelector gain a publications variant; what breaks when a new GrantAction variant is added; why is extending the action set a migration rather than an addition; what does an enrolment-minted boundary permit

# The administrative action set extends; the target selector does not (`SIGNOFF-REPAIR.9.3.4`)

- Date: 2026-09-15 · Leaf: `SIGNOFF-REPAIR.9.3.4` · Decision record

## Context

`SIGNOFF-REPAIR.9.3.1` bound three administrative surfaces to a grant the caller
**holds**, and `SIGNOFF-REPAIR.9.2.1.2` bound two publication verbs the same way.
All five can now say *who* the authority belongs to and none can say *what it is
authority over*: there is no action meaning "may publish" and no selector that
can name a publication, so every such grant is effectively tenant-wide for those
verbs. `.9.3.4` owns the question of whether the vocabularies grow.

## The measurements this rests on

Every number here is a command, and two of them refuted the premise the leaf was
written from.

| question | command | answer |
| --- | --- | --- |
| `GrantAction` variants | parse `pub enum GrantAction` in `crates/reasonbraid-core/src/authority.rs` | **10** — nine thread-shaped plus `TenantAdmin` |
| `TargetSelector` variants | same file | **2** — `TenantWide`, `Threads { threads: Vec<ThreadId> }` |
| is it roadmap-frozen? | `git grep -c "GrantAction\|TargetSelector" ROADMAP.md` | **0** |
| sites with an inexpressible scope | `git grep -n "grant_held_by(\|grant_is_live(" -- ':(glob)crates/reasonbraid-server/src/**'` | **6** (4 held-by covering 5 verbs, 2 is-live) |
| what a fresh boundary permits | `ADMIN_ACTIONS` in `crates/reasonbraid-server/src/api.rs` | **9 of the 10** — `ThreadCreateAuto` is excluded |
| how boundaries are stored | `migrations/0004_authority.sql:15` | `JSONB` array of **GrantAction wire names** |

## Decision

**`GrantAction` extends with administrative verbs. `TargetSelector` does not.
And the extension is a migration, not an addition.**

### Why the action set extends

⛔ **The premise the leaf was written from is false, and it is the premise that
would have declined this.** `.9.3.4` says the vocabularies are "closed wire
vocabularies and §9.8 publishes a stable registry", and reads across to
`.11.7.1`'s frozen-roadmap hold. §9.8 is the **reason-code** registry — a
different vocabulary. `git grep -c` over `ROADMAP.md` returns **0** for both
types. The freeze does not reach them, and `.11.7.1` is not a blocker here.

⭐ **This project has already ruled on how to do it**, in
`docs/decisions/2026-09-06_authority-boundary.md` under *How to apply*:

> "Add an action/ceiling by **extending the checker FIRST** — a grant that
> exceeds the boundary must be **unrepresentable**, not merely unapproved."

So the answer is not merely permitted, it is prescribed, and it comes with an
ordering constraint that the implementation must honour.

The gap is also real rather than theoretical: `TenantAdmin` is **one** action
covering every administrative verb, so a grant issued to record a policy
correction equally permits registering a deployment target and publishing. Verb-
level actions are strictly narrower than that, and they are expressible today.

### Why the target selector does not

`TargetSelector::Threads { threads: Vec<ThreadId> }` **enumerates its objects at
grant time**. That works for threads, which exist before anyone is authorized
over them. It cannot work for the publication case that matters: the ordinary
flow publishes a **new** publication, whose id does not exist when the grant is
issued. A `Publications { publications }` selector would be unusable for exactly
the operation it was added for, and `ResourceTarget` — `Tenant` or `Thread` —
would need the same surgery for the same non-benefit.

⇒ Object-level scoping for publications needs a selector shape this model does
not have (a predicate, not an enumeration). That is a larger design question than
the gap that opened this leaf, and inventing it here would be the
`.11.6` failure: a rule proposed before its population is understood.

⚠️ **The residual is therefore accepted and published, not closed**: after the
action set extends, a publication grant is narrowed *by verb* and remains
tenant-wide *by object*. The book must keep saying so.

### Why it is a migration, not an addition

🔴 **Adding a variant retroactively narrows every existing grant**, and this is
the part a casual reading misses. `permitted_actions` is a JSONB array of wire
names, so **no stored boundary contains a name that did not exist when it was
written**. The moment a publication verb requires `GrantAction::PublicationPublish`,
every already-enrolled tenant's boundary fails to permit it and the verb stops
working for them.

That direction is fail-**closed**, which is the safe one — but it is still live
breakage, and it means the change is three things at once: the vocabulary, the
default `ADMIN_ACTIONS` set, and a disposition for boundaries already stored.

## Consequences

- `.9.3.4` is **decomposed**, because the above is three repairs sharing a leaf
  and because its own site census was stale: it says "the three sites" and there
  are **6**, the fourth of which `SIGNOFF-REPAIR.9.2.1.2` added in this session.
- The implementation honours the ordering rule above: the checker and the
  boundary first, so an over-broad grant is unrepresentable before any verb
  depends on the new action.
- ⛔ **Nothing is claimed narrowed until a coverage check exists.** Holding a
  grant is what `.9.3.1` and `.9.2.1.2` established; covering an action is a
  separate predicate, and until it is written the verbs remain verb-wide.
- `.11.7.1` is **not** a prerequisite. The two were coupled by the false premise
  corrected above; the reason-code registry question stands on its own.

## What was refuted

- **"The roadmap freezes this"** — `ROADMAP.md` names neither type. The leaf's own
  cross-reference to `.11.7.1` rested on it.
- **"Extend both vocabularies"** — refuted by `Threads { threads }`: an
  enumerating selector cannot name an object created after the grant.
- **"Adding a variant is backward-compatible"** — refuted by
  `migrations/0004_authority.sql:15`: boundaries store wire names, so a new
  action is absent from every existing row.
