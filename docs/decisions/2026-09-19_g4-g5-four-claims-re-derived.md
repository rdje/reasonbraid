---
answers:
  - Do the Phase 4 G4 and Phase 5 G5 gate records still hold against the repaired code?
  - Is the "deliberation improves answers" claim still absent from the product?
  - Which part of G4 is still not re-earned, and where is it owned?
---
# G4 and G5's four claims, re-derived: three stand, one must be re-earned — and G4's single test citation no longer resolves

- Date: 2026-09-19
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.4.7.4` (under `.11.4.7`, blocker **C2** — the LAST of the five records)
- Supersedes nothing; **`docs/decisions/2026-09-07_phase4-gate-record.md` and
  `docs/decisions/2026-09-07_phase5-gate-record.md` are byte-unchanged** — `docs/decisions/`
  supersedes rather than mutates, and `.11.4.7` forbids editing them.
- Related: `docs/decisions/2026-09-19_g3-seven-claims-re-derived.md` (which routed the
  positional-pointer finding here), `2026-09-15_g1g2-sixteen-claims-re-derived.md`,
  `2026-09-15_g6g7-shipped-lines-re-derived.md`.

## Context

`.11.4.7`'s rule: re-derive every countable claim of all five gate records against the
**repaired** code and the suites that exist **today**, with a verdict from the closed set
`stands` / `narrowed` / `must be re-earned`, and the command that produces it.

This is the declared fold — G4 (one outcome) and G5 (three), **four claims**, folded because
both gates discharge their blocking function by **subtraction** rather than by evidence. G4
makes unsupported resources fail with a typed refusal; G5 withdrew the "deliberation improves
answers" claim outright after a null result. So both re-derive by asking *is the claim still
absent?* as well as *does the suite still pass?* — and `.11.4.7.4`'s acceptance requires both.

Live evidence, run 2026-09-19:

```bash
bash scripts/run_pg_tests.sh profiles evaluation routing
# profiles 56/56 · evaluation 3/3 · routing 2/2 — 61 tests, 0 failures, rc=0
```

## The pointers, resolved — and the notation settled

`.11.4.7.3` found G3 citing its evidence by test **position** and routed the question here.
⛔ **First, the instrument used to answer it was wrong and was rebuilt.** Counting every `fn`
made `routing 2` resolve to `pool`, a helper. The corrected instrument counts only functions
carrying a test attribute and self-checks against a known answer: it reports `policy.rs` = 14,
matching the 14 tests that passed. Only then were these results read.

⭐ **The notation is an INDEX, and the records settle it themselves.** Demonstration B's step 1
writes `policy 3 (the_proposal_and_the_decision_stay_separate_records)` — a number *and* a
name. At that commit `policy.rs` held 11 tests and index 3 was exactly that test, so the
"count" reading is impossible there.

| pointer | cited for | resolved then | resolves today |
| --- | --- | --- | --- |
| `profiles 23` | **G4's only test citation** | `the_g4_hostile_suite_names_every_refusal` | `a_credential_binding_is_not_inherited_by_another_tenant` |
| `profiles 27` | G5 blind commitment | `the_blind_contributions_commit_at_the_round_advance` | `the_claim_assessments_validate_the_citation` |
| `profiles 28` | G5 twelve terminals | `the_twelve_terminals_and_the_minority_report_ride_the_close` | `the_retention_enforcement_and_the_freshness_surface` |
| `profiles 31` | G5 synthesis coverage | `the_synthesis_record_is_derived_content` | `an_assessment_is_read_by_the_tenant_that_authored_it` |
| `evaluation 3` | G5 benchmark instrument | `the_calibration_accumulates_and_the_gate_only_blocks` | unchanged |
| `routing 2` | G5 routing records | `the_shadow_recommendation_records_and_never_applies` | unchanged |

🔴 **4 of 6 have moved** — `profiles.rs` went **23 → 56** tests — against 2 of 10 for G3, and
G4's *single* citation is one of them. The two that survive do so because their suites have not
grown. ⭐ Every one of the six had the suite's test count **equal to the highest index cited**
when it was written, which is exactly why the ambiguity was invisible to the author.

**All six tests still exist by name and all six pass** in the run above.

## The verdicts

### G4 — "G4 outcome: Met" (unsupported, denied, mutable or non-reproducible resources fail explicitly rather than becoming fabricated evidence)

🔴 **must be re-earned — and unlike G3's three, this one is NOT fully discharged.**

Three defects since the gate land inside the clause's own words:

- `.11.14.3.6` (REPAIR-0224) — `expected_digest` was recorded and read by nothing, so a
  **pinned** reference accepted a snapshot of entirely different bytes. A resource that changed
  became evidence without failing. *Repaired.*
- `.11.14.3.13` (REPAIR-0227) — a snapshot could record that it was an acquisition of one
  document while its reference named another; stored, `200`. Fabricated evidence, in the
  clause's own vocabulary. *Repaired.*
- `.11.14.3.12` — two acquisition arms discarded whether the evidence was persisted, so a
  pinned reference whose page had drifted returned a receipt and **no snapshot, silently**.
  *Repaired.*

⚠️ **And one strand is still OPEN, which is why this verdict is not "re-earned and discharged".**
G4's deferral 1 states that the R3 browser pack's `vm_container` requirement *stays gated in
every current deployment profile*. Measured: `crates/reasonbraid-server/src/resolvers.rs:303`
emits `"container_required": true` — a declaration **about the deployment**, not a property the
product enforces. `.7.3.6` is **open** over exactly this and two further advertised-but-unverified
R3 policy lines (`javascript_policy: "allow-bounded"`, `egress_class: "listed"`), after `.7.3.5`
(REPAIR-0210) found the pack advertising `redirect_policy: "deny"` and
`subresource_policy: "deny"` and enforcing **neither**. An advertised `deny` that is not enforced
is a *denied resource that did not fail explicitly* — G4's clause, from the inside.

The hostile suite the record cites, `the_g4_hostile_suite_names_every_refusal`, exists and
passes; it exercises destination refusal, sandbox and egress requirements, forged fields and
media types. It did not and does not cover the three defects above.

### G5 claim 1 — "Met as a subtraction gate: the product makes NO 'deliberation improves answers' claim"

✅ **stands**, and the census carries a positive control because a zero-hit grep is an absence
claim (`an-absence-claim-is-a-census-over-the-corpus`).

⛔ The first pattern returned **zero** and was **wrong** — it spelled `quality lift` with a
space where the corpus writes `quality-lift`. Corrected:

```bash
git grep -niE "quality.?lift|deliberation improves|improves? (answers|quality)|outperform" \
  -- README.md LIVE_STATUS.md docs/book/src        # 3 hits
```

All **3** are withdrawals, not claims: `LIVE_STATUS.md:1719` (*"Historical G5 subtraction gate
withdrew the quality-lift claim"*), `docs/book/src/qualification-review.md:12` (*"quality-lift
claims retain their"* gating) and `docs/book/src/roadmap.md:99` (*"G5 withdrew the quality-lift
claim"*). The README's mechanism claim is intact verbatim at line 9. The **H1 null** result is
preserved in `docs/evidence/2026-09-07_benchmark-codex-run.md` (lines 13, 40, 69) and has not
been re-litigated. And `git log --diff-filter=A 076e06f..HEAD -- docs/evidence/` returns **two**
files, both gate packages — **no new benchmark run**, so deferral 1's "one feasibility sample"
still describes the whole corpus and no cross-run quality claim can exist.

### G5 claim 2 — "the honest-inconclusive half is SHIPPED and tested"

✅ **stands.** `the_blind_contributions_commit_at_the_round_advance`,
`the_twelve_terminals_and_the_minority_report_ride_the_close` and
`the_synthesis_record_is_derived_content` all exist and pass. ⚠️ All three of its pointers
moved; the claim is true and the record's way of citing it is not.

### G5 claim 3 — "the benchmark-threshold half is BUILT as an instrument"

✅ **stands.** `evaluation` 3/3 and `routing` 2/2, and both pointers still resolve —
`the_calibration_accumulates_and_the_gate_only_blocks` and
`the_shadow_recommendation_records_and_never_applies`. The shadow-only design of deferral 3 is
asserted by the second test's own name.

## The subtractions, as the acceptance separately requires

**G4's five deferrals.** (1) the R3 container gate — **narrowed**, `.7.3.6` open, above.
(2) the RX delivery — **holds**: `git grep -c "capability_call\|CapabilityCall" --
crates/reasonbraid-server/src` returns no hit, so the agent round-trip still does not exist.
(3) the media formats — **holds**: the typed `media_type_unsupported` refusal ships and carries
its own control (`crates/reasonbraid-extract/src/main.rs:141`, asserted at `:742`).
(4) the browser-engine provenance — **holds**: the engine is still not vendored and the
`R3_BROWSER_BIN` override carries it (`crates/reasonbraid-browse/src/main.rs:169`).
(5) the snapshot auto-submission — **holds**, and its worst case was repaired: three
auto-submission sites exist in the acquisition path (`api.rs:2208`, `:2311`, `:2482`), and
`.11.14.3.12` closed the arms that discarded the result.

**G5's subtraction.** Holds, on the positive-controlled census above. The absent claim is still
absent.

## Consequences

- **Blocker C2 closes.** All five gate records are now re-derived: G1–G2 (`.11.4.7.2`,
  REPAIR-0198), G6–G7 (`.11.4.7.1`, REPAIR-0195), G3 (`.11.4.7.3`, REPAIR-0264), and G4+G5
  here. ⛔ No gate record's conclusion changes: G6–G7 remains NOT MET for Internet exposure and
  G3 remains blocked as binding use.
- G4's open strand is `.7.3.6`, and it is named as such rather than folded into this verdict.
- Both gate records and both subtraction records stay byte-unchanged.
- ⛔ Still no doctrine gate for positional pointers, for the reason `.11.4.7.3` gave: four
  hand-written records is not a population (`.11.6`). All four re-derivation records now cite
  by test name, which is the remedy.
