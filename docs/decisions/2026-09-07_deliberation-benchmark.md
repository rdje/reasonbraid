# The WP7 deliberation/routing benchmark (PHASE-0.7)

- Date: 2026-09-07
- Status: accepted
- Owners: Richard DJE (engineering + product)
- Scope: KICKOFF WP7 / `ROADMAP.md` §13.7 (hypotheses H1, H6): a small versioned
  corpus compared across four deliberation workflows, with deterministic
  grading and per-case uncertainty — the evidence the Phase 1 routing decision
  and the WP8 memo rest on.

## Decisions

1. **The benchmark is an offline harness over the adapter contract**
   (`BENCH-001`). The delivery platform (server → inbox → node → thread) is
   already proven by `.6.2`; the benchmark's object is WORKFLOW QUALITY VERSUS
   COST, so it drives [`Adapter`] implementations directly — the deterministic
   scripted agent (default, CI-safe) or the real Codex adapter
   (`RB_LIVE_CODEX=1`, call-budgeted). This keeps the measurement instrument
   hermetic: the workflows (single / blind-independent / critique-revise /
   moderator-synthesis) are deterministic orchestration over adapter calls.

2. **The corpus carries its own oracle** (`BENCH-002`). Every case ships
   scripted outputs AND the scores the harness MUST produce for them; the
   self-test asserts computed == expected over the whole corpus. The
   measurement pipeline is proven before a single real token is spent — a
   wrong grader or a miswired workflow fails loudly, never publishes wrong
   numbers.

3. **Grading is deterministic — never an LLM judge** (`BENCH-003`). Factual
   cases match accepted answer keys; rubric cases score a presence checklist;
   confidence is a REQUIRED parsed number (`CONFIDENCE: 0.XX`); citations are
   range-checked against the case's source list (fabricated `CITE[n]` counted).
   An LLM-as-judge would share the measured models' correlated errors — the
   exact failure mode the benchmark exists to detect.

4. **No independence score, no similarity gate** (`BENCH-004`). Agreement
   between the independent answers is a descriptive count with the explicit
   warning that agreement ≠ correctness; the report's typed shape has no such
   metric, and a test forbids one.

5. **Uncertainty is first-class** (`BENCH-005`). Every case row carries the
   declared confidence beside the score; aggregates are spread-bearing
   (n/min/mean/max), Brier is computed for factual cases only (no invented
   calibration), the moderator workflow must emit a structured `UNRESOLVED`
   register (structure failures are recorded, not papered over), and
   unmeasured metrics (human-review minutes — a WP8 human study) are recorded
   as `not_measured`, never zero.

6. **The corpus is versioned and hashed** (`BENCH-006`). `bench/v1/` carries
   `corpus.json` (8 cases: 2 factual, 2 code-review, 2 ambiguous-policy, 2
   insufficient-evidence — a feasibility sample, §13.7) and `prompts.json`
   (4 workflows × 4 classes); both SHA-256 digests ride every report, so a
   result is traceable to the exact inputs.

7. **Real runs are deliberate and bounded** (`BENCH-007`). `--agent codex`
   refuses without `RB_LIVE_CODEX=1` and enforces `--max-calls` up front (the
   feasibility run: 4 differential cases × 4 workflows = 36 calls).

## Honest boundaries

- The insufficient-evidence honesty trap detects asserted NUMBERS; qualitative
  overclaims need the WP8 human review (documented in the grader).
- The scripted mode proves the harness, not model quality; the real run is the
  quality evidence, and a null/negative result is an accepted outcome that
  narrows the routing claim.
- ADR-017 (cited by the tree leaf as roadmap context) is not a repository ADR;
  the repo has ADR-001 only. This record is the WP7 design authority; the
  routing consequences land in the WP8 decision memo.
