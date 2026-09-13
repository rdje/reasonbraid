//! Spend-breaker arm and reset as ONE guarded transaction
//! (`SIGNOFF-REPAIR.3.3.4.9`).
//!
//! # What this replaces
//!
//! The two routes used to admit the caller through `authorize_tenant_admin` —
//! which opens its own transaction with a SHARED guard — and then run a bare
//! statement ON THE POOL: outside any transaction, and under no guard at all.
//! That is weaker than the shape `SIGNOFF-REPAIR.3.3.4.8` replaced for
//! revocation, which at least held a guard for its second transaction. The
//! admission and the mutation it permitted were not ordered against each other,
//! the mutation was not ordered against an authority change, and the final
//! outcome was recorded nowhere.
//!
//! Everything now happens inside one exclusive-guard transaction: the decision
//! time is database time sampled AFTER the guard wait, the admission is written
//! on that same connection, the breaker row is selected bound to the admitted
//! tenant and locked, the mutation commits with it, and the final effect record
//! joins them.
//!
//! # Why the guard is exclusive
//!
//! Two reasons, neither of them "because revocation is".
//!
//! Classifying `applied` against `no_op` is a read-then-write over a row that
//! may NOT EXIST, and a row lock cannot cover an absent row. Only tenant-level
//! exclusion makes that classification exact.
//!
//! And the reservation path evaluates this latch (`budget::check_spend_breaker_in_tx`)
//! inside thread-command transactions that hold the SHARED guard. Exclusive mode
//! is therefore what orders an arm against every in-flight reservation in the
//! tenant: a reservation sees the breaker as it stood before the arm or after it,
//! never mid-change. That is `budget.rs`'s own rule — the latch never lags the
//! ledger it guards — now true of administration as well as of the trip.
//!
//! # What commits, and what does not
//!
//! Every path commits its ADMISSION. The final effect record is narrower,
//! because it describes an operation: a 403 denial records the admission only,
//! since the request never became an operation and the admission record already
//! says it was denied.
//!
//! Neither verb bumps the tenant's revocation epoch, in any outcome. The epoch
//! invalidates cached authority decisions; a spend latch is not authority.

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeRefusal, BudgetDimensions, GrantAction, GrantSubject,
    ResourceTarget, TenantId,
};

use super::effects::{bounded_detail, record_administrative_effect_in_tx};
use super::transaction::{transact, GuardError, GuardMode};
use super::{authorize_in_tx, AuthorityTransactionError, CommandAuthz};

/// Which administrative verb the caller asked for. Neither carries a target:
/// a breaker's target IS the tenant, which the effect record already names
/// (`SIGNOFF-REPAIR.3.3.4.7.1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerCommand {
    /// Declare the threshold. Re-arming also clears any trip — the established
    /// behaviour, preserved.
    Arm { threshold: BudgetDimensions },
    /// Re-open a tripped latch.
    Reset,
}

impl BreakerCommand {
    fn operation(self) -> AdministrativeOperation {
        match self {
            Self::Arm { .. } => AdministrativeOperation::BreakerArm {},
            Self::Reset => AdministrativeOperation::BreakerReset {},
        }
    }
}

/// What the one transaction decided. Every variant COMMITTED; the caller maps it
/// to a status code and must not infer that a non-applied result rolled back the
/// admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakerResult {
    /// The breaker was armed: it did not exist, or its threshold changed, or a
    /// trip was cleared.
    Armed,
    /// The same threshold was already armed and untripped, so no column of the
    /// row changed. The caller still receives the ordinary success answer.
    AlreadyArmed,
    /// A tripped latch was re-opened.
    Reset,
    /// A breaker is armed and is not tripped: the request is already satisfied.
    NotTripped,
    /// No breaker is armed for this tenant at all.
    NotArmed,
    /// The caller was refused by the authority evaluated inside the guard.
    Denied { reason: String },
}

/// One admitted breaker administration and what it did.
///
/// There is deliberately no decision instant here. The effect record carries
/// `effected_at`, but neither breaker response body has ever contained a
/// timestamp — unlike a revocation's `revoked_at` — and adding one would be an
/// undocumented wire change. A field nothing can read is not carried.
#[derive(Debug, Clone)]
pub struct BreakerAdministration {
    /// The admission this request committed; also the effect record's id when
    /// this result recorded one.
    pub record_id: String,
    pub result: BreakerResult,
}

/// The breaker row as the administrative verbs see it, selected bound to the
/// admitted tenant and locked.
struct Armed {
    threshold: BudgetDimensions,
    tripped: bool,
}

/// Admit, select, mutate and record in ONE exclusive-guard transaction.
pub(crate) async fn administer_breaker_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    command: BreakerCommand,
) -> Result<BreakerAdministration, AuthorityTransactionError> {
    let principal = principal.clone();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            // Database time AFTER the guard wait: authority that ended while
            // this request queued must be evaluated as ended.
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
            let authz = CommandAuthz {
                actor: actor_handle_for_subject(&principal),
                principal: principal.clone(),
                delegation: None,
                action: GrantAction::TenantAdmin,
                target: ResourceTarget::Tenant { tenant_id },
            };
            let record_id = match authorize_in_tx(&mut *conn, &authz, at).await? {
                super::AuthorizationOutcome::Allowed { record_id, .. } => record_id,
                super::AuthorizationOutcome::Denied { record_id, reason } => {
                    // The denial record IS the evidence. No operation existed,
                    // so there is no final effect to describe.
                    return Ok(BreakerAdministration {
                        record_id,
                        result: BreakerResult::Denied { reason },
                    });
                }
            };

            // Tenant-bound selection inside the guard, locked before the change.
            let row: Option<(serde_json::Value, Option<DateTime<Utc>>)> = sqlx::query_as(
                "SELECT threshold, tripped_at FROM spend_breakers WHERE tenant_id = $1 \
                 FOR UPDATE",
            )
            .bind(tenant_id.to_string())
            .fetch_optional(&mut *conn)
            .await?;
            let armed = row
                .map(|(threshold, tripped_at)| {
                    // A stored threshold this build cannot decode is a storage
                    // fault, not a transitionable state — the rule `.8` applies
                    // to a malformed stored status.
                    serde_json::from_value::<BudgetDimensions>(threshold)
                        .map(|threshold| Armed {
                            threshold,
                            tripped: tripped_at.is_some(),
                        })
                        .map_err(|_| {
                            GuardError::Storage(sqlx::Error::Protocol(
                                "the stored breaker threshold is malformed".into(),
                            ))
                        })
                })
                .transpose()?;

            let (result, effect) = match command {
                BreakerCommand::Arm { threshold } => {
                    arm(&mut *conn, tenant_id, threshold, armed).await?
                }
                BreakerCommand::Reset => reset(&mut *conn, tenant_id, armed).await?,
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
                    operation: command.operation(),
                    // Neither wire body carries a reason, so there is none to
                    // record. Requiring one here would be a wire change, and
                    // `SIGNOFF-REPAIR.3.3.4.9` forbids inventing one.
                    submitted_reason: None,
                    outcome: effect,
                    effected_at: at,
                },
            )
            .await?;

            Ok(BreakerAdministration { record_id, result })
        })
    })
    .await
}

/// Declare the threshold. Re-arming clears any trip, which is a protected
/// change; re-arming the SAME threshold on an untripped breaker changes no
/// column of the row, which is not.
///
/// `armed_at` is deliberately absent from the update list, as it always has
/// been: it records when the breaker was first armed.
async fn arm(
    conn: &mut sqlx::PgConnection,
    tenant_id: TenantId,
    threshold: BudgetDimensions,
    armed: Option<Armed>,
) -> Result<(BreakerResult, AdministrativeOutcome), GuardError> {
    if armed
        .as_ref()
        .is_some_and(|armed| !armed.tripped && armed.threshold == threshold)
    {
        return Ok((
            BreakerResult::AlreadyArmed,
            AdministrativeOutcome::NoOp {
                detail: bounded_detail(
                    "the breaker already had this threshold and was not tripped".to_owned(),
                ),
            },
        ));
    }
    sqlx::query(
        "INSERT INTO spend_breakers (tenant_id, threshold, tripped_at, tripped_reason) \
         VALUES ($1, $2, NULL, NULL) \
         ON CONFLICT (tenant_id) DO UPDATE SET \
           threshold = EXCLUDED.threshold, tripped_at = NULL, tripped_reason = NULL",
    )
    .bind(tenant_id.to_string())
    .bind(serde_json::to_value(threshold).expect("threshold serializes"))
    .execute(&mut *conn)
    .await?;
    Ok((BreakerResult::Armed, AdministrativeOutcome::Applied {}))
}

/// Re-open a tripped latch.
///
/// The two ways this finds nothing to do answer the caller identically — one
/// `409 invalid_transition` whose message already admits it covers both — and
/// the effect record is where they stop being the same fact:
///
/// - armed but not tripped is the request ALREADY SATISFIED, so `no_op`, the
///   same shape a repeated revocation records under `SIGNOFF-REPAIR.3.3.4.8`;
/// - no breaker at all is `refused`, coded `invalid_transition` because that is
///   the code the response carries. It is NOT `not_found`: a breaker
///   operation's target is the tenant, and the tenant was found.
async fn reset(
    conn: &mut sqlx::PgConnection,
    tenant_id: TenantId,
    armed: Option<Armed>,
) -> Result<(BreakerResult, AdministrativeOutcome), GuardError> {
    let Some(armed) = armed else {
        return Ok((
            BreakerResult::NotArmed,
            AdministrativeOutcome::Refused {
                code: AdministrativeRefusal::InvalidTransition,
                detail: bounded_detail("no spend breaker is armed for this tenant".to_owned()),
            },
        ));
    };
    if !armed.tripped {
        return Ok((
            BreakerResult::NotTripped,
            AdministrativeOutcome::NoOp {
                detail: bounded_detail("the breaker is armed and was not tripped".to_owned()),
            },
        ));
    }
    sqlx::query(
        "UPDATE spend_breakers SET tripped_at = NULL, tripped_reason = NULL \
         WHERE tenant_id = $1",
    )
    .bind(tenant_id.to_string())
    .execute(&mut *conn)
    .await?;
    Ok((BreakerResult::Reset, AdministrativeOutcome::Applied {}))
}
