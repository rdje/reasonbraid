# Artifact cleanup record

This file holds the date of the last generated-artifact cleanup (startup directive §8). At
startup a session reads the date below; if it is more than 24 hours old, a cleanup is due that
session. Each cleanup overwrites the entry, so only the latest one is kept; earlier entries are
in git history (`git log -p -- docs/ARTIFACT_CLEANUP.md`).

How a cleanup runs, and what it must never touch: `SIGNOFF-REPAIR.11.4.3.1.10`. Retained test
clusters are retired only through `scripts/census_pg_test_clusters.py --retire --confirm`, never
by hand, and `cargo clean` is prohibited because it would take cited evidence under `target/`.

Last cleanup: 2026-09-25 — retired 210 retained PostgreSQL test clusters through the guarded instrument (11,249,710,924 bytes; `df` shows ≈10.7 GiB freed); kept 259 clusters (256 reduced to receipts, 3 cited), the 3 cited stray clusters, 527 uncited top-level logs (5.6 MB) and the 165 GiB `target/debug` build cache.
