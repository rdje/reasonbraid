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

// ── Atomic card import (`SIGNOFF-REPAIR.3.3.4.11.3`) ─────────────────────────
//
// # Why THIS one takes the exclusive guard
//
// Derived, and it lands the other way from the attestation above. The import
// ISSUES a grant, and `SIGNOFF-REPAIR.3.3.4.3.2` established that authority
// issuance takes the exclusive mode — `create_grant_in_guard` asserts that scope
// on the connection it is handed, so an import running under a shared guard
// could not create its grant at all. The parent leaf anticipated this in its own
// Owns line: "Authority selection and exclusive issuance guard belong to the same
// import transaction."
//
// # What the superseded shape left behind
//
// It committed the grant, the `agent_roles` row, the per-principal quota row, the
// enrollment row and the cross-domain receipt as one transaction, and then wrote
// the profile ON THE POOL. A failure there answered `500` with an imported role,
// its grant, its quota and its receipt already durable and NO profile behind
// them — an identity the directory cannot describe, created by a request that
// told the caller it had failed. It also read its admission, its active boundary
// and its effective agreement outside the transaction that used them, so a
// boundary revoked in between produced a grant checked against authority that had
// already ended.
//
// ⚠️ What moving the agreement read inside buys, stated exactly because it is
// smaller than it looks: ONE consistent snapshot with the writes that depend on
// it. It does NOT order the import against a concurrent agreement revocation —
// `federation::{propose, accept, revoke}` take no guard at all, so no guard set
// here can fence them. That ordering arrives with `SIGNOFF-REPAIR.3.3.4.12`,
// which owns the three direction verbs.

use reasonbraid_core::{AuthorityGrant, HumanPrincipalId};

use super::GrantCreateError;

/// The context the imported role's grant refusal is phrased with. It is an
/// established wire string — `cards.rs` asserts the response starts with it — so
/// it lives in one place and both the response and the effect record use it.
const GRANT_REFUSAL_CONTEXT: &str = "the imported role's grant exceeds the importing boundary";

/// What the one transaction decided. Every variant COMMITTED, so the caller must
/// not infer that a non-`Imported` result rolled the admission back — though it
/// did roll back every row the import had provisionally written.
#[derive(Debug)]
pub enum CardImportResult {
    Imported {
        role_id: String,
    },
    /// A pure rung refused: the schema version, or a digest that does not
    /// re-derive from the card's own bytes.
    CardRefused(String),
    /// The allowlist rung: no EFFECTIVE recruitment agreement with the origin.
    /// ⚠️ This is the one post-admission refusal in the fourteen administrative
    /// operations that answers `unauthorized` — it refuses an admitted tenant
    /// administrator over a fact about two tenants rather than about their grant
    /// (`SIGNOFF-REPAIR.3.3.4.7.4`).
    NoAgreement {
        origin_tenant: String,
    },
    /// The importing tenant has no active enrollment boundary to issue under.
    NoActiveBoundary,
    /// The imported role's grant exceeds the importing boundary, or that boundary
    /// is absent or outside its live window at the guarded evaluation.
    GrantRefused(String),
    /// The caller was refused by the authority evaluated inside the guard.
    Denied {
        reason: String,
    },
}

/// One admitted import attempt and what it did.
#[derive(Debug)]
pub struct CardImport {
    /// The admission this request committed; also the effect record's id when
    /// this result recorded one.
    pub record_id: String,
    pub result: CardImportResult,
}

/// Admit, verify every rung, create the whole local identity and record — in ONE
/// exclusive-guard transaction.
///
/// `card_digest` is the digest the server RE-DERIVED from the caller's card, not
/// the one the caller presented, and the difference matters on exactly one path:
/// when the digest rung refuses, the presented string is by definition not a
/// digest of this card, while the re-derived one still names the card the request
/// actually carried. It is also always a usable administrative target id, which a
/// caller-supplied string is not.
pub(crate) async fn import_card_in_one_transaction(
    pool: &sqlx::PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
    card: &crate::cards::AgentCard,
    presented_digest: &str,
    card_digest: &str,
) -> Result<CardImport, AuthorityTransactionError> {
    let principal = principal.clone();
    let card = card.clone();
    let presented_digest = presented_digest.to_owned();
    let card_digest = card_digest.to_owned();
    transact(pool, &[(tenant_id, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let at = tx.database_now().await?;
            let authz = CommandAuthz {
                actor: actor_handle_for_subject(&principal),
                principal: principal.clone(),
                delegate_subject: None,
                delegation_scope: None,
                action: GrantAction::TenantAdmin,
                target: ResourceTarget::Tenant { tenant_id },
            };
            let record_id = {
                let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
                match authorize_in_tx(&mut *conn, &authz, at).await? {
                    super::AuthorizationOutcome::Allowed { record_id, .. } => record_id,
                    super::AuthorizationOutcome::Denied { record_id, reason } => {
                        return Ok(CardImport {
                            record_id,
                            result: CardImportResult::Denied { reason },
                        });
                    }
                }
            };

            let outcome =
                import_after_admission(tx, tenant_id, &card, &presented_digest, at, &principal)
                    .await?;
            let effect = match &outcome {
                CardImportResult::Imported { .. } => AdministrativeOutcome::Applied {},
                CardImportResult::NoAgreement { origin_tenant } => AdministrativeOutcome::Refused {
                    code: AdministrativeRefusal::Unauthorized,
                    detail: bounded_detail(format!(
                        "no effective federation agreement with the origin tenant \
                             `{origin_tenant}`"
                    )),
                },
                CardImportResult::CardRefused(detail) | CardImportResult::GrantRefused(detail) => {
                    AdministrativeOutcome::Refused {
                        code: AdministrativeRefusal::InvalidCommand,
                        detail: bounded_detail(detail.clone()),
                    }
                }
                CardImportResult::NoActiveBoundary => AdministrativeOutcome::Refused {
                    code: AdministrativeRefusal::InvalidCommand,
                    detail: bounded_detail(
                        "the tenant has no active enrollment boundary".to_owned(),
                    ),
                },
                CardImportResult::Denied { .. } => unreachable!("the denial returned above"),
            };

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
                    operation: AdministrativeOperation::ProfileCardImport {
                        card_digest: AdministrativeTargetId::new(card_digest.clone()).map_err(
                            |error| {
                                GuardError::Storage(sqlx::Error::Protocol(format!(
                                    "the imported card's digest is unusable as a target: {error}"
                                )))
                            },
                        )?,
                    },
                    // The wire body carries no reason, so there is none to record.
                    submitted_reason: None,
                    outcome: effect,
                    effected_at: at,
                },
            )
            .await?;

            Ok(CardImport {
                record_id,
                result: outcome,
            })
        })
    })
    .await
}

/// The four rungs and the whole local identity, on the already-admitted
/// transaction. Every refusal returns a VALUE so the effect record can describe
/// it and commit beside it; only a storage failure returns an error, which rolls
/// the admission and every provisional row back together.
async fn import_after_admission(
    tx: &mut super::TenantTransaction<'_>,
    tenant_id: TenantId,
    card: &crate::cards::AgentCard,
    presented_digest: &str,
    at: chrono::DateTime<chrono::Utc>,
    _principal: &GrantSubject,
) -> Result<CardImportResult, GuardError> {
    // The pure rungs stay where the superseded route had them — AFTER the
    // admission. Moving them earlier would have been a wire change in the one
    // direction that matters: a caller who is not this tenant's administrator
    // would learn their card is malformed instead of being refused.
    if let Err(error) = crate::cards::verify_pure_rungs(card, presented_digest) {
        return Ok(match error {
            crate::cards::CardError::Storage(error) => return Err(GuardError::Storage(error)),
            other => CardImportResult::CardRefused(other.to_string()),
        });
    }

    let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
    let importing = tenant_id.to_string();
    if !crate::federation::has_effective_recruitment_agreement_in_tx(
        &mut *conn,
        &importing,
        &card.origin_tenant_id,
    )
    .await?
    {
        return Ok(CardImportResult::NoAgreement {
            origin_tenant: card.origin_tenant_id.clone(),
        });
    }

    let Some(boundary) = super::load_active_boundary_in_guard(tx, tenant_id).await? else {
        return Ok(CardImportResult::NoActiveBoundary);
    };

    let role = GrantSubject::Role(reasonbraid_core::AgentRoleId::new());
    let role_id = role.id_string();
    let grant: AuthorityGrant = crate::api::dev_grant(
        &boundary,
        HumanPrincipalId::new(),
        role.clone(),
        vec![
            GrantAction::ThreadContribute,
            GrantAction::ThreadInvitationRespond,
        ],
    );
    if let Err(error) = super::create_grant_in_guard(tx, &grant).await {
        // The refusal's wording comes from the SAME renderer the HTTP surface
        // uses, so the response and the effect record describing it are the same
        // string. A storage or transaction failure is not a refusal of an
        // admitted operation — it rolls the whole import back.
        return match super::grant_refusal_message(&error, GRANT_REFUSAL_CONTEXT) {
            Some(message) => Ok(CardImportResult::GrantRefused(message)),
            None => Err(match error {
                GrantCreateError::Transaction(error) => error,
                GrantCreateError::Storage(error) => GuardError::Storage(error),
                unrendered => GuardError::Storage(sqlx::Error::Protocol(format!(
                    "an unrendered grant refusal reached the import: {unrendered}"
                ))),
            }),
        };
    }

    let conn = tx.connection(tenant_id, GuardMode::Exclusive)?;
    sqlx::query("INSERT INTO agent_roles (role_id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(&role_id)
        .bind(&importing)
        .bind(&card.profile.display_label)
        .execute(&mut *conn)
        .await?;
    // The identity row implies its quota row (the fail-closed write gate, `.3.5.1`).
    crate::quota::insert_principal_default_in_tx(&mut *conn, &importing, &role_id).await?;
    sqlx::query(
        "INSERT INTO enrollments (principal_id, tenant_id, kind, name) VALUES ($1, $2, 'role', $3)",
    )
    .bind(&role_id)
    .bind(&importing)
    .bind(&card.profile.display_label)
    .execute(&mut *conn)
    .await?;
    // The cross-domain receipt (`.1.4`, ADR-026): the remote reference is the
    // card's digest as the CALLER presented it — it is what that domain's own
    // record is addressed by — and the local reference is the fresh role. The
    // receipt CROSS-REFERENCES; it never merges the chains.
    crate::receipts::record_in_tx(
        &mut *conn,
        &importing,
        &card.origin_tenant_id,
        crate::receipts::KIND_CARD_IMPORT,
        presented_digest,
        &role_id,
    )
    .await?;
    // 🔴 THE repair: the profile is written in THIS transaction. The superseded
    // route committed everything above and then wrote the profile on the pool, so
    // a failure here left an imported role with a grant, a quota and a receipt and
    // no profile at all.
    crate::profiles::write_profile_in_tx(&mut *conn, &role_id, &role_id, &card.profile, at).await?;

    Ok(CardImportResult::Imported { role_id })
}
