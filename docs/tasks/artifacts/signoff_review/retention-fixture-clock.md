# Retention fixture expiry uses recorded creation time

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.9`; REPAIR-0052. Predecessor:
`e5157134c8f6121062bf4012d17d71e8126d8bcc`. Raw evidence:
`target/retention-clock-controls`. Final focused qualification passes; full checkpoint remains incomplete.

## Root cause and baseline binding

The original retention test submits expiry at 2026-09-10T00:00:00Z and expects at
least one new temporary snapshot to be tombstoned. Git blame binds that cutoff
to 5083d9cf under PHASE-4.6.4. The exact original-fixture comparison in REPAIR-0049
reproduces zero tombstones, with created_at values 2026-09-10T06:01:03.342530Z
(temporary) and .344175Z (standard). Database observation at .435009Z independently
finds both the one-day and thirty-day due predicates false. Expiry is correctly
not due for either row. Preserve the stopped original run-jgx210ws.

The current pre-edit profiles.rs hash matches the separately failed full profiles
receipt in target/node-fixture-controls/remaining-profiles.json (thirty pass/one
fail). Its exact current fixture produced the same tombstoned=0 assertion. The
original-fixture comparison rules out the preceding cleanup migration as cause.
The full suite's stopped run-7d88gdkg is preserved, without claiming its failed
individual test rows survived later fixture cleanups. The historical Phase-4 leaf
now records the calendar dependency while retaining its original scoped result.

## Selected test-only correction

Change only the existing retention/freshness test. Read actual created_at values
for its temporary, standard and newly explicit audit snapshots. Drive the existing
HTTP at override around their own recorded TTL boundaries, with no host clock
change or real-TTL sleep. The source predicate is strict: equality is not expiry.
The test requires exact counts instead of an unrelated-row-tolerant lower bound.

The temporary sequence is before boundary (zero), exact boundary (zero), one
microsecond after (one), then repeat (zero). Complete snapshot rows must remain
identical on no-change steps. The standard row remains live during this sequence;
the original license, freshness-list and same-content replay assertions remain.
Replay must preserve original created_at. At the standard thirty-day boundary
expect zero, then one microsecond after expect exactly one. Advancing the fixture
expiry observation beyond both finite TTLs keeps the exact audit row unchanged.

This qualifies the fixture and the named existing class behavior in an isolated
owned database. It does not change or qualify global expiry authorization/scope,
caller-controlled clock policy, atomic object writes, byte retirement or actual
freshness-horizon refresh; those remain SIGNOFF-REPAIR.7.4. Production source,
schema and runner registry remain unchanged.

## Runtime results

The focused retention test passes (one pass/thirty filtered, no skips/ignores,
exit zero) in 23.70 seconds of test body and 334.329802 seconds of command time.
Compilation reports 4m 28s; the one native observation at 261.699600 seconds sees
Cargo and rustc idle. The compiler completes naturally before the diagnostic
sample can revalidate its target; ps returns 1, no sample is dispatched and no
stack report exists. All diagnostic results are consumed. This establishes no
host-wait cause; preserve the observation/preflight under the existing .11.2 owner.

All 31 profiles tests then pass with unchanged source, no skips/ignores, exit
zero: 28.39-second body and 28.763541-second command. The focused test repeats one
of those 31; do not report 32 distinct controls. Successful owned databases
run-vmfetjx2 and run-3h_oo6ae are stopped/removed, preserving original failures.

A separate post-focused SQL observation sees exactly three snapshots: audit has
no deletion or refresh timestamp; temporary and standard have the retention
reason and non-null deletion timestamps; standard alone has refreshed_at. Creation
times are independently retained in the receipt. Exact intermediate counts and
full no-change row comparisons execute inside the tracked regression, not merely
in this final census. The synthetic expiry observation controls eligibility;
deleted_at records the database's actual processing time, not that future cutoff.

The behavior can be re-derived with tracked sources and runner:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh profiles
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-server --all-targets --all-features --locked -- -D warnings
```

Falsification is the exact current/original-fixture failure and independent stored
creation/cutoff predicates; the new boundary checks distinguish strict expiry,
repeat behavior and class identity. Durable regression logic remains in the same
registered profiles test. Local ignored timing/process/hash receipts are historical
observations, not portable future-cleanliness or performance gates. Production
retention qualification and full checkpoint remain separately owned.

## Final verification

Format, strict all-target/all-feature server Clippy and book pass, exit zero:
0.645258, 47.225630 and 0.257364 seconds respectively. Final native verifier exits
zero: all seven recorded groups are absent; both successful databases are gone;
original failures remain stopped. Exact source comparison confines the sole code
change to the existing retention/freshness function and preserves 340 other
non-Markdown files, including all production/schema/runner code. The original
fixed-cutoff predicates rederive false independently; final SQL observes the
expected three distinct classes. Nine rendered book markers match, README remains
52 lines/2017 bytes and LIVE_STATUS categories stay unchanged.
