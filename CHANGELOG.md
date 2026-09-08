# CHANGELOG.md

> Entries older than this session's `.4` lane (the R0–R2 pack history) are
> rotated into the git history (the README-STABILITY rotation threshold) —
> `git log --follow CHANGELOG.md` carries the full record.

# CHANGELOG.md

> Entries older than `2026-09-06` are rotated into the git history (the
> README-STABILITY rotation threshold) — `git log --follow CHANGELOG.md`
> carries the full record.

# CHANGELOG.md

## 2026-09-07 — The scheduled reviews land (`PHASE-6.6`)

- Migration 0045: the seven §15.11 review triggers, the DUE evaluation over the `.5.3` records (one due review per (publication, trigger) — the dedupe; the schedule is idempotent), the due → done transition, and the outcome-trigger vocabulary back-fill (an unknown trigger is now the typed refusal at the outcome registration). The verbs: `POST /v1/policy-reviews/schedule`, `GET /v1/policy-reviews`, `POST /v1/policy-reviews/{id}/done`. Measured: policy 11.

## 2026-09-07 — The drift and the corrections land — the `.5` lane is COMPLETE (`PHASE-6.5.3`)

- Migration 0044: the drift records (the six §15.10 categories over the desired/observed pair), the §4.7 corrections (the authority-grant re-check; the suspension/waiver REQUIRE the expiry; the supersession links the old; the retraction NEVER deletes — the correction is a new row), and the §15.11 outcome records. The verbs: `POST`/`GET /v1/policy-drift`, `/v1/policy-corrections`, `/v1/policy-outcomes`. Measured: policy 10. **The `.5` lane (the target deployment) is COMPLETE.**

## 2026-09-07 — The deployment records land (`PHASE-6.5.2`)

- Migration 0043: the authority-checked targets (the closed type vocabulary), the per-target assignments over the EFFECTIVE publication (the chain gate) with the DESIRED pair (the ref + the digest), and the receipt attesting the OBSERVED digest + the state — the drift's comparison input. The verbs: `POST`/`GET /v1/deployment-targets`, `POST`/`GET /v1/deployments`, `POST /v1/deployments/{target}/{publication}/receipt`. Measured: policy 9.

## 2026-09-07 — ADR-021: the target-deployment contract (`PHASE-6.5.1`)

- ADR-021 accepted (`docs/adr/021-target-deployment.md`): the deployment is per-target waves, never globally atomic; the receipt attests the digest (never the hope); the drift is the six-way §15.10 vocabulary; the §4.7 correction authorities stay distinct (the reversal is fast, the authority is not universally lower). No code.

## 2026-09-07 — The target-deployment lane is decomposed at the census seams (`PHASE-6.5`)

- The deployment lane is the greenfield (no target/wave/receipt/drift/correction record exists); the `.4` effective publications + the digests are the inputs. Children: `.5.1` ADR-021 → `.5.2` the deployment records + the waves → `.5.3` the drift + the corrections.

## 2026-09-07 — The reconciliation matrix lands — the `.4` lane is COMPLETE (`PHASE-6.4.3.3`)

- The `reconciler` module: the pure `reconcile` function over (the DB state, the observed Git state, the expected id) → the six §15.8 actions (the idempotent retry, the verify-and-advance, the stop-and-alert, the freeze-and-repair, the **quarantine-and-adjudicate — never a silent promote**, the out-of-band alert); the kill-point tests map every matrix row + prove the idempotency (the same pair yields the same action). Measured: reconciler 3. **The `.4` lane (the signed canonical publication) is COMPLETE.**

## 2026-09-07 — The Git publication half lands (`PHASE-6.4.3.2`)

- The `publisher` module (the gix plumbing — no CLI): the blobs + the filename-sorted tree + the root commit, the staging branch, the fetch-back verification (the re-derived digest), the IMMUTABLE publication ref (the written-once — the re-publish is the typed refusal), and the EFFECTIVE channel via the compare-and-swap (the stale expectation is the typed `CasMismatch`). The `POST /v1/policy-publications/{id}/publish` verb drives the half and marks the record effective with the ref ids. Measured: publisher 2 + policy 8.

## 2026-09-07 — The publication-store contract (`PHASE-6.4.3.1`)

- The decision record (`docs/decisions/2026-09-07_publication-store-contract.md`): the LOCAL bare repository (the remote-publication profile is a named deferral), the three-ref scheme (the staging branch, the immutable publication ref — written once, the effective channel via the compare-and-swap with the expected old id), and the gix write path (no git CLI). No code.

## 2026-09-07 — The Git-publication half is decomposed at the census seams (`PHASE-6.4.3`)

- The write half is the greenfield (the Phase-4 R1 pack only ACQUIRES — no commit/ref-write path exists; the reconciler exists nowhere); the `.4.2` publication records are the matrix's DB half. Children: `.4.3.1` the publication-store contract → `.4.3.2` the Git publication half → `.4.3.3` the reconciliation matrix + the kill-point tests.

## 2026-09-07 — The publication records land (`PHASE-6.4.2`)

- Migration 0042: the publication is its own aggregate row (never folded into the approval) with the chain-verified references (the proposal must be APPROVED, the decision + the approval must belong to it, the projection must exist), the manifest digest, and the typed state machine (staged → effective with the Git object ids | failed with the reason — never a skip). The verbs: `POST`/`GET /v1/policy-publications` + `/effective` + `/failed`. Measured: policy 7.

## 2026-09-07 — ADR-020: the canonical-publication contract (`PHASE-6.4.1`)

- ADR-020 accepted (`docs/adr/020-canonical-publication.md`): the publication is the nine-step §15.7 state machine over the staged record; the Git refs are the publication truth with the compare-and-swap idempotency; the reconciliation is the six §15.8 rules (the never-silent-promote); the signatures ride the manifest digest. No code.

## 2026-09-07 — The publication lane is decomposed at the census seams (`PHASE-6.4`)

- The inputs + the substrate ship (the decisions/approvals, the byte-identical projections, the Git machinery, the CA keys, the transactional outbox); the publication records + the reconciliation matrix are the greenfield. Children: `.4.1` ADR-020 → `.4.2` the records + the staging → `.4.3` the Git publication + the reconciliation.

## 2026-09-07 — The Codex and the Claude projections land — the `.3` lane is COMPLETE (`PHASE-6.3.3`)

- The compiler crate gains the `codex` (the AGENTS.md fragment — the backticked clause ids) and the `claude` (the CLAUDE.md fragment — the plain ids) renderers over the same stable-sorted core; the §15.5 coverage ships: the backtick escape (the harness-parse safety), the 8192-character statement ceiling (the oversized statement DECLARES itself, never truncates), the per-target byte-identical repeats. Measured: compiler 8 + policy 6. **The `.3` lane (the deterministic compiler) is COMPLETE.**

## 2026-09-07 — The compiler core lands (`PHASE-6.3.2`)

- The hermetic `reasonbraid-policy-compiler` crate (no database/network/clock): the pure `compile` function with the stable sort, the generic bundle + the `policy.lock` renderers, the escaping, the declared unrepresentable, and the ADR-011 digest over the rendered bytes. The server gains migration 0041 + the `POST`/`GET /v1/policy-projections` verbs (the resolve → the compile → the record). Measured: compiler 5 + policy 5.

## 2026-09-07 — ADR-033: the projection-compiler contract (`PHASE-6.3.1`)

- ADR-033 accepted (`docs/adr/033-projection-compiler.md`): the compiler renders the resolved set, never re-resolves; the rendering is byte-identical by construction; the unrepresentable clause is the DECLARED refusal (never a silent omission); the compiler is a separate hermetic crate; the projection digest is the publication's verification primitive. No code.

## 2026-09-07 — The compiler lane is decomposed at the census seams (`PHASE-6.3`)

- The projection compiler is the greenfield (no projection, no target vocabulary, no unrepresentable declaration exists); the `.1` resolution's clause set is its input. Children: `.3.1` ADR-033 → `.3.2` the compiler core → `.3.3` the Codex + the Claude projections.

## 2026-09-07 — The approval records land — the `.2` lane is COMPLETE (`PHASE-6.2.3`)

- Migration 0040: the approval is its OWN row with the AUTHORITY PROOF — the grant re-checked at the approval boundary (active, unexpired, held by the approver — the §4.5 identity/authority at the action time); the quorum snapshot rides the row; the approval advances the proposal (`decided` → `approved`). The verbs: `POST`/`GET /v1/policy-approvals`. Measured: policy 4. **The `.2` lane (the policy lifecycle) is COMPLETE.**

## 2026-09-07 — The proposal and the decision records land (`PHASE-6.2.2`)

- Migration 0039: the proposal is a REFERENCE (the policy version + the deliberation thread — never a copy) with the typed stage machine; the decision freezes the electorate snapshot at the action time and references a verdict of the PROPOSAL's thread (a foreign verdict refuses); one proposal, one decision (the stage gate). The verbs: `POST`/`GET /v1/policy-proposals` + `POST`/`GET /v1/policy-decisions`. Measured: policy 3.

## 2026-09-07 — ADR-032: the policy-lifecycle contract (`PHASE-6.2.1`)

- ADR-032 accepted (`docs/adr/032-policy-lifecycle.md`): the five records never fold (the discussion rides the thread, the decision/approval/publication/deployment are their own rows); the proposal is a reference, not a copy; the decision freezes the electorate snapshot at the action time; the approval re-checks the grant at the approval boundary. No code.

## 2026-09-07 — The lifecycle lane is decomposed at the census seams (`PHASE-6.2`)

- The §15.6 lifecycle records are the greenfield (no proposal/approval/decision row exists); the substrate ships: the `.1` policy registry, the Phase-5 deliberation threads (one thread, multiple decisions — the verdict kind), the Phase-2 authority model (the proofs' substrate). Children: `.2.1` ADR-032 → `.2.2` the proposal + the decision records → `.2.3` the approvals + the proofs.

## 2026-09-07 — The seven-step resolution lands — the `.1` lane is COMPLETE (`PHASE-6.1.3`)

- The `POST /v1/policies/resolve` pipeline runs the seven §15.3 steps (the authority check, the applicability filter, the dependencies/conflicts, the DAG precedence, the exception schemas, the FAIL-CLOSED binding conflict, the explanation tree); the impact map (`GET /v1/policies/{id}/{version}/impact`) is the derivable coverage. Measured: policy 2. **The `.1` lane (the semantic policy schema) is COMPLETE.**

## 2026-09-07 — The typed policy schema lands (`PHASE-6.1.2`)

- Migration 0038: the `PolicyVersion` document (the §15.1 fields — the stable clause ids, the applicability, the exception schema, the provenance) with the OWNERSHIP metadata validated against the authority model (the owning authority must be an ACTIVE grant — the label grants nothing); the ADR-011 digest shape, the semantic version, the closed lifecycle vocabulary, the unique clause ids. The verbs: `POST`/`GET /v1/policies`. Measured: policy 1.

## 2026-09-07 — ADR-019: the semantic-policy contract (`PHASE-6.1.1`)

- ADR-019 accepted (`docs/adr/019-semantic-policy.md`): the policy is a versioned digest-pinned document (never prose); the ownership is the authority binding (the label grants nothing); the `PolicySetVersion` is a lock manifest; the resolution is the seven §15.3 steps, fail-closed, with the explanation tree riding the result. No code.

## 2026-09-07 — The semantic-policy lane is decomposed at the census seams (`PHASE-6.1`)

- The `.1` block is LIFTED: the authority (Phase 2), the deliberation (Phase 5 — now closed), the Git/object consistency (Phase 4), and the correction model all ship; the policy schema itself is the greenfield. Children: `.1.1` ADR-019 (the queue's canonical-policy-schema item) → `.1.2` the typed `PolicyVersion` + the registry → `.1.3` the seven-step layering/precedence + the impact maps.

## 2026-09-07 — The G5 gate package — PHASE 5 IS CLOSED (`PHASE-5.6.2`)

- **G5 Met as a subtraction gate**: the "deliberation improves answers" claim is withdrawn per §25.1 (the first controlled evaluation reads H1 null on the 4-case sample); the honest-inconclusive machinery ships and tests; the benchmark thresholds ship as the instrument; the §19.8 subtraction record + the evidence manifest + the gate record land; the README's stale status line is fixed. The frontier moves to `PHASE-6.1` (the semantic policy lane).

## 2026-09-07 — The G5 evidence census (`PHASE-5.6.1`)

- The claim census: the README carries NO quality claim to retract (its claim is the mechanism one — shipped and tested); the bench's first live run reads H1 null on the 4-case differential sample (no structured workflow beat `single`; the burden 2–4×); five subtraction candidates named (S-1 the narrowed structure claim, S-2 the deferred calibration, S-3 the already-shadow learned routing, S-4 the supported cost claim, S-5 the README's stale status line); the declared domains are the seven §13.8 case classes. No code.

## 2026-09-07 — The G5-exit lane is decomposed at the census seams (`PHASE-5.6`)

- The G5 gate ships as the `.4.4` SERVICE (the calibration + the baseline/threshold gates); the exit needs the PACKAGE: the claim census (the docs' quality claims vs the evidence), the gate record, the §19.8 subtraction record, and the narrowed product claims. Children: `.6.1` the evidence census → `.6.2` the gate package.

## 2026-09-07 — The shadow recommendation lands — the `.5` lane is COMPLETE (`PHASE-5.5.3`)

- Migration 0037: the recommendation maps a class to an arm drawn from the EXISTING registered profiles (never a raise), names its `.4` evidence reference, and is recorded with `applied: false` on its face — the create boundary keeps resolving the RULE table (the shadow proof). Measured: routing 2. **The `.5` lane (the routing policy) is COMPLETE.**

## 2026-09-07 — The rule-based routing policy lands (`PHASE-5.5.2`)

- Migration 0036: the seven §13.8 rows as the built-in rules (one deterministic arm per case class); the resolution is a lookup with an append-only audit row; the create boundary applies the policy ONLY when no explicit profile is named (the explicit profile always wins; the bare thread keeps the `quick_advice` default); the arm must be a registered profile (a phantom arm fails closed). The verbs: `GET /v1/routing/rules`, `POST /v1/routing/resolve`, `GET /v1/routing/resolutions`. Measured: routing 1.

## 2026-09-07 — ADR-031: the routing-policy contract (`PHASE-5.5.1`)

- ADR-031 accepted (`docs/adr/031-routing-policy.md`): the case class is a submitted input (never a derived judgment); the rule-based policy is a deterministic table; the human authority outranks the rule (the explicit profile always wins); the learned routing is the shadow recommendation — an existing arm, recorded with its evidence, never applied, never a raise. No code.

## 2026-09-07 — The routing-policy lane is decomposed at the census seams (`PHASE-5.5`)

- The §13.8 census: the routing decision is the CLIENT's choice today (the create carries the explicit profile; the bare thread defaults to the hardcoded `quick_advice`) — no rule, no case-class vocabulary, no policy; the §13.1 built-ins map §13.8's rows. Children: `.5.1` ADR-031 → `.5.2` the rule-based policy → `.5.3` the shadow recommendation.

## 2026-09-07 — The calibration and the regression gates land — the `.4` lane is COMPLETE (`PHASE-5.4.4`)

- Migration 0035: the calibration accumulates the Brier + the confidence over the NAMED runs (each must be registered — never a fabrication); the gate records the baseline + the threshold; the evaluation compares each measured case against the baseline minus the threshold and APPENDS its result (a drop below is the typed failure with the delta — the gate never rewrites a result, it only blocks). Measured: evaluation 3. **The `.4` lane (the versioned evaluation service) is COMPLETE.**

## 2026-09-07 — The shadow routing trials land (`PHASE-5.4.3`)

- Migration 0034: the trial records the declared seed + the arms + the cohorts + the case ids, and the SERVER computes the seeded assignment (a dependency-free splitmix64 — the `std` hasher is not stable across releases, the draw must be): the same seed + cases re-draw the same assignment. The per-arm results append (never overwrite); the trial never changes production routing (the `.5` lane's decision consumes the records). Measured: evaluation 2.

## 2026-09-07 — The evaluation-service core lands (`PHASE-5.4.2`)

- Migration 0033: the versioned corpus registry (the declared 64-hex digests) + the experiment run records (the workflow arm, the corpus reference, the DECLARED seed — a non-deterministic run without one is the typed refusal — the trial count, the harness's results). The four verbs (`POST`/`GET /v1/evaluations/corpora`, `POST`/`GET /v1/evaluations/runs`) record, never re-grade: the service RECORDS, the WP7 harness MEASURES. The pg script gained the `evaluation` suite.

## 2026-09-07 — ADR-017: the evaluation-service contract (`PHASE-5.4.1`)

- ADR-017 accepted (`docs/adr/017-evaluation-service.md`): the service RECORDS, the harness MEASURES (one grading implementation); the registry is versioned + digest-pinned; the experiment records declare their seeds; the randomized routing trials are shadow-only; the cohorts are recorded labels; the calibration accumulates; the regression gate is the blocking-only G5 threshold. No code.

## 2026-09-07 — The evaluation lane is decomposed at the census seams (`PHASE-5.4`)

- The §13.7/§19.5 census: the WP7 bench harness (Phase 0) is a substantial substrate — the digest-carrying versioned corpus, the four deliberation workflows over the real Adapter contract, the deterministic grading (Brier + rubrics), the spread-bearing reports, the ScriptedAgent self-test; the GREENFIELD is the service itself (the case registry, the run records, the randomized trials, the cohorts, the calibration record, the regression gates). Children: `.4.1` ADR-017 → `.4.2` the service core → `.4.3` the routing experiments + the cohorts → `.4.4` the calibration + the gates.

## 2026-09-07 — The synthesis record lands — the `.3` lane is COMPLETE (`PHASE-5.3.3`)

- The `synthesis` record rides a `summary`-kind contribution on the `synthesize` step: the synthesizer identity, the event-log input range (validated — `1 <= from <= to <=` the thread's max version, so the transformation is re-derivable), the source links, the coverage report. The live pass caught the `.1.3`-lane gap: the create handlers resolved the profile only when named, so a bare thread's steps were EMPTY (the step gates read `none`) — both handlers now resolve always (`None` → `quick_advice`). Measured: profiles 31. **The `.3` lane (moderator/synthesizer constraints) is COMPLETE.**

## 2026-09-07 — The moderation kinds land (`PHASE-5.3.2`)

- The closed moderation vocabulary (`classify`, `request_clarification`, `propose_close`, `draft_summary`, `identify_unanswered`) rides the contribute verb: the capability-shaped fields refuse on it (the §13.5 prohibitions by construction), the action references its target via `ref_event_id` (must exist in the thread), the `moderate` step joins the step vocabulary and gates the kinds, and the action is challengeable — the appeal IS the challenge. Measured: profiles 30.

## 2026-09-07 — ADR-030: the moderation/synthesis contract (`PHASE-5.3.1`)

- ADR-030 accepted (`docs/adr/030-moderation-and-synthesis.md`): the moderation action is a contribution (never a new authority); the closed kind set refuses the capability-shaped fields — the §13.5 prohibitions hold by construction; the appealable action IS the existing challenge; the `moderate` step joins the vocabulary; the synthesis record is re-derivable derived content. No code.

## 2026-09-07 — The moderator/synthesizer lane is decomposed at the census seams (`PHASE-5.3`)

- The §13.5 census: the moderator is ZERO machinery (no role, no kind, no step — the step vocabulary has no `moderate`); the synthesizer is half-shipped (the `synthesize` step name + the `.2.4.1` minority-report shapes). Children: `.3.1` ADR-030 → `.3.2` the moderation kinds → `.3.3` the synthesis record.

## 2026-09-07 — The contribution-side execution lands — the `.2` lane is COMPLETE (`PHASE-5.2.4.2`)

- The kind vocabulary gains `evidence_request` (targets ONE claim digest of THIS thread — the JSONB scan over the server-computed records; the request is not an acquisition) and `verdict` (the judged digest + the rule + the §13.4 outcome, canonicalized); the step gates execute the ADR-016 composition (request → `evidence_request`, verdict → `adjudicate`); the round advance generalizes to the step advance (one step per round, clamped at the terminal — the blind commitment flag is its special case); the `evidence_reference` kind refuses empty refs. Measured: profiles 29. **The `.2` lane (blind-first contributions, structured claims, evidence requests, adjudication, minority reports, the twelve terminals) is COMPLETE.**

## 2026-09-07 — The close vocabulary lands (`PHASE-5.2.4.1`)

- The close outcome speaks §13.4's twelve terminals: `decided`/`inconclusive` stay accepted aliases but never persist — the event and the projection's `close_outcome` carry the canonical name; the family rule replaces the `.1.5.3` check (a decision terminal with an unresolved register is the typed refusal); the minority report rides the close event (synthesizer, input range, sources, coverage). Measured: profiles 28.

## 2026-09-07 — The close/contribute seam splits the `.2.4` leaf (`PHASE-5.2.4`)

- The `.2.4` census: the CLOSE side (a two-valued outcome vs §13.4's twelve; no minority report) and the CONTRIBUTION side (no `evidence_request`/`verdict` kinds; the `evidence_reference` kind accepts empty refs). Children: `.2.4.1` the close vocabulary → `.2.4.2` the contribution-side execution.

## 2026-09-07 — The blind-first lane lands (`PHASE-5.2.3`)

- A contribution posted during the `blind_solicit` step carries `blind: true`; the round advance is the commitment point (the step moves past `blind_solicit`, the event records `blind_committed`); the read surface serves a still-blind contribution to non-authors as `blind_until: round_advance` + the content digest — the ledger keeps the full body (a read rule, never a store rewrite); a non-author challenge of a blind contribution is the typed refusal. Measured: profiles 27.

## 2026-09-07 — The structured records land (`PHASE-5.2.2`)

- The contribute body carries `claims` (content-only input; the server computes the ADR-011 digest — the wire never supplies one); claims ride a `claim`-kind contribution only; the challenge body gains `claim_digest` (the structured objection names ONE server-derived claim of the target — a foreign digest is the typed refusal); the projection carries the `structured_claims` counter; the free-text wire stays valid. Measured: profiles 26.

## 2026-09-07 — ADR-029: the structured-deliberation contract (`PHASE-5.2.1`)

- The `.2.1` leaf accepted ADR-029 (`docs/adr/029-structured-deliberation.md`): the typed claim/objection/revision records are SHAPES over the existing verbs (never a new capability); the blind commitment point is a READ-SURFACE rule (the ledger holds the blind content from post time; the round advance commits); the evidence request is a contribution, not an acquisition; the adjudication is an attributable verdict; the minority report carries the coverage report; the close speaks §13.4's twelve terminals. No code.

## 2026-09-07 — The deliberation lane is decomposed at the census seams (`PHASE-5.2`)

- The §13.4/§13.6 census: FOUR greenfields (the blind-first visibility, the structured claim/objection/revision records, the evidence requests, the adjudication + the minority reports + the terminals) against the reusable pieces (the contribution kind vocabulary, the open_challenges + unresolved registers, the Phase-4 evidence pipeline). Children: `.2.1` ADR-029 → `.2.2` the structured records → `.2.3` the blind-first lane → `.2.4` the requests + the adjudication + the reports.

## 2026-09-07 — The workflow-profile lane is complete: the steps ride the projection (`PHASE-5.1.3`)

- The projection carries the resolved step sequence + the current index: the create seats step 0 with the registry's steps, the close advances to the terminal step — the lifecycle's own transitions are the only step transitions, so the authorization/budget/lifecycle invariants ride every step by construction.
- Measured: profiles 25 (the suite grew 24→25). **`.1` COMPLETE** — frontier → `.2` (the blind-first contributions lane).

## 2026-09-07 — The profile registry ships: the validated reference replaces the stored string (`PHASE-5.1.2`)

- `migrations/0032`: the eight §13.1 built-ins as versioned entries.
- `src/workflows.rs`: the twelve-kind step vocabulary, the three composition rules (the known kinds, the terminal last, the adjudicate-after-blind), the resolve/register/list surfaces.
- The thread's `workflow_profile` is now a VALIDATED reference: the unknown id is the typed refusal at the create boundary; the bare thread defaults to `quick_advice`; the Phase-1 enum is gone.
- Measured: profiles 25. Frontier → `.1.3` (the profile-driven execution).

## 2026-09-07 — ADR-016 is accepted: the profile composes verbs, never capabilities (`PHASE-5.1.1`)

- The workflow profile is VERSIONED CONFIGURATION over the thread aggregates; the composition invariants (no profile bypasses authorization/budget/lifecycle — an invalid profile is invalid at validation time); the eight §13.1 built-ins are the initial registry; the unknown profile is a typed refusal, never a stored string.
- Durable in `docs/adr/016-workflow-profiles.md` (top-level `answers:`). No code. Frontier → `.1.2` (the registry + the validation).

## 2026-09-07 — Phase 5 opens: the census found the workflow profile is an unvalidated string (`PHASE-5.1`)

- The CLI passes `workflow_profile` through to the thread body; `threads.rs` stores it verbatim — no DSL, no validation, no step composition, and ADR-016 is unopened.
- Reusable: the Phase-1 state machines, the typed contributions, the Phase-2 budgets, the Phase-4 evidence pipeline (the `evidence_review` profile's substrate).
- Decomposed: `.1.1` ADR-016 + the census → `.1.2` the profile registry + the validation → `.1.3` the profile-driven execution. Frontier → `.1.1`.

## 2026-09-07 — Phase 4 is closed: the G4 gate is Met (`PHASE-4.7.2`)

- The gate package: the evidence manifest (every G4 clause → a re-runnable artifact), the gate record (**Met**, five named deferrals, top-level `answers:`), the subtraction record (the §20.6 rows 31–35 shipped + the deferrals — no empty lists).
- The supply-chain re-run: `make deny` rc=0 (the R2/R3 duplicate families reviewed + skipped with the rationale; the uluru MPL-2.0 exception narrowed) + `make secret-scan` rc=0 (163 commits, no leaks).
- The tree flips `done`; the frontier moves to `PHASE-5.1`; the book's roadmap chapter reflects the completion.

## 2026-09-07 — The G4 hostile suite ships: eight refusal scenarios, one gate-citable test (`PHASE-4.7.1`)

- `profiles 23` (`the_g4_hostile_suite_names_every_refusal`): the loopback/private/mapped-form refusals through the resolution path, the userinfo refusal, the unsupported scheme's unresolvable-now, the fake digest 400, the unknown assessment kind, the forged-field 422 — each names its reason.
- The worker-side hostile cases ride the extract crate's nine offline refusals (the bomb, the traversal, the encrypted/JS PDFs). Frontier → `.7.2` (the G4 gate record + the subtraction record).

## 2026-09-07 — The G4 exit opens: the refusals exist, the consolidated proof does not (`PHASE-4.7`)

- The explicit-failure machinery is measured per-lane (the refusal matrices, the budget trips, the fake digest/excerpt refusals, the unresolvable-now) — but the G4 gate has no consolidated suite and no subtraction record.
- Decomposed: `.7.1` the hostile-content suite (ONE gate-citable test result) → `.7.2` the G4 gate record + the subtraction record. Frontier → `.7.1`.

## 2026-09-07 — The evidence pipeline is complete: the retention enforces, the freshness surfaces (`PHASE-4.6.4`)

- `migrations/0031`: the `license`, `fresh_until`, `refreshed_at` columns.
- `src/snapshots.rs`: the retention TTLs (the audit class never expires — binding decisions stay addressable), the `expire_due` enforcement (the tombstone rides the class's TTL), the `stale` surface, and the re-fetch policy (the replay refreshes the freshness).
- The verbs (`POST /v1/snapshots/expire-due` with the `at` override, `GET /v1/snapshots/stale`). Measured: profiles 22. **`.6` COMPLETE** — frontier → `.7` (the G4 hostile-content suite).

## 2026-09-07 — The claim-evidence graph ships: the citation is validated, not asserted (`PHASE-4.6.3`)

- `migrations/0030`: the assessment edges — the five kinds (the CHECK constraint), the author/verifier, the excerpt + selector, the rationale, the authority/freshness/independence/uncertainty, the replay index.
- `src/claims.rs`: the typed submission + the CITATION VALIDATION — the excerpt MUST appear in the snapshot's raw bytes (the fake excerpt is refused; citation existence alone never satisfies an evidence gate) — plus the two read surfaces.
- Measured: profiles 21 — the true excerpt accepts + replays, the fake excerpt refuses, the unknown kind names itself. Frontier → `.6.4` (the license/retention + the freshness).

## 2026-09-07 — The derivation graph ships: every transformation is an edge (`PHASE-4.6.2`)

- `migrations/0029`: the `Derivation` edges — the parent link, the derived kind, the content's OWN verified ADR-011 digest, the replay index.
- `src/derivations.rs`: the typed submission (the content MUST hash to the declared digest; the parent must exist; the same parent + kind + digest replays), the `children_of` traversal.
- The verbs + the R2 chunk auto-derivations (the extract chunks land as the snapshot's edges). Measured: profiles 20. Frontier → `.6.3` (the claim-evidence graph + the citation validation).

## 2026-09-07 — The snapshot store ships: the content-addressing is verified, the deletion is a tombstone (`PHASE-4.6.1`)

- `migrations/0028`: `snapshot_objects` (the bytes under their ADR-011 digest — identical bytes, one row) + `evidence_snapshots` (the §12.6 shape; the tombstone state rides the row).
- `src/snapshots.rs`: the typed submission, the VERIFIED digest (the bytes must hash to the declared one — never trusted), the replay, the tombstone (the reason + the time, idempotent).
- The verbs (`POST`/`GET`/`DELETE /v1/snapshots`) + the resolve handler's R0/R2/R5 auto-submits (the acquired bytes land with the provider receipts + the disclosure policies).
- Measured: profiles 19 — the roundtrip, the replay, the mismatch 400, the tombstone. Frontier → `.6.2` (the derivation graph).

## 2026-09-07 — The snapshots lane opens: the receipts exist, nothing persists them (`PHASE-4.6`)

- The `.2`–`.5` packs produce the ADR-011 receipt shapes; NOTHING stores them — no `EvidenceSnapshot`, no `Derivation` edges, no claim-evidence assessments, no tombstone (the object store is the Phase-4 blocker's last leg, the `.1` census's named trigger).
- Decomposed at the census seams: `.6.1` the snapshot store + the tombstone → `.6.2` the derivation graph → `.6.3` the claim-evidence graph + the citation validation → `.6.4` the license/retention + the freshness. Frontier → `.6.1`.

## 2026-09-07 — The highest-risk lane is wired, and the gate is structural (`PHASE-4.5.3`)

- `resolvers.rs`: the startup sync (`sync_gated_entries`) — opening registers the R3/R5/RX rows, closing REMOVES them; the disabled pack has no row, so the resolve can never return it. The auth filter routes credential-carrying references to the `credential` class only.
- `fetcher.rs` + `browse.rs` + `broker.rs`: the per-request authenticated fetch (the credential attaches for THAT acquisition only), the render pre-flight + spawner, the disclosure-bearing `AuthenticatedReceipt` and the network-log `BrowserReceipt`.
- The handler's R5/R3/RX branches run behind the enabled belt; the binary syncs the gate at startup (`RB_ENABLE_R5R3RX`, OFF by default).
- Measured: profiles 18 — closed → the unresolvable-now; open → the authenticated loopback refusal names the class (the SSRF proof through the authenticated path), the render pre-flight refuses before any spawn, the §12.8 capability call publishes; closed again → the rows are gone. **`.5` COMPLETE (the gated lane)** — frontier → `.6` (the snapshots + derivation-graph lane).

## 2026-09-07 — The highest-risk lane's machinery ships, compiled but unwired (`PHASE-4.5.2`)

- `crates/reasonbraid-browse`: the R3 browser worker — the stdio protocol, the bounded interaction (navigate/click/scroll/type + the step budget + the wall-clock ceiling), the network-log disclosure, the provenance-named browser startup check; two tests against the REAL Chrome (the local render + the step-budget refusal before any navigation).
- `src/broker.rs`: the R5 credential broker — the opaque binding ref, the per-request attach, the REDACTED Debug (the value never logs), the `DisclosureRecord`.
- `src/mediated.rs`: the typed §12.8 vocabulary — the six response shapes, the not-inspected-original record, the second-verifier rule.
- Compiled but UNWIRED — the gate is the `.5.3` wiring's. Frontier → `.5.3`.

## 2026-09-07 — The highest-risk lane's contracts are decided: disclosed, contained, and off by default (`PHASE-4.5.1`)

- R5: the LOCAL credential broker — the opaque binding ref, the per-request delegated session, the explicit-disclosure receipt (a credential is a disclosure, not a permission).
- R3: the bounded browser — the step + network-log budgets, the killing worker, and the deployment-checked `vm_container` requirement (the gate refuses to open without it); the census measured chromiumoxide 0.9.1 over headless_chrome 1.0.22; the engine is a pinned, provenance-named chromium with a startup version check.
- RX: the typed §12.8 vocabulary (the six response shapes, the not-inspected-original record, the second-verifier rule).
- The OPT-IN gate: compiled but DISABLED; the enablement is a named recorded change; the resolve never returns a disabled pack.
- Durable in `docs/decisions/2026-09-07_r5r3rx-contracts-opt-in.md` (top-level `answers:`). No code. Frontier → `.5.2` (the gated machinery).

## 2026-09-07 — The R3/R5/RX lane opens: the census found NOTHING exists (`PHASE-4.5`)

- No browser/MCP crate in the lock; the credential surface is the `.1.2` opaque binding-ref plus the fetcher's no-ambient-credentials baseline; §12.8 (the agent-mediated vocabulary) has no machinery.
- Decomposed at the census seams: `.5.1` the three contracts + the OPT-IN gate (default OFF — the packs ship compiled but disabled) → `.5.2` the machinery (gated) → `.5.3` the receipt + the wiring. Frontier → `.5.1`.

## 2026-09-07 — Pack R2 is complete: the pipeline, the receipt, and the media-type routing (`PHASE-4.4.3`)

- `migrations/0027`: the R2 install record (`r2-extract-worker` — the extraction media types, egress `listed` + sandbox `process` — the first ladder-up, the kill-on-budget-trip evidence).
- `src/extraction.rs`: the `ExtractionReceipt` (the Derivation edge — the parent digest, the derived chunk digests, the extractor version, the excluded list) and the spawner (ONE request line, ONE response line, the time budget KILLS the worker).
- `resolvers.rs` + `api.rs`: the resolve's media-type filter (hinted references rank the extraction pack; hintless ones keep the acquisition-only path) and the handler's pipeline (the R0 acquisition under the `.2.1` policy, then the killing-budget worker).
- Measured: profiles 17 — the hinted reference pipelines, the loopback refusal names the class through the resolution path. **`.4` COMPLETE (pack R2)** — frontier → `.5` (the opt-in private/authenticated connectors — the highest-risk lane).

## 2026-09-07 — The extraction worker ships: the stdio quarantine parses the four formats (`PHASE-4.4.2`)

- `crates/reasonbraid-extract` (the new workspace crate): ONE JSON request in, ONE response out, exit — the fresh process IS the quarantine. The per-format parsers (the PDF text layer, the one-level zip/tar archives, the Atom/RSS feeds) derive the chunks (each with its own ADR-011 digest + the parent digest), and the refusal list is mechanical and named (encrypted/JS PDFs, nested archives, traversal, the ratio brake over the compressed envelope, the ceilings).
- Nine tests including two stdio roundtrips spawning the built binary. Frontier → `.4.3` (the receipt + the R2 pack wiring).

## 2026-09-07 — The R2 contract is decided: extraction is a Derivation, parsed in a worker quarantine (`PHASE-4.4.1`)

- The parser census, measured: lopdf 0.44.0 (chosen) vs pdf 0.10.0 (rejected as the lower-level API), zip 8.6.0, tar 0.4.46, atom_syndication 0.12.10 — all pure Rust.
- The contract (`docs/decisions/2026-09-07_r2-extraction-contract.md`, top-level `answers:`): the extraction always produces a Derivation (parent digest + extractor version + derived chunk digests); the parsers run in `process`-class worker processes — the first ladder-up, the stdio quarantine with killing budgets; the named refusals (encrypted/JS PDFs, nested archives, traversal, bombs); the media-type routing (hinted references pipeline acquire→extract).
- No code. Frontier → `.4.2` (the extraction workers).
