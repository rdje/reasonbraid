# MAINTAINING bedrock — read this if you are improving the TEMPLATE itself

> **You are here to work on bedrock the template, not to start a new project from it.**
> (Starting a new project? Run `scripts/bootstrap.sh <name>` — it resets this repo's
> maintainer files to a clean consumer seed and removes this guide.)
>
> This document exists because **no memory of building bedrock survives a session**. It is
> the baton-hand-off: read it and you have full context on what bedrock is, where it came
> from, and how to evolve it — with nothing lost.

## What bedrock is

bedrock is a **project-neutral, harness-agnostic discipline spine** for new Rust projects:
durable 4-layer memory, task-tree tracking, a strict commit workflow, mechanical doctrine
enforcement (git hooks + CI), a derived knowledge map, and an mdBook — all wired together
and **self-enforcing on a fresh clone**. A new project copies bedrock, drops its roadmap
into `ROADMAP.md`, runs `bootstrap.sh`, and grows with that spine as its backbone.

## Provenance & relationship to PGEN (the most important context)

- bedrock is the **neutral spine extracted from PGEN** — a mature, real Rust project (a
  parser generator) that developed and battle-tested this discipline over a long campaign.
  PGEN lives at `../pgen` — a **separate** git repo, typically a sibling directory of bedrock.
- **PGEN is the reference implementation / proving ground.** New doctrines, enforcement
  patterns, and memory-architecture refinements are invented and hardened in PGEN first,
  against a real workload. bedrock is where the **general** parts of that are distilled so
  *any* Rust project benefits.
- **Direction of flow:** PGEN → (generalize) → bedrock → (`update_scaffold.sh`) → other
  projects. bedrock does not depend on PGEN and contains no PGEN-specific content.
- **Boundary rule (the user's standing instruction):** keep bedrock content **out of
  PGEN's git**, and keep PGEN-specific content **out of bedrock**. They are deliberately
  separate. Created 2026-07-24.

## The neutral / project-specific boundary

| Belongs in the bedrock spine (general) | Stays PGEN-specific (never ported) |
| --- | --- |
| Memory architecture (`MEMORY_ARCHITECTURE.md`, the 4 layers) | The grammars / parsers / EBNF / SV / regex domain |
| Task-tree workflow + templates | `EBNF-SOURCE-OF-TRUTH`, `REGEX-SELF-HOSTING` checks |
| Commit workflow (`COMMIT.md`) | cert-coverage / ast-shape-contract / syntax-closure gates |
| Doctrine enforcer **driver + universal checks** | Any check that names a grammar/parser/domain artifact |
| `TOOLBOX.md` (tools-first, generalized) | The specific probes/tracers PGEN ships |
| Knowledge map (derived, drift-proof) | The book content about a specific product |
| The git hooks + CI enforcement layers | Release/version/ledger schemes tied to a product |

The litmus test for porting something from PGEN: **would it help a brand-new, unrelated
Rust project?** If yes → generalize (strip every domain noun) and add it to the spine. If
it only makes sense with grammars/parsers/etc. → it stays in PGEN.

## How to transfer a PGEN improvement into bedrock

1. **Spot it.** A structural/doctrine/memory-arch/enforcement improvement lands in PGEN.
2. **Classify it** against the boundary above (general vs PGEN-specific). Only general
   improvements come across.
3. **Neutralize it.** Remove all domain nouns (grammar, parser, EBNF, SV, regex, corpus,
   the specific gate names). What remains should read as if bedrock never knew about PGEN.
4. **Land it in bedrock** under a `BEDROCK-MAINTENANCE` task-tree leaf (bedrock maintains
   *itself* with its own discipline — task-tree first, `COMMIT.md`, the enforcer).
5. **If it is a re-syncable neutral file**, ensure it is in the `NEUTRAL` allow-list in
   `scripts/update_scaffold.sh` so downstream projects can pull it.
6. **Bump `DOCTRINE_VERSION`** and note the change in `CHANGELOG.md`.

Downstream projects then adopt it with `scripts/update_scaffold.sh <bedrock-url>`.

## The neutrality bar — every doctrine here must be objectively applicable to ANY project

> **Maintainer directive, 2026-07-30:** *"The next projects I will start using bedrock as a
> template should inherit the best of the best, the best SOTA, best signoff, the best discipline,
> that we currently have"* — and *"the doctrines in bedrock shall be project neutral, agnostic …
> objectively applicable to any project, not just [the originating one]."*

Two obligations, and they pull against each other on purpose:

1. **Completeness** — a general improvement that lands upstream and is *not* ported is a defect in
   every project started afterwards. bedrock is a seed, not an archive.
2. **Neutrality** — a doctrine only belongs here if it is *objectively applicable to any project*.
   A check that merely had its nouns renamed is not neutral; a check whose LOGIC names a
   domain artifact is domain-bound however it is described.

### The admission test — ask these two, IN THIS ORDER

**Q1 (primary, and it is a question about VALUE):**
> *Does this objectively benefit **any** present and **any** future project?*

Answer it by stating, in one sentence and using **no project's nouns**, what the check prevents —
then asking whether a brand-new project would be better off with it **on day one**. If the honest
answer needs a qualifier — *"any project **that** uses X"*, *"once a project **has** Y"* — then it
is **conditional, not objective**, and it does not belong here as-is.

**Q2 (secondary, and it is only a filter):**
> *Can it be expressed without domain nouns?*

```sh
sed 's/#.*//' scripts/check_<doctrine>.sh | grep -ciE '<domain nouns>'   # must be 0
```

⛔⛔ **Q2 CANNOT SUBSTITUTE FOR Q1, AND THE ORDERING IS THE WHOLE POINT.** A check can score **0**
domain nouns and still encode a workflow only one project needs — *neutral vocabulary, project-shaped
substance*. Q2 measures whether a thing **can** be neutralized; Q1 asks whether it **should** be.
Running Q2 first waves the impostors straight through.

⭐ **Worked example, measured — this is not hypothetical.** A "destructive automation must require
explicit confirmation" check scored well on Q2 and looked like an easy win. Its logic hardcodes a
`Makefile` path and extracts a `clean:` recipe, so what it actually offers is *"benefits any project
**that builds with make and has a clean target**"*. That is a conditional. The **principle** is
universal and worth having; **that implementation is not portable**, and only Q1 catches the
difference. Compare a check that presumes **only what this template itself ships** (task-trees, a
decisions index, a README, a resume pointer) — that one is objectively applicable, because every
consumer has those by construction.

⇒ **The portability seam to look for:** does the check presume anything beyond what bedrock ships?
If yes, either give it a project-declared seam (a config/list the project supplies) or leave it
upstream. Do not hardcode one project's answer and call it neutral.

⚠️ Honest bound on Q2: 0 is *necessary, not sufficient* — the count treats strings and heredocs as
logic. Use it to rank and to catch self-deception, never as the verdict.

⛔ **A doctrine that fails Q1 stays upstream.** Porting it anyway converts a portable standard into
a fork of one project, which is the failure this repo exists to prevent.

## Transfer runs BOTH WAYS

The flow above is the common case, not the only one — and until 2026-07-30 the process had **no
step for the reverse**, so nothing would have surfaced a spine improvement the reference project
lacked.

⛔ **It happened, and it was found by accident.** bedrock's layer-C check already reconciled every
decision record against `INDEX.md`; upstream asserted only that the index had *more than zero
rows*, and passed at **135 records / 133 rows** — two records invisible to their own index with
the doctrine green. The stronger implementation was **downstream**, and the weaker one would have
kept passing indefinitely.

⭐ **This is structural, not luck: generalizing a check is a REWRITE, not a copy.** Stripping
domain assumptions regularly produces a cleaner, stronger check — so the distillation step can
*improve* the thing. Expect it to recur. The same session produced a second instance: the ported
`WAIVER-ROUTING` check had a latent **fail-open** in its origin (`printf … | grep -q … || continue`
returns failure ON SUCCESS past the pipe buffer under `pipefail`, so the file is silently skipped);
it was **fixed on the way in** rather than inherited, and the fix is owed back upstream.

**So, when you port anything:**

1. Ask whether bedrock's existing version of the same invariant is *already stronger*. If it is,
   say so and push it back — do not silently overwrite it with the upstream one.
2. Ask whether the thing you are porting carries a known defect. Fix it here, and record that the
   fix is owed back.

⚠️ **A raw `diff` of the two copies is NOT the trigger — measured and rejected.** Of the 12 files
present in both repos, **11 differ, by 15–559 lines**, because the upstream copies deliberately
carry project-specific evidence while these are deliberately neutral. A check reporting hundreds
of intended differences teaches its authors to waive it. The right trigger compares **behaviour**:
where both repos implement the same invariant, run both against one fixture and compare verdicts.
That harness is **not built** — recorded as owed, not claimed.

## This repo's dual role (why its own memory files look "used")

bedrock is **both** a template *and* a real project (its project = "maintain the spine").
So this repo's own layer-A/B/C memory describes the maintenance work:

- `MEMORY.md` — bedrock's resume pointer (points here + to the maintenance tree).
- `docs/tasks/BEDROCK-MAINTENANCE.md` — the living maintenance task-tree (frontier + backlog).
- `docs/decisions/reference_bedrock_provenance.md` — the durable provenance/boundary facts.
- `ROADMAP.md` — kept as the **consumer** placeholder (the canonical "replace me" file);
  bedrock's own roadmap is the maintenance tree above.

`scripts/bootstrap.sh` de-templates for a consumer: it resets `MEMORY.md` to a clean seed
and removes this guide + the maintenance tree + the provenance record, so a new project
starts fresh.

## File inventory (the spine)

- Bootstrap: `CLAUDE.md`, `AGENTS.md`. Memory: `MEMORY_ARCHITECTURE.md`, `MEMORY.md`.
- Task-trees: `docs/TASK_TREE.md`, `docs/TASK_TREE_README.md`, `docs/tasks/`.
- Decisions: `docs/decisions/` (+ `INDEX.md`). Commit: `COMMIT.md`.
- Enforcement: `DOCTRINE_ENFORCEMENT.md`, `scripts/check_doctrines.sh` (+ universal
  `check_*.sh`), `scripts/check_doctrines.project.sh` (project slot), `.githooks/`,
  `.github/workflows/`.
- Tools-first: `TOOLBOX.md`. Knowledge map: `KNOWLEDGE_MAP.md` (derived), `knowledge-map/`.
- Docs surface: `docs/book/` (mdBook). Live-docs: `CHANGELOG.md`, `DEV_NOTES.md`,
  `LIVE_STATUS.md`. Rust: `Cargo.toml`, `crates/`, `Makefile`, `rust-toolchain.toml`.
- Consumer entry: `ROADMAP.md`. Versioning: `DOCTRINE_VERSION`. Sync: `scripts/update_scaffold.sh`.

## Working on bedrock

Use bedrock's own discipline on bedrock: create/extend a `BEDROCK-MAINTENANCE` leaf before
changing spine files, run `make gate` (the enforcer), commit via `COMMIT.md`. The enforcer
must stay green — bedrock has to practice what it preaches.
