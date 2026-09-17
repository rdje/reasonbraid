# What a citation list costs, and why a quota is the wrong instrument for it

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.7`
- Related: `2026-09-16_a-citation-registers-the-reference-it-names.md` (which
  created the cost), ADR-034 / §16.11 (the quota machinery).

## The measurement

`.11.14.3.2` made each citation register a §12.1 reference **inside the thread's
aggregate transaction**, which holds `FOR UPDATE` on the thread's row. So one
request converts into O(n) statements that block every other command on that
thread for their duration. Reproduced: **64 distinct citations register 64
references**, asserted by a live control.

⚠️ **A correction to the leaf's own census.** It recorded *"the one
`quota::check_in_tx` call is `OP_INVITE`'s"*. Re-derived:

```bash
grep -rn "quota::check_in_tx" crates/ --include=*.rs   # -> 2
```

`threads.rs:1201` (`OP_INVITE`, tenant scope) and `mcp_write.rs:104` (the MCP
write gate, principal scope). The machinery reaches **two** verbs across **two**
of its four scope kinds, not one verb.

## The decision, in three parts

### 1. A windowed quota is the wrong instrument for THIS defect

⭐ Measured rather than argued: the cost is **inside one request**. A per-hour
ceiling on `thread.contribute` bounds how many requests arrive and says nothing
about how long any one of them holds the thread's row. A single request with a
very large citation list is exactly as damaging under a quota as without one.

So the dimension that bounds this defect is **request size**, not call rate.

### 2. What bounds a request today, measured

`grep -rn 'DefaultBodyLimit' crates/reasonbraid-server/src` returns **0**, so the
extractor's framework default applies. Read from the vendored source rather than
recalled:

```
.project-data/cargo/registry/src/…/axum-core-0.5.6/src/ext_traits/request.rs:319
    const DEFAULT_LIMIT: usize = 2_097_152; // 2 mb
```

⇒ **2 MiB of request body is the operative bound**, and it is inherited from a
dependency rather than declared by this product.

⛔ **Declaring it explicitly was considered and NOT taken here**, with a reason:
one global limit would apply to `POST /v1/snapshots` too, which legitimately
carries base64 evidence bytes and needs a larger bound than a contribution. So
"make the limit explicit" is a **per-route** decision, not a one-line
substitution, and taking it inside this leaf would be exactly the unmeasured
policy the leaf was opened to avoid.

### 3. The de-duplication, which is derived rather than chosen

A reference's identity is the `(original_locator, expected_digest)` **pair**
(`.11.14.3.2`), so two citations naming the same pair name ONE row and the second
registration can only fetch back what the first just wrote. De-duplicating the
**work** therefore invents no number and changes no semantics.

⛔ **It de-duplicates the work, never the record.** The decision is a pure
function returning one answer per citation, so the caller emits one
`RegisteredEvidenceRef` per citation, in order, with its own note. A contribution
citing one pair 48 times still shows 48 citations.

⚠️ **It is not a bound.** The cost of N *distinct* citations is unchanged. It
removes the trivially amplifying case, and the leaf says so rather than implying
the defect is closed.

## How it is verified, and what that cost

⭐ **The de-duplication is not observable through any product surface.** The row
count is **1 either way**, because the pair replay already returns the existing
row.

⛔ A `pg_stat_user_tables` scan-counter instrument was written for it and
**discarded**: run against the unrepaired handler it reported **the same value**,
so its assertion could not fail. A control that passes identically either way
measures nothing, and shipping it would have converted an unverified change into
one that looks verified.

The de-duplication is falsified **directly** instead, as a unit test over the
pure function the handler calls — observed RED against a locator-keyed
implementation (`[0, 0, 0, 0]` where the pair-keyed answer is `[0, 1, 0, 3]`).
The rule is promoted as
`docs/knowledge/a-change-no-surface-can-see-needs-a-seam.md`.

## What stays open, owned rather than noted

`SIGNOFF-REPAIR.11.14.3.14` owns §16.11's real question — **which verbs take a
quota and on which scope** — with this leaf's census as its starting point: two
call sites, two of four scope kinds, and twelve thread operations of which one is
bounded. ⛔ It is opened with the explicit statement that it does **not** close
this defect, so a later reader cannot mistake quota coverage for a bound on
per-request work.

## Verification

- `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **52 passed / 0 failed**,
  including the O(n) measurement, the record's one-entry-per-citation shape, and
  the pair semantics (one locator at two digests stays two references).
- `cargo test -p reasonbraid-server --lib threads::` — the pure function's unit
  test, falsified by injection.
