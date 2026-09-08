//! The declared secret-store profiles (`PHASE-7.1.4.2`,
//! `docs/decisions/2026-09-08_declared-profiles-secrets-classification.md`):
//! the secret store is a CONFIGURATION choice, never an ambient dependency.
//! The registry is the ONLY seam — the server's key reads route through the
//! resolved profile, and an UNDECLARED profile name is the typed refusal
//! (never a silent fallback to the dev rows).
//!
//! The shipped profile is `dev_database`: the plaintext dev rows (the
//! `server_ca` material) — the ADR-007 dev stance, declared rather than
//! hidden. The external stores (vault, KMS, …) arrive as NEW profile names
//! in this registry — a configuration change, not a code migration.

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
}
