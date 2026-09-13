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
| R-CONSENT | A principal can act **on behalf of** another without that subject's consent | `.3.4.1` measured the four gates that DO apply — the actor's own authority for the same target, the subject's authority, the widening invariant, and the actor's own thread participation — so the consequence is bounded to ATTRIBUTION: the actor gains no reach, only the audit naming the subject alongside it. No gate asks the subject | a caller disputes an audited action attributed to them, or delegation is exposed beyond one tenant's own participants | security | **new (2026-09-13)** — ⚠️ **needs a director decision**: §16.3 requires "actor/subject participation and consent" and no ADR says how a subject would express consent. Recorded rather than invented |
| R-GUARDCOVER | The guarded administrative shape covers 18 routes; **46 mutating routes do not** | `.3.3.4.13` re-derived the census with two tracked instruments and mapped every one of the 46 to a named owner (`.9.1`, `.8.2`, `.7.4`, `.5.2`, `.7.1`, `.9.3`, `.3.2`, `.10.1/.2`, `.8.1`, `.5.1`) | any document or claim describing the administrative surface as guarded without naming the 46 | architecture | **new (2026-09-13)** — being owned is not being repaired; the closure certifies none of them |
| R-REVOKE | Revoking a node did **nothing** to a node that was running, and kept doing nothing | `.4.1.3` measured every channel surface after a real revocation: `heartbeat`/`poll`/`ack`/`events` all ALLOWED, only `handshake`/`rotate` refused. Because a heartbeat read no ledger fact, a revoked node renewed its own 60 s lease for ever and never reached the two operations that re-present a certificate. Lease RENEWAL now requires a certificate that is neither revoked nor expired, bounding the tail to the lease already held | a document or claim describing node revocation as terminating a node's access, without naming the remaining lease tail; or the tail growing beyond `LEASE_TTL` again | security | **new (2026-09-13)** — the unbounded half is repaired; ⚠️ during the bounded tail the server still hands the revoked node NEWLY enqueued work (measured, deliberately not repaired there — `.4.1.3.1` owns the decision), and ⚠️ nothing at the transport re-checks a certificate because the dev wire is plain HTTP |

Update a row when an experiment changes likelihood, impact, or disposition.
Do not duplicate the entire §25 table here.
