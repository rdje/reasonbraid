//! The publication records + the staging (`PHASE-6.4.2`, ADR-020): the
//! publication is the nine-step §15.7 machine's AGGREGATE — its own row
//! (never folded into the approval) carrying the decision/approval/
//! projection references, the manifest digest, the typed state
//! (staged → effective | failed), the Git object ids, and the failure
//! reason. A step that cannot complete is the typed failure, never a skip.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// The publication-state vocabulary (the §15.7 machine's typed states).
pub const PUBLICATION_STATES: [&str; 3] = ["staged", "effective", "failed"];

/// The publication submission (`.4.2`): the references the machine verifies.
///
/// ⛔ **`manifest_digest` is NOT an input** (`SIGNOFF-REPAIR.9.2.1.3.1`).
/// ADR-020 §15.7 makes the digest the SERVER's product of steps (2)–(3) —
/// *compile the canonical bundle + the publication manifest … hash …
/// the manifest* — and step (4) STORES it. It used to arrive in the request,
/// be shape-checked, and be written into the row where nothing ever read it.
///
/// ⚠️ The field survives as an OPTIONAL ASSERTION: when a caller supplies one
/// it must equal the digest staging derives, or the request is refused naming
/// both. That is the shape the publish verb's `expected_effective` already
/// uses on this same surface — a client stating what it believes, checked
/// rather than trusted.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationInput {
    pub publication_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approval_id: String,
    pub projection_id: String,
    /// The grant the caller HOLDS (`SIGNOFF-REPAIR.9.2.1.2.2`). ⛔ Staging was
    /// the one publication verb left on enrolment alone, and it is the verb
    /// that decides WHICH projection an approval publishes.
    ///
    /// ⚠️ Modelled `Option` although it is REQUIRED, and the reason is the
    /// wire contract rather than the semantics: this input is typed, so a
    /// missing required field is refused by the deserializer as a bare `422`
    /// with a plain-text body — while the three transition verbs read the same
    /// field out of an untyped body and answer `400 invalid_command`. Four
    /// verbs asking one question must not give two different refusals, so the
    /// absence is graded here and answers exactly as its siblings do.
    #[serde(default)]
    pub owning_authority: Option<String>,
    #[serde(default)]
    pub manifest_digest: Option<String>,
}

/// The publication manifest (ADR-020 §15.7 step 2), as ONE definition.
///
/// ⛔ Both the digest `stage` stores and the bytes `publish` writes are
/// computed from this function, so they cannot drift. Before
/// `SIGNOFF-REPAIR.9.2.1.3.1` the manifest was composed inline in the publish
/// handler and the stored digest came from the request — two values that were
/// never the same thing and were never compared.
pub fn manifest(
    publication_id: &str,
    proposal_id: &str,
    decision_id: &str,
    approval_id: &str,
    projection_id: &str,
    projection_digest: &str,
) -> String {
    serde_json::to_string(&serde_json::json!({
        "publication_id": publication_id,
        "proposal_id": proposal_id,
        "decision_id": decision_id,
        "approval_id": approval_id,
        "projection_id": projection_id,
        "projection_digest": projection_digest,
    }))
    .expect("the manifest serializes")
}

/// `sha256:<hex>` over the manifest bytes (ADR-020 §15.7 step 3).
pub fn manifest_digest(manifest: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(manifest.as_bytes()))
}

/// The stored publication row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredPublication {
    pub publication_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approval_id: String,
    pub projection_id: String,
    pub state: String,
    pub manifest_digest: String,
    /// The grant the publication was STAGED under (`.9.2.1.2.2`). `None` means
    /// the row predates the column — *staged before this was recorded*, never
    /// *staged by nobody* (`migrations/0081`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owning_authority: Option<String>,
    pub git_object_ids: Vec<String>,
    pub failed_reason: Option<String>,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationError {
    UnknownProposal(String),
    UnknownDecision(String),
    UnknownApproval(String),
    UnknownProjection(String),
    ForeignRecord {
        kind: String,
        id: String,
        proposal_id: String,
    },
    WrongStage {
        publication_id: String,
        state: String,
    },
    MalformedDigest(String),
    /// The staging request named no authority (`SIGNOFF-REPAIR.9.2.1.2.2`).
    MissingAuthority,
    /// The projection does not carry the policy version the proposal was
    /// approved for (`SIGNOFF-REPAIR.9.2.1.3.2`).
    ProjectionMissesApprovedPolicy {
        projection_id: String,
        policy_id: String,
        policy_version: String,
    },
    /// The projection predates `migrations/0082` and records no resolved set,
    /// so the question cannot be answered. ⛔ A DIFFERENT fact from the
    /// variant above, and said so: *unrecorded* is not *absent*.
    ProjectionResolvedSetUnrecorded(String),
    /// A caller asserted a manifest digest that is not the one staging derives
    /// (`SIGNOFF-REPAIR.9.2.1.3.1`). ⛔ BOTH values are named: a refusal that
    /// said only "wrong" would leave a caller unable to tell a stale client
    /// from a moved projection.
    DigestMismatch {
        asserted: String,
        derived: String,
    },
    EmptyObjectIds,
    /// Declared Git object ids that name nothing in the publication
    /// repository (`SIGNOFF-REPAIR.9.2.1.3`).
    UnknownObjects(Vec<String>),
    /// The publication repository could not be read at all — a deployment
    /// fault, and deliberately NOT the same answer as an absent object.
    RepositoryUnreadable(String),
    Duplicate(String),
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicationError::UnknownProposal(p) => {
                write!(f, "proposal `{p}` does not exist")
            }
            PublicationError::UnknownDecision(d) => {
                write!(f, "decision `{d}` does not exist")
            }
            PublicationError::UnknownApproval(a) => {
                write!(f, "approval `{a}` does not exist")
            }
            PublicationError::UnknownProjection(p) => {
                write!(f, "projection `{p}` does not exist")
            }
            PublicationError::ForeignRecord {
                kind,
                id,
                proposal_id,
            } => {
                write!(
                    f,
                    "the {kind} `{id}` does not belong to proposal `{proposal_id}`"
                )
            }
            PublicationError::WrongStage {
                publication_id,
                state,
            } => {
                write!(f, "publication `{publication_id}` is at stage `{state}` — the transition does not apply")
            }
            PublicationError::MissingAuthority => {
                write!(
                    f,
                    "the owning_authority is required — staging names a grant the caller HOLDS"
                )
            }
            PublicationError::ProjectionMissesApprovedPolicy {
                projection_id,
                policy_id,
                policy_version,
            } => {
                write!(
                    f,
                    "projection `{projection_id}` does not carry policy `{policy_id}` \
                     version {policy_version}, which is what this proposal was approved \
                     for — a publication may not publish bytes its approval never covered"
                )
            }
            PublicationError::ProjectionResolvedSetUnrecorded(p) => {
                write!(
                    f,
                    "projection `{p}` records no resolved policy set, so it cannot be \
                     shown to carry the approved policy — re-register it to record one"
                )
            }
            PublicationError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not the ADR-011 `sha256:<64 hex>` shape")
            }
            PublicationError::DigestMismatch { asserted, derived } => {
                write!(
                    f,
                    "the asserted manifest digest `{asserted}` is not this \
                     publication's — its manifest digest is `{derived}` \
                     (the server composes and hashes the manifest; a request \
                     need not carry one at all)"
                )
            }
            PublicationError::EmptyObjectIds => {
                write!(f, "the effective publication records its Git object ids")
            }
            PublicationError::UnknownObjects(ids) => {
                write!(
                    f,
                    "the declared Git object ids name nothing in the publication repository: {}",
                    ids.join(", ")
                )
            }
            PublicationError::RepositoryUnreadable(why) => {
                write!(f, "the publication repository cannot be read: {why}")
            }
            PublicationError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
        }
    }
}

fn is_sha256_hex(digest: &str) -> bool {
    digest
        .strip_prefix("sha256:")
        .map(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or(false)
}

/// The stored publication row shape (the query tuple).
type PublicationRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Value,
    Option<String>,
    // `owning_authority` (`.9.2.1.2.2`) — NULL for rows staged before it was
    // recorded, never a claim that nobody staged them.
    Option<String>,
);

/// Stage one publication (the §15.7 steps 1–4's record half): the
/// references must resolve AND belong to the proposal; the proposal must be
/// APPROVED (the publication follows the approval — the ADR-032 chain).
/// The publication must be one the caller's tenant OWNS
/// (`SIGNOFF-REPAIR.6.1.5.2.1`).
///
/// ⛔ `.6.1.5.2` gave every publication a tenant and deliberately gated nothing,
/// so until now any enrolled principal could mark another tenant's publication
/// effective, failed or published — the `held_publication_authority` sites
/// being the sharpest, because a grant the CALLER holds answers *may this
/// principal act on publications at all*, never *is this publication theirs*.
///
/// 🔴 **This sentence said "the three `held_publication_authority` sites" while
/// there were two** (`.9.2.1.2.1`): `mark_publication_failed` never called it,
/// so the comment described a repair the code did not have and a reader
/// auditing the surface from here would have concluded `failed` was bound.
/// There are three now, and the count is asserted by
/// `crates/reasonbraid-server/tests/policy.rs::the_failed_transition_requires_an_authority_the_caller_holds`
/// rather than by this sentence.
///
/// ⚠️ A foreign publication answers exactly as an ABSENT one
/// (`docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`), or
/// the refusal is an existence oracle over every other tenant's publication ids.
///
/// ⛔ A publication whose `tenant_id` is NULL is owned by NOBODY and may be
/// advanced by nobody. `migrations/0073` leaves a row NULL only when its lineage
/// is unattributable, and an unowned governance record that anyone may advance
/// is worse than one that is frozen — `.7.1.2.2`'s disposition for an
/// unattributable audit row, applied to a control surface.
pub async fn owned_by(
    pool: &PgPool,
    publication_id: &str,
    tenant_id: &str,
) -> Result<(), PublicationError> {
    let owner: Option<Option<String>> =
        sqlx::query_scalar("SELECT tenant_id FROM policy_publications WHERE publication_id = $1")
            .bind(publication_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    match owner.flatten() {
        Some(owner) if owner == tenant_id => Ok(()),
        _ => Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        )),
    }
}

pub async fn stage(
    pool: &PgPool,
    tenant_id: &str,
    input: &PublicationInput,
) -> Result<StoredPublication, PublicationError> {
    // ⛔ `.9.2.1.2.2`: graded HERE as well as in the handler. The handler must
    // answer first, because authorization precedes every lookup; this arm is
    // what makes the core safe for a caller that is not that handler.
    let Some(owning_authority) = input.owning_authority.clone() else {
        return Err(PublicationError::MissingAuthority);
    };
    // ⚠️ The shape check keeps its original POSITION — before any record is
    // looked up — so a malformed assertion is still refused without disclosing
    // anything about the proposal it names. Only its subject changed: it now
    // grades an OPTIONAL assertion rather than a required input.
    if let Some(asserted) = &input.manifest_digest {
        if !is_sha256_hex(asserted) {
            return Err(PublicationError::MalformedDigest(asserted.clone()));
        }
    }
    // ⛔ `tenant_id` joins the read `SIGNOFF-REPAIR.6.1.5.2`: a publication's
    // tenant is its PROPOSAL's, not its caller's. The publication is staged FROM
    // this proposal — every reference below is checked against it — so stamping
    // the caller's tenant on a publication of another tenant's proposal would
    // hide it from the only party it concerns, which is `.6.1.5`'s own trap
    // re-entered from the write side.
    //
    // ⚠️ This LABELS the row; it does not GATE the write. `stage` still admits
    // any enrolled principal to stage another tenant's approved proposal —
    // measured, and owned by `.6.1.5.2.1` rather than quietly widened here.
    // ⭐ `SIGNOFF-REPAIR.9.2.1.3.2`: the proposal's POLICY joins the read. The
    // decision and the approval were always matched to `proposal_id`; the
    // projection — the one reference that carries the bytes — was not, and
    // this is the value that lets it be.
    let proposal: Option<(String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT status, tenant_id, policy_id, policy_version \
         FROM policy_proposals WHERE proposal_id = $1",
    )
    .bind(&input.proposal_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(input.proposal_id.clone()))?;
    let Some((status, proposal_tenant, approved_policy_id, approved_policy_version)) = proposal
    else {
        return Err(PublicationError::UnknownProposal(input.proposal_id.clone()));
    };
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: the proposal must be the caller's. A foreign
    // one answers as an absent one — see `owned_by` above for why.
    if proposal_tenant.as_deref() != Some(tenant_id) {
        return Err(PublicationError::UnknownProposal(input.proposal_id.clone()));
    }
    if status != "approved" {
        return Err(PublicationError::WrongStage {
            publication_id: input.proposal_id.clone(),
            state: status,
        });
    }
    let decision_proposal: Option<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_decisions WHERE decision_id = $1")
            .bind(&input.decision_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownDecision(input.decision_id.clone()))?;
    match decision_proposal {
        None => return Err(PublicationError::UnknownDecision(input.decision_id.clone())),
        Some(proposal) if proposal != input.proposal_id => {
            return Err(PublicationError::ForeignRecord {
                kind: "decision".to_string(),
                id: input.decision_id.clone(),
                proposal_id: input.proposal_id.clone(),
            })
        }
        _ => {}
    }
    let approval_proposal: Option<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_approvals WHERE approval_id = $1")
            .bind(&input.approval_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownApproval(input.approval_id.clone()))?;
    match approval_proposal {
        None => return Err(PublicationError::UnknownApproval(input.approval_id.clone())),
        Some(proposal) if proposal != input.proposal_id => {
            return Err(PublicationError::ForeignRecord {
                kind: "approval".to_string(),
                id: input.approval_id.clone(),
                proposal_id: input.proposal_id.clone(),
            })
        }
        _ => {}
    }
    // ⛔ `SIGNOFF-REPAIR.6.1.5.3`: `AND tenant_id = $2`. The probe checked only
    // EXISTENCE, so a publication could be staged against ANOTHER tenant's
    // projection — its compiled bytes and its declared unrepresentables, which
    // are a function of that tenant's own resolution request. The decision and
    // the approval need no such predicate: both are matched to `proposal_id`,
    // and the proposal has already been proved the caller's.
    //
    // ⭐ `SIGNOFF-REPAIR.9.2.1.3.1`: this probe now returns the projection's
    // DIGEST rather than an `EXISTS`, because the manifest is composed from it.
    // Absence still answers exactly as it did — `UnknownProjection`, under the
    // same tenant predicate — so the refusal this query produces is unchanged.
    let projection: Option<(String, Option<Value>)> = sqlx::query_as(
        "SELECT digest, resolved_policies FROM policy_projections \
         WHERE projection_id = $1 AND tenant_id = $2",
    )
    .bind(&input.projection_id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProjection(input.projection_id.clone()))?;
    let Some((projection_digest, resolved_policies)) = projection else {
        return Err(PublicationError::UnknownProjection(
            input.projection_id.clone(),
        ));
    };
    // ⛔ `SIGNOFF-REPAIR.9.2.1.3.2` — THE CORRESPONDENCE. `publish` writes this
    // projection's bytes, and the authority for writing them is the approval
    // of a proposal that names a policy version. The bytes must carry it.
    //
    // ⚠️ The projection legitimately carries MORE than the proposal's policy —
    // it resolves a SET for a target layer, and that is what a deployment
    // consumes. The accepted, published residual is therefore that an approval
    // occasions the publication of a resolved set that includes it; what is
    // refused is a set that does not include it at all.
    //
    // ⛔ An UNRECORDED set fails CLOSED, and says which fact it is reporting.
    // `owned_by` takes the same disposition for an unattributable publication:
    // a governance record that cannot be shown to carry what was approved is
    // worse admitted than frozen.
    let Some(resolved_policies) = resolved_policies else {
        return Err(PublicationError::ProjectionResolvedSetUnrecorded(
            input.projection_id.clone(),
        ));
    };
    let resolved: Vec<crate::policy::PolicyRef> = serde_json::from_value(resolved_policies)
        .map_err(|_| {
            PublicationError::ProjectionResolvedSetUnrecorded(input.projection_id.clone())
        })?;
    if !resolved
        .iter()
        .any(|p| p.policy_id == approved_policy_id && p.version == approved_policy_version)
    {
        return Err(PublicationError::ProjectionMissesApprovedPolicy {
            projection_id: input.projection_id.clone(),
            policy_id: approved_policy_id,
            policy_version: approved_policy_version,
        });
    }
    // ADR-020 steps (2)–(3), finally performed here rather than delegated to
    // the caller: compose the manifest, hash it, and store THAT.
    let derived_digest = manifest_digest(&manifest(
        &input.publication_id,
        &input.proposal_id,
        &input.decision_id,
        &input.approval_id,
        &input.projection_id,
        &projection_digest,
    ));
    if let Some(asserted) = &input.manifest_digest {
        if asserted != &derived_digest {
            return Err(PublicationError::DigestMismatch {
                asserted: asserted.clone(),
                derived: derived_digest.clone(),
            });
        }
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
          manifest_digest, tenant_id, owning_authority) \
         VALUES ($1, $2, $3, $4, $5, 'staged', $6, $7, $8)",
    )
    .bind(&input.publication_id)
    .bind(&input.proposal_id)
    .bind(&input.decision_id)
    .bind(&input.approval_id)
    .bind(&input.projection_id)
    .bind(&derived_digest)
    .bind(&proposal_tenant)
    .bind(&owning_authority)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(PublicationError::Duplicate(format!(
            "publication `{}`",
            input.publication_id
        )));
    }
    Ok(StoredPublication {
        publication_id: input.publication_id.clone(),
        proposal_id: input.proposal_id.clone(),
        decision_id: input.decision_id.clone(),
        approval_id: input.approval_id.clone(),
        projection_id: input.projection_id.clone(),
        state: "staged".to_string(),
        manifest_digest: derived_digest,
        owning_authority: Some(owning_authority),
        git_object_ids: Vec::new(),
        failed_reason: None,
    })
}

/// Mark one publication EFFECTIVE (the §15.7 step 8's record half): the
/// staged → effective transition with the Git object ids recorded.
pub async fn mark_effective(
    pool: &PgPool,
    tenant_id: &str,
    publication_id: &str,
    git_object_ids: Vec<String>,
    repository: &crate::publisher::PublicationRepository,
) -> Result<StoredPublication, PublicationError> {
    if git_object_ids.is_empty() {
        return Err(PublicationError::EmptyObjectIds);
    }
    owned_by(pool, publication_id, tenant_id).await?;
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT state, publication_id FROM policy_publications WHERE publication_id = $1",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some((state, _)) = row else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    if state != "staged" {
        return Err(PublicationError::WrongStage {
            publication_id: publication_id.to_string(),
            state,
        });
    }
    // `SIGNOFF-REPAIR.9.2.1.3`: a declared object id is a CLAIM about the
    // repository, and nothing used to check it — the row recorded whatever the
    // caller sent, and three of this project's own fixtures drove the
    // transition with `abc123`, which is not even a well-formed id. ⛔ The
    // check lives HERE, in the core both publish verbs go through, rather than
    // in either handler: a seam-level repair would leave the sibling caller
    // open and the seam's own suite green (`.6.1.2`). The repository is a
    // `PublicationRepository`, which has no constructor but
    // `publisher::resolve_repository`, so it has already passed containment.
    // ⚠️ Ordered after the stage read on purpose, so the existing empty-list
    // and wrong-stage refusals answer first and a doomed request never pays
    // for a repository open.
    let missing = crate::publisher::missing_objects(repository, &git_object_ids)
        .map_err(|e| PublicationError::RepositoryUnreadable(e.to_string()))?;
    if !missing.is_empty() {
        return Err(PublicationError::UnknownObjects(missing));
    }
    sqlx::query(
        "UPDATE policy_publications SET state = 'effective', git_object_ids = $2 \
         WHERE publication_id = $1",
    )
    .bind(publication_id)
    .bind(serde_json::to_value(&git_object_ids).expect("the object ids serialize"))
    .execute(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    load(pool, publication_id).await
}

/// Mark one publication FAILED (the typed failure — never a skip): the
/// staged → failed transition with the reason.
pub async fn mark_failed(
    pool: &PgPool,
    tenant_id: &str,
    publication_id: &str,
    reason: &str,
) -> Result<StoredPublication, PublicationError> {
    owned_by(pool, publication_id, tenant_id).await?;
    let row: Option<String> =
        sqlx::query_scalar("SELECT state FROM policy_publications WHERE publication_id = $1")
            .bind(publication_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some(state) = row else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    if state != "staged" {
        return Err(PublicationError::WrongStage {
            publication_id: publication_id.to_string(),
            state,
        });
    }
    sqlx::query(
        "UPDATE policy_publications SET state = 'failed', failed_reason = $2 \
         WHERE publication_id = $1",
    )
    .bind(publication_id)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    load(pool, publication_id).await
}

/// Load one publication row (pub — the `.4.3.2` publish verb reads it).
/// ⚠️ `load` stays UNBOUND, deliberately, and this is one of the reads
/// `SIGNOFF-REPAIR.6.1.5.3` names rather than binds: every caller reaches it
/// only after `owned_by` has already refused a foreign publication, and the two
/// public entry points that read a publication by id — `GET /v1/policy-publications`
/// and the three transition verbs — are bound at their own boundary. A second
/// predicate here would be a third copy of one fact.
pub async fn load(
    pool: &PgPool,
    publication_id: &str,
) -> Result<StoredPublication, PublicationError> {
    let row: Option<PublicationRow> = sqlx::query_as(
        "SELECT publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
             manifest_digest, git_object_ids, failed_reason, owning_authority \
             FROM policy_publications WHERE publication_id = $1",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some((
        publication_id,
        proposal_id,
        decision_id,
        approval_id,
        projection_id,
        state,
        manifest_digest,
        git_object_ids,
        failed_reason,
        owning_authority,
    )) = row
    else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    Ok(StoredPublication {
        publication_id,
        proposal_id,
        decision_id,
        approval_id,
        projection_id,
        state,
        manifest_digest,
        owning_authority,
        git_object_ids: serde_json::from_value(git_object_ids).expect("the object ids parse"),
        failed_reason,
    })
}

/// The publications, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list(pool: &PgPool, tenant_id: &str) -> Result<Vec<StoredPublication>, sqlx::Error> {
    let rows: Vec<PublicationRow> = sqlx::query_as(
        "SELECT publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
             manifest_digest, git_object_ids, failed_reason, owning_authority \
             FROM policy_publications WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                publication_id,
                proposal_id,
                decision_id,
                approval_id,
                projection_id,
                state,
                manifest_digest,
                git_object_ids,
                failed_reason,
                owning_authority,
            )| StoredPublication {
                publication_id,
                proposal_id,
                decision_id,
                approval_id,
                projection_id,
                state,
                manifest_digest,
                owning_authority,
                git_object_ids: serde_json::from_value(git_object_ids)
                    .expect("the object ids parse"),
                failed_reason,
            },
        )
        .collect())
}
