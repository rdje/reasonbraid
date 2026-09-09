---
answers:
  - How should browser tests establish original listener shutdown when ports can be reused?
  - How is concurrent profile isolation qualified without relying on a short overlap window?
  - What verifies browser storage after moving the runtime root?
---
# Qualify browser storage and origin ownership by identity

- Owner: `SIGNOFF-REPAIR.11.4.3.1.5.3`; REPAIR-0040.
- Evidence: docs/tasks/artifacts/signoff_review/browser-combined-qualification.md.

A later connection to an old address is not evidence that the original listener
survived. The failed integration run did not capture its successful connection's
peer. A separate native control closes its first listener, binds an owned successor
to the same port and receives a successor-specific payload; even the numeric file
descriptor is reused. Keep the original failure and distinguish this demonstrated
mechanism from the unknown identity of its peer.

Wrap the test listener with a dedicated close receipt sent only after its socket is
dropped. Origin.finish must consume both its serving task and that exact receipt.
The live-listener control first requires an empty receipt. Delegate accept/error
handling to the pinned axum listener implementation; do not replace its behavior
or weaken graceful connection completion.

Hold the two overlap navigation responses behind an explicit origin gate until the
observer establishes both live groups and distinct profiles. Release on successful
observation, caught observer failure and origin shutdown. Delay the second launch
by four seconds after the first navigation arrives, deliberately exceeding the old
three-second scroll-based window. A timing window cannot substitute for observing
the intended concurrency state.

Use the identical committed production worker before and after renaming a private
runtime root. Require the same device/inode, successful real renders, distinct
invocation directories, absence of completed directories and an unchanged witness.
Also invoke the real worker against an owned linked storage parent: it must refuse
before browser ownership and leave its target untouched.

Fifteen integration controls and strict focused lint pass; five production unit
controls retain their separately recorded REPAIR-0039 evidence against unchanged
production source. Every result is consumed. Twenty-six final groups are independently
absent; successful fixtures are gone and the nine prior failed fixtures remain.
This closes the worker/test prerequisite, not parent termination, retained-data quotas,
detached-process containment, host startup investigation or full local/remote CI.
