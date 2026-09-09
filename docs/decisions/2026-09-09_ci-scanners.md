---
answers:
  - How are CI scanner downloads pinned and verified before execution?
  - Does scanner --verify-only establish a passed security gate?
  - Why did the first native Gitleaks version probe refuse a correct release?
  - Which scanner files can be retired and which evidence must remain?
---
# Verify exact scanner releases before running a gate

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.2`; REPAIR-0035.
- Evidence: docs/tasks/artifacts/signoff_review/ci-scanners.md.
- Status: thirteen focused controls, final affected configuration controls and
  native version probes pass; workflow wiring/full local/remote gates remain pending.

Replace reliance on the old action's implicit toolchain and unchecked archive
extraction with explicit cargo-deny 0.20.2/Gitleaks 8.30.1 release pins. The action's
annotated v1 tag peels to 3f4a782664881cf5725d0ffd23969fcce89fd868; its Dockerfile
uses Rust 1.71.0 and cargo-deny 0.14.21. Source mismatch is proven, remote failure
is not inferred. Pin expected archive bytes/SHA-256 in reviewed source and verify
before decoding or executing. All eight catalog archives match API and published
checksums and pass bounded executable-member inspection.

Use one exclusively created on-volume run directory. Disable default curl
configuration, clear inherited TLS/QUIC diagnostic destinations, require HTTPS
redirects and bound downloader/process/archive/member/executable resources. Extract
only the expected regular executable to a fixed exclusive path and check its exact
version. Reuse the established CI store and process primitives; selected installed
OS/compiler tools remain documented read-only inputs. This is not a filesystem
sandbox or independent publisher/security qualification.

Keep version-only evidence distinct from a passed dependency or secret gate.
The first real Gitleaks probe found an incorrect expected string despite green
instrumented controls: --version includes the "gitleaks version" prefix, while
the version subcommand emits only the number. Compare both forms on the same
hash-verified binary, correct the exact contract and preserve the failure/retry.
Never loosen accepted versions to make a surprising result pass.

Nonzero scanner results remain nonzero; completed receipt state describes an ended
operation, so inspect scope and exit_code. Retain logs, redacted reports and phase/
PID receipts. Only retire the invocation's known archive/executable after consumed
normal execution. Failed/interrupted setup retains data and its last recorded
identity; a stale started PID requires actual process inspection before cleanup.
Native aarch64 macOS execution is verified; other platform archives are not a
cross-platform runtime pass. Workflow wiring .3.3 and full checkpoint .11.4.3.1.2
must supply the remaining actual results before returning to CLI transport.
