# Where the checkpoint's wall time actually goes

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.15`; REPAIR-0090. Date: 2026-09-12.
Instrument: `scripts/measure_check_phases.py` (tracked).
Raw evidence: `target/check-phases/run-20260911T224102Z/` (six phase logs + `phases.json`).

## The gap, re-derived before anything was measured

`target/checkpoint-ci/full-165cb3a/summary.txt` records `02-check rc=0 seconds=3922`.
From its own log `02-check.log`:

| Quantity | Command that derives it | Value |
| --- | --- | --- |
| cargo-reported compile | `grep -oE 'Finished .* in .*' 02-check.log` -> `9m 38s`, `0.46s`, `3m 55s` | 813.46 s |
| harness-reported test time | 97 `finished in Ns` blocks, summed | 126.95 s |
| unaccounted | 3922 − 813 − 127 | **2,982 s (76 %)** |

The leaf's three numbers reproduce exactly. Its leading candidate — nine rustdoc
doctest-harness builds — is named there as a candidate and not as the answer, and
it is refuted below.

## What the phases actually cost

`make check` run phase by phase, each with its own clock, plus the doc/non-doc
split that `cargo test --all` hides:

| Phase | Seconds | cargo-reported | harness-reported |
| --- | ---: | ---: | ---: |
| `fmt` | 0.8 | — | — |
| `clippy --all-targets --all-features` | 293.1 | (in log) | — |
| `build --workspace --bins` | 10.4 | — | — |
| `test --no-run` (compile every test target) | 61.3 | — | — |
| **`test --lib --bins --tests`** | **3,162.8** | 0.26 | 119.52 across 88 blocks |
| `test --doc` | 310.5 | 207.0 | 0.00 across 9 blocks |
| **total** | **3,838.8** | 637.3 accounted | **3,201.5 unaccounted (83 %)** |

**The doc phase is not the answer.** It costs 310 s of a 3,839 s run — at most 8 %,
and 207 s of that is rustdoc compilation cargo does report. The candidate is
refuted by measurement, not by argument.

**The answer is `test-run`: 3,162.8 s, of which 0.26 s is compilation and 119.5 s
is the 88 harnesses' own time.** That leaves ~3,043 s spent neither compiling nor
testing, while 88 test binaries were executed.

## The oracle nobody had to build

The same commands run on the Linux runner, from a COLD checkout, in run
34652116508's `check` job. Re-derived from that job's own log rather than from a
notification:

| | Linux runner (cold) | This machine (warm) |
| --- | ---: | ---: |
| step wall time | 444 s | 3,922 s |
| cargo-reported compile | 381 s across **514** `Compiling` lines | 813 s across **12** |
| harness-reported test time | 44.28 s / 97 blocks | 126.95 s / 97 blocks |
| **unaccounted** | **18.7 s (4.2 %)** | **2,982 s (76 %)** |

The workflow step runs `cargo fmt`, `cargo clippy --all-targets --all-features`,
`cargo build --workspace --bins` and `ci_browser.py -- cargo test --all --locked`
— the same gate. The runner compiles 514 crates from cold in less time than this
machine compiles 12 warm, and has essentially no unaccounted time at all.

**The duration is therefore NOT inherent to `--all-features` clippy followed by
`cargo test --all`**, which is the question the leaf asked. It is local.

## Root cause, pinpointed

A freshly linked test binary, timed directly:

```
first execution : real 31.17  user 0.00  sys 0.00
second execution: real  0.00  user 0.00  sys 0.00
third execution : real  0.00  user 0.00  sys 0.00
```

`user 0.00 sys 0.00` against 31 s of wall time means the process did no work of
its own — it was blocked outside itself — and the result is cached per file
identity afterwards.

The same bytes, copied and executed for the first time on each volume in the same
minute:

| Repository volume `/Volumes/SSD` | Internal boot volume |
| ---: | ---: |
| 21.66 s | 0.12 s |
| 22.42 s | 0.17 s |
| 22.36 s | 0.12 s |
| 21.60 s | 0.19 s |

Mean ≈ **21.9 s** against ≈ **0.15 s** — about **150×**, with a spread under
0.8 s on the slow side, so this is an interval and not a lucky point estimate.

Three further facts fix the mechanism:

- **The cost is fixed, not proportional to size.** An 81.6 MB binary costs
  21.25 s; a 2.3 MB binary costs 21.6 s. Reading bytes is not what is happening.
- **No third-party agent is involved.** `ps -axo comm` matches only Apple's own
  stack: `XProtect`, `XprotectService`, `xprotectd`, `syspolicyd`.
- **There is no file-level lever.** `com.apple.provenance` is kernel-managed and
  survives `xattr -c`; clearing it changed nothing (21.56 s against a 19.69 s
  control).

⇒ **macOS first-execution validation of a newly written executable costs about
21–22 seconds per executable on this repository volume, and about 0.15 s on the
boot volume.** It is paid once per file identity and cached thereafter, which is
why a warm re-run of the same binary is free and why every checkpoint — which
relinks — pays it again.

## The predictive model

    checkpoint test time ≈ (distinct newly-written executables actually run) × ~21.9 s
                           + the harnesses' own time

88 test binaries alone predict ~1,930 s. The measured overhead was ~3,043 s,
implying roughly 139 first executions — more than 88 because suites spawn
freshly built executables of their own (the browse and extract workers, `rb`,
the node binaries, the conformance stubs), and each pays the cost once.

That residual is stated as a consequence of the model, not measured per spawn:
the per-suite spawn census is not run here.

## What this does and does not establish

- It explains the 2,982 s the checkpoint could not account for, and it identifies
  a cause outside the workspace and outside cargo.
- It does **not** identify a repair that this repository can make in its own
  sources. The levers are a system security setting or the repository's volume,
  and both are the director's to decide — see
  `docs/decisions/2026-09-12_checkpoint-cost-model.md`.
- No gate was weakened, skipped or reordered to produce these numbers. The phase
  split runs strictly more than `make check` does, not less.
- The clippy phase measured 293 s here against the checkpoint's 578 s because
  this tree was warmer. Compile times are cache-dependent and are not the claim;
  the per-executable constant is, and it is cache-independent by construction.
