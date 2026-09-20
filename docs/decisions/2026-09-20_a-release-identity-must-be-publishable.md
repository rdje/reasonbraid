---
answers:
  - Can a release manifest be verified without the private signing key?
  - How does a third party verify a ReasonBraid release?
  - What does `rb-release-manifest pubkey` do?
  - Why are `--key` and `--public-key` mutually exclusive?
---
# A release identity must be publishable, or the signature is ceremonial

- **Type:** decision
- **Status:** accepted; `pubkey` + `--public-key` ship with their control
- **Owner:** `SIGNOFF-REPAIR.11.24.1.4.1`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.4`, whose re-key recovery had no
  publishable end state
- **Related:** ADR-027 (signing and distribution),
  `docs/decisions/2026-09-20_a-re-key-signs-the-manifest-not-the-binaries.md`,
  `docs/runbooks/signing-key-incident.md`

## The gap, stated at its real width

`rb-release-manifest verify` and `certify verify` both took `--key`, read the
PKCS8 private key with `load_key`, and derived the public key from it via
`ring::signature::KeyPair::public_key()`. `keygen` exported nothing.

🔴 **So every verification required the signing key, and the only party who could
verify a release was the party who signed it.** A signature whose verifier must
hold the signing key proves nothing to anybody else — which is the property
signatures exist for. What the shipped arrangement actually caught was
accidental corruption of the manifest or a binary; it could not catch forgery,
because anyone able to run the check was equally able to re-sign.

⚠️ **This is not a claim the posture was reckless.** ADR-027 places the key with
the releaser as the explicit dev stance and names the distribution channel as a
deferral, and nothing is distributed in the dev profile — so there was no second
party being misled. The defect is that `SIGNOFF-REPAIR.11.24.1.4`'s re-key
recovery, which exists precisely for the moment an identity must change hands,
had nowhere to end.

## The decision

**`rb-release-manifest pubkey --key <private> --out <path>`** exports the raw
32-byte Ed25519 public key as hex, and **`verify --public-key <path>`** (and
`certify verify --public-key`) is the third-party verification path.

- ⛔ **The two inputs are mutually exclusive and passing both is REFUSED**, not
  silently resolved. They can name different identities, and a pass whose
  meaning depends on an argument order nobody wrote down is not a verification.
- ⚠️ **The releaser's own path is unchanged**, and the default moved to make
  that possible: `--key` no longer carries a clap `default_value`; the fallback
  to `release-key.pk8` is applied when NEITHER input is given. That keeps
  "the caller gave nothing" distinguishable from "the caller gave `--key`",
  which is what makes the both-were-given refusal expressible at all.
- ⛔ **A public key of the wrong length is a named caller error**, not a
  signature failure. Letting an 8-byte file travel into `ring` and come back as
  *the signature does not verify* would blame the signature for a mistyped path.
- ⛔ **`pubkey` never overwrites.** The key is public by definition, so unlike
  `keygen` it tightens no permissions — but a published identity replaced in
  place is exactly the failure this whole area is about, so a re-key writes a
  new file under a new name.
- ✅ **`make release` now exports the identity** beside the manifest and the
  signature, skipped when the file already exists. A release that ships a
  signature and no way to check it is the state this decision ends.

## The control, and the leg that carries it

`a_manifest_verifies_from_the_public_key_alone` builds two directories: the
signing one, and the published one holding the manifest, the signature, the
binaries and the exported public key. **It then deletes the private key** and
verifies.

⭐ **That is structural rather than inspectional.** A `--public-key` path that
silently still reached for `release-key.pk8` would be the same defect with a new
flag, and no amount of reading the code proves it does not. Falsified: a mutation
that accepts `--public-key` and then derives from the private key anyway fails at
exactly that leg, with `read the key release-key.pk8: No such file or directory`.

| Mutation | Caught by |
| --- | --- |
| `--public-key` accepted, private key used anyway | the verify-with-no-private-key leg |
| the both-inputs refusal removed | the ambiguity leg |
| the 32-byte length check removed | the wrong-shape leg |

The negative legs sit beside it — another identity's public key must not verify,
and a wrong-length file must be refused by name — with the releaser's own
`--key` path asserted still working as the positive control.

## What is still deferred

⚠️ **The CHANNEL, not the capability.** ADR-027 names the distribution channel,
the reproducible builders and the protected identities as deferrals. Publishing
the `.pub` file is a manual act with no automated audience, and this decision
does not invent one. What changed is that there is now something to publish.
