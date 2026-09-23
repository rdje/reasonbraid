---
answers:
  - What presence does a role report when its own availability policy keeps it from being woken?
  - Why a seventh state rather than `draining`, or `available` with a reason field?
  - Where does `held` sit in the presence precedence, and why?
  - What does a client that only reads `state` see, and what does a matching expression see?
---
# A policy-held role reads `held`, and presence has seven states

- **Type:** decision
- **Status:** accepted — decided under the director's delegation of 2026-09-23 (*your decision, state of the art and signoff-grade, in the project's best interest*)
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §10.2 (amended by this record's commit); `docs/decisions/2026-09-23_the-wake-checklist-is-a-node-gate-and-the-auto-grant-carries-six-bounds.md`
  (DOC-0135, REPAIR-0419: the wake gate at the delivery boundary); `SIGNOFF-REPAIR.11.24.1.2`
  (the honest-state repair for `busy`); `crates/reasonbraid-server/src/presence.rs`,
  `crates/reasonbraid-server/src/wake.rs`

## Context

REPAIR-0419 made two declared-and-unread profile fields into gates: a role whose `wake_policy` is
`manual_only`, or whose `operating_hours` exclude the instant, is handed nothing by the replay. It
left one thing open: such a role still read `available`. §10.2 fixed six states and none named
*enrolled, leased, deliberately not woken*. The leaf offered three shapes and stopped, because the
vocabulary is the roadmap's; the director delegated the choice.

## Decision

**A seventh state, `held`, with the reason beside it.** The state vocabulary is the one field
every client reads, and it must not lie; a reason field beside a lying state (option c) keeps the
lie where it is read most. `draining` (option b) is defined as *winding down* — declared zero
capacity — and a role outside its hours is not winding down; it is waiting for nine o'clock.
Every presence model that has had to answer this question answers it with a distinct
do-not-disturb state (XMPP's `dnd`, the calendar-driven presences of the desktop suites): a
deliberate unavailability is not a transient one and not an absence. So:

1. `PresenceState::Held`, wire `"held"`: the wake evaluator (`wake::hold`) holds the role by
   `manual_only`, by `off_hours`, or because the stored availability block is unreadable
   (fail-closed, exactly as the delivery boundary treats it).
2. Every presence read carries `hold` beside `state`, naming the reason by its wire name —
   `manual_only`, `off_hours`, `unreadable` — and `draining` for the drain switch, whose STATE
   stays `draining`. A client that reads only `state` sees the honest state; one that reads
   `hold` sees why.
3. Precedence: `unknown` > `suspended` > `offline` > `draining` > **`held`** > `busy` >
   `available`. Below `draining` and above `busy` by the durability argument the chain already
   uses: a policy hold is a declaration about the role, like zero capacity and unlike the count of
   what it holds this instant; `draining` outranks it because zero capacity winds down whatever the
   hours say; `busy` is outranked because a held role is handed nothing, so *at capacity* would
   promise capacity that is not coming.
4. ROADMAP §10.2 is amended in the same commit to name seven states — the roadmap, the code and
   the book move together, which is the repository's rule for its own vocabulary. The matching
   expression's `presence_states` defaults to `available`, so a held role is not recruited unless
   the initiator lists `held`; the eligibility filter compares wire names and needs no change.

## Consequences

- One derivation, `presence::presence_state`, gains a sixth input — the evaluator's verdict — and
  the five readers pass it: the node-channel presence read, the operator's presence listing, the
  respondent gate, the directory match and the directory presence listing. The verdict is computed
  from the same stored block the delivery boundary reads (`presence::hold_from_stored`), so the
  presence a role reports and the delivery it receives are one fact.
- ⛔ A client branching exhaustively on §10.2's states gains a value. No client in this
  repository does: the CLI prints thread states, the MCP tools pass `presence_states` through, and
  the node holds no presence mirror.
