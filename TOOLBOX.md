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

### The per-file validation cost is not additive when builds run concurrently

`SIGNOFF-REPAIR.11.14.3.2` re-measured the signature above during a cold
`cargo test` build and found the same mechanism at a very different scale.
Three `rustc` processes, alive 13:06, 06:17 and 04:09, had accumulated
**4.13 s, 5.74 s and 4.04 s** of CPU — and re-sampling eight seconds later
returned those three numbers **byte-identically**, while `syspolicyd` sat at
39–55 %.

```bash
# the two-sample test, over every rustc at once
for i in 1 2; do
  pgrep -f "bin/rustc" | while read p; do printf '%s ' "$(ps -o time= -p $p | tr -d ' ')"; done
  echo; sleep 10
done
ps -o %cpu= -p "$(pgrep -x syspolicyd | head -1)"
```

⭐ **The useful correction: 21.9 s is the cost of ONE validation, not the delay a
build sees.** macOS serializes them, so a compilation that loads three fresh
proc-macro dylibs waits behind every other validation the build has queued — and
a thirteen-minute block is that queue, not a hung process. ⛔ So "it has been
blocked for ten minutes" is NOT evidence of a hang on this volume, and killing a
build on that reasoning throws away the queue position it already paid for.
⚠️ The discriminator remains CPU: a queued process holds its CPU total exactly,
and `syspolicyd` is busy on its behalf. If `syspolicyd` were idle too, that would
be a different finding.

### A blocked process looks exactly like a slow one — and one run tells you neither

`SIGNOFF-REPAIR.9.2.1.1` watched `cargo check -p reasonbraid-server --tests`
apparently crawl, and it was not running at all:

```bash
for i in 1 2 3 4; do ps -o time= -p "$(pgrep -n rustc)"; sleep 5; done
```

`0:03.84` four times. **A slow process accumulates CPU; a blocked one does not,
and the wall clock cannot tell you which you have.** That much is durable, and it
is the reason to reach for `ps -o time=` before forming any opinion.

🔴 **Everything that leaf then concluded from it was wrong, and `.9.2.1.1.1`
(DOC-0018) had to withdraw it.** The mistakes are worth more than the conclusion:

- **It published a point estimate for a quantity whose spread is 4.5×.** Three
  runs of the same command, same work each time: **627.0 s, 148.9 s, 140.7 s**.
  The leaf published *2 m 39 s* from one warm run and used it to contradict an
  earlier ">10 minutes" note — which run 1 reproduces.
- **It divided two arms by hand and reported one answer for both.** `--tests` ran
  at **207 %** of a core and the lib-only check at **9 %**, a factor of 22, and
  they were published adjacent as though they agreed.
- **It named a mechanism this project had already adjudicated the other way.**
  `docs/decisions/2026-09-12_checkpoint-cost-model.md` had measured the same
  blocked-on-this-volume signature and attributed it to macOS first-execution
  validation, naming `syspolicyd` — which the leaf independently observed at 74 %
  and did not look up. `docs/CLAIM_VERIFICATION.md` leg 2 says the cheapest
  oracle is your own project's history, and that the earlier ruling wins unless
  you can name a difference. None was named.

⭐ **What the interval actually shows, and it is the useful part.** The CPU work is
constant — total CPU per run **340.0 s, 324.5 s, 309.0 s**, a spread of 31 s —
while the wall clock moves by a factor of 4.5. The build does the same work every
time; what changes is how long it waits. Run 1 spent about **472 s not computing**.

⇒ The practical rules, in the order they would have saved the work above:

1. **`ps -o time=` twice** before deciding a process is slow.
2. **Three runs, reported as an interval**, before publishing a duration. One run
   per arm is how *"these agree"* gets published when the within-arm spread is
   larger than the difference.
3. **Let the instrument compute the ratio.** `scripts/measure_check_phases.py`
   now records `user_seconds`, `system_seconds` and `cpu_percent` per phase, and
   its `--self-test` arms are the three measurements that were got wrong.
4. **Search your own decisions directory before naming a mechanism.**

## Enumerate a goal line's mechanisms against the children a split produces

When a broad leaf is censused and split, the children are drawn from the leaf's own goal
line — and nothing re-reads that goal line afterwards to ask whether they cover it. Twice
in this project a split dropped a mechanism its goal line names verbatim:
`SIGNOFF-REPAIR.3.4` dropped two, found eleven commits later; `SIGNOFF-REPAIR.7.2`
dropped one, found three commits later, and it was a live credential leak.

> Write the mechanisms down in a table beside the children, **in the leaf**. The act is
> the instrument; the count is not.

⛔ **This is a method and not a gate, measured twice over** (`SIGNOFF-REPAIR.11.15`).
Requiring the table would have fired on **13 of 14** split-declaring leaves the day it was
registered — `SIGNOFF-REPAIR.11.9`'s rejected shape. Comparing a prose count of children to
the real child count produces false positives by construction: `.11.9`'s own section says
"three children" and has one, because the sentence is about another leaf's three. And
counting the mechanisms IN a goal line is not mechanizable at all — the goal lines use
semicolons, slashes and "and" interchangeably, so any counter is guessing at grammar.

⭐ **The argument is that it pays on first use.** Writing one of the thirteen tables —
`.7.2`'s — found `max_time` declared in `GitLimits`, defaulted to 120 seconds and read by
nothing, while R0's identically-named field is enforced; and three ref-verification clauses
owned by the lane with no child. The backlog is tracked by
`scripts/census_split_coverage.py`, not by memory.

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

### A restated number needs a PRODUCER, not a rule

When a document restates a number something else computes and it goes stale, the obvious
response is a gate: *"a number in the mirror must match its census."* Ask a smaller
question first — **can the thing that owns this number print it?** If it can, delete the
number and cite the command; if it cannot, the number is history, and it needs a citation
in its own SENTENCE so a reader can see it is not a claim about now.

The reason the gate loses is that it has to tell a FROZEN historical measurement from a
LIVE claim about the current tree, and only the second can be wrong. That is a judgement
over prose — the failure mode above — and `SIGNOFF-REPAIR.11.16` priced three mechanical
approximations of it against the 103 numerals in this project's own doctrine registry:
**66%**, **50%**, and, scoped to the staged diff, **10 of the 14 commits in 200 that add
one (71%)**. `.11.9`'s gate was rejected at 87% and `.11.15`'s at 93%; `POSITIONAL-REF`
shipped at 9.5%. ⛔ A per-numeral allowlist loses to this file's own neighbour — the
`VISIBILITY-POLICY` row's *"an allowlist thirty entries long teaches bypass"*, against 103.

⭐ **And the measurement redirected the work instead of only blocking it.** Verifying the
numbers one at a time found that the four that had actually gone stale share a property
the other 99 do not: each is a POPULATION SIZE the named instrument enumerates on every
run and never printed — and each was ALSO stale in that instrument's own header comment,
so the mirror was a mirror of a mirror. Two `--census` flags fixed all four with no rule
at all. Full method and the four failure shapes:
[`docs/knowledge/a-restated-number-needs-a-producer.md`](docs/knowledge/a-restated-number-needs-a-producer.md).

⚠️ An instrument's own banner is prose too. One self-test printed a hardcoded `9/9 arms`
beside nine arms — correct, and one added arm away from publishing a false total.

### Ask the renderer, not the specification

When a rule is about a DOCUMENT FORMAT, the authority is the tool that publishes the
page — not the prose that describes the format, and not a re-implementation of it. This
is "ask the library, never restate it" (`SIGNOFF-REPAIR.11.9.1.1.2`) reaching text
instead of code, and it is the cheaper question: one render answers it in seconds.

`scripts/check_table_arity.sh` exists to catch GFM's silent cell-dropping. Its own cell
splitter treats a pipe inside an inline code span as non-separating, and its `--self-test`
asserts **0** defects for `` | `x | y` | 2 | `` — the shape at issue. Building that table
with **mdbook/pulldown-cmark, the renderer this project publishes its book with**, emits
**two** cells, `` `x `` and `` y` ``, and discards the `2` entirely. The escaped `\|` form
renders as one `<code>x | y</code>` cell, which is what GFM's spec says too.

⚠️ The consequence was not hypothetical and not visible to the gate: a census over 322
tracked markdown files found **2** rows losing content under the renderer's rule while the
gate reported **0** — one of them the doctrine registry's `INDEX-FRONTIER` row, whose third
cell rendered as a bare `—` so the published page never named the enforcer that runs it.

Two things make this worth a section rather than a note:

1. **The self-test could not catch it, for the reason `TOOLBOX.md` already records** — the
   fixtures were written by the author of the splitter, in the same mental model, so the
   arm that encodes the bug reads as the arm that proves the feature. A rendered page has
   no such loyalty.
2. **A re-implementation is not a second opinion.** Reading the spec and writing a second
   parser gives you two parsers and no oracle. Rendering gives you the answer the reader
   will actually see.

So: before asserting what a markup rule does, render one minimal example with the project's
own toolchain and read the output. If the project ships a renderer, it is already the
instrument.

### A leaf that cannot be picked up and finished is not an owner

Recording a defect against a leaf feels like ownership and is not. The test is simple and
mechanical: **can someone open that leaf and finish it?** A leaf whose goal line names six
mechanisms, carries no acceptance of its own, and has never been split, cannot be
finished — it can only be read.

Measured across the ten leaves one session's findings named as owner
(`SIGNOFF-REPAIR.11.13`): **eight had no `- Acceptance:` line of their own and six had no
children.** The one finding that WAS properly owned got that way because the clause ledger
classified it `unowned`, and that state's rule forces a new leaf. Every finding classified
`owned` or `attach` got a sentence inside a container.

⭐ **That is a gap the ledger's own vocabulary cannot see, and naming it is the point.**
`owned` means *a leaf owns the clause and its own text makes the clause visible*; `attach`
means the text does not, and is gated. Both measure whether a clause is VISIBLE to a leaf.
Neither asks whether that leaf is EXECUTABLE.

So, when routing a finding to an existing leaf, check the leaf:

1. Does it carry an `- Acceptance:` line of its own, or only a goal line?
2. Would executing it mean qualifying several unrelated mechanisms at once?
3. Is there a control someone could write tomorrow and watch go red?

If any answer is no, the routing is a note. Split the container, on the findings you
already measured — that is the cheapest moment, because the measurement is what makes the
children writable.

⛔ This is a method, not a gate, and deliberately: "this leaf is too big to finish" is a
judgement about scope, and a checker that flagged every acceptance-less leaf would fire on
most of a healthy tree's upper nodes. What IS worth keeping mechanical is the convention —
write `- Acceptance:` with the colon, so the question can at least be asked by a command.

### A count written while reading is not a count

`TOOLBOX.md` already says to measure a population before proposing a rule over it. This
is the narrower case and it is easier to miss: a number that appears INSIDE a finding,
written down while reading the code, because it felt like an observation rather than a
claim.

`SIGNOFF-REPAIR.11.9.1.1.1` recorded that a fixture accepts an unbound verdict digest
"in four places". Two tranches later a different record sent a different reader to the
same file, who ran `grep -c`: **seven**, in **seven distinct test functions**. And the
corpus had not moved — `git diff --stat` between the two commits over that file is empty,
and the count at the earlier commit was already seven. The number was wrong when written.

⚠️ The part worth keeping is where it happened: inside the clause ledger, the instrument
this project built specifically so findings would be re-derivable. Nothing about building
a careful mechanism protects the numbers you type into it.

So: **every number in a durable record comes from a command, and the command goes in the
record beside it.** `grep -c`, `wc -l`, a `--json` field, a one-line script. If you cannot
name the command that produced a figure, it is an impression with a digit in front of it.

⭐ The correction was also the cheapest possible: one `grep -c` and one `git show` at the
original commit, which together distinguish "the corpus grew" from "the number was
wrong". Ask that second question — a stale number and a false number need different
repairs, and only one of them says something about the author.

### Prefer a ratio, a single-hit grep, or a two-site contrast to a reading

Reading a function and reporting what it does is the weakest form of a source finding:
the reader has to trust your reading. Three cheap shapes carry their own proof, and
`SIGNOFF-REPAIR.11.9.1.2.3` produced one of each in a single leaf.

- **A RATIO says how much of the intended range is affected.** `run_browse` waits for the
  child to exit before reading its piped stdout — true, and unactionable. The request it
  writes sets `max_output_bytes` to **4 MiB**; an OS pipe buffer is at most **64 KiB**. The
  deadlock therefore covers everything above roughly a sixty-fourth of the configured
  limit, which is a statement about the design rather than about an edge case.
- **A SINGLE-HIT GREP quantifies over every call site at once.** "Nothing enforces the
  deadline" invites the reply "did you check all of them?". `grep -rn deadline` over every
  adapter source and the supervisor returning exactly ONE hit — the struct field's own
  declaration — closes that question in one command. ⚠️ Scope it WHOLE; a path filter that
  excludes a directory makes this shape lie (`SIGNOFF-REPAIR.11.2.2`).
- **A CONTRAST over a small closed population is conclusive rather than suggestive.**
  "`report_dead_letter` does not acknowledge its outgoing row" could be the design. There
  are exactly TWO callers of `record_outgoing_event`: `Node::emit_event` runs
  `record_outgoing_event` -> `send_event` -> `acknowledge_event`, and this one runs
  `record_outgoing_event` -> `send_event` -> `eprintln!`. Two of two is the whole
  population, so the omission is not a reading.

⭐ The same leaf found the shape a fourth time without looking for it: `codex.rs` and
`claude.rs` carry four identical defects, so every one of them is "at two sites" rather
than "in the adapter". Two sites is a claim about the codebase; one site is a claim about
a file.

⛔ None of these is a substitute for reading the source — each one came OUT of a reading.
The rule is about what you publish: when a cheap command can turn your reading into a
number, spend the command.

### A claim of sameness is worth exactly the call graph that enforces it

When one surface's documentation says it runs another's checks — *"the same queries + the
same authorization as the HTTP handlers"* — and the code beneath it is a SECOND
implementation, the sentence is holding an invariant no tool is maintaining. It is right
the day it is typed and free to be wrong from the next commit, and it is worse than
silence: the next reader trusts it INSTEAD of reading the query.

`SIGNOFF-REPAIR.6.1.1` measured the end state of that. Three MCP read tools, a header
asserting sameness, and a private re-implementation that ran none of it — one tool
declared a `principal` it never read, one took its tenant as an underscore-prefixed
unused parameter, one computed the correct foreign-reader class and returned the full
projection beside it. Live, a caller in one tenant received another tenant's entire
thread projection.

⛔ **The repair is the CALL, not the three missing checks.** Adding them beside the copy
leaves the mechanism intact and makes the sentence true, so the next omission is harder
to see. Extract the shared halves, have the second surface consume them, and delete the
copy — then sameness is a fact about the call graph and breaking it is a change the
compiler sees. ⭐ The by-product is free correctness: the seam inherits everything the
real read had accreted and the copy had not (here, a derived view and a view-backed
select — two divergences nobody had found).

Two traps met while proving it, both general:

- **A fail-fast control is the wrong arity for a claim about a SET.** The claim was "all
  three read tools"; the control stopped at the first breach and reported one. Accumulate
  each leg's verdict and assert once at the end — one red leg became eight.
- ⛔ **`git diff` cannot prove a change landed in a NEW file.** A falsification harness
  printed the diff as evidence a gate had been neutralized; the file was untracked, the
  diff was empty, and the harness reported success while proving nothing. Use an
  instrument that does not depend on the file's tracking state.

⭐ **The placement corollary** (`SIGNOFF-REPAIR.6.1.2`, the same rule arriving from the
other side): when two surfaces share a core, an authorization binding belongs in the CORE,
and the giveaway for having put it in the wrong place is that a seam-level repair would
leave a sibling caller broken **while every test you can see passes**. Measured: a finding
named an MCP seam; the HTTP verb over the same core took no tenant at all and was equally
open, so repairing at the seam would have shipped a green suite over an open surface. ⚠️ A
finding names the surface where someone happened to look — enumerate the core's callers
before repairing at the one you were handed.

Full record: `docs/knowledge/a-claim-of-sameness-is-worth-its-call-graph.md`.

### A number that decorates a finding is not covered by the finding's evidence

Three claims published in one session were graded on challenge and failed, while every
claim they were attached to held (`SIGNOFF-REPAIR.11.9.1`, 2026-09-14). The three legs had
been run on the FINDINGS — re-derived, falsified, durable. The numbers that failed were
ornaments on those findings, and nothing was watching them:

- **"the 728-line `policy.rs`"** — measured, then the same session EDITED that file. It is
  731. A constant about a file you go on to change is stale the moment you change it.
- **"five sites"** — a correct CLASSIFICATION published beside a raw `git grep`, so a
  reader re-deriving gets a different number. And the population was itself wrong:
  `src/*.rs` is a glob that does **not recurse**, and it missed a subdirectory.
- **"the third header asserting a check nobody wrote"** — it was two. The third was
  accurate about its own code; folding it in made the generalization describe a population
  that does not exist.

⛔ **The test: for each number in a published sentence, is it the CLAIM's evidence, or is it
scenery?** Scenery gets none of the three legs by default, because the legs were aimed at
the claim. Either give it the legs — pin it to a commit, publish the classification beside
the command, derive it — or cut it. A finding loses nothing by not saying how long a file
is.

⚠️ Two riders, both measured here. **Pin a census that your own change will invalidate**:
`git grep <pattern> <sha> -- <path>` costs one argument and survives the repair. And
**`*.rs` is not a recursive scan** — the whole-census warning this project already carries
still caught its own author, because the scoping mistake was in the glob rather than in
the filter.

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
| `scripts/check_file_termination.sh` | which tracked text files do not end with exactly one newline, and whether each is a reviewed generated/digest-bound exception; `--census` prints the population size, which used to be restated in prose and had drifted from 642 to 773 (`SIGNOFF-REPAIR.11.16`); `--self-test` proves the classifier | `bash scripts/check_file_termination.sh [--census] [--self-test]` |
| `scripts/check_self_tests.sh` | which tracked scripts carry a `--self-test` and are therefore RUN by the `SELF-TEST` gate, and which check/census instruments carry none and are reached by nothing. ⛔ Ask it rather than restating either number: both were written into prose as 17 and 11-of-28 and were 29 and 9-of-38 when `SIGNOFF-REPAIR.11.16` measured them | `bash scripts/check_self_tests.sh [--census] [--self-test]` |
| `scripts/check_visibility_policy.sh` | every tracked-Markdown sentence that states a private visibility for this repository, and whether it is a reviewed exception; `--self-test` proves the matcher fires and does not over-match | `bash scripts/check_visibility_policy.sh [--self-test]` |
| `scripts/check_book_frontier.sh` | whether a book page names a frontier leaf the task tree no longer has; `--self-test` proves row-1 extraction and the claim matcher | `bash scripts/check_book_frontier.sh [--self-test]` |
| `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` | the WP3 journal kill-point sweep + CLI integration tests (file-based SQLite, no service) | `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` |
| `scripts/census_reason_codes.py` | which reason codes the server emits, which the §9.8 registry publishes, and which the book documents — the three sets and their differences, with client-side sentinels excluded; `--check` is the `REASON-CODE-DOC` gate and `--self-test` proves the client/server split both ways plus the book-row matcher | `python3 -B scripts/census_reason_codes.py [--check] [--json] [--self-test]` |
| `scripts/census_split_coverage.py` | which leaves DECLARE a split, how many children each really has, and whether the leaf carries a mechanism-to-child mapping or only a prose count. ⛔ A BACKLOG instrument, never a gate: `SIGNOFF-REPAIR.11.15` measured the gate firing on **13 of 14** and rejected it as `.11.9`'s shape, and measured the count-comparison rule producing false positives by construction. It counts the ACT of enumerating, never the mechanisms in a goal line, which is not mechanizable. `--self-test`'s 6 controls prove the tree id is not hardcoded, that a grandchild is not a child, that a table is not inherited across sections, and — the blind spot this instrument shipped with — that an INDENTED mapping table is seen, which a `grep -c` disagreeing with its first number is what caught | `python3 -B scripts/census_split_coverage.py [--self-test]` |
| `scripts/census_record_reconciliation.py` | which source-census records are cited by the leaves they route to, which are not, and how far the clause ledger has classified them. `--rank` orders the backlog by each record's NARROWEST candidate leaf — ⭐ the OPPOSITE of ordering by the record's own fan-out, which `SIGNOFF-REPAIR.11.9.1` measured and rejected. `--classified` re-reads `docs/tasks/artifacts/signoff_review/RECONCILIATION.md` and refuses an unknown record, a state outside the closed six, an owner that is no leaf, a `none` row with an owner, a non-`none` row without one, a duplicated clause, or — the `ATTACH-LANDED` gate — an `attach` clause whose owning leaf's own section does not NAME its record, which is the one property a row cannot carry about itself. Never a defect count; `--self-test`'s 42 controls prove both record-id shapes, that a parent's section excludes its children, line-RANGE citation matching, the elided `` `:N` `` form and — the fifth defect this instrument had — that a SOURCE path's line number after a census citation is not one; seven more fire the `attach` rule in BOTH directions, including that a longer record id does not satisfy a shorter one | `python3 -B scripts/census_record_reconciliation.py [--uncited] [--rank] [--classified] [--json] [--self-test]` |
| `scripts/census_broken_tables.py` | every blank line INSIDE a Markdown table body, and the ROWS it orphans — a blank line ends a GFM table, so those rows render as literal pipe-text. `--check` is the `BROKEN-TABLE` gate; `--calibrate N` re-measures it across history. ⭐ Every parsing rule is mdbook's answer, not the specification's: an arity mismatch between header and delimiter is NOT a table, 4 spaces of indent is a code block and 3 is not, and a blank mid-body yields one paragraph rather than a second table. ⛔ It refuses on an unbalanced fence rather than reading every later line as fenced. `--self-test`'s 12 arms include THREE negatives — a blank that legitimately ENDS a table, a table at EOF, and a table inside a fence — because without them the rule degenerates into "no blank line near a table" | `python3 -B scripts/census_broken_tables.py [--check] [--json] [--calibrate N] [--self-test]` |
| `scripts/census_mirror_numbers.py` | every numeral `DOCTRINE_ENFORCEMENT.md`'s registry restates, classified `anchored` (its sentence cites a leaf, a commit or upstream), `structural` (a section number, an ordinal, a compound noun, a code span) or `bare`. ⛔ A REVIEW instrument, never a gate, and the rejection is MEASURED rather than remembered: `SIGNOFF-REPAIR.11.16` priced three candidate rules at 66%, 50% and — `--calibrate`, the staged-diff-scoped one — **10 of the 14 commits in 200 that add a numeral (71%)**, against `.11.9`'s rejected 87%, `.11.15`'s 93% and `POSITIONAL-REF`'s accepted 9.5%. ⚠️ `bare` is a candidate for REVIEW, not a defect: frozen-historical versus current-population is a judgement over prose (`.11.6`), and this partition only approximates it. `--self-test`'s 12 arms prove citation and sha anchoring AND its negative, all four structural exclusions, that an identifier like `REPAIR-0062` is not a restated measurement, and that an escaped `\|` does not tear a row into phantom cells | `python3 -B scripts/census_mirror_numbers.py [--json] [--calibrate N] [--self-test]` |
| `scripts/census_memory_warnings.py` | which of `MEMORY.md`'s standing warnings are anchored in a durable layer and which name no leaf — run it BEFORE evicting one at the byte cap, so the choice is derived rather than "whatever looks least costly"; `--self-test` proves the segmentation both ways (a marker after a sentence terminator opens a warning, one after `;`/`:` continues it, and a terminator inside `**`/`*`/`` ` `` still opens) | `python3 -B scripts/census_memory_warnings.py [--json] [--self-test]` |
| `scripts/census_pg_test_clusters.py` | which disposable PostgreSQL clusters the runner retained after a failure, how big and how old each is, and which are safe to retire — the §8 periodic review, with the judgement in the tool instead of in a habit; `--self-test` fires all six refusals plus the citation guard's two directions | `python3 -B scripts/project_env.py python3 -B scripts/census_pg_test_clusters.py [--retire --confirm]` |
