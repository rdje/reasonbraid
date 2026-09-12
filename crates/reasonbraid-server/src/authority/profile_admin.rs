//! Profile and card administration as ONE guarded transaction per verb
//! (`SIGNOFF-REPAIR.3.3.4.11`).
//!
//! The parent's census found this family's tenant predicates already correct —
//! unlike `.10`'s, where four of five mutations addressed targets nobody had
//! shown belonged to the caller. What was missing here is the transaction and the
//! evidence. `.11.1` gave the profile writer an `_in_tx` form and an anchor that
//! serializes its writers; this module puts the two ADMITTED routes on it,
//! starting with the owner's capability attestation.
//!
//! # Why this one takes a SHARED guard
//!
//! Derived, not inherited. `.10.2` took the exclusive guard because it advances
//! the tenant's revocation epoch and is therefore a revocation in the sense the
//! guard contract means. Neither reason holds here:
//!
//! - an attestation writes no authority and advances no revocation epoch, so
//!   nothing caches a decision that it invalidates;
//! - there is no classify-then-write over a row that may not exist. The role's
//!   profile anchor either exists — in which case `FOR UPDATE` makes the
//!   read-modify-write exact at the granularity that actually conflicts — or it
//!   does not, and then there is no profile and no mutation to be exact about.
//!
//! What the guard must do is fence the whole request against an authority
//! change, and shared mode does that, because a revocation takes the guard
//! exclusively.
//!
//! ⚠️ The superseded route admitted through `authorize_guarded`, which also takes
//! the SHARED guard. So the mode does not change in this repair, and — as
//! `docs/knowledge/proving-a-race-is-closed.md` records — no lock-holding fixture
//! discriminates it in either mode. The properties that changed are atomicity and
//! the lost update; those are what the controls assert.

use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeRefusal, AdministrativeTargetId, AgentRoleId, GrantAction,
    GrantSubject, ResourceTarget, TenantId,
};

use super::effects::{bounded_detail, record_administrative_effect_in_tx};
use super::transaction::{transact, GuardError, GuardMode};
use super::{authorize_in_tx, AuthorityTransactionError, CommandAuthz};

/// What the one transaction decided. Every variant COMMITTED; the caller maps it
/// to a status code and must not infer that a non-`Attested` result rolled the
/// admission back.
#[derive(Debug, Clone)]
pub enum AttestResult {
    Attested(Box<crate::profiles::CurrentProfile>),
    /// No profile for the role, or no capability by that taxonomy id in it. ONE
    /// answer for both, exactly as the superseded route gave — the record below
    /// is where they stop being the same fact.
    NoSuchClaim,
    /// The caller was refused by the authority evaluated inside the guard.
    Denied {
        reason: String,
    },
}

/// One admitted attestation attempt and what it did.
#[derive(Debug, Clone)]
pub struct Attestation {
    /// The admission this request committed; also the effect record's id.
    pub record_id: String,
    pub result: AttestResult,
}

/// Admit, upgrade the claim and record in ONE shared-guard transaction.
///
/// `tenant_id` is the ROLE's tenant, resolved by the caller so the guard set can
/// be declared before the transaction opens; it is re-read inside, under the
/// guard, and a role that no longer resolves to it is the same `NoSuchClaim`
/// answer as a missing profile.
pub(crate) async fn attest_capability_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    role_id: &str,
    taxonomy_id: &str,
    evidence_ref: &str,
) -> Result<Attestation, AuthorityTransactionError> {
    let principal = principal.clone();
    let role_id = role_id.to_owned();
    let taxonomy_id = taxonomy_id.to_owned();
    let evidence_ref = evidence_ref.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            // Database time AFTER the guard wait: authority that ended while this
            // request queued must be evaluated as ended, and the version this
            // writes is stamped with the instant it was actually written rather
            // than one read before the wait.
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
                    // The denial record IS the evidence. No operation existed, so
                    // there is no final effect to describe.
                    return Ok(Attestation {
                        record_id,
                        result: AttestResult::Denied { reason },
                    });
                }
            };

            // The role must still belong to the tenant this request was admitted
            // against. The caller resolved it on the pool to choose the guard
            // key; that read is outside the transaction by necessity, so it is
            // re-taken here, where the answer is the one the mutation will use.
            let still_here: Option<String> =
                sqlx::query_scalar("SELECT tenant_id FROM agent_roles WHERE role_id = $1")
                    .bind(&role_id)
                    .fetch_optional(&mut *conn)
                    .await?;

            // The read-modify-write happens under the role's OWN anchor lock, so
            // two administrators attesting two different capabilities of one role
            // no longer each write a profile carrying only their own upgrade.
            // An absent anchor means no profile at all — `.11.1`'s writer always
            // creates the anchor before any version row — so this decides the
            // `NoSuchClaim` answer without creating anything.
            let anchored = match still_here {
                Some(tenant) if tenant == tenant_id.to_string() => {
                    crate::profiles::lock_existing_profile_anchor_in_tx(&mut *conn, &role_id)
                        .await?
                }
                _ => None,
            };
            let upgraded = match anchored {
                Some(_) => {
                    crate::profiles::attest_capability_in_tx(
                        &mut *conn,
                        &role_id,
                        &actor_handle_for_subject(&principal).to_string(),
                        &taxonomy_id,
                        &evidence_ref,
                        at,
                    )
                    .await?
                }
                None => None,
            };

            let (result, outcome) = match upgraded {
                Some(written) => (
                    AttestResult::Attested(Box::new(written)),
                    AdministrativeOutcome::Applied {},
                ),
                None => (
                    AttestResult::NoSuchClaim,
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::NotFound,
                        detail: bounded_detail(
                            "no profile for that role, or no capability by that taxonomy id in it"
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
                    operation: AdministrativeOperation::CapabilityClaimAttest {
                        role_id: role_id.parse::<AgentRoleId>().map_err(|error| {
                            GuardError::Storage(sqlx::Error::Protocol(format!(
                                "the attestation target is not a role id: {error}"
                            )))
                        })?,
                        taxonomy_id: AdministrativeTargetId::new(taxonomy_id.clone()).map_err(
                            |error| {
                                GuardError::Storage(sqlx::Error::Protocol(format!(
                                    "the attested taxonomy id is unusable: {error}"
                                )))
                            },
                        )?,
                    },
                    // The wire body carries no reason, so there is none to record.
                    // Requiring one would be a wire change this leaf was not asked
                    // to invent; the evidence reference rides the claim itself.
                    submitted_reason: None,
                    outcome,
                    effected_at: at,
                },
            )
            .await?;

            Ok(Attestation { record_id, result })
        })
    })
    .await
}
