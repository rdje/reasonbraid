//! Grant and boundary revocation as ONE guarded transaction
//! (`SIGNOFF-REPAIR.3.3.4.8`).
//!
//! # What this replaces
//!
//! The route used to run two guarded transactions: the HTTP handler admitted the
//! caller through `authorize_tenant_admin`, which opens its own transaction with
//! a SHARED guard, and then called a revocation service that opened a second one
//! with an EXCLUSIVE guard. Between them the admission could be a fact about
//! authority that no longer held, and the submitted reason and the final outcome
//! were never recorded with the mutation at all.
//!
//! Everything now happens inside one exclusive-guard transaction: the decision
//! time is database time sampled AFTER the guard wait, the admission is written
//! on that same connection, the target is selected bound to the admitted tenant
//! and locked, the status change and the tenant's revocation epoch commit
//! together with it, and the final effect record joins them.
//!
//! # What commits, and what does not
//!
//! Every path below commits its ADMISSION — an admitted caller who asked for
//! something impossible is a fact worth keeping. The final effect record is
//! narrower, because it describes an operation:
//!
//! - a 403 denial records the admission only: the request never became an
//!   operation, and the admission record already says it was denied;
//! - a 400 malformed reason likewise, for the same reason;
//! - a 404 missing-or-foreign target DOES record `refused`/`not_found`, in the
//!   CALLER's tenant, naming the id the caller itself supplied. It leaks no
//!   existence: the two cases are indistinguishable to the caller, exactly as
//!   `SIGNOFF-REPAIR.3.1` requires, and the record contains nothing the caller
//!   did not already know;
//! - a repeated revocation records `no_op`, with the epoch unchanged;
//! - a successful one records `applied`.

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeReason, AdministrativeRefusal, AdministrativeTargetId,
    GrantAction, GrantSubject, ResourceTarget, TenantId,
};

use super::effects::{bounded_detail, record_administrative_effect_in_tx};
use super::transaction::{transact, GuardError, GuardMode};
use super::{authorize_in_tx, bump_revocation_epoch, AuthorityTransactionError, CommandAuthz};

/// Which authority family a revocation names. The two rows behave identically
/// and differ only in the table, key column and status vocabulary, so they share
/// one transaction body rather than two copies of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevocationTarget {
    Grant,
    Boundary,
}

impl RevocationTarget {
    /// ⛔ These are the ONLY source of SQL identifiers below; no caller value
    /// ever reaches an identifier position.
    const fn table(self) -> &'static str {
        match self {
            Self::Grant => "authority_grants",
            Self::Boundary => "enrollment_boundaries",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Grant => "grant_id",
            Self::Boundary => "boundary_id",
        }
    }

    const fn noun(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Boundary => "boundary",
        }
    }

    fn operation(self, id: AdministrativeTargetId) -> AdministrativeOperation {
        match self {
            Self::Grant => AdministrativeOperation::GrantRevoke { grant_id: id },
            Self::Boundary => AdministrativeOperation::BoundaryRevoke { boundary_id: id },
        }
    }

    /// The stored status this family calls "revoked". Any other stored value is
    /// a storage fault, not a transitionable state — the existing `.3.3.4.3.2`
    /// contract, preserved.
    fn parse_status(self, stored: &str) -> Result<bool, GuardError> {
        let known = match self {
            Self::Grant => stored
                .parse::<reasonbraid_core::GrantStatus>()
                .map(|status| status == reasonbraid_core::GrantStatus::Revoked),
            Self::Boundary => stored
                .parse::<reasonbraid_core::BoundaryStatus>()
                .map(|status| status == reasonbraid_core::BoundaryStatus::Revoked),
        };
        known.map_err(|_| {
            GuardError::Storage(sqlx::Error::Protocol(format!(
                "stored {} status is malformed",
                self.noun()
            )))
        })
    }
}

/// What the one transaction decided. Every variant COMMITTED; the caller maps it
/// to a status code and must not infer that a non-`Applied` result rolled back
/// the admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevocationResult {
    /// The target changed and the tenant's revocation epoch advanced once.
    Applied,
    /// The target was already revoked. No status change, no epoch change.
    AlreadyRevoked,
    /// No such target in the admitted tenant — missing and foreign are one
    /// answer by design.
    NotFound,
    /// The caller was refused by the authority evaluated inside the guard.
    Denied { reason: String },
    /// The submitted reason is outside its documented bounds.
    InvalidReason(String),
}

/// One admitted revocation attempt and what it did.
#[derive(Debug, Clone)]
pub struct Revocation {
    /// The admission this request committed; also the effect record's id when
    /// this result recorded one.
    pub record_id: String,
    /// Database time sampled inside the transaction, after the guard wait. This
    /// is the instant the decision and the mutation actually share, rather than
    /// a process clock read afterwards.
    pub effected_at: DateTime<Utc>,
    pub result: RevocationResult,
}

/// Admit, select, mutate and record in ONE exclusive-guard transaction.
///
/// `target_id` is the raw identifier the caller supplied. It is bound as a
/// parameter and never interpolated; the only identifiers that reach the SQL
/// come from [`RevocationTarget`]'s constants.
pub(crate) async fn revoke_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    target: RevocationTarget,
    target_id: &str,
    submitted_reason: &str,
) -> Result<Revocation, AuthorityTransactionError> {
    let principal = principal.clone();
    let target_id = target_id.to_owned();
    let submitted_reason = submitted_reason.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            // Database time AFTER the guard wait: authority that ended while
            // this revocation queued must be evaluated as ended.
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
            let authz = CommandAuthz {
                actor: actor_handle_for_subject(&principal),
                principal: principal.clone(),
                delegation: None,
                action: GrantAction::TenantAdmin,
                target: ResourceTarget::Tenant { tenant_id },
            };
            let outcome = authorize_in_tx(&mut *conn, &authz, at).await?;
            let record_id = match &outcome {
                super::AuthorizationOutcome::Allowed { record_id, .. } => record_id.clone(),
                super::AuthorizationOutcome::Denied { record_id, reason } => {
                    // The denial record IS the evidence. No operation existed,
                    // so there is no final effect to describe.
                    return Ok(Revocation {
                        record_id: record_id.clone(),
                        effected_at: at,
                        result: RevocationResult::Denied {
                            reason: reason.clone(),
                        },
                    });
                }
            };

            // The reason is checked after admission, preserving the established
            // order in which a malformed request still leaves an audit record.
            let reason = match AdministrativeReason::new(submitted_reason.clone()) {
                Ok(reason) => reason,
                Err(error) => {
                    return Ok(Revocation {
                        record_id,
                        effected_at: at,
                        result: RevocationResult::InvalidReason(error.to_string()),
                    })
                }
            };
            let operation = target.operation(
                AdministrativeTargetId::new(target_id.clone()).map_err(|error| {
                    GuardError::Storage(sqlx::Error::Protocol(format!(
                        "the revocation target id is unusable: {error}"
                    )))
                })?,
            );

            // Tenant-bound selection inside the guard, locked before the change.
            let row: Option<(String,)> = sqlx::query_as(&format!(
                "SELECT status FROM {} WHERE {} = $1 AND tenant_id = $2 FOR UPDATE",
                target.table(),
                target.key()
            ))
            .bind(&target_id)
            .bind(tenant_id.to_string())
            .fetch_optional(&mut *conn)
            .await?;

            let (result, effect) = match row {
                None => (
                    RevocationResult::NotFound,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::NotFound,
                        detail: bounded_detail(format!(
                            "no {} with that id in this tenant",
                            target.noun()
                        )),
                    },
                ),
                Some((stored,)) if target.parse_status(&stored)? => (
                    RevocationResult::AlreadyRevoked,
                    AdministrativeOutcome::NoOp {
                        detail: bounded_detail(format!(
                            "the {} was already revoked",
                            target.noun()
                        )),
                    },
                ),
                Some(_) => {
                    // ⭐ `revoked_at` is written by the SAME statement that
                    // changes the status (`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1`).
                    // A second statement would be a second source of truth about
                    // one act, and could leave a row revoked with no instant if
                    // anything between them failed. The value is `at` — the
                    // database time this transaction sampled after the guard wait
                    // — which is also what stamps the effect record below, so the
                    // row and the audit trail agree by CONSTRUCTION rather than
                    // by two clocks happening to match.
                    //
                    // ⛔ The column answers WHEN and never WHETHER. `status`
                    // remains the sole answer to whether authority stands, so
                    // `grant_is_live` and the `node_inbox_state` view are
                    // untouched; a NULL `revoked_at` on a revoked row means the
                    // instant is not recoverable (a revocation predating
                    // `migrations/0058`), never that the row is live.
                    sqlx::query(&format!(
                        "UPDATE {} SET status = 'revoked', revoked_at = $3 \
                         WHERE {} = $1 AND tenant_id = $2",
                        target.table(),
                        target.key()
                    ))
                    .bind(&target_id)
                    .bind(tenant_id.to_string())
                    .bind(at)
                    .execute(&mut *conn)
                    .await?;
                    bump_revocation_epoch(&mut *conn, &tenant_id.to_string()).await?;
                    // `SIGNOFF-REPAIR.4.1.2`: an enrollment token does not outlive
                    // the authority that issued it. Redemption validated only the
                    // token's own fields, so a token issued before this revocation
                    // still enrolled its node afterwards.
                    //
                    // ⛔ The check is HERE and not in redemption. `enroll` takes no
                    // tenant authority guard, so re-reading a grant's status there
                    // would read authority state outside the guard this
                    // transaction holds exclusively — the class `.3.3.4.5`
                    // repaired. Here the ordering is free: this transaction holds
                    // the exclusive guard, and the token row is already the
                    // redemption's declared serialization point, so the row lock
                    // alone decides. A redemption that got there first commits and
                    // its node is separately revocable (`.4.1.3.1` made that
                    // effective); one that arrives second reads a voided row.
                    //
                    // The selection is EXACT rather than tenant-wide: the token
                    // names the admission that issued it, and that record already
                    // carries the grant and boundary it selected. A token issued
                    // under a grant beneath a revoked BOUNDARY is caught by the
                    // same predicate, because the record stores both.
                    //
                    // No separate effect record, for 0059's reason: the
                    // revocation's own effect already names this target, and the
                    // column is the durable evidence.
                    sqlx::query(&format!(
                        "UPDATE node_enrollment_tokens t SET voided_at = $3 \
                         WHERE t.tenant_id = $2 \
                           AND t.used_at IS NULL AND t.superseded_at IS NULL \
                           AND t.voided_at IS NULL \
                           AND EXISTS (SELECT 1 FROM authorization_records r \
                                       WHERE r.record_id = t.issued_under \
                                         AND r.{} = $1)",
                        target.key()
                    ))
                    .bind(&target_id)
                    .bind(tenant_id.to_string())
                    .bind(at)
                    .execute(&mut *conn)
                    .await?;
                    (RevocationResult::Applied, AdministrativeOutcome::Applied {})
                }
            };

            record_administrative_effect_in_tx(
                &mut *conn,
                &AdministrativeEffectRecord {
                    record_id: record_id.parse().map_err(|_| {
                        GuardError::Storage(sqlx::Error::Protocol(
                            "the admission this effect names is not a record id".into(),
                        ))
                    })?,
                    tenant_id,
                    operation,
                    submitted_reason: Some(reason),
                    outcome: effect,
                    effected_at: at,
                },
            )
            .await?;

            Ok(Revocation {
                record_id,
                effected_at: at,
                result,
            })
        })
    })
    .await
}
