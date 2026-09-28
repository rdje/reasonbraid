# Artifact cleanup record

This file holds the date of the last generated-artifact cleanup (startup directive §8). At
startup a session reads the date below; if it is more than 24 hours old, a cleanup is due that
session. Each cleanup overwrites the entry, so only the latest one is kept; earlier entries are
in git history (`git log -p -- docs/ARTIFACT_CLEANUP.md`).

How a cleanup runs, and what it must never touch: `SIGNOFF-REPAIR.11.4.3.1.10`. Retained test
clusters are retired only through `scripts/census_pg_test_clusters.py --retire --confirm`, never
by hand, and `cargo clean` is prohibited because it would take cited evidence under `target/`.

Last cleanup: 2026-09-28 — retired 18 PostgreSQL test clusters through the guarded instrument (`scripts/census_pg_test_clusters.py --retire --confirm`, 1,057,583,685 bytes; residue verified; 262 kept by the guard as cited or reduced evidence; `STRAY: none`). `scripts/census_retained_fixtures.py`: nothing reducible (6 kept whole: cited, or under the 1-hour floor). Three uncited retained browser runs in `target/ci-browser/` reduced to their receipts under `.11.4.8`'s rule, keep the receipt and drop the reproducible payload: `runtime/` and `archive.zip` removed, `browser.json`/`version.log` kept, each re-checked for no running process and no tracked citation (1,225,620 KiB; evidence `target/cleanup_2026-09-28/`). Kept: the cited run `mac-arm64-0281kcb0`, the demonstration bundles (145 MB), and the build cache (`target/debug`), which is removed only on the director's word.
