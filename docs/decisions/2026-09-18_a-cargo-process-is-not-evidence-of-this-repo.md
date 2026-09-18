# A cargo process on this machine is not evidence about this repository

- Date: 2026-09-18
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.20.1`
- Related: `scripts/check_no_background_jobs.sh` (the handoff gate),
  `scripts/project_env.py` (why this repository's builds are repo-local).

## The fact

`pgrep -fl cargo` answers *is a cargo running on this machine*, which is not the
question a handoff asks. Measured at this session's start: two live processes,

```
16063 bash ../scripts/run_with_memory_guard.sh … cargo build --release --target-dir target/lowmem …
16075 …/bin/cargo build --release --target-dir target/lowmem …
```

whose relative `--target-dir` and relative script path read exactly like this
repository's. They were not. The discriminator is one command:

```bash
lsof -a -p <pid> -d cwd -Fn | sed -n 's/^n//p'     # -> /Volumes/SSD/Documents/github/pgen/rust
```

⭐ `scripts/check_no_background_jobs.sh` was right throughout — it reported
`handoff: OK` for this repository while both processes ran, because it asks
about repository-owned jobs rather than about the process table. The instrument
already knew; the reading of `pgrep` did not.

## The consequence

- ⛔ **Do not attribute a build lock, a stalled `cargo`, or a slow machine to
  this repository on the strength of a `pgrep` line.** Read the working
  directory first. A relative `--target-dir` in the command line tells you
  nothing about which tree it is relative to.
- ⚠️ The converse matters more at a handoff: a foreign build does **not** make
  this tree unsafe to leave, and treating it as a blocker stalls a clean
  handoff for another project's work.
- ⭐ The contention that IS real is physical — CPU, memory and, on this volume,
  the serialized first-execution validation
  (`2026-09-12_checkpoint-cost-model.md`). That argues for not *starting* a
  heavy build beside one, which is a scheduling choice, not an ownership claim.

## Why this is a record rather than a line in `MEMORY.md`

It was a line in `MEMORY.md`, and that is the defect it also illustrates. The
repaired `scripts/census_memory_warnings.py` classified it `UNCITED` and a hand
check found it in **no** durable layer — an environment fact living only in the
overwrite-only layer-A pointer, one eviction from being lost.
`MEMORY_ARCHITECTURE.md` §4 routes exactly this class here.
