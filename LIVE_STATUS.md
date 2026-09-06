# LIVE_STATUS.md — authoritative live progress tracker

Rows use ONLY these four states: **Done · Mostly Done · In Progress · Not Started**.
Review and update before every commit whenever actual closure or remaining scope changes;
summarize the snapshot in every commit-workflow completion message.

| Area | Status | Notes |
| --- | --- | --- |
| Discipline spine | Done | memory architecture · task-trees · commit workflow · doctrine enforcement · mdBook skeleton |
| Roadmap pair landed (`ROADMAP.md` + companion `KICKOFF.md`) | Done | `RB-SEED.1`; v0.4.1 frozen; Phase 0 companion is `KICKOFF.md` |
| Roadmap seeded into task-trees | Done | `PROGRAM` + `PHASE-0`…`PHASE-9`; census in `RB-SEED.2` |
| Claim-verification architecture | Done | `docs/CLAIM_VERIFICATION.md`; `RB-SEED.3` |
| Phase 0 — contracts and kill-risk experiments | Done | `.0.1`–`.0.8` + `.1.1`–`.1.4` done (WP1 complete); `.2.1` atomic transaction + `.2.2` leased outbox worker done (**WP2 complete**); `.3.1` node journal + inspection CLI + `.3.2` outbound node channel done (**WP3 complete**); `.4.1` fake adapter + supervisor + `.4.2` first real harness (Codex CLI) done (**WP4 complete**); `.5.1` authority engine + `.5.2` budget engine done (**WP5 complete**); `.6.1` control API + CLI + `.6.2` node wiring + two-host demo done (**WP6 complete**); `.7` deliberation benchmark done (**WP7 complete**); `.8.1` gate package + `.8.2` exit-gate closure (**WP8 complete** — ADR-002 **signed by the accountable owner 2026-09-06**: Phase 0 formally exits, KICKOFF §7); `MAINT-1` (README_POLICY re-adopted) + `MAINT-2` (ReasonBraid-only naming sweep) done — **the PHASE-0 tree is complete** |
| Phase 1 — trustworthy LAN vertical slice | Not Started | opened by the ADR-002 signature; `.1` (coordinator modular monolith + PG migrations + aggregate/event/outbox) unblocked |
