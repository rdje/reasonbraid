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

/// Tombstone ONE named snapshot as a site act (`SIGNOFF-REPAIR.7.4.4`).
///
/// A snapshot row is shared: two tenants that cite the same locator at the same
/// digest hold the same row. Saying "this evidence must not be relied upon by
/// anyone" is therefore a statement about bytes other tenants cite, and it needs
/// the same authority the sweep needs — which is why `DELETE /v1/snapshots/{id}`
/// no longer reaches `snapshots::tombstone` and withdraws the caller's own
/// citation instead.
///
/// ⛔ `Action::EvidenceExpire` deliberately, rather than a new variant. Expiring
/// a named row and expiring every due row are the same authority over the same
/// store, and a boundary already granted for the sweep is the one an operator
/// would expect to cover this. Extending `GrantAction` is also a MIGRATION —
/// `permitted_actions` stores wire names, so no existing boundary could contain
/// a new one — and that decision is `SIGNOFF-REPAIR.9.3.4.1`'s, not this leaf's.
///
/// Idempotent through `snapshots::tombstone`: a row already tombstoned keeps its
/// first reason, and the receipt reports `tombstoned: false`. The act is still
/// authorized and still audited, so an operator can prove it ran.
pub async fn tombstone_evidence(
    pool: &PgPool,
    subject: &GrantSubject,
    snapshot_id: &str,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let snapshot_id = snapshot_id.to_owned();
    // The operator's own reason goes on the ROW, not just in the audit record.
    // The sweep hardcodes "the retention expired" because that is a factual
    // claim about a class TTL no caller may assert; a named tombstone has no
    // such fact behind it, so the row records why the operator said so.
    let deletion_reason = reason.as_str().to_owned();
    authorized(
        pool,
        subject,
        Action::EvidenceExpire,
        json!({ "store": "evidence_snapshots", "snapshot_id": snapshot_id }),
        reason.as_str(),
        move |conn, _at| {
            let snapshot_id = snapshot_id.clone();
            let deletion_reason = deletion_reason.clone();
            Box::pin(async move {
                let tombstoned =
                    crate::snapshots::tombstone_in(conn, &snapshot_id, &deletion_reason).await?;
                Ok(Ok(Effect::write(
                    json!({ "snapshot_id": snapshot_id, "tombstoned": tombstoned }),
                    u64::from(tombstoned),
                )))
            })
        },
    )
    .await
}
