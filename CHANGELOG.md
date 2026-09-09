# CHANGELOG.md

## 2026-09-09 — Actual-parent command authority (`SIGNOFF-REPAIR.3.3.3.1`)

Normal commands select usable caller/delegated grants through deterministic
32-row pages and each grant's actual parent. Requested delegation scope participates
in selection. Denial records preserve the authority source; absent sources carry
no grant or parent. Malformed stored candidates fail with a storage error. Six
pure evaluator controls, all 37 live authority/command API tests and strict
focused lint pass. All results are consumed and the owned cluster removed. Frozen-admin reads and revocation serialization remain separate repairs.

## 2026-09-09 — Verified changelog rotation (`SIGNOFF-REPAIR.11.4.1`)

Retain the ten recent corrective-review records and rotate 120 older entries
through the existing Git-history terminal. The exact predecessor and whole-record
segments reconstruct all 95,038 source bytes; the 96,000-byte cap is unchanged.
History retrieval and historical qualification limits are documented below.

## 2026-09-09 — Bound tenant authority (`SIGNOFF-REPAIR.3.3.2`)

Bind grants to their named parent, tenant and evaluated subject; enforce nonempty
half-open validity and action-target selector coverage. Thread-scoped grants cannot
administer or list an entire tenant. Core 51 unit + 3 subject tests, six evaluator
controls, 32 live authority/command API tests and strict core/server lint pass.
All results are consumed and the owned runner stopped/removed its cluster. Book and decision
record document the contract and remaining loader/transaction repairs.

## 2026-09-09 — Canonical core subject JSON (`SIGNOFF-REPAIR.3.3.1`)

Core human/role subjects now serialize as kind/id objects and reject malformed,
duplicate, unknown or mismatched input through an object-only parser. Public
command-envelope strings and split database fields retain their contracts; only
the generated schema description changes. Direct and enclosing payload failures
are reproduced and corrected. Core tests pass 49 unit + 3 integration controls;
strict core/server lint and all 40 live authority/command API/site-receipt
compatibility controls pass. The owned cluster stopped and was removed. ADR-009's
synthetic token-size comparison is withdrawn and the missing evidence is owned
by the delegation repair.

## 2026-09-09 — Site authority enforced on registry HTTP (`SIGNOFF-REPAIR.3.2.3`)

All seven adapter/region operations now require explicit site grants and use the
atomic authority/effect/audit service. Mutations require bounded reasons; success
bodies retain their keys with a committed audit header. Refusals distinguish
malformed input, authority, domain and storage failures. Invalid UTF-8 paths now
return typed JSON after a regression exposed the extractor bypass. Tenant
enrollment confers no site authority. The live HTTP controls pass all eight tests;
strict focused lint passes. The selected security run completed with 58 passes;
the final corrected HTTP/registry run completed with 12 passes. Both supervised
clusters stopped and were removed; REPAIR-0009 closes the implementation commit's
verification-pending record.

## 2026-09-09 — Protected site operator CLI (`SIGNOFF-REPAIR.3.2.2`)

Added `rb-site` for explicit boundary/grant issuance, disabling and audited
paginated inventory. It requires a selected loopback database, verifies storage
on the repository volume and uses explicit credentials without home lookup.
The runner now supplies a matching private synthetic passfile after source
inspection exposed SQLx's fallback from the former missing placeholder.
Validation: 16 focused CLI/service/ownership tests; strengthened CLI controls
repeated with 3 passes; 13 runner controls; final strict CLI/server lint, format,
script syntax and rendered book checks. HTTP enforcement remains `.3.2.3`.

## 2026-09-09 — Explicit site-authority service (`SIGNOFF-REPAIR.3.2.1`)

Added separate site boundaries/grants, protected database-session issuance and
disabling, and a registry service that commits effects with attributable audit.
Actual-parent liveness, scope/window ceilings, usable-grant selection and a shared
transaction guard fence site revocation; tenant enrollment grants no site rights.
Native libpq controls reproduced stale prepared role membership after a wait;
fresh text-protocol checks close that path. Ten live service controls and strict
focused lint pass. The book documents capabilities, examples and integration
limits. Operator CLI and HTTP enforcement follow as separate committed leaves.

## 2026-09-09 — Bind revocation to the authorized tenant (`SIGNOFF-REPAIR.3.1`)

Grant/boundary revocation now selects and locks only a target in the authorized
tenant before changing status and epoch. Foreign targets remain 404 with victim
state unchanged. Rejected repeats no longer increment the epoch again, and two
requests forced to contend on the same grant produce one transition/epoch bump.
Validation: both foreign-target defects and the repeated-epoch defect reproduced;
34 corrected API/authority/escalation tests passed; format and focused strict
Clippy passed. Final documentation gates run in the commit workflow. Atomic final-effect auditing and
administrator-authority serialization remain explicitly owned by `.3.3`.

## 2026-09-09 — Disposable ownership at test connections (`SIGNOFF-REPAIR.2.2.2`)

Server, CLI and MCP database fixtures now validate a live runner receipt and
verify server identity on every new pool connection before fixture SQL. Missing
ownership changes from a reproduced two-row write to refusal with zero public
tables. Restore CREATE/DROP uses verified connections; CI uses the same runner
and repository-local compiler stores. Active command receipts discard stale exits.
Validation: 40 tests across eight selected suites, including forged ownership,
replacement connections, restore, migration and RLS; format, book and CI syntax
checks passed. Strict all-target/all-feature Clippy passed with warnings denied;
final restore and three malformed/missing/absent-environment controls passed.

## 2026-09-09 — Supervised focused PostgreSQL verification (`SIGNOFF-REPAIR.2.2.1`)

The runner now creates unique owned clusters, ignores caller database targets,
checks server identity before creation, and runs named suites serially. It records
process/command receipts and logs, verifies shutdown before deletion, and retains
failure evidence. A reproduced spawn/signal race is closed by deferred signals
and an exec trampoline. Test-side refusal and CI wiring remain `.2.2.2`.
Validation: 12 lifecycle controls, 4 live PostgreSQL controls, 9 existing authority
tests, syntax checks, book build and the staged doctrine gate.

## 2026-09-09 — Repository-local command environment (`SIGNOFF-REPAIR.2.1`)

Added a launcher and Makefile integration for repository-derived Cargo, build,
temporary, XDG and CLI stores. Locked cache seeding verifies archives and index
copies without deleting shared sources. Installed tools remain read-only inputs.
Validation: five focused controls; offline locked metadata (496 local packages);
core tests 49 passed, 1 intentional ignored schema writer; book built; staged
doctrine gate runs in the commit hook.

## 2026-09-09 — Corrective ownership and site-authority decision (`SIGNOFF-REPAIR.1`)

Completed the required roadmap, tracked-codebase and mdBook read before editing.
Recorded the source census and bounded repair leaves, selected explicit site-operator
authority for shared registries, and corrected progress pointers and qualification
limits. The former live status carried 42,374 bytes in 18 lines and omitted separate
Phases 5–7/9 rows; its history remains in git and phase records. This is documentation
and design work; implementation and runtime reproduction remain pending.
Validation: mdBook built; 13 doctrine checks passed; diff whitespace clean; owner/phase omission controls detected. Runtime tests remain pending.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. Retrieve the complete pre-rotation
ledger, including its earlier rotation notice, from the repository root:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That snapshot contains 130 dated entries, including the ten retained above.
Its Git blob is `0bc51d581f9158ebafcef94cfb6717464722c6cb`; exact byte/line counts,
SHA-256 identities and transition evidence are in
`docs/decisions/2026-09-09_changelog-rotation.md`. Use
`git log --follow -- CHANGELOG.md` for earlier versions. Keep the reachable Git
history when cloning or handing off; a shallow checkout may need the named commit
before retrieval. A missing object is a retrieval failure, never evidence that
history was empty.

Historical success statements describe the recorded revisions and assertions.
Current qualification is in `LIVE_STATUS.md` and the mdBook's qualification review;
open repairs remain tracked in `docs/tasks/SIGNOFF-REPAIR.md`.
