# The declared profiles — the secret store is a configuration choice, and each classification control refuses at its decision point (`PHASE-7.1.4.1`)

- Date: 2026-09-08 · Leaf: `PHASE-7.1.4.1` · Decision record (the §16.8 secret-manager + classification-controls contract)

## Context

ADR-034 fixed the stance: "the secrets and the regions are DECLARED
profiles" — the secret-manager integration is an interface over the declared
store profiles (the external store is a configuration choice, never an
ambient dependency), and the classification drives the region/retention/
export/evaluator decisions — "a classification without the controls is the
typed refusal, never a silent general". The `.1.4` census measured the
shipped state: the dev profile's secrets are the plaintext rows (`server_ca`,
the enrollment tokens) plus the HASHED node secret (`node_keys` — the
handshake compares the digest); the `Classification` is RECORDED-ONLY (the
Phase-1 deferral); no region/export machinery exists; the snapshot
`retention_class` is a free string. This record turns the ADR's vocabulary
into the implementation contract the `.1.4.2`/`.1.4.3` slices execute.

## Decision

- **The secret store is a DECLARED PROFILE, never an ambient dependency.**
  The registry names the profiles: the shipped `dev_database` (the
  plaintext rows + the hashed node secret — the ADR-007 dev stance,
  declared rather than hidden) and the external stores (vault, KMS, …) as
  the deployment configuration's choice. The server's key reads route
  through the registry — the store is resolved by the profile name; an
  UNDECLARED profile (or a request for a store the deployment never
  configured) is the typed `secret_store_unconfigured` refusal, never a
  silent fallback to the dev rows.
- **Each classification control refuses at ITS decision point.** The
  evaluator control binds at the DISPATCH: a confidential thread's work
  delivery requires a confidential-qualified evaluator profile — the dev
  profile registers none, so the dispatch is the typed refusal (`.1.4.3`).
  The retention control binds at the SWEEP (the snapshot retention class
  gains the confidential tier). The export control binds at any
  export-capable surface (none exists — the named deferral). The region
  control binds at the storage placement (none exists — the named
  deferral).
- **The confidential classification stays CREATABLE.** The thread exists,
  humans deliberate and close — the classification's controls refuse the
  PROVIDER use, never the thread itself. "Never a silent general" means
  the confidential thread is never treated as a general one at any
  decision point; it does not mean the classification is uncreatable.

## answers:

- **The registry is the only seam**: every secret read goes through the
  declared profile; the dev rows stay the honest dev stance, named — the
  external store arrives as a configuration change, not a code migration.
- **A control without a decision point is a named deferral, not a silent
  pass**: the region and the export controls have no machinery in the dev
  profile — each is recorded with the exact trigger that creates its
  decision point (any multi-region storage; any export-capable surface).
- **The dispatch is the dev profile's bindable control** because the
  evaluator set is known and closed (the adapter ledger) — the refusal is
  checkable today; the retention tier is the second bindable control (the
  snapshot store's sweep).
