---
answers:
  - How does command authorization choose among multiple grants?
  - Which parent and grant does a delegated denial record reference?
  - Does candidate selection establish revocation serialization?
---
# Select usable command authority from each grant's actual parent

- Owner: `SIGNOFF-REPAIR.3.3.3.1`.
- Status: implemented and verified; all 37 live authority/command API tests, six evaluator controls and strict focused lint pass. All results and owned-cluster shutdown are consumed.
- Predecessor: `docs/decisions/2026-09-09_bound-authority-evaluation.md`.

The former loader chose one active grant by newest valid_from and independently
loaded the tenant's active boundary. The corrected evaluator rejects mismatched
parents, but the loader could still hide usable older grants and record an
unrelated parent on a denial. Delegated absence could retain the caller's grant
as the alleged authority source.

Normal command selection now reads pages of 32 active grant rows ordered by
valid_from descending and grant_id ascending with explicit bytewise C collation.
PostgreSQL documents this byte-value ordering in its [standard collation contract](https://www.postgresql.org/docs/16/collation.html#COLLATION-MANAGING-STANDARD). The same ordering drives keyset continuation. Each grant loads its named parent
and runs the existing identity, tenant, subject, ceiling, liveness and target
checks. The first usable candidate wins; the first deterministic refusal is held
only as the fallback when every candidate refuses. There is no arbitrary total
scan cutoff. The page bounds row count, not the size of a malformed database value.

Caller permission strips delegation. Delegated-source selection includes the
whole requested scope, so a newer grant that covers the immediate target but not
the requested attenuation cannot hide a usable broader source. Missing source
candidates carry no grant or boundary references and use no-policy. A caller's
denial still prevents allowance, but its grant is never substituted for an absent
delegated source. The existing audit schema names the selected authority source;
it is not a complete record of every rejected candidate or a cryptographic chain.

Malformed selected rows are storage errors, with no guessed decision or panic.
Boundary parsing rejects negative delegation depth instead of wrapping it. The
boundary/grant decision digest retains its existing input format and limitations.

Actual-parent reads are separate statements on the command executor. This child
does not claim one authorization/revocation serialization point across those
reads; the transaction guard and final effect audit remain `.3.3.4`. Delegation
consent/depth remain `.3.4`. The separate frozen-own-tenant administrative read
helper remains `.3.3.3.2` and keeps its approved boundary-status exception.

The owning leaf records baseline, corrected, positive/negative and compatibility
results. Qualification is not inferred from the implementation description.
