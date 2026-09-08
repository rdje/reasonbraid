//! `rb-release-manifest` — the release identity's signing tool (`PHASE-7.2.3`,
//! ADR-027): the per-binary digest manifest (the ADR-011 `sha256:<hex>`
//! scheme) + the Ed25519 signature over the CANONICAL manifest bytes (the
//! ring provider — the workspace single-provider rule).
//!
//! The manifest is the SINGLE verification unit: the binaries verify
//! through it, never individually. The canonical form is the struct's field
//! order with the `binaries` map sorted (the byte-identical regeneration is
//! the re-derivation contract).
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
}

/// The manifest — the canonical field order; `binaries` is the SORTED map
/// (the byte-identical regeneration contract).
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
    }
}
