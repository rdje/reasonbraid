//! The evidence retention sweep as a site act (`SIGNOFF-REPAIR.7.4.3`).
//!
//! `POST /v1/snapshots/expire-due` admitted any enrolled principal and read its
//! cutoff from the request body, while the sweep itself carries no tenant
//! predicate — so one request naming a far-future instant tombstoned every
//! tenant's live `standard` and `temporary` evidence, irreversibly.
//!
//! Which snapshots are DUE is a property of `evidence_snapshots.retention_class`
//! — a column on the shared row, not on a citation (`migrations/0062`) — so it
//! is not a per-tenant fact and enforcing it cannot be a tenant verb. It takes
//! the shape `docs/decisions/2026-09-09_site-operator-authority.md` defines for
//! every other site-wide act.

use super::*;

/// Authorize, sweep and audit as one ordered site transaction.
///
/// The cutoff is the database's own `clock_timestamp()`, read AFTER the guard
/// lock by [`lock`] — the same instant the audit record is stamped with. The
/// caller supplies a reason, never a time: a deletion reason of "the retention
/// expired" is a factual claim, and a caller that chooses the clock can make it
/// false about every tenant's evidence at once.
///
/// A denial commits its own audit record, and an audit failure rolls back an
/// otherwise applied sweep.
pub async fn expire_evidence(
    pool: &PgPool,
    subject: &GrantSubject,
    reason: &Reason,
) -> Result<Receipt, Error> {
    authorized(
        pool,
        subject,
        Action::EvidenceExpire,
        json!({ "store": "evidence_snapshots" }),
        reason.as_str(),
        |conn, at| {
            Box::pin(async move {
                let tombstoned = crate::snapshots::expire_due(conn, at).await?;
                // A sweep that tombstones nothing is still an authorized act and
                // still records one, so an operator can prove it ran and found
                // nothing due.
                Ok(Ok(Effect::write(
                    json!({ "tombstoned": tombstoned, "swept_at": at }),
                    tombstoned,
                )))
            })
        },
    )
    .await
}
