# Authority writer coverage, re-derived

Owner: `SIGNOFF-REPAIR.3.3.4.3.4`; REPAIR-0103. Predecessor: `cfa0f67`.
Supersedes nothing: `docs/tasks/artifacts/signoff_review/tenant-authority-paths.md`
remains the `.3.3.4.1` census at its own baseline, and this record is the
comparison against it.

## The instrument, not just the number

`.3.3.4.1` recorded 42 direct named-call locations over 101 tracked Rust source
files, with a corpus SHA-256. That is a measurement someone took once. The
producer lived in the session that took it, so the only way to re-measure was to
rebuild the predicate from the table — and a rebuilt predicate that differs by
one name makes every later comparison incomparable while still looking like a
comparison.

`scripts/census_authority_paths.py` is now that producer, tracked, with a
`<sha>` mode so any commit can be measured. Its own correctness check is
reproduction of the recorded baseline:

```bash
python3 -B scripts/census_authority_paths.py 1ba6184
# 101 files · 1,749,975 bytes
# SHA-256 340c4af65db88e48496797c650bbce851bdfde47aaaefac5d93565cd38d26c2c
# 42 direct named-call locations
```

All four figures reproduce exactly. Only because they do is the current
measurement comparable to the baseline at all — this is the step that makes the
rest of this record evidence rather than a second opinion.

## What moved

| | baseline `1ba6184` | current | change |
| --- | --- | --- | --- |
| tracked Rust source files | 101 | 111 | +10 |
| corpus bytes | 1,749,975 | 1,971,693 | +221,718 |
| direct named-call locations | 42 | 39 | **−3** |

The corpus grew while the direct authority call sites shrank. Per file and call:

| file | call | base | now |
| --- | --- | --- | --- |
| `authority.rs` | `bump_revocation_epoch` | 2 | 0 |
| `authority.rs` | `insert_boundary_in_tx` | 1 | 0 |
| `authority/issuance.rs` | `bump_revocation_epoch` | 0 | 2 |
| `authority/issuance.rs` | `insert_boundary_in_tx` | 0 | 1 |
| `api.rs` | `create_grant_in_tx` | 2 | 0 |
| `authority.rs` | `create_grant_in_tx` | 1 | 0 |

Two different events, and separating them is the point of a per-file diff rather
than a total:

- **A module split, net zero.** Three calls moved from `authority.rs` into the
  new `authority/issuance.rs`. Nothing migrated; the code has a new home.
- **A bridge that genuinely went away.** All three `create_grant_in_tx` call
  sites disappeared, and `git grep -n "\bcreate_grant_in_tx\b" -- 'crates/**/*.rs'`
  now returns **nothing at all** — not even a declaration. The guarded issuance
  service replaced it during `.3.3.4.3.2`/`.3.3.4.3.3.2`, and the bridge was
  removed with its last caller rather than left behind.

## Obsolete bridges: none remain

This leaf owns removing bridges whose consumers have migrated. The measurement
says there is nothing to remove, which is a result rather than an absence of
work.

A census over every function declared in the authority modules — `authority.rs`,
its five submodules, and the three `site_authority` modules — finds **86
declared functions**, of which 10 have zero non-declaration references. All 10
are `#[test]` functions in `authority/evaluation_tests.rs`, reached by the test
harness rather than by name, so they are false positives of a lexical predicate
and not dead code.

Zero production functions in the authority modules are declared without a
caller.

## The remaining unguarded paths, re-verified against their recorded owners

Guarded connections — a `tx.connection(tenant, GuardMode::…)` — number 18:
`api.rs` 8, `api/bootstrap.rs` 4, `authority/issuance.rs` 6.

The two families `.3.3.4.1` left open are still open, still unguarded, and still
owned where it said:

| path | evidence at this commit | recorded owner |
| --- | --- | --- |
| `revoke_node` (`api.rs:1524`) | calls `authority::bump_revocation_epoch(&mut tx, …)` at `:1571` on a RAW transaction, not `tx.connection(…, GuardMode::…)` | `.3.3.4.10` |
| `import_profile_card` (`api.rs:4791`) | `authorize_tenant_admin` then proceeds to its writes with no guard | `.3.3.4.11` |

The ownership table in `tenant-authority-paths.md` is therefore accurate at this
commit and needs no correction — which is itself worth recording, because an
ownership table that nobody re-checks is the same failure shape as a frontier
column that nobody derives.

## The compatibility set, live

```bash
RB_DEMO=0 bash scripts/run_pg_tests.sh authority authority_transaction \
    authority_issuance enrollment_transaction bootstrap_recovery \
    migration_upgrade command_api
```

`rc=0`, **109 tests across seven suites** — 22, 16, 14, 9, 11, 4 and 33 — with
`pg-tests: stopped and removed target/pg-tests/run-touut7ci`. The runner's status
was captured BEFORE any filter, after a run earlier in this session reported exit
0 through a `grep` while a test had failed.

`migration_upgrade` is in the set deliberately: this leaf's subject is a module
split plus a removed bridge, and the question a split raises is whether an
upgrade path still reads what earlier writers wrote.

## Limits, unchanged from the original

This is a bounded lexical census. It does not see dynamic dispatch, aliasing, or
arbitrary SQL, and it is not a security-boundary proof. A guard integration is
not evidence that separately owned caller, target or consent policy passes.
`.3.3.4.1`'s statement of these limits stands as written; re-running its
instrument does not widen them.
