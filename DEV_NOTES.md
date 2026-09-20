# DEV_NOTES.md

## 2026-09-21 — The word that was wrong and looked right

`ordinal_word(0)` returned `'twentieth'`.

Not an exception, not an empty string — a real English ordinal, correctly
spelled, drawn from `ORDINALS[-1]` because Python indexes backwards from the end
without complaining. And `render_footer` asks for exactly that: it computes
`prev_word = ordinal_word(ordinal - 1)` so the notice can say which rotation came
before this one.

So the first rotation of any new ledger would have written, into a governed file,
under a heading that says every figure in it was re-derived rather than typed:

> It carries the twentieth rotation's notice in turn, and each earlier notice
> names the one before it, so the chain walks all the way back.

There is no twentieth rotation of that ledger. There is no first one yet. The
sentence is false in every clause, and a reader has no way to tell, because it is
the same sentence the thirty-seventh rotation writes truthfully.

I found it by reading the chain-start path before writing any code for it, which
is the only reason this is a note about a defect that never shipped rather than
one that did. The fix is two lines — refuse below 1 instead of wrapping — and the
interesting part is not the fix.

The interesting part is that this defect was unreachable for the entire life of
the tool. `apply` refuses before `render_footer` is called if there is no
predecessor notice, so no input could reach the wraparound. It became reachable
the moment I set out to let a ledger start a chain, and it would have fired on
the very first use of the new path — on a real file, writing a real false
sentence into a real notice. A latent defect is not a harmless one; it is one
whose guard is an accident of the call graph, and the accident expires when
someone adds the caller the guard was never designed to cover.

The other half of the leaf is a refusal I put in deliberately, in the opposite
direction. `--bootstrap` starts a chain; on a ledger that already has one it
would reset the ordinal to 1 and orphan every earlier notice — the one genuinely
irreversible thing this tool could do to a file. So it refuses, by name, saying
which ordinal it found. Both directions have a control. A flag that can only do
the right thing on the right file is worth more than a flag with a warning in its
docstring.

And a sequencing rule I had to obey rather than admire: this entry is being
written before the rotation runs, not after. `REPAIR-0350` published the tool's
own stdout as the committed result and `DOC-0090` had to correct it, because the
same commit then appended its changelog entry and moved every figure. A ledger
rotation is the one operation guaranteed to be invalidated by the commit that
performs it. So the entry goes in first, the rotation runs last, and every number
published afterwards is read back off the committed file.

Postscript, written after the rotation ran, because the best part happened last.

The rotation appended its notice to the end of the file. When a notice replaces
an earlier one there is a tail after it that ends in a newline; when it is
appended there is no tail, so the file ended without one. `FILE-TERMINATION`
caught it and refused the commit, which is exactly what it is for.

Then I re-ran the bound probe from three commits ago — the one whose whole
purpose is proving that a file is or is not bounded — and it reported that all
five core live documents now refuse oversize input. Including `LIVE_STATUS.md`,
which I have not touched, and which has no bound at all.

The probe appends bytes to one file and runs the whole enforcer. The enforcer
answers about the tree. So any breach anywhere in the tree — in this case my own
missing newline, in a completely different file — reads as the probed file being
bounded. The instrument written to prove an absence had started reporting a
presence, for a reason that had nothing to do with its subject.

That is the failure mode this repository has a promoted note about, and I wrote
the probe that fell into it, and the probe's own leaf is where the note is cited.
The only reason it did not ship is that the result was implausible: I knew
`LIVE_STATUS.md` was unbounded, so "it refuses now" was a claim I had a reason to
doubt. Checking the refusal *reason* rather than the return code took one command
and turned five-of-five into the true four-of-five.

There is a general shape here worth more than the incident. **A probe that asks a
composite instrument a question about one input can only be read alongside the
reason it gives back.** The return code is the composite's verdict on everything,
not the probe's verdict on the input. Either the probe must isolate the check it
means, or it must parse the reason and confirm the right subject is named. Mine
now does the second, by hand, in the leaf.

And the smaller rule the same run produced: `--plan` said the ledger would land
at 36,525 bytes and the committed file is 37,874, because the notice's own bytes
are part of the result and the plan is computed before the notice exists. Read
the number back off the file. Always.

Second postscript, because the commit would not land.

`LESSON-PROMOTION` refused it: *"1 new lesson entry in DEV_NOTES.md with NO
promotion and NO explicit decline"*. The staged task tree carried fifty-three
lines matching that gate's own decline token, including the one in the owning
leaf. The gate was reading the opposite of what was in front of it.

The scan was `grep -E "$DECLINE_TOKEN \(..*\)" "$f" | grep -vqF "…"`. The `-q`
makes the consumer exit on its first match, which closes the pipe, which sends
SIGPIPE to the producer, which exits 141, which `set -o pipefail` promotes to the
pipeline's status, which the `if` reads as "no decline found".

It is a race. It only fires once the producer's output is big enough that it is
still writing when the consumer leaves. Measured on the real tree, where the
matches now total 17,388 bytes: 34 of 40 runs returned non-zero. On a file with
one matching line: 0 of 40. **The gate did not change. The task tree grew past
it.**

Two things about this are worth more than the fix.

The first is the direction of failure. This gate fails CLOSED — it refuses
correct work. That is the safe direction, and it is exactly why nobody found it:
a gate that wrongly passes is eventually caught by the defect it admitted, while
a gate that wrongly refuses is caught by an author who assumes they are the one
who is wrong and goes looking through their own commit for the mistake. I did
that for several minutes before I suspected the tool.

The second is that the self-test could not have caught it. Nine controls, all
green, all testing `lesson_promotion_verdict` — a small pure function taking
three integers. The verdict was never broken. The break was in the shell that
computes those three integers, and that shell had no controls at all, because
extracting a pure function to make it testable is satisfying in a way that
testing the messy part is not. A control over the decision says nothing about the
extraction that feeds it.

So the new arm builds 400 long matching lines and scans them twelve times. Against
the repaired code it passes; against the old form, restored deliberately, it
reports "MISSED on 9 of 12 runs over a large file — the SIGPIPE race is back" and
exits 2. A fixture of one tidy line would have been green on the broken code,
which is the whole reason the fixture is 400 lines of noise.

And the gate had one more thing to say: my first version of that arm called
`mktemp -d`, which follows `TMPDIR`, which is on another volume here.
STORAGE-LOCALITY refused it by name. Three refusals in one commit, all three
correct, two of them mine.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.3` — the unreachable
guard is a further instance of
`docs/knowledge/a-guard-your-own-process-cannot-reach.md`, and the sequencing rule
is `.11.4.1.6`'s, already carried in the tree and in the resume pointer.

## 2026-09-21 — The number I typed before I measured it

Two things happened while generalising the ledger rotation to a second file, and
the second one is the one worth keeping.

The first was ordinary good luck of the kind a careful process manufactures. I
parameterised `rotate_changelog.py` by ledger, widened its record boundary to the
one quoted from `check_lesson_promotion.sh`, and then had to prove that none of
this changed the ledger already in production. The previous leaf had already
shown the two boundary patterns agree on all 664 versions of `CHANGELOG.md`, and
I could have leaned on that. I did not, because a count agreeing is not the same
as a split boundary agreeing — two patterns can find the same number of headings
and still cut in different places. So `--plan` was captured before the first
edit, and diffed after the parser change and again after a later fix. Identical
output, same SHA-256, all three times. That is the claim I can actually make.

The second thing is that while writing the `Ledger` record for `DEV_NOTES.md` I
needed a threshold, and I typed `65000`.

There was no reason for that number. It was a placeholder to make the dataclass
valid while I finished threading the parameter through, and I fully intended to
derive it afterwards. Which I did — and the derivation came out at 76,309.

Had I been interrupted, or had the derivation felt like a formality after the
code already ran green, `65000` would have shipped. It would have been a
threshold in a governed registry with no producer behind it, sitting in the
repository as a number that looked derived because it lived next to numbers that
were. That is the entire subject of `SIGNOFF-REPAIR.11.6` — measure the
population before proposing the rule — and I committed a small version of it
inside the leaf whose one job was deriving that ceiling.

The real derivation is worth stating because it makes the number arguable rather
than personal. `CHANGELOG.md` has a reviewed threshold of 96,000 bytes and a
measured p90 entry of 4,734 bytes per non-rotation commit. That is a live window
of 20.279 entries. The question "how big should the second ledger be?" then stops
being a preference and becomes "the same window, in that ledger's own units":
20.279 × 3,763 = 76,309, rounded down to 76,000 because rounding up would grant
headroom the derivation does not support. Both p90s come from the same function,
so the ratio is not quietly comparing two different quantities — which is the
mistake I nearly made earlier by reaching for the p90 *record size* instead of
the p90 *commit delta*.

One more note, on the calibration. The rule fires on 385 of 430 versions of
`DEV_NOTES.md` — 90%. This project has already rejected a gate for firing on 114
of 131, so 90% should stop you. What rescues it is not an argument, it is the
shape: the first 45 versions are clean, version 46 crosses, and every version
after that fires with no recovery. A rule that fires on a scattered 90% is
describing itself. A rule that fires on a contiguous tail after a single crossing
is describing a file that went over its bound once and was never rotated — which
is precisely what was measured. And the remedy clears it in one command: retire
427 records, land at 36,774 bytes, ten commits of runway, no fire. The rejected
gate had no such remedy. That asymmetry, not the percentage, is the test.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.2`. The crash — the
tool raising `ValueError` on a ledger that had simply never been rotated — is a
further instance of `docs/knowledge/an-instrument-must-explain-its-own-failure.md`,
already promoted. The derived-once-held-fixed rule is a disposition about these
two files rather than a transferable method, so it is a decision record instead.

## 2026-09-21 — Two gates over one file, disagreeing about what a record is

Before pointing the changelog rotation at a second ledger I asked what I thought
was a formality: what is one record in `DEV_NOTES.md`?

The rotation splits on `^## \d{4}-\d{2}-\d{2}`. The gate that has governed this
file since it was ported — `check_lesson_promotion.sh` — splits on
`^## .*[0-9]{4}-[0-9]{2}-[0-9]{2}`, and its self-test explicitly pins both
spellings the file uses, the bare `## 2026-09-04 — …` and the italicised
`## _(2026-09-04)_ — …`.

On this file those two regexes are not a stylistic difference. Narrow sees 291
records and 717,211 bytes. Wide sees 439 and 895,767. **148 records and 178,556
bytes — a third of the file — are invisible to the parser the rotation would
have used.**

A rotation is not a formatter. It splits a ledger at record boundaries, retires
the tail, and writes a chain notice saying how many records it retired and where
they can be retrieved. Run on a parser blind to a third of the records, it
retires content it never counted and publishes a number that is false in its own
notice. This project has already been bitten once by a rotation instrument whose
parser could not see the thing under investigation: a count-based detector
reported 43 rotations where the history has 45, and the two it missed were the
two being investigated.

The fix is not to pick the better regex. It is to notice that the question was
already answered. `check_lesson_promotion.sh` is not a bystander here — it is the
gate that governs this file, its definition of a record is enforced on every
commit, and it has been right about both dialects the whole time. So the rotation
adopts that definition rather than carrying its own. A second definition of one
fact is the failure `SCAFFOLD-COVERAGE` and `INDEX-FRONTIER` were both written to
refuse, and I was two minutes from writing a third.

Two things made this safe to conclude rather than merely plausible.

First, widening the parser must not change the ledger already in production. I
did not argue that; I walked all 664 versions of `CHANGELOG.md` and compared both
counts on each. They agree in every one. Zero disagreements.

Second, a wider parser can be wrong in the other direction. On `DEV_NOTES.md`,
441 headings exist and the wide parser matches 439. The two it skips are
`## clause-1 [org-baseline 1.0.0]` and the template placeholder
`## _(YYYY-MM-DD)_ — bootstrap`, which carries no real date. Both are correctly
not records. The parser is exact here, not merely generous, and I checked that
because an over-matching boundary would corrupt a rotation just as thoroughly.

And the part worth keeping: **my own first split of this file used the narrow
parser.** I reported 291 records and a p90 of 3,467 bytes, and only noticed
because 291 dated headings against 441 total looked like too large a remainder to
ignore. The true p90 is 3,199. Had I derived a rotation threshold from the first
number, it would have been 268 bytes per record too high — wrong in the
conservative direction, which is exactly the kind of error that never announces
itself downstream. The instrument's first population described its parser, not
the file. That is a promoted lesson here, and I still walked into it inside the
leaf whose entire subject is two parsers disagreeing.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.1` —
`docs/knowledge/an-instruments-first-population-describes-its-parser.md` already
carries the statement, and the quoted-not-chosen rule is a disposition about this
repository's files rather than a transferable method, so it is a decision record
instead.

## 2026-09-21 — A closure is only as wide as the document it is anchored to

This repository has one routing-closure check, and it is a good one. It refuses
any destination the README links, or the guard's own failure hint emits, or a
registry row's control column names, unless a governed row covers it. It follows
routes transitively through that third leg. It has been green for months.

It cannot see `DEV_NOTES.md`.

The reason is not a bug in the check. It is the check's key. The closure
enumerates *paths named by the landing page* — that is its population, and the
population is complete with respect to that key and no wider. `README.md` has
never linked `DEV_NOTES.md`. What names `DEV_NOTES.md` is `COMMIT.md`, step 3,
which requires an update to it on every single commit. So the repository has an
author-overflow destination that the commit workflow makes mandatory and that
the closure protecting against exactly that pressure has never enumerated.

The measurement: 428 versions, 427 of them larger than their predecessor, and
**zero bytes removed in the entire life of the file**. It is the purest
append-only file here, at 894,723 bytes.

The second finding is worse in kind, because this one had a row. The registry
gave `LIVE_STATUS.md` the pressure control *"current status table, overwritten
rather than appended"*. Over its 632 versions: 614 grew, 17 shrank, 646,037
bytes added against 39,251 removed, and the current version is the file's
all-time maximum. The declared control is false, and its own history is what
refutes it. Nothing checks a declared control — the guard validates that a row
has four non-empty fields, which is arity, not truth. A free-text field that
nothing reads is a comment with a pipe character in it.

What makes both of these a defect rather than a fact of life is sitting one row
away in the same registry. `CHANGELOG.md` takes the same append pressure under
the same per-commit mandate, and **1,099,952 bytes have been removed from it
across 46 shrinks**, holding a 663-version file at 48,896 bytes, below its peak,
with a rotation tool and two gates on it. Containment is not theoretical here.
It is running, next door, on the neighbouring file.

The part I want to remember is how the absence was established. "Nothing bounds
this file" is an absence claim, and this project has been burned by absence
claims taken from searches. So the instrument does not search: `--probe-bounds`
appends 200,000 real bytes to the real file, runs the real enforcer, and
restores the file byte-identically. `README.md`, `MEMORY.md` and `CHANGELOG.md`
each refuse, and each refusal names a size cap — those three are the positive
controls, in the same run, and without them rc=0 on the other two would be
indistinguishable from a broken gate. The baseline mattered too: the enforcer
was confirmed green before the probe started. And every file came back with a
matching SHA-256, because a falsification that leaves the tree changed has
measured the gate and damaged the repository in the same run.

Promotion: declined, and recorded in `SIGNOFF-REPAIR.11.4.2.5`. Both statements
are confirming instances rather than new ones —
`docs/knowledge/a-census-is-as-wide-as-its-key.md` already answers *why did a
route census miss an endpoint*, and
`docs/knowledge/trust-comes-from-the-check-not-the-shape.md` already carries the
declared-but-unchecked shape. Neither clears the threshold `SIGNOFF-REPAIR.11.6`
holds for a statement the layer lacks.

## 2026-09-20 — A procedure can be fully specified except for the one number that decides everything

The changelog rotation here has a decision record, a chain of notices each
naming its predecessor, byte-exact retrieval and a losslessness argument. It is
one of the most carefully specified procedures in the repository. It says
nothing about how much to retire, and that is the only parameter that decides
whether the procedure runs again tomorrow.

I performed two rotations and chose, both times, the minimum that cleared the
threshold: 344 and 296 bytes of headroom. Every rotation before mine left a
median of 18,741. The first of mine forced the second one commit later. Nothing
in the record told me I was doing it wrong, because the record had never
considered the question.

**The measurement had to be taken twice.** The first instrument called a
rotation a fall in the dated-entry count and reported 43. A minimal rotation
retires exactly as many records as its own commit adds, so the count does not
move — meaning the detector was blind to precisely the class under
investigation, and the two rotations I was trying to measure were the two it
could not see. Detecting a heading *disappearing* gives 45.

**The target is now derived rather than chosen**: ten commits of runway at the
p90 entry size over the last sixty non-rotation commits. The point is not the
number, it is that nobody has to pick one. A typed constant would have been
right today and wrong in a month, which is the failure this whole file is about.

**And the choice between p90 and median was made on a table, not on taste.**
Asked of the whole ledger history, the p90 bar fires 64 times and fails to
predict 1 of 45 rotations; the median bar fires 37 times and misses 13. For a
rule whose false positive costs one early rotation and whose false negative is
the defect itself, recall wins. Writing that down is what makes it reviewable.

**One thing is recorded unresolved.** After registering the check, the gate
stalled about four minutes with the trace inside the new script and essentially
no CPU consumed. The obvious explanation — concurrent gate runs deadlocking on a
git index lock — was tested and refuted: no gate check takes one. What remains
consistent is this machine's recorded behaviour when a freshly written
executable is first run by path, since the same file ran in a second through an
explicit interpreter and has been fast since. That is an observation with a
refuted alternative, not a root cause, and it is written down that way. The
practical consequence is real: the enforcer invokes checks by path, so the first
gate run after adding a check script can stall here.

Promotion: **declined** — recorded in the leaf. The blind detector is another
instance of `an-instruments-zero-describes-its-reach`; the derived target is
`TOOLBOX.md`'s *derive, never carry*. Both are already in the retrievable layer.

## 2026-09-20 — Two instruments that disagree are worth more than two that agree

`SIGNOFF-REPAIR.11.4.2.4.1` shipped `POINTER-CURRENCY`, the gate its parent leaf
left owed after declining §6's generator. The gate itself is small. What was
worth the day is how its two defects were found.

**Defect one was found by a falsification that failed to falsify.** Three
deliberate breaks were written into `MEMORY.md` and the check returned rc=0 on
all three. Not because the rules were wrong — because the check reads the INDEX,
which is what a commit carries, and the breaks were unstaged. The check was
right and the test was meaningless, and the two are indistinguishable from the
outside. This repository has now met that shape four times. The preference stays;
the divergence is announced, so nobody can read a hand-run as a verdict on what
is on screen.

**Defect two was found because two instruments disagreed.** The census said the
history holds 17 defects. The gate, calibrated over the same history, fired 151
times. Neither number is self-evidently wrong, and the temptation is to trust
the newer instrument. Classifying the 140 excess instead: **136 had the very
commit under test as the owner of the work unit it named** — the gate's history
walk started at today's `HEAD`, so it could see the commit it was judging, and a
pointer correctly naming its own commit looked like one reusing an old id. At
real pre-commit time `HEAD` is the parent and that commit does not exist yet.
The rule was right; the harness was wrong.

**The part worth keeping is that neither instrument could have found it alone.**
The gate would have shipped with a calibration claiming it blocks 151 of 646
commits, which is the shape `SIGNOFF-REPAIR.11.9`'s rejected gate had at 114 of
131 — a number that would have been read as *this rule is too aggressive* rather
than as *this harness is looking at the wrong history*. The census would have
gone on reporting 17 with nothing to contradict it. What surfaced the defect is
that they were asked the same question by different routes and came back with
answers that could not both be true.

And a third, smaller: **my first fix for `check_tree_index_frontier.sh` was
refused by that gate's own self-test.** The shorthand `.5.3` under tree `PHASE-8`
was being offered a second reading — `PHASE-5.3`, another tree's leaf. A reading
that can match anything is a checker that cannot refuse, and the control that
caught it is one that looks redundant until the moment it isn't.

Promotion: **declined** — recorded in the leaf. Defects one and two are both
`an-instruments-zero-describes-its-reach`, promoted two commits ago, met from
the calibration side. `SIGNOFF-REPAIR.11.6`'s threshold wants a statement the
layer lacks; the layer has this one.

## 2026-09-20 — A rate needs a denominator, and a field that says "derive it" is not in the denominator

`MEMORY_ARCHITECTURE.md` §6 prefers a derived resume pointer to a hand-written
one. `SIGNOFF-REPAIR.11.4.2.4` owed the measurement, and the measurement's whole
difficulty turned out to be the denominator rather than the count.

**The question as posed.** *How often has each derivable field in `MEMORY.md`'s
current-state block been wrong?* Over the 645 commits that have touched the file,
the naive answer is a ratio of defects to versions. That ratio is meaningless:
for most of its life `latest_commit` did not carry a value at all — it read
`derive on read with git log -1 --oneline`, which cannot be stale. Counting those
645 versions as 482 correct ones would have published a 1.1% drift rate for a
field whose real rate, where it makes a claim, is **4.3%**.

**The answer, per field, over what carried a value.**

| field | carried | derive-on-read / no claim | wrong | rate |
| --- | --- | --- | --- | --- |
| `latest_commit` | 162 | 482 + 1 unreadable | 7 | 4.3% |
| frontier leaf | 385 | 222 prose | 10 | 2.6% |
| ahead-of-origin | 174 (173 transitions) | 140 + 331 absent | 152 | 88% |

**What decided the generator question was not the rate.** Three things did.
First, the 88% field is the one the pointer *already stopped carrying* — it now
states the derivation command — so the working repair cost no instrument at all.
Second, `latest_commit` was derive-on-read for 75% of its history and every
defect is in the remaining quarter; the corpus had run the experiment. Third and
decisively, the disagreement between the pointer and the tree is **symmetric**:
in 5 of the 10 frontier mismatches the tree's own row 1 was the copy that had not
moved, because a parent leaf had just been decomposed and the pointer named the
first child. A generator that treats row 1 as ground truth would have written the
wrong value in half of them. So: generator declined, check owed, calibration done.

**The instrument was wrong three times, and each error had the same shape.**
227 bad SHAs where the history has 2 — a `review baseline` cited beside a
derivation instruction, read as a latest-commit claim. 34 frontier disagreements
manufactured by concatenating `PHASE-1` with `.1.1.1`. A SHA quoted inside a
bullet's prose outranking the work-unit id that opened it, scoring one claim 601
commits stale. Every one of the three is a statement about the instrument's reach
that I first read as a statement about the history — the lesson promoted one
commit earlier, met three more times inside the tool written to avoid it.

**And the correction worth keeping is the one that changed nothing.** The truth
table was keyed off the pointer-touching commits alone, so a parent that never
touched `MEMORY.md` looked as though it had no work-unit id — a census drawing
its ground truth from the same population as its subject. Fixed to walk the whole
`git log`; the verdicts were identical before and after. A fix with no effect is
still a finding, and recording it is what stops the next reader assuming it was
load-bearing.

Promotion: **declined** — recorded in the leaf. These are further instances of
`an-instruments-zero-describes-its-reach` and of
`calibrate-over-the-history-that-contains-the-instance`, both already in the
retrievable layer. `SIGNOFF-REPAIR.11.6`'s threshold wants a statement the layer
lacks, and this leaf produced instances rather than a new rule.

## 2026-09-20 — A snapshot of a transient defect measures how often it is repaired

`SIGNOFF-REPAIR.11.22.1` opened on a frontier table naming one unfinished leaf
at two rows. Its census was careful and it was a snapshot:

> A census over the `## Current Frontier` table of all **15** tracked trees
> returns exactly **one** leaf carrying more than one UNFINISHED row.

One instance across fifteen trees reads as *rare*. The leaf was written on that
reading — worth a gate arm, maybe, if the calibration supported it.

The calibration, over the whole history rather than HEAD:

```text
commits the rule would have BLOCKED: 38 of 672
distinct leaves implicated         : 12
  SIGNOFF-REPAIR.11.14.3.10: 24 commits
  SIGNOFF-REPAIR.11.2.4:      5 commits
  SIGNOFF-REPAIR.3.5.4:       4 commits
  … eight more
```

Twelve leaves, not one. And `.11.14.3.10` held the shape for twenty-four
consecutive commits — a defect that was present, in the table a fresh session
resumes from, for the better part of a week.

⭐ The reason the snapshot said "one" is not that the census was careless. It is
that **the defect is transient**: someone eventually notices a duplicate row and
deletes it. A count at HEAD therefore measures *how many instances happen to be
un-repaired right now*, which is a function of how quickly they are repaired —
not of how often they occur. The two numbers differ by whatever the mean
lifetime is, and here that is large enough to turn 1 into 12.

**The rule:** for a defect that gets repaired, a census at one commit is a
lower bound with an unknown multiplier. If the question is *should we gate
this*, walk the history.

⚠️ And the leaf's prediction turned into an observation while I was working it.
It said *"a later commit promoted the same leaf to row 1, the pair came back"*.
I did exactly that twice during this session — `.11.7.1` at rows 1 and 4, then
`.11.22.1` at rows 1 and 5 — and both commits passed `make gate`, because two
`pending` rows for a `pending` leaf agree with the leaf and the two existing
rules had nothing to object to. Then, closing this leaf, I did it a third time
with `.11.4.2`, and the new rule refused the commit by name.

⭐ Four instances shipped; the fifth did not. That is the clearest before/after
this repository has produced for a gate, and it is worth noticing that the
author who wrote the rule is the one it caught — a gate that only catches other
people's mistakes is a gate nobody needed.

⚠️ Held rather than promoted (`.11.20`): it sharpens
`calibrate-over-the-history-that-contains-the-instance`, which is already in the
layer and is the rule this leaf's acceptance was written from. It is recorded in
the gate's own docstring. A second snapshot understating a recurrent shape would
earn it a note.

## 2026-09-20 — An arm that consumes state needs its own fixture

The coverage control for `SIGNOFF-REPAIR.9.3.4.2` has three arms over five
surfaces, and the whole design is one knob: set `authority_grants.actions`,
send the same request, watch the outcome change. Arm 1 sets an unrelated
action and expects a refusal; arm 2 sets the surface's own action and expects
admission; arm 3 sets `["tenant_admin"]` and expects admission by subsumption.

Arm 3's approval leg failed:

```text
400 invalid_command — proposal `cv-prp` is at stage `approved`
                      — a decision rides a `draft` proposal only
```

Not a coverage refusal. Arm 2's approval had advanced the proposal
`decided → approved`, and arm 3 was approving an already-approved proposal.

⭐ What makes this worth a note is which way the failure fell. It failed
**loudly**, because arm 3 expects a 200 and got a 400. Had the arms run the
other way round — the admitting arm first, the refusing arm second — the second
arm would have asserted `status != 200`, got its 400 from the *stage* check,
and **passed**. A control asserting "this is refused" is satisfied by any
refusal, including one that has nothing to do with the thing under test.

Four of the five surfaces are one-shot in the same way: a staged publication is
spent, a target id is unique, a correction and an approval are their own rows.
So each arm now drives fresh ids, with a proposal, a decision and a publication
seeded per arm.

**The rule:** when a control runs the same request under several conditions, and
the request CONSUMES state, give every arm its own fixture. Otherwise the arms
are not independent, and the ones asserting a refusal will absorb whatever
refusal the earlier arms left behind.

⚠️ This is the negative-assertion weakness in a new dress. `assert_ne!(status,
200)` is a weak claim, and it is weakest exactly where the fixture is shared —
which is why arm 1 of this control is the one that had to be checked against a
*named* refusal rather than merely a non-200 in the arms where the message is
available.

⚠️ Held rather than promoted (`.11.20`): it is close to
`a-control-that-passes-for-an-unrelated-reason`, and a second instance should
decide whether the fixture-independence half earns its own note.

## 2026-09-20 — A control over a closed wire vocabulary is written in the wire spelling

`SIGNOFF-REPAIR.9.3.4.1` added five members to `GrantAction`. The obvious way to
write the controls is the one the type system invites:

```rust
let g = grant(vec![GrantAction::PolicyCorrectionRecord], …);
```

That control cannot be observed red. Before the variant exists the test does not
compile, and "it does not compile" is not an observation about behaviour — it is
an observation about the test. Worse, the leg that actually mattered would have
been impossible to write at all: the stored-boundary disposition is a claim
about a row written *before* the vocabulary grew, and expressing "before" needs
a spelling that predates the type.

The wire name does predate it. `migrations/0004_authority.sql` stores
`permitted_actions` as a JSONB array of action **names**, and
`boundary_from_row` parses them with `serde_json::from_value::<Vec<GrantAction>>`.
So the controls are written like this:

```rust
fn action(name: &str) -> GrantAction {
    name.parse::<GrantAction>()
        .unwrap_or_else(|_| panic!("`{name}` is not in the authority registry"))
}
```

They compile against the unrepaired code, and their red is:

```text
`policy_version_register` is not in the authority registry
```

— which is the defect, stated.

⭐ And it unlocked the leg that carries the disposition. The stored-boundary
control deserializes the **exact nine-name array** a pre-change tenant holds,
through the same call the production row-loader makes, and asserts *first* that
the array does not contain the new name:

```rust
assert!(!permitted.contains(&action("policy_correction_record")),
        "the stored row does NOT contain the new name — that is the whole difficulty");
```

Without that assertion the leg could be measuring a boundary somebody had
already migrated, and would pass for the wrong reason.

⚠️ One of the four controls is a **guard**, not a red-first leg, and saying so
matters. `tenant_admin_does_not_subsume_the_thread_actions` passes before the
change — before it there is no subsumption at all — and exists to catch the
repair widening something it should not. Its value is that it must stay green.
Filing it under "controls observed red first" would have been false.

**The rule:** when the thing under repair is a closed wire vocabulary — an enum
whose members are persisted as strings — write the control in the persisted
spelling. The type is the repair; the string is the contract, and the contract
is what a control should be written against.

⚠️ Held rather than promoted (`.11.20`): this is a sharpening of
`a-control-that-passes-for-an-unrelated-reason` and it is written into the
control's own doc comment, where the next author of one will meet it. A second
wire vocabulary extending would earn it a note.

## 2026-09-20 — I generalised from the exception and shipped it

Two surfaces in `api.rs` refuse a malformed body differently. `site_request`
maps axum's `JsonRejection` into the API's `{code, message}` shape with `400`.
Every other typed handler lets the rejection through as a bare `422` with a
plain-text body.

I looked at those two and concluded the `422` was off-contract. Then I wrote a
repair against that conclusion: `owning_authority` became `Option<String>` on
the staging input, and the handler graded its absence by hand — *"so all four
verbs answer one question with one refusal."* It shipped.

The census I should have run first:

```text
46   typed Json<T> extractors in api.rs, all answering 422
 9   site routes normalizing the rejection to 400
 4   test suites asserting the 422 BY NAME
 1   repair (SIGNOFF-REPAIR.4.2.2) that DEPENDS on it
```

And the assertions are not incidental. `command_api` checks the rejection body
`contains("unknown field")` — naming the forged field is the point of it.
`profiles` writes *"An unknown field is the typed 422"* two lines above *"A
malformed digest is the typed 400"*: the two levels, named as two levels, in one
test. `node_channel` calls it "the strict wire boundary".

So the contract has two levels and both are deliberate. `422` means the body is
not this verb's shape and the handler never ran. `400 invalid_command` means the
handler ran and the request is semantically wrong. My repair had taught one
typed verb to answer level 2 for a level 1 fault.

⭐ What makes this worth writing down is that the evidence was not buried. The
exception explains itself, one line above its own code:

> Do not echo malformed caller input or driver diagnostics. Keep body-size and
> media-type refusal statuses; normalize JSON syntax/schema errors to typed 400.

That is an argument about *the site routes*. A convention does not need a reason
written next to it; an exception does, and this one had one. **A surface that
documents why it differs is telling you it is the minority.**

⚠️ And the finding underneath the false premise was real, which is the part that
made the false premise plausible: the three publication transitions took an
untyped body, so their required fields *were* graded by hand and an unknown
field *was* silently ignored. The defect was real, the direction was backwards.
Typing them puts them on the convention instead of moving the convention to
them — and it turned up a fixture that had been sending `git_object_ids` to
`publish`, which does not take it, ignored for the life of that control.

Promoted to
`docs/knowledge/a-convention-is-what-the-corpus-asserts-not-what-one-surface-does.md`.

## 2026-09-20 — A value you can read out of a rendered artefact is not the same value

Publications had to start requiring that the projection they publish carries
the policy the proposal was approved for. The obvious question: where is the
resolved policy set?

It looks like it is already on disk. `render_generic` writes one heading per
clause:

```text
## clause-1 [org-baseline 1.0.0]
```

So `policy_projections.bytes` appears to contain exactly the `(policy_id,
version)` pairs the check needs, and a column looks like a migration nobody has
to run. Two facts kill that.

**The renderer is per target.** `render_lock` renders the LOCK rows, not the
clauses. A `lock` projection's bytes carry a different vocabulary, so one parser
cannot serve the check and the same predicate would answer differently for two
targets of the same resolution.

**And the renderer is lossy, in the direction that matters.** `compile` filters
unrepresentable clauses out *before* rendering:

```rust
let representable: Vec<InputClause> = clauses.iter().filter(|clause| { … }).cloned().collect();
let body = match request.target.as_str() { "generic" => render_generic(&representable), … };
```

A policy that resolved, and whose clauses cannot ride this target, appears in
`unrepresentable` and **nowhere in the body**. Parsing the bytes would say that
policy is not in the projection. The resolution says it is. Both are true of
different questions, and the approval is asking the resolution's.

⭐ The rule: **a value recovered from a rendered artefact answers a question
about the rendering, not about the input.** When a check needs to know what went
*in*, record what went in. The renderer is a lossy, target-specific projection
of it — that is its job.

So `migrations/0082` stores the set, taken from `resolution.resolved` at compile
time, and rows older than the column fail closed rather than being backfilled:
the library has moved, and re-resolving now would record a set the artefact was
not built from.

⭐ The fail-closed arm earned its keep immediately. A sibling control seeds its
projection with direct SQL, so it had no resolved set — and that control went
red on the new refusal in the same run the check shipped. A disposition with a
live witness outside its own test is a much stronger thing than one with an
assertion written to match it.

⚠️ Held, not promoted (`.11.20`): this is close enough to
`a-control-that-passes-for-an-unrelated-reason` that a third instance should
decide whether it is its own note.

The entries before those above were rotated into reachable Git history at the
**first rotation** (`SIGNOFF-REPAIR.11.4.2.6.3`, which owns this ledger’s rotation). The exact predecessor — every
byte this file held immediately before the rotation — is:

```bash
git show acead2ce92c1aef22589b1c86cd956f80a4512cd:DEV_NOTES.md
```

That snapshot is 905695 bytes and 5434 lines, and contains 441 dated
entries; its Git blob is `83b5385e8eea666b9e981e94b388a9dc745b7460` and its SHA-256 is
`3ff21b64e2ed5592f5e549d578f86f0483cdd9d2eef1821f916d464d4b90174a`. It carries NO earlier rotation notice: this is the first rotation of this
ledger, so the chain starts here and every later notice will name this one.

⛔ **430 record(s) rotated out, 12 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
