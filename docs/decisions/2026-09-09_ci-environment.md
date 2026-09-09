---
answers:
  - How does CI establish local writable stores before installing Rust?
  - Does the CI launcher inherit developer database or scanner overrides?
  - What happens if CI toolchain installation fails, times out or is cancelled?
---
# Establish the CI environment before installing or dispatching

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.1`; REPAIR-0034.
- Evidence: docs/tasks/artifacts/signoff_review/ci-environment.md.
- Status: launcher passes eight focused controls and eighteen adjacent controls;
  workflow wiring and actual remote execution remain separate checkpoint children.

Use scripts/ci_env.py for CI command setup. It establishes the existing configured
repository stores before any installer, disables bytecode, uses private creation
mode and clears the explicitly documented inherited database/provider/scanner/
compiler overrides. The developer project_env launcher keeps its existing
behavior; CI deliberately requires reproducible gate inputs. Neither launcher
confines arbitrary explicit output paths or every third-party cache.

The optional --rust mode requires the exact numeric repository toolchain pin.
Reuse a complete compiler in .project-data/installed-toolchains or ask the installed
read-only rustup executable to provision it there, with Cargo downloads and scratch
already local. Keep rustfmt/clippy and disable rustup self-update. Validate the
selected local executable components, then supply explicit compiler binaries.
The control installer is instrumented; successful remote downloads are not inferred
from environment/process tests.

Reuse the existing supervised process primitive for the bounded installer lifetime.
Failure, timeout or a handled terminal signal must consume cleanup before returning
and must prevent dispatch. Once setup succeeds, exec the requested command from the
root. Its own runtime budget belongs to the command/workflow. Eight real-child
controls cover relocation, pin/reuse, failure, timeout, terminal cancellation and
linked/floating inputs; eighteen existing environment/runner controls remain green.

Complete and commit this reusable prerequisite before scanner setup .3.2 and
workflow wiring .3.3. The full checkpoint still requires its fixture repairs,
actual local results and consumed GitHub outcomes before return to CLI transport.
