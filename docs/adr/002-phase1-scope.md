# ADR-002 — Phase 1 scope: GO on the LAN vertical slice, single-agent-default routing

- **Status:** `proposed` (awaiting the accountable owner's signature)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-0.8.1`
- **Requirements:** `ROADMAP.md` §20.3, §13.8, §2.3 (H1/H6); KICKOFF WP8; Phase 0 exit gate G0

## Context

Phase 0 is complete: WP1–WP7 delivered the contracts, the atomic-transaction
and outbox proofs, the node journal and channel, the authority and budget
engines, the control API + CLI, the two-host crash/reconnect demonstration
(all acceptance checks), and the deliberation benchmark. The gate now requires
the explicit go/rework/pivot/stop decision and the Phase 1 scope.

The measured facts that bear on the decision:

- The platform mechanics survived real kill points (server SIGKILL, node
  SIGKILL after dispatch, duplicate transport, budget exhaustion) — the G0
  boundaries have executable evidence (manifest: `docs/evidence/2026-09-07_phase0-evidence-manifest.md`).
- The WP7 real run found **no quality gain from structured workflows** on the
  4 differential cases, at 2–4× the tokens (`docs/evidence/2026-09-07_benchmark-codex-run.md`).
- One real adapter (Codex CLI) is qualified; the second is not yet.
- Per-call latency 7–42 s and ~16k ambient input tokens per real call set the
  throughput floor for Phase 1 estimates.

## Options

1. **GO** — proceed to the Phase 1 LAN vertical slice (§20.3), with the
   routing default single-agent (structure only behind case-class
   justification) and the dev-profile dependencies replaced before any
   non-loopback exposure.
2. **Rework** — redo part of Phase 0 (nothing in the evidence asks for this;
   every Phase 0 acceptance point passed).
3. **Pivot** — change the product shape (not supported by the Phase 0
   charter; the architecture is the hypothesis).
4. **Stop** — the benchmark's null result is a routing lesson, not a reason
   to stop: the delivery/governance mechanics are independently proven.

## Evidence

- G0 map: `docs/evidence/2026-09-07_phase0-evidence-manifest.md` (identity,
  authority, thread, delivery, budget — each with suites and commands).
- Benchmark: `docs/evidence/2026-09-07_benchmark-codex-run.md` (re-derive:
  `RB_LIVE_CODEX=1 rb-bench --agent codex --case … --max-calls 36`; falsify:
  the harness self-test reproduces the corpus oracle; durability: corpus +
  prompt digests ride every report).
- Phase 0 effort: ≈ 9.5 engineer-weeks measured against the 8–14 range —
  within estimate; the 2×-estimate review is not triggered (recorded in the
  SubtractionRecord).

## Choice

**GO** (recommended): Phase 1 builds the §20.3 LAN vertical slice on the
proven mechanics, defaults routing to a single agent (the WP7 result), adds
the second real adapter, and replaces the dev-profile trust boundaries before
any exposure beyond the loopback profile. **v0.5.0 of the roadmap remains
forbidden** until the Phase 0 evidence package and a working Phase 1 slice
exist (unchanged).

## Consequences

- Easier: Phase 1 starts from working contracts and a proven pipeline; the
  benchmark harness re-runs as the Phase 1 regression gate.
- Harder: two real adapters must be maintained; the dev-profile headers and
  single-writer assumptions must be retired before exposure (no exception).
- Forbidden: claiming structured-deliberation superiority without new
  evidence; Internet exposure without G6–G7.
- Rollback cost: low at this point — Phase 1 reuses, does not replace, the
  Phase 0 substrate; a rework decision mid-Phase-1 would discard at most one
  work package.

## Rollback / revisit trigger

Reopen if (a) the second real adapter cannot be qualified, (b) a Phase 1
benchmark cohort (larger sample, repeated shots) shows structure LOSING on
every class AND the LAN slice cannot demonstrate honest inconclusive outcomes,
or (c) the name/license clearance fails — in which case the scope record
narrows to internal-only operation.

Signatures: Richard DJE (engineering + product) — **pending**.
