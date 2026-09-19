answers: does a call the handler refuses still spend quota; does an idempotent replay spend quota; why does the MCP write gate commit its quota use in its own transaction; is the quota a bound on effects or on calls

# The MCP write quota counts ADMITTED CALLS, and the pipeline counts EFFECTS

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** leaf `SIGNOFF-REPAIR.6.1.4`, deciding a question `SIGNOFF-REPAIR.11.13` routed here and measuring the behaviour that was already shipped

## The fact / decision

`mcp_write::gate` commits its quota `use` row in **its own transaction, before the
handler runs**. A call the handler then refuses has still spent quota, and an
idempotent **replay** — which produces no second effect — spends quota again.

**That is intended, and it is now asserted by a control rather than only stated
in a header comment.** The per-principal write quota is a bound on **call
volume**, not on domain effects.

⛔ This record decides the SEMANTICS. It changes no behaviour: the code already
did this and `crates/reasonbraid-server/src/mcp_write.rs`'s module header already
said so. What was missing was a control that observed the two commits separately
and a record of why they are separate, so that closing the split by accident
would fail a test instead of passing silently.

## Why

**The abuse surface is the call, not the effect.** The quota exists for the
invitation-storm shape (ADR-034, §16.11): a principal making calls faster than a
system can absorb them. A caller whose every call is refused is still making
them — parsing, authorizing, hitting the database — so a bound that refunded on
refusal would give an attacker an *unlimited* supply of refused calls, which is
the surface inverted rather than bounded.

The same argument settles the replay. An idempotent repeat costs the same
admission work as the original; that the pipeline correctly declines to write a
second event is a property of the *pipeline*, not a reason to stop counting the
call.

**Measured, in both directions** (`tests/mcp_write.rs::the_quota_counts_the_admitted_call_and_not_the_effect`):

| What happened | quota `use` | quota `denial` | effects |
| --- | --- | --- | --- |
| An enrolled principal with **no** `thread_contribute` grant calls `respond` | **+1** | 0 | none — `handler:unauthorized` |
| A granted principal calls `respond` | **+1** | 0 | one `thread.contribution_submitted` |
| The **same** call again (deterministic key → replay) | **+1** | 0 | **still one** — the original event id, `replayed: true` |

⭐ The third row is the whole decision in one line: **one caller, two calls, one
contribution.** It observes the split by its OUTCOMES rather than by interrupting
between the two commits, which no test could do reliably.

⛔ A quota `denial` row is a different thing and is NOT what any of these produce.
It is written only when the *quota itself* refuses at the ceiling, and it commits
too — a refusal is a recorded fact, never silent.

## How to apply

- **Do not "fix" the gate to refund quota on a handler refusal.** It would remove
  the bound's reason for existing. If a future requirement genuinely needs
  effect-counting, that is a second meter, not a change to this one.
- **Do not move the quota check into the handler's transaction to make the seam
  look like `quota.rs`'s general pattern.** That module's header describes the
  check running in the caller's transaction, and it is right about its own
  callers; this seam deliberately differs, because it bounds admission rather
  than a guarded write. Two call sites doing the same-looking thing differently
  is not automatically a defect — the question is whether the difference has a
  reason, and here it is written down.
- **A test heading may not name the effect, the audited allowance and the quota
  use as one pipeline.** The first two share `run_thread_command`'s transaction;
  the third does not. Test 3's heading said so and was corrected by this leaf.
- Related: [[2026-09-19_the-policy-library-is-shared-the-lifecycle-is-its-tenants]]
  for the other place this session distinguished what a surface *records* from
  what it *gates*.
