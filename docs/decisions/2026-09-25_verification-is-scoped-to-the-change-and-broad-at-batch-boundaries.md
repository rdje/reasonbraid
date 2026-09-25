---
answers:
  - How much verification does one corrective leaf need, and when does the full live suite run?
  - How is mutation testing scoped so it costs minutes rather than hours?
---
# Verification is scoped to the change, and the full live suite runs at batch boundaries

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.12.1`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-DOC-0164`
- **Source:** the director delegated it (2026-09-25) after a progress review measured where a leaf's time goes.

## The fact / decision

Measured on 2026-09-24: most of a code leaf's wall time was waiting on verification, not on the change. One `cargo mutants` run over a few functions of `reasonbraid-server` took **44–77 minutes**, because every mutant rebuilt the package's roughly fifty integration-test binaries although the mutated functions were covered by library unit tests (`m44103`: 9 mutants, 44 min; `m713`: 10 mutants, 77 min). A broad live run took about 25 minutes and was run after every leaf. Scoped as below, the same ten mutants take 14 minutes.

**Per leaf, always:** the failing control observed first; the targeted live suites that exercise the changed modules; `cargo fmt` and strict `cargo clippy --all-targets -D warnings`; mutation testing of the changed functions; hand mutants for paths only the live runner can see (SQL inside strings, cross-crate wiring), each asserted APPLIED before its run.

**Mutation testing is scoped to the tests that cover the mutants, with `--cargo-arg=--lib` (`-C --lib`).** Measured on the same ten mutants (`resolvers.rs`, `executable|select_executable`), all ten caught every time:

| Invocation | Wall time | What each mutant rebuilt |
| --- | --- | --- |
| `-- --lib` (the form used until now) | **77 min** (`m713`) | every integration-test binary: `cargo test --no-run --package=reasonbraid-server` |
| `--cargo-test-arg=--lib` | **2 h** (`m713scoped`) | the same: cargo-mutants 27 applies this flag to the test RUN only, never to the `--no-run` build |
| `--cargo-arg=--lib` | **14 min** (`m713lib`) | the library test target only: `cargo test --no-run … --lib`, about a minute per mutant |

⚠️ The middle row is a measured failure, kept because it is the obvious flag and it makes things WORSE. The logs name the exact command each mutant ran (`mutants.out/log/*.log`); read them rather than the flag's name.

**Scoping cannot manufacture confidence.** A mutant only the integration tests can see comes back MISSED under `-C --lib`, never caught, so the failure direction is a visible gap. The gap is answered as this session answered every one: a unit test when the function is pure, or a hand mutant run through the live PostgreSQL runner (asserted APPLIED before its run) when the path is SQL or cross-crate wiring. For a non-PostgreSQL integration test, scope to that one binary instead (`-C --test -C <name>`).

**The broad live run** (`RB_DEMO=0 bash scripts/run_pg_tests.sh`) is required:

- on every leaf that changes a **shared primitive**: a migration, a type in `reasonbraid-core`, the authority and transaction guards, the node-channel wire, or the router composition;
- otherwise at a **batch boundary**, at most three code leaves apart, and always before a push;
- when a batch's broad run fails, the failure is bisected over the batch's commits (each leaf is committed separately, so this is a handful of targeted runs).

## Why

- **Where the time went is measured, not assumed**, and the fix removes waiting without removing a check: every mutant is still run and every control still exists.
- **The broad run keeps a short leash.** It found a real, pre-existing failure this session (`SIGNOFF-REPAIR.11.35`: a suite red since REPAIR-0431, whose own verification ran only targeted suites). A batch of at most three leaves bounds how long such a failure can hide, and the shared-primitive rule keeps the risky changes on the per-leaf path. This is test-impact selection with a full run at integration points, the shape large codebases run.

## How to apply

- A leaf's `NO REGRESSION` box names which rule applied: shared primitive (broad run on this leaf) or batch (the batch's broad run, with its commit).
- The mutation command names its scope in the leaf, so a reader can see what was and was not rebuilt.
- Never commit while an in-place mutation run is in progress: the mutant is in the working tree, and the per-commit `RUST-FORMATTING` check reads the working tree (measured 2026-09-25, when it refused a docs-only commit mid-run).
