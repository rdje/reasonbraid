//! `rb-release-manifest` — the release identity's signing tool (`PHASE-7.2.3`,
//! ADR-027): the per-binary digest manifest (the ADR-011 `sha256:<hex>`
//! scheme) + the Ed25519 signature over the CANONICAL manifest bytes (the
//! ring provider — the workspace single-provider rule).
//!
//! The manifest is the SINGLE verification unit: the binaries verify
//! through it, never individually. The canonical form is the struct's field
//! order with the `binaries` map sorted, so a manifest read back and
//! re-serialized is byte-identical to itself.
//!
//! ⛔ **That is NOT a claim that `generate` is reproducible, and this header
//! used to read as though it were** (`SIGNOFF-REPAIR.11.24.1.4`). It said *the
//! byte-identical regeneration is the re-derivation contract*, which invites
//! the conclusion that re-running `generate` over unchanged binaries
//! reproduces the manifest. It does not: `created_at` is `Utc::now()`, so two
//! runs over one unchanged binary produce different bytes and different
//! signatures — measured, and the reason `re-sign` exists. What IS re-derived
//! by `verify` is each BINARY's digest against the stored one; the manifest
//! itself is a durable artefact to be kept, not one to be rebuilt.
//!
//! The release identity key is the dev placement: the releaser's own file
//! (`--key`, default `release-key.pk8`, raw PKCS8 DER — gitignored). The
//! protected identities + the reproducible builders are the ADR-027 named
//! deferrals.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rb-release-manifest",
    version,
    about = "the release manifest signer (ADR-027)"
)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Generate the release identity key (refuses to overwrite an existing one).
    Keygen {
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
    },
    /// Build + sign the release manifest for the named binaries.
    Generate {
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
        /// The directory holding the built binaries.
        #[arg(long)]
        bin_dir: PathBuf,
        /// One binary name per flag (repeatable).
        #[arg(long, action = clap::ArgAction::Append)]
        bin: Vec<String>,
        /// The manifest output path (the signature lands at `<out>.sig`).
        #[arg(long)]
        out: PathBuf,
        /// The release name the manifest records.
        #[arg(long, default_value = "0.1.0")]
        release_name: String,
    },
    /// Re-sign an EXISTING manifest's exact bytes with another identity
    /// (`SIGNOFF-REPAIR.11.24.1.4`) — the signing-key incident's recovery.
    ///
    /// ⛔ **This is not `generate` with a different key, and it cannot be.**
    /// `generate` rebuilds the manifest from `--bin-dir`, which (a) needs the
    /// original binaries still present and byte-identical — exactly what a
    /// compromise investigation cannot assume — and (b) stamps a fresh
    /// `created_at`, so it produces DIFFERENT BYTES even from identical
    /// binaries. Measured: two `generate` runs over one unchanged binary
    /// differ, and so do their signatures. A re-key performed that way
    /// publishes a NEW manifest, not the same one under a new identity, and
    /// anything pinning the manifest's own digest — which ADR-027 names as
    /// part of the verification unit — breaks.
    ///
    /// ⭐ So this verb reads the manifest's bytes VERBATIM and signs those.
    /// It parses them only to refuse a file that is not a manifest; it never
    /// re-serializes, because a second canonicalization is a second chance to
    /// produce different bytes.
    ///
    /// ⛔ It does NOT verify the old signature first, deliberately: the key
    /// whose signature that would check is the one presumed lost or
    /// compromised, so requiring it would make the command unusable in the
    /// only situation it exists for. What the digests assert about the
    /// binaries is unchanged and unaffected — that is why the manifest is
    /// still worth re-signing at all.
    #[command(name = "re-sign")]
    Resign {
        /// The NEW release identity (`keygen` it first).
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
        /// The existing manifest, signed byte-for-byte as it stands.
        #[arg(long)]
        manifest: PathBuf,
        /// Where the new signature is written. Required and never
        /// overwritten: the old signature is the incident's evidence, and
        /// the runbook's "keep it" is a rule the tool should hold rather
        /// than a sentence someone has to remember.
        #[arg(long)]
        sig: PathBuf,
    },
    /// Verify the manifest: the stored digests re-derived against the
    /// binaries AND the signature over the manifest's exact bytes.
    Verify {
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
        #[arg(long)]
        bin_dir: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        sig: PathBuf,
    },
    /// Sign/verify a certification record (the `.4.3` qualification
    /// report — the ADR-027 identity over the adapter certification).
    Certify {
        #[command(subcommand)]
        certify: CertifyCmd,
    },
}

#[derive(Debug, Subcommand)]
enum CertifyCmd {
    /// Sign a certification record: the self-digest re-derives first
    /// (a record whose digest lies is refused before any signature),
    /// then the canonical bytes are written back + signed.
    Sign {
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
        /// The qualification record JSON.
        #[arg(long)]
        record: PathBuf,
    },
    /// Verify a certification record: the signature over the record's
    /// exact bytes + the self-digest re-derivation.
    Verify {
        #[arg(long, default_value = "release-key.pk8")]
        key: PathBuf,
        #[arg(long)]
        record: PathBuf,
        #[arg(long)]
        sig: PathBuf,
    },
}

/// The manifest — the canonical field order; `binaries` is the SORTED map, so
/// a parsed manifest re-serializes to its own bytes. ⚠️ `created_at` is what
/// stops a fresh `generate` reproducing an earlier manifest; see the module
/// header.
#[derive(serde::Serialize, serde::Deserialize)]
struct Manifest {
    release: String,
    created_at: String,
    binaries: BTreeMap<String, String>,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest_of(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    use sha2::Digest;
    Ok(format!("sha256:{}", hex(&sha2::Sha256::digest(&bytes))))
}

fn load_key(path: &std::path::Path) -> Result<ring::signature::Ed25519KeyPair, String> {
    let der = std::fs::read(path).map_err(|e| format!("read the key {}: {e}", path.display()))?;
    ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(&der)
        .map_err(|e| format!("the key {} does not parse: {e}", path.display()))
}

fn canonical_bytes(manifest: &Manifest) -> Result<Vec<u8>, String> {
    serde_json::to_vec(manifest).map_err(|e| format!("the manifest serializes: {e}"))
}

fn main() -> Result<(), String> {
    match Args::parse().cmd {
        Cmd::Keygen { key } => {
            let rng = ring::rand::SystemRandom::new();
            let doc = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
                .map_err(|e| format!("the key generates: {e}"))?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&key)
                .map_err(|e| {
                    format!(
                        "the key {} cannot be created (an existing key is never overwritten): {e}",
                        key.display()
                    )
                })?;
            use std::io::Write;
            file.write_all(doc.as_ref())
                .map_err(|e| format!("write the key {}: {e}", key.display()))?;
            // The key file must not be group/world readable (the dev stance
            // is still the releaser's own key).
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600))
                    .map_err(|e| format!("lock the key permissions: {e}"))?;
            }
            eprintln!(
                "the release identity key is at {} (keep it out of git)",
                key.display()
            );
            Ok(())
        }
        Cmd::Generate {
            key,
            bin_dir,
            bin,
            out,
            release_name,
        } => {
            if bin.is_empty() {
                return Err("--bin is required at least once".to_string());
            }
            let mut binaries = BTreeMap::new();
            for name in &bin {
                let path = bin_dir.join(name);
                let digest = digest_of(&path)?;
                eprintln!("{}: {digest}", path.display());
                binaries.insert(name.clone(), digest);
            }
            let manifest = Manifest {
                release: release_name,
                created_at: chrono::Utc::now().to_rfc3339(),
                binaries,
            };
            let bytes = canonical_bytes(&manifest)?;
            let signing_key = load_key(&key)?;
            let signature = signing_key.sign(&bytes);
            let sig_path = std::path::PathBuf::from(format!("{}.sig", out.display()));
            std::fs::write(&out, &bytes)
                .map_err(|e| format!("write the manifest {}: {e}", out.display()))?;
            std::fs::write(&sig_path, hex(signature.as_ref()))
                .map_err(|e| format!("write the signature {}: {e}", sig_path.display()))?;
            eprintln!(
                "the manifest is at {} (the signature at {})",
                out.display(),
                sig_path.display()
            );
            Ok(())
        }
        Cmd::Resign { key, manifest, sig } => {
            // ⛔ VERBATIM. The bytes that are signed are the bytes on disk —
            // the whole point of the verb is that the manifest does not change.
            let bytes = std::fs::read(&manifest)
                .map_err(|e| format!("read the manifest {}: {e}", manifest.display()))?;
            // Parsed only to refuse a file that is not a manifest. A signature
            // is an assertion about what the bytes ARE, so signing an arbitrary
            // file as a release manifest is the one thing this must not do.
            let parsed: Manifest = serde_json::from_slice(&bytes)
                .map_err(|e| format!("the manifest {} parses: {e}", manifest.display()))?;
            let signing_key = load_key(&key)?;
            let signature = hex(signing_key.sign(&bytes).as_ref());
            // ⭐ THE SILENT NO-OP THIS VERB EXISTS TO AVOID. Ed25519 signing is
            // deterministic, so re-signing with the SAME key reproduces the
            // existing signature byte for byte — a recovery that appears to
            // succeed and re-keys nothing. When the previous signature is
            // where `generate` puts it, that is checkable, and it is checked.
            let previous_path = std::path::PathBuf::from(format!("{}.sig", manifest.display()));
            if let Ok(previous) = std::fs::read_to_string(&previous_path) {
                if previous.trim() == signature {
                    return Err(format!(
                        "the key {} is the key that already signed {} — no re-key happened; \
                         run `keygen` for a NEW identity first",
                        key.display(),
                        manifest.display()
                    ));
                }
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&sig)
                .map_err(|e| {
                    format!(
                        "the signature {} cannot be created (an existing signature is never \
                         overwritten — it is the incident's evidence; archive it, or pass a \
                         different --sig): {e}",
                        sig.display()
                    )
                })?;
            use std::io::Write;
            file.write_all(signature.as_bytes())
                .map_err(|e| format!("write the signature {}: {e}", sig.display()))?;
            eprintln!(
                "the manifest {} is re-signed under {} ({} binaries, the digests unchanged); \
                 the signature is at {}",
                manifest.display(),
                key.display(),
                parsed.binaries.len(),
                sig.display()
            );
            Ok(())
        }
        Cmd::Verify {
            key,
            bin_dir,
            manifest,
            sig,
        } => {
            let bytes = std::fs::read(&manifest)
                .map_err(|e| format!("read the manifest {}: {e}", manifest.display()))?;
            let parsed: Manifest =
                serde_json::from_slice(&bytes).map_err(|e| format!("the manifest parses: {e}"))?;
            // 1. The signature over the manifest's EXACT bytes (a tampered
            //    manifest fails here).
            let sig_hex = std::fs::read_to_string(&sig)
                .map_err(|e| format!("read the signature {}: {e}", sig.display()))?;
            let sig_bytes: Vec<u8> = (0..sig_hex.trim().len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&sig_hex.trim()[i..i + 2], 16)
                        .map_err(|e| format!("the signature is not hex: {e}"))
                })
                .collect::<Result<_, _>>()?;
            let key_pair = load_key(&key)?;
            use ring::signature::KeyPair;
            let public = key_pair.public_key();
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public.as_ref())
                .verify(&bytes, &sig_bytes)
                .map_err(|_| "the signature does not verify".to_string())?;
            // 2. The stored digests re-derived against the binaries (a
            //    changed binary fails here — the re-derivation, never a
            //    comparison against a second copy of the manifest).
            for (name, stored) in &parsed.binaries {
                let actual = digest_of(&bin_dir.join(name))?;
                if &actual != stored {
                    return Err(format!(
                        "the binary {name} does not match the manifest ({actual} != {stored})"
                    ));
                }
            }
            eprintln!(
                "the manifest {} verifies ({} binaries, the signature + the digests)",
                manifest.display(),
                parsed.binaries.len()
            );
            Ok(())
        }
        Cmd::Certify { certify } => match certify {
            CertifyCmd::Sign { key, record } => {
                let bytes = std::fs::read(&record)
                    .map_err(|e| format!("read the record {}: {e}", record.display()))?;
                let report: reasonbraid_adapter::CertificationReport =
                    serde_json::from_slice(&bytes)
                        .map_err(|e| format!("the record parses: {e}"))?;
                if !report.digest_verifies() {
                    return Err(format!(
                        "the record {} fails its self-digest — a lying record is refused before any signature",
                        record.display()
                    ));
                }
                if !report.sdk_version_matches() {
                    return Err(format!(
                        "the record's contract version {} drifts from the SDK token — refuse",
                        report.sdk_version
                    ));
                }
                // The canonical re-serialization (the byte-identical
                // regeneration contract) is the signed + stored form.
                let canonical = serde_json::to_vec(&report)
                    .map_err(|e| format!("the record serializes: {e}"))?;
                let signing_key = load_key(&key)?;
                let signature = signing_key.sign(&canonical);
                std::fs::write(&record, &canonical)
                    .map_err(|e| format!("write the record {}: {e}", record.display()))?;
                let sig_path = std::path::PathBuf::from(format!("{}.sig", record.display()));
                std::fs::write(&sig_path, hex(signature.as_ref()))
                    .map_err(|e| format!("write the signature {}: {e}", sig_path.display()))?;
                eprintln!(
                    "the record {} is signed (the signature at {})",
                    record.display(),
                    sig_path.display()
                );
                Ok(())
            }
            CertifyCmd::Verify { key, record, sig } => {
                let bytes = std::fs::read(&record)
                    .map_err(|e| format!("read the record {}: {e}", record.display()))?;
                let report: reasonbraid_adapter::CertificationReport =
                    serde_json::from_slice(&bytes)
                        .map_err(|e| format!("the record parses: {e}"))?;
                let sig_hex = std::fs::read_to_string(&sig)
                    .map_err(|e| format!("read the signature {}: {e}", sig.display()))?;
                let sig_bytes: Vec<u8> = (0..sig_hex.trim().len())
                    .step_by(2)
                    .map(|i| {
                        u8::from_str_radix(&sig_hex.trim()[i..i + 2], 16)
                            .map_err(|e| format!("the signature is not hex: {e}"))
                    })
                    .collect::<Result<_, _>>()?;
                let key_pair = load_key(&key)?;
                use ring::signature::KeyPair;
                let public = key_pair.public_key();
                ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public.as_ref())
                    .verify(&bytes, &sig_bytes)
                    .map_err(|_| "the signature does not verify".to_string())?;
                if !report.digest_verifies() {
                    return Err("the record fails its self-digest".to_string());
                }
                if !report.sdk_version_matches() {
                    return Err(format!(
                        "the record's contract version {} drifts from the SDK token",
                        report.sdk_version
                    ));
                }
                eprintln!(
                    "the record {} verifies (the signature + the self-digest, scenario {})",
                    record.display(),
                    report.scenario
                );
                Ok(())
            }
        },
    }
}
