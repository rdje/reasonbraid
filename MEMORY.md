# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 7 execution: the `PHASE-7` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–7 CLOSED.** Phase 7's exit: G6–G7 NOT MET for the
  Internet exposure, Met as the hardening-machinery exit for the LAN
  profile (the exposure stays UNCLAIMED per the §25.1 kill/pivot —
  the three external preconditions: the reviewed threat model, the
  prompt-injection suite, the pen-test). The gate record + the
  subtraction (S-1…S-12) + the unsupported matrix + the evidence
  manifest ship.
- **Active tree:** `PHASE-8` (the federation/interoperability lane) →
  frontier `.2.4`. **The `.1` lane is COMPLETE**; `.2` done (the A2A
  census) → decomposed; `.2.1` done (ADR-025); `.2.2` done (the pin);
  `.2.3` done (the facade core — the `reasonbraid-a2a` crate over the
  exact 0.3.0 pin + the semantic-loss record).
- **Next action:** execute `PHASE-8.2.4` — the A2A compatibility
  demonstration + the qualification: the a2a-cli (or the SDK's test
  peer) roundtrip against the facade (the measured exchange + the
  recorded semantic losses), the tested spec/conformance revision, the
  qualification record (the broad claims stay gated; the gateway
  deployable-off).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **Defects:** 0 tracked leaves outstanding (`MAINT-1` §13 locality, `MAINT-2`
  drain race, `MAINT-3` toolchain pin — all closed).
- **In-flight uncommitted work:** none.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude`
  (2.1.263, Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** the machine clock crossed midnight mid-session
  (2026-09-07 → 2026-09-08); new records use the machine date — the historical
  anomaly stands (some prior-session records dated 2026-09-07 inside
  2026-09-06 commits).
