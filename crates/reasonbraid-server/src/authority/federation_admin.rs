//! Federation direction administration as ONE guarded transaction per verb
//! (`SIGNOFF-REPAIR.3.3.4.12`).
//!
//! Three verbs over one table, so one module — but three DISTINCT transaction
//! bodies, because their state machines differ: a proposal is an upsert, an
//! acceptance is a conditional update that must tell two idle states apart, and a
//! revocation is a conditional update whose idle state is a no-op the caller is
//! told about in its own response field.
//!
//! # Why these take the EXCLUSIVE guard
//!
//! Derived, and it is `SIGNOFF-REPAIR.3.3.4.9`'s first reason rather than its
//! second. None of these advances the tenant's revocation epoch, so none is a
//! revocation in the sense `.10.2` is. What they all are is a **classify-then-write
//! over a row that may not exist**: each decides `applied` against `no_op` or a
//! refusal from how many rows its statement touched, and a row lock cannot cover
//! an absent row. Under READ COMMITTED a concurrent proposal can insert the
//! direction between another verb's look and its write, so the exactness has to
//! come from tenant-level exclusion.
//!
//! # One guard key, not two
//!
//! Each verb writes only the LOCAL tenant's own direction row — the pairing is
//! both-sides precisely so that neither side mutates the other's — and the
//! acceptance's cross-domain receipt is a local row naming a remote reference. A
//! proposal additionally READS `tenants` to check its counterparty exists, which
//! is the minimal foreign-ID existence probe the guard contract permits: it
//! decodes no foreign policy and mutates no foreign state.
//!
//! ⚠️ What that leaves open, stated here because the next leaf depends on it: a
//! card import holds the IMPORTING tenant's guard, so it is now fenced against
//! that tenant revoking its own direction — and NOT against the origin tenant
//! revoking its side. Closing that needs the import to declare both keys in one
//! sorted set, which is `SIGNOFF-REPAIR.3.3.4.12.1`.

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeRefusal, GrantAction, GrantSubject, ResourceTarget,
    TenantId,
};

use super::effects::{bounded_detail, record_administrative_effect_in_tx};
use super::transaction::{transact, GuardError, GuardMode, TenantTransaction};
use super::{authorize_in_tx, AuthorityTransactionError, CommandAuthz};

/// What a proposal decided. Every variant COMMITTED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposeResult {
    /// The direction was created, or its terms changed.
    Proposed {
        agreement_id: String,
    },
    /// The direction already stood on exactly these terms. The response is
    /// identical to `Proposed`'s; the record is where they differ.
    Unchanged {
        agreement_id: String,
    },
    /// The named counterparty is not a tenant of this deployment.
    ///
    /// ⛔ Before `SIGNOFF-REPAIR.3.3.4.12` this was a RAISED foreign-key
    /// violation and a `500`. A proposal must name a real counterparty, so the
    /// answer distinguishing a real tenant from an absent one is intrinsic to the
    /// operation rather than an incidental leak — and tenant ids are unguessable,
    /// so it is an existence check on an id the caller already holds.
    UnknownRemote,
    Denied {
        reason: String,
    },
}

/// What an acceptance decided. Every variant COMMITTED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptResult {
    Accepted,
    /// This side already accepted. The request is already satisfied.
    AlreadyAccepted,
    /// There is no proposed direction here to accept — none was ever made, or
    /// this side revoked. ONE wire answer with [`Self::AlreadyAccepted`], as
    /// `.9` and `.10.2` established; the record tells them apart.
    NothingProposed,
    Denied {
        reason: String,
    },
}

/// What a revocation decided. Every variant COMMITTED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevokeDirectionResult {
    Revoked,
    /// Nothing to revoke: already revoked, or no direction was ever recorded.
    /// The caller is told so by the existing `revoked: 0` response field.
    NothingToRevoke,
    Denied {
        reason: String,
    },
}

/// One admitted direction request and what it did.
#[derive(Debug, Clone)]
pub struct DirectionOutcome<T> {
    /// The admission this request committed; also the effect record's id when
    /// this result recorded one.
    pub record_id: String,
    pub result: T,
}

/// The deterministic direction id. It is derived rather than stored-and-read, so
/// a caller receives the same id whether the proposal changed anything or not.
fn agreement_id(tenant: TenantId, remote: TenantId) -> String {
    format!("fed_{tenant}_{remote}")
}

/// The admission shared by all three verbs: the local tenant's `tenant_admin`,
/// evaluated on the guarded connection at `at`.
///
/// A direction is the LOCAL tenant's own record of a relationship, so the local
/// administrator is the right and only authority for it. Accepting is not a
/// permission the remote side grants; it is this side's own statement.
async fn admit(
    tx: &mut TenantTransaction<'_>,
    principal: &GrantSubject,
    tenant_id: TenantId,
    at: DateTime<Utc>,
) -> Result<Result<String, (String, String)>, GuardError> {
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegate_subject: None,
        delegation_scope: None,
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant { tenant_id },
    };
    let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
    Ok(match authorize_in_tx(&mut *conn, &authz, at).await? {
        super::AuthorizationOutcome::Allowed { record_id, .. } => Ok(record_id),
        super::AuthorizationOutcome::Denied { record_id, reason } => Err((record_id, reason)),
    })
}

/// Record one direction verb's final outcome on the caller's transaction.
async fn record(
    tx: &mut TenantTransaction<'_>,
    tenant_id: TenantId,
    record_id: &str,
    operation: AdministrativeOperation,
    outcome: AdministrativeOutcome,
    at: DateTime<Utc>,
) -> Result<(), GuardError> {
    let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
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
            // None of the three verbs takes a reason on the wire, so none is
            // recorded. Requiring one would be a wire change this leaf was not
            // asked to invent.
            submitted_reason: None,
            outcome,
            effected_at: at,
        },
    )
    .await?;
    Ok(())
}

/// Propose (or re-propose) one direction, in ONE exclusive-guard transaction.
pub(crate) async fn propose_direction_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    remote_tenant_id: TenantId,
    directory_visibility: bool,
    recruitment: bool,
) -> Result<DirectionOutcome<ProposeResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let record_id = match admit(tx, &principal, tenant_id, at).await? {
                Ok(id) => id,
                Err((record_id, reason)) => {
                    return Ok(DirectionOutcome {
                        record_id,
                        result: ProposeResult::Denied { reason },
                    })
                }
            };
            let id = agreement_id(tenant_id, remote_tenant_id);

            // ⛔ The counterparty check runs BEFORE the insert, and it is not
            // defensive habit: `federation_agreements.remote_tenant_id` references
            // `tenants`, so an absent counterparty RAISES a foreign-key violation
            // — which would abort the transaction now carrying the admission and
            // this effect record, making the refusal unrecordable
            // (`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`).
            let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
            let remote_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tenants WHERE tenant_id = $1)")
                    .bind(remote_tenant_id.to_string())
                    .fetch_one(&mut *conn)
                    .await?;
            let (result, outcome) = if !remote_exists {
                (
                    ProposeResult::UnknownRemote,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::NotFound,
                        detail: bounded_detail(
                            "no such tenant to federate with in this deployment".to_owned(),
                        ),
                    },
                )
            } else {
                // The `WHERE` on the conflict arm is what separates `applied` from
                // `no_op`: re-proposing the SAME terms touches no column, returns
                // no row, and records a no-op — the shape `.9` established for a
                // breaker re-arm. The response is identical either way.
                let changed: Option<String> = sqlx::query_scalar(
                    "INSERT INTO federation_agreements \
                     (agreement_id, tenant_id, remote_tenant_id, directory_visibility, \
                      recruitment, status) \
                     VALUES ($1, $2, $3, $4, $5, 'proposed') \
                     ON CONFLICT (tenant_id, remote_tenant_id) DO UPDATE SET \
                         directory_visibility = EXCLUDED.directory_visibility, \
                         recruitment = EXCLUDED.recruitment, \
                         status = 'proposed', accepted_at = NULL \
                     WHERE federation_agreements.directory_visibility \
                               IS DISTINCT FROM EXCLUDED.directory_visibility \
                        OR federation_agreements.recruitment \
                               IS DISTINCT FROM EXCLUDED.recruitment \
                        OR federation_agreements.status IS DISTINCT FROM 'proposed' \
                     RETURNING agreement_id",
                )
                .bind(&id)
                .bind(tenant_id.to_string())
                .bind(remote_tenant_id.to_string())
                .bind(directory_visibility)
                .bind(recruitment)
                .fetch_optional(&mut *conn)
                .await?;
                match changed {
                    Some(agreement_id) => (
                        ProposeResult::Proposed { agreement_id },
                        AdministrativeOutcome::Applied {},
                    ),
                    None => (
                        ProposeResult::Unchanged {
                            agreement_id: id.clone(),
                        },
                        AdministrativeOutcome::NoOp {
                            detail: bounded_detail(
                                "the direction already stood on exactly these terms".to_owned(),
                            ),
                        },
                    ),
                }
            };

            record(
                tx,
                tenant_id,
                &record_id,
                AdministrativeOperation::FederationDirectionPropose { remote_tenant_id },
                outcome,
                at,
            )
            .await?;
            Ok(DirectionOutcome { record_id, result })
        })
    })
    .await
}

/// Accept the remote side's proposal — this tenant's OWN row — in ONE
/// exclusive-guard transaction, with its cross-domain receipt.
pub(crate) async fn accept_direction_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    remote_tenant_id: TenantId,
) -> Result<DirectionOutcome<AcceptResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let record_id = match admit(tx, &principal, tenant_id, at).await? {
                Ok(id) => id,
                Err((record_id, reason)) => {
                    return Ok(DirectionOutcome {
                        record_id,
                        result: AcceptResult::Denied { reason },
                    })
                }
            };
            let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
            // Database time rather than `now()`: `now()` is BEGIN time, which is
            // before the guard and admission waits this acceptance queued behind.
            let accepted = sqlx::query(
                "UPDATE federation_agreements SET status = 'accepted', accepted_at = $3 \
                 WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status = 'proposed'",
            )
            .bind(tenant_id.to_string())
            .bind(remote_tenant_id.to_string())
            .bind(at)
            .execute(&mut *conn)
            .await?
            .rows_affected();

            let (result, outcome) = if accepted > 0 {
                // The cross-domain receipt (`.1.4`, ADR-026), now in the SAME
                // transaction as the status change it describes rather than
                // merely the same untenanted one.
                crate::receipts::record_in_tx(
                    &mut *conn,
                    &tenant_id.to_string(),
                    &remote_tenant_id.to_string(),
                    crate::receipts::KIND_AGREEMENT,
                    &remote_tenant_id.to_string(),
                    &agreement_id(tenant_id, remote_tenant_id),
                )
                .await?;
                (AcceptResult::Accepted, AdministrativeOutcome::Applied {})
            } else {
                // Two idle states, ONE wire answer. The extra query runs on the
                // refusal path only.
                let status: Option<String> = sqlx::query_scalar(
                    "SELECT status FROM federation_agreements \
                     WHERE tenant_id = $1 AND remote_tenant_id = $2",
                )
                .bind(tenant_id.to_string())
                .bind(remote_tenant_id.to_string())
                .fetch_optional(&mut *conn)
                .await?;
                if status.as_deref() == Some("accepted") {
                    (
                        AcceptResult::AlreadyAccepted,
                        AdministrativeOutcome::NoOp {
                            detail: bounded_detail(
                                "this side had already accepted the direction".to_owned(),
                            ),
                        },
                    )
                } else {
                    (
                        AcceptResult::NothingProposed,
                        AdministrativeOutcome::Refused {
                            code: AdministrativeRefusal::InvalidTransition,
                            detail: bounded_detail(
                                "no proposed direction here to accept".to_owned(),
                            ),
                        },
                    )
                }
            };

            record(
                tx,
                tenant_id,
                &record_id,
                AdministrativeOperation::FederationDirectionAccept { remote_tenant_id },
                outcome,
                at,
            )
            .await?;
            Ok(DirectionOutcome { record_id, result })
        })
    })
    .await
}

/// Revoke this tenant's direction, in ONE exclusive-guard transaction.
pub(crate) async fn revoke_direction_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    remote_tenant_id: TenantId,
) -> Result<DirectionOutcome<RevokeDirectionResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let record_id = match admit(tx, &principal, tenant_id, at).await? {
                Ok(id) => id,
                Err((record_id, reason)) => {
                    return Ok(DirectionOutcome {
                        record_id,
                        result: RevokeDirectionResult::Denied { reason },
                    })
                }
            };
            let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
            let revoked = sqlx::query(
                "UPDATE federation_agreements SET status = 'revoked' \
                 WHERE tenant_id = $1 AND remote_tenant_id = $2 AND status != 'revoked'",
            )
            .bind(tenant_id.to_string())
            .bind(remote_tenant_id.to_string())
            .execute(&mut *conn)
            .await?
            .rows_affected();

            let (result, outcome) = if revoked > 0 {
                (
                    RevokeDirectionResult::Revoked,
                    AdministrativeOutcome::Applied {},
                )
            } else {
                (
                    RevokeDirectionResult::NothingToRevoke,
                    AdministrativeOutcome::NoOp {
                        detail: bounded_detail(
                            "no live direction here to revoke — already revoked, or never recorded"
                                .to_owned(),
                        ),
                    },
                )
            };

            record(
                tx,
                tenant_id,
                &record_id,
                AdministrativeOperation::FederationDirectionRevoke { remote_tenant_id },
                outcome,
                at,
            )
            .await?;
            Ok(DirectionOutcome { record_id, result })
        })
    })
    .await
}
