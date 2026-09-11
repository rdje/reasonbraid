# The browser worker's refusal carries the render fact and the cleanup fact separately

- **Type:** `decision`
- **Date:** `2026-09-12`
- **Status:** accepted
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.3.1.2.27`
- **Supersedes:** the response tail introduced with the owned-browser lifetime
  (`.11.4.3.1.2.3`), which relabelled a render refusal as
  `browser_cleanup_unconfirmed`.

## Context

The R3 browser worker finishes with two outcomes that are established
independently:

- the **render** either produced a response or refused with a named kind
  (`time_budget_exceeded`, `navigation_failed`, `output_too_large`, …);
- the **cleanup** either observed the owned browser group exit and its owned
  tasks be consumed, or it did not.

The superseded tail carried both in one `kind`. When cleanup was unconfirmed it
wrote `browser_cleanup_unconfirmed` into `kind` and preserved the render's kind
only as a prefix inside the human message. A caller reading `kind` therefore
lost its own actionable fact — the budget it set, the selector it supplied —
and received the operator's fact instead.

That collapse is why the navigation-deadline control passed on the remote runner
and failed on this development machine. The control asserted
`kind == "time_budget_exceeded"`, and the value of that field depended on a
second, unrelated condition: whether the host was slow enough for a browser
helper to outlive the ten-second cleanup budget. The control was asserting a
conjunction it never intended.

The underlying slow-host condition is already diagnosed and is not new: detached
`chrome_crashpad_handler` and `GoogleUpdater` processes retain the browser's
inherited stderr write endpoint after the browser group exits, so no EOF arrives
and the stderr reader cannot be confirmed finished
(`docs/tasks/artifacts/signoff_review/browser-checkpoint-timing.md`). The pinned
Chrome for Testing runtime removed the common trigger; it did not remove the
class, and it cannot, because the worker does not own processes that leave its
group.

## Decision

**A refusal carries both facts, explicitly, and neither is inferable from the
other.**

| Render | Cleanup | `kind` | `cleanup_confirmed` | `cleanup_error` |
| --- | --- | --- | --- | --- |
| succeeded | confirmed | *(success response)* | — | — |
| refused | confirmed | the render's own kind | `true` | absent |
| refused | unconfirmed | **the render's own kind** | `false` | the cleanup detail |
| succeeded | unconfirmed | `browser_cleanup_unconfirmed` | `false` | the cleanup detail |

Three points settle the doctrine tension deliberately rather than by preference:

1. **The rule is never to CLAIM an unobserved termination — not to destroy the
   other fact.** Carrying `cleanup_confirmed: false` satisfies the rule exactly,
   and loses nothing. Destroying the render's kind satisfied it too, but paid a
   caller's fact for it.
2. **The success/unconfirmed corner keeps its name, because there the refusal is
   doing work rather than labelling.** A successful render has no failure of its
   own to report, and returning it would let a caller treat the invocation as
   complete and retire the workspace while a browser may still be running. That
   corner is unchanged from the superseded contract.
3. **`cleanup_confirmed: true` on a refusal raised before any browser was
   spawned is not an unbacked claim.** The field means *no owned browser process
   or task is known to outlive this worker*, and a refusal with nothing spawned
   satisfies it by construction. That is the only way it is true without an
   observation, and it is stated in the type's own documentation.

The cleanup detail is **also** appended to the message. That duplication is
deliberate: the server-side spawner reads only `kind` and `message`, so until it
reads the new fields the message is the only channel carrying the operator's
fact into its log. Both are written from one expression in `settle`, so they
cannot drift.

## Consequences

- `crates/reasonbraid-browse/src/main.rs` gains `BrowseError::cleanup_confirmed`
  / `cleanup_error` and the pure `settle` combinator that holds the table above.
  The decision is a total function over four cases, so it is asserted by a unit
  test on every host rather than by waiting for a slow one.
- The wire change is **additive**. `crates/reasonbraid-server/src/browse.rs`
  reads the error envelope as a `serde_json::Value` and takes only `kind` and
  `message`, so it is unaffected; the success shape is untouched.
- Eleven distinct render kinds stop being relabelled, not one. The budget is
  simply the kind that exposed it.
- The acquisition failure kind the R2/R3 caller sees
  (`api.rs`, `BrowseError::WorkerRefused { kind, .. }`) is now the render's own
  kind. Today the server's own deadline usually preempts the worker's, so this
  becomes caller-visible for the other ten kinds immediately and for the budget
  once `.7.3.1` repairs the spawner/deadline interaction.
- **Not taken up here:** having the server-side spawner read and surface
  `cleanup_confirmed` rather than relying on the message. That belongs to the
  spawner's own integration and is named in `.7.3.1`.
- This changes no cleanup behaviour, no budget, and no process handling. It
  changes only which facts the response states. The escaped-writer condition
  remains real and unrepaired; it is now reported honestly instead of
  overwriting the caller's result.
