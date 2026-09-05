# Live risk register

Working register for programme risks. The full qualitative table remains in
`ROADMAP.md` §25; this file is the **live** subset with owners and Phase 0
actions. Likelihood/impact here are still qualitative until experiments update
them. “Stop/reframe” means pause or narrow the affected claim, not abandon
useful infrastructure.

Owner roles are titles until `PHASE-0.0.6` names people.

| ID | Risk | Phase 0 action | Stop/reframe trigger | Owner role | Status |
| --- | --- | --- | --- | --- | --- |
| R-NAME | Name/brand cannot be cleared | ADR-001: working name only; repo private | before public repo/package launch | architecture | open (ADR-001) |
| R-AMB | Provider ambiguity causes duplicate charge/effect | WP3/WP4: `outcome_unknown`; no silent retry | product claims generic automatic safe retry | architecture | open |
| R-ADAPT | Adapter/provider API churn leaks into core | WP4: capability contract; vendor DTOs stay out of core | vendor semantics in core aggregates | architecture | open |
| R-VALUE | Complexity exceeds value vs copy/paste | WP6/WP7: two-host demo + small benchmark | Phase 1 users gain no value (`ROADMAP.md` §25.1) | architecture | open |
| R-SECRET | Secret appears in prompt/log/event/artifact | WP4/WP5: credentials never enter events/fixtures | unresolved secret flow | security | open |
| R-EXACT | False exactly-once / no-duplicate-charge claim | forbidden in Phase 0 operating rules | any such claim in docs or API | architecture | open |
| R-SCOPE | Architecture ratchet (features without evidence) | parking lot + v0.4.1 freeze | v0.5.0 without Phase 0/1 evidence | architecture | open |

Update a row when an experiment changes likelihood, impact, or disposition.
Do not duplicate the entire §25 table here.
