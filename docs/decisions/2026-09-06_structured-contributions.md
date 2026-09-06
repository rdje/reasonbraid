# 2026-09-06_structured-contributions.md

## Context

`PHASE-1.5.1` (backlog 17's first contract): the contribution body was a free
string `content` with no typing and no evidence attachments, and the §8.5
message kinds existed only as roadmap prose.

## Decision

- **The contribution `kind` is a typed deny-unknown enum over the §8.5 initial
  subset a CONTRIBUTION can carry** — `position` (the stated default: a
  contribution without a kind IS a position), `claim`, `assumption`,
  `evidence_reference`, `question`, `summary`. Out-of-registry values are typed
  refusals at the body boundary, never silently stored. Challenge and revision
  keep their own verbs (their content stays free text this leaf — scope
  discipline).
- **Evidence attaches as REFERENCES, never bytes** — `evidence_refs` is a list
  of `{uri, digest?, note?}`; accepting a reference is not a promise the core
  can resolve it (§3.7), and acquisition stays Phase 4. The event body carries
  kind + refs; the projection is untouched (content lives in the event log —
  additive event growth, pre-`.1.5.1` stored projections unaffected).
- **The CLI normalizes the human kebab spelling to the wire's snake_case**
  (`--kind evidence-reference` → `evidence_reference`; the `.1.1.3` profile
  precedent — its e2e first run taught that lesson).

## Consequences

- `thread.contribute` events now render kind + evidence refs in the inspection
  view (the events surface) — nothing silently dropped; the audit timeline
  keeps the typed facts.
- New `command_api` test (default / named / out-of-registry / foreign-ref-field
  legs) + the e2e's contribute leg extended with the flags; all twelve live
  suites + offline suites + the demo stay green.

answers:

- **A typed field's stated default must be a documented variant, not an empty
  profile.** `kind` defaults to `position` — the same shape as `.1.1.3`'s
  `single_agent` default: unnamed means exactly one thing, and the tests assert
  it.
- **Content lives in the event log; the projection keeps counts.** Typing the
  contribution body needed NO projection change — the event body is the record,
  so the additive growth happens at the event layer, and stored projections
  from before the change still parse by construction.
- **References are a Phase-1 fact, acquisition is a Phase-4 act.** `evidence_refs`
  stores what the contributor CLAIMS to cite (URI + optional digest); nothing
  in `.1.5.1` fetches, snapshots, or validates those references — the honest
  split keeps §3.7's promise.
