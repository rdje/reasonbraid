# The SDK compatibility matrix (`PHASE-8.4.2`)

> The `.4.1` schema (`docs/decisions/2026-09-08_sdk-compatibility-matrix-schema.md`)
> governs this document: the six columns, the measured-only fill (a cell
> comes from a run, never from a sibling), the explicit `untested` (never
> blank, never inferred from the SemVer). `scripts/check_compatibility_matrix.sh`
> (wired into the doctrine gate's project slot) mechanically verifies the
> token + the evidence artifacts.
>
> Scope: the adapter surfaces (the contract's implementers) and the resolver
> surfaces (the §12.2 packs). The A2A/MCP facades carry their own
> demonstrations in the `.2`/`.3` lanes.

| sdk_version | surface_id | protocol_profile | platform | qualification | evidence |
| --- | --- | --- | --- | --- | --- |
| `1` | `fake` | the adapter contract (the §11.2 dispatch shape) | the dev profile | `conformance-suite` | `crates/reasonbraid-adapter/tests/adapter_conformance.rs` — `fake_adapter_passes_the_conformance_suite` |
| `1` | `codex` | the adapter contract over the codex CLI (the offline shape probes) | the dev profile | `conformance-suite` | `crates/reasonbraid-adapter/tests/adapter_conformance.rs` — `codex_adapter_passes_the_conformance_suite` |
| `1` | `codex` | the real-provider live run | — | `untested` | the env-gated `RB_LIVE_CODEX=1` run — no CI measurement |
| `1` | `claude` | the adapter contract over the claude CLI (the offline shape probes) | the dev profile | `conformance-suite` | `crates/reasonbraid-adapter/tests/adapter_conformance.rs` — `claude_adapter_passes_the_conformance_suite` |
| `1` | `claude` | the real-provider live run | — | `untested` | the env-gated `RB_LIVE_CLAUDE=1` run — no CI measurement |
| `1` | the corpus | the §19.4 failure-fixture replay oracle | the dev profile | `fixture-corpus` | `crates/reasonbraid-adapter/fixtures/MANIFEST.json` (version 1 — the additive-only manifest) |
| `1` | `r0-https-fetcher` | the `https` scheme (the §12.2 acquire) | the dev profile | `live-roundtrip` | the guard's `profiles` suite (the R0 acquisition legs) |
| `1` | `r1-git-fetcher` | the `git` scheme (the R1 acquire) | the dev profile | `live-roundtrip` | the guard's `profiles` suite (the R1 acquisition legs) |
| `1` | `r2-extract-worker` | the `extract` ability (the R2 acquire) | the dev profile | `live-roundtrip` | the guard's `profiles` suite (the R2 acquisition legs) |
| `1` | `r3-browser-worker` | the `web+render` scheme | — | `untested` | the gated pack — the gate is CLOSED in the dev profile (the `.5.2` opt-in) |
| `1` | `r5-credential-broker` | the credential-bound `https` | — | `untested` | the gated pack — the gate is CLOSED in the dev profile |
| `1` | `rx-agent-mediated` | the `web+agent` scheme | — | `untested` | the gated pack — the gate is CLOSED in the dev profile |

## The named untested cells

- **The real-provider adapter runs** (`codex`/`claude` live): the env-gated
  measurements (`RB_LIVE_CODEX=1`, `RB_LIVE_CLAUDE=1`) exist but produce no CI
  measurement — the cells stay `untested` until a measured run cites its log
  (the schema's fill rule, never a prose qualification).
- **The gated resolver packs** (R3/R5/RX): the gate is closed in the dev
  profile by design (the §12.8 opt-in) — the cells stay `untested` until the
  gate opens and a measured acquisition cites its evidence.
