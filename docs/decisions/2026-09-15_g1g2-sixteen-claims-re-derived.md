---
answers:
  - Which of Phase 1's G1–G2 claims still re-derive against the repaired code?
  - How many deferrals does the Phase-1 gate record have?
  - Why do `make deny` and `make secret-scan` fail today?
  - What has been running the supply-chain gates, and how long have they been red?
  - Were the Phase-1 deferrals' revisit triggers ever checked?
  - Does the Phase-1 gate's outcome change?
---
# G1–G2's sixteen claims, re-derived: three live failures, and a deferral nobody revisited

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2`
- **Date:** 2026-09-15
- **Supersedes:** nothing. It **adds** to `2026-09-07_phase1-gate-record.md`, which is
  left byte-unchanged (`docs/decisions/` supersedes rather than mutates).

## Context

`.11.4.7` found that every gate record counted the evidence its suites then carried,
and that none could know what a later full source read would find. This is the second
of four re-derivations. The Phase-1 record (`PHASE-1-G1G2`, evidence revision
`ecceb8d`, 2026-09-07) makes **sixteen** countable claims: seven §26.1 acceptance
rows, four G1 lines and five G2 lines.

⭐ **Ordering is exact rather than rhetorical.** **203** `REASONBRAID-{REPAIR,DOC}`
commits postdate the record, of which **111** touched product source. So every verdict
below is about work that came after, and none has to argue about sequence.

⚠️ **The record misidentifies its own commit, and this is worth a sentence because it
is the same class as everything else here.** Its header reads "Evidence revision:
`ecceb8d` … the `.1.8.2` close commit (this record ships in it)". `ecceb8d` is
**PHASE-1.8.1**'s commit (02:15:05, the audit-reconstruction demo leg); the record
actually ships seven minutes later in `7e76030`, PHASE-1.8.2. ⛔ It changes no verdict
— the counts above are identical from either base — and it is recorded only because a
self-referential claim that nobody re-derives is exactly what this whole activity is
about. ⭐ The record has otherwise **never been edited**:
`git log --oneline 7e76030..HEAD -- docs/decisions/2026-09-07_phase1-gate-record.md`
returns nothing, so supersede-never-mutate held.

## The verdicts

Each is from the closed set `stands` / `narrowed` / `must be re-earned`, and each names
the command that produces it.

### The §26.1 acceptance table

| Row | Verdict |
| --- | --- |
| (1) no manual message relaying between agents | **stands** — `git log ecceb8d..HEAD -- crates/reasonbraid-adapter/src/claude.rs crates/reasonbraid-adapter/src/codex.rs` returns **zero** repairs; the agent content still comes from the adapter |
| (2) restart/reconnect loses no accepted command and creates no duplicate domain effect | 🔴 **must be re-earned — and it is the ONE of the sixteen that is still an OPEN defect** (see below) |
| (3) offline inbox and resume cursor work | **must be re-earned** — five live defects inside its own subject, all since repaired: `.4.2.3` (a lapsed lease renewed by an in-flight heartbeat), `.4.1.3` (a revoked node renewing its lease for ever), `.4.2.10` (the wake gate's uncontrolled states), `.3.3.4.10.3` (inbox mutations crossing the tenant boundary, one destroying another tenant's rows), `.3.5.3` (the inbox inspection reading another tenant in full) |
| (4) agent role, incarnation, harness, model/provider route and attempt remain distinguishable | **must be re-earned** — `.4.1.5` (a replacement not ending the old machine's session, host and incarnation), `.4.2.9` (a rotated identity not written where the next start looks), `.4.1.4`. ⭐ Its own named deferral #4 is **discharged**: `INSERT INTO incarnations` (`node_channel.rs:2221`) and `INSERT INTO runs` (`api.rs:6517`) both exist |
| (5) spend and uncertainty are visible | **narrowed** — the budget read surface is unchanged (`api.rs:820`), but `.3.3.4.9` measured the spend breaker's two administrative verbs mutating **on the connection pool, outside any transaction, under no guard, recording nothing**. Spend was visible; changes to the spend CONTROL were not |
| (6) the system can conclude `inconclusive` with minority/unresolved items | **stands** — `git log -S "Inconclusive" ecceb8d..HEAD -- crates` returns **0** repairs (positive control: the same command on `idempotency_key` returns **3**) |
| (7) all state is inspectable through supported CLI/UI, not database surgery | **narrowed** — the surfaces still cover the state and the credential-oracle caveat stands. ⚠️ What the review found is the OPPOSITE failure: supported surfaces returned **more** than the caller's own tenant (`.3.5.3`, `.6.1.1`). Inspectability held; confinement did not, and this row cannot be read as evidence for it |

### G1 — component gate

| Line | Verdict |
| --- | --- |
| unit baseline (39 offline + 12 live-PG + CLI e2e, all green) | 🔴 **must be re-earned — the suite does not pass, and does not finish.** `cargo test --all --locked` rc=**101**, ABORTING at the 11th test binary: only `a2a`, `adapter` and `browse` run; `cli`, `core`, `node` and `server` never execute. 10 suites ok / 80 tests, then `browser_roundtrip` 16 passed / 1 failed. ⛔ Reproduced on an idle machine (81.30 s single-threaded), so not a load flake. ⚠️ The claimed shape is also gone: 39 + 12 suites then, **76** test files now. Owner: `SIGNOFF-REPAIR.11.4.7.2.4` |
| property-flavored baseline; **fuzz deferred** | **must be re-earned** — the baseline itself stands (KP-1…KP-9 all nine still in `journal_kill_points.rs`, 34 transition-table sites, 2 grant-canary sites). ⛔ But the FUZZ deferral's trigger has **fired and nobody revisited it** (below) |
| dependency + license checks (`make deny` → all ok, rc=0) | 🔴 **must be re-earned — LIVE FAILURE.** `cargo deny check` returns rc=**1**: `advisories FAILED, bans ok, licenses ok, sources ok`. RUSTSEC-2026-0285, `rustls 0.23.43`, fixed in ≥0.23.45 |
| secret scan (`make secret-scan` → no leaks, rc=0) | 🔴 **must be re-earned — LIVE FAILURE.** `gitleaks detect` returns rc=**1** with one finding |

### G2 — vertical slice gate

| Line | Verdict |
| --- | --- |
| real durable stores (PostgreSQL + per-node SQLite) | **stands** — unchanged, and exercised by every live-PG suite |
| node journal (durable boundary records, `outcome_unknown` recovery, read-only `rb-journal`) | **narrowed** — `.3.4.3` and `.3.4.3.1.2` corrected the freshness window to run from the earlier of the two clocks, so the recorded boundary times the record relied on were computed against one clock only |
| adapter (two genuinely distinct real adapters **live-qualified** + the deterministic fake) | **narrowed** — two distinct real adapters still exist (`claude.rs` 400 lines, `codex.rs` 328) plus the deterministic oracle, all passing the conformance suite. ⛔ But the record says *live-qualified* while the project's own `docs/compatibility-matrix.md` now records **both** real-provider live runs as `untested` — "no CI measurement" — among **9** `untested` cells |
| recovery demonstration (the two-host demo's real kill points) | **must be re-earned** — the demo passed 30/30 while `.4.2.2`, `.4.2.3`, `.4.2.4` and `.4.1.3` were live on paths it does not drive. ⛔ This is the record's own lesson turned on itself: a demo asserting a property is not the property holding |
| exit-gate clauses (killed node resumes without duplicated effects; provider ambiguity visible; accepted messages appear once despite redelivery) | **must be re-earned** — clause 3 is `.3.4.6`'s open defect restated; clauses 1–2 inherit rows (2) and (3) |

## The one still open

🔴 **Row (2) is the only one of the sixteen whose defect is not repaired.**
`SIGNOFF-REPAIR.3.4.6`, read at HEAD: `api.rs::request_hash(operation, principal,
body, authority)` hashes those four and nothing else. The thread id arrives as a
**path segment** and the typed bodies carry `tenant_id` but no `thread_id`, so the
target is in neither hashed input; `migrations/0001_atomic_transaction.sql:39` then
makes `idempotency` `PRIMARY KEY (tenant_id, idempotency_key)` — a **tenant-wide** key.

⛔ So the same principal, operation, body and idempotency key aimed at a **different
thread in the same tenant** hashes identically, and the second request **replays the
first thread's stored result** instead of answering `idempotency_mismatch`. That is
precisely "a duplicate domain effect" — arrived at from the other direction, as a
wrong answer rather than a repeated one.

## Three things are red right now, and that is the bigger finding

⚠️ **`make deny`, `make secret-scan` AND the workspace test suite all fail today.** The
first two have not been run by anything since Phase 1; the third aborts the run at its
11th test binary, so four crates never execute.

- `grep -c 'deny\|secret-scan\|gitleaks'` returns **0** for `scripts/check_doctrines.sh`,
  **0** for `scripts/check_doctrines.project.sh` and **0** for `.githooks/pre-commit`.
  The doctrine gate — the thing that runs on every commit — does not touch them.
- They live only in `.github/workflows/supply-chain.yml`, which runs in **remote CI**.
- 🔴 Remote CI **has never run** (blocker C1), and `origin/main` is at 2026-09-12 while
  the gitleaks finding entered the tree on **2026-09-13**, in the unpushed range.

⭐ **So C1 is not only a limit on what may be CLAIMED. It is the reason two supply-chain
gates have been red with nobody able to see it.** That is a different and worse fact
than "the authoritative run has not happened yet", and it is the finding this
re-derivation exists to surface.

### The advisory

`RUSTSEC-2026-0285` — rustls accepted TLS 1.3 handshake messages sent at the wrong
encryption level when they followed a key-changing message in the same record. The
transcript stays authenticated, so this is not handshake forgery; the effect is that a
peer can send in plaintext what should have been encrypted without rustls rejecting it.

⚠️ **It is dependency drift, not advisory-database drift against a fixed graph.**
`git show ecceb8d:Cargo.lock | grep -c '^name = "rustls"$'` returns **0** — rustls was
not in the graph when this gate passed (positive control: the same command for `tokio`
returns **1**). Owner: `SIGNOFF-REPAIR.11.4.7.2.2`.

### The leak

One finding, `generic-api-key`, at
`crates/reasonbraid-core/tests/delegation_representation.rs:74` — the literal
`idempotency_key: "<redacted>"`, entropy 3.875 — introduced 2026-09-13 by REPAIR-0134.

⭐ **It is a false positive: a test fixture, not a credential.** That does not make the
gate green. `.gitleaksignore` carries **exact historical fingerprints only** and says so
in its own header, and the CI wrapper runs the same pinned gitleaks 8.30.1 with the same
arguments, so CI would fail identically. Owner: `SIGNOFF-REPAIR.11.4.7.2.3`.

## The deferral count, corrected

⛔ **The record says "five named deferrals" twice and lists six.**
`grep -c "^| [1-6] |"` returns **6**; the Outcome line and the final `answers:` bullet
both say five. `git log -S` shows the sixth row — TLS/mTLS, supervision units,
containers, PG automation, config files — shipped in **the same commit as the sentence**
(`7e76030`, the Phase-1 gate package), so the record was internally inconsistent the day
it was written rather than drifting later.

**The count is six.** The original record is byte-unchanged; this record carries the
correction.

## The deferrals' triggers have fired, and nobody re-derived them

🔴 **Every one of the six deferrals names a revisit trigger, and no mechanism has ever
checked whether any of them fired.** Spot-measured here rather than assumed:

| # | Deferred | Trigger | Measured today |
| --- | --- | --- | --- |
| 4 | Incarnation/run row **writers** | Phase 2 identity | ✅ **discharged** — `INSERT INTO incarnations` and `INSERT INTO runs` both exist |
| 5 | **Fuzz baseline** | "the first untrusted parser (Phase 4's resource packs)" | 🔴 **fired and untouched** — Phase 4 has a gate record; `fetcher.rs`, `git.rs` and the `reasonbraid-extract` / `-browse` crates now parse untrusted input; `git ls-files \| grep -ic fuzz` returns **0**; the word `fuzz` appears in exactly **one** decision record — the Phase-1 one that deferred it — and **zero** times in Phase 4's |

⚠️ **The other four are not adjudicated here**, and saying so is the point: this
re-derivation owns sixteen claims, and "were the deferrals discharged" is a different
question that deserves its own measurement rather than a paragraph at the end of
somebody else's leaf. Owner: `SIGNOFF-REPAIR.11.4.7.2.1`.

⭐ **A deferral with a trigger nobody checks is an omission with extra steps.** The
Phase-1 record was careful — it refused to call the deferrals omissions precisely
because each had an owner and a condition that would revive it. What no one built was
the thing that notices the condition.

## Consequences

- ⛔ **The gate's OUTCOME is not restated here.** This record re-derives claims; whether
  Phase 1 still exits is the release authority's call on the corrected evidence, and
  inventing a new outcome would be exactly the over-reach `.11.4.7` warns against.
- **Two gates are red and both now have owners**, which is the difference between a
  finding and a repair.
- ⚠️ **Not claimed: that re-earning the eight is repair work.** Except for row (2), the
  defects are repaired; what is missing is the coverage measurement that would let each
  claim be counted again.
