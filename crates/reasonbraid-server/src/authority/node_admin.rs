//! Node administration as ONE guarded transaction per verb
//! (`SIGNOFF-REPAIR.3.3.4.10`).
//!
//! The parent's census measured five routes in three different shapes. This
//! module takes them one child at a time: `.10.1` enrollment-token issuance,
//! then `.10.2` certificate revocation and `.10.3` the inbox verbs.
//!
//! # Why this one takes a SHARED guard
//!
//! `SIGNOFF-REPAIR.3.3.4.9` took the exclusive guard and said why; the reasons it
//! gave do not hold here, so this takes the shared one. Deriving the mode rather
//! than inheriting it is the point:
//!
//! - there is no classify-then-write over a possibly-absent row. A single
//!   `INSERT … ON CONFLICT DO NOTHING RETURNING` decides `applied` against
//!   `refused` atomically, so nothing needs tenant-level exclusion to be exact;
//! - concurrent issuance for the SAME node is already serialized by migration
//!   0018's partial unique index, at the granularity that actually conflicts;
//! - issuance touches no state the reservation path reads, so there is no
//!   ordering to buy with exclusion.
//!
//! What the guard must do is order the whole request against an authority
//! change, and shared mode does that: a revocation takes the guard EXCLUSIVELY,
//! so it fences this transaction entirely rather than landing between its
//! admission and its write. Exclusive mode would add no invariant and would
//! block every concurrent thread command in the tenant.
//!
//! # There is no target row to verify here, and that is measured
//!
//! `node_enrollment_tokens.node_id` has no foreign key to `nodes`, deliberately:
//! a token is issued BEFORE the node it names exists, which is what the token is
//! for. So the tenant-bound target check the parent anticipated has nothing to
//! bind to at issuance — the row is stamped with the ADMITTED tenant, and the
//! question of whether a node may later enroll into that tenant is lineage at
//! REDEMPTION, owned by `SIGNOFF-REPAIR.4.1`. Inventing a refusal here would be
//! a wire change on another leaf's finding.

use chrono::{DateTime, Duration, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeRefusal, AdministrativeTargetId, GrantAction,
    GrantSubject, ResourceTarget, TenantId,
};

use super::effects::{bounded_detail, record_administrative_effect_in_tx};
use super::transaction::{transact, GuardError, GuardMode};
use super::{authorize_in_tx, AuthorityTransactionError, CommandAuthz};

/// What the one transaction decided. Every variant COMMITTED; the caller maps it
/// to a status code and must not infer that a non-`Issued` result rolled the
/// admission back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenIssueResult {
    Issued {
        token_id: String,
        nonce: String,
        expires_at: DateTime<Utc>,
    },
    /// An unused token for this node is already outstanding. The established
    /// `409 invalid_command` contract, preserved exactly.
    AlreadyOutstanding,
    /// The caller was refused by the authority evaluated inside the guard.
    Denied { reason: String },
}

/// One admitted issuance attempt and what it did.
#[derive(Debug, Clone)]
pub struct TokenIssue {
    /// The admission this request committed; also the effect record's id when
    /// this result recorded one.
    pub record_id: String,
    pub result: TokenIssueResult,
}

/// Admit, insert and record in ONE shared-guard transaction.
///
/// `node_id` and `host_claim` are the caller's own values, bound as parameters.
/// The token id and nonce stay server-generated and opaque.
pub(crate) async fn issue_enrollment_token_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    node_id: &str,
    host_claim: &str,
    ttl: Duration,
) -> Result<TokenIssue, AuthorityTransactionError> {
    let principal = principal.clone();
    let node_id = node_id.to_owned();
    let host_claim = host_claim.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            // Database time AFTER the guard wait: authority that ended while
            // this request queued must be evaluated as ended, and the token's
            // lifetime runs from the instant the decision was actually made
            // rather than from a process clock read before it.
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Shared)?;
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
                    return Ok(TokenIssue {
                        record_id,
                        result: TokenIssueResult::Denied { reason },
                    });
                }
            };

            // A token that LAPSED unused is stamped superseded first, in THIS
            // transaction (`SIGNOFF-REPAIR.4.1.1`). 0018's partial index keyed
            // `used_at IS NULL`, and an expired token still satisfies that, so a
            // lapsed token occupied the index for ever and closed the node's only
            // enrollment path — while the `409` told the operator to "expire it",
            // the one recovery that could not work.
            //
            // ⛔ The liveness is evaluated HERE rather than in the index predicate
            // because a partial index predicate must be IMMUTABLE and `now()` is
            // not (migration 0059 says the same). `at` is the same database time
            // the admission was evaluated at, so a token that lapsed while this
            // request queued for the guard is treated as lapsed.
            //
            // ⚠️ Scoped to the ADMITTED tenant. A lapsed token owned by another
            // tenant still blocks, deliberately: superseding it would be a write
            // into another tenant's rows while holding only this tenant's guard,
            // which is a worse thing than the narrow block it would relieve.
            //
            // The stamp is `superseded_at` and never `used_at`: this token was
            // never redeemed, and recording it as used would make the token store
            // lie to anyone reading it. No separate effect record is written — the
            // issuance's own effect already names this node, and the column is the
            // durable evidence that the replacement happened.
            sqlx::query(
                "UPDATE node_enrollment_tokens SET superseded_at = $3 \
                 WHERE node_id = $1 AND tenant_id = $2 \
                   AND used_at IS NULL AND superseded_at IS NULL AND expires_at <= $3",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .bind(at)
            .execute(&mut *conn)
            .await?;

            // ⛔ `ON CONFLICT DO NOTHING RETURNING` rather than letting the
            // partial unique index raise. A constraint violation ABORTS the
            // transaction, which would take the admission and the effect record
            // down with it — so the established `409` could not be recorded as
            // an outcome at all. Returning zero rows is the same refusal without
            // the abort.
            //
            // `issued_under` is the admission that allowed THIS issuance
            // (`SIGNOFF-REPAIR.4.1.2`). The record was written by `authorize_in_tx`
            // in this same transaction and already carries the `grant_id` and
            // `boundary_id` it selected, so revoking either can later void exactly
            // the tokens that authority issued — never a whole tenant's.
            let issued: Option<(String, String, DateTime<Utc>)> = sqlx::query_as(
                "INSERT INTO node_enrollment_tokens \
                 (token_id, tenant_id, node_id, host_claim, nonce, expires_at, issued_under) \
                 VALUES ('ntk_' || gen_random_uuid()::text, $1, $2, $3, \
                         gen_random_uuid()::text, $4, $5) \
                 ON CONFLICT DO NOTHING \
                 RETURNING token_id, nonce, expires_at",
            )
            .bind(tenant_id.to_string())
            .bind(&node_id)
            .bind(&host_claim)
            .bind(at + ttl)
            .bind(&record_id)
            .fetch_optional(&mut *conn)
            .await?;

            let (result, effect) = match issued {
                Some((token_id, nonce, expires_at)) => (
                    TokenIssueResult::Issued {
                        token_id,
                        nonce,
                        expires_at,
                    },
                    AdministrativeOutcome::Applied {},
                ),
                None => (
                    TokenIssueResult::AlreadyOutstanding,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::InvalidCommand,
                        detail: bounded_detail(
                            "an unused enrollment token for that node is already outstanding"
                                .to_owned(),
                        ),
                    },
                ),
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
                    operation: AdministrativeOperation::NodeEnrollTokenIssue {
                        node_id: AdministrativeTargetId::new(node_id.clone()).map_err(|error| {
                            GuardError::Storage(sqlx::Error::Protocol(format!(
                                "the issuance target id is unusable: {error}"
                            )))
                        })?,
                    },
                    // The wire body carries no reason, so there is none to
                    // record. Requiring one would be a wire change this leaf was
                    // told not to invent.
                    submitted_reason: None,
                    outcome: effect,
                    effected_at: at,
                },
            )
            .await?;

            Ok(TokenIssue { record_id, result })
        })
    })
    .await
}

// ── Node certificate revocation (`SIGNOFF-REPAIR.3.3.4.10.2`) ────────────────
//
// # Why THIS one takes the exclusive guard
//
// Derived, and it lands the other way from `.10.1` above. Node revocation bumps
// the tenant's revocation epoch, which is what invalidates every cached node-side
// admission decision — so this IS a revocation in the sense the guard contract
// means, and `SIGNOFF-REPAIR.3.3.4.8` established that a revocation takes the
// EXCLUSIVE mode precisely so that it fences every admission which has not
// already selected its evidence. A shared acquisition here would leave the book's
// stated ordering false for one of the three writers that advance the epoch.
//
// # What the superseded shape got wrong beyond the missing record
//
// The tenant-bound existence probe ran as its OWN pool query, and the mutation
// that followed matched on `node_id` alone — so the check and the act were two
// different reads of two different snapshots. Both now happen inside one
// transaction, and the UPDATE carries its own tenant predicate rather than
// trusting the probe that preceded it.

use reasonbraid_core::AdministrativeReason;

/// What the one transaction decided. Every variant COMMITTED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRevokeResult {
    /// Active certificates were revoked and the tenant's epoch advanced once.
    Revoked { certificates: i64 },
    /// The node exists and every certificate it has is already revoked — the
    /// repeated-revocation shape. No epoch change.
    AlreadyRevoked,
    /// The node exists and has never had a certificate to revoke.
    NoCertificate,
    /// No such node in the admitted tenant — missing and foreign are one answer.
    NotFound,
    /// The caller was refused by the authority evaluated inside the guard.
    Denied { reason: String },
    /// The submitted reason is outside its documented bounds.
    InvalidReason(String),
}

/// One admitted node revocation and what it did.
#[derive(Debug, Clone)]
pub struct NodeRevocation {
    pub record_id: String,
    /// Database time sampled inside the transaction, after the guard wait — the
    /// instant the decision, the mutation and the epoch bump actually share.
    pub effected_at: DateTime<Utc>,
    pub result: NodeRevokeResult,
}

/// Admit, select, revoke and record in ONE exclusive-guard transaction.
pub(crate) async fn revoke_node_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    node_id: &str,
    submitted_reason: &str,
) -> Result<NodeRevocation, AuthorityTransactionError> {
    let principal = principal.clone();
    let node_id = node_id.to_owned();
    let submitted_reason = submitted_reason.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            // Database time AFTER the guard wait: authority that ended while this
            // revocation queued must be evaluated as ended.
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
                    return Ok(NodeRevocation {
                        record_id,
                        effected_at: at,
                        result: NodeRevokeResult::Denied { reason },
                    });
                }
            };

            // The reason is checked AFTER admission, preserving the established
            // order in which a malformed request still leaves an audit record.
            let reason = match AdministrativeReason::new(submitted_reason.clone()) {
                Ok(reason) => reason,
                Err(error) => {
                    return Ok(NodeRevocation {
                        record_id,
                        effected_at: at,
                        result: NodeRevokeResult::InvalidReason(error.to_string()),
                    })
                }
            };
            let operation = AdministrativeOperation::NodeRevoke {
                node_id: AdministrativeTargetId::new(node_id.clone()).map_err(|error| {
                    GuardError::Storage(sqlx::Error::Protocol(format!(
                        "the revocation target id is unusable: {error}"
                    )))
                })?,
            };

            // Tenant-bound selection inside the guard, locked before the change.
            // This replaces a probe that ran on the pool, outside the transaction
            // whose decision depended on it.
            let node: Option<(String,)> = sqlx::query_as(
                "SELECT node_id FROM nodes WHERE node_id = $1 AND tenant_id = $2 FOR UPDATE",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .fetch_optional(&mut *conn)
            .await?;

            let (result, effect) = if node.is_none() {
                (
                    NodeRevokeResult::NotFound,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::NotFound,
                        detail: bounded_detail(
                            "no enrolled node with that id in this tenant".to_owned(),
                        ),
                    },
                )
            } else {
                // The UPDATE carries its own tenant predicate rather than
                // trusting the select above: the binding is then visible in the
                // statement that actually writes.
                let revoked = sqlx::query(
                    "UPDATE node_certificates c SET revoked_at = $3 \
                     WHERE c.node_id = $1 AND c.revoked_at IS NULL \
                       AND EXISTS (SELECT 1 FROM nodes n \
                                   WHERE n.node_id = c.node_id AND n.tenant_id = $2)",
                )
                .bind(&node_id)
                .bind(tenant_id.to_string())
                .bind(at)
                .execute(&mut *conn)
                .await?
                .rows_affected() as i64;

                if revoked > 0 {
                    super::bump_revocation_epoch(&mut *conn, &tenant_id.to_string()).await?;
                    (
                        NodeRevokeResult::Revoked {
                            certificates: revoked,
                        },
                        AdministrativeOutcome::Applied {},
                    )
                } else {
                    // The wire answers one 409 either way. The record is where a
                    // repeat stops being the same fact as a node that never had
                    // a certificate at all — one extra query, on the refusal
                    // path only.
                    let had: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM node_certificates WHERE node_id = $1",
                    )
                    .bind(&node_id)
                    .fetch_one(&mut *conn)
                    .await?;
                    if had > 0 {
                        (
                            NodeRevokeResult::AlreadyRevoked,
                            AdministrativeOutcome::NoOp {
                                detail: bounded_detail(
                                    "every certificate this node has was already revoked"
                                        .to_owned(),
                                ),
                            },
                        )
                    } else {
                        (
                            NodeRevokeResult::NoCertificate,
                            AdministrativeOutcome::Refused {
                                code: AdministrativeRefusal::InvalidTransition,
                                detail: bounded_detail(
                                    "the node has never had a certificate to revoke".to_owned(),
                                ),
                            },
                        )
                    }
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

            Ok(NodeRevocation {
                record_id,
                effected_at: at,
                result,
            })
        })
    })
    .await
}

// ── Node inbox administration (`SIGNOFF-REPAIR.3.3.4.10.3`) ──────────────────
//
// Three operator verbs over one table, and three DISTINCT transaction bodies:
// their state machines differ, and forcing them into a shared body would be the
// mistake `SIGNOFF-REPAIR.3.3.4.8` avoided only because its two revocation
// targets genuinely were the same machine.
//
// # The defect these close is cross-tenant, and it was measured rather than read
//
// `node_inbox` has carried a `tenant_id` column since migration 0003, and none of
// the three verbs used it. A probe run before this repair, with an administrator
// of tenant A acting on a node whose inbox rows belong to tenant B:
//
//   quarantine a foreign row  -> 200
//   replay a foreign row      -> 200
//   prune a foreign inbox     -> 200  {"deleted":2,"before":2,"after":0}
//
// The prune DESTROYED both of the other tenant's rows. Each verb now selects
// bound to the admitted tenant, so a foreign target is indistinguishable from an
// absent one — `SIGNOFF-REPAIR.3.1`'s rule — and mutates nothing.
//
// # Why these three take the SHARED guard
//
// None of them writes authority and none advances the revocation epoch, so none
// is a revocation in the sense `.10.2` is. What they need is to be fenced BY a
// revocation, and shared mode gives that. Exactness against a concurrent
// administrator comes from `FOR UPDATE` on the inbox row, which is the
// granularity that actually conflicts.

/// What a quarantine decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineResult {
    Quarantined {
        at: DateTime<Utc>,
    },
    /// The command is already quarantined — the request is already satisfied.
    AlreadyQuarantined {
        at: DateTime<Utc>,
    },
    /// No such command in this node's inbox IN THIS TENANT. Missing and foreign
    /// are one answer.
    NotInInbox,
    Denied {
        reason: String,
    },
    InvalidReason(String),
}

/// What a replay decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayResult {
    Replayed,
    /// The command exists and is not dead-lettered, so there is no quarantine to
    /// reverse.
    NotDeadLettered,
    NotInInbox,
    Denied {
        reason: String,
    },
}

/// What a prune decided. The counts are the operator's receipt and are measured
/// inside the same transaction as the delete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PruneResult {
    Pruned {
        /// Every row this call removed — the two classes below, summed.
        deleted: i64,
        /// Rows the node acknowledged holding, aged by `acknowledged_at`.
        deleted_delivered: i64,
        /// Rows that were NEVER delivered and reached §10.6's `expired`,
        /// aged by the admitting grant's own `expires_at`
        /// (`SIGNOFF-REPAIR.11.24.1.1.2.1.1`).
        deleted_expired: i64,
        before: i64,
        after: i64,
        cutoff: DateTime<Utc>,
    },
    /// Nothing was old enough, or the node has no rows in this tenant at all.
    /// The counts are still the caller's answer, and so is the cutoff.
    NothingToPrune {
        before: i64,
        after: i64,
        cutoff: DateTime<Utc>,
    },
    Denied {
        reason: String,
    },
    InvalidWindow(String),
}

/// One admitted inbox administration and what it did.
#[derive(Debug, Clone)]
pub struct InboxAdministration<T> {
    pub record_id: String,
    /// Database time sampled inside the transaction, after the guard wait.
    pub effected_at: DateTime<Utc>,
    pub result: T,
}

/// The admission every inbox verb shares: shared guard, database time after the
/// wait, and the admission written on the same connection.
///
/// Returning the record id rather than a closure keeps each verb's body its own,
/// which is the point — they differ below this line.
macro_rules! admit_or_return {
    ($conn:expr, $principal:expr, $tenant_id:expr, $at:expr, $denied:expr) => {{
        let authz = CommandAuthz {
            actor: actor_handle_for_subject($principal),
            principal: $principal.clone(),
            delegation: None,
            action: GrantAction::TenantAdmin,
            target: ResourceTarget::Tenant {
                tenant_id: $tenant_id,
            },
        };
        match authorize_in_tx(&mut *$conn, &authz, $at).await? {
            super::AuthorizationOutcome::Allowed { record_id, .. } => record_id,
            super::AuthorizationOutcome::Denied { record_id, reason } => {
                // The denial record IS the evidence; no operation existed.
                return Ok(InboxAdministration {
                    record_id,
                    effected_at: $at,
                    result: $denied(reason),
                });
            }
        }
    }};
}

/// Quarantine one inbox command, with its reason, in ONE shared-guard transaction.
pub(crate) async fn quarantine_command_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    node_id: &str,
    command_id: &str,
    submitted_reason: &str,
) -> Result<InboxAdministration<QuarantineResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    let node_id = node_id.to_owned();
    let command_id = command_id.to_owned();
    let submitted_reason = submitted_reason.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Shared)?;
            let record_id = admit_or_return!(conn, &principal, tenant_id, at, |reason| {
                QuarantineResult::Denied { reason }
            });

            let reason = match AdministrativeReason::new(submitted_reason.clone()) {
                Ok(reason) => reason,
                Err(error) => {
                    return Ok(InboxAdministration {
                        record_id,
                        effected_at: at,
                        result: QuarantineResult::InvalidReason(error.to_string()),
                    })
                }
            };
            let operation = inbox_operation(&node_id, &command_id, true)?;

            // Tenant-bound selection inside the guard, locked before the change.
            // The tenant predicate is the repair: without it this verb mutated
            // another tenant's row and answered 200.
            let row: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                "SELECT quarantined_at FROM node_inbox \
                 WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3 FOR UPDATE",
            )
            .bind(&node_id)
            .bind(&command_id)
            .bind(tenant_id.to_string())
            .fetch_optional(&mut *conn)
            .await?;

            let (result, effect) = match row {
                None => (
                    QuarantineResult::NotInInbox,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::InvalidCommand,
                        detail: bounded_detail(
                            "no such command in that node's inbox in this tenant".to_owned(),
                        ),
                    },
                ),
                Some((Some(already),)) => (
                    QuarantineResult::AlreadyQuarantined { at: already },
                    AdministrativeOutcome::NoOp {
                        detail: bounded_detail("the command was already quarantined".to_owned()),
                    },
                ),
                Some((None,)) => {
                    sqlx::query(
                        "UPDATE node_inbox SET quarantined_at = $4, quarantine_reason = $5 \
                         WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3",
                    )
                    .bind(&node_id)
                    .bind(&command_id)
                    .bind(tenant_id.to_string())
                    .bind(at)
                    .bind(reason.as_str())
                    .execute(&mut *conn)
                    .await?;
                    (
                        QuarantineResult::Quarantined { at },
                        AdministrativeOutcome::Applied {},
                    )
                }
            };

            record_inbox_effect(
                &mut *conn,
                &record_id,
                tenant_id,
                operation,
                Some(reason),
                effect,
                at,
            )
            .await?;
            Ok(InboxAdministration {
                record_id,
                effected_at: at,
                result,
            })
        })
    })
    .await
}

/// Replay one dead-lettered inbox command in ONE shared-guard transaction.
///
/// ⛔ Which decision facts a replay refreshes is NOT this leaf's to change —
/// `SIGNOFF-REPAIR.3.4` owns the cached-decision rebinding. The same two fields
/// are refreshed as before; only the transaction they are refreshed in, the
/// tenant binding, and the clock they read have changed.
pub(crate) async fn replay_command_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    node_id: &str,
    command_id: &str,
) -> Result<InboxAdministration<ReplayResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    let node_id = node_id.to_owned();
    let command_id = command_id.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Shared)?;
            let record_id = admit_or_return!(conn, &principal, tenant_id, at, |reason| {
                ReplayResult::Denied { reason }
            });
            let operation = inbox_operation(&node_id, &command_id, false)?;

            let row: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                "SELECT quarantined_at FROM node_inbox \
                 WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3 FOR UPDATE",
            )
            .bind(&node_id)
            .bind(&command_id)
            .bind(tenant_id.to_string())
            .fetch_optional(&mut *conn)
            .await?;

            let (result, effect) = match row {
                None => (
                    ReplayResult::NotInInbox,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::NotFound,
                        detail: bounded_detail(
                            "no such command in that node's inbox in this tenant".to_owned(),
                        ),
                    },
                ),
                // Only a DEAD-LETTERED command replays: quarantine is the
                // terminal the replay reverses, and a live command re-delivered
                // twice would double-dispatch.
                Some((None,)) => (
                    ReplayResult::NotDeadLettered,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::InvalidTransition,
                        detail: bounded_detail(
                            "the command is not dead-lettered, so there is no quarantine to reverse"
                                .to_owned(),
                        ),
                    },
                ),
                Some((Some(_),)) => {
                    // The fresh admission decision (`.1.5.2` shape): the CURRENT
                    // revocation epoch, the decision clock restarted. Read under
                    // the same guard that fences the writer which advances it.
                    let epoch: i64 = sqlx::query_scalar(
                        "SELECT revocation_epoch FROM tenants WHERE tenant_id = $1",
                    )
                    .bind(tenant_id.to_string())
                    .fetch_one(&mut *conn)
                    .await?;
                    sqlx::query(
                        "UPDATE node_inbox SET \
                           quarantined_at = NULL, \
                           quarantine_reason = NULL, \
                           decided_at = $4, \
                           revocation_epoch = $5, \
                           cursor = (SELECT COALESCE(MAX(cursor), 0) + 1 \
                                     FROM node_inbox WHERE node_id = $1) \
                         WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3",
                    )
                    .bind(&node_id)
                    .bind(&command_id)
                    .bind(tenant_id.to_string())
                    .bind(at)
                    .bind(epoch)
                    .execute(&mut *conn)
                    .await?;
                    (ReplayResult::Replayed, AdministrativeOutcome::Applied {})
                }
            };

            record_inbox_effect(
                &mut *conn, &record_id, tenant_id, operation, None, effect, at,
            )
            .await?;
            Ok(InboxAdministration {
                record_id,
                effected_at: at,
                result,
            })
        })
    })
    .await
}

/// Prune delivered inbox rows older than a window, in ONE shared-guard
/// transaction.
///
/// The §16.11 preservation rule is unchanged: a QUARANTINED row is never
/// deleted, because a dead-lettered row is acknowledged by definition and an
/// age-based sweep would otherwise destroy the quarantine evidence.
pub(crate) async fn prune_node_inbox_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    node_id: &str,
    min_age_seconds: i64,
) -> Result<InboxAdministration<PruneResult>, AuthorityTransactionError> {
    let principal = principal.clone();
    let node_id = node_id.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let conn = tx.connection(tenant_id, GuardMode::Shared)?;
            let record_id = admit_or_return!(conn, &principal, tenant_id, at, |reason| {
                PruneResult::Denied { reason }
            });

            if min_age_seconds < 0 {
                return Ok(InboxAdministration {
                    record_id,
                    effected_at: at,
                    result: PruneResult::InvalidWindow("min_age_seconds must be >= 0".to_owned()),
                });
            }
            let cutoff = at - Duration::seconds(min_age_seconds);
            let operation = AdministrativeOperation::NodeInboxPrune {
                node_id: AdministrativeTargetId::new(node_id.clone()).map_err(|error| {
                    GuardError::Storage(sqlx::Error::Protocol(format!(
                        "the prune target id is unusable: {error}"
                    )))
                })?,
            };

            // Every count is tenant-bound too: an operator's before/after receipt
            // must describe the inbox it is allowed to see, not the node's rows
            // in some other tenant.
            let before: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM node_inbox WHERE node_id = $1 AND tenant_id = $2",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .fetch_one(&mut *conn)
            .await?;
            let deleted_delivered = sqlx::query(
                "DELETE FROM node_inbox \
                 WHERE node_id = $1 AND tenant_id = $2 \
                   AND acknowledged_at IS NOT NULL AND acknowledged_at <= $3 \
                   AND quarantined_at IS NULL",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .bind(cutoff)
            .execute(&mut *conn)
            .await?
            .rows_affected() as i64;

            // The SECOND class, and the reason it is a second statement rather
            // than an `OR` (`SIGNOFF-REPAIR.11.24.1.1.2.1.1`): it is aged by a
            // DIFFERENT clock, and the operator's receipt reports it separately.
            //
            // A row that reached §10.6's `expired` was never delivered, so it
            // carries no `acknowledged_at` and the delivered statement above can
            // never reach it — which is what left it permanently unprunable once
            // `.11.24.1.1.2.1` stopped the cursor ack writing a receipt it had
            // not earned. Its window is the ADMITTING GRANT'S OWN `expires_at`:
            // the exact instant the row entered the terminal, so `min_age_seconds`
            // measures time IN that state, which is what a retention window means.
            //
            // ⛔ `revoked` is NOT included, and the omission is measured rather
            // than an oversight: `authority/revocation.rs` writes
            // `SET status = 'revoked'` and `authority_grants` has no `revoked_at`,
            // so nothing records WHEN a grant was revoked. Ageing those rows by
            // any other column would delete work an operator was told they could
            // still see. The trigger is that column — `.11.24.1.1.2.1.1.1`.
            let deleted_expired = sqlx::query(
                "DELETE FROM node_inbox \
                 WHERE node_id = $1 AND tenant_id = $2 \
                   AND acknowledged_at IS NULL \
                   AND quarantined_at IS NULL \
                   AND EXISTS (SELECT 1 FROM authorization_records r \
                                 JOIN authority_grants g ON g.grant_id = r.grant_id \
                                WHERE r.record_id = node_inbox.authz_ref \
                                  AND g.status = 'active' \
                                  AND g.expires_at <= $3)",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .bind(cutoff)
            .execute(&mut *conn)
            .await?
            .rows_affected() as i64;
            let deleted = deleted_delivered + deleted_expired;
            let after: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM node_inbox WHERE node_id = $1 AND tenant_id = $2",
            )
            .bind(&node_id)
            .bind(tenant_id.to_string())
            .fetch_one(&mut *conn)
            .await?;

            let (result, effect) = if deleted > 0 {
                (
                    PruneResult::Pruned {
                        deleted,
                        deleted_delivered,
                        deleted_expired,
                        before,
                        after,
                        cutoff,
                    },
                    AdministrativeOutcome::Applied {},
                )
            } else {
                (
                    PruneResult::NothingToPrune {
                        before,
                        after,
                        cutoff,
                    },
                    AdministrativeOutcome::NoOp {
                        detail: bounded_detail(
                            "no delivered row, and no row whose authority expired, was older \
                             than the window in this tenant"
                                .to_owned(),
                        ),
                    },
                )
            };

            record_inbox_effect(
                &mut *conn, &record_id, tenant_id, operation, None, effect, at,
            )
            .await?;
            Ok(InboxAdministration {
                record_id,
                effected_at: at,
                result,
            })
        })
    })
    .await
}

/// The per-command operation both quarantine and replay name.
fn inbox_operation(
    node_id: &str,
    command_id: &str,
    quarantine: bool,
) -> Result<AdministrativeOperation, GuardError> {
    let unusable = |error: reasonbraid_core::AdministrativeTextError| {
        GuardError::Storage(sqlx::Error::Protocol(format!(
            "the inbox target id is unusable: {error}"
        )))
    };
    let node_id = AdministrativeTargetId::new(node_id).map_err(unusable)?;
    let command_id = AdministrativeTargetId::new(command_id).map_err(unusable)?;
    Ok(if quarantine {
        AdministrativeOperation::NodeCommandQuarantine {
            node_id,
            command_id,
        }
    } else {
        AdministrativeOperation::NodeCommandReplay {
            node_id,
            command_id,
        }
    })
}

/// The one effect write the three verbs share — the record, not the decision.
#[allow(clippy::too_many_arguments)]
async fn record_inbox_effect(
    conn: &mut sqlx::PgConnection,
    record_id: &str,
    tenant_id: TenantId,
    operation: AdministrativeOperation,
    submitted_reason: Option<AdministrativeReason>,
    outcome: AdministrativeOutcome,
    at: DateTime<Utc>,
) -> Result<(), GuardError> {
    record_administrative_effect_in_tx(
        conn,
        &AdministrativeEffectRecord {
            record_id: record_id.parse().map_err(|_| {
                GuardError::Storage(sqlx::Error::Protocol(
                    "the admission this effect names is not a record id".into(),
                ))
            })?,
            tenant_id,
            operation,
            submitted_reason,
            outcome,
            effected_at: at,
        },
    )
    .await?;
    Ok(())
}
