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
                delegate_subject: None,
                delegation_scope: None,
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

            // ⛔ `ON CONFLICT DO NOTHING RETURNING` rather than letting the
            // partial unique index raise. A constraint violation ABORTS the
            // transaction, which would take the admission and the effect record
            // down with it — so the established `409` could not be recorded as
            // an outcome at all. Returning zero rows is the same refusal without
            // the abort.
            let issued: Option<(String, String, DateTime<Utc>)> = sqlx::query_as(
                "INSERT INTO node_enrollment_tokens \
                 (token_id, tenant_id, node_id, host_claim, nonce, expires_at) \
                 VALUES ('ntk_' || gen_random_uuid()::text, $1, $2, $3, \
                         gen_random_uuid()::text, $4) \
                 ON CONFLICT DO NOTHING \
                 RETURNING token_id, nonce, expires_at",
            )
            .bind(tenant_id.to_string())
            .bind(&node_id)
            .bind(&host_claim)
            .bind(at + ttl)
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
                delegate_subject: None,
                delegation_scope: None,
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
