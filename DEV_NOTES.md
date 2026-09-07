# DEV_NOTES.md

## _(2026-09-07)_ — PHASE-2.1.4.2: under delegation, the record binds the authority source, not the caller

- **The dual evaluation changed what "the grant" means in the audit row.** Before the leaf, `delegate_subject` was dormant (audit-only). Now the SUBJECT's grant is the authority source: the caller's own grant is checked independently (a non-holder cannot delegate), the record's `grant_id` + the policy digest bind the SUBJECT, and the pre-existing audit test had to move to the dual semantics (it seeded one grant; the new contract needs both).
- **The scope ladder is two subset checks, not one.** The request's target must be within the REQUESTED scope, and the requested scope within the subject's grant selector — the widening refusal names which leg failed.
- promotion: declined (the dual-evaluation semantics and the record-binds-the-subject rule are per-slice engine facts recorded in the leaf — no new cross-cutting decision). **Frontier `PHASE-2.1.5` (the cached-decision semantics).**

## _(2026-09-07)_ — PHASE-2.1.4.1: decide the representation from the plumbing that already exists

- **The census chose the ADR's answer in advance.** The delegation plumbing was pre-shaped (`delegate_subject` + the audit subject split) — chain-in-envelope rides it for free, while a capability token would add an issuance/store/signature lifecycle duplicating the `.1.3` grant filters that already revoke. The spike's job was to prove the invariant (a pure subset function) and measure the wire delta, not to re-litigate the shape.
- **A tagged-newtype enum is not a wire field.** `GrantSubject` derives Serialize with a tag; serializing `DelegationConstraints` containing it fails ("cannot serialize tagged newtype variant"). The envelope's `authority_context` must carry the subject as a STRING and parse it — the size probe surfaced the refusal the implementation would have hit.
- promotion: declined (the GrantSubject tagged-newtype wire note is a per-slice serialization fact for .1.4.2, recorded in the leaf — no new cross-cutting decision). **Frontier `PHASE-2.1.4.2` (the delegation implementation).**

## _(2026-09-07)_ — PHASE-2.1.3.2: a revoked ceiling must freeze writes, never the operator's eyes

- **The tests found the governance semantics, not the other way around.** The boundary-revocation test's FIRST run failed with the admin's own inspection list returning 403: the tenant_admin authorization evaluates against the active boundary, so revoking it refused everything — including the surfaces meant to prove the revocation. The fix is a named carve-out: admin READS authorize against the principal's own grant (no ceiling); admin WRITES stay ceiling-checked, so a boundary revocation is an honest freeze.
- **The freeze is the feature.** After a boundary revocation every grant under it — including the bootstrap human's — is refused at the next decision, and the tenant is read-only until a future superseding act (Phase 5's correction machinery). Recorded, not smuggled.
- Promoted to `docs/decisions/2026-09-07_boundary-revocation-freeze.md` (`answers:` present). **Frontier `PHASE-2.1.4` (the delegated authority context).**

## _(2026-09-07)_ — PHASE-2.1.2.2: when every valid proof is refused, isolate the legs before touching the crypto

- **The ladder probe turned a mystery into a named interop fact.** 16 channel tests refused 401 with all-valid certificates; the probe (chain / SPKI / self-SPKI / ring-only control / digest variants) isolated it in three runs: the chain verified, the keys matched byte-for-byte, a pure-ring control passed — and yet ring's `UnparsedPublicKey` refused rcgen's SPKI DER in every format. The fix: verify against the **bare EC point** (the path webpki uses internally — which is why the chain check always worked). Probe removed before commit; the fix + record remain.
- **A failing FIXED leg proves nothing about the ASN.1 leg.** `ECDSA_P256_SHA256_FIXED` expects a 64-byte raw r‖s; rcgen emits ASN.1 (70–72 bytes). Both FIXED probes were structurally invalid from the start — a good reminder that a probe variant must be a VALID test of its hypothesis.
- **The rotation contract kept a running session safe by being additive.** The rotate endpoint issues a fresh fingerprint WITHOUT retiring the old one; the node rotates at ≤50% lifetime and the fresh identity signs the NEXT handshake — nothing is cut mid-session.
- Promoted to `docs/decisions/2026-09-07_cert-proof-verification.md` (`answers:` present). **Frontier `PHASE-2.1.3` (revocation surfaces).**

## _(2026-09-07)_ — PHASE-2.1.2.1: a CA that must survive restarts belongs in the control plane's own store

- **The demo's kill point chose the storage.** The server is SIGKILLed and restarted mid-demo, so the CA cannot live in memory: `server_ca` is ONE row (id 1) the boot loads or generates — and the enrollment suite's rebuild test asserts two `ensure_server_ca` passes return the SAME key + cert (previously issued leaves keep chaining).
- **The token row is the issuance serialization point for free.** Enrollment was already exactly-once (token `FOR UPDATE` + `used_at`); issuing the leaf INSIDE that transaction inherits the guarantee — the replay refusal issues no second certificate, asserted in the suite.
- **rcgen 0.14's CA reloaders live on `Issuer`, not `CertifiedIssuer`** — `Issuer::from_ca_cert_der` (behind the `x509-parser` feature) rebuilds the signing issuer from the stored DER; the compiler's "no associated function" note (not the error) carried the answer.
- **The coherent interim is load-bearing**: the node now stores `cert.der`/`key.der` while the HMAC channel stays live — the demo passes with the files unused, so the v3 swap (`.1.2.2`) has a clean, green base.
- promotion: declined (the server-generated dev-escrowed node key is the .1.2.1 trust-store stance recorded in the leaf + ADR-007's honest limits — the Internet profile re-evaluates; no new cross-cutting decision). **Frontier `PHASE-2.1.2.2` (the channel v3 cert-proof swap).**

## _(2026-09-07)_ — PHASE-2.1.1: decide the issuer from a measured spike, not from the roadmap's candidate list

- **A spike's job is to make the refusal cases real.** The experiment drives a REAL TLS 1.3 client-cert handshake (rustls) and asserts the §16.2 contract: the trusted allowlisted leaf completes; foreign-CA, expired, and unregistered-fingerprint certificates are refused on BOTH sides; rotation is additive. Issuance latency N=200: p50 63 µs / p95 69 µs — cert issuance is effectively free at LAN scale, so rotation can be aggressive.
- **The bans doctrine caught a real dependency split.** `make deny` failed on two base64 versions (0.22 via hyper-util, 0.23 via rcgen's optional `pem` feature). The fix was dropping the UNUSED feature (the spike consumes DER only), not skip-listing — a skip entry would have hidden a choice the spike never needed to make.
- **rustls's blocking reader yields WouldBlock until `complete_io` decrypts.** The spike's first iteration passed the whole handshake and then failed the ping read — the fix is the drive-IO-until-readable loop; `.1.2`'s channel upgrade must carry the same pattern or reproduce the failure in product code.
- Promoted to `docs/decisions/2026-09-07_workload-identity-issuance.md` (`answers:` present). **Frontier `PHASE-2.1.2` (the certificate lifecycle + channel v3).**

## _(2026-09-07)_ — PHASE-1.8.2: a phase closes on evidence, not on a checklist

- **The gate package is a census, not a victory lap.** The `.1.8` pickup census mapped every §26.1 acceptance point to an EXISTING demo beat (real SIGKILL kill points) before any new work; the only gap was the audit-reconstruction claim (closed in `.1.8.1`), and the gate record cites per-row evidence — bundle files, guard logs, suites — never prose.
- **A deferral without an owner and a trigger is an omission.** The five named deferrals (capability advertisement → Phase 3; expected-artifact/decision-rule + synthesis → Phase 5; incarnation/run writers → Phase 2; fuzz → Phase 4; ops hardening → Phase 2) each carry the condition that revives them, and the subtraction record's lists are none empty.
- **"No database surgery" has one legitimate exception: credentials.** The demo's two psql reads obtain the fencing token to forge the duplicate transport as the node itself — a least-privilege API must not expose a live credential, so the oracle stays (documented in the script); every STATE assertion rides the CLI/API/console/`rb-journal`.
- Promoted to `docs/decisions/2026-09-07_phase1-gate-record.md` (`answers:` present). **Frontier `PHASE-2.1` (identity/recovery).**

## _(2026-09-07)_ — PHASE-1.7.2: prove the artifact, not only the behavior

- **Debug proves the code; the release-built demo proves the package.** The census found the demo builds `--bins` (debug) only, so the packaging claim had zero evidence. The cheapest honest proof: a `--release` flag that switches the build root — the script already had a build-root seam (`BIN_SERVER`/`BIN_NODE`/…), so the flag is a selection, not a fork; `make demo` stays debug.
- **A runbook without a practiced path is fiction.** The demo's ssh two-host mode existed before any runbook; `deploy/README.md`'s job is to route operators to what is already repeatable and state the honest limits at the point of use (dev trust store, plain HTTP — trusted LAN only).
- **Self-containment is the packaging invariant.** Migrations + the console embed at compile time, so a deployed binary needs no runtime path back to the checkout (§12); PostgreSQL and the node's journal volume are explicitly operator-owned.
- Promoted to `docs/decisions/2026-09-07_deployment-packaging.md` (`answers:` present). **Frontier `PHASE-1.8` (G1–G2 exit + Demonstration A).**

## _(2026-09-07)_ — PHASE-1.7.1: a dev loop is the test harness's boot machinery with a different lifecycle

- **Reuse the boot, change the lifecycle.** The ephemeral-PG machinery (initdb → on-volume `$ROOT/target/*.XXXXXX` → `pg_ctl` → createdb → trap cleanup, the `PHASE-1-MAINT-1` §13 shape) was authored for one-shot verification; the dev loop needs the same boot with a foreground server and interactive teardown. `scripts/dev.sh` copies the shape verbatim instead of inventing a second PG boot path.
- **A self-verification beat earns its keep before the first green run.** `dev.sh --check`'s first three runs caught: the list verb is `inspect threads` (not `threads`), the dev-profile CLI requires `--as <principal>` (the suppressed-stderr blind spot — the manual probe with stderr visible is the fix), and a residue census must observe the CLEANED state (teardown before the census, not after the EXIT trap).
- **A census over the filesystem is only true after the teardown it audits** — the trap's cleanup runs at exit, so the check tears down explicitly and the trap's re-run is a no-op.
- Promoted: declined (the §13 ephemeral-PG shape and the CLI dev-profile `--as` contract are already recorded facts; no cross-cutting decision). **Frontier `PHASE-1.7.2` (release packaging + LAN runbook).**

## _(2026-09-06)_ — PHASE-1.6.2: the shell's own tests enforce the page's honesty

- **A static page's safety properties are greppable — so grep them, in a test.** The offline contract test asserts what the page MUST be: it references only the documented GET surfaces, it names no write verb, and it never assembles HTML from data. The first run caught MY OWN app.js naming the forbidden API in a comment — reworded, so the assertion is honest (the page does not even name it).
- **Embedding is the "no build pipeline" that also keeps one binary.** `include_str!` + a state-free `ui_router` at `/`: no runtime paths (§12), no filesystem reads, no artifact class. A runtime `--web-dir` would reintroduce paths and drift.
- **The page is a client, not a surface.** Same-origin GETs with the dev-profile header + tenant query — every gate, denial, and audit row applies exactly as it does to the CLI; the `ui_router` adds zero API routes.
- Promoted to `docs/decisions/2026-09-06_ui-embedding.md` (`answers:` present). **Frontier `PHASE-1.6.3` (the demo/evidence leg).**

## _(2026-09-06)_ — PHASE-1.6.1: a read surface exposes ledger rows, it does not recompute them

- **The census question is "which inspection surfaces exist", and budgets answered: none.** The ledger tables were fully formed (`0005_budget.sql`), the engine enforced against them, and no GET/CLI verb touched them — the `.1.6` goal named a surface that did not exist. The `.1.6.1` child IS that census finding.
- **Pass-through beats summary.** `GET /v1/threads/{id}/budget` returns the ceiling + every reservation row (held vs settled usage, denials with reasons) — the same rows the engine writes; any computed "summary" would be a second authority that can drift. The test's first run proved the surface right and the TEST wrong (the row's denial reason is the engine's raw `detail`, `the ceiling does not cover …`, not the dispatch site's `budget denied the dispatch:` prefix — the latter rides the work item).
- **A new surface reuses the existing gate.** The endpoint rides `thread_inspect` (the `get_thread` path) — zero new authority, zero new write path, zero new tables; the page inherits the typed 403 + audit row.
- Promoted to `docs/decisions/2026-09-06_budget-read-surface.md` (`answers:` present). **Frontier `PHASE-1.6.2` (the embedded static shell).**

## _(2026-09-06)_ — PHASE-1-MAINT-3: an unpinned `stable` toolchain is a slow-motion formatter drift

- **`cargo fmt --check` failing on files the current leaf never touched is a toolchain defect, not a style slip.** The `.1.6.1` verification caught it: both installed rustfmt builds (`1.9.0-stable` 2026-04-14 from rustc 1.95.0 and 2026-08-18 from 1.98.0) flag the SAME pre-existing hunks — the tree's last full fmt run predates the stable-channel move, and recent leaves verified clippy but not fmt, so the drift sat undetected.
- **Pin the channel; never trust `stable` for formatting.** `rust-toolchain.toml` now pins `1.98.0` (the newest installed) and the CI toolchain inputs name it explicitly — reproducible rustfmt/clippy instead of whatever `stable` resolves to locally vs in CI.
- **The gate that would have caught this is `make check` (fmt first), not clippy** — clippy 1.98 was clean the whole time; only the fmt job saw the drift. A toolchain pin converts that from a surprise into a non-event.
- Promoted to `docs/decisions/2026-09-06_pinned-toolchain.md` (`answers:` present). **Frontier `PHASE-1.6.1` (the budget read surface).**

## _(2026-09-06)_ — PHASE-1-MAINT-2: a flake that reproduces once is a bug with evidence

- **The missing thing was the test's NAME, not its output.** The `.1.3.1` report lost it; the `.1.5.3` verification captured `nonzero_exit_produces_failed_known_with_the_stderr_tail` with an EMPTY tail — instantly a real race: the spawned stderr drainer had not consumed the pipe's tail when the EOF path snapshotted the buffer after `child.wait()`. Load widens the scheduling window; it never creates it.
- **Spawned consumers need a join point.** `drain_stderr` now returns its JoinHandle and the EOF path awaits it (bounded 5 s against a stderr-inheriting grandchild) before the snapshot. Fixed in `codex.rs` AND the `claude.rs` mirror — mirrors inherit defects.
- **The 10× loop is the confidence instrument** for a race fix: both adapter suites ×10 green + the full offline workspace, before the commit.
- Promoted to `docs/decisions/2026-09-06_stderr-drain-race.md` (`answers:` present). **Frontier `PHASE-1.6` (Web UI/CLI).**

## _(2026-09-06)_ — PHASE-1.5.3: a terminal outcome must be a machine fact

- **"Honest inconclusive" as prose would be unassertable.** The core machine gains `Inconclusive` (the `Closing → FinalizeInconclusive` edge, the exhaustive table + terminal-rejection tests extended — the canary pattern); the state IS the answer to "did this thread decide?", assertable by the demo and the audit alike.
- **The register is event content; the state is the outcome.** `unresolved` rides the close EVENT — no projection change (the `.1.5.1` event-layer pattern, now three leaves old). The close body's `outcome` picks the terminal.
- **Contradictory close bodies are refusals, not warnings.** `decided` + non-empty `unresolved` → typed 400: the dishonest case the feature exists to prevent. Every stored terminal is truthful by construction.
- **The flake finally reproduced itself — and it was real.** During this leaf's offline verification, `codex_adapter::nonzero_exit_produces_failed_known_with_the_stderr_tail` failed with an EMPTY stderr tail: the stderr-drain task hadn't consumed the pipe's tail when the EOF path snapshotted the buffer — a real race in the `.4.2` code (and its `claude.rs` mirror), load only widens the window. Repro + root cause recorded in `PHASE-1-MAINT-2`; the fix executes next.
- Promoted to `docs/decisions/2026-09-06_honest-inconclusive-close.md` (`answers:` present). **`.1.5` complete; next: `PHASE-1-MAINT-2` (the captured drain race).**

## _(2026-09-06)_ — PHASE-1.5.2: server-assigned facts cannot be forged

- **Assign at the boundary, never validate what the client named.** The round is server-assigned — contributions land in the CURRENT round and `thread.advance_round` is the only mover — so the "current or current+1?" validation ladder never exists. A client-supplied round would have created the mismatch class for no gain.
- **A process-shaping act deserves its own grant name.** `thread_advance_round` joined the registry with the canary extended FIRST (it failed until the canary row was added — the `.1.1.3` lesson as a pre-built habit); roles stay deny-by-default: they shape content, humans shape the process, and the 403 leg of the suite proves it.
- **Rounds are a projection fact, not a thread state.** No core state-machine change: `open` threads advance freely, and the round rides the contribution event exactly like `.1.5.1`'s kind — the event-layer growth pattern, re-applied.
- Promoted to `docs/decisions/2026-09-06_rounds.md` (`answers:` present). **Frontier `PHASE-1.5.3` (the honest close).**

## _(2026-09-06)_ — PHASE-1.5.1: type the body, not the plumbing

- **A typed default is a documented variant, not an empty profile.** `kind` defaults to `position` (a contribution without a kind IS a position) — the same shape as `.1.1.3`'s `single_agent`; the tests assert the default so the wire contract is enforced, not just stated.
- **Content lives in the event log; the projection keeps counts.** Typing the contribution body needed NO projection change: the event body is the record, so the additive growth happens at the event layer, and pre-`.1.5.1` stored projections parse by construction.
- **References are a Phase-1 fact, acquisition is a Phase-4 act.** `evidence_refs` stores what the contributor CLAIMS to cite (URI + optional digest, deny-unknown at the ref itself — a foreign field like `password` is refused); nothing in `.1.5.1` fetches or validates — §3.7's honest split.
- **The kebab lesson re-applied from the start.** The CLI normalizes `--kind evidence-reference` to the wire's `evidence_reference` (`replace('-', "_")`) and the e2e drives the kebab spelling through the REAL binary — the `.1.1.3` first-run failure is now a pre-built habit, not a re-learned one.
- Promoted to `docs/decisions/2026-09-06_structured-contributions.md` (`answers:` present). **Frontier `PHASE-1.5.2` (rounds).**

## _(2026-09-06)_ — PHASE-1.4: probe the wire before coding the wire

- **Three bounded live dispatches replaced a guessed event shape.** Before writing `claude.rs` I ran `claude -p --output-format stream-json` with and without `--verbose`, plus one deliberate refusal: the no-verbose probe failed with the CLI's own error ("stream-json requires --verbose") — the REQUIRED flag was discovered by the tool itself, not by reading prose. Probe evidence on-volume in `target/claude-probes/` (the /tmp originals deleted, census-verified — §13).
- **A variadic flag eats the prompt.** `--tools ''` swallowed the positional prompt on the first probe ("Input must be provided either through stdin or as a prompt argument") — which taught the `--` separator the adapter's `EXEC_ARGS` now ship, and the stub suite pins (the prompt is always the LAST arg).
- **Receipt shape is adapter-specific, the contract is not.** Claude reports MONEY (`total_cost_usd`) and pre-folded token counts; Codex reports tokens only and needs a reasoning fold. Both normalize onto the same `NormalizedUsage` — the differences live in each adapter's `normalize_usage`, never in the contract.
- **"Resume" is not "query".** `claude --resume` continues a session; it cannot prove a past attempt's outcome — `query_status` stays `Unsupported` and a lost response stays `outcome_unknown`. The same honest leg as Codex.
- **The live qualification passed FIRST TRY on the real harness** (`RB_LIVE_CLAUDE=1`, 1 passed in 1.91 s: completed, exact usage + money cost, session id attached as the provider handle) — the reward for probing first; the `.1.4.1` offline suite also caught its own test-authoring slip (the multi-chunk assertion) on its first run, fixed before commit.
- Promoted to `docs/decisions/2026-09-06_claude-cli-adapter.md` (`answers:` present). **`.1.4` complete (Codex + Claude + the deterministic fake); frontier `PHASE-1.5` (structured contributions).**

## _(2026-09-06)_ — PHASE-1-MAINT-1: same-volume locality is re-derived per tool, not inherited

- **A policy adoption does not reach backwards into pre-existing tools.** `run_pg_tests.sh` kept defaulting its ephemeral PG cluster to `${TMPDIR:-/tmp}` after §13 landed — the script predated the adoption and no reader re-derived its temp data from the repo root. The fix is the runtime `ROOT` derivation plus the one-line data-dir change; the defect leaf made the re-check itself the work item.
- **The repo root is the runtime authority, not the caller's CWD.** `ROOT` comes from the script's own location, so the suite behaves identically from any directory — persisted paths are repo-root-relative (§12) and absolute only at runtime (§13).
- **Move the data, keep the mechanics.** `mktemp`'s per-run uniqueness and the cleanup trap are untouched — only the parent moved (`$ROOT/target/`, gitignored). Evidence: a 2 s poll observed the cluster at `target/pg-ephemeral.BPJbkS` during the run (~4 s in; a one-shot 25 s probe of the first run missed it — timing noise, so the second run polled), and the post-run residue census left nothing on either volume.
- Promoted to `docs/decisions/2026-09-06_same-volume-pg-ephemeral.md` (`answers:` present). **Frontier `PHASE-1.4` (second real adapter — director decision pending); MAINT-1 closed.**

## _(2026-09-06)_ — PHASE-1.3.2: compatible transitions are not a serialization bug

- **The race test taught a DOMAIN fact before it proved the lock.** My first shape raced accept vs remove and BOTH returned 200 — the suite's own assertion caught it. They are COMPATIBLE transitions: the serialized order accept-then-revoke is legitimate, the snapshot is `revoked` either way, and a revoked role's late result folds to a stored rejection. The conflict pair is accept vs decline (both consume the same pending offer) — rerun green. A race test must first establish which transitions the domain declares conflicting; asserting exactly-one-winner on a compatible pair asserts a fiction.
- **A typed rule becomes doctrine the day a boundary enforces it.** `allow_join_requests`/`allow_explicit_invites` sat recorded for several leaves; the refusals landed here. Until the enforcement test exists, a rule field is a suggestion — the `.1.1.3` honest-limits should have said so louder.
- **The self-request path needs no reservation machinery.** An invitation reserves a slot; a join is admitted or refused in one command. Adding join-tokens for symmetry would have been scope creep.
- Promoted to `docs/decisions/2026-09-06_join-subscriptions.md` (`answers:` present). **`.1.3` complete; frontier `PHASE-1.4` (second real adapter).**

## _(2026-09-06)_ — PHASE-1.3.1: a capability and a grant answer different questions

- **The invitation (offer/reserve) is the capability; the grant is the gate.** Accept/decline authorize against the PENDING invitation naming the actor AND the new `thread_invitation_respond` grant the role default carries. Grant-only would hand the acceptance right to non-invitees; invitation-only would bypass the audited authorization flow. Two checks, two layers, one command.
- **Don't split a contract across leaves when the interim is incoherent.** The lifecycle (invited roles may not act) and the dispatch move (work rides accept) only hold together — my own decomposition separated them, and the contradiction surfaced while scoping: work would still arrive to a role that cannot accept it, the suites red between commits. Amended same-day into ONE leaf. Decompositions are hypotheses; a contradiction is an amendment, not a workaround.
- **Derived expiry needs no event.** An expiry event would have to ride SOME command's transaction — but the command that observes expiry (accept) is refused, and refused commands commit nothing. Deriving at read + enforcing at the boundary gives observability AND enforcement without a sweeper: the same shape as channel-lease presence, now the repo's third instance (leases, presence, invitations).
- **Race tests assert the SNAPSHOT, not a winner.** Concurrent accept/remove has no predetermined winner (the aggregate head lock serializes — whichever lands first). The test asserts exactly one 200, exactly one transition event, the snapshot matches the winner, and the spent invitation refuses a late accept either way.
- **Adding an audited verb shifts every audit-timeline assertion.** The first full run failed in exactly one place — command_api's expected audit list gained the accept's `thread_invitation_respond` record. That is the audit trail doing its job; the fix is the expectation, not the code.
- Promoted to `docs/decisions/2026-09-06_explicit-participants.md` (`answers:` present). **Frontier `PHASE-1.3.2` (simple subscriptions).**

## _(2026-09-06)_ — PHASE-1.2.3: quarantine is a row fact, and a measured prune beats a background sweep

- **A quarantine that lives in the handler would be a promise; a quarantine on the row is an invariant.** Two nullable columns + a `quarantined_at IS NULL` filter in BOTH delivery paths (handshake replay and live poll) make "never re-delivered" true for every path that exists — a future third path inherits the filter by construction, not by remembering to check a flag.
- **Delivery-control is not result-suppression.** A node that received a command BEFORE its quarantine may still return a result; the domain applies it. Splitting those semantics keeps quarantine scoped to what it can honestly promise (the inbox), and keeps `load_command` unfiltered on purpose — documented, not accidental.
- **A measured before/after IS the dry-run.** The prune's count/delete/recount ride one transaction, so the response is the operator's receipt; a separate dry-run toggle would only duplicate the measurement. No background sweeper — retention is a decision with an operator in the loop.
- **Audited operator actions ride the AUTHORITY engine, not a new audit table.** Quarantine and prune reuse `authorize()` (tenant_admin): allowed AND denied leave an authorization record. The `.1.2.1` enrollment needed its own table only because the node-side enroll carries no principal header — when there IS a principal, the engine is the audit.
- **The `.1.2.2` closure lesson re-applied immediately:** the new suite's request-builder closures took a borrowed `&str` and broke on the unnameable-lifetime error before the first run; owned params fixed it. Recorded twice, applied once, now a habit.
- **Two test-side sqlx slips, both caught by the live run, not the compiler:** the seed-tenant row was missing (FK 23503 — the same shape `node_channel.rs` already solved with an `ON CONFLICT DO NOTHING` seed insert), and a `query_scalar` count was annotated `(i64,)` (a RECORD decode error — `query_scalar` wants the bare scalar type). Both are per-suite boilerplate mistakes, not server defects; the server code was correct on the first run.
- Promoted to `docs/decisions/2026-09-06_node-inbox-retention.md` (`answers:` present). **`.1.2` complete; frontier `PHASE-1.3` (invitation/subscription semantics).**

## _(2026-09-06)_ — PHASE-1.2.2: the fence rotates at the handshake; presence is derived, never stored

- **A missing field and a wrong credential are different refusals.** `serde`'s strict wire boundary (required fields + `deny_unknown_fields`) rejects a MISSING `key_proof`/`fencing_token` as 422 before the handler runs; a WRONG value reaches the verifier and gets 401. My first test drafts expected 401 for both — the wire contract is under-specified unless both statuses are asserted.
- **Rotation and renewal are different operations with different authority.** The handshake owns identity re-proof (new key-proof → fresh `fnc_…` token), the heartbeat owns liveness (extends the expiry, echoes the token). Conflating them would let a stolen heartbeat credential escalate into a fresh identity grant.
- **Presence must be a derived fact, not a stored flag.** `node_presence` computes `online` from `lease_expires_at` — a crashed process cannot leave a stale `online` row, and the fencing token (a credential) is never exposed by the observability surface.
- **Cross-side crypto is mirrored, not shared.** Server and node each serialize their own `ProofCoverage`; the wiring suite computes proofs with the node's public `compute_key_proof` against the server's verifier — a shared wire crate would have hidden any drift between the two canonicalizations.
- **Suite purge lists are all-or-nothing.** Purging `tenants` in the channel suite failed until the list covered every tenant-referencing table (`human_principals` left by an earlier suite in the same run); a suite that owns a table owns everything referencing it, in FK order.
- **The authenticated handshake exposed a latent `.1.2.1` strictness:** issuance/enroll accepted only `nod_…` while the dev wiring's node id IS the `rol_…` role wire id (the demo's own contract). The identity space now accepts both — a superset, so the `.1.2.1` suites never moved.
- Promoted to `docs/decisions/2026-09-06_node-channel-auth.md` (`answers:` present). **Frontier `PHASE-1.2.3` (durable inbox retention + quarantine).**

## _(2026-09-06)_ — PHASE-1.2.1: one-time tokens are a row property, and refusals are rows too

- **"One-time" lives in the token row, not the handler.** Enrollment serializes on `FOR UPDATE` + a nullable `used_at`: a second use is impossible at the database level, and the handler only maps the row state to a typed, audited refusal. Binding the token to node id + host claim + nonce means a stolen token cannot enroll a different identity.
- **A refusal that must be durable rides the denial-row pattern** (budget-engine precedent): the audit row commits in the caller's transaction BEFORE the error returns — refusals are data, not just responses. The new suite asserts four refusal classes each left exactly one audited row and zero identity rows.
- **sqlx's `Transaction::commit(self)` consumes the transaction** — a helper cannot commit the `&mut Transaction` it was passed (E0507 + the `self` signature). The working shape: validate → write the audit row on the borrowed transaction → commit once in the caller → return the typed error. My first two drafts fought this (a committing helper, then a borrowing closure) — the third is the budget engine's own shape.
- Promoted to `docs/decisions/2026-09-06_node-enrollment.md` (`answers:` present). **Frontier `PHASE-1.2.2` (authenticated channel + lease/presence).**

## _(2026-09-06)_ — PHASE-1.1.3: the typed default is part of the contract

- **An enum-shaped wire field stays honest when its default is a documented variant, not an empty profile.** The create profile landed as serde enums with `#[default]` variants: unnamed means exactly `general` / `single_agent` / explicit-invites-only — and the tests assert those defaults, so the ADR-002 single-agent decision is enforced on the wire, not just stated in prose.
- **A registry grows by naming the entry in the one place the registry lives and letting the enumerating test fail first.** Adding `thread_cancel` to `GrantAction` broke the wire-name round-trip test until it was extended — the registry's own test is the canary that a new authority name was not forgotten.
- **Two terminals must not share a reason field.** Cancel and close are separate events with separate `cancel_reason`/`close_reason` projection fields; the API test asserts `close_reason` stays null on cancel — a cancelled thread that answered "why did it close?" would be lying.
- Promoted to `docs/decisions/2026-09-06_thread-api-completion.md` (`answers:` present). **Frontier `PHASE-1.2` (node: SQLite journal + enrollment + lease/presence + durable inbox).**

## _(2026-09-06)_ — PHASE-1.1.2: the identity store — the table is the record, the FK is the enforcer

- **The identity table is the record; the enrollment table is the map.** Migration 0007 adds `tenants`/`human_principals`/`agent_roles`/`hosts`/`nodes`/`incarnations`/`runs` beside the `.6.1` `enrollments` table without upgrading the map into the schema — the identity tables model §8.1 exactly, and the dev name→id map stays disposable.
- **Parent-row-first inside one transaction, and the FK enforces the order.** `enroll` inserts the tenant row before the principal row; a future caller that forgets the order gets a failed transaction, not a comment to remember. `identity_store`'s fail-closed test proves it: a principal with no tenant row (and a node with no host row) is refused by the database.
- **A re-enroll is a replay at the identity layer too** — the replay path returns before any insert AND the identity tables carry their own unique keys, so idempotent bootstrap is double-enforced (count assertions after re-enroll prove no second row).
- Promoted to `docs/decisions/2026-09-06_identity-store.md` (`answers:` present). **Frontier `PHASE-1.1.3` (thread command API completion).**

## _(2026-09-06)_ — PHASE-1.1.1: the aggregate/event/outbox library is an extraction, not a rewrite

- **A proven write path extracts cleanly when the old shape becomes a shim that owns NO SQL.** `tx.rs` shrank to type conversions + delegation over `agg` (claim → locked head → event → state → outbox → result, one transaction); every Phase 0 caller kept its exact `tx::` shape, so the zero-behavior-change acceptance is proven by switching no call site and re-running the full regression — the live-PG suites AND the two-host demo rode the new library with every acceptance check green.
- **The revision precondition defaults OFF and stays honest.** `AggregateCommand::expected_revision` adds optimistic concurrency (`Some(n)` requires the head at n; a fresh aggregate is revision 0) without touching existing behavior — the shim passes `None` and its impossible-arm `unreachable!` turns a future drift into a crash instead of a silent divergence. `apply_fresh_in_tx`'s doc now names the fresh-aggregate case explicitly: `FOR UPDATE` takes no row lock when the row doesn't exist, so the first write's serialization point is the primary-key insert.
- **The library is the durability spine, not the domain.** Authorization (`authority`) and validation (`threads`) compose OVER it inside one transaction — the modular-monolith pattern made explicit, and the ADR names the extraction trigger (a measured boundary need) so a separate store crate stays forbidden until measured.
- Promoted to `docs/decisions/2026-09-06_aggregate-library.md` (`answers:` present; ADR-004 records the pattern decision). **Frontier `PHASE-1.1.2` (migration 0007 identity store).**

## _(2026-09-06)_ — ReasonBraid-only naming: 90 scaffold-name tokens swept from 28 files

- **Census before reword, always.** (case-insensitive `git grep` census over the scaffold-name token) → 90 occurrences in 28 tracked files: the template's own name had survived the bootstrap in provenance comments (the scaffold tracker ids), the version file, the scaffold-pull tooling, and the landing page. An ordered token map (compounds first, bare tokens last) plus prose polish removed every one; the facts survived (`REASONBRAID-MAINTENANCE.N` ids, `reasonbraid-scaffold 0.6.1` version string). A naked sed for the bare token first would have mangled the compounds and the crate names.
- **A guard's fixture must mirror the docs it guards.** The README-STABILITY self-test exercised the scaffold-URL span; when the README moved to `<reasonbraid-url>`, the fixture moved with it — a self-test asserting a placeholder the landing page no longer uses teaches the wrong lesson.
- `promotion: declined (the directive and its census are recorded in the MAINT-2 leaf; no durable cross-cutting fact beyond the rebrand)`. **The PHASE-0 tree is complete — next executable work: `PHASE-1.1`.**

## _(2026-09-06)_ — README_POLICY re-adoption: the closure leg caught real destinations on its first run

- **The routing-pressure-closure leg reproduced the upstream cautionary tale in miniature.** The moment the guard actually censused the tree it flagged three genuinely unrouted destinations — `COMMIT.md`, `docs/adr/001-uncleared-working-name.md`, and the scaffold-URL placeholder inside the scaffold span — plus a real measured legacy ceiling (`CHANGELOG.md` at 48,495 bytes against a provisional 10,240). A guard that had never been asked the question could never have caught them; the upstream policy's 1,547,057-byte neighboring sink starts exactly this way.
- **Derived caps beat template defaults.** 300 lines / 16,384 bytes was meaningless for a 47-line landing page; 60 / 2,400 is the reviewed survivor plus explicit headroom, and raising it now requires a task-tree decision — the cap became a contract instead of folklore.
- **BSD `cut` on a no-delimiter line prints the WHOLE line (GNU prints empty).** The control-field census initially swallowed the registry's comment lines and flagged `README.md` / `scripts/check_readme_stability.sh` as unrouted destinations. Fix: `grep -v '^#'` before the field cut. Portability lesson for every future bash guard.
- Promoted to `docs/decisions/2026-09-06_readme-policy-readoption.md` (`answers:` present). **Frontier `PHASE-0-MAINT-2` (scaffold-reference cleanup, director directive).**

## _(2026-09-06)_ — Phase 0 exit gate closed: the owner signs the go record, the agent records it

- **The signature closes a gate that only the owner can close.** ADR-002 moved `proposed` → `accepted` on the accountable owner's explicit session decision ("Sign ADR-002 (GO) now"), and the ADR's signature line records WHO signed and WHEN — the agent drafts and records; the signature itself is the owner's act. KICKOFF §7's last item ("a named owner signs a go, rework, pivot, or stop record") is now satisfied, so Phase 0 formally exits and the PHASE-1 tree opens at `.1`.
- **Tree states carry the handoff, not chat.** PHASE-0's frontier became `MAINT-1` (executing next), PHASE-1 flipped `proposed` → `active` with `.1` unblocked, and LIVE_STATUS gained a Phase 1 row — a fresh session reads the same next-action from the durable layers with zero conversation context.
- **A dating anomaly surfaced and was recorded, not rewritten.** The host clock and git commit timestamps say 2026-09-06; the previous session's records carry 2026-09-07 dates (filenames and changelog entries inside 2026-09-06 commits). Today's records use the machine-consistent 2026-09-06; the anomaly is flagged to the director rather than renamed (history is immutable; re-dating committed records is churn with no corrective value).
- `promotion: declined (the acceptance is recorded IN the ADR itself — docs/adr/002-phase1-scope.md status + signature; no separate cross-cutting fact beyond it)`. **Frontier `PHASE-0-MAINT-1` (executing).**

## _(2026-09-07)_ — WP8 gate package: the subtraction record is the architecture ratchet's counterweight

- **The SubtractionRecord forced honest accounting of what Phase 0 did NOT do.** §19.8's shape turns "we didn't get to X" into a decision with a revisit trigger. The deferrals that matter most: the authenticated streaming channel + workload identity (revisit: any non-loopback exposure), the second real adapter (revisit: Phase 1), and the shared wire crate (revisit: a second consumer — the per-side `deny_unknown_fields` duplication is deliberate until then).
- **The 2×-estimate gate is arithmetic, not vibes.** Phase 0 measured ≈ 9.5 engineer-weeks against the roadmap's 8–14 range — no review triggered — but the number is now written down where a future phase can compare against it (§20.1.2: "re-estimate from measured throughput").
- **Two new risk rows came straight out of the `.7` real run** — provider run-to-run variance on identical prompts (code-002 single: 1.000 → 0.667 across runs) and ~16k ambient input tokens per real call. Both were observable only because the harness recorded ACTUAL usage and kept per-case scores; an average-only report would have hidden both.
- **The go decision is drafted, not self-signed.** ADR-002 is `proposed` with the GO recommendation and a pending signature line — the accountable owner (the director) signs; an agent drafting the package must not close its own gate.
- No new code in this leaf (documents + registers only) — the TASK-ACCEPTANCE boxes record that the gate's evidence comes from the WP1–WP7 suites, not from new tooling. **Frontier: exhausted; awaiting the signature.**

## _(2026-09-07)_ — WP7 benchmark: the first real run falsified the harness before any claim could ride on it

- **The scripted oracle CANNOT see prompt-wiring bugs — the real run can.** The critique/revise workflow rendered the SAME template for the critique and the revision leg, so every revision call was instructed to critique: all four real `critique_revise` rows came back with NO confidence line (`structure_valid: false`) and fact-001's "revision" broke a correct answer (1.0 → 0.0). The scripted agent answers by ROLE and never reads the prompt, so the corpus self-test stayed green through the whole defect. Lesson: prompt wiring needs a prompt-level check — the corpus now carries `the_critique_and_revision_prompts_are_distinct` (distinct templates, role-naming instructions), and the workflow renders `critique`/`revision` separately.
- **A trap that flags the question's own echo is a false positive machine.** The honesty trap (any digit in the answer) flagged a refusal that merely quoted "2026" back from the statement. Now it flags only numbers NOT present in the statement — still deterministic, no longer self-defeating.
- **Real cost accounting surprised us in a good way to have measured**: each `codex exec` call carried ~16k input tokens of ambient overhead (the user's Codex config), independent of the benchmark's ~100-character prompts — the H6 accounting would have been fantasy without recording ACTUAL usage. The harness records provider-reported tokens, so the overhead is visible instead of assumed away.
- **The benchmark's own verdict on itself was negative-or-null on this sample** (single agent matched or beat the structured workflows on the four differential cases at 1× the calls) — that is the WP7 acceptance's point: it narrows the routing claim for the WP8 memo rather than decorating it. See `docs/evidence/2026-09-07_benchmark-codex-run.md`.
- Promoted to `docs/decisions/2026-09-07_deliberation-benchmark.md` (`answers:` present). **Frontier `.8`.**

## _(2026-09-07)_ — WP6 node wiring: three bugs the demo and the suite caught before they shipped

- **The live suite caught a domain-semantics inversion in the first dispatch draft.** The revise work item originally carried the CHALLENGED CONTRIBUTION's event id as its target; the domain's `thread.revise` targets a CHALLENGE. The test failed with the server's own `invalid_command: revision target … is a contribution, not a challenge` — the contribution id is only the author-lookup key, the challenge's own event id is the revise target. Fixed; the assertion now checks the revise work item targets the challenge event.
- **`$$` inside a `( … )` subshell is the SCRIPT's pid, not the subshell's.** The demo's first pidfile scheme recorded the script's own pid — `node_kill` SIGKILLed the demo itself, the cleanup trap died before killing the server, and a later run hit `AddrInUse` with five leaked processes. Now: local nodes capture `$!` of the directly backgrounded binary; remote nodes capture the remote `$!` via `nohup … & echo \$!`. Every kill is followed by a `wait` reap (also silences bash's `Killed: 9` job banners).
- **`wait_for` under `set -e` is a footgun.** A probe timeout returning 1 aborted the script before the FAIL summary could print. Timeouts now record the FAIL and return 0 — the summary exit status decides.
- **A test-harness purge race, same class as `command_api`'s correct pattern.** The first `node_work` run failed `active == 1` because `pool()` purged the shared tables BEFORE the suite mutex was acquired, so a parallel test in the same binary purged rows mid-test. Guard first, purge second — matching `command_api`, which already had it right.
- **JSON shape assumptions bite in demo scripts.** `rb inspect thread --json` nests the events list one level deep (`.events.events[]`, the wrapper of three API views), and `--json` is PRETTY-printed — raw `"state":"closed"` greps fail; the checks now allow optional whitespace (`grep -Eq`). `jq` became the extraction tool of record (documented dependency of the demo).
- **Two independent dedupe layers, both exercised.** The duplicate-transport leg proves the `node_events` receipt dedupe (`accepted:false`) AND the idempotency-claim replay (same work result under a NEW event id still yields exactly one contribution) — the demo re-POSTs the node's ORIGINAL submission reconstructed from `rb-journal events`.
- Promoted to `docs/decisions/2026-09-07_node-channel-wiring.md` (`answers:` present). **Frontier `.7`.**

## _(2026-09-06)_ — WP6 control API: the subset checker caught the bootstrap bug, and rejections became idempotent results

- **The `.5.1` temporal subset rule caught THIS leaf before it shipped.** The first live run of the enroll bootstrap failed: a role grant created microseconds after its boundary "outlived" it (`grant.expires_at > boundary.expires_at`), the same wall-clock-skew class the `.5.1` fixtures exposed. Fix: dev grants are COEXTENSIVE with their boundary's validity window (`valid_from`/`expires_at` copied from the boundary) — and the failure itself is the evidence the checker binds.
- **Rejections are the command's semantic result, stored for replay.** A denied or domain-refused command stores `{"ok": false, "error": {code, message}}` in the idempotency row, and a replay reproduces the ORIGINAL status (stable code→status map) and body. This required the `tx` split — claim FIRST, then authorize/validate/apply — because a replay must return the original result WITHOUT re-validating against state the original command may have since changed (a replayed contribution after close must not fail).
- **The `FOR UPDATE` read is the consistency trick.** The domain validation reads the projection with `FOR UPDATE`; `apply_fresh_in_tx` re-reads the SAME row in the SAME transaction — so the version derived for the write can never diverge from the state validated. No check-then-write race, no second locking scheme.
- **The e2e run caught a classic URL bug the unit layer could not.** The CLI's inspect joined `/events` AFTER the query string (`?tenant_id=…/events`), corrupting the tenant param — the real-binary suite failed loudly with the server's own `invalid_command`. Lesson: the e2e suite earns its place by exercising the actual bytes the binary sends.
- **Clippy's `too_many_arguments` struck the verb runner (8/7)** — grouped into `ThreadVerbArgs`, the same class as `.5.1`'s `policy_digest` fix. And `clone_on_copy` hit `BudgetDimensions` (it derives Copy) — removed the clone, kept the one `String` clone the projection needs.
- **Axum's Json extractor answers forged fields with 422**, not the handler's 400 — the `.3.2` channel convention; the test asserts the 422 + the serde rejection naming the field.
- Promoted to `docs/decisions/2026-09-06_control-api-cli.md` (`answers:` present). **Frontier `.6.2`.**

## _(2026-09-06)_ — WP5 budget: one invariant, two ledgers, and a mandatory parameter that audits its own refusals

- **The acceptance is one sentence enforced twice:** "no provider dispatch without an applicable reservation" — the SERVER refuses to issue what the ceiling cannot cover (with a denial ROW), and the NODE refuses to dispatch what it has not been issued (journaled `failed_before_dispatch`, adapter never invoked — proven with a counting adapter). Two ledgers, one invariant (§14.3 step 4 is a LOCAL check by design).
- **Fail-closed coverage caught its own doc lie.** The first `covers` shipped with a doc comment claiming untracked dimensions "impose no constraint" while the code denied them; the tests exposed the contradiction and fail-closed was pinned. A ceiling that does not meter a dimension cannot vouch for it — period.
- **Refusals are results, not errors.** A refused dispatch returns a `FailedBeforeDispatch` report with the reason journaled as evidence — the attempt trail is complete for what did NOT happen. This matches `.5.1`'s denial-row philosophy (the audit covers refusals).
- **Indeterminate attempts keep their hold** (§14.6: release only amounts not potentially consumed). This cost the supervisor a deliberate asymmetry: pre-dispatch refusals release, completions settle actual usage, ambiguity holds — and the hold is the signal that adjudication is still owed.
- **Patch surgery on tests is a smell.** Mass-editing call sites with regex + helper insertion produced THREE distinct mangling rounds (nested helpers, dropped parens, misattached `#[tokio::test]`). The lesson: when a signature change touches many call sites, edit the files directly and compile after each file — not regex-batch then fix-forward.
- Promoted to `docs/decisions/2026-09-06_budget-reservation.md` (`answers:` present). **WP5 complete; frontier `.6.1`.**

## _(2026-09-06)_ — WP5 authority: the subset checker was more precise than the fixtures, and that is the point

- **The temporal subset rule caught the fixtures before they caught it.** The first live run failed 6/9: every grant "outlived its boundary" because each fixture helper read its own `Utc::now()` — a grant built microseconds after its boundary exceeded the window by those microseconds. A wall-clock-skew bug class that a weaker checker would have shipped silently; the fixtures now use wide boundary windows, and the failure itself is the evidence the rule binds.
- **Serde's tagged enums cannot wrap a sequence in a newtype variant** — `TargetSelector::Threads(Vec<ThreadId>)` cannot serialize (`cannot serialize tagged newtype variant containing a sequence`). Struct-like variants (`Threads { threads }`) fix it. A rule to internalize: any tagged enum variant holding a Vec must be struct-like.
- **`should_implement_trait` earned its keep again** — four authority `from_str` helpers became real `FromStr` impls with a shared `UnknownAuthorityName` error (the same lint that shaped `ProviderAttemptState` in `.3.1`); and `policy_digest` went from 8 params to 6 by passing the boundary struct (clippy's `too_many_arguments`).
- **The sqlx executor-shape split is real:** `&PgPool` and `&mut Transaction` satisfy `Executor` differently, so a shared loader abstraction fights the type system. The pragmatic shape: pool-based loaders for the public paths, INLINED lookups in the transactional path, and `apply_command_in_tx` as a generic `E: DerefMut + for<'c> &'c mut E::Target: Executor<'c>` (the `.2.1` body extracted with its public signature untouched — its 5 tests stayed green through the refactor).
- **Denials are audited events.** The acceptance reads "every command records … decision" — a refused command commits its denial record (reason + digest) and applies NOTHING; the audit trail is complete for what did NOT happen, not just what did.
- Promoted to `docs/decisions/2026-09-06_authority-boundary.md` (`answers:` present). **Frontier `.5.2`.**

## _(2026-09-06)_ — WP4 first real harness: the boundary that REVEALS its handle in the stream, and the lookup that honestly does not exist

- **The acceptance's honest leg was designed to be exercised by a REAL adapter — and Codex exercised it.** `codex exec` has no first-class status query for a past attempt (`exec resume` CONTINUES a thread and bills again; it is not a lookup), so `query_status` is `Unsupported`, a lost response lands `outcome_unknown` with no retry language, and the streamed thread id stays attached as the proof handle an operator would adjudicate with. No capability was fabricated to make the demo prettier.
- **Providers reveal request handles at different times.** The contract's `DispatchAck` carried the handle "when known"; Codex reveals its thread id in the stream's FIRST event, after dispatch. The contract gained `AttemptEvent::ProviderRequestId`, and the supervisor attaches streamed handles exactly like ack-carried ones. The ack ≠ completion acceptance now has its sharpest proof: the ack carries NOTHING, the handle arrives later, and the result later still.
- **The stub boundary caught a mis-wiring exactly as it should.** The first stub branched on `$1` — which is `exec`, not the prompt — so every scenario misbehaved and the suite failed loudly. The lesson is the same as the PG-queue lesson: test doubles must re-derive their inputs the way the REAL boundary receives them (here: the prompt is the LAST argv of `codex exec …`).
- **Live dispatch is one gated command away, never accidental:** `RB_LIVE_CODEX=1 cargo test … -- --ignored`. Default CI never spends a token; the ledger's revalidation trigger (CLI release) and the release gate both re-run it deliberately.
- **The vendor boundary stayed vendor-free:** no Codex DTO entered core; the only core-touching change across `.4.1`+`.4.2` is the one proof-gated machine edge from `.4.1`. Credentials: none — the adapter has no credential field; Codex uses its ambient login.
- Promoted to `docs/decisions/2026-09-06_real-adapter-codex.md` (`answers:` present) + `docs/evidence/2026-09-06_codex-adapter-qualification.md`. **WP4 complete; frontier `.5.1`.**

## _(2026-09-06)_ — WP4 adapter boundary: the conformance corpus caught the boundary-vs-refusal conflict, and two probes caught the rest

- **The corpus earned its keep on the FIRST replay.** `fail_before_dispatch` failed the moment it met the supervisor: the `.3.1` rule journals `dispatched` BEFORE `invoke` (conservative, crash-safe), but the machine had no edge to record the adapter's certified "no dispatch ever began". The fix is a proof-gated correction edge — `(dispatched, fail_before_dispatch) → failed_before_dispatch` — the exact inverse of the §11.3 lookup-proof edges, and the `.3.1` record's philosophy holds: only PROOFS move the machine, never guesses.
- **Two more real bugs, probed not guessed** (the ack test hung twice, with different causes): (1) `Notify::notify_waiters` loses a wake if the waiter has not registered yet — a scheduling race invisible without stress; `notify_one` stores a permit and is the correct primitive for one-shot signals (used in the fake's hang-cancel AND the test double). (2) The supervisor looped past terminal events — a stream yielding `Completed` repeatedly spun forever; the loop now breaks on the FIRST terminal event. A stream is not a source of multiple results.
- **The indeterminate outcome is an error, not a success.** `execute_attempt` returns `Err(OutcomeUnknown)` with the attempt journaled `outcome_unknown` — and its Display deliberately contains no "retry" (a test asserts the absence). Retrying ambiguity needs duplicate-risk authorization (§14.6); the boundary never volunteers advice.
- **The ack ≠ completion acceptance is proven AT the journal boundary**, not by assertion: a signaling test adapter pauses between the dispatch ack and the result, and the test observes the attempt durably `dispatched` in the journal in that window.
- **Credentials are enforced mechanically, not by convention:** the corpus's credential-shape scan (api_key/secret/password/credential/bearer) is a red test — a fixture with a credential fails CI. The contract has no credential field at all.
- Rejected and recorded: `async-trait` (native `async fn` in traits + documented `#[allow(async_fn_in_trait)]`), sleeps for the hang (Notify instead), parsing provider output in the adapter (chunks are opaque), a `cancelled_known` state (a confirmed cancel still leaves the result unknowable — honest `outcome_unknown`), and any blanket retry helper.
- Promoted to `docs/decisions/2026-09-06_fake-adapter.md` (`answers:` present). **Frontier `.4.2`.**

## _(2026-09-06)_ — WP3 node channel: the node reports what it durably holds; the server replays the tail; reconciliation gates schedulability

- §17.4 steps 1–7 became a protocol: the node reports its resume facts (last acked cursor, pending operation ids, ambiguous attempts), the server replays `cursor > reported` plus two directive kinds (`adjudicated` when it holds a receipt for the operation's event, `needs_adjudication` otherwise), and the node applies everything before becoming schedulable. The load-bearing rules that make it sound:
- **The node is authoritative for what it durably holds.** Replay is computed from the node's report, never from the server's ack bookkeeping (which exists for operations, not replay). A node reporting a cursor AHEAD of the server's ledger is refused with `version_conflict` — a journal-lost-class anomaly must stop the world, not re-base it.
- **The pending-operation exchange must DO something or it is ceremony.** First cut: `known_events` was redundant — the node re-emitted every pending event anyway, so the server's receipt report changed nothing. Fixed: a known event is marked acknowledged locally and NOT re-sent; only events the server never got travel the wire. The exchange became load-bearing, and the test proves it (1 receipt for the known event, 1 for the unknown one).
- **Schedulability is a state machine, not a flag someone remembers to set.** `Offline → Reconciling → Schedulable`, the terminal step written ONLY after the handshake is fully applied; `emit_event` refuses before it; ANY reconcile failure returns to `Offline`. Ambiguity does NOT block schedulability — a `needs_adjudication` attempt stays bounded and visible while the node works (the exit gate wants "ambiguous outcomes visible and bounded", not "everything terminal").
- **The first live run failed exactly one test — a fixture bug, not a protocol bug:** the test emitted for an operation id the journal had never created, and the FK correctly refused it. The protocol passed 11/11 on its first live run; the fix was in the test (use the replay-created operation id), recorded here for the pattern: seed through the real path, not parallel ids.
- Transport is HTTP/1 JSON (axum + reqwest) over loopback, unauthenticated: real sockets for the experiment, the planned §9.3 stack for later phases, and a loud "dev-only until WP5" boundary. A bespoke TCP protocol was rejected as throwaway code; SSE push, node-command leases, and the shared wire crate are deferred with owners (ADR-006/WP8, WP5, `reasonbraid-protocol`).
- Promoted to `docs/decisions/2026-09-06_node-channel.md` (`answers:` present). **WP3 complete; frontier `.4.1`.**

## _(2026-09-06)_ — WP3 node journal: the boundary record precedes the dispatch; ambiguity is recovered, never guessed

- KICKOFF WP3 / §11.4 / §17.4 require the node to persist a fact before advancing the corresponding boundary — and the dispatch is THE boundary that makes or breaks honest recovery. The fix: `record_dispatch` commits `prepared → dispatched` (with the provider request id when known) BEFORE the adapter is invoked, and a test proves a SECOND connection already sees the boundary record before the adapter would run.
- Recovery then has exactly two honest answers: `prepared` → `safe_to_redeliver` (the boundary was never crossed — claiming ambiguity would forbid a safe redelivery AND poison the operator's ambiguity signal), `dispatched` → `outcome_unknown` (the node may have dispatched; it cannot know). `prove_result` is the only exit carrying a result (adapter status lookup), `reconcile` the authorized adjudication — the §11.3 provider-lookup edges now exist in the core machine.
- **The core machine gained `failed_known` + `(dispatched, fail_known)` + `(outcome_unknown, complete|fail_known)`.** This supersedes the `.1.3` note that `failed_known` is out of Phase 0: the exclusion was about GUESSING, and a proven failure is not a guess. `cancelled_known` stays out. (Kill-risk Q4's honest minimal answer is preserved — indeterminate stays `outcome_unknown` until proof or adjudication.)
- **Durability is recorded, not just set.** `synchronous=FULL` is a per-connection pragma: any OTHER connection reading the file sees its own default, so a health view that reads pragmas would report a lie. The journal verifies the profile on its live connection at open AND writes it into `journal_meta`, which is what the read-only CLI reports. WAL alone is not a power-loss guarantee (§11.4); FULL is the conservative dev default.
- **Kill-point coverage as a table.** KP-1…KP-9 walk every `.3.1` seam (before/after command record, operation, prepare, dispatch, result, ambiguity, event emission, ack) by dropping the journal handle mid-flight with no checkpoint — each transition is its own transaction, so what survives IS what a killed process leaves. No sleeps, no mocks: the caller-supplied clock is stored as RFC 3339 TEXT.
- **The CLI is read-only by construction** (`SQLITE_OPEN_READONLY`) and proven non-mutating by byte-comparing the journal file after every inspection; WAL mode lets it run beside a live node (tested with the writer handle held open).
- Driver choice: sqlx SQLite over rusqlite — one driver stack with the server's Postgres side, `sqlx::migrate!` already proven in this repo, async-native for the future Tokio node. The first path dependency in the repo (`reasonbraid-core`) had to be version-pinned (`version = "0.1.0"`) or cargo-deny's wildcard ban rejects it.
- Promoted to `docs/decisions/2026-09-06_node-journal.md` (`answers:` present). **Frontier `.3.2`.**

## _(2026-09-06)_ — WP2 leased outbox worker: three commit points, per-claim fencing tokens, and a queue that owns its tests

- `ROADMAP.md` §17.3 / KICKOFF WP2 require leased claims plus "fencing tokens prevent a stale worker from committing after a newer lease," but `.2.1` left the outbox write-only. The fix is a three-phase loop where **each phase is its own commit point** — `claim_ready` (atomic `UPDATE … FOR UPDATE SKIP LOCKED` issuing a fresh `gen_random_uuid()` token + expiry + attempt++), `deliver` (dedupe sink keyed on `event_id`), `complete` (`WHERE lease_token = current AND lease_until > now`; otherwise `LeaseLost`). KICKOFF's kill points 3–5 are precisely the seams between those commits.
- **Fencing has two independent legs, both load-bearing:** a superseded claim fails the token check; an expired lease fails the liveness check *even with a matching token* (it must re-claim first). An attempt-CAS alone (the cheaper rejected design) cannot refuse the second case.
- **The clock is caller-supplied** (`chrono` → `TIMESTAMPTZ` via sqlx's `chrono` feature): tests advance past a lease expiry by passing `now + 61s`, never by sleeping or faking the DB clock.
- **The first live run failed 7/7 and the failure taught the real lesson** (TOOLBOX: probed, not guessed): the outbox is ONE shared queue, and my tests ran in parallel — each test's global oldest-first claim stole rows other tests (and the `atomic_transaction` binary) had seeded, and even single-threaded runs leaked leased-but-incomplete rows into later tests once their leases lapsed. The suite now serializes under a module-level async mutex, purges the queue under the guard, and cleans its own item at the end. A shared queue demands exclusive ownership from its tests; the worker API itself was correct throughout.
- Rejected designs recorded in the decision record: dispatching inside the claim (collapses kill points), a worker-global epoch table (per-claim token suffices; epoch earns its keep only for Phase 2 all-items quarantine), DB-clock expiry, and a `next_eligible_at` backoff column (nothing fails delivery yet — Phase 2 retry policy will add it).
- Promoted to `docs/decisions/2026-09-06_outbox-worker-fencing.md` (`answers:` present). **WP2 complete; frontier `.3.1`.**

## _(2026-09-06)_ — WP2 atomic transaction: claim-first idempotency, four tables in one commit

- `ROADMAP.md` §8.6 / KICKOFF WP2 require "one transaction writes current state, ordered event, idempotency result, and outbox item," but nothing enforced it — four autocommit `INSERT`s could tear, and a "check-then-insert" idempotency check races under redelivery. The fix is structural: `apply_command` claims the `(tenant_id, idempotency_key)` primary key **first** (`INSERT … ON CONFLICT DO NOTHING`), so the unique index is the concurrency control — a redelivered message either replays (same hash → original stored result) or conflicts (different hash).
- The four writes (idempotency claim, `event_log`, `aggregate_state`, `outbox`) run on one transaction; a failure at any step drops it and rolls back the claim too. The outbox FK → `event_log` makes "outbox row implies durable event" a schema fact, not an assertion.
- **The proof needs a live Postgres** — the tests skip when `DATABASE_URL` is unset (so `make check` stays green offline) and run for real only in `scripts/run_pg_tests.sh` (ephemeral `initdb`/`pg_ctl`, no background service) and the `pg-tests` CI job. "successful response ⇔ committed durable state" is asserted by reading all four tables back from a *separate* connection after commit.
- **Honest limit:** `next_version = MAX+1` under `FOR UPDATE` serializes writers to an *existing* aggregate, but a fresh aggregate's first insert isn't gap-locked — two concurrent first-writes to the same new aggregate aren't fully serialized. Phase-1 concern, out of WP2's single-writer scope.
- **`deny.toml` was wrong for the tool it names.** It was authored against an old cargo-deny schema (when deps were zero) and only TOML-parsed, never run through cargo-deny — so the first real `make deny` failed. Corrected for cargo-deny 0.20: `[advisories].unmaintained` is a *scope* (`all`/`workspace`/`transitive`/`none`), not a lint level (`deny`); added `BSD-3-Clause` for `subtle` (constant-time crypto, via sqlx SCRAM); `skip` for the reviewed `getrandom`/`hashbrown`/`syn` sqlx-tree duplicates. Lesson: a config for a tool that isn't installed is a *draft*, not a gate.
- Promoted to `docs/decisions/2026-09-06_atomic-transaction.md` (`answers:` present).

## _(2026-09-06)_ — WP1 typed errors + reason-code registry: complete §9.8, unknown codes preserved

- `ROADMAP.md` §9.8 lists reason codes but not how to treat an unknown one. The two naive shapes both fail: a closed enum *rejects* the future (deserialization error), a bare `String` *loses* the typing of the known set. The fix is a two-layer `ReasonCode` — `Known(KnownReasonCode)` + `Unknown(String)` with `#[serde(untagged)]` — which gets both properties at once.
- `KnownReasonCode` is the *complete* 20-code §9.8 registry (not a demo subset): a "stable registry" re-carved every leaf isn't stable, and client/server must be able to name any §9.8 code consistently. Forward-compat is `Unknown`'s job, not a reason to trim the list.
- `DomainError` carries code + tri-state `Retryability` (`no`/`yes`/`requires_authorization`) + safe `message` + optional `correlation_id` + filtered `details`; secrets/policy internals/cross-tenant existence stay off the type.
- `From<TransitionError> for DomainError` maps `.1.3`'s deterministic rejection to `invalid_transition`, proving the registry classifies real errors rather than sitting unused.
- Acceptance tests: every known code round-trips to its snake_case name; `future_semantic_reason` deserializes to `Unknown` and re-serializes verbatim; a near-miss (`invalid_transition_typo`) is preserved, not misclassified.
- Promoted to `docs/decisions/2026-09-06_reason-codes.md` (`answers:` present). **WP1 complete.**

## _(2026-09-06)_ — WP1 state machines: minimal lifecycles, deterministic fallible `apply`

- `ROADMAP.md` §8.4 lists lifecycle *states* but not *edges*; §8.6 requires "a deterministic aggregate may accept and translate to an event." The gap is closed with three minimal state enums whose only operation is `apply(transition) -> Result<state, TransitionError>` — total, deterministic, fallible, no panics, no history rewinds.
- Chosen edges: thread `open → closing → closed` (two-step close, not a direct `open → closed`) plus `open/closing → cancelled`; participation `invited → {accepted, declined, expired}` and `accepted → left`; provider-attempt `prepared → {dispatched, failed_before_dispatch}`, `dispatched → {completed, outcome_unknown}`, `outcome_unknown → reconciled`.
- A *proven* post-dispatch failure (`failed_known`/`cancelled_known`) is deliberately out of Phase 0 scope — the honest minimal answer to an indeterminate attempt is `outcome_unknown → reconciled` (kill-risk Q4), not a guessed failure.
- Exhaustive tests assert BOTH that every listed edge resolves to its target AND that every unlisted (state, transition) pair is rejected — rejection is a property of the table, not a side effect. State enums serialize `snake_case`; transition enums are transient (the wire catalogue is backlog 6).
- Added the deferred `ProviderAttemptId` (`patt`) to complete the WP1 distinct-types acceptance.
- Promoted to `docs/decisions/2026-09-06_state-transitions.md` (`answers:` present).

## _(2026-09-06)_ — WP1 envelopes: intent in, authority out, forgery rejected

- `ROADMAP.md` §9.1 sketches the command/event split but nothing enforced it — serde ignores unknown fields by default, so a struct that merely *omits* authoritative fields would still accept them from a client. The fix is mechanical: `#[serde(deny_unknown_fields)]` on `CommandEnvelope`, `ClientContext`, and `CommittedEvent` makes the same deserialization that accepts a valid command reject a forged one.
- `CommandEnvelope` = intent only (operation, `request_id`, idempotency key, optional expected aggregate version, opaque `body`, correlation/causation context); `CommittedEvent` = server-assigned authority (event id, tenant, aggregate, sequence, actor principal, timestamps, authorization record, schema version). Optional fields are nullable (`null` on the wire, `#[serde(default)]` on read) to match §9.1's explicit nulls.
- Golden fixtures (`fixtures/`) cover every wire payload the demo uses: `command-thread-create.json`, `event-thread-created.json`, and a `command-with-authoritative-fields.json` that must fail. `schemars` (derive + a manual `Id<K>` impl) generates JSON Schema goldens (`schema/`) guarded by a drift test; regenerate with `cargo test -p reasonbraid-core -- --ignored write_schema_goldens`.
- Timestamps stay `String` (RFC 3339) until ADR-010 pins the time type; `ActorPrincipalId` (`agt`) is opaque — which principal kind it names is a WP5 concern.
- Promoted to `docs/decisions/2026-09-06_envelope-representation.md` (`answers:` present).

## _(2026-09-06)_ — WP1 strong IDs: branded newtypes over UUIDv7, prefix-checked on the wire

- Landed `crates/reasonbraid-core` (first real crate; the scaffold's placeholder `crates/app` binary is removed). `KICKOFF.md` §3 names this crate "IDs, envelopes, minimal thread and attempt states".
- ID representation: a generic `Id<K>` newtype over `uuid::Uuid` (v7) branded by a zero-sized marker `K`; eight families (tenant, human principal, host, node, agent role, agent incarnation, run, thread), each a distinct three-letter wire prefix validated on deserialization.
- The non-interchangeability guarantee is tested two ways: `TypeId::of::<X>()` pairwise-distinct (compile-time newtypes, not aliases) and serde round-trip with wrong-prefix rejection (wire-level non-confusability).
- Dependencies `serde` + `uuid` (dev `serde_json`) are all permissive-licensed; licenses checked against `deny.toml`'s allow list by hand (cargo-deny itself runs in CI, not installed locally).
- Promoted to `docs/decisions/2026-09-06_id-representation.md` (`answers:` present).

## _(2026-09-06)_ — G0 contract drafts: ID-scheme gap closed, drafts live in `spec/`

- `ROADMAP.md` §19.1 names six illustrative requirement-ID prefixes (ID / AUTH / DELIV / RES / POL / SEC) but §20.2 gates G0 on five boundaries — identity, authority, **thread**, delivery, **budget**. Thread and budget had no prefix, so the "stable IDs" acceptance could not be met without a choice.
- Decided: add `THREAD-*` and `BUDGET-*` (first-class now); reserve `RES-*` (Phase 4), `POL-*` (Phase 6), `SEC-*` (Phase 7). Gap-fill for backlog 4, not a feature — the frozen roadmap is untouched.
- Contract drafts live under `spec/` (beside the code, §7.1/§19.1), not `docs/`; each is headed "draft — not normative".
- Promoted to `docs/decisions/2026-09-06_g0-contract-id-scheme.md` (`answers:` present).

## _(2026-09-06)_ — supply-chain skeleton

- Added `deny.toml` (cargo-deny: advisories/bans/licenses/sources), `.github/workflows/supply-chain.yml` (cargo-deny + gitleaks secret scan), and `docs/ci.md`; the Makefile gained `make deny` / `make secret-scan`.
- The `deny.toml` schema was copied from the authoritative cargo-deny `main` template (EmbarkStudios repo), not reconstructed from memory: current shape is `[graph]`/`[advisories]`/`[bans]`/`[bans.std-replacements]`/`[sources]`/`[licenses]`, with no `version` key.
- `cargo-deny` and `gitleaks` are NOT installed locally; the Makefile targets forward to them and CI installs them. Local runs need `cargo install cargo-deny` / `brew install gitleaks`.
- Validated: `deny.toml` parses (python3 `tomllib`), `make -n deny` → `cargo deny check`, `make -n secret-scan` → `gitleaks detect --source . --redact`, `make gate` 13/13, `make check` 1 test ok.

## _(2026-09-06)_ — external dependency ledger skeleton

- Created `docs/dependencies/external-ledger.yaml` from `ROADMAP.md` §7.4: one entry per protocol/SDK/CLI/provider/harness, `checked_at` dated, a `revalidation_trigger` per row.
- Stubbed MCP, A2A, Codex, and Claude rows from the 2026-09-04 corrected baseline (§28.1). `license` is `"unverified"` until a spike records it from package metadata — never asserted from memory.
- Validated with `ruby -ryaml` (4 entries, required fields present) so the file parses clean before it is committed.

## _(2026-09-05)_ — KICKOFF.md is a companion, not a second roadmap

- Director dropped both `ROADMAP.md` (v0.4.1 master) and `KICKOFF.md` (Phase 0 execution).
- They are one pair: the master is frozen scope/gates; the kickoff is the Phase 0 task board.
- Promoted to `docs/decisions/2026-09-05_kickoff-companion-to-roadmap.md` (`answers:` present).

## _(2026-09-04)_ — a template's trial must include the first commit

- Every gate was green on the generated project and the first commit still failed: the doctrines judge STAGED
  code, and nothing had been staged until the user tried. Trial the path a user walks, to its end.
- `grep -c` prints `0` and exits 1. `$(grep -c … || echo 0)` therefore yields `0⏎0` — a second line — which
  here started a flush-left line inside a checklist bullet and hid its evidence from the box-scoped extractor.
  Capture the count, then default the empty case; never append a fallback to grep's own output.

## _(2026-09-04)_ — a green gate that judges nothing is the class a template must not ship

- Two of the four doctrine ports in `.2.6` were wrong on first run and their own RED self-test arms said so:
  a `python3 - <<'PY'` detector whose stdin was the heredoc (every arm read 0 rows), and a `grep -c … | grep -qx 0`
  control under `pipefail` (`grep -c` prints 0 and exits 1). A self-test with only GREEN arms would have passed both.
- The neutrality bar is measured, not felt: `grep -ciE 'grammar|parser|…'` over each ported script → 0, after the
  generic uses of "corpus" and "grammar" were re-worded ("tree", "syntax") so the count means what it says.

Detailed technical notes — root cause, implementation, validation — per slice. The
engineering-continuity surface (not the public docs; that's `docs/book/`). Newest first.

## _(YYYY-MM-DD)_ — bootstrap

Repo created from the ReasonBraid spine template: durable 4-layer memory, task-tree tracking, the
strict commit workflow, and the mechanical doctrine enforcer are in place and enforced by
git hooks + CI. No project code yet.
