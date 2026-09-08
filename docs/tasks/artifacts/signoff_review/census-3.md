# Source census — part 3

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Status of all records: pending reproduction or explicit refutation. Repair contracts: `docs/tasks/SIGNOFF-REPAIR.md`.

## R-56-57-1

- Repair candidates: `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`.
- State: open source-review record; runtime pending.

threads.rs OP_REVISE checks only target event type is thread.challenged, then saturating_sub global open_challenges; no per-challenge resolved tracking, no challenged-author binding. Multiple different idempotency-key revisions of same challenge can hide other unresolved challenges. Source evidence runtime pending.

## R-56-57-2

- Repair candidates: `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

threads.rs OP_CLOSE decision-family validates only client body.unresolved empty, not projection.open_challenges or durable objections; can claim decided with unresolved work. Workflow close jumps terminal irrespective current step; assess contract vs intentional organizer authority.

## R-56-57-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.11.3`.
- State: open source-review record; runtime pending.

threads.rs InviteBody expiry adds ChronoDuration::seconds(client i64) without demonstrated bounds; overflow/past expiry potential. OP_INVITE doesn't ensure role enrolled/tenant here (check caller before repair).

## R-56-57-4

- Repair candidates: `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

threads.rs moderation structural vocabulary permits arbitrary content prose; documented no verdict structural only, avoid overclaim actual semantic protection. Verdict target_digest does not get same existence binding as evidence request target. Synthesis validates numeric range exists not body transformation/provenance correctness.

## R-58-1

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

workflows.rs register public API permits new versions of built-in IDs without protection, MAX+1 nontransactional; resolve chooses latest and maps all DB errors UnknownProfile, list silently drops malformed DB rows. Thread projection persists profile ID+steps but not resolved version despite documented versioned reference.

## R-58-2

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

Integration test fixtures purge broad tables via DATABASE_URL without explicit test database guard in each pool; inspect run_pg_tests shell protections. allowlist tests assert every human tenant admin mutates global registry; need replace with explicit site operator fixture, retain non-admin/read semantics. Current tests no boundary revocation/cross-tenant regressions.

## R-58-3

- Repair candidates: `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

authority test pool comment says authority tables exclusively owned binary but other tests purge/use same tables. Must confirm run_pg_tests serializes test binaries; cargo test normally binary sequence but independently concurrent invocations unsafe.

## R-59-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

authority integration delegated success uses both grants delegable=false and boundary.delegable=false, demonstrates tests codify bypass candidate (need ADR semantics review). expired_and_revoked_grants_are_denied only exercises expired, no revoke at all; test title coverage overclaim.

## R-59-2

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.2.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

backup_restore.rs uses std::env::temp_dir (locality only if TMPDIR forced), own URL string parser mishandles passwords/user percent encoding/query/IPv6, logs raw driver errors possible credentials; fixed ceil_restore seed non-idempotent on failure, cleanup on success only, ignores failed spawn dropdb. Requires safe guard before running.

## R-61-62-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

command_api later section DOES exercise grant revoke and boundary revoke on next command; earlier coverage gap limited authority.rs named test, not entire suite. No cross-tenant wrong-ID-afterwrite test observed yet. Boundary list stays readable after revoke (approved inspection carveout).

## R-61-62-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

command_api delegation test allows human contribution on behalf of role that never joined, uses actor participation and actor event author while audit.subject role. Need clarify trusted impersonation vs dual-authority operation semantics in ADR; no delegation consent token and nondelegable grants remain major candidate.

## R-61-62-3

- Repair candidates: `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

cards wrong schema test passes invalid 8-digit digest and asserts only400, so doesn't isolate compatibility rung; need full valid recomputed digest unsupported schema and specific error code to validate order.

## R-63-1

- Repair candidates: `SIGNOFF-REPAIR.3.1`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

escalation.rs cross-tenant headline 'at every boundary' tests create/read/idempotent denied replay only, no admin revoke wrongtenant ID or global registries. Existing revocation test intentionally permits exact original result replay postrevoke; preserve with actor/target-bound request hash, don't blanket require fresh grant for authorized replay.

## R-63-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

command_api metrics test comment claims replay counter delta but only asserts authorization_denials; replay fixture no counter assertion. Need truthfulness correction/coverage depending endpoint counter present.

## R-65-1

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

threads OP_INVITE/OP_JOIN examines stored Invited before derived expiry, so expired invitation remains stored Invited and cannot be reinvited/joined despite comment terminal expired may reinvite. invitations.rs tests reinvite only declined thenexpiryseparately; add expire→reinvite regression. ttl -1 intentionallyallowedfixture, actualunboundedoverflow remains.

## R-65-2

- Repair candidates: `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.6.2`.
- State: open source-review record; runtime pending.

mcp_listen integration only two delivery IDs; does not reach64-window truncation or outofordercursor/firstinsertconcurrency.

## R-66-1

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

mcp_write test claims quota use+auditedallowance same transaction but seam gate commits separate from handler; test onlycountsdoesn'ttestatomicity. Clarify quota admission may intentionallycountdeniedattempts vs successatomic claim.

## R-66-2

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

migration_upgrade test unconditionally DROP SCHEMA public CASCADE at arbitraryDATABASE_URL; no local test-db safety check. All-but-last movingtestno longerprovesbackfills migration0047; commentacknowledgesbutsectionclaimsquota backfill.

## R-66-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`.
- State: open source-review record; runtime pending.

node_channel seed_node_in_tenant binds HOSTtenant constant seedtenant regardlesstenantargument, node row differenttenant allowed => identity foreignkeys fail existence only, not tenant consistency. ImportantproductionFKmigration validation.

## R-67-68-1

- Repair candidates: `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

node_channel.rs restart/kill tests abort axum task not process or PG; describe as listener restart, don't claim process crash durability from this alone.

## R-67-68-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_channel.revoking_a_node test deliberately preserves livelease online=true after revoke; distinguishes documented reentry suspension from immediate cutoff. Source indefiniteheartbeatrenewal stillrisk: bounded residual session vs indefinitely renewable compromised cert authority needs clear trackeddecision.

## R-67-68-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_channel wake gate tests onlyconcurrencyzero then2; nopositiveconcurrency activecount. Testservercert handshake over plainhttp demonstrates mTLSstandalone test not shippedlistener.

## R-67-68-4

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_channel handshake bad proof test malformed cert00 refusesbeforeproof, doesn't isolate wellformedcertwrongsignature. Need exact cryptographicnegativefixtures +capturedvalidproofreplay.


