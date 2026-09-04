# bedrock provenance & the neutral / project-specific boundary

- **Type:** `reference`
- **Date:** `2026-07-24`
- **Status:** `active`
- **Owner / source:** created when the spine was extracted from PGEN

## The fact

bedrock is the **project-neutral, harness-agnostic discipline spine extracted from PGEN** —
a mature Rust parser-generator project at `../pgen` (a
**separate** git repo). PGEN is the reference implementation / proving ground where
doctrines, enforcement patterns, and the memory architecture were invented and hardened
against a real workload; bedrock distills the **general** parts so any Rust project gets
them without re-deriving them. Direction of flow: **PGEN → generalize → bedrock →
`update_scaffold.sh` → other projects.** bedrock does not depend on PGEN and holds no
PGEN-specific content.

## Why

Discipline that lives only in an agent's head evaporates across session loss, model
switches, and harness switches. Putting it **in-repo and enforcing it at the git level**
(hooks + CI) makes it portable and unignorable. The spine earned its keep in PGEN; bedrock
makes it reusable as a starting point rather than a per-project rediscovery.

## How to apply

- **Boundary (standing user instruction):** keep bedrock content OUT of PGEN's git, and
  PGEN-specific content OUT of bedrock. They are deliberately separate.
- **Porting a PGEN improvement:** classify general vs PGEN-specific → neutralize (strip
  every domain noun: grammar/parser/EBNF/SV/regex/corpus/specific gate names) → land it
  under a `BEDROCK-MAINTENANCE` task-tree leaf → if re-syncable, add it to the `NEUTRAL`
  allow-list in `scripts/update_scaffold.sh` → bump `DOCTRINE_VERSION`. The full process +
  the boundary table live in `MAINTAINING.md`.
- The living maintenance work is tracked in `docs/tasks/BEDROCK-MAINTENANCE.md`.
