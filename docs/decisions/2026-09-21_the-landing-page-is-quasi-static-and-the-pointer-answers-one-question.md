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

## ⛔ Where the rule lives — this record is provenance, not authority

Each of these files already has a governing document, and this record does **not**
restate their rules. It carries the instruction, its date, and the measurements
that occasioned it; the normative text was written into the documents that own it:

| File | Its governing document | What was added there |
| --- | --- | --- |
| `MEMORY.md` | `MEMORY_ARCHITECTURE.md` §6 | the one-question rule and *it shall not grow*, beside the existing *overwrite, don't append* and *no history* |
| `README.md` | `README_POLICY.md` | nothing — *quasi-static, amended rarely and only for the ramp-up sequence* is the intent its existing caps and routing hint already serve |

⚠️ **This separation is the point, and it was got wrong first.** The initial version
of this record stated *"`MEMORY.md` answers one question and nothing else"* as a
normative rule of its own — a second claimant to authority `MEMORY_ARCHITECTURE.md`
§6 already held. That is the stop condition `SIGNOFF-REPAIR.11.4.2.7` had invoked
one commit earlier to decline the donor's root doctrine, applied by the same author
in the next commit. Corrected at `SIGNOFF-REPAIR.11.4.2.7.2`.

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
file**, sitting on its cap twice. It is now **425 bytes, 6% of the same cap**, with
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

Read the governing document, not this record: `MEMORY_ARCHITECTURE.md` §6 for the
resume pointer, `README_POLICY.md` for the landing page. Both state the test and
the routing. `README-STABILITY` prints the routing list when it refuses, and it
applies before a cap is reached rather than at it.

## Related

- `README_POLICY.md` — the adopted policy this instruction gives intent to.
- `docs/decisions/2026-09-06_readme-policy-readoption.md` — its local adoption.
- `docs/knowledge/an-overwrite-only-file-cannot-accumulate.md` — the diagnostic
  that found the `MEMORY.md` instance.
- `MEMORY_ARCHITECTURE.md` §6 — the layer-A contract, which already prescribed
  eviction for exactly this symptom.
