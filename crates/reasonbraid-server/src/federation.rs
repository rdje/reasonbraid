//! The explicit federation trust agreements (`PHASE-8.1.2`, ADR-026): the
//! NAMED tenant-to-tenant pairing — the single capability source. The
//! pairing is BOTH-SIDES: each side records its own row; the EFFECTIVE
//! agreement is the pair of `accepted` rows. A one-sided proposal widens
//! nothing; a revocation falls back to the network pseudonym.

use sqlx::PgPool;

/// Propose (or re-propose) one agreement direction. The proposal widens
/// NOTHING by itself (the pairing needs the remote side's own row).
pub async fn propose(
    pool: &PgPool,
    tenant_id: &str,
    remote_tenant_id: &str,
    directory_visibility: bool,
    recruitment: bool,
) -> Result<String, sqlx::Error> {
    let agreement_id = format!("fed_{}_{}", tenant_id, remote_tenant_id);
    sqlx::query(
        "INSERT INTO federation_agreements \
         (agreement_id, tenant_id, remote_tenant_id, directory_visibility, recruitment, status) \
         VALUES ($1, $2, $3, $4, $5, 'proposed') \
         ON CONFLICT (tenant_id, remote_tenant_id) DO UPDATE SET \
             directory_visibility = EXCLUDED.directory_visibility, \
             recruitment = EXCLUDED.recruitment, \
             status = 'proposed', accepted_at = NULL",
    )
    .bind(&agreement_id)
    .bind(tenant_id)
    .bind(remote_tenant_id)
    .bind(directory_visibility)
    .bind(recruitment)
    .execute(pool)
    .await?;
    Ok(agreement_id)
}

/// Accept the REMOTE side's proposal (this tenant's own row). The effect
/// of the agreement engages only when BOTH rows are accepted.
pub async fn accept(
    pool: &PgPool,
    tenant_id: &str,
    remote_tenant_id: &str,
) -> Result<u64, sqlx::Error> {
    let rows = sqlx::query(
        "UPDATE federation_agreements SET status = 'accepted', accepted_at = now() \
         WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'proposed'",
    )
    .bind(tenant_id)
    .bind(remote_tenant_id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows)
}

/// Revoke this tenant's direction (the fallback: the network pseudonym).
pub async fn revoke(
    pool: &PgPool,
    tenant_id: &str,
    remote_tenant_id: &str,
) -> Result<u64, sqlx::Error> {
    let rows = sqlx::query(
        "UPDATE federation_agreements SET status = 'revoked' \
         WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status != 'revoked'",
    )
    .bind(tenant_id)
    .bind(remote_tenant_id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows)
}

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
