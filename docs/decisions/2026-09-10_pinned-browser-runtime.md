---
answers:
  - How do local and CI tests select the same browser dependency?
  - What does the testing browser archive verification establish?
  - How must a process supervisor handle a denied shutdown observation?
---
# Pin test dependencies and require observed shutdown

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.5`; REPAIR-0046.
- Evidence: docs/tasks/artifacts/signoff_review/ci-browser-runtime.md.

Browser qualification must select the tested runtime explicitly. A desktop browser
can start unrelated updater/crash-report processes, so its presence does not make
it an interchangeable test dependency. Make and CI now use a dedicated exact Chrome
for Testing version with source-pinned archive length/SHA-256, private local
installation, validated paths/internal framework links and an exact version probe.
Refuse unsupported or altered inputs before command dispatch. Keep failed evidence;
retire only successful invocation-owned payloads. Direct Cargo bypasses this setup.

Integrity is a statement about the exact acquired/extracted bytes. It does not
establish vendor signing or production isolation. The official macOS testing build
has an ad-hoc signature; do not re-sign it or change OS policy to manufacture a
different verification claim. All-platform archive validation and native macOS
execution are distinct. Hostile-content/container qualification remains separate.

Outer harness timeouts must let a launcher consume its owned children. Abruptly
killing the launcher can strand independently grouped tools. Use cooperative
signals, bounded escalation, direct-child reaping and actual group absence.

On native macOS, an owned child can become a zombie between polling and inspecting
its group, producing EPERM. Three matched before/after controls establish this
race. A denied signal/probe must remain unconfirmed, followed by bounded reaping
and fresh observation. Only actual absence permits success; a persistent refusal
still preserves evidence and fails. This rule applies to the shared Python tool
supervisor as well as the Rust browser lifetime code.
