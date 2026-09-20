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

The recovery is a **re-key plus a re-signature over the manifest that already
exists**. The digests inside it still name the binaries correctly — a key loss
says nothing about what was built — so the manifest is kept and re-signed, never
rebuilt.

```bash
# 1. A NEW identity. The old key file is left alone; keygen refuses to
#    overwrite one, so this cannot destroy the incident's evidence.
rb-release-manifest keygen --key release-key-2.pk8

# 2. Re-sign the EXISTING manifest, byte for byte. The manifest is not
#    touched; the new signature is written to a path of its own, and an
#    existing signature is never overwritten.
rb-release-manifest re-sign \
  --key release-key-2.pk8 \
  --manifest target/release/release-manifest.json \
  --sig     target/release/release-manifest.json.sig.rekeyed

# 3. Verify under the new identity — the signature AND the binary digests.
rb-release-manifest verify \
  --key release-key-2.pk8 \
  --bin-dir target/release \
  --manifest target/release/release-manifest.json \
  --sig     target/release/release-manifest.json.sig.rekeyed
```

⛔ **Do NOT re-run `generate` for this** (`SIGNOFF-REPAIR.11.24.1.4`). It
rebuilds the manifest from `--bin-dir`, which needs the original binaries still
present and byte-identical — the one thing a compromise investigation cannot
assume — and it stamps a fresh `created_at`, so it produces a **different
manifest** even from an unchanged binary. That is a new release document, not
the same one under a new identity. This runbook used to say *re-sign the SAME
manifest content* with no command that could do it; `re-sign` is that command.

⭐ **Passing the old key to `re-sign` is refused by name.** Ed25519 signing is
deterministic, so re-signing with the compromised key would reproduce the
existing signature exactly — a recovery that appears to succeed and re-keys
nothing. The tool compares against the signature beside the manifest and stops.

- **The key is lost (not compromised):** the three commands above; the
  artifacts' identities never changed.
- **The key is compromised:** the same re-key PLUS the incident record (the
  compromised window: which manifests could have been forged — in the dev
  profile the distribution channel does not exist, so the window is the
  local artifacts only, stated honestly).
- **The honest dev stance:** the key is a single local file — the protected
  release identity + the hardware-backed key are the ADR-027 named
  deferrals, not invented here.
- ⚠️ **A stated limit:** `verify` takes `--key`, the PRIVATE key file, and
  derives the public key from it. There is no public-key-only verification path,
  so "publish the new identity so others can verify" is not an operation this
  tool has. That is inside ADR-027's named distribution-channel deferral —
  nothing is distributed in the dev profile — and it is owned at
  `SIGNOFF-REPAIR.11.24.1.4.1` rather than left as a surprise here.

## Evidence preservation

- The old key file (the incident's evidence).
- The re-key + the re-sign logs (the verify outputs before/after).

## Communication

- The operator reports the incident (the compromised window + the re-key)
  to the accountable owner; a public disclosure rides the SECURITY.md
  policy (nothing is public yet, so nothing is announced).

## Closure tests

- The release-manifest suite (the wrong-key + the tampered-manifest
  refusals) on every offline pass — including
  `the_manifest_re_signs_under_a_new_identity_and_the_old_one_stops_verifying`,
  which drives this runbook's recovery end to end: the re-key, the untouched
  manifest, the new identity verifying, the old one no longer verifying, and the
  same-key refusal.
- The `make release` verify leg (the end-to-end sign → verify) — re-run on
  every release.
