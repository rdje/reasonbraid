---
answers:
  - Why can a completed CLI writer leave state.lock busy?
  - Does the controlled fork probe prove the exact original checkpoint cause?
  - What does process exit guarantee when lock descriptors are inherited?
---
# Bind state-lock release claims to descriptor ownership

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.11.1`; REPAIR-0055.
- Evidence: docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md.

The unchanged CLI API reproduces lock retention after success, HTTP error and
future cancellation when a forked child retains its actual publication descriptor.
No-child controls release immediately; exiting the child releases all three
retained locks. This is a concrete close-only guard defect with immediate repair
owner .2.11.2. Preserve legitimate active-writer exclusion and snapshot boundaries;
do not serialize tests or retry away a leaked lifetime.

The original source-8d1504d checkpoint failed an initial independent flock before
the entrypoint assertion. Its fixture was deleted during panic cleanup, so its
exact holder/spawn interval is unavailable. Unchanged concurrent, isolated and
serial reruns pass. The controlled mechanism is established; attribution of that
specific historical incident remains an inference, not an observed trace.

Apple's flock/fork contract and the actual descriptor witness show that
close-on-exec does not prevent inheritance before exec. Explicit normal release
can end a shared lock despite retained child references, but destructors do not
run after process death. Do not promise immediate crash release while inherited
references survive; qualify that boundary separately under the existing broader
CLI interruption/restart leaf .3.3.4.3.3.3.3.3. Never delete state.lock to bypass
contention or infer server rollback from any local lock outcome.
