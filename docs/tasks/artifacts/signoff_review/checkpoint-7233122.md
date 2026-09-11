# The first complete pre-push checkpoint

Owner: `SIGNOFF-REPAIR.11.4.3.1.2`; source `7233122`. Receipts:
`target/checkpoint-ci/full-7233122/summary.txt` and its per-command logs.

Every earlier attempt stopped: `7e01097` at two browser timing witnesses,
`b0cddfe` at `identity_store` fixture cleanup, `8d1504d` at a state-writer lock,
`ec8df08` at two PDF tests, `165cb3a` at `04-pg-demo`, `5c8609e` at `02-check`.
This is the first run in which all eight commands returned 0.

## Receipts

| Command | rc | Seconds |
| --- | ---: | ---: |
| `01-build` workspace binaries | 0 | 52 |
| `02-check` format, strict lint, workspace tests with the pinned browser | 0 | 2,523 |
| `03-python` control modules | 0 | 22 |
| `04-pg-demo` full owned PostgreSQL collection with `--demo` | 0 | 4,034 |
| `05-gate` thirteen doctrines | 0 | 3 |
| `06-deny` pinned cargo-deny | 0 | 3 |
| `07-gitleaks` pinned Gitleaks over configured history | 0 | 4 |
| `08-book` mdBook | 0 | 0 |

The driver was `exec`'d, so the reported status is the driver's own. The
`165cb3a` run was announced as "exit code 0" while its receipt said
`CHECKPOINT STOPPED`, because that invocation ended with an `echo`; the receipts
remain the authority regardless.

## Re-derived, not read off the exit code

- `04-pg-demo`: 41 runner invocations covering all 40 registered suites, 291
  tests passed and 0 failed across 42 result blocks. The `mcp` suite runs as
  `cargo test --locked -p reasonbraid-mcp`, a whole-package invocation rather
  than `--test mcp`.
- The demonstration ran and reported `ALL acceptance checks passed`, with its
  bundle under `target/demo/20260911-151220/evidence`.
- `06-deny`: `scanner.json` records cargo-deny **0.20.2**, `scope: gate`,
  `exit_code: 0` — the real gate, not `--verify-only`.
- `07-gitleaks`: Gitleaks **8.30.1**, `scope: gate`, `exit_code: 0`, over the
  configured history with its two reviewed historical fingerprints.
- `08-book`: zero seconds is rounding; the log shows mdBook writing the HTML.

## Falsified

A green suite that never executed the interesting path is the failure mode this
project has already met twice, so the pass was attacked before being published:

| Question | Answer |
| --- | --- |
| Did any suite silently skip for a missing database? | `SKIP: DATABASE_URL unset` appears 0 times |
| Did any test skip for a missing browser? | `browser_roundtrip.rs` ran **16** tests against the pinned Chrome for Testing |
| Were tests ignored? | 3, all in `02-check`: the deliberately env-gated `RB_LIVE_CLAUDE` and `RB_LIVE_CODEX` live-provider dispatches |
| Did every registered suite run? | 40 expected, 40 ran |
| Is the claim about the source that is checked out? | `HEAD` is `7233122` and `git status --porcelain` is empty |

## What this does and does not establish

It establishes that the full local checkpoint passes on this source. The
recorded policy in `docs/decisions/2026-09-09_public-repository-policy.md`
conditions the already-authorized push on exactly that.

It does not establish remote CI, which has never run and must be consumed after
the push. It does not close G6/G7 Internet qualification, the name-clearance
gate, or the license decision. It does not retroactively qualify the historical
phase closures under corrective review, and it does not make the open repair
leaves disappear: `.7.4.2`, `.7.2.1`, `.7.3.3.4`, `.11.4.3.1.2.15` and `.11.5`
remain open, and `.11.4.3.1.2.15` specifically owns the roughly 2,982 seconds of
`02-check` wall time that neither compilation nor test execution accounts for.

Nine local repairs stand between the first stopped attempt and this pass:
REPAIR-0060 and 0063–0064 closed the production R2 input boundary, 0065 restored
a recreated schema's grant and made a site refusal honest, 0067 removed a clock
from fixture naming, 0068 replaced colliding evidence identifiers, and
0061/0062/0066 mechanized three document invariants.
