answers: may I add a line to README.md; what is README.md for; what belongs in MEMORY.md; why is the resume pointer so short; can I put a standing warning in MEMORY.md; is there a no-growth rule on the landing page; where do I put something that does not fit the landing page

# The landing page is quasi-static; the resume pointer answers one question

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Authority:** director instruction, 2026-09-21, given three times in one
  session while reviewing `SIGNOFF-REPAIR.11.4.2.7`'s finding.
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.2.7.1`.
- **Status:** accepted; binding on every session.

## The instruction, in the director's own terms

> *"`README.md` shall just be a startup document that provides information on how
> to ramp up, what to read to ramp up and be up to speed quickly. … `README.md`
> shall be quasi-static, so I am not sure why you keep adding stuff to it, you
> should not!"*

> *"`MEMORY.md` should just be a pointer to the next action, task, slice, lane,
> and overwrite shall happen, not append. … It is used to answer the question:
> 'What's next?', that's it."*

## What this settles

**`README.md` is a ramp-up document and is quasi-static.** It says what the
project is, what to read, and which commands to run. It is not a status page, not
a changelog, not a catalogue. Adding to it is the exception that needs a reason,
not the default.

**`MEMORY.md` answers one question — what is next — and nothing else.** A standing
warning, a lesson, a measurement, a blocker's detail or an environment fact
written there is in the wrong layer, whatever its merit.

## The record, measured rather than characterised

⛔ Two claims about `README.md` circulated in this session's reporting and both
were wrong; they are corrected here with the commands that settle them.

- **It is not growing under agent activity.** `git log --oneline -- README.md`
  returns **12 versions in the project's entire life**, and the most recent is
  `335d6cf` (2026-09-15), six days before this record and outside the session
  that reported it as a pressure case.
- **It has never been refused.** It sits at 2,188 of its 2,400-byte cap and no
  commit has ever been blocked on it.
- **Its two largest growths were status narrative** — `076e06f` (+5 lines) and
  `14d1d16` (+7 lines) wrote phase-completion prose onto the landing page — and
  `b58c646` already replaced both with a one-line pointer to `LIVE_STATUS.md`,
  **shrinking the file by 185 bytes**. That is the routing rule working, and it
  is the pattern to repeat.

`MEMORY.md` was the real instance: **26 standing warnings weighing 81% of the
file**, sitting on its cap twice. It is now 951 bytes, 13% of the same cap, with
zero warnings — `python3 -B scripts/census_memory_warnings.py` reports it.

## ⛔ No no-growth ratchet on the landing page, and the decline is measured

The obvious mechanisation of *quasi-static* is a ratchet refusing any commit that
grows `README.md`. Priced over its whole history before being proposed
(`SIGNOFF-REPAIR.11.6`): of its **11 changes, 6 grew the file** — it would have
fired on **55%**, and several of those are content the landing page should carry
(the prerequisites line, a quick-start command, the licence section). A rule that
refuses the majority of legitimate changes teaches bypass, which is
`SIGNOFF-REPAIR.11.5`'s standing constraint.

What is already mechanical is better aimed: `README-STABILITY` enforces a line cap
**and** a byte cap, refuses date-stamped history on the landing page, and emits a
routing hint naming where each kind of detail belongs instead. The instruction
above is the intent that guard serves; the guard is not replaced by it.

## How to apply it

Before adding a line to `README.md`, ask what the reader is ramping up to do. If
the answer is *know the current status*, it belongs in `LIVE_STATUS.md`; *know
what is being worked on*, `docs/tasks/`; *know why a choice was made*,
`docs/decisions/`; *use a feature*, `docs/book/`. The guard prints this list when
it refuses, and it applies before the cap is reached, not at it.

Before adding a line to `MEMORY.md`, ask whether it answers *what is next*. If it
does not, it belongs in the layer that owns it, and the pointer may name that
layer in a few words.

## Related

- `README_POLICY.md` — the adopted policy this instruction gives intent to.
- `docs/decisions/2026-09-06_readme-policy-readoption.md` — its local adoption.
- `docs/knowledge/an-overwrite-only-file-cannot-accumulate.md` — the diagnostic
  that found the `MEMORY.md` instance.
- `MEMORY_ARCHITECTURE.md` §6 — the layer-A contract, which already prescribed
  eviction for exactly this symptom.
