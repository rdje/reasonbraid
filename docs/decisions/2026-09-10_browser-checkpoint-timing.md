---
answers:
  - Why can a render timeout fail to prove navigation cancellation?
  - How do browser tests prove overlap without assuming a fast startup?
  - What browser checkpoint evidence remains unqualified after a test failure?
---
# Qualify the intended phase with observed events

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.4`; REPAIR-0045.
- Evidence: docs/tasks/artifacts/signoff_review/browser-checkpoint-timing.md.

A timeout around launch and rendering may expire before navigation starts. A
passing timeout refusal alone cannot qualify cancellation during navigation.
Require an independently observed origin arrival and hold that response pending
until the worker returns. Budget the witness using the production startup bound;
retain a separate short cancelled-launch control.

Overlap requires both actual arrivals, two distinct simultaneously live browser
groups/profiles and no completion before the observation. Give sequential startup
phases their explicit bounded windows, release the origin on observation failure,
and consume both command results. Record errors before propagating another panic;
otherwise the most useful dispatch evidence disappears.

A deterministic startup-delay control reproduces both original test failures with
unchanged binaries. It proves the faulty timing assumptions; it does not recover
the original host's missing phase timings. Preserve original logs, executables and
fixtures. Record a failed or unstarted full gate distinctly, and resume the full
checkpoint after the bounded repair is committed. Product code and budget semantics
are unchanged by this test repair; broader host/parent/container owners remain open.

The stronger witness exposes a separate configuration boundary: desktop Chrome's
crash reporter/updaters retain stderr outside the owned browser group. Native pipe
identity establishes why EOF is absent. A dedicated official testing runtime passes
the selected delayed controls and full collection; pin and wire it under .11.4.3.1.2.5.
Do not infer untrusted-content isolation or global process containment from this
trusted-fixture result. Keep .7.3.2 open and keep honest cleanup refusals.

Verify the actual distribution contract. This macOS testing archive has an ad-hoc
linker signature; the extra Developer ID check fails and stays preserved. Exact
upstream source disables its macOS installer. Official HTTPS archive provenance,
published integrity, locally recorded SHA-256, verified extraction and exact version
are distinct from vendor signing. Do not re-sign or change OS policy to convert a
failed assumption into a claimed verification.
