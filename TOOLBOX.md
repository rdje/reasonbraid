# TOOLBOX.md — the tools-first diagnostic doctrine

⛔ **TOOLS-FIRST.** For ANY unknown — a failure, a crash, a hang, a surprising result, a
"why isn't this working" — reach for a diagnostic tool FIRST. Never eyeball the code and
guess a root cause.

## The rule

- A code change cannot land without **tool-backed WHY + WHERE** and a **measured
  before→after** recorded in its task-tree leaf (see the acceptance checklist in
  `DOCTRINE_ENFORCEMENT.md`).
- If no existing tool shows WHY+WHERE, **build one** — a probe, a tracer, a counter, a
  minimal reproduction harness. The diagnostic tool is a first-class deliverable, kept in
  the repo, not a throwaway.
- **ANTI-SPIN TRIPWIRE:** if you have analyzed for ~2 turns without producing NEW tool
  output that pinpoints WHY+WHERE, STOP — run a tool, build one, or escalate. Never loop
  on analysis.

## "Remote-only" is a hypothesis, not a category

A failure that only happens on CI is not automatically a failure that needs CI to
observe. Before accepting one as remote-only, name what the runner has that this
machine does not — or, more usefully, what this machine has that the runner does
not — and take it away. Two of the three remote-only failures repaired on
2026-09-11 were reproduced locally within minutes this way, after hours of
round-trips:

- Four `git::tests::*` failed with `AuthorMissing` because the fixtures read the
  developer's ambient git identity. Suppressing git configuration reproduced it
  exactly, and exposed a fourth broken site the first repair had missed.
- A `pg_guard` fixture failed a check-then-act race that had never once fired
  locally, because `target/` stays warm between local runs and the racing branch
  is then dead code. `rm -rf` the control directory and the same test fails here
  on the first try.

The genuine kernel differences — `ETXTBSY`, the 108-byte `sun_path` limit, inode
reuse — do need a Linux runner. **Accumulated local state does not.** A cold tree
and a clean environment are instruments, and they cost one command instead of one
CI round-trip. Reach for them before spending a push.

## The 3-step UNKNOWN protocol (adapt the specific tools to your domain)

1. **WIDEN** — dump the full picture: enumerate all cases/states, the broadest inventory,
   so the failing one is visible in context.
2. **NARROW** — probe the specific failing case for its exact verdict + position/state.
3. **PINPOINT** — a scoped trace that names the exact function/rule/line that fails and why.

The point is to convert "it's broken somewhere" into "line X of function Y rejects input Z
because predicate P is false" before writing a single line of fix.

## A slow gate is a measurement, not a verdict on the gate

Before shrinking a gate because it is slow, find out whose seconds they are.
The full checkpoint's `02-check` took 3,922s and reported 940s of it. The
remaining 2,982s was not clippy, not the test suites and not rustdoc: it was
macOS validating each freshly written executable on its FIRST run, ~21.9s
apiece on the repository volume against ~0.15s on the boot volume — a fixed
cost, cached per file identity, with the process blocked throughout
(`user 0.00 sys 0.00`).

Two instruments settled it, and neither was expensive:

- **The oracle you did not build.** The Linux runner already runs the same
  commands, and its own log reports 444s with 18.7s unaccounted — from a COLD
  checkout. A local number that is 8.8x a remote one for the same work is a
  statement about the machine, not about the gate.
- **The same bytes in two places.** Copy one binary to each volume and time its
  first execution. 21.9s against 0.15s, four pairs, spread under 0.8s. That is
  a controlled experiment costing ninety seconds, and it beats any amount of
  reading about why a build might be slow.

The practical consequence is a number to plan with: **one more integration-test
file costs about 22 seconds of every future checkpoint here**, whatever it
tests. Publish that, and the next throughput argument is about evidence.

## Measure the population before proposing the rule over it

A rule, threshold or severity reasoned carefully from source is often changed by the
first measurement of the population it quantifies over. Not refined — **changed**, and
sometimes rejected outright.

Measured across the gates this project has shipped (`SIGNOFF-REPAIR.11.6`), which is the
only population where "a rule was proposed" is mechanically countable: **4 of 8 assessed
had their rule revised by the census that preceded them.** Twice the OBVIOUS rule was
rejected entirely — wrapping `git diff --check` for file termination would have flagged
nine legitimate files, and regenerating the task index's frontier column would have
destroyed accurate curated prose in 13 of 14 rows. Once the rule's KEY moved: "a closing
leaf must stage `MEMORY.md`" became a rule about the author's own claim, because the
project does not follow the obvious version (6 of 10) and it would still have missed the
defect.

⚠️ **It is 4 of 8, not 8 of 8, and the other half matters too.** One gate was measured and
shipped UNCHANGED — the cost census for running every `--self-test` found 1.01 s against a
3.15 s enforcer, so the rule proceeded as proposed. A measurement that confirms is not a
wasted measurement; it is the outcome that lets you say the number rather than guess it.

So the practice, and its honest strength:

1. Before proposing a rule, **count the population it quantifies over** — and count the
   one that would REFUTE you, not the one that confirms you.
2. If the census changes the rule, record the superseded version rather than editing it
   into agreement. The reversal is the evidence.
3. "Measured and deliberately not mechanized" is a legitimate outcome. So is "measured and
   unchanged".

⛔ This is a method, not a gate, and that is a measured decision rather than a shrug: you
cannot mechanically detect "this leaf proposed a rule without measuring its population" —
it is a judgement about prose. One narrow shape IS gated, by `GAP-CLAIM-CENSUS`: a
"nothing checks X" sentence must carry its census in the same section.

### Check an instrument's first number against one obtained a different way

Re-running the same instrument proves it is deterministic, not that it is right. Compare
its first number against one reached by a DIFFERENT route — a throwaway script, a manual
count, an existing report — before believing it.

`SIGNOFF-REPAIR.11.8`'s route census scanned line by line and reported 63 product routes.
A five-line per-file count written earlier, for another purpose, said 103. The instrument
was missing every `.route(` whose path sat on the next line — 39 of one file's 92, a 42 %
undercount that would have published a SMALLER documentation gap than the real one.

⚠️ Its `--self-test` did not catch it either, and that is the part worth remembering: the
fixtures were written by the same author in the same idiom as the bug, so they were all
single-line. **A self-test written alongside the code shares its blind spots.** The
disagreeing second number is what has no such loyalty.

### When correctness depends on enumerating what to exclude, make the failure cheap

An exclusion list has to be right. Prefer a mechanism where being wrong costs little.

`SIGNOFF-REPAIR.11.4.3.1.7.2`'s gate runs every instrument's `--self-test`, discovered by
grepping for the flag. It excluded its own path — carefully, with a comment about
self-reference. Then REGISTERING it put the flag's literal into the enforcer's
description, discovery found the enforcer, ran it with the flag, and the enforcer (which
ignores unknown arguments) ran every check including this one. Unbounded recursion on the
first `make gate`.

The exclusion by path is still there. What makes it safe is the other defence: an exported
guard variable that makes any nested invocation exit immediately, so a discovery mistake
costs one process instead of a machine.

## This project's toolbox

<!-- Fill this in as your project grows. List each diagnostic tool, what question it
answers (WHY / WHERE / how-much), and how to invoke it (binary, flag, env var). The next
agent should be able to reach for the right tool without reading the source. -->

| Tool | Answers | How to invoke |
| --- | --- | --- |
| `scripts/project_env.py` | selected repository-local stores; verified copying of locked Cargo cache data; literal-argument command execution | `python3 -B scripts/project_env.py --print` or append a command |
| `scripts/measure_check_phases.py` | where `make check`'s wall time actually goes — each phase on its own clock, plus the doc/non-doc split `cargo test --all` hides; the receipt names `accounted_seconds` vs `unaccounted_seconds` | `python3 -B scripts/project_env.py python3 -B scripts/measure_check_phases.py [--only <phase>]` |
| `scripts/ci_browser.py` | exact verified testing browser; local installation, version and consumed command receipts | `python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py --verify-only` or `-- cargo test -p reasonbraid-browse --test browser_roundtrip --locked` |
| `rb-journal` (`crates/reasonbraid-node`) | journal health (durability profile, `quick_check`, counts), pending attempts/events, ambiguous attempts with boundary history — without opening SQLite by hand | `rb-journal inspect\|pending\|ambiguous <node.db> [--json]` |
| `scripts/run_pg_tests.sh` | named suites with test-side ownership checks in a supervised PostgreSQL 16 cluster; `--list` shows names; no names runs the broad collection | `bash scripts/run_pg_tests.sh authority command_api` |
| Site authority controls | explicit operator gate, tenant-only denial, actual-parent liveness, audit rollback and ordered revocation | `RB_DEMO=0 bash scripts/run_pg_tests.sh site_authority` |
| Protected operator CLI | explicit deployment-local endpoint and operator role; issue/list/disable site authority and inspect audit | `python3 -B scripts/project_env.py target/debug/rb-site --help` |
| cold-tree probe | whether a failure needs a REMOTE or merely a clean one — a fixture whose setup branch only runs on a fresh checkout is dead code on a warm `target/` | `rm -rf target/<control-dir> && cargo test …`, repeated, since the race is probabilistic |
| clean-machine git probe | whether a test depends on the developer's ambient git identity — the `AuthorMissing` class that passes locally and fails on any clean runner | `env GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 cargo test …` |
| `scripts/check_book_links.sh` | which intra-book links do not resolve, with the page and target named; `--self-test` proves anchor stripping, external-scheme skipping and inline-code immunity | `bash scripts/check_book_links.sh [--self-test]` |
| `scripts/check_file_termination.sh` | which tracked text files do not end with exactly one newline, and whether each is a reviewed generated/digest-bound exception; `--self-test` proves the classifier | `bash scripts/check_file_termination.sh [--self-test]` |
| `scripts/check_visibility_policy.sh` | every tracked-Markdown sentence that states a private visibility for this repository, and whether it is a reviewed exception; `--self-test` proves the matcher fires and does not over-match | `bash scripts/check_visibility_policy.sh [--self-test]` |
| `scripts/check_book_frontier.sh` | whether a book page names a frontier leaf the task tree no longer has; `--self-test` proves row-1 extraction and the claim matcher | `bash scripts/check_book_frontier.sh [--self-test]` |
| `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` | the WP3 journal kill-point sweep + CLI integration tests (file-based SQLite, no service) | `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` |
| `scripts/census_record_reconciliation.py` | which source-census records are cited by the leaves they route to, and which are not — the population to classify, never a defect count; `--self-test` proves both record-id shapes, that a parent's section excludes its children, line-RANGE citation matching, and the elided `` `:N` `` form that inherits the nearest filename | `python3 -B scripts/census_record_reconciliation.py [--uncited] [--json] [--self-test]` |
| `scripts/census_memory_warnings.py` | which of `MEMORY.md`'s standing warnings are anchored in a durable layer and which name no leaf — run it BEFORE evicting one at the byte cap, so the choice is derived rather than "whatever looks least costly"; `--self-test` proves the segmentation both ways (a marker after a sentence terminator opens a warning, one after `;`/`:` continues it, and a terminator inside `**`/`*`/`` ` `` still opens) | `python3 -B scripts/census_memory_warnings.py [--json] [--self-test]` |
| `scripts/census_pg_test_clusters.py` | which disposable PostgreSQL clusters the runner retained after a failure, how big and how old each is, and which are safe to retire — the §8 periodic review, with the judgement in the tool instead of in a habit; `--self-test` fires all six refusals plus the citation guard's two directions | `python3 -B scripts/project_env.py python3 -B scripts/census_pg_test_clusters.py [--retire --confirm]` |
