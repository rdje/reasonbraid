# ADR-018 — Resolver sandbox/runtime and network isolation: the isolation classes pin the registry; the resolvers are the future packs

- **Status:** `accepted` (evidence-gated — the §12.2 registry's advertise
  fields (the sandbox level + the egress class) need the CLASS vocabulary
  BEFORE the `.1.3` registry lands, and the resolver packs (`.2`–`.4`) will
  declare against it)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-4.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 018 (resolver sandbox/
  runtime and network isolation); §12.2 (the registry's advertise fields)

## Context

The roadmap queued "resolver sandbox/runtime and network isolation" as
ADR-018. The dev profile has NO resolvers yet (the packs `.2`–`.4` are the
future) — so the ADR cannot choose a sandbox runtime; it must pin the
CLASS vocabulary the registry advertises and the packs declare, so a
pack's isolation claims are comparable from the first pack.

## Options

1. **Pin the isolation classes now; the runtimes ride the packs** — the
   sandbox level (a bounded ladder: `none` < `process` < `seccomp-
   constrained process` < `vm/container`) and the egress class (the allowed
   destinations: `none` / `loopback` / `listed hosts` / `any`) + the rule
   that a resolver's egress claim is the MAXIMUM, never the minimum. The
   actual runtimes (the sandboxed extraction workers, the browser workers)
   land with their packs (`.2`–`.4`).
2. Adopt a sandbox runtime now — machinery with no resolver to sandbox (the
   subtraction doctrine's placeholder-infrastructure lie).

## Evidence

- **The §12.2 advertise fields name the classes** (the sandbox level, the
  egress class, the allowed destinations) — the vocabulary is the roadmap's
  own; only the VALUES need pinning.
- **The explicit-failure doctrine is the G4 exit** — unsupported/mutable
  resources fail explicitly: the classes let a reference FAIL EXPLICITLY
  (no resolver in the required class) instead of silently degrading to a
  weaker sandbox.

## Decision

Accept option 1. The isolation vocabulary: the sandbox level ladder
(`none` < `process` < `constrained_process` < `vm_container`) and the
egress class (`none` / `loopback` / `listed` / `any` — the claim is the
MAXIMUM, and a reference may require a class the registry compares). The
runtimes land with their packs; the `.1.3` registry stores the classes and
the resolution refuses a reference whose required isolation no eligible
resolver declares (the explicit failure, never the silent downgrade).

## Consequences

- The `.1.3` registry's advertise shape carries the two classes from day
  one.
- The `.2`–`.4` packs declare their classes against the ladder — the G4
  hostile-content suite (`.7`) can test the CLAIMS.
- No sandbox runtime, no extraction workers, no browser machinery in the
  `.1` lane.

## Revisit trigger

The first resolver pack (`.2`'s R0) — its runtime + the isolation tests
land with the pack, against this ADR's classes.
