answers: which gate is authoritative before a push; do I have to run the two-hour local checkpoint before pushing; should I change a macOS security setting to speed up the build; should the repository move to the boot volume; what should I run locally before pushing

# The remote run is the authoritative pre-push gate; the local checkpoint is a diagnostic

- **Type:** `decision`
- **Date:** `2026-09-12`
- **Status:** accepted
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.3.1.2.28`
- **Builds on:** `docs/decisions/2026-09-12_checkpoint-cost-model.md`, which measured the
  cost and left the two levers open. This record closes them.
- **Delegation:** the director delegated both open findings explicitly on 2026-09-12.

## Context

The full local checkpoint's `02-check` costs 3,922 s, of which 2,982 s is macOS
first-execution validation of each newly written executable on the repository
volume (~21.9 s apiece, fixed, cached per file identity, ~0.15 s on the boot
volume). The cost-model record named two levers and deferred both. They are
decided here, together with the question they were really about: **which gate is
authoritative before a push.**

## Decision 1 — the macOS security setting is REJECTED

Adding the shell to the system's Developer Tools privacy category would exempt
**everything that shell runs** from Gatekeeper assessment. Two reasons, either
sufficient:

- **It is the wrong trade on this project specifically.** ReasonBraid
  deliberately executes untrusted content: the browser worker renders arbitrary
  pages, the extraction worker parses hostile documents, and the threat model
  (`ROADMAP.md` §16.6) treats every fetched byte as untrusted. Turning off the
  operating system's own validation of freshly written executables, on the
  machine where those workers are built and run, buys throughput with exactly
  the control that is load-bearing here.
- **It cannot be committed.** It lives in a system preference, not in the
  repository. `MEMORY_ARCHITECTURE.md` §2 is explicit that a store missing any
  durability property is cache, never the system of record — and a performance
  fix that no other machine, no fresh clone and no runner inherits is a local
  secret that silently falsifies the published cost model for everyone else.

## Decision 2 — moving the repository is REJECTED, on measured capacity

Feasibility was measured rather than assumed, and the two halves disagree:

| Quantity | Measured |
| --- | --- |
| tracked content plus full history (`git count-objects -vH`) | **3.77 MiB** |
| build tree (`du -sh target`) | **185 GB** |
| boot volume available (`df -h /`) | **249 GB** |

The repository itself is trivially movable. Its **build tree is not**: §13
requires project data to live on the repository's volume, so `target/` follows
the checkout, and hosting 185 GB on the boot volume would consume 74 % of its
remaining space and leave 64 GB on the volume that also carries the OS and the
user's home — for a tree that grows and is rewritten continuously.

Note also what §13 does and does not say: it requires project data on **the
repository's own volume**, not on a particular volume. There is no policy defect
here to repair. The measurement is the first evidence that the *choice* of
volume carries a large hidden cost, and that is now published in the cost model
rather than discovered again.

## Decision 3 — change what the gate IS, not what it costs (ADOPTED)

The lever that was never on the list is the one worth pulling. **The remote CI
run is the authoritative pre-push gate. The full local checkpoint becomes an
explicitly invoked diagnostic.**

This is a superset, verified from the workflow files themselves rather than from
prose — every one of the checkpoint's eight commands runs remotely:

| Checkpoint command | Remote equivalent |
| --- | --- |
| `cargo build --workspace --bins --locked` | `rust.yml` check job |
| `make check` (fmt · clippy `--all-targets --all-features` · build · `ci_browser -- cargo test --all`) | `rust.yml` check job, plus explicit `test -x` worker assertions the local run does not make |
| `unittest discover -s scripts/tests` | `rust.yml` pg-tests job |
| `run_pg_tests.sh --demo` | `rust.yml` pg-tests job |
| `make gate` (the doctrine enforcer) | `doctrines.yml` |
| `ci_scanners.py cargo-deny` | `supply-chain.yml` |
| `ci_scanners.py gitleaks` | `supply-chain.yml` |
| `make book` | `rust.yml` book job, which additionally pins and asserts mdBook 0.5.4 |

Two of the eight are **stricter** remotely. None is weaker. And the remote is
where the defects that actually escape have lived: every one of the six repaired
in the recent remote-CI sequence — `ETXTBSY`, the 108-byte `sun_path` limit,
inode reuse, `AuthorMissing`, a `pg_guard` check-then-act, and a stub race — was
invisible on this machine.

**What to run locally before a push**, all of which cost seconds because none of
them links or executes a new binary:

```bash
make gate                      # the doctrine enforcer (the pre-commit hook runs it too)
make book                      # the rendered book
cargo fmt --all -- --check     # formatting
python3 -B scripts/project_env.py python3 -B -m unittest discover -s scripts/tests -p 'test_*.py'
```

The full checkpoint driver stays available and tracked for the case it is
actually good at — reproducing something without spending a push — and its
measured cost (>2 hours; `01`–`04` alone were 7,447 s at `165cb3a`) is stated up
front so nobody starts it by accident.

## Consequences

- `COMMIT.md`'s push cadence no longer requires a complete local checkpoint
  before every push; `docs/ci.md` records which gate is authoritative.
- **No gate is removed, weakened, reordered or skipped.** Every command still
  runs; the decision is about *where*, and the where is strictly stronger.
- **The trade this creates, named rather than hidden:** with the authoritative
  gate remote, gate latency is bounded by the push cadence (~300 commits). That
  cadence is the director's standing instruction of 2026-09-11 and is **not**
  changed here. One adjacent fact for that decision, since it was part of the
  cadence's stated rationale: GitHub-hosted runner minutes are free for public
  repositories, and this repository is public and must remain so.
- The residual risk is a run of commits that accumulates a failure the focused
  per-commit checks cannot see. The mitigation is unchanged and already in
  place: the pre-commit hook runs the enforcer on every commit, §16's focused
  checks run per slice, and the full local checkpoint remains one command away
  when a change is risky enough to want it.
