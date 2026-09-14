//! The policy-lifecycle records (`PHASE-6.2.2`, ADR-032): the proposal and
//! the decision NEVER fold into the discussion — the proposal is a REFERENCE
//! (the policy version + the deliberation thread), the decision freezes the
//! ELECTORATE snapshot at the action time + the verdict reference, and the
//! proposal's status machine is the typed stage vocabulary (the `.2.3`
//! approval, the `.4` publication, and the `.5` deployment advance it).

use reasonbraid_core::GrantSubject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// The proposal-status vocabulary (the §15.6 stages, typed).
pub const PROPOSAL_STATUSES: [&str; 6] = [
    "draft",
    "decided",
    "approved",
    "published",
    "deployed",
    "withdrawn",
];

/// The proposal submission (`.2.2`): the target policy version + the
/// deliberation thread. A REFERENCE, never a content copy.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalInput {
    pub proposal_id: String,
    pub policy_id: String,
    pub policy_version: String,
    pub thread_id: String,
}

/// The decision submission (`.2.2`): the rule + the frozen electorate
/// snapshot + the verdict reference. The decision ADVANCES the proposal's
/// status (`draft` → `decided`) — one proposal, one decision.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionInput {
    pub decision_id: String,
    pub proposal_id: String,
    pub rule: String,
    pub electorate: Value,
    pub verdict_event_id: String,
}

/// The stored proposal row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredProposal {
    pub proposal_id: String,
    pub policy_id: String,
    pub policy_version: String,
    pub thread_id: String,
    pub status: String,
}

/// The stored decision row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredDecision {
    pub decision_id: String,
    pub proposal_id: String,
    pub rule: String,
    pub electorate: Value,
    pub verdict_event_id: String,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    UnknownPolicy(String),
    UnknownThread(String),
    UnknownProposal(String),
    WrongStage { proposal_id: String, status: String },
    UnknownVerdict(String),
    EmptyElectorate,
    Duplicate(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::UnknownPolicy(p) => {
                write!(f, "policy `{p}` at that version is not registered")
            }
            LifecycleError::UnknownThread(t) => {
                write!(f, "thread `{t}` does not exist")
            }
            LifecycleError::UnknownProposal(p) => {
                write!(f, "proposal `{p}` does not exist")
            }
            LifecycleError::WrongStage {
                proposal_id,
                status,
            } => {
                write!(f, "proposal `{proposal_id}` is at stage `{status}` — a decision rides a `draft` proposal only")
            }
            LifecycleError::UnknownVerdict(v) => {
                write!(
                    f,
                    "verdict event `{v}` is not a verdict contribution of the proposal's thread"
                )
            }
            LifecycleError::EmptyElectorate => {
                write!(f, "the electorate snapshot names at least one participant")
            }
            LifecycleError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
        }
    }
}

/// Register one proposal (the draft stage). The thread reference is checked
/// under the CALLER's tenant claim (`.1.3.1`): a proposal may only name a
/// thread of the caller's tenant — the RLS layer enforces the read.
pub async fn register_proposal(
    pool: &PgPool,
    tenant_id: &str,
    input: &ProposalInput,
) -> Result<StoredProposal, LifecycleError> {
    let policy: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM policy_versions WHERE policy_id = $1 AND version = $2)",
    )
    .bind(&input.policy_id)
    .bind(&input.policy_version)
    .fetch_one(pool)
    .await
    .map_err(|_| LifecycleError::UnknownPolicy(input.policy_id.clone()))?;
    if !policy.unwrap_or(false) {
        return Err(LifecycleError::UnknownPolicy(input.policy_id.clone()));
    }
    let thread_ref = input.thread_id.clone();
    let thread: Option<bool> = crate::rls::with_tenant_claim(pool, tenant_id, |tx| {
        Box::pin(async move {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM aggregate_state \
                 WHERE aggregate_id = $1 AND aggregate_type = 'thread')",
            )
            .bind(&thread_ref)
            .fetch_one(&mut *tx)
            .await
        })
    })
    .await
    .map_err(|_| LifecycleError::UnknownThread(input.thread_id.clone()))?;
    if !thread.unwrap_or(false) {
        return Err(LifecycleError::UnknownThread(input.thread_id.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status) \
         VALUES ($1, $2, $3, $4, 'draft')",
    )
    .bind(&input.proposal_id)
    .bind(&input.policy_id)
    .bind(&input.policy_version)
    .bind(&input.thread_id)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(StoredProposal {
            proposal_id: input.proposal_id.clone(),
            policy_id: input.policy_id.clone(),
            policy_version: input.policy_version.clone(),
            thread_id: input.thread_id.clone(),
            status: "draft".to_string(),
        }),
        Err(_) => Err(LifecycleError::Duplicate(format!(
            "proposal `{}`",
            input.proposal_id
        ))),
    }
}

/// Record one decision (the draft → decided transition). The verdict must be
/// a verdict-kind contribution of the PROPOSAL's thread — checked under the
/// CALLER's tenant claim (`.1.3.1`); the electorate snapshot freezes the
/// participants at the action time.
pub async fn record_decision(
    pool: &PgPool,
    tenant_id: &str,
    input: &DecisionInput,
) -> Result<StoredDecision, LifecycleError> {
    let proposal: Option<(String, String)> =
        sqlx::query_as("SELECT thread_id, status FROM policy_proposals WHERE proposal_id = $1")
            .bind(&input.proposal_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| LifecycleError::UnknownProposal(input.proposal_id.clone()))?;
    let Some((thread_id, status)) = proposal else {
        return Err(LifecycleError::UnknownProposal(input.proposal_id.clone()));
    };
    if status != "draft" {
        return Err(LifecycleError::WrongStage {
            proposal_id: input.proposal_id.clone(),
            status,
        });
    }
    let participants = input
        .electorate
        .get("participants")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if participants == 0 {
        return Err(LifecycleError::EmptyElectorate);
    }
    let verdict_ref = input.verdict_event_id.clone();
    let thread_ref = thread_id.clone();
    let verdict: Option<bool> = crate::rls::with_tenant_claim(pool, tenant_id, |tx| {
        Box::pin(async move {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM event_log \
                 WHERE event_id = $1 AND aggregate_id = $2 AND event_type = 'thread.contribution_submitted' \
                 AND body ->> 'kind' = 'verdict')",
            )
            .bind(&verdict_ref)
            .bind(&thread_ref)
            .fetch_one(&mut *tx)
            .await
        })
    })
    .await
    .map_err(|_| LifecycleError::UnknownVerdict(input.verdict_event_id.clone()))?;
    if !verdict.unwrap_or(false) {
        return Err(LifecycleError::UnknownVerdict(
            input.verdict_event_id.clone(),
        ));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&input.decision_id)
    .bind(&input.proposal_id)
    .bind(&input.rule)
    .bind(&input.electorate)
    .bind(&input.verdict_event_id)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(LifecycleError::Duplicate(format!(
            "decision `{}`",
            input.decision_id
        )));
    }
    sqlx::query("UPDATE policy_proposals SET status = 'decided' WHERE proposal_id = $1")
        .bind(&input.proposal_id)
        .execute(pool)
        .await
        .map_err(|_| LifecycleError::UnknownProposal(input.proposal_id.clone()))?;
    Ok(StoredDecision {
        decision_id: input.decision_id.clone(),
        proposal_id: input.proposal_id.clone(),
        rule: input.rule.clone(),
        electorate: input.electorate.clone(),
        verdict_event_id: input.verdict_event_id.clone(),
    })
}

/// The proposals, newest first.
pub async fn list_proposals(pool: &PgPool) -> Result<Vec<StoredProposal>, sqlx::Error> {
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT proposal_id, policy_id, policy_version, thread_id, status \
         FROM policy_proposals ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(proposal_id, policy_id, policy_version, thread_id, status)| StoredProposal {
                proposal_id,
                policy_id,
                policy_version,
                thread_id,
                status,
            },
        )
        .collect())
}

/// The decisions, newest first.
pub async fn list_decisions(pool: &PgPool) -> Result<Vec<StoredDecision>, sqlx::Error> {
    let rows: Vec<(String, String, String, Value, String)> = sqlx::query_as(
        "SELECT decision_id, proposal_id, rule, electorate, verdict_event_id \
         FROM policy_decisions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(decision_id, proposal_id, rule, electorate, verdict_event_id)| StoredDecision {
                decision_id,
                proposal_id,
                rule,
                electorate,
                verdict_event_id,
            },
        )
        .collect())
}

// ── The approval records + the authority proofs (`.2.3`, ADR-032) ──────────────────

/// The approval submission (`.2.3`): the proposal + the decision it approves,
/// the approver, the AUTHORITY PROOF (the grant id — re-checked at the
/// approval boundary: the status, the expiry, and the subject match), and
/// the quorum snapshot.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalInput {
    pub approval_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approver: String,
    pub grant_id: String,
    pub quorum: Value,
}

/// The stored approval row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredApproval {
    pub approval_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approver: String,
    pub grant_id: String,
    pub quorum: Value,
}

impl LifecycleError {
    fn unknown_decision(decision_id: &str) -> Self {
        LifecycleError::UnknownProposal(format!("decision `{decision_id}`"))
    }
    fn foreign_decision(decision_id: &str, proposal_id: &str) -> Self {
        LifecycleError::UnknownVerdict(format!(
            "decision `{decision_id}` does not belong to proposal `{proposal_id}`"
        ))
    }
    fn invalid_proof(grant_id: &str, approver: &str) -> Self {
        LifecycleError::UnknownVerdict(format!(
            "the authority proof fails: grant `{grant_id}` is not an active, unexpired \
             grant held by `{approver}`"
        ))
    }
    fn empty_quorum() -> Self {
        LifecycleError::EmptyElectorate
    }
}

/// Record one approval (the decided → approved transition). The authority
/// proof is the grant RE-CHECK at the approval boundary — the identity +
/// the authority at the action time (§4.5), never the proposal time's
/// memory.
pub async fn record_approval(
    pool: &PgPool,
    principal: &GrantSubject,
    input: &ApprovalInput,
) -> Result<StoredApproval, LifecycleError> {
    let proposal: Option<String> =
        sqlx::query_scalar("SELECT status FROM policy_proposals WHERE proposal_id = $1")
            .bind(&input.proposal_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| LifecycleError::UnknownProposal(input.proposal_id.clone()))?;
    let Some(status) = proposal else {
        return Err(LifecycleError::UnknownProposal(input.proposal_id.clone()));
    };
    if status != "decided" {
        return Err(LifecycleError::WrongStage {
            proposal_id: input.proposal_id.clone(),
            status,
        });
    }
    let decision: Option<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_decisions WHERE decision_id = $1")
            .bind(&input.decision_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| LifecycleError::unknown_decision(&input.decision_id))?;
    let Some(decision_proposal) = decision else {
        return Err(LifecycleError::unknown_decision(&input.decision_id));
    };
    if decision_proposal != input.proposal_id {
        return Err(LifecycleError::foreign_decision(
            &input.decision_id,
            &input.proposal_id,
        ));
    }
    let quorum_participants = input
        .quorum
        .get("participants")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if quorum_participants == 0 {
        return Err(LifecycleError::empty_quorum());
    }
    // The authority proof: the grant must be LIVE and HELD BY the approver —
    // and the approver must BE the authenticated caller.
    //
    // ⛔ `SIGNOFF-REPAIR.9.3.1` found this site half-bound and it was the
    // SAFEST of the five in the family, which is what made it worth reading
    // closely: it alone matched the grant to a subject. But the subject it
    // matched was `input.approver`, a string off the wire, and nothing tied
    // that to the caller — so a caller could approve AS another principal by
    // naming their grant, which is derivable from their id (`grt_<id>`). The
    // grant was bound to a CLAIM, and the claim to nobody.
    //
    // ⚠️ Its predicate also matched `subject_id` alone, without
    // `subject_kind`; the shared one binds both.
    if principal.id_string() != input.approver {
        return Err(LifecycleError::invalid_proof(
            &input.grant_id,
            &input.approver,
        ));
    }
    let proof = crate::authority::grant_held_by(pool, &input.grant_id, principal)
        .await
        .map_err(|_| LifecycleError::invalid_proof(&input.grant_id, &input.approver))?;
    if !proof {
        return Err(LifecycleError::invalid_proof(
            &input.grant_id,
            &input.approver,
        ));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_approvals \
         (approval_id, proposal_id, decision_id, approver, grant_id, quorum) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&input.approval_id)
    .bind(&input.proposal_id)
    .bind(&input.decision_id)
    .bind(&input.approver)
    .bind(&input.grant_id)
    .bind(&input.quorum)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(LifecycleError::Duplicate(format!(
            "approval `{}`",
            input.approval_id
        )));
    }
    sqlx::query("UPDATE policy_proposals SET status = 'approved' WHERE proposal_id = $1")
        .bind(&input.proposal_id)
        .execute(pool)
        .await
        .map_err(|_| LifecycleError::UnknownProposal(input.proposal_id.clone()))?;
    Ok(StoredApproval {
        approval_id: input.approval_id.clone(),
        proposal_id: input.proposal_id.clone(),
        decision_id: input.decision_id.clone(),
        approver: input.approver.clone(),
        grant_id: input.grant_id.clone(),
        quorum: input.quorum.clone(),
    })
}

/// The approvals, newest first.
pub async fn list_approvals(pool: &PgPool) -> Result<Vec<StoredApproval>, sqlx::Error> {
    let rows: Vec<(String, String, String, String, String, Value)> = sqlx::query_as(
        "SELECT approval_id, proposal_id, decision_id, approver, grant_id, quorum \
         FROM policy_approvals ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(approval_id, proposal_id, decision_id, approver, grant_id, quorum)| StoredApproval {
                approval_id,
                proposal_id,
                decision_id,
                approver,
                grant_id,
                quorum,
            },
        )
        .collect())
}
