# Live risk register

Working register for programme risks. The full qualitative table remains in
`ROADMAP.md` §25; this file is the **live** subset with owners and Phase 0
actions. Likelihood/impact here are still qualitative until experiments update
them. “Stop/reframe” means pause or narrow the affected claim, not abandon
useful infrastructure.

Owner roles resolve to Richard DJE (architecture + security), per
`PHASE-0.0.6` / `docs/decisions/2026-09-06_accountable-owners.md`.

| ID | Risk | Phase 0 action | Stop/reframe trigger | Owner role | Status |
| --- | --- | --- | --- | --- | --- |
| R-NAME | Name/brand cannot be cleared | ADR-001: working name; public repository authorized, package/domain clearance open | before package/domain/marketing release | architecture | open (ADR-001; public-repository clause corrected by director) |
| R-AMB | Provider ambiguity causes duplicate charge/effect | WP3/WP4: `outcome_unknown`; no silent retry | product claims generic automatic safe retry | architecture | **mitigated in Phase 0** (`.3.1`/`.4.1` proof; demo leg 7) — watch Phase 1 |
| R-ADAPT | Adapter/provider API churn leaks into core | WP4: capability contract; vendor DTOs stay out of core | vendor semantics in core aggregates | architecture | open (Phase 1 watch: two adapters) |
| R-VALUE | Complexity exceeds value vs copy/paste | WP6/WP7: two-host demo + small benchmark | Phase 1 users gain no value (`ROADMAP.md` §25.1) | architecture | open — **narrowed**: WP7 null result → no structure-beats-single claim; single-agent-default routing (ADR-002) |
| R-SECRET | Secret appears in prompt/log/event/artifact | WP4/WP5: credentials never enter events/fixtures | unresolved secret flow | security | open (two non-secret fixture findings resolved by qualified exact fingerprints; configured history gate passes under SIGNOFF-REPAIR.11.4.3.1.2.2) |
| R-EXACT | False exactly-once / no-duplicate-charge claim | forbidden in Phase 0 operating rules | any such claim in docs or API | architecture | open — no such claim exists (duplicate transport proven to one domain effect) |
| R-SCOPE | Architecture ratchet (features without evidence) | parking lot + v0.4.1 freeze | v0.5.0 without Phase 0/1 evidence | architecture | open — SubtractionRecord published (`.8.1`); v0.5.0 still forbidden |
| R-VARIANCE | Provider run-to-run variance on identical prompts | observed in `.7`: code-002 single 1.000 → 0.667 across runs | Phase 1 benchmark treats single shots as stable | architecture | **new (2026-09-07)** — Phase 1 harness repeats samples per case |
| R-OVERHEAD | Ambient per-call token cost (harness/Codex config ~16k in) distorts H6 accounting | recorded honestly in `.7` usage totals | a Phase 1 estimate assumes bare-prompt costs | architecture | **new (2026-09-07)** — production prompt profiles must be lean; measure |

Update a row when an experiment changes likelihood, impact, or disposition.
Do not duplicate the entire §25 table here.
