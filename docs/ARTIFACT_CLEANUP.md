# Artifact cleanup record

This file holds the date of the last generated-artifact cleanup (startup directive §8). At
startup a session reads the date below; if it is more than 24 hours old, a cleanup is due that
session. Each cleanup overwrites the entry, so only the latest one is kept; earlier entries are
in git history (`git log -p -- docs/ARTIFACT_CLEANUP.md`).

How a cleanup runs, and what it must never touch: `SIGNOFF-REPAIR.11.4.3.1.10`. Retained test
clusters are retired only through `scripts/census_pg_test_clusters.py --retire --confirm`, never
by hand, and `cargo clean` is prohibited because it would take cited evidence under `target/`.

Last cleanup: 2026-09-26 — retired 186 retained PostgreSQL test clusters through the guarded instrument (`scripts/census_pg_test_clusters.py --retire --confirm`, 10,089,259,915 bytes; residue verified). Kept by the guard: 261 clusters — cited evidence, runs under an hour old, and one whose recorded postmaster pid is now another program's (`caffeinate`: a reused pid, kept conservatively). Not touched, reported for a decision: 10 stray clusters outside `target/pg-tests`, 7 of them this session's own stopped harness clusters under `target/r11_3_5/` (about 640 MB); the build cache (`target/debug`), which is removed only on the director's word.
