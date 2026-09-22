---
answers:
  - Which of ROADMAP §18.5's five unexposed operator surfaces only need a read route?
  - Why was `.4.6`'s claim that three of them "need no new machinery, only a view" wrong?
  - What must PRODUCE each missing fact before a route can return it?
  - Why is audit-checkpoint age deferred rather than built?
  - In what order are the five built?
---
# Three of the five missing operator surfaces have no stored fact, so each needs a producer before it needs a route

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.6.1`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §18.5 (operator surfaces), §11.4 (ambiguous attempts), §18.1–§18.2;
  `docs/adr/022-audit-hash-chain-groundwork.md`;
  `docs/decisions/2026-09-22_the-observability-row-is-four-commitments-and-two-were-already-discharged.md`
  (this record corrects that record's verdict-2 note and `.4.6.1`'s premise)

## Context

`.4.6` found five of §18.5's nine operator surfaces unexposed: (1) service and dependency
health with freshness, (4) ambiguous attempts and safe resolution actions, (6) resolver
denials, (8) audit-chain checkpoint age, (9) backup/restore status and active incidents.
It opened `.4.6.1` with a premise: *three of the five need no new machinery, only a view*.
In bullets 4, 6 and 8, it said, *the fact exists and no route returns it*.

That premise was a reading of mechanisms, not of storage. A route can only return a fact
that something has already stored. This record checks the storage for each bullet.

## The corrected census — what is actually stored

Each row gives the check that reproduces its answer.

| § | surface | is the fact stored? | evidence |
| --- | --- | --- | --- |
| 4 | ambiguous attempts | 🔴 **no** | `handshake` (`crates/reasonbraid-server/src/node_channel.rs`) builds a `NeedsAdjudication` directive for each reported attempt, returns it in the response, and forgets it. The handshake inserts only into `node_proof_nonces` and `node_leases`. `git grep -niE "outcome_unknown\|ambiguous" -- migrations` finds only comments. `outcome_unknown` is stored only in the node's local journal. `runs` has no status column. |
| 6 | resolver denials | 🔴 **no** (one exception) | `resolve_resource` (`api.rs`) admits the caller with `reader_tenant` alone, so it writes no `authorization_records` row. Every `AcquisitionError` and every `unresolvable_now` exists only in the response. The exception is a quota refusal, which inserts `quota_events(kind = 'denial')` (`quota.rs`); no route reads that table. |
| 8 | audit-chain checkpoint age | 🔴 **no, and there is no chain** | `docs/adr/022-audit-hash-chain-groundwork.md`: *No `prev_hash` column, no checkpoint table, no verification CLI in the dev profile*. `git grep -niE "prev_hash\|checkpoint" -- migrations` finds nothing. |
| 1 | health with freshness | 🔴 **no producer** (as `.4.6` said) | No health, readiness or liveness route exists on the api, node or ui routers. After boot, nothing probes Postgres, the CA, the secret store or the publication root. |
| 9 | backup/restore status, active incidents | 🔴 **no producer** (as `.4.6` said) | `tests/backup_restore.rs` restores into an isolated database and then drops it, leaving no record. `scripts/backup.sh` writes `target/backups/*.dump` and stores nothing in the database. `git grep -niE incident -- migrations crates/reasonbraid-server/src` finds only a correction-outcome kind and a workflow-profile name. |

⭐ **All five need a producer, not three of them.** `.4.6` was right that the *mechanisms*
exist for bullets 4 and 6, but mechanisms are not stored facts. The handshake decides an
attempt and forgets the decision. A resolver refuses and forgets the refusal. For bullet 8,
the checkpoint itself does not exist.

## Decision — one child per bullet, with the producer first

| child | bullet | verdict | the producer it owns |
| --- | --- | --- | --- |
| `.4.6.1.1` | 4 ambiguous attempts | 🔨 **BUILD** | The handshake records each `needs_adjudication` report per node: first and last reported instants and the directive issued. A later handshake that adjudicates the attempt, or no longer reports it, closes the row. A tenant-admin list route returns open rows with the safe action the directive names. |
| `.4.6.1.2` | 6 resolver denials | 🔨 **BUILD** | The resolve path records each refusal it answers (kind, message, resource, tenant, instant). A tenant-admin list route returns them. Quota denials already stored in `quota_events` are joined into the same view rather than stored twice. |
| `.4.6.1.3` | 8 checkpoint age | ⏸️ **DEFERRED with ADR-022**, whose trigger it shares | None now. See below. |
| `.4.6.1.4` | 1 health with freshness | 🔨 **BUILD** | A probe of each runtime dependency (Postgres, the CA, the secret store, the declared publication root). Each probe records its outcome with an `observed_at`, and the route reports that instant's age. The leaf must decide how the route can say *Postgres is down* when its own authorization depends on Postgres. |
| `.4.6.1.5` | 9 backup/restore, incidents | 🔨 **BUILD after a decision** | The leaf decides who records a backup or restore test (the scripts, the test, or a server verb). It also decides whether an *active incident* is a new aggregate or an existing correction/workflow record that is already open. |
| `.4.6.1.6` | all nine | 🔨 **BUILD** | The census instrument `.4.6` said is owed: a script that re-derives the nine-bullet mapping from the router and the migrations, so this verdict cannot silently decay. |

**Order:** `.1` → `.2` → `.4` → `.5` → `.6`. Bullets 4 and 6 come first because each has a
producing code path to extend, and each extension is one table plus one route. Health and
backup each need a design decision first. The census comes last, so it measures the
finished state rather than a moving one.

## Why checkpoint age is deferred rather than built

§18.5 asks for *audit-chain verification and checkpoint age*. The age is measured from the
chain's last signed checkpoint. ADR-022 recorded that the chain itself is not built in the
Developer profile. Its revisit trigger is *the first non-loopback deployment or the G7 ops
gate*, which is the same readable condition `.4.6` gave the OpenTelemetry sink.

Building a checkpoint age before its checkpoint would mean inventing a number. Building the
checkpoint here would mean building ADR-022's whole chain under an operator-surface leaf,
which is the wrong owner. ⇒ `.4.6.1.3` is `blocked` on ADR-022's trigger. When that trigger
fires, the chain's leaf must deliver the age as part of its surface. The verification half
of bullet 8 is already partial: `/v1/threads/{id}/audit` and `/v1/audit/receipts`
reconstruct the linkage.

## Consequences

- 🔴 **`.4.6`'s decision record said three of the five "need no new machinery". That is
  corrected here.** The record's verdict table stays accurate: it graded the three bullets
  *partial*, and they are partial. Only the note about their shape was wrong.
- ✅ Every one of the five bullets now has an owner and a stated producer.
- ⚠️ Health (`.4.6.1.4`) contains a real design conflict. The neighbouring admin routes
  authorize against Postgres, so a health route gated the same way cannot report Postgres
  being down. The child leaf must resolve this explicitly and not copy the neighbours'
  rule without thinking.

## What would make this wrong

- ⛔ If a route that returns the handshake's directives or a resolver's refusals existed
  under a name the census did not search for, bullets 4 and 6 would be views after all.
  The census searched the router (`api.rs`), the node channel and the UI router, and the
  migrations. A route in a submodule would not count, because this server does not serve one.
- ⛔ If ADR-022's trigger has already fired (a non-loopback deployment or the G7 gate
  passing), then `.4.6.1.3` is not blocked and must be built with the chain.
