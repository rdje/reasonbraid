# An advertised policy line carries an adjudicated verdict, at the value it advertises

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** `SIGNOFF-REPAIR.7.3.6.1` (REASONBRAID-REPAIR-0266) — measured

## The fact / decision

A resolver pack publishes six policy fields to every caller that reads the §12.2
capability registry. There are six packs, so **36 advertised lines**, and every
one of them now carries a verdict in `.doctrine/advertised_policy_verdicts.tsv`
with the evidence that earned it. `scripts/census_advertised_policies.py`
enumerates the lines from their two producers and refuses, as a registered
doctrine gate, any line the ledger does not cover **at the value the producer
currently advertises**.

Measured across all 36: **`enforced` 3 · `unverified` 5 · `vacuous` 14 ·
`misdescribed` 7 · `undefined` 7**. Three of thirty-six are enforced by a
mechanism with a control that has been observed refusing.

## Why

`SIGNOFF-REPAIR.7.3.5` (REPAIR-0210) found the R3 browser pack advertising
`redirect_policy: "deny"` and `subresource_policy: "deny"` and enforcing
neither, and repaired exactly those two. The finding generalises by
construction — a caller choosing a pack reads every field, not the two somebody
happened to audit — but the remaining lines were not a population anyone could
enumerate, so no clause of the parent leaf's acceptance was evaluable.

Three things the census established that reading could not:

1. **Four of the six policy fields are consumed by nothing.**
   `redirect_policy`, `archive_policy`, `subresource_policy` and
   `javascript_policy` occur 34 times in tracked Rust — on 32 lines, since the
   registry INSERT's column list names three of them on one line — and every
   occurrence is a declaration, a write or a comment. The claim carries its enumeration in both
   directions: the classifier's default for an occurrence no rule explains is
   `read`, so the finding can only be under-stated. Only `egress_class` and
   `sandbox_level` are consulted — by the ADR-018 vocabulary check and by the
   resolution filter.

2. **`vacuous` is not `enforced`, and 14 lines are vacuous.** R0 advertises
   `javascript_policy: "deny"` and runs no script engine: true today, guarded by
   nothing, and false the day R0 gains one. The two lines `.7.3.5` repaired had
   been vacuous in exactly that way until the pack they described started
   executing pages. Collapsing the two verdicts would hide the next instance.

3. **`archive_policy` is defined nowhere in the repository** — 7 of the 7
   `undefined` lines. Its sharpest instance is a contradiction visible without
   leaving the row: R2 advertises `archive_policy: "deny"` two fields above a
   `media_types` list containing `application/zip` and `application/x-tar`,
   both of which its worker expands.

**The value is part of the ledger row because the instrument shipped without it
and carried the exact defect it was built to find.** Keyed by pack and field
alone, a verdict outlived the line that earned it: R3's `subresource_policy`
flipped `deny` → `allow` in the producer left the census GREEN, still reporting
`enforced — SIGNOFF-REPAIR.7.3.5` for a line advertising the opposite of what
that leaf repaired. Measured in situ, restored byte-identical, then replayed
against the repair. That is `docs/CLAIM_VERIFICATION.md` §5B's derived-constant
rule: a judgement about a word is gated on the word, or it is a comment saying
*do not edit by hand*.

**The verdict ledger is a separate file from the census on purpose.** A verdict
is a judgement with evidence behind it; an enumeration is a command. Keeping
them apart is what lets the census be re-run against a tree whose verdicts have
changed, and it is why `--check` can be a gate at all. The census never grades;
the ledger never counts.

**Calibrated over the full history rather than a window**, because the instance
is older than any 300-commit one (`[[calibrate-over-the-history-that-contains-the-instance]]`):
300 commits returns `0 of 300` and is survivorship. Over all **594**: 4 (0.7%)
would have been blocked, all four the commits that ADDED a pack — the moment the
adjudication is owed. No commit has ever moved a value, so the stale-verdict arm
guards the next one rather than presenting a backlog.

## How to apply

- **A new pack, or a changed policy word, is adjudicated before it commits.**
  The gate names the line; add it to `.doctrine/advertised_policy_verdicts.tsv`
  with one of `enforced · unverified · vacuous · unenforced · undefined ·
  misdescribed · open` and the leaf that earned it.
- **Do not write `enforced` without a control that has been observed RED**, and
  name the leaf or commit that observed it in the evidence column. When a
  mechanism is present but no RED is on record, the verdict is `unverified` —
  the named gap `docs/CLAIM_VERIFICATION.md` §4 asks for, not a weaker
  `enforced`.
- **Do not collapse `vacuous` into `enforced`.** The difference between
  *refused* and *impossible* is the difference between a guarantee and a
  coincidence, and a coincidence goes false silently.
- Re-derive any number in this record with
  `python3 -B scripts/census_advertised_policies.py` and
  `… --readers`; never read it from prose, including this record.
- The four defects the census found are owned, not reported:
  `SIGNOFF-REPAIR.7.3.6.2` (the upsert writes 6 of 18 columns),
  `.7.3.6.3` (the advertisement and the enforcement are two constants a comment
  holds together), `.7.3.6.4` (the egress ladder's comparison direction) and
  `.7.3.6.5` (the undefined and misdescribed terms, including G4's open strand).

Related: `[[2026-09-12_r2-acquisition-accept-set]]` — the decision `.7.3.6.2`
found the upsert breaking; `[[a-claim-of-sameness-is-worth-its-call-graph]]` —
the shape of an advertisement outrunning its call graph.
