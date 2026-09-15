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

/// The EFFECTIVE directory-visibility agreement: BOTH directions accepted
/// AND both rows carry `directory_visibility`. The one-sided proposal or a
/// revoked direction widens nothing.
pub async fn has_effective_directory_agreement(
    pool: &PgPool,
    tenant_a: &str,
    tenant_b: &str,
) -> Result<bool, sqlx::Error> {
    let pair: (bool, bool) = sqlx::query_as(
        "SELECT \
             COALESCE((SELECT directory_visibility FROM federation_agreements \
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted'), false), \
             COALESCE((SELECT directory_visibility FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted'), false)",
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
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted'), false), \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted'), false)",
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
                       WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'accepted'), false), \
             COALESCE((SELECT recruitment FROM federation_agreements \
                       WHERE tenant_id = $2 AND remote_tenant_id = $1 AND status = 'accepted'), false)",
    )
    .bind(tenant_a)
    .bind(tenant_b)
    .fetch_one(&mut *tx)
    .await?;
    Ok(pair == (true, true))
}
