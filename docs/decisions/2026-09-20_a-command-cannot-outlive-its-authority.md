# A command cannot outlive the authority that admitted it

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.24.1.1.2`
- Supersedes: nothing. Extends
  `2026-09-19_a-transport-receipt-is-not-an-acknowledgement.md`, which repaired
  the name of one shipped state and left four §10.6 states underived.

## The defect

A queued command for a node that never comes back was held **forever**.

The only `DELETE` against `node_inbox` in the server is the operator prune
(`authority/node_admin.rs`), and its predicate is `acknowledged_at IS NOT NULL`
— delivered rows only. An UNDELIVERED row is therefore unreachable by every
removal path the server has. `PHASE-3.2.2` deferred *the offline-delivery expiry
+ max age* to `PHASE-3.5`, `PHASE-3.5` closed without them, and
`git grep -ci "max_age|offline_expiry|delivery_expiry" -- crates` returns
**0 files**.

§10.6 names the two terminals this leaves missing:

```text
queued → offered → transport_received → acknowledged → consumed
                  ↘ expired / revoked / dead_lettered
```

## The decision

`expired` and `revoked` are the two ways an undelivered command's **authority**
can end: by the passage of **time**, and by an **act**. Both are derived, in
`migrations/0076_node_inbox_authority_terminals.sql`, from the grant the command
was admitted under.

The predicate is not new. It is the negation of the one question this repository
already asks about authority — `authority::grant_is_live`, the definition
`SIGNOFF-REPAIR.9.3.1` created by collapsing five different spellings of it:

```sql
status = 'active' AND valid_from <= now() AND expires_at > now()
```

Split into its reasons, `status <> 'active'` is the act and
`expires_at <= now()` is the time. No sixth spelling is introduced.

**The link is total, and that is proved by construction rather than sampled.**
`node_inbox.authz_ref` is the admitting `authorization_records.record_id`
(`migrations/0013`), and that record's `grant_id` is non-null for every ALLOWED
decision: `authority/selection.rs` returns `Decision::Allowed` only from the
branch carrying `grant: Some(grant)`, while the grant-less
`AuthoritySelection::absent()` is `Denied`. A work item is enqueued only on
`AuthorizationOutcome::Allowed`.

## The max age is derived, never chosen

`SIGNOFF-REPAIR.11.6` requires a threshold's population to be measured before the
rule over it is proposed, and `.11.24.1.1.2` restated the prohibition: the max
age must not be a number invented here.

It is not. **The ceiling on how long an undelivered command may wait is the
admitting grant's own `expires_at`** — a bound the issuing tenant set when it
issued the grant. That is the shape §9.2 asks for: *retention depends on the
operation's retry horizon and consequences*. This decision picks no number.

## What was refused, and why that matters more than what was taken

**The tenant revocation epoch.** `node_inbox.revocation_epoch` and
`tenants.revocation_epoch` are both shipped, so
`revoked ⇐ row.epoch <> tenant.epoch` was the cheap derivation available. It is
wrong: that comparison is `CachedDecision::is_invalidated`, whose verdict is
`CacheVerdict::Stale` — *re-ask the authority store* — not a denial, and any
revocation anywhere in the tenant moves it, including one that never touched
this command's grant. Deriving a terminal from it would publish `revoked` for a
command whose authority is intact — the same defect `migrations/0075` repaired
one migration earlier, a §10.6 word used for a fact that is not the one §10.6
names.

**Node suspension.** `node_presence` (0017) derives `suspended` from the node's
certificates, and 0017 deliberately makes it REVERSIBLE — the replacement ritual
issues a fresh active certificate and the node stops being suspended. A terminal
derived from a reversible predicate is a terminal a row can leave. A grant is
named by id, so revoking it is permanent for this row: re-issuing authority mints
a new `grant_id` the row does not point at.

## The precedence, and the argument that changed it

The first draft placed both terminals BELOW the delivery states, on the reading
that only an undelivered row can expire. **That draft was refuted by the cursor
ack**, and the refutation is why the order is what it is: `acknowledge` marks
every row up to the acked cursor, withheld rows included, so a terminal below
`transport_received` is one the row leaves without anything having been
delivered. The shipped order is therefore:

1. `dead_lettered` — §16.11's preservation rule already outranks everything here.
2. `consumed` — a `work_result` event is the strongest fact a row carries.
3. `revoked`, then `expired` — above `transport_received`. A command whose grant
   was revoked while the node held it is §10.6's `revoked`: the node's own
   dispatch boundary refuses to run it, so the work will not happen. `revoked`
   precedes `expired` because a grant can be both, and the act is the more
   informative answer to an operator asking why a command was never delivered.

One consequence is stated rather than discovered later: because `consumed`
outranks both terminals, a command already dispatched when its authority was
withdrawn reads `revoked` until its result lands and `consumed` afterwards — a
refinement by later evidence, the same shape `consumed` already has over
`transport_received`.

## Withheld, not deleted

`replay` — the one function the handshake and the poll both read the tail
through — gains the same predicate. That is part of the repair, not a nicety:
without it a row could reach `revoked` and then be delivered and acknowledged
back into `transport_received`, and the view would be a liar.

The rows are **withheld, never dropped**, and stay inspectable at
`GET /v1/nodes/inbox` and through the MCP `list_inbox` tool. A command that
expired while its node was away is not the same as one that never existed, and
an operator must be able to tell them apart — `PHASE-3.2.2`'s own
offline-KNOWN-versus-unknown distinction, applied to a command rather than to a
node.

⚠️ Wire-visible, and stated rather than slipped in: `delivery_state` gains two
values a client may not have seen. Pre-1.0, development profile, and the
alternative is a field reporting `queued` forever for work that will never move.

## Evidence

- `bash scripts/run_pg_tests.sh node_work` → `10 passed; 0 failed`. The suite was
  **8** at `592994f` and **10** here, measured with
  `git show HEAD:…/node_work.rs | grep -c '#\[tokio::test\]'` against the same
  count on the working tree — two controls added, not a delta reasoned from the
  shape of the change.
- Neighbours unchanged: `node_channel` 40/40, `node_inbox` 8/8, `quarantine`
  1/1, `node_replacement` 2/2, `mcp_listen` 6/6.
- **Falsified in two places, each in situ and restored byte-identical**
  (`sha256` matched before and after):
  - the derivation — `THEN 'revoked'` → `THEN 'queued'` in the new migration
    alone fails the control with `left: "queued"`, `9 passed; 1 failed`;
  - the filter — removing the authority predicate from `replay` alone fails
    BOTH controls with `left: Some(1)`, `right: Some(0)`, `8 passed; 2 failed`.

## What is still owed

`offered` and §10.6's `acknowledged` remain underived, with their dispositions at
`SIGNOFF-REPAIR.11.24.1.1` and their work at `.11.24.1.1.1`. Two findings this
leaf measured and did not repair carry leaves of their own:
`.11.24.1.1.2.1` (a cursor ack records a transport receipt for a row the
transport never carried) and `.11.24.1.1.2.2` (a dispatched work item's budget
reservation expires ten minutes after dispatch, and nothing bounds delivery to
that window).
