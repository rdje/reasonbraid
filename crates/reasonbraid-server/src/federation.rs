//! The explicit federation trust agreements (`PHASE-8.1.2`, ADR-026): the
//! NAMED tenant-to-tenant pairing — the single capability source. The
//! pairing is BOTH-SIDES: each side records its own row; the EFFECTIVE
//! agreement is the pair of `accepted` rows. A one-sided proposal widens
//! nothing; a revocation falls back to the network pseudonym.
//!
//! ⛔ **The WRITERS are not here.** `authority::federation_admin` owns the three
//! direction verbs — propose, accept and revoke — each as ONE transaction under
//! the local tenant's exclusive authority guard, holding the admission, the
//! mutation, the acceptance's cross-domain receipt and the final effect record
//! together (`SIGNOFF-REPAIR.3.3.4.12`). This module is the READ side: the
//! bilateral effective-agreement predicates the consumers ask.
//!
//! The superseded pool-based `propose`, `accept` and `revoke` that used to sit
//! here were DELETED rather than left beside their replacements
//! (`SIGNOFF-REPAIR.3.3.4.12.2`), for the reason `.3.3.4.8` gave when it removed
//! the two superseded revocation services: a second, unordered path is a path
//! someone eventually takes. They had no caller in the workspace, and being
//! `pub` on a `pub mod` is exactly why nothing flagged them — the compiler's
//! dead-code analysis cannot see a public item.

use sqlx::PgPool;

/// The digest of a direction's TERMS (`SIGNOFF-REPAIR.5.3.1`): the canonical
/// form is the pair and the two flags, newline-separated, the booleans spelled
/// `true`/`false`, SHA-256, hex, `sha256:`-prefixed. ONE recipe in two places —
/// here for every write, and in `migrations/0096_federation_terms_digest.sql`
/// for the backfill — and a control derives it by both routes. It is what an
/// acceptance pins: the receipt's `remote_ref` and the accepting row's
/// `accepted_against` are the COUNTERPARTY'S digest as read at acceptance, so
/// the trail says which terms each side saw.
pub fn terms_digest(
    tenant_id: &str,
    remote_tenant_id: &str,
    directory_visibility: bool,
    recruitment: bool,
) -> String {
    use sha2::Digest;
    let canonical =
        format!("{tenant_id}\n{remote_tenant_id}\n{directory_visibility}\n{recruitment}");
    format!(
        "sha256:{}",
        sha2::Sha256::digest(canonical.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

/// The EFFECTIVE directory-visibility agreement: BOTH directions accepted
/// AND both rows carry `directory_visibility`. The one-sided proposal or a
/// revoked direction widens nothing — and neither does an EXPIRED one
/// (`SIGNOFF-REPAIR.5.3.4`): a row past its `expires_at` is not there, in
/// every predicate below.
pub async fn has_effective_directory_agreement(
    pool: &PgPool,
    tenant_a: &str,
    tenant_b: &str,
) -> Result<bool, sqlx::Error> {
    let pair: (bool, bool) = sqlx::query_as(
        "SELECT \
             COALESCE((SELECT directory_visibility FROM federation_agreements \
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false), \
             COALESCE((SELECT directory_visibility FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false)",
    )
    .bind(tenant_a)
    .bind(tenant_b)
    .fetch_one(pool)
    .await?;
    Ok(pair == (true, true))
}

/// The EFFECTIVE recruitment agreement (the card import's allowlist
/// rung): BOTH directions accepted AND both rows carry `recruitment`.
pub async fn has_effective_recruitment_agreement(
    pool: &PgPool,
    tenant_a: &str,
    tenant_b: &str,
) -> Result<bool, sqlx::Error> {
    let pair: (bool, bool) = sqlx::query_as(
        "SELECT \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false), \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false)",
    )
    .bind(tenant_a)
    .bind(tenant_b)
    .fetch_one(pool)
    .await?;
    Ok(pair == (true, true))
}

/// The same bilateral read on a caller-owned transaction, so the rung and the
/// writes that depend on it share ONE snapshot (`SIGNOFF-REPAIR.3.3.4.11.3`).
///
/// ⭐ A snapshot was ALL it bought when this was written, because the three
/// direction verbs took no tenant authority guard at all and nothing a caller
/// held could order this read against them. That is no longer the limit:
/// `authority::federation_admin` puts each verb under its own tenant's EXCLUSIVE
/// guard (`SIGNOFF-REPAIR.3.3.4.12`), and the card import declares BOTH tenants'
/// keys in one predeclared sorted set (`.3.3.4.12.1`) — so an import is now fenced
/// by a revocation from either side. The superseded sentence is recorded here
/// rather than simply deleted, because it was carried into two other source files
/// and the book, and stayed true-sounding in all of them for two leaves after it
/// stopped being true (`SIGNOFF-REPAIR.3.3.4.12.2`).
pub(crate) async fn has_effective_recruitment_agreement_in_tx(
    tx: &mut sqlx::PgConnection,
    tenant_a: &str,
    tenant_b: &str,
) -> Result<bool, sqlx::Error> {
    let pair: (bool, bool) = sqlx::query_as(
        "SELECT \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false), \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted' \
                         AND (expires_at IS NULL OR expires_at > now())), false)",
    )
    .bind(tenant_a)
    .bind(tenant_b)
    .fetch_one(&mut *tx)
    .await?;
    Ok(pair == (true, true))
}

#[cfg(test)]
mod terms_digest_vectors {
    /// Two vectors pinned by an INDEPENDENT route (Python's `hashlib` over the
    /// documented canonical string), so the recipe cannot drift silently — and
    /// the migration's SQL backfill is held to the same two values by the
    /// upgrade suite.
    #[test]
    fn the_recipe_matches_the_pinned_vectors() {
        assert_eq!(
            super::terms_digest("ten_a", "ten_b", true, false),
            "sha256:724dd4fbcbd8f9b944c6fb62552624c5df7fdf3f4c6e8af574eae99e9334fae6"
        );
        assert_eq!(
            super::terms_digest("ten_b", "ten_a", true, true),
            "sha256:cc4ce56edf7697431d32b0636a833ab192e89a5f2414762bcfdddff0e14e74f9"
        );
    }
}
