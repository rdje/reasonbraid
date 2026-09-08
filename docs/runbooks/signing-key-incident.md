# Runbook: signing-key incident

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: the RELEASE identity key (`release-key.pk8`, the `.2.3` ADR-027
  machinery) is lost, leaked, or suspected compromised — the signed release
  manifests can no longer be trusted.

## Detection

- **The verify failure:** `rb-release-manifest verify` refuses (the wrong
  key, a tampered manifest, a changed binary) — the typed refusal names the
  leg.
- **The operator's suspicion:** the key file leaked, the machine it lived on
  is gone.

## Authority

- Generating the release identity: the releaser (the dev placement — the
  key is the releaser's own file, gitignored).
- Declaring the incident: the accountable owner.

## Safe first actions

1. **Stop trusting the old signatures.** The key's signatures are suspect
   from the suspected-compromise moment — do not distribute artifacts
   signed after it.
2. **Do not overwrite the old key file** — keep it as the incident's
   evidence (the keygen refuses to overwrite an existing key anyway).

## Diagnostic queries

- `rb-release-manifest verify` against the published manifest (the
  pass/fail per artifact).
- The manifest's digests (the binaries' identities — the digests stay
  trustworthy even when the signature does not).

## Containment

- The old identity signs NOTHING new; the manifest's digests still name the
  binaries (the digest truth survives the key loss — the re-sign below
  re-uses them).

## Recovery

- **The key is lost (not compromised):** `rb-release-manifest keygen` a new
  identity, re-sign the SAME manifest content (the digests unchanged), and
  re-verify — the artifacts' identities never changed.
- **The key is compromised:** the same re-key PLUS the incident record (the
  compromised window: which manifests could have been forged — in the dev
  profile the distribution channel does not exist, so the window is the
  local artifacts only, stated honestly).
- **The honest dev stance:** the key is a single local file — the protected
  release identity + the hardware-backed key are the ADR-027 named
  deferrals, not invented here.

## Evidence preservation

- The old key file (the incident's evidence).
- The re-key + the re-sign logs (the verify outputs before/after).

## Communication

- The operator reports the incident (the compromised window + the re-key)
  to the accountable owner; a public disclosure rides the SECURITY.md
  policy (nothing is public yet, so nothing is announced).

## Closure tests

- The release-manifest suite (the wrong-key + the tampered-manifest
  refusals) on every offline pass.
- The `make release` verify leg (the end-to-end sign → verify) — re-run on
  every release.
