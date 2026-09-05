# Evidence — <experiment title>

- **Date:** `YYYY-MM-DD`
- **Leaf:** `<PHASE-X.Y.Z>`
- **Status:** `planned` | `running` | `reported` | `obsolete`
- **Decision owner:** <role or person>
- **Related ADR:** <ADR-NNN or none>

Copy to `docs/evidence/YYYY-MM-DD-<short-kebab-slug>.md`.

## Question

One question this experiment can actually answer.

## Competing options

1. …
2. …

## Fixture

How to reproduce: commands, seeds, hosts, versions. Paths are repo-root-relative.
Store artifacts under a repository-derived directory, never `/tmp`.

## Observable result

What was measured. For any published number, name the three legs
(re-derive / falsify / durable) or name the gap.

```text
command:
output:
```

## Decision

What we will do because of this result. Link the ADR if one is written.

## Deletion plan

When this fixture/report may be archived or deleted, and what must remain
retrievable (git history, sealed evidence, ADR).
