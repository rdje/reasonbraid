# DEV_NOTES.md

## 2026-09-21 — Three corrections from one question, and none of them were the code

The director asked why `MEMORY.md` keeps hitting its cap when it is overwrite-only.
The answer was that 81% of it was standing warnings and only 19% was the pointer, so
the thing hitting the cap was never the thing the contract describes. That is written
up in its own note. What is worth recording separately is that the question produced
three corrections in a row, and two of them were about what I had written rather than
about the repository.

The first was the diagnosis. I had measured a real symptom, classified it as "this
surface lacks an early-warning threshold", and opened a leaf to add one — on a file
whose own architecture document names that exact symptom and prescribes eviction. A
threshold there would have been a warning whose correct response is to delete what
made it fire.

The second was the eviction breaking a gate, which was the useful half. The census's
live-corpus self-test arm asserted that the real `MEMORY.md` yields warnings. That was
written when 26 of them were the normal state, so the arm encoded the defect's
presence as ground truth and failed the instant the file became correct. It is the
same shape as the collation control that passed on this host and failed on the runner:
a control anchored to today rather than to its rule. The fixture arms already control
the segmenter in both directions — breaking `collect_warnings` reddens four of them by
name — so the live arm was never the positive control it looked like. It now checks
only the thing only it can check: that the census can still read the real file.

The third correction was not about code at all. The director asked why I kept
modifying `README.md`. I had not touched it — it is byte-identical to `HEAD`, absent
from all five of this session's commits, and last changed six days earlier by someone
else's work. What I had done was write it into a report, beside `MEMORY.md`, as a
surface that "refuses at the cap with no earlier signal". That sentence is true about
the mechanism and false as a picture: twelve versions in the project's entire life,
never once refused, and its two largest growths already corrected back into a pointer
by the routing rule doing its job.

So I built a finding out of one real instance and one sentence that made it sound like
a pattern, and the second half survived three rounds of my own review because it was
never checked — it was inferred from the fact that both files have caps. A shared
mechanism is not a shared symptom. The check costs one `git log`.

There is a pattern across the three: in each case the measurement existed, was cheap,
and I wrote the conclusion first. The census was already tracked and already reported
81%. The fixture arms were already in the file. The README history was eleven lines of
`git log`. None of these needed new tooling; they needed the sentence to come second.

⛔ promotion: the structural half is promoted as
`docs/knowledge/an-overwrite-only-file-cannot-accumulate.md`. The reporting half is
recorded in the decision that carries the director's instruction, because it is a
correction to this project's record rather than a transferable method — and
`a-restated-number-needs-a-producer` already covers the general form.

## 2026-09-21 — The file that could not grow, and did

The director read my finding and asked one question: `MEMORY.md` is overwrite-only,
so why is it hitting its limit so often — that makes no sense.

It does not make sense. That is the whole content of the question, and it is the part
I had skipped. I had measured that the cap was being hit, classified it as "this
surface lacks an early-warning threshold", and opened a leaf to add one. Every step
after the first was reasonable. The first step was to accept a contradiction and
build on top of it.

The census I already had answers it in one line: 26 standing warnings, 5,184 of 6,412
bytes, 81% of the file. The resume pointer — the five fields the architecture names —
was 19%. So the file was not growing as a pointer. It was growing as a warnings board
that happened to live in the same file as a pointer, and the cap was being hit by the
board.

`MEMORY_ARCHITECTURE.md` §6 names this symptom in its own words — "the Current state
block accumulating, under a heading that says to overwrite it" — and prescribes
eviction, not a threshold. I had read that section. I quoted the neighbouring
paragraph about line caps and byte caps in two different commits this session. I did
not apply it to the file it is about.

And it is the same defect this lane repaired four commits ago. `LIVE_STATUS.md` was a
status snapshot that had become a correction log; splitting the two roles took it from
620,448 bytes to 13,995. `MEMORY.md` is a pointer that had become a warnings board.
Identical shape, one lane apart, and I turned the lens on one file and not the other.

The health target I proposed would have been worse than doing nothing. Its correct
response, every time it fired, would have been "delete the thing that made me fire" —
a warning that makes the bloat survivable is a warning that makes it permanent. The
test I now think is right: name the remedy before setting the threshold. If the remedy
is a command, a threshold fits. If the remedy is a judgement about where content
belongs, the content belongs elsewhere and the threshold is a distraction.

The eviction itself was the easy half, and the only part that needed care was proving
nothing was lost. The census refuses to let that be assumed — its own output says an
uncited warning is a population to classify rather than a count of things safe to
delete, because one of them once turned out to be an environment fact that existed in
this file and nowhere else. Seventeen were instrument-anchored. I hand-classified the
other nine against the durable layers; all nine were recorded. 6,412 bytes to 1,682.

I also withdrew half my own finding. I had paired `README.md` with `MEMORY.md` as two
surfaces under cap pressure. `README.md` has twelve versions in the project's entire
life and has never once been refused. Its two largest growths were project status
narrative, and a later commit had already replaced those with a one-line pointer and
shrunk the file — the routing rule doing exactly its job. There was no second pressure
case. There was one, and a sentence I had written to make it sound like a pattern.

## 2026-09-21 — Three drafts of one sentence, each refuted by its own command

The donor-package review was supposed to be the easy slice: audit twelve deliverables,
decide, record. The audit itself went cleanly — ten met, one met by a different local
mechanism, one genuinely missing with a real seven-instance cost behind it.

Then I wrote the lockstep box, and wrote it three times.

Draft one: "No docs/book/ change — `grep -rn "containment" docs/book/src/` returns
nothing." It returns six. I had written the claim and appended a plausible command to
it, which is precisely backwards. Reading the six showed they are a different sense of
the word — four are policy containment, publication scope, and two are process
containment, worker sandboxing — so the substance of the claim survived. The evidence
did not.

Draft two: narrow the search. "`grep -rn "live.document\|LIVE_DOCUMENT" docs/book/src/`
returns 0." It returns one. And the one is the row I added to the book four commits
earlier, in this same lane.

So the book already spoke to the subject, the change was owed, and two drafts of a
box asserting otherwise had each been written before the command ran. Draft three
extends the row: the contract is met against the external standard it was adopted
from, and one requirement of that standard is still unmet and owned.

What makes this worth a note rather than an embarrassed edit is that it is the exact
defect the lane exists for. `.11.4.2.6.7` shipped a gate because a registry row stated
a control and nothing evaluated it. A lockstep box is a registry row: a claim about
this commit, in prose, with a checker (`LOCKSTEP-CLAIM`) that validates its SHAPE —
whether it names a core live document the commit does not stage — and not its truth.
`grep -rn ... returns nothing` is a declared control, and nothing read it.

I do not think that gap should be closed by a gate. A box's evidence is arbitrary shell
in prose, and a checker that tried to execute it would be running whatever an author
typed, which is the trap `check_doctrines.sh` refuses at the top of its own registry.
The discipline is the cheap one: run the command, then write the sentence from what came
back. Both failures here are the same inversion, one commit apart, in the same box.

⛔ promotion: declined. `a-restated-number-needs-a-producer` already covers a figure
written where nothing derives it, and this is that rule applied to a search rather than
a count. Recorded in the leaf and in this note.

## 2026-09-21 — The falsification that refused for the wrong reason

The scheme is an optional fifth field on each routed-destination row: space-separated
`kind=operand` terms sitting beside the human sentence, evaluated every commit. Fourteen
of the twenty rows declare something; six declare nothing, and that is a judgement
recorded rather than a gap.

The interesting part was not the design. It was the falsification.

I wanted to show every evaluator can refuse, so I did the obvious thing: patch a
declared operand in the registry to a false value, run the check, confirm it refuses,
restore. Seven kinds, seven refusals, all naming the right row, registry restored
byte-identically. It looked complete.

It was not. Every one of the seven refused through the DRIFT leg.

The scheme has a double-entry property: the instrument carries a hand reading of each
sentence, the row carries a declaration, and the check refuses when they disagree in
either direction. That is there so the field cannot be quietly emptied, and so a term
cannot be invented that the sentence never made. But it means that patching an operand
in the registry ALWAYS produces a disagreement — so the refusal I was reading as
"the ceiling evaluator works" was really "the declaration no longer matches the prose",
which would have printed identically if every evaluator returned True unconditionally.

Seven controls, one reason, and not the reason any of them was written for. This is
`a-control-that-passes-for-an-unrelated-reason`, and I produced a clean example of it
in the pass whose entire purpose was rigour — the same shape `.11.4.2.6.4` found when
three positive controls all failed in the same direction as their subject.

The isolation is to call each evaluator directly with its declared operand and with a
wrong one, which cannot involve the drift leg at all. Seven of seven return True then
False, each naming its own reason.

The other falsification needed no edit at all, and I prefer it for that. The original
defect was a growth claim — a row saying a file was overwritten rather than appended
while its tip sat at its all-time high — so the check takes `--as-of`, and asking it as
of the commit before that file was split refuses it by name with the original numbers.
What makes it evidence rather than theatre is that the two sibling ledgers declare the
IDENTICAL claim and both hold at that same commit. A falsification where everything
fails at an old date proves nothing about which thing is broken.

One more thing I would have got wrong by reflex. `docs/TASK_TREE.md` declared the
doctrine `TABLE-ARITY` and the enforcer registers `TABLE-ARITY-RATCHET`. The cheap fix
is a prefix match — one of those is a prefix of the other, after all. That is the move
`.11.27` refused for a different instrument: it makes today's answer right and hides
the next one. The sentence is corrected instead, and the refusal names the closest
registered id so an author is helped without the check being weakened.

⛔ promotion: declined. The transferable method is
`an-adjudication-is-keyed-to-the-words-it-judged`, promoted one commit ago and applied
here; double-entry is its mechanical form and is recorded in the source and in the
doctrine reference rather than restated as a second note.

## 2026-09-21 — The identity check that could not see the thing it was checking

The next leaf has to falsify a scheme against a growth claim, so I went to price a
growth assertion before designing one. `measure_history` spawns one `git cat-file -s`
per version. The three ledgers a growth assertion must cover hold 1,756 versions and
cost 23.24 s. The whole doctrine enforcer costs 28.74 s. That is not a leg you add to
a pre-commit gate; `.11.5`'s whole point is that a gate people route around is a gate
that lies, and the surest way to earn that is to double its cost.

One `git cat-file --batch-check` process per path instead of one per version: 0.11 s.
Same numbers. I put it inside the shared function rather than beside it, because a
fast twin is two producers of one number, and this repository has repaired that shape
more times than any other.

Then the part worth writing down.

I proved output-identity the way `.11.4.2.6.6.2` prescribes: extract HEAD's own script
with `git show` and run it against the SAME tree, so a code change is not conflated
with a tree that moved underneath. 24 paths, 4,579 versions, every field of every
record compared rather than just the size. Zero disagreements. That is a strong result
and I nearly stopped there.

What stopped me was asking what the change actually alters. It alters one thing: how a
revision whose object does not resolve is handled. The old walk tested `returncode == 0`
and skipped; the new one drops a `missing` line. Everything else is arithmetic that did
not move. So the only semantic at risk is the skip — and the question is whether 4,579
versions exercise it.

They do not. `git log -- path` reports a commit that DELETED the path, and `rev:path`
does not resolve there, so the skip only fires for a path that has been deleted. No
routed path has ever been deleted. All twenty exercise it exactly zero times. The
comparison I was about to publish as proof was green for a reason unrelated to the
thing it was proving — `a-control-that-passes-for-an-unrelated-reason`, produced by me,
in the pass whose entire purpose was to be rigorous about it.

There are three such paths in this repository's history. Each has 2 revisions and 1
resolvable object; old and new agree on all three and each drops exactly one. That is
the measurement that carries the claim. The 4,579 carries a different and weaker one —
that nothing else broke — and both are now stated as what they are.

I also checked that the comparison can report a difference at all, which is the same
question one layer down. A reader that counts a miss as 0 instead of dropping it
returns `[12, 0, 7]` where the shipped one returns `[12, 7]`. It can.

⛔ promotion: declined. The method — price a gate leg against the gate it runs inside —
is `.11.5`'s standing constraint and already durable, and the observation above is an
instance of an existing note rather than a new one. Recorded in the leaf.

## 2026-09-21 — Counting what twenty sentences actually claim

`.11.4.2.6.7` is the leaf that will make a declared pressure control checkable,
and it opened with a constraint on itself: census the population first, because
*a scheme that fits 3 of 20 is a scheme for 3 rows*. This is that census.

The registry is `.doctrine/readme_routes.txt`, twenty rows of
`path|class|pressure control|owner`, and the third field is a sentence a human
wrote about what holds that destination's growth. The guard that reads the file
validates four non-empty fields. Arity, not truth — which is how `LIVE_STATUS.md`
declared *overwritten rather than appended* for months while 614 of its 632
versions grew and the tip sat at its all-time high.

The first thing I did was the wrong thing, on purpose, and it is worth keeping.

I wrote the obvious extractor: pull doctrine-shaped tokens and byte-shaped numbers
straight out of the prose. Eighteen operands came back. Five of them are not
claims at all. `PHASE-0` and `PHASE-1` are task-tree names in a sentence about
what `KICKOFF.md` is; `620448`, `908850` and `48495` are a predecessor size, a
pre-rotation size and a recorded baseline — figures a row cites as HISTORY,
sitting in the same sentence as the ceiling it must not be confused with. The
extractor cannot tell a cited number from a governing one, because nothing in the
text marks the difference.

And it is wrong in the other direction at the same time. Six of the nineteen real
claims belong to kinds no such regex can see: an index-coverage claim, a build
target, a guard's refuse-list, a content-identity claim. The naive reading does
not merely under-perform; it is answering a different question and returning a
number for this one.

So the operand has to be DECLARED by the row, not parsed out of it. That is the
census's actual output, and it was measured rather than argued.

The count itself refuted the worry the parent leaf opened with. Fourteen of the
twenty rows carry at least one machine-expressible claim; six are genuinely
narrative. Nineteen claims across six kinds, and I evaluated eighteen of them in
the same run rather than calling them evaluable — seventeen hold and one is
refuted.

The refuted one is the same defect one row over. `docs/TASK_TREE.md` says
*TABLE-ARITY and TASK-TREE-OWNERSHIP doctrines enforce its shape*. There is no
doctrine called `TABLE-ARITY`. The enforcer registers `TABLE-ARITY-RATCHET`, and
that ratchet does run over staged `*.md`, so the file IS governed — the row names
its governor wrongly, and nothing in the repository has ever compared the two.
I want to be precise about that rather than dramatic: this is a name defect, not
an absent control, and a census that reported it as "unenforced" would be making
the second mistake while reporting the first.

One claim is expressible and unanchored, and I gave it its own outcome instead of
filing it under narrative. `KICKOFF.md` declares *content identity once the
PHASE-0 tree closes*. That is an identity claim — pin a revision and it is one
`git cat-file` — but the moment it starts from is prose. It has already been
broken: `PHASE-0` first read `done` on 2026-09-06 and the file changed on
2026-09-09, by a legitimate correction that kept the repository public. The
correction was right. The row had no way to notice it happened.

The part I will reuse is the shape of the instrument, not its numbers.

Deciding whether a sentence makes a machine-evaluable claim is a judgement over
prose. A matcher that tried would be guessing at paraphrase, which is the failure
`.11.6` is about, so the classification is hand-written and carried as data. What
makes that acceptable is that everything around it is mechanical: the adjudication
must cover the registry exactly in both directions, so a new row refuses the census
rather than going unclassified; every adjudicated clause must still be a verbatim
substring of its row's control, so rewriting the sentence detaches the judgement
loudly; and a kind with no evaluator is refused.

That third rule — the verbatim quote in the key — is the one I promoted. This is
its second instance and the first one shipped: `census_advertised_policies.py` was
keyed by pack and field alone, and flipping a resolver pack's `subresource_policy`
from `deny` to `allow` left it green, still reporting `enforced` for a line that
now advertised the opposite. Both instruments record the method in their own
source and neither is reachable from the other, which is when the note is owed.

⛔ No scheme is proposed here, no gate registered, no registry row touched. The
count is the input the next leaf is forbidden to design without.

## 2026-09-21 — The notice that described a procedure I did not follow

`LIVE_STATUS.md` is bounded. 620,448 bytes to 13,995, the snapshot it is named for
at the top, the correction log sealed into git behind a chain notice. That closes
the lane: all five core live documents now refuse oversize input, each refusal
naming its own file, and the corpus went from 1,559,250 bytes to 157,217.

The thing worth writing down happened in the last ten minutes of it.

I generated the chain notice with `render_footer`, the same function the changelog
rotation has used thirty-seven times. It produced a well-formed notice with the
right predecessor commit, the right blob, the right SHA-256. It also said this:

> That snapshot is 620448 bytes and 3184 lines, and contains **0 dated entries**
> […] ⛔ **0 record(s) rotated out, 0 kept, lossless** — every retired heading was
> retrieved from the predecessor named above before this notice was written […]
> ⭐ The cut is DERIVED, not chosen: it retires whole records until the ledger has
> at least 10 commits of runway at the p90 entry size.

Three claims, all false here, all in confident language.

There were no dated entries because this file never used that heading form — so
"0" reads as "nothing was retired" when in fact 600 KB was. No retired heading was
retrieved from the predecessor, because there were no headings; the retrieval
check that sentence describes never ran. And the cut was not derived by a runway
rule at all: it was a wholesale seal, chosen precisely because no record boundary
existed to retire records at.

None of this is a bug in `render_footer`. It is a renderer doing exactly what it
was written to do, applied to a case it was not written for. **A renderer written
for one procedure will describe a different procedure in that procedure's words**,
and the output is not merely wrong, it is wrong in the register of something that
was verified. Every one of those sentences exists because an earlier leaf wanted
the notice to be checkable. Reused out of shape, they are the opposite.

What makes this the worst kind of defect is the audience. A chain notice is read
by someone trying to retrieve history they cannot see. They cannot check it
against the thing it describes, because the thing it describes is what they are
trying to reach. The notice is load-bearing precisely where verification is
hardest.

So the notice is written for a seal. It says it is a seal, says why rotation was
unavailable — 320 emoji-led candidate lines and no rule that separates a headline
from a continuation — and keeps only the claims this transition actually earns:
the predecessor's identity, re-derived from the object itself with plain git, and
the fact that nothing was rewritten or summarised.

Two checks I would not skip again. First, the chain still has to parse: I ran the
tool's own `split_ledger` and `NOTICE` over the new file and confirmed it finds one
record, finds the footer, reads the ordinal `first`, and would number the next
rotation `second`. A notice a human finds convincing and the tool cannot read is a
chain that has quietly ended. Second, before claiming no book change I checked
what the book actually says, and it says this file "carries the same snapshot at
the repository root". Splitting it makes that sentence true for the first time —
the file used to contain a snapshot at 1.58% of its bytes.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.6`. The statement is
a further instance of `docs/knowledge/a-fields-name-is-not-its-contract.md` one
level up — there a field's name, here a renderer's sentences. If a second renderer
is reused out of shape it earns its own note; the count is one.

## 2026-09-21 — The derivation whose anchor moved

Two commits ago I gave `DEV_NOTES.md` a rotation threshold and was pleased with
how it was obtained. Not chosen — derived, from an authority that already existed:
`CHANGELOG.md` has a reviewed threshold of 96,000 bytes and a measured p90 entry
of 4,734 bytes per non-rotation commit, so it runs a live window of 20.279
entries. The second ledger gets the same window in its own units. No taste
involved.

Today I went to do the same for the third ledger and measured `CHANGELOG.md`'s
p90 again. It is 4,146.

The window is now 23.155 entries.

Nothing changed about the method. `CHANGELOG.md`'s recent entries have simply been
a bit smaller, so the same threshold buys more of them, so the window computed
from it is wider. Apply the formula today and `LIVE_STATUS.md` gets a ceiling 14%
more generous than the one `DEV_NOTES.md` received on Tuesday — not because
anything about the two files justifies a difference, but because of which day the
division was performed.

That is the failure mode, and it is subtler than the one I was guarding against. I
had already written, in the `DEV_NOTES.md` threshold comment, that the number must
be pinned rather than re-derived on read, because a ceiling recomputed at run time
lets a file widen its own bound by growing. That was right. What I did not notice
is that the same argument applies one level up: **the WINDOW is derived too, and
nothing pinned it.** I pinned the output and left the method's input floating.

So the window is now a named constant in the source, `LIVE_WINDOW = 20.279`, with
its inputs and the commit it was taken at written beside it, and every ledger's
threshold is that one window in its own measured units. `DEV_NOTES.md`'s 76,000 is
untouched, which is the check passing rather than an omission — re-deriving it
today is exactly the mistake.

Two controls hold it now. A threshold may not exceed the pinned window times the
p90 recorded at its own derivation, so a ceiling cannot be quietly raised above
what its derivation supports; and the window is pinned against `96000 / 4734`, so
the constant cannot drift silently either.

The second thing I want to keep is a method correction, and it is the more useful
of the two.

Two commits ago I proved a code change moved no behaviour by capturing `--plan`
before the edit and diffing it after. That looked rigorous and it has a hole: it
assumes the only thing that changed between the two runs is the code. Today, with
a larger gap between runs, the tree moved underneath — `CHANGELOG.md`'s p90 fell
and its plan went from retiring 3 records to 6. A before/after diff would have
shown a difference and I would have gone looking for it in my own edit.

The clean version is to hold the tree still and vary only the code: extract
HEAD's own script with `git show HEAD:scripts/rotate_changelog.py`, run it and the
modified one against the same working tree, and diff those. Byte-identical for
both enforced ledgers. That separates the two variables instead of hoping only one
of them moved.

It costs one extra command and it is strictly better, and I only reached for it
because the earlier method had visibly stopped working.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.6.2`. The statement —
*a derivation whose anchor moves is not a derivation* — is a disposition about
this repository's ledger thresholds rather than a transferable method, and it is
written beside the constant where the next person to touch it will meet it. If a
second derived constant here turns out to have drifted since it was set, it earns
a note; the count is one.

## 2026-09-21 — The question that decided the migration was whether a boundary exists

I was about to create a record boundary for `LIVE_STATUS.md`'s correction log so
the rotation could split it. The obvious approach: the entries are emoji-led
paragraphs, so turn each one into a dated `## ` heading and let the shipped
mechanism handle it exactly as it handles the other two ledgers.

Before writing that, I asked whether the entries can be identified at all.

320 lines in the file start with an emoji at column 0 and bold text. 265 of them
carry a leaf id or a work-unit id, which is what a headline looks like here. 55
do not. So the question is whether those 55 are headlines that omitted their id,
or second paragraphs of the entry above them.

I tested the discriminator I expected to work: what precedes the line. A headline
should follow the previous entry's last bullet; a continuation should follow other
prose. Of the 55 id-less candidates, 46 directly follow another headline, 7 follow
prose, 2 follow a bullet.

Then I ran the same classification over the 265 I was confident about, as the
control. 95 of them also directly follow another headline. 169 follow a bullet.

**The two distributions have the same shape.** Preceding context cannot tell a
second paragraph of one entry from the first paragraph of the next, because both
happen, in both classes, at similar rates. There is no mechanical boundary in this
file to find.

I want to be clear about why running the control mattered. Looking at the 55 alone,
"46 of 55 follow another headline" reads like a finding — it sounds like those 46
are continuations. It is only when the same measurement over the known headlines
returns 95 that the number stops meaning anything. A classification that produces
the same answer for both classes is not a classification. That is the third time
this week a number only became interpretable next to its control.

What this changed is the migration, and for the better.

A boundary could still be created by hand-classifying all 320. This project does
that — it has hand-classified seven, seventeen and forty-five instance
populations, and they were the right call each time. But every one of those was
classifying evidence that was going to be USED. Here the work would go into
retrofitting structure onto content that is entirely historical, so that a
rotation mechanism could later retire it — when the content is already, by
definition, all retirable.

So: seal the whole log into git history in one transition, and create the boundary
only for entries written after the split, where it is enforced from the first entry
instead of inferred from 320 old ones. Sealing preserves every byte exactly and
retrievably, which is what rotation is for, and it deletes the single step in the
migration where a record could be silently merged or lost.

The shipped rotation is not the wrong tool. It is the tool for this file's future
rather than its past, and the leaf says so explicitly so nobody re-opens it.

One correction to my own numbers, and it is the familiar one. I wrote "320
correction entries" and used it as an entry count. It is the count of emoji-led
lines, which is exactly the conflation the leaf is about — and the file's own
earlier census reported 318 the same way. Neither number is an entry count. The
leaf publishes none, which is the honest position when the whole finding is that
no rule produces one. Third time this session that a first population described my
parser rather than the corpus.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.6.1` — a third
instance of `docs/knowledge/an-instruments-first-population-describes-its-parser.md`,
and the sealing choice is a disposition about this file rather than a transferable
method.

## 2026-09-21 — The rule whose only firing would be the report of the defect

I set out to extend a gate and ended up declining one, which is the right outcome
and took a measurement to reach.

Yesterday's finding was a stale line-number citation into a prepend-only file.
The obvious follow-up is a rule: do not add positional references into files that
get written from the top, because the next entry breaks them. This repository
already has the shape for it — a ratchet, which refuses a RISE against HEAD
rather than demanding a clean corpus, and which `RELATIVE-LEAF-REF` uses.

Before registering anything the ratchet has to be priced, so I counted how many
of the last thirty commits touching tracked Markdown would have raised the count.

One. Commit `87d2b6f`. That is the commit I made an hour earlier, whose entire
subject is the stale citation, and which added eleven such references because it
quotes the broken pointer in order to name it as broken.

**A count of mentions cannot separate a use from a mention.** The rule would have
fired exactly once in thirty commits, against the report of the defect it exists
to prevent. That is not a threshold problem to be tuned; it is the rule being
unable to see the difference between someone doing the bad thing and someone
writing it down.

This repository already has that scar. `check_self_tests.sh` records in its own
header that registering it put its flag's literal text into the doctrine
registry's description, so discovery found the enforcer and ran it, recursively.
A control that searches for a string it contains. Same family: an instrument that
counts occurrences of a pattern will count its own discussion of the pattern.

So the instrument ships with no `--check` arm, and its docstring says why, so that
the gate cannot be added later by someone who thinks it was an oversight.

The second thing the census found was better than the rule would have been. Of
the 131 Markdown-target positional references, only 9 sit in a live document or
the book. Three are my deliberate mentions. The other six are all inside
`LIVE_STATUS.md`'s correction log — which is historical content living inside a
live file, which is exactly the two-roles defect I measured yesterday. **They do
not need a rule. They need the file split**, after which they are history, where
a positional reference is a record of what someone read and is correctly frozen.
The remaining 122 are in closed leaves and acceptance checklists, and editing
those would rewrite the record of what was done.

So: nothing in the live corpus requires repair. That is a measured result, not an
absence of effort, and it is worth saying plainly because "I found 131 of
something" reads like a backlog when it is a census.

Three instrument failures on the way, and I am recording all of them because each
one produced a confident number.

The prepend-only test was wrong twice, and both wrong answers were **zero**. The
first asked whether the old version is a suffix of the new one. That returns 0 of
24 for `DEV_NOTES.md`, a file which is 100% prepend-only, because the file starts
with `# DEV_NOTES.md` and a prepended entry lands *after* the header. The second
stripped the title, which fixed two files and still returned 0% for
`LIVE_STATUS.md`, whose insertion point is below a preamble *and* a section
heading. Both times the instrument was confidently describing its own reach. The
test that works asks the question without assuming where the header ends: take
the longest common suffix in lines and require it to cover the old version.

Then the working derivation corrected my own candidate list — it found a fifth
prepend-only document I had not thought to check,
`docs/tasks/artifacts/signoff_review/INDEX.md`, at 100%. That is the argument for
deriving a registry instead of writing one.

And adding that row immediately exposed a bug in the instrument I had written
twenty minutes earlier: the prepend lookup keyed on the **basename**, so the
moment a second `INDEX.md` existed in the registry, `docs/adr/INDEX.md` — a
different file, not prepend-only — would have been reported as broken by
construction. A key too loose returns the wrong instance, and mine was too loose
for exactly as long as the registry had no collisions in it.

Last one, and it is a repeat. My first control for the prepend test used a
four-line fixture and failed — not because the code was wrong, but because a rule
expressed as "95% of the older version's lines" cannot be exercised by a fixture
where a two-line header is half the file. Two days ago the `LESSON-PROMOTION`
race arm needed 400 lines for the same kind of reason. **A fixture has to be
scaled to the rule, not merely realistic.** If a third arm needs this, it stops
being an instance of an existing note and becomes its own.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.5.1` — every
statement is a further instance of an already-promoted note
(`an-instruments-zero-describes-its-reach` twice,
`a-key-too-loose-returns-the-wrong-instance`, and
`a-self-test-cannot-be-tidier-than-the-real-input` for the second time in two
commits). The last is the count to watch.

## 2026-09-21 — A pointer that was wrong the day it was written

`LIVE_STATUS.md:1719`.

That citation sits in a decision record from two days ago, attached to the text
*"Historical G5 subtraction gate withdrew the quality-lift claim"*. I went looking
at line 1719 because I am about to restructure that file and wanted to know which
consumers a restructuring would break.

Line 1719 is about git-lfs legacy version URLs. The cited text is at line 2,647.

The obvious story is drift: `LIVE_STATUS.md` is prepend-only, 928 lines of
correction entries have gone in above that row since the record was written, and
every line number below them moved by 928. That story is true and it is not the
interesting part.

The interesting part is what happened when I checked the reference at its own
commit. There is exactly one commit that touches that decision record, `b204c87`.
At `b204c87`, line 1719 of `LIVE_STATUS.md` is `| --- | --- | --- |` — a Markdown
table separator — and the cited text is at line 1728.

**The pointer was wrong by nine lines the day it was written.** It then became
wrong by 928. Two independent failures in one citation, and the first one had
nothing to do with drift at all: someone counted to the top of the table instead
of to the row.

There is a detail here I keep turning over. The title of the record containing
that citation ends: *"and G4's single test citation no longer resolves"*. It is a
record whose subject is a citation that stopped resolving. It contains a citation
that never did.

I am not repairing it by editing it. `docs/decisions/` supersedes rather than
mutates, and that record's own header says so in terms — it goes out of its way
to state that the two gate records it corrects are byte-unchanged. Editing it to
fix a line number would break a stronger contract than the one it violated. So
the correction lives in a new record, which is what this directory is for, and
the broken citation stays exactly where it is with a record pointing at it.

What the instance earns is a rule, and the rule is narrower and harder than "keep
citations up to date":

> **Do not cite a line number into a prepend-only file.** Not "check it
> occasionally" — do not write it. Every entry added at the top invalidates every
> reference below it, so such a citation is broken by construction. The next
> commit breaks it. There is no maintenance regime that fixes this, only a
> different way of pointing: cite the row, the heading, the content.

And the part that should have caught it did not, for a reason worth writing down.
This repository has a `POSITIONAL-REF` doctrine and a census behind it. That
census reports 597 positional references across 29 files with `unresolved=0`. It
was green over this the entire time, because its population is positional
references to **source** files — a `.rs` path followed by a line number. A positional reference whose
target is a tracked Markdown file is not in its population at all.

So the gate was not wrong. It was answering a different question than the one I
assumed it answered, and `unresolved=0` was a true statement about a set that did
not contain the instance. That is the same shape as
`a-census-is-as-wide-as-its-key`, which I cited two days ago about a different
census, in this same lane. Twice in one week, and both times the number was
correct and the population was not what I thought.

Promotion: declined and recorded in `SIGNOFF-REPAIR.11.4.2.6.5`. The prepend-only
rule is a disposition about this repository's own files, so it is a decision
record, carried where a reader of the broken citation will meet it. If a second
prepend-only file ever acquires positional consumers it becomes a knowledge note;
the count is one.

The entries before those above were rotated into reachable Git history at the
**second rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 918f446079a0c8e616fd3b4074d2c6bf46b50f36:DEV_NOTES.md
```

That snapshot is 71256 bytes and 1244 lines, and contains 19 dated
entries; its Git blob is `1e8af2e5e248bbe84a685d7863be132bfba60aae` and its SHA-256 is
`5c5e97b6ee788bb7d6a58a15fc3f4fbfcfae328c3544213af1b130ea73b5639e`. It carries the first rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **13 record(s) rotated out, 7 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
