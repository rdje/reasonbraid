# 2026-09-07_r1-git-acquisition-contract.md

## Context

`PHASE-4.3` (pack R1 — public Git, backlog 33) decomposed at the census
seams: NOTHING fetches Git (`git grep -c "gix|git2|gitoxide" efc9ba8 --
crates/` → rc=1; no git library in the lock or the registry cache). The
§12.5 rules need a typed contract before the acquisition machinery (`.3.2`)
exists — the same order the `.2` lane used (the policy before the fetcher).

## Decision

- **The library is `gix` (gitoxide).** Measured `2026-09-07`:
  `cargo add --dry-run gix` → v0.87.1, a pure-Rust crate family (32
  activated features, no C); `cargo add --dry-run git2` → v0.21.0 whose
  feature list is `openssl-probe`, `openssl-sys`, `vendored-libgit2`,
  `vendored-openssl` — a C build surface the lean supply-chain doctrine
  rejects (the workspace already chose ring over aws-lc-rs and miniz_oxide
  over zlib-C for the same reason).
- **The transport rides the classified reqwest stack.** The gix
  reqwest-backed transport (the `http-client-reqwest` backend) is a
  REQUIREMENT, not a preference: every git dial must pass the `.2.1`
  destination classification at both layers (the pre-flight that names the
  class + the classified DNS belt) — the same public-only rule the R0
  fetcher enforces. The `.3.2` leaf verifies the client-injection seam
  mechanically; the fallback if the seam is absent is the pre-flight-only
  gate (the belt is the second layer, never the only one).
- **The ref grammar.** `https://<host>/<path>[#<ref>]` — the fragment
  carries the selector (a branch, a tag, or a full commit sha). A bare URL
  resolves the default branch's tip. The receipt records BOTH the
  requested URL/ref AND the resolved immutable commit — the immutability is
  the recorded commit, never a promise about the URL.
- **The budget vocabulary** (typed ceilings, the `FetchLimits` pattern):
  total object count, file count, path depth, decompressed size, and total
  transfer bytes — each refusal names its ceiling.
- **The refusal list — ALL default-deny, named, never a prompt, never a
  silent skip:** submodules, hooks, filters, alternates, external
  diff/clean drivers, Git LFS. A repository that requires any of them is
  refused with the name.
- **No checkout execution.** The worktree is never materialized; hooks and
  filters never run. Reading objects is the whole acquisition — build/test
  execution is a separate sandboxed tool action with separate authority
  and budget (the §12.5 closing rule).
- **The test seam.** The R1 acquirer exposes the `.2.2` seams (the
  injectable resolver + policy) so the wire tests stay OFFLINE: the
  loopback origin holds a local bare repo and the test policy allows it —
  the SSRF proof is the same measured refusal.

## Consequences

- The `.3.2` acquisition and the `.3.3` receipt implement this contract
  verbatim; a deviation is a contract change, not an implementation detail.
- The R1 registry entry (`.3.3`) declares the `git` scheme with egress
  `listed` (the public-only destination classes) — the same honest-claims
  discipline as the R0 entry.

answers:

- **A git library is a supply-chain decision first.** The acquisition
  semantics both libraries provide are equivalent; the C build surface
  (libgit2 + openssl) is the differentiator the lean doctrine decides on.
- **The ref selector lives in the fragment.** The `ResourceReference`
  already carries `fragment_or_selector` — the git pack consumes it, so no
  new URL grammar or table column is needed.
- **Refusal before fetch.** The default-deny list is enforced at the
  configuration level (never at content-read time): a repo carrying a
  submodule pointer must be refused by its advertised requirements, not
  discovered mid-clone.
