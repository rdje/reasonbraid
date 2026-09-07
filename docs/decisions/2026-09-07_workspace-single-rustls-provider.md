# 2026-09-07_workspace-single-rustls-provider.md

## Context

`PHASE-4.2.2` (the safe HTTPS fetcher) added reqwest with
`rustls-tls-native-roots` — rustls built with the `ring` provider feature —
to `reasonbraid-server`. The leaf's offline verification (`cargo test --all`)
then FAILED: `reasonbraid-cert-spike`'s `issuance_model` test panicked inside
rustls's `CryptoProvider::get_default_or_install_from_crate_features` with
"make sure exactly one of the 'aws-lc-rs' and 'ring' features is enabled".
The spike's own `rustls = "0.23"` (default features → the aws-lc-rs provider)
was compiled into the SAME rustls unit as the server's ring request: the
cargo book documents that building multiple workspace packages
(`--workspace` / `--all` — resolver 2 included) UNIFIES dependency features
across all of those packages. The spike passed under `cargo test -p` (the
single-graph resolution) and at `b26f529` (before the ring feature existed);
the failure was measured only by the workspace-wide run.

## Decision

- **One rustls provider per workspace, named explicitly everywhere.** Every
  crate that depends on rustls declares `default-features = false` and names
  the provider feature itself. The workspace's provider is **ring** (the
  crypto family the channel proof, the CA spike's rcgen, and the R0 fetcher
  already use). `reasonbraid-cert-spike` moved from `rustls = "0.23"` to
  `rustls = { version = "0.23", default-features = false, features = ["ring",
  "std", "tls12"] }`, with a comment in its `Cargo.toml` recording WHY (the
  cross-member unification makes any default-features rustls request an
  ambiguity bomb).
- **The workspace-wide offline run is the provider-ambiguity detector.** A
  `-p`-scoped run cannot see the cross-member union; `cargo test --all`
  (recorded in every leaf's NO REGRESSION box) is the check that catches it.
- Observed alongside (routed, not fixed here): the crate-wide
  `cargo clippy --all --all-targets -- -D warnings` run fails on 9
  pre-existing findings in Phase-3 modules + `resources.rs` — the repair
  leaf `PHASE-4-MAINT-1` owns them.

## Consequences

- Any future rustls consumer (a TLS connector, a client stack, a spike) must
  add its row to the same provider family or the workspace-wide test run
  breaks — the failure mode is a runtime panic in rustls's provider
  selection, not a compile error.
- The aws-lc-rs crates stay in `Cargo.lock` but nothing compiles them; a
  provider-family change (e.g., a FIPS requirement) is a deliberate,
  whole-workspace decision — update this record + every pin in one leaf.

answers:

- **Feature unification is a workspace property, not a crate property.** The
  resolver 2's per-graph isolation does NOT apply when cargo builds multiple
  workspace packages in one invocation — the union of every member's
  requests is what compiles, so "my crate's features" is only true of
  single-package builds.
- **Mutually exclusive features cannot be reconciled by cargo.** rustls's
  aws-lc-rs and ring provider features are an XOR choice; the union compiles
  silently and panics at runtime — the ONLY defense is naming one provider
  explicitly at every dependency edge.
- **The workspace-wide test run is not optional evidence.** It is the only
  run where the cross-member union materializes; a `-p`-scoped green result
  proves the crate, not the workspace.
