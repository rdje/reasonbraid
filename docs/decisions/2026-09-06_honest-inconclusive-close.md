# 2026-09-06_honest-inconclusive-close.md

## Context

`PHASE-1.5.3` (backlog 17's third contract): the core thread machine had only
`closed` and `cancelled` terminals — a thread whose deliberation simply did not
converge had no honest way to end. ROADMAP §26.1 requires Demonstration A to
"conclude `inconclusive` with minority/unresolved items".

## Decision

- **`Inconclusive` is a core terminal state, not a doc claim.** The machine gains
  `Closing → Inconclusive` (`ThreadTransition::FinalizeInconclusive`); the
  exhaustive state-table test and the terminal-rejection test both extended (the
  canary pattern — the table is the single source of truth for the machine).
- **The close command carries the outcome and the register.** `thread.close`
  gains `outcome` (`decided` **stated default** | `inconclusive`) and
  `unresolved` (the items that prevented a decision). The outcome picks the
  terminal; the register rides the close EVENT (event-layer growth — the
  projection is untouched; the state itself is the outcome fact).
- **Honest typing at the boundary:** a `decided` close carrying `unresolved`
  items is a typed refusal — naming what is still open while claiming a decision
  would be dishonest. `inconclusive` refuses further content verbs
  (`invalid_transition`) exactly like the other terminals.
- **The demo's budget-denied thread B closes inconclusively** — the genuine
  case: the budget gate blocked the revision, the challenge stands, and the demo
  asserts both the terminal and the register.

## Consequences

- The CLI gains `--outcome`/`--unresolved` (kebab-normalized); the e2e drives the
  inconclusive close through the real binary and asserts the terminal + the
  register; the new `command_api` test covers the terminal, the register, the
  late-contribution refusal, and the dishonest-decided refusal.
- `.1.5` is complete (structured bodies, rounds, honest close — backlog 17).

answers:

- **A terminal outcome must be a machine fact, not prose.** "Honest
  inconclusive" as a doc paragraph would leave the demo unable to assert it and
  the audit unable to answer "did this thread decide?" — the state IS the answer.
- **The register is event content; the state is the outcome.** Adding projection
  fields for `unresolved` would duplicate what the event already records — the
  `.1.5.1` event-layer pattern, now three leaves old.
- **Contradictory close bodies are refusals, not warnings.** A decided close
  with unresolved items is the dishonest case the feature exists to prevent;
  refusing it at the boundary keeps every stored terminal truthful.
