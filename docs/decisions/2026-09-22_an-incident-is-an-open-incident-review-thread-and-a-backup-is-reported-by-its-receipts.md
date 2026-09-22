---
answers:
  - What is an "active incident" in ReasonBraid, and does it need a new aggregate?
  - Who produces the facts behind §18.5's backup/restore status?
  - Why do the backup scripts write receipts instead of inserting database rows?
  - When is a backup accepted as a recovery control?
---
# An incident is an open incident-review thread, and a backup is reported by its receipts

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.6.1.5`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §18.5 (its ninth bullet: *backup/restore status and active
  incidents*), §17.5 (*a backup that has never been restored is not accepted as a
  recovery control*), §13's workflow-profile table (`incident_review`: *timeline,
  hypotheses, evidence, actions* for *operational/security events*), §4.7;
  `docs/decisions/2026-09-22_three-of-the-five-missing-operator-surfaces-have-no-stored-fact.md`

## Context

DOC-0131 left two questions for this leaf. Who records a backup and a restore test?
And is an *active incident* a new aggregate, or a record the system already keeps?

## Decision 1 — an active incident is an open `incident_review` thread

The roadmap already gives incidents a home. `incident_review` is a built-in workflow
profile (`migrations/0032_workflow_profiles.sql`), defined for *operational/security
events*, with the steps *solicit, evidence_request, synthesize, decide*. A thread under
that profile has everything an incident needs:
- a subject;
- participants;
- a timeline, which is the event log;
- evidence and challenges;
- a counted or owner decision;
- a close.

Declaring and resolving one are the existing, audited thread verbs.

⇒ **No new aggregate.** An active incident is a thread in the tenant whose projection
names `workflow_profile = incident_review` and which is not closed or cancelled. §18.5
needs only a view: `.4.6.1.5.1`.

⛔ **Rejected: a separate `incidents` table with its own declare/resolve verbs.** It
would duplicate the thread's lifecycle, audit and authority, and create a second place
where "is this incident over?" could get a different answer.

## Decision 2 — a backup and a restore test are reported by the receipts their scripts write

`scripts/backup.sh` and `scripts/restore.sh` are the documented procedure, and the
restore runbook uses them. They are therefore the producers, but they do not write
database rows. A row inserted by a shell script sits outside domain commands and audit,
which §18.5's closing rule forbids. It would also claim a backup took place without the
dump file present to prove it.

Instead, each script writes a **receipt file beside the dump**, only after its step
succeeds:
- `backup.sh` writes the dump's byte count, SHA-256, the instant it was taken, and the
  database name. The name excludes credentials.
- `restore.sh` first checks the dump's SHA-256 against that receipt, and refuses on a
  mismatch. After `pg_restore`, it checks that the restored database carries the applied
  migrations. Only then does it write the restore receipt.

The server is given the directory (`--backup-dir`). It **re-verifies what it can**: each
receipt's dump must exist with the recorded size. It reports each backup, the time since
the newest one, and the time since the newest restore-tested one. By §17.5, the verdict
`recovery_control: accepted` requires at least one backup with a restore receipt. A
backup that was never restored is listed but not counted. This is `.4.6.1.5.2`.

⚠️ **A receipt is a file the operator's own tools wrote.** It is evidence that the
procedure ran, not proof against a hostile operator. The server's re-verification of the
size catches a dump that was deleted or truncated since. A byte-exact check is the
restore script's job, because the restore is the step that reads the whole dump anyway.

## Consequences

- `.4.6.1.5` is split into `.5.1` (incidents, a view) and `.5.2` (backups, receipts plus
  a view).
- ⚠️ Neither surface makes a backup happen. §17.5's scheduled, encrypted, off-site
  backups and point-in-time recovery are deployment-profile work beyond the Developer
  profile. The status surface shows their absence plainly rather than implying a policy.
