# Phase 0 ADR set (`PHASE-0.8.1`)

- Date: 2026-09-07 · Status: accepted (traceability record — the decisions
  themselves were accepted in their own records)
- Purpose: KICKOFF WP8 requires "ADR decisions for persistence/outbox, node
  journal, transport, adapter boundary, provider ambiguity, initial
  authorization, and Phase 1 scope". Each topic is already decided in a
  dedicated record; this file is the audit map, not a re-decision.

| Required ADR topic | Decided in | Decision in one line |
| --- | --- | --- |
| Persistence / outbox | `docs/decisions/2026-09-06_atomic-transaction.md` + `2026-09-06_outbox-worker-fencing.md` | one PostgreSQL transaction per command (idempotency + event + state + outbox); leased outbox worker with per-claim fencing tokens, each phase its own commit |
| Node journal | `docs/decisions/2026-09-06_node-journal.md` | SQLite WAL + synchronous=FULL, boundary-before-boundary ordering, crash recovery reports honest `outcome_unknown`, proof or adjudication are the only exits |
| Transport | `docs/decisions/2026-09-06_node-channel.md` | HTTP/1 JSON over loopback (dev profile); nodes initiate connections; cursor-reporting replay + reconciliation handshake; authenticated streaming profile deferred to WP7/Phase 2 |
| Adapter boundary | `docs/decisions/2026-09-06_fake-adapter.md` + `2026-09-06_real-adapter-codex.md` | capability-declaring contract (ack ≠ completion, no credentials, unsupported lookup is never retry advice); deterministic fake oracle; first real harness = Codex CLI behind `exec --json` |
| Provider ambiguity | `docs/decisions/2026-09-06_state-transitions.md` + `2026-09-06_node-journal.md` | proof-gated `(dispatched, fail_before_dispatch)` edge; ambiguous attempts stay `outcome_unknown`, bounded and visible, never silently retried |
| Initial authorization | `docs/decisions/2026-09-06_authority-boundary.md` | development enrollment boundary as a ceiling; grants are boundary subsets enforced at creation AND evaluation; membership grants nothing; every decision audited |
| Phase 1 scope | `docs/adr/002-phase1-scope.md` (this package) | GO (recommended): LAN vertical slice per `ROADMAP.md` §20.3, single-agent-default routing from the WP7 null result, second adapter, dev-profile dependencies replaced before any non-loopback exposure |
| Benchmark methodology | `docs/decisions/2026-09-07_deliberation-benchmark.md` | offline harness over the adapter contract; corpus-carried oracle; deterministic graders only; no independence score; env-gated call-budgeted real runs |
| Node wiring / demo | `docs/decisions/2026-09-07_node-channel-wiring.md` | dispatch rides the command transaction; node results fold in claim-first keyed on the inbox command id; no silent retry |

The repository's formal ADRs are `docs/adr/001-uncleared-working-name.md` and
`docs/adr/002-phase1-scope.md`; the `docs/decisions/` records above are the
durable engineering decisions they build on (as INDEXed there).
