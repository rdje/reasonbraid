---
answers:
  - Do the findings published on 2026-09-22/23 (REPAIR-0405…0415, DOC-0131…0135) still hold?
  - Which were re-derived by a route different from the one that produced them?
  - What moved, and what was corrected at its source?
  - What does the OPERATOR-SURFACES census NOT prove?
---
# This session's findings, re-derived against the running server

- **Type:** verification record
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.6.1.7` (the durability gap) and the leaves named below
- **Date:** 2026-09-23
- **Cites:** `docs/CLAIM_VERIFICATION.md` §3 (re-derive · falsify · durability), §4.1 (graded on
  PROSE, NUMBER, NAMED INSTANCE separately)

## How

The director asked that the findings be verified. Each claim below was re-derived by a route
DIFFERENT from the one that published it. Mostly that meant the **built `rb-server` binary**:
- running against an ephemeral PostgreSQL under `target/`;
- driven over HTTP;
- queried directly with `psql`.

The original route was source reading or a unit or integration test. The script was run twice,
and its data was removed after each run.

## Held — PROSE, NUMBER and NAMED INSTANCE all unchanged

| claim | original route | re-derivation route | result |
| --- | --- | --- | --- |
| §18.5: 8 of 9 surfaces exposed | route registration (`census_operator_surfaces.py`) | each of the 24 mapped routes called on the RUNNING binary; a router 404 has an empty body | **all 24 reached a handler** (200/400/404-with-body/422; the two 405s are GET on POST-only collections) |
| auto-initiation is bounded only by `.5.2`'s once-per-tenant key | source reading of the key | the same role initiated twice, then a third time with another body | 1st `200`; 2nd `200 "replayed": true`, **same thread id**; 3rd `409 idempotency_mismatch` naming `auto_{role}_{tenant}`; **1 thread** |
| `backup.sh` no longer prints or records a password | unit tests of `redact` | `backup.sh` run with `postgres://postgres:S3cretPW@…` | password occurrences: **0** in output, **0** in the receipt; the output shows `postgres://127.0.0.1:…/rbv` |
| a never-restored backup is `not_accepted` | integration test | the served `GET /v1/admin/backups` after a real `backup.sh` | `declared: true`, `recovery_control: not_accepted` |
| row 7: no node-binding registry | source reading (agent) | `information_schema` of the live database | no table has both `node_id` and `role_id`; `nodes` = `node_id, host_id, tenant_id, created_at` |
| row 8: mTLS is not on the listener | source reading (agent) | every call above went over plain `http://` and succeeded | holds |
| row 16: the matcher is not semantic | source reading (agent) | the live schema: columns named like `%embed%` or typed `%vector%` | **0** |
| row 17: the RX answer is consumed nowhere | `git grep -c` (agent) | position relative to `#[cfg(test)]` | `AcquisitionAnswer` is defined at `mediated.rs:14`/`:46`, and every use sits in the test module below `:66`; product code builds only `AcquisitionCall` |
| `operating_hours` / `wake_policy` never read | `grep` of the source | a role whose profile says hours `"never"`, wake `"manual_only"`, then initiates | profile stored verbatim; initiation **200** |
| 8 held / 4 moved of 12 re-derived | the table as written | a parser over the COMMITTED file (`git show HEAD:`) | moved `[7, 8, 16, 17]`, held `[5, 6, 9, 10, 11, 12, 15, 19]` |
| 3.4 GB of retained fixtures reduced | the census's summary line | the sum of every `retired.json` written in the last 30 h | **65** fixtures, **3,410,239,220** bytes, exactly the published figure |

## Moved

1. 🔴 **NAMED INSTANCE — row 14 of the 21-row pass still read `fired and open`.** `.4.6`
   re-derived it on 2026-09-22 as 1 open, 1 partial and 2 discharged. That correction reached
   `.4.6`'s leaf and its decision record, but **never the row**. DOC-0133 then parsed the rows
   and published `fired and open: 1`, a count carried from a stale row. ✅ Row 14 is now
   corrected at the row: 2 discharged, the sink deferred on a readable trigger, and §18.5 at
   8 of 9 with 1 blocked on ADR-022. The re-derived tally is **discharged 9 · partial/splits 8
   · not yet triggered 2 · superseded 2 · fired-and-open 0**. `6 of 14 re-derived verdicts
   were wrong` is unaffected: it already counted row 14's original verdict as wrong.
2. 🔴 **NUMBER — "17 commits"** in the end-of-session report. `git log --oneline
   1cdfe83..HEAD | wc -l` gives **16**. That was a hand count in chat prose, and nothing
   tracked carried it.

## New findings from the re-derivation, each owned

- ⚠️ **DURABILITY — `OPERATOR-SURFACES` proves REGISTRATION, not SERVICE.** It reads route
  strings out of the source. `/v1/health` and `/v1/admin/backups` live in routers that exist
  only if `rb-server`'s `main` merges them. A router dropped from that merge would leave the
  census green while the surface was gone. The live run shows all 24 are served today, and
  nothing keeps that true. Owner: `SIGNOFF-REPAIR.4.6.1.7`. The fix is one app-composition
  function, used by both `main` and a test that calls every mapped route.
- ⚠️ **`operating_hours` accepts any text**: `"never"` was stored. A field that nothing reads
  and nothing validates is the declared-but-unread defect twice over. Attached to
  `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`, which owns the field's meaning.
- ⚠️ **Thread sub-reads disagree about whether a thread exists.** For an id that does not
  exist, `GET /v1/threads/{id}` and `…/budget` answer `404`, while `…/events` answers
  `200 {"events": []}` and `…/audit` returns the caller's own inspection records. Nothing
  leaks, since no other tenant's data appears. But one question gets two answers. Owner:
  `SIGNOFF-REPAIR.17`.

## The three legs, stated

- **Re-derived:** every row above, by the route named.
- **Falsified:** the live checks are oracles this session did not build. They are the
  shipped binary and PostgreSQL's own catalogue; the tests are not re-run against themselves.
- **Durable: PARTIALLY.** The census, the unit tests and the integration tests are tracked
  and gated. The live script is not, and the served-not-merely-registered property has no
  gate yet. ⛔ That gap is named here and owned by `.4.6.1.7`. It is not hidden.
