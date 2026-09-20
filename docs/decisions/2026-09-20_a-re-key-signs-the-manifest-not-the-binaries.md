---
answers:
  - How is a compromised release signing key recovered from?
  - Is `rb-release-manifest generate` reproducible?
  - Why can't a re-key just re-run `generate` with a new key?
  - Can a release manifest be verified without the private key?
---
# A re-key signs the manifest, not the binaries

- **Type:** decision
- **Status:** accepted; `rb-release-manifest re-sign` ships with its control
- **Owner:** `SIGNOFF-REPAIR.11.24.1.4`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1`, adjudicating `PHASE-7.4.2`'s
  signing-key record
- **Related:** ADR-027 (signing and distribution), ADR-011 (digest scheme),
  `docs/runbooks/signing-key-incident.md`

## The gap

`docs/runbooks/signing-key-incident.md` prescribed the recovery as *`keygen` a
new identity, re-sign the SAME manifest content (the digests unchanged), and
re-verify*. `crates/reasonbraid-release-tool` shipped `keygen`, `generate`,
`verify` and `certify {sign,verify}`; `git grep -ci "re-key\|rekey\|re_sign\|resign" -- crates`
returned **0**. There was no command that re-signs an existing manifest.

## The measurement that makes it a defect rather than an inconvenience

The obvious workaround — re-run `generate` with the new key — does not produce
the same manifest, and the reason is in the tool's own source:
`created_at: chrono::Utc::now().to_rfc3339()`.

Measured, two `generate` runs over one unchanged binary:

```
{"release":"0.1.0","created_at":"2026-09-20T14:02:01.829345+00:00","binaries":{"rb":"sha256:353797a1…"}}
{"release":"0.1.0","created_at":"2026-09-20T14:02:03.042701+00:00","binaries":{"rb":"sha256:353797a1…"}}
```

Same digest, different bytes, different signature. 🔴 **So a re-key performed
that way publishes a NEW release document rather than the same one under a new
identity** — and ADR-027 names *the manifest's own digest* as part of the
verification unit, so anything pinning it breaks.

⛔ **And the tool's module header said the opposite.** It read *the
byte-identical regeneration is the re-derivation contract*, which invites
exactly the conclusion that a re-`generate` reproduces the manifest. What is
actually byte-identical is a parsed manifest re-serialized to its own bytes; the
re-derivation `verify` performs is of each BINARY's digest. The header is
corrected, because a false sentence there is how a runbook came to describe an
impossible recovery.

## The decision

**`rb-release-manifest re-sign --key <new> --manifest <path> --sig <out>`**: the
manifest's bytes are read verbatim and signed as they stand.

- ⛔ **Verbatim, never re-serialized.** The manifest is parsed only to refuse a
  file that is not one — a signature asserts what the bytes ARE, so signing an
  arbitrary file as a release manifest is the one thing this must not do. A
  second canonicalization is a second chance to produce different bytes.
- ⛔ **The old signature is not checked first**, deliberately: the key it would
  check is the one presumed lost or compromised, so requiring it would make the
  command unusable in the only situation it exists for. What the digests assert
  about the binaries is untouched, which is why the manifest is worth re-signing.
- ⭐ **Passing the old key is refused by name.** Ed25519 signing is
  deterministic, so re-signing with the compromised key reproduces the existing
  signature byte for byte — a recovery that appears to succeed and re-keys
  nothing. When the previous signature sits where `generate` puts it, that is
  checkable, and it is checked.
- ⛔ **`--sig` is required and never overwritten.** The old signature is the
  incident's evidence; the runbook's *keep it* becomes a rule the tool holds
  rather than a sentence someone has to remember — the same stance `keygen`
  already takes toward an existing key.

## The control, and the leg that makes it mean something

`the_manifest_re_signs_under_a_new_identity_and_the_old_one_stops_verifying`
drives the runbook end to end: `generate` is shown non-reproducible first, the
same key is refused, the recovery runs, the manifest's bytes are asserted
unchanged, the old signature survives, the new identity verifies and the old one
does not — with the original pairing still verifying as the positive control.

⭐ **Leg 5 is the one that is not decoration.** Every manifest in the fixture
round-trips through serde to its own bytes, so an implementation that signed
`canonical_bytes(&parsed)` instead of the file would pass every other leg — the
control would measure nothing about the property the verb is named for. A
**pretty-printed** manifest parses to the same struct and serializes to different
bytes, and `verify` checks the signature against the file's own bytes. Falsified:
that mutation turns leg 5 red and nothing else.

Three mutations, three legs, each caught by the leg written for it:

| Mutation | Caught by |
| --- | --- |
| sign the re-serialization instead of the file | leg 5 (`the signature does not verify`) |
| drop the same-key refusal | leg 1 |
| allow the signature path to be overwritten | leg 4 |

## What is NOT solved, and is owned rather than noted

⚠️ **There is no public-key-only verification path.** `verify` and
`certify verify` both take `--key`, the PRIVATE PKCS8 file, and derive the public
key from it. So the natural next step of a re-key — publish the new identity so
somebody else can verify — is not an operation this tool has. It sits inside
ADR-027's named distribution-channel deferral (nothing is distributed in the dev
profile), and it is owned at `SIGNOFF-REPAIR.11.24.1.4.1` so the runbook does not
have to carry it as a surprise.
