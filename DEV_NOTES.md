# DEV_NOTES.md

## _(2026-09-06)_ — G0 contract drafts: ID-scheme gap closed, drafts live in `spec/`

- `ROADMAP.md` §19.1 names six illustrative requirement-ID prefixes (ID / AUTH / DELIV / RES / POL / SEC) but §20.2 gates G0 on five boundaries — identity, authority, **thread**, delivery, **budget**. Thread and budget had no prefix, so the "stable IDs" acceptance could not be met without a choice.
- Decided: add `THREAD-*` and `BUDGET-*` (first-class now); reserve `RES-*` (Phase 4), `POL-*` (Phase 6), `SEC-*` (Phase 7). Gap-fill for backlog 4, not a feature — the frozen roadmap is untouched.
- Contract drafts live under `spec/` (beside the code, §7.1/§19.1), not `docs/`; each is headed "draft — not normative".
- Promoted to `docs/decisions/2026-09-06_g0-contract-id-scheme.md` (`answers:` present).

## _(2026-09-06)_ — supply-chain skeleton

- Added `deny.toml` (cargo-deny: advisories/bans/licenses/sources), `.github/workflows/supply-chain.yml` (cargo-deny + gitleaks secret scan), and `docs/ci.md`; the Makefile gained `make deny` / `make secret-scan`.
- The `deny.toml` schema was copied from the authoritative cargo-deny `main` template (EmbarkStudios repo), not reconstructed from memory: current shape is `[graph]`/`[advisories]`/`[bans]`/`[bans.std-replacements]`/`[sources]`/`[licenses]`, with no `version` key.
- `cargo-deny` and `gitleaks` are NOT installed locally; the Makefile targets forward to them and CI installs them. Local runs need `cargo install cargo-deny` / `brew install gitleaks`.
- Validated: `deny.toml` parses (python3 `tomllib`), `make -n deny` → `cargo deny check`, `make -n secret-scan` → `gitleaks detect --source . --redact`, `make gate` 13/13, `make check` 1 test ok.

## _(2026-09-06)_ — external dependency ledger skeleton

- Created `docs/dependencies/external-ledger.yaml` from `ROADMAP.md` §7.4: one entry per protocol/SDK/CLI/provider/harness, `checked_at` dated, a `revalidation_trigger` per row.
- Stubbed MCP, A2A, Codex, and Claude rows from the 2026-09-04 corrected baseline (§28.1). `license` is `"unverified"` until a spike records it from package metadata — never asserted from memory.
- Validated with `ruby -ryaml` (4 entries, required fields present) so the file parses clean before it is committed.

## _(2026-09-05)_ — KICKOFF.md is a companion, not a second roadmap

- Director dropped both `ROADMAP.md` (v0.4.1 master) and `KICKOFF.md` (Phase 0 execution).
- They are one pair: the master is frozen scope/gates; the kickoff is the Phase 0 task board.
- Promoted to `docs/decisions/2026-09-05_kickoff-companion-to-roadmap.md` (`answers:` present).

## _(2026-09-04)_ — a template's trial must include the first commit

- Every gate was green on the generated project and the first commit still failed: the doctrines judge STAGED
  code, and nothing had been staged until the user tried. Trial the path a user walks, to its end.
- `grep -c` prints `0` and exits 1. `$(grep -c … || echo 0)` therefore yields `0⏎0` — a second line — which
  here started a flush-left line inside a checklist bullet and hid its evidence from the box-scoped extractor.
  Capture the count, then default the empty case; never append a fallback to grep's own output.

## _(2026-09-04)_ — a green gate that judges nothing is the class a template must not ship

- Two of the four doctrine ports in `.2.6` were wrong on first run and their own RED self-test arms said so:
  a `python3 - <<'PY'` detector whose stdin was the heredoc (every arm read 0 rows), and a `grep -c … | grep -qx 0`
  control under `pipefail` (`grep -c` prints 0 and exits 1). A self-test with only GREEN arms would have passed both.
- The neutrality bar is measured, not felt: `grep -ciE 'grammar|parser|…'` over each ported script → 0, after the
  generic uses of "corpus" and "grammar" were re-worded ("tree", "syntax") so the count means what it says.

Detailed technical notes — root cause, implementation, validation — per slice. The
engineering-continuity surface (not the public docs; that's `docs/book/`). Newest first.

## _(YYYY-MM-DD)_ — bootstrap

Repo created from the `bedrock` template: durable 4-layer memory, task-tree tracking, the
strict commit workflow, and the mechanical doctrine enforcer are in place and enforced by
git hooks + CI. No project code yet.
