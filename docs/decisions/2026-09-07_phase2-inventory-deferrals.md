# Phase 2.4's inventory groundwork: four §17.5/§17.6 controls have nothing to bind yet — deferred with triggers (`PHASE-2.4.3`)

- Date: 2026-09-07 · Leaf: `PHASE-2.4.3` · Decision record

## Context

`.4` picked up the backup/PITR/inventory/migration lane. The census found
zero backup tooling (fixed by `.4.1`), the upgrade path unexercised (fixed by
`.4.2`), and four controls from §17.5–§17.6 that have nothing to BIND in the
dev profile: there is no object store (Phase 4), no canonical Git mirror (the
repository is the dev workspace), no release signing, and no production
deployment whose keys need a recovery procedure. Building machinery for
absent systems would violate the subtraction doctrine (§19.8) — so the honest
act is to name each deferral with its trigger, not to build placeholder
infrastructure.

## Decision

- **Object-store inventory** (§17.5: "object-store versioning or immutable
  backup with manifest inventory") — deferred until the object store exists
  (Phase 4's resource store). Trigger: the first object-store-backed
  resolver/resource write.
- **Canonical Git mirror in a separately administered location** — deferred
  until the canonical policy/repository product surfaces exist (Phase 6's
  Git publication). Trigger: the first policy repository publication.
- **Signing-key recovery procedure** (or a documented non-recoverable key
  rotation design) — deferred until a signing key exists (the release-signing
  design, Phase 9's G9 territory). Trigger: the first signed release
  artifact.
- **Post-restore reconciliation** (database, objects, and Git agree after
  restoration) — deferred with the two above; the `.4.1` restore exercise
  already reconciles the DATABASE's state against its pre-mutation self (the
  only store that exists). Trigger: the first restore exercise that spans
  more than one store.
- **Backup encryption** — deferred with the key story; the dev-profile dump
  is plaintext and the `.4.1` scripts say so.

## answers:

- **A deferred control must name its trigger**, not just disappear — each
  item above names the exact product surface whose arrival re-opens it
  (the subtraction doctrine's shape).
- **The `.4.1` restore exercise is the reconciliation that exists** — it
  proves the database restores to its pre-mutation state on every guard
  run; a multi-store reconciliation is a STRENGTHENING of that proof, not a
  missing control.
- **No placeholder infrastructure** — building an inventory for an absent
  object store would be the exactly-once-of-inventory lie; the dev profile
  inventories what it has (the database, via the restore exercise) and
  defers the rest.
