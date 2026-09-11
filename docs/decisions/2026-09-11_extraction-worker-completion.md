---
answers:
  - What does the extraction spawner know about the worker process it started?
  - May an extraction failure leave a live worker with nobody waiting for it?
  - When may a caller treat an extraction worker as finished?
---
# Report direct extraction worker completion, never assume it

- Owner: `SIGNOFF-REPAIR.7.3.3.2.2`; REPAIR-0060.
- Evidence: docs/tasks/artifacts/signoff_review/extraction-worker-completion.md.
- Follows: docs/decisions/2026-09-10_extraction-input-boundary.md.

The R2 extraction spawner owns exactly one process: the direct worker child it
creates. Its returns must say which of three things is true about that child —
it was never started, this process observed its exit and reaped it, or a bounded
stop left that unresolved. Those are separate facts from the request's own
outcome. A named worker refusal is still a finished worker; a tripped time
budget says nothing by itself about whether the stop was observed.

Every return path passes through one bounded stop and reap. A worker whose stdin
is already closed gets a short grace to finish on its own before the stop
escalates to a signal; a tripped time budget skips that grace, because the
killing budget is the quarantine's enforcement. The whole cleanup is capped, so
a worker that ignores the signal cannot hold its caller open. A stop request
that fails is recorded in the evidence and the wait continues — it never becomes
a claimed termination.

A live child that no one waits for is a defect, not a tolerable leak: it keeps
reading the request's input, holds its own resources and cannot be attributed
later. Returning an error is not a reason to stop owning the process that error
came from.

An attempted kill, an error return, or a successful response is not evidence
that an input reader has stopped. Only an observed, reaped exit is. A caller
whose input lifetime or cleanup depends on the reader being finished must read
the completion evidence and retain its owned input when that evidence is
unconfirmed.

This decision covers one process. It establishes nothing about descendants that
worker may start, about pipe pressure, about total deadlines or about aggregate
retained storage; those remain `SIGNOFF-REPAIR.7.3.4`. Exclusive same-volume
input creation, source/response digest binding and cleanup gated on confirmed
completion remain `SIGNOFF-REPAIR.7.3.3.3`, which must also migrate the R2 API
caller onto the completion-bearing entrypoint. The public error vocabulary and
its classification are unchanged; only a timeout message that asserted a kill it
had not confirmed was corrected.
