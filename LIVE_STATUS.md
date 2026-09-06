# LIVE_STATUS.md — authoritative live progress tracker

Rows use ONLY these four states: **Done · Mostly Done · In Progress · Not Started**.
Review and update before every commit whenever actual closure or remaining scope changes;
summarize the snapshot in every commit-workflow completion message.

| Area | Status | Notes |
| --- | --- | --- |
| Discipline spine (`bedrock`) | Done | memory architecture · task-trees · commit workflow · doctrine enforcement · mdBook skeleton |
| Roadmap pair landed (`ROADMAP.md` + companion `KICKOFF.md`) | Done | `RB-SEED.1`; v0.4.1 frozen; Phase 0 companion is `KICKOFF.md` |
| Roadmap seeded into task-trees | Done | `PROGRAM` + `PHASE-0`…`PHASE-9`; census in `RB-SEED.2` |
| Claim-verification architecture | Done | `docs/CLAIM_VERIFICATION.md`; `RB-SEED.3` |
| Phase 0 — contracts and kill-risk experiments | In Progress | `.0.1`–`.0.8` + `.1.1`–`.1.4` done (WP1 complete); `.2.1` atomic transaction + `.2.2` leased outbox worker done (**WP2 complete**); `.3.1` node journal + inspection CLI + `.3.2` outbound node channel done (**WP3 complete**); `.4.1` fake adapter + supervisor + `.4.2` first real harness (Codex CLI) done (**WP4 complete**); `.5.1` authority engine + `.5.2` budget engine done (**WP5 complete**); `.6.1` control API + CLI + `.6.2` node wiring + two-host crash/reconnect demo (real kill points, acceptance-asserting evidence bundle) done (**WP6 complete**); frontier `PHASE-0.7` small deliberation/routing benchmark |
