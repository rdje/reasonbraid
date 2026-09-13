//! The declared secret-store profiles (`PHASE-7.1.4.2`,
//! `docs/decisions/2026-09-08_declared-profiles-secrets-classification.md`):
//! the secret store is a CONFIGURATION choice, never an ambient dependency.
//! An UNDECLARED profile name is the typed refusal, never a silent fallback
//! to the dev rows.
//!
//! The shipped profile is `dev_database`: the plaintext dev rows (the
//! `server_ca` material) — the ADR-007 dev stance, declared rather than
//! hidden.
//!
//! # What this store actually routes (`SIGNOFF-REPAIR.4.2.5`)
//!
//! ⛔ **One read: the CA material.** This module used to claim it was "the
//! ONLY seam" for "the server's key reads", and the decision record went
//! further with "every secret read goes through the declared profile" and
//! "the external store arrives as a configuration change, not a code
//! migration". Measured against the code, the first is true of a smaller
//! surface than it sounds, and the other two are false. The scope is stated
//! here instead, so a reader gets the size of the seam rather than its
//! ambition:
//!
//! - The server performs exactly ONE read of key material in production —
//!   `load_ca_material` below — and it IS routed. Nothing else reads a key:
//!   `node_certificates.key_der` and `node_keys.key_secret` are written and
//!   handed to the node, never selected back (the only `SELECT` of either in
//!   the workspace is in a test fixture).
//! - ⛔ The enrollment TOKEN — a plaintext dev secret the decision record
//!   names as one — is read straight from `node_enrollment_tokens` by the
//!   enrollment path. So "every secret read" was never true.
//! - ⛔ **Creation is NOT routed.** `ca::ensure_server_ca_with_store` reads
//!   through the store and then, on a fresh database, `INSERT`s into
//!   `server_ca` directly. With any profile whose backend is not this
//!   database, first boot would write here, read back there, and the
//!   `expect` on that read-back would abort the server. Adding an external
//!   store is therefore a CODE change, not only a configuration one.
//!
//! That last point is held by [`tests::the_registry_is_read_only_and_single_profile`]
//! rather than by this comment: adding a profile name fails that test until
//! the creation path is routed too.

use sqlx::PgPool;

/// The shipped profile: the control plane's own database rows.
pub const PROFILE_DEV_DATABASE: &str = "dev_database";

/// The declared profile names. An external store joins by adding its name
/// here AND its implementation to [`SecretStore`].
pub const DECLARED_PROFILES: [&str; 1] = [PROFILE_DEV_DATABASE];

/// A profile name outside the declared set (the boot-time typed refusal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredStore {
    pub name: String,
}

impl std::fmt::Display for UndeclaredStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the secret-store profile `{}` is not declared (declared: {})",
            self.name,
            DECLARED_PROFILES.join(", ")
        )
    }
}

impl std::error::Error for UndeclaredStore {}

/// The resolved store for THIS deployment. The resolution happens ONCE, at
/// boot, from the deployment's configuration — every key read goes through
/// it from then on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStore {
    /// The control plane's database rows (the shipped dev stance).
    DevDatabase,
}

impl SecretStore {
    /// The test/dev convenience: the shipped profile, resolved.
    pub fn dev() -> SecretStore {
        SecretStore::DevDatabase
    }

    /// Resolve a declared profile name — the boot-time seam. An undeclared
    /// name is the typed refusal; the server does not start with an
    /// undeclared store (never a silent fallback).
    pub fn resolve(name: &str) -> Result<SecretStore, UndeclaredStore> {
        match name {
            PROFILE_DEV_DATABASE => Ok(SecretStore::DevDatabase),
            other => Err(UndeclaredStore {
                name: other.to_string(),
            }),
        }
    }

    /// The CA material read THROUGH the declared store: the persisted
    /// (ca_der, key_der) pair, or `None` on a fresh database. The ONLY path
    /// the CA loader uses — the store is the seam, not an option.
    pub async fn load_ca_material(
        &self,
        pool: &PgPool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, sqlx::Error> {
        match self {
            SecretStore::DevDatabase => {
                sqlx::query_as("SELECT ca_der, key_der FROM server_ca WHERE ca_id = 1")
                    .fetch_optional(pool)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The declared profile resolves; the undeclared name is the typed
    /// refusal naming itself (never a silent fallback).
    #[test]
    fn the_declared_profile_resolves_and_the_undeclared_refuses() {
        let store =
            SecretStore::resolve(PROFILE_DEV_DATABASE).expect("the shipped profile resolves");
        assert_eq!(store, SecretStore::DevDatabase);

        let refused = SecretStore::resolve("vault").expect_err("the undeclared store refuses");
        assert_eq!(refused.name, "vault");
        assert!(
            refused.to_string().contains("vault"),
            "the refusal names the profile: {refused}"
        );
        assert!(
            refused.to_string().contains(PROFILE_DEV_DATABASE),
            "the refusal names the declared set: {refused}"
        );
    }

    /// The dev convenience is the shipped profile (the tests' default is
    /// the declared default — never a hidden third path).
    #[test]
    fn the_dev_convenience_is_the_declared_default() {
        assert_eq!(
            SecretStore::dev(),
            SecretStore::resolve(PROFILE_DEV_DATABASE).unwrap()
        );
    }

    /// The declared-profiles list contains the shipped profile (the boot
    /// error message lists THIS set).
    #[test]
    fn the_declared_list_names_the_shipped_profile() {
        assert!(DECLARED_PROFILES.contains(&PROFILE_DEV_DATABASE));
    }

    /// `SIGNOFF-REPAIR.4.2.5` — the tripwire that replaces a prose warning.
    ///
    /// The store routes the CA material READ. It does not route the WRITE:
    /// `ca::ensure_server_ca_with_store` `INSERT`s into `server_ca` directly
    /// and then reads back through the store, with an `expect` on the
    /// read-back. While there is ONE profile that is an invariant, because
    /// the write and the read share a backend. A SECOND profile makes it a
    /// bug that aborts the server on first boot — so a second profile must
    /// not be addable without confronting that.
    ///
    /// ⛔ Do not relax this to make room for a new profile. Route the
    /// creation path through the store first, then change the count here in
    /// the same commit.
    #[test]
    fn the_registry_is_read_only_and_single_profile() {
        assert_eq!(
            DECLARED_PROFILES.len(),
            1,
            "a second profile requires `ca::ensure_server_ca_with_store`'s \
             INSERT to be routed through the store first — today it writes to \
             `server_ca` directly and then `expect`s the read-back, which \
             holds only while both ends are this database"
        );
    }
}
