---
answers:
  - Which items of `SIGNOFF-REPAIR.5.1`'s goal line are met by code that already ships, and by what?
  - Which are live defects, where exactly, and which child owns each?
  - In what order are the children built?
---
# The directory-isolation goal line: three items met, three live defects

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.1` (the census `MEMORY.md` named as its next action)
- **Date:** 2026-09-23
- **Work unit:** `REASONBRAID-DOC-0149`
- **Cites:** ROADMAP §10.1 (*capabilities with taxonomy identifiers, confidence, evidence, and expiry*), §10.3 (stage 1 and stage 2; *every feature has source, version, contribution, and visibility-safe explanation*; *initiator preferences within authority*), §10.4 (*dependence indicators*, never an independence claim); `docs/decisions/2026-09-23_the-calls-remote-form-was-deferred-behind-itself-and-its-first-half-is-not-federation.md` (DOC-0144, the census that measured the first item).

## The fact / decision

`.5.1`'s goal line has eight items. Its first (visibility per candidate tenant) was
repaired by REPAIR-0438 (`.5.1.1`). Of the remaining seven, read against the code at
`f324f15`:

| Item | Verdict | Where / by what |
| --- | --- | --- |
| Equally qualified foreign fixtures | ✅ MET | `the_match_surface_classifies_each_candidate_by_its_own_tenant` (`crates/reasonbraid-server/tests/profiles.rs`, REPAIR-0438): B holds the same owner-attested `code_review` claim as the member's tenant-mate, so its exclusion can only be the visibility, not its qualification. The older match control's `role B` is the owner's second role and is not a foreign fixture — it is not claimed as one. |
| Serialize updates and attestations | ✅ MET | `profiles::write_profile_in_tx` takes the role's version anchor with insert-then-lock (`.3.3.4.11.1`); the attest's read-modify-write holds it through `lock_existing_profile_anchor_in_tx` (`.3.3.4.11.2`). |
| Concurrency | ✅ MET | `presence::presence_state` reads `busy` when `in_flight >= declared` and `draining` at `0`; the default expression admits `available` only; `wake::validate` refuses a negative declaration on write. |
| Capability expiry | 🔴 LIVE → `.5.1.2` | `CapabilityClaim.expires_at` is stored and returned and never read: `matching::eligible` compares `taxonomy_id` and `confidence` only, so an expired claim satisfies a requirement (`docs/book/src/profiles.md`, *What is not here yet*, says so). |
| Missing dependence facts | 🔴 LIVE → `.5.1.3` | Three shapes, one cause — an absent fact is read as variation. (a) `rank_with_dependence` gives a candidate with NO facts a diversity of `1.0`, the maximum, where its own doc says *unknown contributes nothing, never a guess*; a candidate missing only its provider is never penalized for provider overlap, so declaring less ranks higher. (b) `dependence_indicators` explains an attribute nobody declared as *the attribute varies across the panel*. (c) the sharer test compares a value against ALL five of another member's attributes, not the same one. |
| Ranking bounds | 🔴 LIVE → `.5.1.4` | Every feature score is in `[0, 1]`, but `RankingPreferences`' six weights are unvalidated `f64`: a negative diversity weight ranks the most correlated panel first, and weights near `f64::MAX` overflow the total to `inf`, which `partial_cmp(...).unwrap_or(Equal)` then sorts as a tie. |
| Private-feature leaks | ⚠️ the three attached clauses → `.5.1.5` | `filter_profile`'s enforcement is correct; the DEFAULT policy is wider than its doc, `field_visible`'s doc example is reversed, and `ReaderClass::Full` never receives `incarnation_id` or `visibility`. |

The children are built smallest first: `.5.1.2` → `.5.1.4` → `.5.1.3` → `.5.1.5`.

## Why

A goal line is closed item by item against code, not by the absence of a complaint
(`.5.2`'s DOC-0142 and `.5.3`'s DOC-0143 did the same). Each MET verdict names the
function that meets it, so it can be re-derived; each LIVE verdict names the line that
fails, so the child starts from a measurement, not a hypothesis.

Expiry is §10.1's own field: a claim whose evidence has lapsed is not a claim the
directory may act on, and the half-open rule the grants use (`expires_at > now()` is
live) is the one to reuse — one meaning of *expired* in the repository.

Bounding each weight to `[0, 1]` loses nothing: the ranking is ordinal, so only the
ratios between weights matter, and every ratio is expressible below `1`. It makes
every contribution and the total bounded (`[0, 6]`) and finite by construction, and
refuses the one inversion §10.4 forbids by intent — preferring correlation.

## How to apply

- Build `.5.1.2` first: `eligible` takes the evaluation instant, and an expired claim
  refuses with its own reason. RED first.
- Do not re-open the three MET items without a new measurement that contradicts the
  cited function.
- The `.5.1.5` default-visibility question is a product decision (a narrow default
  makes a fresh profile invisible to discovery); it is decided in that leaf with its
  reason, not assumed from the doc comment.
