# DEV_NOTES.md

## 2026-09-21 — Eleven findings held, two did not, and both had one root cause (`SIGNOFF-REPAIR.11.2.1.3.1.1`)

`REASONBRAID-DOC-0109`. The director's *ensure your findings still hold*, graded on all three legs of `docs/CLAIM_VERIFICATION.md` — every claim re-derived by a route **structurally different** from the one that produced it, because §1 of that standard is explicit that a repeated pass repeats its own blind spot.

- ⭐ **ELEVEN OF THIRTEEN HOLD EXACTLY.** **61 call sites** (raw grep returns 65 LINES; the four-line gap is four COMMENTS explaining the rule — the same self-reference class this batch hit twice in its own instruments); **the three breaches verbatim** at their published lines; **14 of 15** named files, re-tested per file; **496 packages**, counted from `[[package]]` blocks rather than `cargo metadata`; **59 expressions** by raw `git grep` with no classifier (17 + 42), and the **13 SOURCE** enumerated by what each JOINS; **1/2/1 → 0/0/0** on two binaries both still on disk; **both clean commits** verified clean for the stated reason; **221,496 KiB / 2,375 entries** unchanged; and **`historical-residue.json` absent from every tree of every reachable commit**, re-derived by enumeration rather than by `--diff-filter=A`.
- ⭐ **ONE CAME BACK STRONGER THAN PUBLISHED.** `CARGO_TARGET_TMPDIR` was re-tested with a THIRD instrument built to disagree: a bare crate, its own workspace, no runner, no project environment, plain `cargo test`, an integration test reporting the variable itself → `None`, while `CARGO_MANIFEST_DIR` arrives populated. That rules out every project-specific explanation the first two routes left open. Published as *unset in this project's invocation*; it is the toolchain. The stated limit — not a claim about every machine — is unchanged.
- 🔴 **TWO DID NOT HOLD, AND BOTH HAVE ONE ROOT CAUSE**: the citation census was a shell loop typed at a prompt over a **hardcoded** family list, published unpinned, with no tracked producer.
  1. **It is 13 individuals, not six.** The list was typed from what I had noticed rather than derived from the code that CREATES the families, and it was **13 families short of 26**. Seven cited individuals were never examined. ⭐ **The conclusion is unchanged and stronger: all 13 are ABSENT**, where six of six were before.
  2. **The 57 drifted and my own commits moved it** — exactly 57 at `69374f6`, **85** at HEAD by the same command, because the commits that published it added references. It carried no revision. 🔴 The class `.11.33.1` already corrected here, recurring four commits later.
- ⛔ **AND THE LEAF'S OWN ACCEPTANCE HAD SAID SO.** `.11.2.1.3.1` required *the citation census is reproducible*; I closed it having measured with a shell loop. That is `REPAIR-0364`'s defect — closing a leaf against unmet acceptance — repeating in the batch that recorded it.
- ✅ **The durability leg is now paid**: `scripts/census_fixture_citations.py` ships tracked and self-tested, deriving families **from the producer**, printing all four units so the ambiguous one cannot be picked by accident, and reporting each individual with whether it still exists.
- 🔴 **The new instrument's own first version was 13 families short too, and the shell loop is what caught it** — it missed `conformance-stubs`, built by joining `"target"` and the name in two separate statements. An instrument that misses what the thing it replaces caught is not ready to replace it.
- ⛔ No decision changed: cleanup remains safe, on stronger evidence. No product code, no gate behaviour, no threshold, no fixture removed.

## 2026-09-21 — Every individually-cited fixture is already gone, and three records say otherwise (`SIGNOFF-REPAIR.11.2.1.3.1`)

`REASONBRAID-DOC-0108`. The clause `.11.4.2.9` made non-optional: measure the citations before removing anything.

- ⭐ **THE MEASUREMENT, and it answers in the opposite direction to the ledgers.** Tracked Markdown carries **57 references into the thirteen fixture families**. Most name a FAMILY — but **six name an INDIVIDUAL entry beneath one**, which is exactly the citation `.11.4.2.9` refused to break when it found 4,092 references into the tree it was about to retire.
- 🔴 **All six are ABSENT**: `target/publisher-tests/pub-83115`, `target/conformance-stubs/lose-14317-1/claude`, `target/browser-production-controls/baseline-d1y3whgw`, `target/bench/codex-run/report.json`, `target/browser-lifetime-controls/inventory.json` and `…/candidate.log`. Three of their four families are EMPTY. `target/` is gitignored and regenerated, so a citation into it dangles **by construction**. ⚠️ **CORRECTED by `.11.2.1.3.1.1`'s verification: the population was 13 families short, so it is **13** individuals, not six — and the conclusion is UNCHANGED and stronger, because **all 13 are ABSENT**. The superseded six stand per `TOOLBOX.md`. The *57* was a per-family line sum over that short list and was published UNPINNED; at `69374f6` it is 57, at HEAD 97 lines / 119 occurrences over 26 families. Re-derive rather than read: `python3 -B scripts/census_fixture_citations.py`.**
- ✅ **So cleanup is SAFE, measured rather than hoped**: nothing live is cited, everything cited is already lost. The clause is discharged in the direction that unblocks the mechanism (`.11.2.1.3.2`).
- 🔴 **The measurement then found a defect nobody was looking for: three records assert PRESENT-TENSE retention of bytes that are gone.** `ci-checkpoint-census.md:98` — *"are retained as ambiguous/evidence-bearing residue"*; `publisher-fixtures.md:49` — *"is preserved"*; `SIGNOFF-REPAIR.md:8476` — *"Baseline runtime evidence: target/…"*. Plus `.11.2`'s own follow-up, *"Raw target/browser-production-controls and its tracked … evidence remain intact"*, where only the TRACKED half does.
- 🔴 **And one cites a durable record that was NEVER COMMITTED.** `publisher-fixtures.md:49` says *"historical-residue.json records nineteen files with sizes/hashes before final comparison"*. `git log --all --diff-filter=A -- '*historical-residue*'` returns **nothing**: it has never been added to this repository, so those nineteen files with their sizes and hashes are recorded **nowhere**.
- ⭐ **One record got it right, and its form is the rule.** `docs/evidence/2026-09-07_benchmark-codex-run.md:7` writes *"(regenerated artifacts, not tracked — the scripted self-test and this record are the durable evidence)"* — it names the artifact AND says where durability actually lives.
- ⛔ **The ARRANGEMENT was never wrong; the sentences were.** A raw fixture under `target/` plus a tracked summary under `docs/tasks/artifacts/` is the right shape, and all three summaries checked are present. Nothing in the prose distinguished the ephemeral half from the durable one.
- ✅ Four sites corrected with their superseded wording kept. ⛔ No evidence file deleted, none rewritten to make a sentence true, and **no fixture removed** — this leaf establishes that removal loses nothing cited; it does not perform one.

## 2026-09-21 — The rule becomes a gate, and the gate's own calibration found three false negatives in it (`SIGNOFF-REPAIR.11.2.1.2.3`)

`REASONBRAID-REPAIR-0369`. The gate decision `.11.2.1.2` required, with the population it rests on.

- ✅ **`RUNTIME-ROOT` SHIPS** — registered, mirrored in `DOCTRINE_ENFORCEMENT.md`, in the scaffold's NEUTRAL list, **14 self-test classifications**, whole-tree scan under a second. The enforcer now runs **26** checks.
- ⭐ **CALIBRATED over 42 commits sampled evenly across all 743**: it would have FIRED on **40**, on **24 rising to 46** sites, and fires on **0** today. The two clean commits are the initial one and the second. Zero today over a real historical population is the shape this repository registers (`REASON-CODE-DOC`); the shape it has rejected three times is a rule firing on most of a correct population.
- ⛔ **It is NOT a ban on `env!("CARGO_MANIFEST_DIR")`.** 13 uses name tracked source that travels with its crate — three `schema/` goldens, a `bench/v1` corpus, eight reads of `../../migrations` — and are correct. A pattern matching the macro itself would condemn all thirteen and teach bypass, the failure mode `.11.2.2` avoided by ruling on the ARGUMENT rather than the call.
- 🔴 **DECIDABILITY WAS THE OPEN LEG — the one `.11.31.1` declined its gate over — and the corpus settled it.** The first calibration returned `UNRESOLVED: 1` at **fifteen consecutive commits**: a base BOUND to a local and joined in a LATER statement. Following that one name to the end of the statement that uses it takes UNRESOLVED to 0.
- 🔴 **THEN THE GATE'S OWN `--calibrate` FALSIFIED THE GATE, THREE TIMES, by disagreeing with the census that opened the leaf.** It reported 42, then 43, then 44 breaches where the census said 46. Each gap was a shape the classifier could not follow, and each was repaired in the instrument:
  1. **A join that is pure traversal names the ROOT, not a destination** — `.join("../..")` was read as SOURCE because it contains no `target`, while what lands under it is decided later. Four sites.
  2. **A destination can be PUSHED rather than joined**, out of a `for component in ["target", …]` array literal.
  3. **The root can be RE-BOUND to an alias** — `let mut parent = root;` — with the pushes on the alias. This one read as **SOURCE**: a false negative rather than a fail-closed one, and therefore the dangerous direction.
- ⭐ **After all three, the two independently built instruments agree SITE FOR SITE — 46 breach / 13 source / 0 unresolved** — and `UNRESOLVED at 0` across the whole sample. That is the re-derivation leg `docs/CLAIM_VERIFICATION.md` asks for, reached through the disagreement rather than around it.
- ⛔ It **fails CLOSED** on a base it cannot place, and a **comment** naming the macro is documentation of the rule rather than a breach of it.
- ✅ **Falsified against the exact pre-repair sources** restored from `21b44c2`: red by name across all four shapes, then green after `git checkout HEAD`, `git status --porcelain` empty.
- 🔴 Two existing gates refused this leaf before it could commit and both were right: `SCAFFOLD-COVERAGE` for a registered check the scaffold did not carry, and `FRONTIER-STATUS` for a row saying `pending` while its leaf said `active`.
- 🔴 **`.11.2.1` DOES NOT CLOSE, and its own acceptance is what says so.** Both children are `done` and four of five clauses are met, but **`explicit consumed cleanup` has never been touched** — measured at **221,496 KiB across thirteen fixture families, 2,375 entries in `journal-tests` alone**. ⭐ That accumulation is the COST of the property `.11.2.1.1` bought: a fixture that refuses to reuse a directory makes a new one every run. Owned by the new `.11.2.1.3`.
- ⛔ No product code, no Rust source and no test touched by this leaf.
- promotion: declined — *measure what a proposed gate would fire on before registering it* is `.11.9`'s rejection and `.11.2.2`'s decision; *repair the instrument rather than the finding* is `TOOLBOX.md`, carried three times already in this batch.

## 2026-09-21 — Zero scratch bases left, and a clock in a shipped binary (`SIGNOFF-REPAIR.11.2.1.2.2.2`)

`REASONBRAID-REPAIR-0368`. The last seven, closing `.11.2.1.2.2` on its own executed acceptance.

- ✅ **ZERO.** The census returns **13 storage-base expressions in tracked Rust, every one SOURCE, no scratch bucket at all, `0 unresolved`** — and `git grep 'var_os("CARGO_TARGET_TMPDIR")' -- '*.rs'` returns nothing. Every base that reaches generated project data now derives the repository root at RUNTIME, and the twelve-plus-one that read tracked SOURCE keep the compile-time form, which is correct for paths that travel with their crate.
- ✅ **`rb-bench`'s clock is REPAIRED, not routed.** A shipped binary named its report directory `target/bench/<UTC %Y%m%d-%H%M%S>` and created it with `create_dir_all`, so a second-granularity clock was the only uniqueness source: two runs starting in one second shared a directory and the later overwrote the earlier's `report.json`. It now PROPOSES the stamp and PROVES it — `DirBuilder::create`, advancing `-1`, `-2` … to a bound of 64, then a typed refusal. The timestamp stays because the book documents `target/bench/<run>/` and operators read these by date. ⭐ Invisible to `STORAGE-LOCALITY` all along, because that gate matches `subsec_nanos`/`as_nanos` and this is a `format!`.
- ⭐ **Both added dependencies PRICED: 496 packages before, 496 after**, two lines in `Cargo.lock`.
- 🔴 **The browser suite failed 4 of 18 controls on the first run — and attribution was TESTED rather than assumed.** All four `worker deadline exceeded`, all writing to the correct repaired location. The pre-repair source was restored and passed 18/18 in 17.99 s; the repair was restored and passed 18/18 in 12.02 s, then three more times at loads 9.19, 9.62 and 9.69. **The same source produced both outcomes — 1 failure in 5 — so the change is excluded as the cause.**
- ⭐ **The occurrence is ROUTED rather than consumed as a passing rerun.** It goes to `.11.2`'s browser verification follow-up with its co-occurring condition: the failing run began immediately after a 2 m 22 s four-crate Clippy build — the *unit-only rebuild overlapped execution* that follow-up already names — and took **45.03 s** against 12–18 s for every passing run, at load 8.3 with 6.8 of 8 GB of swap used. ⛔ A condition, not a cause; no wait stack was captured.
- 🔴 **That routing first named `.11.26`, and reading the leaf refuted it.** `.11.26` is the **Python script gate** — `test_ci_browser.py` children exceeding a 15-second bound, 2 failures in 17 runs — a different suite in a different language. The *42 runs* phrasing that made the two feel like one belongs to `.11.25.1`. Corrected before it shipped.
- ✅ **`.11.2.1.2.2` closed on its OWN acceptance, executed** — `.11.4.2.6`'s lesson that a parent is not closed by its children — with every clause re-checked against the running system and its escape clause (*or given a leaf that owns it*) left unused.
- ⛔ One production file changed, a developer benchmark binary; its harness passes `5 passed; 0 failed` and the book's documented path shape is unchanged.

## 2026-09-21 — The server's eleven, and a correction to the number two commits ago (`SIGNOFF-REPAIR.11.2.1.2.2.1`)

`REASONBRAID-REPAIR-0367`. The first half of the decomposition; the second is `.11.2.1.2.2.2`.

- ✅ **`reasonbraid-server` contributes ZERO storage bases now** — its last 3 ambient `CARGO_TARGET_TMPDIR` readers and all 5 compile-time scratch bases anchored on `reasonbraid_core::repository_root()`. ⭐ Two of the six files already proved locality properly: `publisher.rs` walks `["target","publisher-tests"]` creating each component 0700 and asserting `symlink_metadata` and a matching device. **Only the anchor was ever wrong**, so every one of those assertions is byte-for-byte unchanged.
- 🔴 **THE REAL FINDING IS A CORRECTION TO THIS BATCH'S OWN PREVIOUS COMMIT, AND THE DEFECT WAS IN MY INSTRUMENT.** `REPAIR-0366` published the population as **47 scratch / 12 source**. It is **46 / 13**. The census read a fixed six-line window, but a Rust TAIL expression carries no `;` — so `rb-bench`'s `default_corpus_dir`, which reads the tracked `bench/v1` corpus, absorbed the `target` literal from `default_out_dir` four lines below and was classified as scratch.
- ⭐ **Re-derived rather than recomputed**: the repaired census was run against `21b44c2` in a throwaway git worktree, returning `59 expressions, 17 ambient + 29 compile scratch, 13 SOURCE, 0 unresolved`. The superseded 47/12 is left standing beside the correction at every site.
- 🔴 **A SECOND parser defect, and it is the `SELF-TEST` family exactly**: the census flagged `crates/reasonbraid-core/src/paths.rs:3` — the helper's own doc comment, which quotes `env!("CARGO_MANIFEST_DIR")` in order to explain why not to use it. An instrument that cannot tell documentation of a rule from a breach of it will always flag the file that states the rule best.
- ⛔ **Both were caught by the instrument's own `0 unresolved` requirement, not by review.** The `paths.rs` hit arrived as `UNRESOLVED: 1`, and chasing it is what exposed the window defect. A census permitted to report an unclassified remainder would have hidden both.
- ✅ 89 tests, 0 failed: `41` node_channel, `2` node_replacement, `12` node_work, `26` policy, `1` backup_restore on a disposable cluster that stopped and removed itself, plus `7` publisher. 🔴 Strict lint refused the first version — `backup_restore.rs`'s `PathBuf` import became unused when its only consumer was the base replaced.
- ⛔ No product code; all six files are test fixtures and no assertion changed.
- promotion: declined — *repair the instrument before publishing its number* is `TOOLBOX.md`, already exercised in this batch at `.11.2.1.1`; *a gate cannot see its own documentation* is the `SELF-TEST` doctrine's founding shape, recorded at `.11.2.2`.

## 2026-09-21 — The fallback was the only path, and it resolves at compile time (`SIGNOFF-REPAIR.11.2.1.2.1`)

`REASONBRAID-REPAIR-0366`.

- 🔴 **`CARGO_TARGET_TMPDIR` is UNSET here**, measured three ways — a cargo target-runner probe, the same with `CARGO_BUILD_BUILD_DIR` removed, and runner-free from `target/tmp` being empty since 2026-09-07 while today's integration run wrote to the fallback at 16:13. So the branch written as the exception at 17 sites was the only one ever taken, and a source comment asserted the opposite as fact.
- 🔴 **`env!("CARGO_MANIFEST_DIR")` resolves at COMPILE time**, so the checkout's absolute path is baked into the artifact — §12's *the repo root can be moved, even onto a different filesystem*. Confirmed with `strings`, not inferred: the pre-repair `journal_cli` binary carries `'../../target'=1  CARGO_TARGET_TMPDIR=2  abs-crate-path=1`, and `file!()` in a sibling binary is relative, so those strings are storage bases rather than debug information.
- 🔴 **The opening leaf's population was wrong and it was mine**: 47, not 17. The seventeen were only the ambient readers.
- ⭐ **The 12 SOURCE uses are why the rule is about the ARGUMENT, not the call.** A compile-time manifest dir is CORRECT for a golden, a corpus or the migrations, because those travel with the crate. A pattern matching the macro would condemn twelve correct sites and teach bypass.
- ✅ Three predicates for one question became one: `reasonbraid_core::repository_root`, with `project_storage::repository_root()` delegating. The rebuilt binary measures 0, 0 and 0.
- 🔴 The doctrine gate refused the first version for naming two children before they existed. Both exist now.
- promotion: declined — §12's *derive the root at runtime* is already the director's standing policy and is already stated in `project_storage.rs`'s own header, which is the comment this leaf moved rather than invented.

## 2026-09-21 — A name proposed ownership that only creation can prove (`SIGNOFF-REPAIR.11.2.1.1`)

`REASONBRAID-REPAIR-0365`. The parent's *per-family bounded children*, with the census run first.

- 🔴 **The parent's source list was stale by 14 of 15, and a SIBLING is what staled it.** `REPAIR-0085` (`.11.4.3.1.2.21`) removed the fractional-second naming and installed exclusive creation across that whole family; `.11.2.1` went on naming them. Measured per file: clock-named tokens **0 of 15**, exclusive creation present in **14 of 15**. The one exception, `conformance/stubs.rs`, is not stale for the reason the parent gives — its name is a process id, never a fractional second.
- ⭐ **The census, run over the whole tree instead of that list: 61 `create_dir_all` sites, every one classified, 0 unresolved** — 22 uuid-named and exclusive, 33 subdirectories of an already-owned root, 3 exclusive under another naming, **3 that name a path and then adopt it**.
- 🔴 **Its own first version reported 6.** It knew `DirBuilder::new()` as the exclusivity idiom and not `std::fs::create_dir`, which this project also uses and which equally refuses an existing path — three false positives, all in `reasonbraid-cli`. Repaired in the instrument before a number was published.
- ⭐ **What a process id buys, measured rather than argued**: it separates concurrent test BINARIES, because two live processes cannot share one; nothing WITHIN a binary, because every `#[test]` runs under one id; and nothing ACROSS runs, because an id is reused — which is the case `create_dir_all` turns from a refusal into an adoption. Reproduced against an occupied path: the superseded call returned `Ok(())` with the earlier run's sentinel file intact, the repaired one `AlreadyExists`.
- ⛔ **No collision is claimed.** The caller names in flight are distinct today, so uniqueness holds BY CALLER DISCIPLINE — which is the parent's own sentence and exactly the property replaced.
- ⭐ The dependency is PRICED: `496` packages before and after, one line in `Cargo.lock`.
- ⛔ No product code, schema, migration or wire surface; every existing assertion unchanged.
- promotion: declined — a name proposes storage and only exclusive creation proves it is already in `STORAGE-LOCALITY`'s own header; repairing the instrument before publishing its number is `TOOLBOX.md`; pricing a dependency with `cargo metadata` is `.6.8`.

## 2026-09-21 — The rule was fine; the matcher was the problem

The leaf I picked up said: install `.11.32`'s narrow rule as a gate. The narrow
rule is "an output-emitting command whose double-quoted argument carries an
unescaped backtick", and it exists because the obvious broad rule — any unescaped
backtick inside a double-quoted string — returned 19 hits of which 17 were false.
That is a sensible-looking piece of engineering: measure, find the noise, narrow
until the noise is gone.

It is also the wrong repair, and the leaf's own pre-work measurement was already
pointing at why: a regex fitted to the narrow sentence returns 13 hits, not 2,
because the false positives are backticks inside single quotes nested inside a
command substitution, and no amount of narrowing the *command* changes what
quote you are inside. The deliverable was never the sentence. It was a parser.

Once there is a state machine that tracks quote nesting, the broad rule returns
zero on today's corpus and exactly the real defects at the commit that carried
them — and the narrow rule returns the same set. The restriction bought nothing.
It had been compensating for the matcher, and it would have shipped a gate that
missed a backtick in an assignment or a heredoc for no reason anyone could have
reconstructed later.

Two bugs in my own parser were found the same way, by the corpus rather than by
reading. Scanning line by line reported six hits, all inside the multi-line awk
and Python programs these shell scripts embed: a single-quoted awk program spans
many lines, so each line starts in the wrong state. Carrying state across lines
left one survivor, in a `<<EOF` heredoc, where `'` and `"` are ordinary
characters at the top level but a nested `$( )` restores the normal grammar. The
tempting fix at that point is an exception for the file. The correct one is to
model the heredoc body as its own context, which is four lines and removes the
class.

The general lesson is the inverse of one this project already has written down: a
control can pass for an unrelated reason, and a control can also be narrowed for
an unrelated reason. When a rule has to be narrowed to escape false positives, it
is worth asking whether the false positives belong to the rule or to the thing
reading it — because if they belong to the reader, the narrowing is permanent,
invisible, and costs coverage nobody will ever re-measure.

## 2026-09-21 — I was one command away from rotating the wrong thing

The last strand of the containment lane is "collection bounds", and the largest
collection in this repository is the task trees: 5.6 MB, of which one file is
3.96 MB. Everything about that file says rotate it. It accumulates. It is
governed by the same registry as the ledgers that were rotated three weeks ago.
It is twenty times larger than the next tree and two and a half times larger than
every other governed collection put together. The remedy was sitting right there,
already built, already proved lossless on two other files.

The measurement that stopped it took one command. The ledger rotation was
authorised at .11.4.2.6.1 by a specific property, written down at the time: no
consumer cites an individual record. That is what makes retiring a record into
git history lossless rather than destructive — nothing was pointing at where it
used to be. So the question for this file is not "is it big" but "does anything
cite an individual leaf", and there is already a tracked instrument that counts
exactly that. It reports 4,092 internal references inside this one tree, 67% of
all such references in the project.

So the two collections are the same shape and have opposite retrieval models, and
the resemblance is not superficial — they accumulate for the same reasons, they
are written in the same sessions, they reach the same discomfort at the same
size. Nothing visible in either file distinguishes them. The discriminator lives
in what else in the project points into them, which is a property of the corpus.

The general form is in docs/knowledge/. What I want recorded here is the near
miss, because the reasoning that nearly did it was not sloppy: it was
"this is like the thing I fixed, and the fix worked". That is a good heuristic
and it is exactly how a remedy gets applied to a case whose properties it does
not depend on. The repair to the heuristic is small — name the property the
remedy depends on, then measure it here — and it cost one command against an
afternoon of undoing.

## 2026-09-21 — Three controls, two of them wrong, and that was the productive part

Shipping the anchor took an hour. Getting the controls to measure the anchor
rather than something adjacent took the rest, and every one of the three defects
was found by a control failing rather than by reading the code.

The falsification arm went first. It applies today's rule to a past tree and asks
whether the instances that were found by hand would have been refused. Written
naively, it required both instances to be refused at every revision it was
pointed at — so it reported a failure at the second commit, where the first
instance already had a row added a lane earlier and not refusing it is the rule
behaving correctly. The arm was measuring the calendar. Each instance is now
scored at its own commit, and the other is printed as context with its real
state, which is also more useful to read.

The second was worse because it was silent until the change landed. The closure
verification surveyed HEAD to build its prediction and then perturbed the
CHECKOUT to take its measurement. Those are the same tree right up until you are
in the middle of changing the gate, at which point the arm confidently reports
that the gate disagrees with a closure that no longer exists. The fix is one
word, and the reason it matters is that the gate judges the checkout: a census
whose only answer is HEAD is answering a different question from the check it is
about.

The third is not a defect so much as a design question the run forced. The arm
that demonstrates the original finding — a row outside the closure can be deleted
without the gate noticing — has no subject once the anchor makes every row
reachable. Deleting it would lose the demonstration; leaving it to fail would be
a control that goes red when the defect is fixed, which this lane repaired an
instance of three commits ago. It now reports "no subject: that is the repair"
and names the mode where the demonstration still lives.

Separately, one measurement corrected a sentence I wrote yesterday. Reporting
that the enforcement doctrine document is bounded by nothing is true, and the
emphasis implied the unboundedness was the anomaly. Probing its three spine peers
took four minutes and returned the same verdict for all of them, two of which
have had rows for months. The missing row was the defect; the missing bound is a
property of the class, and it now has a leaf instead of an implication.

## 2026-09-21 — A rule declined on a count nobody took

The leaf that opened this census had already written the conclusion into its own
acceptance note: a rule demanding a registry row for every Markdown file in the
tree is "a different and much worse rule". I agreed with that when I read it. It
is wrong, and the reason it is wrong is arithmetic rather than judgement.

The registry resolves by prefix — a row's path governs that path and everything
under it. So the cost of the complete rule is not the number of documents it
covers; it is the number of entries it would require, which is the minimal set of
containers whose members are all undeclared, plus the loose files that sit beside
governed siblings and cannot collapse. Eighty-nine undeclared documents come to
fourteen entries, four of which are directories carrying 49, 13, 11 and 6 members.

What makes this worth writing down is that the wrong number was never computed.
Nobody counted 432 and decided; the sentence was written from the shape of the
idea — "a row for every file" sounds like one row per file — and it then sat in
the acceptance criteria as a constraint on what the census was allowed to
recommend. A prior stated as a fact in a task leaf is heavier than one stated in
conversation, because the next reader inherits it as scope rather than as opinion.

The general form is in `docs/knowledge/a-prefix-closed-rule-costs-terminals-not-members.md`.
The local consequence is that the recommendation this census makes is the one the
leaf had pre-declined, and it makes it on a routing fact rather than the price:
`check_lesson_promotion.sh` requires every promoted lesson to land in
`docs/knowledge/`, a collection with no lifecycle, no owner and no ceiling. A gate
that moves pressure into an ungoverned destination is precisely what the closure
exists to forbid, and it has been doing it all along.

Two smaller things, both caught by controls rather than by reading. The
instrument's self-test refused a span extractor that would have lifted two shell
functions when asked for one, because the function it was asked for had no closing
brace on its own line. And the closure verification's negative arm failed on its
first run — the gate went red when I predicted green — for a reason that had
nothing to do with the closure: a different check's adjudication table must match
the registry exactly, so removing any row reddens it. The finding survived; the
control did not, and the difference between those two outcomes is the only thing
that made the run worth anything.

The entries before those above were rotated into reachable Git history at the
**third rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 69374f681f0c0f63e77776f1eaa2dac7f804c5e3:DEV_NOTES.md
```

That snapshot is 68552 bytes and 955 lines, and contains 21 dated
entries; its Git blob is `8014e41889d92a9702048c24271a5fa3cbb3cadb` and its SHA-256 is
`4e987a112e4c439d9e330242abd1b224c989cdc0a58b2b0463841dbf32f808a7`. It carries the second rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **12 record(s) rotated out, 10 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
