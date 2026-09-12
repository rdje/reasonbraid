//! The control API for the WP6 CLI (`PHASE-0.6.1`): the command surface a human or
//! agent drives — enroll, create thread, invite, contribute, challenge, revise,
//! close, inspect — over HTTP JSON (`ROADMAP.md` §9.3/§9.4's public/control layer).
//!
//! # What this module owns
//!
//! - [`api_router`] — the axum surface. Thread commands ride the WP1
//!   [`CommandEnvelope`]; every state/event/idempotency/outbox write goes through the
//!   WP2 transaction (`tx`), every command through the WP5 authorization engine
//!   (`authority`), and every acceptance through the thread domain (`threads`).
//! - The **dev-profile actor resolution**: the `x-reasonbraid-principal` header
//!   carries the presented principal (`hpr_…`/`rol_…`). Authentication and
//!   certificate issuance are out of Phase 0 scope (`ID-003`; WP5 "development
//!   credentials") — the header is TRUSTED, documented, and the actor handle recorded
//!   in audit rows is derived deterministically from it
//!   ([`reasonbraid_core::actor_handle_for_subject`]), so audit rows stay linkable.
//! - The **request hash** for idempotency: SHA-256 over `operation`, the presented
//!   principal, and the canonical (struct-order) body JSON — the same key with a
//!   different body/hash is a conflict, never a silent overwrite (`§9.2`).
//!
//! # Idempotent rejections
//!
//! A rejection IS the semantic result of its command: the idempotency row stores
//! `{"ok": false, "error": {code, message}}`, and a replay returns the SAME status and
//! body (the code→status map is stable). A replayed success returns the original
//! result with `"replayed": true` added — the stored value itself is never mutated.

use std::sync::Arc;

mod bootstrap;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AgentRoleId, BoundaryStatus, BudgetDimensions, BudgetError,
    CommandEnvelope, EnrollmentAuthorityBoundary, GrantAction, GrantStatus, GrantSubject,
    HumanPrincipalId, ResourceTarget, RiskClass, TargetSelector, TenantId, ThreadId,
    PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::authority::{
    self, authorize_in_tx, AuthorizationOutcome, CommandAuthz, GrantCreateError, GrantRefused,
    GuardMode, TenantTransaction,
};
use crate::budget;
use crate::node_channel;
use crate::site_authority::{self as site, Reason, RegistryCommand, RegistryName};
use crate::threads::{self, CreateBody};
use crate::tx::{self, ApplyError, ClaimOutcome, Command};

/// The dev-profile principal header: the presented principal's wire id. Trusted and
/// documented (no certificate issuer in Phase 0).
pub const PRINCIPAL_HEADER: &str = "x-reasonbraid-principal";

// ── Errors ───────────────────────────────────────────────────────────────────────

/// A control-API error: a stable §9.8 machine-readable code and a safe message (no
/// secrets, no cross-tenant existence confirmation).
#[derive(Debug)]
pub struct ControlApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ControlApiError {
    pub fn unauthenticated(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthenticated",
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "unauthorized",
            message: message.into(),
        }
    }

    pub fn invalid_command(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_command",
            message: message.into(),
        }
    }

    pub fn invalid_transition(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "invalid_transition",
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    pub fn scope_hidden() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "scope_hidden",
            message: "the requested thread is not visible in this scope".to_string(),
        }
    }

    pub fn protocol_incompatible(got: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "protocol_incompatible",
            message: format!(
                "protocol version `{got}` is not supported (expected `{PROTOCOL_VERSION}`)"
            ),
        }
    }

    /// The quota window's ceiling is reached (`.1.3.2`) — the denial is
    /// recorded; the client may retry after the window slides.
    pub fn quota_exceeded(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "quota_exceeded",
            message: message.into(),
        }
    }

    /// The scope has NO configured quota (`.1.3.2`, fail-closed) — the
    /// surface refuses until the operator declares a bound.
    pub fn quota_unconfigured(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "quota_unconfigured",
            message: message.into(),
        }
    }

    /// The thread's classification lacks a qualified evaluator (`.1.4.3`,
    /// the evaluator-access control) — the delivery refuses until the
    /// deployment registers a qualified profile.
    pub fn classification_unqualified(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "classification_unqualified",
            message: message.into(),
        }
    }

    /// Retain caller-specific structural refusal prose while keeping missing
    /// authority and unavailable storage out of the violation list.
    fn grant_creation(error: GrantCreateError, refusal_context: &str) -> Self {
        match error {
            GrantCreateError::MissingBoundary { boundary_id } => {
                Self::invalid_command(format!("boundary `{boundary_id}` does not exist"))
            }
            GrantCreateError::Refused(GrantRefused { violations }) => {
                Self::invalid_command(format!(
                    "{refusal_context}: {}",
                    violations
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                ))
            }
            GrantCreateError::Storage(error) => error.into(),
            GrantCreateError::Transaction(error) => error.into(),
            GrantCreateError::BoundaryNotLive { boundary_id } => Self::invalid_command(format!(
                "boundary `{boundary_id}` is not live for grant issuance"
            )),
        }
    }

    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "dependency_unavailable",
            message: "internal server error".to_string(),
        }
    }

    /// [`Self::internal`] with the detail logged server-side (the wire keeps the
    /// safe generic message).
    pub fn internal_with_log(message: String) -> Self {
        eprintln!("control api: {message}");
        Self::internal()
    }

    /// The HTTP status a stored error code maps back to (idempotent replay of a
    /// stored rejection reproduces the ORIGINAL status).
    fn status_for_code(code: &str) -> StatusCode {
        match code {
            "unauthenticated" => StatusCode::UNAUTHORIZED,
            "unauthorized" => StatusCode::FORBIDDEN,
            "invalid_command" => StatusCode::BAD_REQUEST,
            "invalid_transition" => StatusCode::CONFLICT,
            "scope_hidden" => StatusCode::NOT_FOUND,
            "idempotency_mismatch" => StatusCode::CONFLICT,
            "protocol_incompatible" => StatusCode::BAD_REQUEST,
            "quota_exceeded" => StatusCode::TOO_MANY_REQUESTS,
            "quota_unconfigured" => StatusCode::SERVICE_UNAVAILABLE,
            "classification_unqualified" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stored failure result for a rejection ([`Self::failure_result`]).
    fn failure_result(&self) -> Value {
        json!({
            "ok": false,
            "error": { "code": self.code, "message": self.message },
        })
    }
}

impl std::fmt::Display for ControlApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ControlApiError {}

impl IntoResponse for ControlApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "code": self.code, "message": self.message })),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for ControlApiError {
    fn from(e: sqlx::Error) -> Self {
        eprintln!("control api: database error: {e}");
        ControlApiError::internal()
    }
}

impl From<authority::AuthorityTransactionError> for ControlApiError {
    fn from(error: authority::AuthorityTransactionError) -> Self {
        match error {
            authority::AuthorityTransactionError::Storage(error) => error.into(),
            error @ (authority::AuthorityTransactionError::Commit(_)
            | authority::AuthorityTransactionError::CommitDeadline) => {
                eprintln!("control api: {error}");
                if let Some(source) = std::error::Error::source(&error) {
                    eprintln!("control api: commit error source: {source}");
                }
                Self {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "commit_outcome_unconfirmed",
                    message:
                        "transaction outcome is unconfirmed; inspect the target before retrying"
                            .into(),
                }
            }
            error => Self::internal_with_log(error.to_string()),
        }
    }
}

impl From<ApplyError> for ControlApiError {
    fn from(e: ApplyError) -> Self {
        match e {
            ApplyError::IdempotencyConflict { .. } => ControlApiError {
                status: StatusCode::CONFLICT,
                code: "idempotency_mismatch",
                message: e.to_string(),
            },
            ApplyError::Sql(e) => {
                eprintln!("control api: database error: {e}");
                ControlApiError::internal()
            }
        }
    }
}

impl From<threads::ThreadError> for ControlApiError {
    fn from(e: threads::ThreadError) -> Self {
        match e {
            threads::ThreadError::InvalidCommand(msg) => ControlApiError::invalid_command(msg),
            threads::ThreadError::InvalidTransition(te) => {
                ControlApiError::invalid_transition(te.to_string())
            }
            threads::ThreadError::ThreadNotFound => ControlApiError::scope_hidden(),
            threads::ThreadError::NotAParticipant { principal } => ControlApiError::unauthorized(
                format!("principal `{principal}` is not a participant of this thread"),
            ),
            threads::ThreadError::AlreadyParticipant { principal } => {
                ControlApiError::invalid_command(format!(
                    "principal `{principal}` already participates in this thread"
                ))
            }
            threads::ThreadError::InvitationPending { principal } => {
                ControlApiError::invalid_transition(format!(
                    "principal `{principal}` is invited but has not accepted yet — \
                     thread.accept_invitation first"
                ))
            }
            threads::ThreadError::NoPendingInvitation { principal } => {
                ControlApiError::invalid_command(format!(
                    "principal `{principal}` has no pending invitation in this thread"
                ))
            }
            threads::ThreadError::InvitationExpired { principal } => {
                ControlApiError::invalid_transition(format!(
                    "principal `{principal}`'s invitation has expired"
                ))
            }
            threads::ThreadError::CorruptState(detail) => {
                eprintln!("control api: corrupt stored thread state: {detail}");
                ControlApiError::internal()
            }
            threads::ThreadError::QuotaRefused(q) => match q {
                crate::quota::QuotaError::Unconfigured { .. } => {
                    ControlApiError::quota_unconfigured(q.to_string())
                }
                crate::quota::QuotaError::Exceeded { .. } => {
                    ControlApiError::quota_exceeded(q.to_string())
                }
                crate::quota::QuotaError::Storage(detail) => {
                    eprintln!("control api: quota storage failure: {detail}");
                    ControlApiError::internal()
                }
            },
        }
    }
}

// ── Principal resolution (dev profile) ───────────────────────────────────────────

/// Resolve the presented principal from the trusted dev header. A missing or
/// malformed value is `unauthenticated` — the dev profile trusts the header, but it
/// must still be WELL-FORMED and typed.
fn resolve_principal(headers: &HeaderMap) -> Result<GrantSubject, ControlApiError> {
    let value = headers
        .get(PRINCIPAL_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ControlApiError::unauthenticated(format!(
                "missing `{PRINCIPAL_HEADER}` header (dev profile: hpr_… | rol_…)"
            ))
        })?;
    if let Ok(human) = value.parse::<HumanPrincipalId>() {
        return Ok(GrantSubject::Human(human));
    }
    if let Ok(role) = value.parse::<AgentRoleId>() {
        return Ok(GrantSubject::Role(role));
    }
    Err(ControlApiError::unauthenticated(format!(
        "malformed `{PRINCIPAL_HEADER}` value `{value}` (expected hpr_… | rol_…)"
    )))
}

/// Resolve the delegation from the envelope's `authority_context` (`.1.4.2`,
/// ADR-009 — chain-in-envelope): the subject rides `delegate_subject` (the
/// authority source), the requested scope rides `delegation_scope`. The actor
/// keeps its own identity (the caller check in the dual evaluation).
fn delegation_from_envelope(
    envelope: &CommandEnvelope,
) -> Result<(Option<GrantSubject>, Option<TargetSelector>), ControlApiError> {
    let Some(ctx) = &envelope.authority_context else {
        return Ok((None, None));
    };
    if let Ok(human) = ctx.on_behalf_of.parse::<HumanPrincipalId>() {
        return Ok((Some(GrantSubject::Human(human)), Some(ctx.scope.clone())));
    }
    if let Ok(role) = ctx.on_behalf_of.parse::<AgentRoleId>() {
        return Ok((Some(GrantSubject::Role(role)), Some(ctx.scope.clone())));
    }
    Err(ControlApiError::invalid_command(format!(
        "malformed `on_behalf_of` value `{}` (expected hpr_… | rol_…)",
        ctx.on_behalf_of
    )))
}

/// The canonical idempotency request hash: operation + presented principal +
/// canonical (struct-field-order) body JSON, SHA-256 hex.
pub(crate) fn request_hash(operation: &str, principal: &GrantSubject, body: &Value) -> String {
    let input = format!(
        "{operation}\n{}\n{}",
        principal.describe(),
        serde_json::to_string(body).expect("canonical body serializes")
    );
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

// ── State ────────────────────────────────────────────────────────────────────────

pub struct ApiState {
    pool: PgPool,
    /// The built-in R0 fetcher (the `.2.3` pack wiring): the production
    /// shape — https-only, the system roots, the `.2.1` public-only policy.
    fetcher: std::sync::Arc<crate::fetcher::Fetcher>,
    /// The built-in R1 acquirer (the `.3.3` pack wiring): the same
    /// public-only policy through the classified git transport.
    git_fetcher: std::sync::Arc<crate::git::GitFetcher>,
    /// The `.5.3` OPT-IN gate: the R3/R5/RX packs resolve ONLY while this
    /// flag is set (the startup sync keeps the registry rows in step).
    r5r3rx_enabled: bool,
    /// The local credential broker (the R5 pack's engine).
    broker: std::sync::Arc<crate::broker::Broker>,
}

/// The gate's environment switch: `RB_ENABLE_R5R3RX=1|true` — OFF by
/// default everywhere (the `.5.1` contract).
pub fn r5r3rx_enabled() -> bool {
    matches!(
        std::env::var("RB_ENABLE_R5R3RX").as_deref(),
        Ok("1" | "true" | "TRUE")
    )
}

impl ApiState {
    pub fn new(pool: PgPool) -> Self {
        Self::with_gate(
            pool,
            r5r3rx_enabled(),
            std::sync::Arc::new(crate::broker::Broker::default()),
        )
    }

    /// The test/deployment seam: an explicit gate + a pre-populated broker,
    /// with the PRODUCTION acquisition fetcher (https-only, the system roots,
    /// the `.2.1` public-only policy).
    pub fn with_gate(
        pool: PgPool,
        enabled: bool,
        broker: std::sync::Arc<crate::broker::Broker>,
    ) -> Self {
        Self::with_acquisition(
            pool,
            enabled,
            broker,
            std::sync::Arc::new(
                crate::fetcher::Fetcher::new(crate::fetcher::FetchLimits::default())
                    .expect("the built-in R0 fetcher builds (the system roots are present)"),
            ),
        )
    }

    /// The acquisition seam (`SIGNOFF-REPAIR.7.3.3.4.1`): the deployment
    /// supplies the R0 fetcher every acquisition leg uses.
    ///
    /// This is a construction profile, not a relaxation: `new` and `with_gate`
    /// keep building `Fetcher::new`, so the shipped https-only public-destination
    /// policy is unchanged wherever they are used — which is everywhere the
    /// server binary constructs its state. A caller that wants another
    /// destination policy has to say so here, in its own source, and owns what
    /// it admitted. The live R2 control uses it to acquire from an origin it
    /// runs itself; production could use it for a deployment whose egress is
    /// a listed internal mirror.
    pub fn with_acquisition(
        pool: PgPool,
        enabled: bool,
        broker: std::sync::Arc<crate::broker::Broker>,
        fetcher: std::sync::Arc<crate::fetcher::Fetcher>,
    ) -> Self {
        Self {
            pool,
            fetcher,
            git_fetcher: std::sync::Arc::new(crate::git::GitFetcher::new(
                crate::git::GitLimits::default(),
            )),
            r5r3rx_enabled: enabled,
            broker,
        }
    }
}

/// The control API router (`§9.4` Phase 0 subset).
pub fn api_router(pool: PgPool) -> Router {
    let state = Arc::new(ApiState::new(pool));
    api_router_with_state(state)
}

/// The test seam: an explicit gate + a pre-populated broker (the
/// profiles suite drives both gate states).
pub fn api_router_gated(
    pool: PgPool,
    enabled: bool,
    broker: std::sync::Arc<crate::broker::Broker>,
) -> Router {
    api_router_with_state(Arc::new(ApiState::with_gate(pool, enabled, broker)))
}

/// The acquisition seam's router (`SIGNOFF-REPAIR.7.3.3.4.1`): the same
/// router over a state whose R0 fetcher the caller supplied. `api_router`
/// and `api_router_gated` still build the production fetcher.
pub fn api_router_with_acquisition(
    pool: PgPool,
    enabled: bool,
    broker: std::sync::Arc<crate::broker::Broker>,
    fetcher: std::sync::Arc<crate::fetcher::Fetcher>,
) -> Router {
    api_router_with_state(Arc::new(ApiState::with_acquisition(
        pool, enabled, broker, fetcher,
    )))
}

fn api_router_with_state(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/v1/enrollments", post(enroll))
        .route("/v1/nodes/enroll-tokens", post(issue_node_enroll_token))
        .route("/v1/nodes/quarantine", post(quarantine_command))
        .route("/v1/nodes/replay", post(replay_command))
        .route("/v1/nodes/inbox", get(inspect_node_inbox))
        .route("/v1/nodes/inbox/prune", post(prune_node_inbox))
        .route("/v1/nodes/revoke", post(revoke_node))
        .route(
            "/v1/federation-agreements",
            post(propose_federation_agreement),
        )
        .route(
            "/v1/federation-agreements/accept",
            post(accept_federation_agreement),
        )
        .route(
            "/v1/federation-agreements/revoke",
            post(revoke_federation_agreement),
        )
        .route("/v1/admin/grants/{grant_id}/revoke", post(revoke_grant))
        .route(
            "/v1/admin/boundaries/{boundary_id}/revoke",
            post(revoke_boundary),
        )
        .route("/v1/admin/grants", get(list_grants))
        .route(
            "/v1/admin/authorization-records/{record_id}",
            get(inspect_authorization_record),
        )
        .route("/v1/admin/boundaries", get(list_boundaries))
        .route("/v1/admin/incarnations", get(list_incarnations))
        .route("/v1/admin/runs", get(list_runs))
        .route(
            "/v1/admin/breakers",
            post(arm_breaker).get(inspect_breakers),
        )
        .route("/v1/admin/breakers/reset", post(reset_breaker))
        .route("/v1/admin/usage", get(admin_usage))
        .route("/v1/admin/adapters", get(list_adapters).post(allow_adapter))
        .route(
            "/v1/admin/adapters/{adapter_id}/revoke",
            post(revoke_adapter),
        )
        .route("/v1/admin/regions", get(list_regions).post(declare_region))
        .route("/v1/admin/regions/{from}/pair/{to}", post(pair_regions))
        .route("/v1/admin/regions/{from}/unpair/{to}", post(unpair_regions))
        .route("/v1/audit/receipts", get(list_cross_domain_receipts))
        .route("/v1/admin/metrics", get(admin_metrics))
        .route("/v1/admin/nodes/presence", get(list_node_presence))
        .route("/v1/directory/presence", get(directory_presence))
        .route("/v1/directory/match", post(directory_match))
        .route("/v1/calls", post(open_recruitment_call))
        .route("/v1/calls/{call_id}/respond", post(respond_to_call))
        .route("/v1/calls/{call_id}/close", post(close_call))
        .route("/v1/calls/{call_id}", get(inspect_call))
        .route("/v1/profiles/{role_id}", put(put_profile).get(get_profile))
        .route("/v1/profiles/{role_id}/card", get(get_profile_card))
        .route("/v1/profiles/cards/import", post(import_profile_card))
        .route(
            "/v1/profiles/{role_id}/versions",
            get(list_profile_versions),
        )
        .route(
            "/v1/profiles/{role_id}/versions/{version}",
            get(get_profile_version),
        )
        .route(
            "/v1/profiles/{role_id}/attest",
            post(attest_capability_claim),
        )
        .route("/v1/resources", post(submit_resource))
        .route("/v1/resources/{resource_id}", get(get_resource))
        .route(
            "/v1/resources/{resource_id}/resolve",
            post(resolve_resource),
        )
        .route(
            "/v1/workflow-profiles",
            post(register_workflow_profile).get(list_workflow_profiles),
        )
        .route(
            "/v1/evaluations/corpora",
            post(register_evaluation_corpus).get(list_evaluation_corpora),
        )
        .route(
            "/v1/evaluations/runs",
            post(record_evaluation_run).get(list_evaluation_runs),
        )
        .route(
            "/v1/evaluations/trials",
            post(create_evaluation_trial).get(list_evaluation_trials),
        )
        .route(
            "/v1/evaluations/trials/{trial_id}/results",
            post(record_trial_results).get(list_trial_results),
        )
        .route(
            "/v1/evaluations/calibrations",
            post(record_evaluation_calibration).get(list_evaluation_calibrations),
        )
        .route(
            "/v1/evaluations/gates",
            post(record_evaluation_gate).get(list_evaluation_gates),
        )
        .route(
            "/v1/evaluations/gates/{gate_id}/evaluations",
            post(evaluate_gate_endpoint).get(list_gate_results),
        )
        .route("/v1/routing/rules", get(list_routing_rules))
        .route("/v1/routing/resolve", post(resolve_routing_class))
        .route("/v1/routing/resolutions", get(list_routing_resolutions))
        .route(
            "/v1/routing/recommendations",
            post(record_routing_recommendation).get(list_routing_recommendations),
        )
        .route("/v1/policies", post(register_policy).get(list_policies))
        .route("/v1/policies/resolve", post(resolve_policies))
        .route(
            "/v1/policy-proposals",
            post(register_policy_proposal).get(list_policy_proposals),
        )
        .route(
            "/v1/policy-decisions",
            post(record_policy_decision).get(list_policy_decisions),
        )
        .route(
            "/v1/policy-approvals",
            post(record_policy_approval).get(list_policy_approvals),
        )
        .route(
            "/v1/policy-projections",
            post(project_policies).get(list_policy_projections),
        )
        .route(
            "/v1/policy-publications",
            post(stage_publication).get(list_publications),
        )
        .route(
            "/v1/policy-publications/{publication_id}/effective",
            post(mark_publication_effective),
        )
        .route(
            "/v1/policy-publications/{publication_id}/failed",
            post(mark_publication_failed),
        )
        .route(
            "/v1/policy-publications/{publication_id}/publish",
            post(publish_publication),
        )
        .route(
            "/v1/deployment-targets",
            post(register_deployment_target).get(list_deployment_targets),
        )
        .route(
            "/v1/deployments",
            post(assign_deployment).get(list_deployments),
        )
        .route(
            "/v1/deployments/{target_id}/{publication_id}/receipt",
            post(record_deployment_receipt),
        )
        .route(
            "/v1/policy-drift",
            post(record_policy_drift).get(list_policy_drift),
        )
        .route(
            "/v1/policy-corrections",
            post(record_policy_correction).get(list_policy_corrections),
        )
        .route(
            "/v1/policy-outcomes",
            post(record_policy_outcome).get(list_policy_outcomes),
        )
        .route("/v1/policy-reviews", get(list_policy_reviews))
        .route("/v1/policy-reviews/schedule", post(schedule_policy_reviews))
        .route(
            "/v1/policy-reviews/{review_id}/done",
            post(mark_policy_review_done),
        )
        .route(
            "/v1/policies/{policy_id}/{version}/impact",
            get(policy_impact),
        )
        .route("/v1/snapshots", post(submit_snapshot))
        .route("/v1/snapshots/expire-due", post(expire_due_snapshots))
        .route("/v1/snapshots/stale", get(list_stale_snapshots))
        .route("/v1/derivations", post(submit_derivation))
        .route("/v1/assessments", post(submit_assessment))
        .route(
            "/v1/snapshots/{snapshot_id}/assessments",
            get(list_snapshot_assessments),
        )
        .route(
            "/v1/claims/{claim_id}/assessments",
            get(list_claim_assessments),
        )
        .route(
            "/v1/snapshots/{snapshot_id}/derivations",
            get(list_derivations),
        )
        .route(
            "/v1/snapshots/{snapshot_id}",
            get(get_snapshot).delete(tombstone_snapshot),
        )
        .route("/v1/resolvers", post(register_resolver))
        .route("/v1/threads", post(create_thread))
        .route("/v1/threads/auto", post(create_thread_auto))
        .route("/v1/threads", get(list_threads))
        .route("/v1/threads/{thread_id}", get(get_thread))
        .route("/v1/threads/{thread_id}/events", get(get_events))
        .route("/v1/threads/{thread_id}/audit", get(get_audit))
        .route("/v1/threads/{thread_id}/budget", get(get_thread_budget))
        .route("/v1/threads/{thread_id}/commands", post(thread_command))
        .with_state(state)
}

/// A success/failure JSON result, out through the right status.
fn json_response(status: StatusCode, body: Value) -> Response {
    (status, Json(body)).into_response()
}

// ── Enroll (dev bootstrap) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollRequest {
    /// Omitted: bootstrap a NEW tenant (human enrollment only — the tenant's
    /// boundary is created here). Present: enroll into an existing tenant.
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Canonical RequestId for recovery of one new-human bootstrap. Persist it
    /// before sending; omit for intentionally distinct no-key requests.
    #[serde(default)]
    pub bootstrap_request_id: Option<String>,
    /// `"human"` or `"role"`.
    pub kind: String,
    pub name: String,
    /// For a role: the granted actions (wire names). Defaults: thread_contribute
    /// and thread_invitation_respond.
    /// For a human: always the dev admin set (bootstrap trust — documented).
    #[serde(default)]
    pub actions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrollResponse {
    pub tenant_id: String,
    pub principal_id: String,
    pub kind: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_request_id: Option<String>,
    /// `true` for an existing (tenant, kind, name) or keyed bootstrap outcome:
    /// the original principal is returned; no new identity or grant was created.
    pub replayed: bool,
}

/// The dev admin action set a bootstrap human receives. It includes `tenant_admin`
/// EXPLICITLY — never implied (`.5.1`); `thread_cancel` joins in `.1.1.3`,
/// `thread_invitation_respond` in `.1.3.1`, `thread_advance_round` in `.1.5.2`.
const ADMIN_ACTIONS: [GrantAction; 9] = [
    GrantAction::ThreadCreate,
    GrantAction::ThreadInvite,
    GrantAction::ThreadContribute,
    GrantAction::ThreadInspect,
    GrantAction::ThreadClose,
    GrantAction::ThreadCancel,
    GrantAction::ThreadInvitationRespond,
    GrantAction::ThreadAdvanceRound,
    GrantAction::TenantAdmin,
];

/// The dev enrollment boundary for a fresh tenant (one ACTIVE per tenant).
fn dev_boundary(tenant_id: &TenantId, now: DateTime<Utc>) -> EnrollmentAuthorityBoundary {
    EnrollmentAuthorityBoundary {
        boundary_id: format!("bnd_{tenant_id}"),
        tenant_id: *tenant_id,
        parent_or_root_authority: "dev-root".to_string(),
        target_owner: "dev-operator".to_string(),
        permitted_actions: ADMIN_ACTIONS.to_vec(),
        permitted_domains: vec!["deliberation".to_string()],
        risk_ceiling: RiskClass::Low,
        spend_ceiling: Some(json!({ "amount": 1000.0 })),
        delegable: false,
        max_delegation_depth: 0,
        valid_from: now,
        expires_at: now + chrono::Duration::days(365),
        charter_digest: "dev-charter-000".to_string(),
        policy_version: "dev-authz-1".to_string(),
        status: BoundaryStatus::Active,
    }
}

fn dev_grant(
    boundary: &EnrollmentAuthorityBoundary,
    issuer: HumanPrincipalId,
    subject: GrantSubject,
    actions: Vec<GrantAction>,
) -> reasonbraid_core::AuthorityGrant {
    reasonbraid_core::AuthorityGrant {
        grant_id: format!("grt_{}", subject.id_string()),
        boundary_id: boundary.boundary_id.clone(),
        tenant_id: boundary.tenant_id,
        issuer,
        subject,
        actions,
        selector: TargetSelector::TenantWide,
        risk_ceiling: RiskClass::Low,
        spend_limits: None,
        delegable: false,
        // Coextensive with the boundary: a grant must never outlive its boundary
        // (the subset checker enforces it; wall-clock skew between enroll calls
        // would otherwise make a later grant overrun an earlier boundary's window).
        valid_from: boundary.valid_from,
        expires_at: boundary.expires_at,
        status: GrantStatus::Active,
    }
}

async fn enroll(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<EnrollRequest>,
) -> Result<Json<EnrollResponse>, ControlApiError> {
    let kind = match req.kind.as_str() {
        "human" => "human",
        "role" => "role",
        other => {
            return Err(ControlApiError::invalid_command(format!(
                "kind `{other}` is not supported (expected `human` or `role`)"
            )))
        }
    };
    let tenant_id: TenantId = match &req.tenant_id {
        Some(raw) => raw.parse().map_err(|_| {
            ControlApiError::invalid_command(format!("tenant_id `{raw}` is malformed"))
        })?,
        None => {
            if kind == "role" {
                return Err(ControlApiError::invalid_command(
                    "a role enrolls into an existing tenant — tenant_id is required",
                ));
            }
            TenantId::new()
        }
    };

    if let Some(raw) = &req.bootstrap_request_id {
        let key = bootstrap::validate_key(raw, &req)?;
        return bootstrap::enroll(&state.pool, req, key, tenant_id).await;
    }

    authority::transact_with_error(
        &state.pool,
        &[(tenant_id, GuardMode::Exclusive)],
        authority::Limits::default(),
        move |tx| Box::pin(enroll_in_guard(tx, req, kind, tenant_id)),
    )
    .await
}

/// Replay and every authority/identity/quota write share this exclusive guard.
/// Returning a typed error aborts all provisional work; only the outer owner
/// commits and can report the successful or replayed response.
async fn enroll_in_guard(
    tx: &mut TenantTransaction<'_>,
    req: EnrollRequest,
    kind: &'static str,
    tenant_id: TenantId,
) -> Result<Json<EnrollResponse>, ControlApiError> {
    // Replay: the same (tenant, kind, name) returns the ORIGINAL principal id.
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT principal_id, kind FROM enrollments \
         WHERE tenant_id = $1 AND kind = $2 AND name = $3",
    )
    .bind(tenant_id.to_string())
    .bind(kind)
    .bind(&req.name)
    .fetch_optional(tx.connection(tenant_id, GuardMode::Exclusive)?)
    .await?;
    if let Some((principal_id, stored_kind)) = existing {
        return Ok(Json(EnrollResponse {
            tenant_id: tenant_id.to_string(),
            principal_id,
            kind: stored_kind,
            name: req.name,
            boundary_id: None,
            grant_id: None,
            bootstrap_request_id: None,
            replayed: true,
        }));
    }

    let principal: GrantSubject = if kind == "human" {
        GrantSubject::Human(HumanPrincipalId::new())
    } else {
        GrantSubject::Role(AgentRoleId::new())
    };

    // The bootstrap human issues its own dev grant (no certificate issuer in Phase 0;
    // grant issuance is dev-trusted — documented). The issuer id is the human's own
    // for a human enrollment, and the enrolling tenant's bootstrap human is not
    // known for a role — the dev profile records a fresh human issuer handle.
    // That handle is not an authenticated issuer identity (recorded limitation).
    let issuer = match principal {
        GrantSubject::Human(h) => h,
        GrantSubject::Role(_) => HumanPrincipalId::new(),
    };

    // New tenant, boundary, grant, identity, quota and enrollment commit together.
    let mut boundary = None;
    if req.tenant_id.is_none() {
        let b = dev_boundary(&tenant_id, tx.database_now().await?);
        // The tenant's identity row FIRST: the human_principals insert below
        // references it (PHASE-1.1.2, migrations/0007).
        sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
            .bind(tenant_id.to_string())
            .execute(tx.connection(tenant_id, GuardMode::Exclusive)?)
            .await?;
        // The tenant's default quotas (`.1.3.2`): the invite-storm bound
        // rides the SAME transaction — a tenant exists with its bounds.
        crate::quota::insert_defaults_in_tx(
            tx.connection(tenant_id, GuardMode::Exclusive)?,
            &tenant_id.to_string(),
        )
        .await?;
        authority::insert_boundary_in_tx(tx.connection(tenant_id, GuardMode::Exclusive)?, &b)
            .await?;
        boundary = Some(b);
    }

    let boundary_ref = match &boundary {
        Some(b) => b.clone(),
        None => authority::load_active_boundary_in_guard(tx, tenant_id)
            .await?
            .ok_or_else(|| {
                ControlApiError::invalid_command(
                    "the tenant has no active enrollment boundary — enroll a human first",
                )
            })?,
    };

    let actions: Vec<GrantAction> = if kind == "human" {
        ADMIN_ACTIONS.to_vec()
    } else {
        match &req.actions {
            None => vec![
                GrantAction::ThreadContribute,
                // `.1.3.1`: the invitation-response right every role's default
                // carries — the invitation itself stays the real capability.
                GrantAction::ThreadInvitationRespond,
            ],
            Some(names) => names
                .iter()
                .map(|n| {
                    n.parse::<GrantAction>().map_err(|_| {
                        ControlApiError::invalid_command(format!(
                            "action `{n}` is not in the dev registry ({})",
                            ADMIN_ACTIONS
                                .iter()
                                .map(|a| a.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        }
    };

    let grant = dev_grant(&boundary_ref, issuer, principal.clone(), actions);
    authority::create_grant_in_guard(tx, &grant)
        .await
        .map_err(|error| {
            ControlApiError::grant_creation(error, "the dev grant exceeds its boundary")
        })?;

    // The identity record (PHASE-1.1.2, migrations/0007): the principal's row in
    // its identity table commits in the SAME transaction as the enrollment row —
    // an enrollment implies its identity row. The tenants row already exists
    // (the bootstrap branch above, or an earlier bootstrap's transaction); an
    // enrollment into a tenant with no tenants row fails the FK — fail closed,
    // never a silent half-identity.
    match &principal {
        GrantSubject::Human(h) => {
            sqlx::query(
                "INSERT INTO human_principals (principal_id, tenant_id, name) \
                 VALUES ($1, $2, $3)",
            )
            .bind(h.to_string())
            .bind(tenant_id.to_string())
            .bind(&req.name)
            .execute(tx.connection(tenant_id, GuardMode::Exclusive)?)
            .await?;
        }
        GrantSubject::Role(r) => {
            sqlx::query("INSERT INTO agent_roles (role_id, tenant_id, name) VALUES ($1, $2, $3)")
                .bind(r.to_string())
                .bind(tenant_id.to_string())
                .bind(&req.name)
                .execute(tx.connection(tenant_id, GuardMode::Exclusive)?)
                .await?;
        }
    }

    // The MCP write gate's per-principal quota (`.3.5.1`): the identity row
    // implies its quota row — the fail-closed check refuses an unbound
    // principal, so the bound must exist from the principal's creation.
    crate::quota::insert_principal_default_in_tx(
        tx.connection(tenant_id, GuardMode::Exclusive)?,
        &tenant_id.to_string(),
        &principal.id_string(),
    )
    .await?;

    sqlx::query(
        "INSERT INTO enrollments (principal_id, tenant_id, kind, name) VALUES ($1, $2, $3, $4)",
    )
    .bind(principal.id_string())
    .bind(tenant_id.to_string())
    .bind(kind)
    .bind(&req.name)
    .execute(tx.connection(tenant_id, GuardMode::Exclusive)?)
    .await?;

    Ok(Json(EnrollResponse {
        tenant_id: tenant_id.to_string(),
        principal_id: principal.id_string(),
        kind: kind.to_string(),
        name: req.name,
        boundary_id: boundary.as_ref().map(|b| b.boundary_id.clone()),
        grant_id: Some(grant.grant_id),
        bootstrap_request_id: None,
        replayed: false,
    }))
}

// ── Node enrollment (admin side, PHASE-1.2.1) ───────────────────────────────────

/// The `POST /v1/nodes/enroll-tokens` body: an authorized human issues a ONE-TIME
/// enrollment token bound to tenant + expected node id + host claim + expiry + nonce
/// (`ROADMAP.md` §16.2). The node consumes it at `POST /v1/nodes/enroll`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueNodeTokenRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub host_claim: String,
    /// Default 3600 s (the dev profile; a consumed token has no second use anyway).
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueNodeTokenResponse {
    pub token_id: String,
    pub nonce: String,
    pub expires_at: String,
}

/// Issue a one-time node enrollment token.
///
/// Since `SIGNOFF-REPAIR.3.3.4.10.1` the admission, the token INSERT and the
/// final effect record share ONE transaction under the tenant's SHARED authority
/// guard. Before this the handler admitted through `authorize_guarded` — its own
/// transaction — and then inserted **on the connection pool**, outside any
/// transaction, so nothing ordered the write against the admission that
/// permitted it and no record said what the request finally did.
///
/// The guard is shared rather than exclusive, and that is derived rather than
/// inherited from `.9`: the refusal is decided atomically by a single
/// `ON CONFLICT DO NOTHING RETURNING`, same-node contention is already
/// serialized by the partial unique index, and a revocation's exclusive mode
/// fences this transaction whole either way.
///
/// Every answer carries the `x-reasonbraid-authorization` receipt naming the
/// admission — which is the effect record's id when one was written. The
/// validation refusal below is deliberately OUTSIDE that: it happens before any
/// admission exists, so there is no receipt to name.
async fn issue_node_enroll_token(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<IssueNodeTokenRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if !crate::node_channel::is_valid_node_identity(&req.node_id) {
        return Err(ControlApiError::invalid_command(format!(
            "node_id `{}` is not a valid node identity (a `nod_…` node id or the `rol_…` \
             role wire id the dev profile serves)",
            req.node_id
        )));
    }

    let issue = authority::issue_enrollment_token_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.host_claim,
        chrono::Duration::seconds(req.ttl_seconds.unwrap_or(3600)),
    )
    .await?;
    let receipt = issue.record_id;
    let response = match issue.result {
        authority::TokenIssueResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // One UNUSED token per node (migration 0018's partial index): a re-issue
        // while one is outstanding is a TYPED refusal, never a database error on
        // the wire. The status, code and message are unchanged.
        authority::TokenIssueResult::AlreadyOutstanding => ControlApiError {
            status: StatusCode::CONFLICT,
            code: "invalid_command",
            message: format!(
                "an unused enrollment token for node `{}` already exists — \
                 consume or expire it before issuing another",
                req.node_id
            ),
        }
        .into_response(),
        authority::TokenIssueResult::Issued {
            token_id,
            nonce,
            expires_at,
        } => Json(IssueNodeTokenResponse {
            token_id,
            nonce,
            // Database time from inside the transaction plus the requested TTL,
            // so the token's lifetime runs from the instant the decision was
            // made rather than from a process clock read before the guard wait.
            expires_at: expires_at.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

// ── Node inbox hardening (admin side, PHASE-1.2.3) ──────────────────────────────

/// The `tenant_admin` gate the inbox operator actions share with token issuance:
/// the decision is audited by [`authorize`] (allowed or denied — the `.5.1`
/// stance), so quarantine and prune leave an authorization record behind them.
async fn authorize_tenant_admin(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
) -> Result<(), ControlApiError> {
    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant { tenant_id },
    };
    match authority::authorize_guarded(pool, &authz).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )))
        }
        AuthorizationOutcome::Allowed { .. } => Ok(()),
    }
}

/// The `POST /v1/nodes/replay` body (`.2.4`): the operator re-delivers ONE
/// dead-lettered command — the quarantine clears, the admission decision
/// refreshes (a fresh `decided_at` + the CURRENT revocation epoch — the old
/// decision's facts stay bound), and the row re-sequences to the node's tail
/// so the next poll delivers it again.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub command_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayResponse {
    pub node_id: String,
    pub command_id: String,
    pub replayed_at: String,
}

/// Replay one dead-lettered inbox command.
///
/// Since `SIGNOFF-REPAIR.3.3.4.10.3` the admission, the tenant-bound row
/// selection, the mutation and the final effect record share ONE transaction
/// under the tenant's SHARED authority guard. Before this the admission ran in
/// its own transaction and the mutation in a second, unguarded one whose
/// predicate was `node_id` and `command_id` alone — so an administrator of one
/// tenant could replay another tenant's inbox row, measured at 200.
async fn replay_command(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ReplayRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let replay = authority::replay_command_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.command_id,
    )
    .await?;
    let receipt = replay.record_id;
    let response = match replay.result {
        authority::ReplayResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // Missing and foreign are ONE answer (`SIGNOFF-REPAIR.3.1`).
        authority::ReplayResult::NotInInbox => ControlApiError::not_found(format!(
            "no command `{}` in node `{}`'s inbox",
            req.command_id, req.node_id
        ))
        .into_response(),
        // ⚠️ This also answers the caller that LOST a concurrent replay. The row
        // lock makes the two states indistinguishable, and the superseded
        // "a concurrent replay won" message is retired rather than kept as a
        // claim about a race the lock no longer permits.
        authority::ReplayResult::NotDeadLettered => ControlApiError::invalid_transition(
            "the command is not dead-lettered — replay only reverses a quarantine",
        )
        .into_response(),
        authority::ReplayResult::Replayed => Json(ReplayResponse {
            node_id: req.node_id.clone(),
            command_id: req.command_id.clone(),
            // Database time from inside the transaction, so the response and the
            // refreshed decision report the same instant.
            replayed_at: replay.effected_at.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

/// The `POST /v1/nodes/quarantine` body: an operator quarantines one inbox
/// command WITH a reason — the replay/poll paths skip it from then on (a
/// quarantined command is never re-delivered, `.1.2.3`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuarantineRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub command_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuarantineResponse {
    pub node_id: String,
    pub command_id: String,
    pub quarantined_at: String,
}

/// Quarantine one inbox command, with its reason.
///
/// One transaction under the tenant's shared guard since
/// `SIGNOFF-REPAIR.3.3.4.10.3`. The superseded shape ran a check and a
/// conditional update as two separate POOL statements with no tenant predicate,
/// so it both raced itself and mutated other tenants' rows.
async fn quarantine_command(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<QuarantineRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let quarantine = authority::quarantine_command_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.command_id,
        &req.reason,
    )
    .await?;
    let receipt = quarantine.record_id;
    let response = match quarantine.result {
        authority::QuarantineResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::QuarantineResult::InvalidReason(detail) => {
            ControlApiError::invalid_command(format!(
                "the quarantine reason is required (a quarantine without a reason is a silent \
                 skip): {detail}"
            ))
            .into_response()
        }
        authority::QuarantineResult::NotInInbox => ControlApiError::invalid_command(format!(
            "no command `{}` in node `{}`'s inbox",
            req.command_id, req.node_id
        ))
        .into_response(),
        authority::QuarantineResult::AlreadyQuarantined { at } => {
            ControlApiError::invalid_transition(format!(
                "the command is already quarantined ({at})"
            ))
            .into_response()
        }
        authority::QuarantineResult::Quarantined { at } => Json(QuarantineResponse {
            node_id: req.node_id.clone(),
            command_id: req.command_id.clone(),
            quarantined_at: at.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

/// One inbox row as the inspection surface reports it (`.1.2.3`): the delivery
/// and quarantine facts, WITH the payload (an operator judging a quarantine
/// needs to see what was quarantined).
#[derive(Debug, Clone, Serialize)]
pub struct InboxRow {
    pub cursor: i64,
    pub command_id: String,
    pub thread_id: String,
    pub payload: Value,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub quarantined_at: Option<DateTime<Utc>>,
    pub quarantine_reason: Option<String>,
    /// The derived §10.6 delivery state (`.5.1`): `queued` |
    /// `acknowledged` | `consumed` | `dead_lettered`.
    pub delivery_state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InboxInspection {
    pub node_id: String,
    pub rows: Vec<InboxRow>,
}

#[derive(Debug, Deserialize)]
pub struct InboxInspectionParams {
    pub tenant_id: TenantId,
    pub node_id: String,
}

/// The `GET /v1/nodes/inbox` inspection surface: every row's delivery +
/// quarantine facts, in cursor order. `tenant_admin` authority.
async fn inspect_node_inbox(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Query(params): Query<InboxInspectionParams>,
) -> Result<Json<InboxInspection>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, params.tenant_id).await?;
    #[derive(sqlx::FromRow)]
    struct InboxRowRow {
        cursor: i64,
        command_id: String,
        thread_id: String,
        payload: Value,
        acknowledged_at: Option<DateTime<Utc>>,
        quarantined_at: Option<DateTime<Utc>>,
        quarantine_reason: Option<String>,
        delivery_state: String,
    }
    let rows: Vec<InboxRowRow> = sqlx::query_as(
        "SELECT cursor, command_id, thread_id, payload, acknowledged_at, quarantined_at, quarantine_reason, delivery_state \
         FROM node_inbox_state WHERE node_id = $1 ORDER BY cursor",
    )
    .bind(&params.node_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(InboxInspection {
        node_id: params.node_id,
        rows: rows
            .into_iter()
            .map(|r| InboxRow {
                cursor: r.cursor,
                command_id: r.command_id,
                thread_id: r.thread_id,
                payload: r.payload,
                acknowledged_at: r.acknowledged_at,
                quarantined_at: r.quarantined_at,
                quarantine_reason: r.quarantine_reason,
                delivery_state: r.delivery_state,
            })
            .collect(),
    }))
}

/// The `POST /v1/nodes/inbox/prune` body: the retention window — DELIVERED
/// rows (acknowledged by the node) at least this old are deleted. A
/// QUARANTINED row is never prunable (§16.11, `.1.3.3`): the preservation
/// survives the disposition. Cleanup is an explicit, measured operator
/// action; nothing sweeps on its own.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PruneInboxRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub min_age_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PruneInboxResponse {
    pub deleted: i64,
    pub before: i64,
    pub after: i64,
    pub cutoff_at: String,
}

/// Prune delivered inbox rows older than a window.
///
/// One transaction under the tenant's shared guard since
/// `SIGNOFF-REPAIR.3.3.4.10.3`. The counts were already measured in one
/// transaction; what they were NOT was bound to the caller's tenant, so this
/// verb deleted another tenant's rows and reported them as its own — measured at
/// `{"deleted":2,"before":2,"after":0}` against a foreign inbox.
///
/// The §16.11 preservation rule is unchanged: a quarantined row is never deleted.
async fn prune_node_inbox(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<PruneInboxRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let prune = authority::prune_node_inbox_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        req.min_age_seconds,
    )
    .await?;
    let receipt = prune.record_id;
    let response = match prune.result {
        authority::PruneResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::PruneResult::InvalidWindow(detail) => {
            ControlApiError::invalid_command(detail).into_response()
        }
        // Both outcomes answer with the same measured receipt: an operator asked
        // what was removed, and zero is an answer.
        authority::PruneResult::Pruned {
            deleted,
            before,
            after,
            cutoff,
        } => Json(PruneInboxResponse {
            deleted,
            before,
            after,
            cutoff_at: cutoff.to_rfc3339(),
        })
        .into_response(),
        authority::PruneResult::NothingToPrune {
            before,
            after,
            cutoff,
        } => Json(PruneInboxResponse {
            deleted: 0,
            before,
            after,
            cutoff_at: cutoff.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

// ── Node revocation (`.1.3.1`) ──────────────────────────────────────────────

/// The `POST /v1/nodes/revoke` body: an authorized human revokes the node's
/// ACTIVE workload certificates. The `.1.2.2` handshake ladder refuses a
/// revoked leaf at the next crossing (the row check is already live), and the
/// presence view (0012) reads `suspended`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeNodeRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevokeNodeResponse {
    pub node_id: String,
    pub revoked_certificates: i64,
    pub revoked_at: String,
}

/// Revoke a node's active workload certificates.
///
/// Since `SIGNOFF-REPAIR.3.3.4.10.2` the admission, the tenant-bound node
/// selection, the certificate revocation, the epoch bump and the final effect
/// record share ONE transaction under the tenant's EXCLUSIVE authority guard.
/// Before this the handler admitted in its own transaction, ran a tenant-bound
/// existence probe as a SEPARATE pool query, and then mutated in a third
/// transaction whose UPDATE matched on `node_id` alone — so the check and the
/// act read two different snapshots and neither was ordered against an authority
/// change.
///
/// The mode is exclusive because this bumps the tenant's revocation epoch: it is
/// a revocation in the sense the guard contract means, and the exclusive mode is
/// what fences every admission that has not already selected its evidence.
///
/// Every answer carries the `x-reasonbraid-authorization` receipt.
async fn revoke_node(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<RevokeNodeRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let revocation = authority::revoke_node_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.reason,
    )
    .await?;
    let receipt = revocation.record_id;
    let response = match revocation.result {
        authority::NodeRevokeResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::NodeRevokeResult::InvalidReason(detail) => {
            ControlApiError::invalid_command(format!("the revocation reason is required: {detail}"))
                .into_response()
        }
        // Missing and foreign are ONE answer, so a caller learns nothing about
        // another tenant's nodes (`SIGNOFF-REPAIR.3.1`).
        authority::NodeRevokeResult::NotFound => {
            ControlApiError::not_found(format!("no enrolled node `{}` in this tenant", req.node_id))
                .into_response()
        }
        // One 409 for both idle states, unchanged. Which one it was is in the
        // effect record, not on the wire.
        authority::NodeRevokeResult::AlreadyRevoked
        | authority::NodeRevokeResult::NoCertificate => ControlApiError::invalid_transition(
            format!("node `{}` has no active certificate to revoke", req.node_id),
        )
        .into_response(),
        authority::NodeRevokeResult::Revoked { certificates } => Json(RevokeNodeResponse {
            node_id: req.node_id.clone(),
            revoked_certificates: certificates,
            // Database time from inside the transaction, so the response, the
            // certificate rows and the effect record report the same instant.
            revoked_at: revocation.effected_at.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

// ── The federation trust agreements (PHASE-8.1.2; ADR-026) ─────────────────────

/// The agreement body: one DIRECTION of the named tenant-to-tenant pairing
/// (the EFFECTIVE agreement is the both-sides accepted pair).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FederationAgreementRequest {
    pub tenant_id: TenantId,
    pub remote_tenant_id: TenantId,
    #[serde(default)]
    pub directory_visibility: bool,
    #[serde(default)]
    pub recruitment: bool,
}

/// `POST /v1/federation-agreements` — propose one direction. A proposal
/// widens NOTHING by itself (the pairing needs the remote side's own row).
async fn propose_federation_agreement(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<FederationAgreementRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    let agreement_id = crate::federation::propose(
        &state.pool,
        &req.tenant_id.to_string(),
        &req.remote_tenant_id.to_string(),
        req.directory_visibility,
        req.recruitment,
    )
    .await?;
    Ok(Json(
        json!({ "agreement_id": agreement_id, "status": "proposed" }),
    ))
}

/// The accept/revoke body: the direction this tenant records.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FederationAgreementAction {
    pub tenant_id: TenantId,
    pub remote_tenant_id: TenantId,
}

/// `POST /v1/federation-agreements/accept` — accept the remote side's
/// proposal (this tenant's own row). The effect engages only when BOTH
/// rows are accepted.
async fn accept_federation_agreement(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<FederationAgreementAction>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    let rows = crate::federation::accept(
        &state.pool,
        &req.tenant_id.to_string(),
        &req.remote_tenant_id.to_string(),
    )
    .await?;
    if rows == 0 {
        return Err(ControlApiError::invalid_transition(
            "no PROPOSED agreement in this direction to accept (the remote side must propose first)",
        ));
    }
    Ok(Json(json!({ "status": "accepted" })))
}

/// `POST /v1/federation-agreements/revoke` — revoke this tenant's
/// direction (the fallback: the network pseudonym).
async fn revoke_federation_agreement(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<FederationAgreementAction>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    let rows = crate::federation::revoke(
        &state.pool,
        &req.tenant_id.to_string(),
        &req.remote_tenant_id.to_string(),
    )
    .await?;
    Ok(Json(json!({ "revoked": rows })))
}

// ── The operator's offline-known enumeration (PHASE-3.2.2; backlog 27) ─────────

/// `GET /v1/admin/nodes/presence?tenant_id=…` — the tenant's enrolled nodes
/// with their DERIVED presence state + the lease clock: the operator's
/// "this node is known, just quiet" rows. Uses the own-tenant inspection gate.
async fn list_node_presence(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::NodesPresence {},
        |pool| async move {
            type Row = (
                String,
                bool,
                bool,
                Option<chrono::DateTime<chrono::Utc>>,
                Option<chrono::DateTime<chrono::Utc>>,
                Option<i64>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT np.node_id, np.online, np.suspended, np.last_seen_at, np.lease_expires_at, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) \
         FROM node_presence np WHERE np.tenant_id = $1 ORDER BY np.node_id",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&pool)
    .await?;
            let nodes: Vec<Value> = rows
        .into_iter()
        .map(
            |(node_id, online, suspended, last_seen_at, lease_expires_at, concurrency)| {
                json!({
                    "node_id": node_id,
                    "state": crate::presence::presence_state(true, suspended, online, concurrency)
                        .as_str(),
                    "online": online,
                    "suspended": suspended,
                    "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                    "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
                })
            },
        )
        .collect();
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "nodes": nodes,
            })))
        },
    )
    .await
}

// ── The resolver capability registry (PHASE-4.1.3; backlog 31) ──────────────────────

/// `POST /v1/resolvers` — the operator registers (or replaces) one resolver's
/// §12.2 advertise (the tenant_admin gate; the future packs call it at their
/// install).
async fn register_resolver(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(advertise): Json<crate::resolvers::ResolverAdvertise>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no resolver",
        ));
    };
    authorize_tenant_admin(
        &state.pool,
        &principal,
        tenant.parse().map_err(|_| ControlApiError::internal())?,
    )
    .await?;
    if let Some(error) = advertise.isolation_error() {
        return Err(ControlApiError::invalid_command(error));
    }
    crate::resolvers::register(&state.pool, &advertise).await?;
    Ok(Json(json!({
        "resolver_id": advertise.resolver_id,
        "registered": true,
    })))
}

/// The resolution request: the caller's required ADR-018 classes.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolveRequest {
    #[serde(default = "default_sandbox")]
    required_sandbox: String,
    #[serde(default = "default_egress")]
    required_egress: String,
}

fn default_sandbox() -> String {
    "process".to_string()
}
fn default_egress() -> String {
    "loopback".to_string()
}

/// `POST /v1/resources/{id}/resolve` — the §12.2 resolution order: the
/// scheme + the ADR-018 isolation filters FIRST, then the latency rank. No
/// eligible resolver → the explicit `resource_unresolvable_now` — the
/// reference is PRESERVED (still submitted, never fabricated).
async fn resolve_resource(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(resource_id): Path<String>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<crate::resolvers::ResolutionOutcome>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal resolves no reference",
        ));
    }
    let Some((_, reference, _, _)) = crate::resources::get(&state.pool, &resource_id).await? else {
        return Err(ControlApiError::not_found(format!(
            "no reference `{resource_id}`"
        )));
    };
    let mut outcome = crate::resolvers::resolve(
        &state.pool,
        &reference.scheme,
        reference.media_type_hint.as_deref(),
        reference.credential_binding_ref.as_deref(),
        &req.required_sandbox,
        &req.required_egress,
    )
    .await?;
    // The built-in packs execute when they rank first: the acquisition
    // runs under the pack's own ceilings + the `.2.1` policy; a refusal is
    // the NAMED error, and the reference stays submitted either way.
    match outcome.resolvers.first().map(String::as_str) {
        Some(crate::resolvers::R0_RESOLVER_ID) => {
            match state.fetcher.fetch(&reference.original_locator).await {
                Ok(document) => {
                    let receipt = crate::fetcher::AcquisitionReceipt::from_document(
                        &reference.original_locator,
                        &document,
                        chrono::Utc::now(),
                    );
                    // The snapshot store (`.6.1`): the acquired bytes land
                    // under their digest — a persistence failure leaves the
                    // receipt returned (the acquisition succeeded).
                    let _ = crate::snapshots::submit(
                        &state.pool,
                        &crate::snapshots::SnapshotSubmission {
                            reference_id: resource_id.clone(),
                            original_locator: reference.original_locator.clone(),
                            final_locator: receipt.final_url.clone(),
                            resolver_id: crate::resolvers::R0_RESOLVER_ID.to_owned(),
                            resolver_version: "0.1.0".to_owned(),
                            network_class: "public".to_owned(),
                            auth_class: "none".to_owned(),
                            provider_receipt: serde_json::json!({ "chain": receipt.chain }),
                            immutable_source_version: None,
                            raw_digest: receipt.digest.clone(),
                            byte_length: receipt.byte_count as i64,
                            media_type: receipt
                                .content_type
                                .clone()
                                .unwrap_or_else(|| receipt.sniffed.to_string()),
                            storage_class: "standard".to_owned(),
                            retention_class: "standard".to_owned(),
                            extraction_version: None,
                            quarantine_status: "none".to_owned(),
                            redactions: serde_json::json!([]),
                            disclosure_policy: serde_json::json!({}),
                            license: None,
                            fresh_until: None,
                        },
                        &document.bytes,
                        chrono::Utc::now(),
                    )
                    .await;
                    outcome.acquisition = Some(crate::resolvers::Acquisition::Web(receipt));
                }
                Err(error) => {
                    outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                        kind: error.kind().to_owned(),
                        message: error.to_string(),
                    });
                }
            }
        }
        Some(crate::resolvers::R5_RESOLVER_ID) if state.r5r3rx_enabled => {
            // The R5 pack: the broker resolves the binding at the request
            // boundary, the credential attaches for THIS acquisition only
            // (never ambient), and the disclosure rides the receipt.
            let binding = reference.credential_binding_ref.clone().unwrap_or_default();
            match state.broker.resolve(&binding) {
                Err(error) => {
                    outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                        kind: "credential_unavailable".to_owned(),
                        message: error.to_string(),
                    });
                }
                Ok(credential) => {
                    match state
                        .fetcher
                        .fetch_authenticated(
                            &reference.original_locator,
                            &credential.authorization_header().1,
                        )
                        .await
                    {
                        Ok(document) => {
                            let disclosure = state.broker.disclose(
                                &binding,
                                document.final_url.host_str().unwrap_or("unknown"),
                                chrono::Utc::now(),
                            );
                            match disclosure {
                                Ok(disclosure) => {
                                    let receipt = crate::fetcher::AcquisitionReceipt::from_document(
                                        &reference.original_locator,
                                        &document,
                                        chrono::Utc::now(),
                                    );
                                    let _ = crate::snapshots::submit(
                                        &state.pool,
                                        &crate::snapshots::SnapshotSubmission {
                                            reference_id: resource_id.clone(),
                                            original_locator: reference.original_locator.clone(),
                                            final_locator: receipt.final_url.clone(),
                                            resolver_id: crate::resolvers::R5_RESOLVER_ID.to_owned(),
                                            resolver_version: "0.1.0".to_owned(),
                                            network_class: "public".to_owned(),
                                            auth_class: disclosure.credential_class.clone(),
                                            provider_receipt: serde_json::json!({ "chain": receipt.chain }),
                                            immutable_source_version: None,
                                            raw_digest: receipt.digest.clone(),
                                            byte_length: receipt.byte_count as i64,
                                            media_type: receipt
                                                .content_type
                                                .clone()
                                                .unwrap_or_else(|| receipt.sniffed.to_string()),
                                            storage_class: "standard".to_owned(),
                                            retention_class: "standard".to_owned(),
                                            extraction_version: None,
                                            quarantine_status: "none".to_owned(),
                                            redactions: serde_json::json!([]),
                                            disclosure_policy: serde_json::json!({ "credential_class": disclosure.credential_class, "host": disclosure.host }),
                                            license: None,
                                            fresh_until: None,
                                        },
                                        &document.bytes,
                                        chrono::Utc::now(),
                                    )
                                    .await;
                                    outcome.acquisition =
                                        Some(crate::resolvers::Acquisition::Authenticated(
                                            Box::new(crate::broker::AuthenticatedReceipt {
                                                disclosure,
                                                receipt,
                                            }),
                                        ));
                                }
                                Err(error) => {
                                    outcome.acquisition_error =
                                        Some(crate::resolvers::AcquisitionError {
                                            kind: "credential_unavailable".to_owned(),
                                            message: error.to_string(),
                                        });
                                }
                            }
                        }
                        Err(error) => {
                            outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                                kind: error.kind().to_owned(),
                                message: error.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Some(crate::resolvers::R3_RESOLVER_ID) if state.r5r3rx_enabled => {
            // The R3 pack: the pre-flight classifies the URL BEFORE the
            // worker spawns (the worker receives an already-classified
            // URL); the render receipt carries the network log.
            match state.fetcher.preflight(&reference.original_locator).await {
                Err(error) => {
                    outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                        kind: error.kind().to_owned(),
                        message: error.to_string(),
                    });
                }
                Ok(_) => match crate::browse::run_browse(
                    &reference.original_locator,
                    std::time::Duration::from_secs(120),
                ) {
                    Ok(response) => {
                        outcome.acquisition = Some(crate::resolvers::Acquisition::Browse(
                            crate::browse::BrowserReceipt {
                                parent_digest: response.parent_digest,
                                chunks: response.chunks,
                                network_log: response.network_log,
                                page_title: response.page_title,
                                browser_version: response.browser_version,
                                requested_url: reference.original_locator.clone(),
                                acquired_at: chrono::Utc::now(),
                            },
                        ));
                    }
                    Err(error) => {
                        let kind = match &error {
                            crate::browse::BrowseError::WorkerMissing(_) => {
                                "browser_worker_missing"
                            }
                            crate::browse::BrowseError::SpawnFailed(_) => "browser_spawn_failed",
                            crate::browse::BrowseError::RequestFailed(_) => "render_failed",
                            crate::browse::BrowseError::TimedOut => "render_timed_out",
                            crate::browse::BrowseError::WorkerRefused { kind, .. } => kind,
                        };
                        outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                            kind: kind.to_owned(),
                            message: error.to_string(),
                        });
                    }
                },
            }
        }
        Some(crate::resolvers::RX_RESOLVER_ID) if state.r5r3rx_enabled => {
            // The RX pack: the capability-call publication (the §12.2/
            // §12.8 shape) — the enrolled agents answer with the typed
            // vocabulary; the delivery rides the capability-call lane.
            outcome.acquisition_call = Some(crate::mediated::AcquisitionCall {
                call_id: format!("cal_{resource_id}"),
                locator: reference.original_locator.clone(),
                requires_second_verifier: false,
            });
        }
        Some(crate::resolvers::R2_RESOLVER_ID) => {
            // The R2 pipeline: the R0 fetcher acquires the bytes (under the
            // `.2.1` policy — the refusal names the class), then the worker
            // derives the chunks (the killing budget is the quarantine's
            // enforcement). The reference stays submitted either way.
            let hint = reference.media_type_hint.clone().unwrap_or_default();
            // The pack that was RANKED advertises the formats it exists to
            // handle, and the acquisition leg admits exactly those in addition
            // to R0's own accept set — read from the registry row per
            // resolution, never compiled in
            // (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`). A
            // read failure yields the shipped accept set, not a wider one.
            let admitted = crate::resolvers::advertised_media_types(
                &state.pool,
                crate::resolvers::R2_RESOLVER_ID,
            )
            .await
            .unwrap_or_default();
            match state
                .fetcher
                .fetch_admitting(&reference.original_locator, &admitted)
                .await
            {
                Err(error) => {
                    outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                        kind: error.kind().to_owned(),
                        message: error.to_string(),
                    });
                }
                Ok(document) => {
                    // The acquired bytes become an input this process OWNS: a
                    // private file on the repository volume, released only once
                    // no worker can still be reading it, and a response bound to
                    // the exact bytes supplied (`.7.3.3.3`).
                    let extraction = crate::extraction_input::extract_acquired_bytes(
                        &document.bytes,
                        &hint,
                        crate::extraction::WorkerLimits::default(),
                        std::time::Duration::from_secs(60),
                    );
                    match extraction {
                        Ok(response) => {
                            let snapshot = crate::snapshots::submit(
                                &state.pool,
                                &crate::snapshots::SnapshotSubmission {
                                    reference_id: resource_id.clone(),
                                    original_locator: reference.original_locator.clone(),
                                    final_locator: document.final_url.to_string(),
                                    resolver_id: crate::resolvers::R2_RESOLVER_ID.to_owned(),
                                    resolver_version: "0.1.0".to_owned(),
                                    network_class: "public".to_owned(),
                                    auth_class: "none".to_owned(),
                                    provider_receipt: serde_json::json!({ "chain": document.chain.iter().map(|u| u.to_string()).collect::<Vec<_>>() }),
                                    immutable_source_version: None,
                                    raw_digest: crate::fetcher::digest_sha256_hex(&document.bytes),
                                    byte_length: document.bytes.len() as i64,
                                    media_type: document
                                        .content_type
                                        .clone()
                                        .unwrap_or_else(|| document.sniffed.to_string()),
                                    storage_class: "standard".to_owned(),
                                    retention_class: "standard".to_owned(),
                                    extraction_version: Some(response.extractor_version.clone()),
                                    quarantine_status: "none".to_owned(),
                                    redactions: serde_json::json!([]),
                                    disclosure_policy: serde_json::json!({}),
                                    license: None,
                                    fresh_until: None,
                                },
                                &document.bytes,
                                chrono::Utc::now(),
                            )
                            .await;
                            // A failed snapshot is NOT a successful acquisition.
                            // This path used to discard the error and still set
                            // `outcome.acquisition`, so a caller was told the
                            // document had been acquired while no evidence row
                            // and no derivation existed (`.7.4.2`).
                            let Ok(snapshot) = snapshot.inspect_err(|error| {
                                crate::log_event!(
                                    "acquisition_evidence_unstored",
                                    "resource_id" => resource_id.clone(),
                                    "reason" => error.to_string(),
                                );
                                outcome.acquisition_error =
                                    Some(crate::resolvers::AcquisitionError {
                                        kind: "evidence_unstored".to_owned(),
                                        message: error.to_string(),
                                    });
                            }) else {
                                // No snapshot, so no derivations and no
                                // acquisition receipt: the refusal above is the
                                // whole outcome for this reference.
                                return Ok(Json(outcome));
                            };
                            // The derivation graph: every derived chunk is a
                            // Derivation edge — the parent stays addressable
                            // (a chunk is NEVER the original).
                            {
                                for chunk in &response.chunks {
                                    let _ = crate::derivations::submit(
                                        &state.pool,
                                        &crate::derivations::DerivationSubmission {
                                            parent_snapshot_id: snapshot.snapshot_id.clone(),
                                            derived_kind: "chunk".to_owned(),
                                            derived_digest: chunk.digest.clone(),
                                            content: chunk.text.clone(),
                                            extraction_version: Some(
                                                response.extractor_version.clone(),
                                            ),
                                            source_selector: None,
                                        },
                                    )
                                    .await;
                                }
                            }
                            outcome.acquisition = Some(crate::resolvers::Acquisition::Extract(
                                crate::extraction::ExtractionReceipt {
                                    parent_digest: response.parent_digest,
                                    chunks: response.chunks,
                                    excluded: response.excluded,
                                    extractor_version: response.extractor_version,
                                    requested_url: reference.original_locator.clone(),
                                    acquired_at: chrono::Utc::now(),
                                },
                            ));
                        }
                        Err(error) => {
                            let kind = match &error {
                                crate::extraction::ExtractionError::WorkerMissing(_) => {
                                    "worker_missing"
                                }
                                crate::extraction::ExtractionError::SpawnFailed(_) => {
                                    "worker_spawn_failed"
                                }
                                crate::extraction::ExtractionError::RequestFailed(_) => {
                                    "extraction_failed"
                                }
                                crate::extraction::ExtractionError::TimedOut => {
                                    "extraction_timed_out"
                                }
                                crate::extraction::ExtractionError::WorkerRefused {
                                    kind, ..
                                } => kind,
                                // A response for other bytes never becomes a
                                // snapshot, a derivation or a receipt.
                                crate::extraction::ExtractionError::SourceMismatch { .. } => {
                                    "extraction_source_mismatch"
                                }
                            };
                            outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                                kind: kind.to_owned(),
                                message: error.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Some(crate::resolvers::R1_RESOLVER_ID) => {
            match state.git_fetcher.acquire(&reference.original_locator).await {
                Ok(acquisition) => {
                    let requested_ref = url::Url::parse(&reference.original_locator)
                        .ok()
                        .and_then(|u| u.fragment().map(str::to_owned))
                        .unwrap_or_else(|| "HEAD".to_owned());
                    outcome.acquisition = Some(crate::resolvers::Acquisition::Git(
                        crate::git::GitReceipt::from_acquisition(
                            &reference.original_locator,
                            &requested_ref,
                            &acquisition,
                            chrono::Utc::now(),
                        ),
                    ));
                }
                Err(error) => {
                    outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                        kind: match error {
                            crate::git::GitError::DestinationRefused { .. } => {
                                "destination_refused".to_owned()
                            }
                            crate::git::GitError::SchemeNotAllowed(_) => {
                                "scheme_not_allowed".to_owned()
                            }
                            crate::git::GitError::UserinfoForbidden => {
                                "userinfo_forbidden".to_owned()
                            }
                            crate::git::GitError::AmbiguousNumericHost(_) => {
                                "ambiguous_numeric_host".to_owned()
                            }
                            crate::git::GitError::PortNotAllowed(_) => {
                                "port_not_allowed".to_owned()
                            }
                            crate::git::GitError::NoHost => "no_host".to_owned(),
                            crate::git::GitError::DnsLookupFailed => "dns_lookup_failed".to_owned(),
                            crate::git::GitError::RefSelectorInvalid(_) => {
                                "ref_selector_invalid".to_owned()
                            }
                            crate::git::GitError::HttpStatus(_) => "http_status".to_owned(),
                            crate::git::GitError::DepthCeilingExceeded { .. } => {
                                "depth_ceiling_exceeded".to_owned()
                            }
                            crate::git::GitError::BudgetExceeded { .. } => {
                                "budget_exceeded".to_owned()
                            }
                            crate::git::GitError::Refused { .. } => "refused".to_owned(),
                            crate::git::GitError::NoHeadRef => "no_head_ref".to_owned(),
                            crate::git::GitError::ResolvedCommitMissing => {
                                "resolved_commit_missing".to_owned()
                            }
                            crate::git::GitError::TimedOut => "timed_out".to_owned(),
                            crate::git::GitError::UrlTooLong(_)
                            | crate::git::GitError::UrlHasControlCharacters
                            | crate::git::GitError::UrlUnparseable
                            | crate::git::GitError::TransferFailed(_) => {
                                "acquisition_failed".to_owned()
                            }
                        },
                        message: error.to_string(),
                    });
                }
            }
        }
        _ => {}
    }
    Ok(Json(outcome))
}

// ── The workflow-profile registry (PHASE-5.1.2; backlog 36) ─────────────────────────

/// `POST /v1/workflow-profiles` — register a custom profile (the
/// operator's verb): the steps MUST pass the composition validation.
async fn register_workflow_profile(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::workflows::ResolvedProfile>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no profile",
        ));
    }
    let profile_id = body
        .get("profile_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the profile_id is required"))?;
    let steps: Vec<String> = body
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ControlApiError::invalid_command("the steps are required"))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    match crate::workflows::register(&state.pool, profile_id, &steps).await {
        Ok(resolved) => Ok(Json(resolved)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/workflow-profiles` — the registry's latest versions.
async fn list_workflow_profiles(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::workflows::ResolvedProfile>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no profiles",
        ));
    }
    Ok(Json(crate::workflows::list(&state.pool).await?))
}

// ── The evaluation service (PHASE-5.4.2; ADR-017, backlog 37) ─────────────────────

/// `POST /v1/evaluations/corpora` — register one corpus version (the
/// content-addressed registry row; the digests are the declared 64-hex
/// file digests — the harness re-derives them at run time).
async fn register_evaluation_corpus(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(registration): Json<crate::evaluation::CorpusRegistration>,
) -> Result<Json<crate::evaluation::RegisteredCorpus>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no corpus",
        ));
    }
    match crate::evaluation::register_corpus(&state.pool, &registration).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/corpora` — the registry rows.
async fn list_evaluation_corpora(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::evaluation::RegisteredCorpus>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no corpora",
        ));
    }
    Ok(Json(crate::evaluation::list_corpora(&state.pool).await?))
}

/// `POST /v1/evaluations/runs` — record one experiment run (the seed
/// declares the randomness; a non-deterministic run without one refuses).
async fn record_evaluation_run(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(run): Json<crate::evaluation::RunRecord>,
) -> Result<Json<crate::evaluation::StoredRun>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no run",
        ));
    }
    match crate::evaluation::record_run(&state.pool, &run).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/runs` — the recorded runs, newest first.
async fn list_evaluation_runs(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::evaluation::StoredRun>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no runs",
        ));
    }
    Ok(Json(crate::evaluation::list_runs(&state.pool).await?))
}

/// `POST /v1/evaluations/trials` — create the SHADOW routing trial (`.4.3`):
/// the server computes the seeded assignment (the record alone reproduces
/// the draw); the trial never changes production routing.
async fn create_evaluation_trial(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::evaluation::TrialSubmission>,
) -> Result<Json<crate::evaluation::StoredTrial>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal creates no trial",
        ));
    }
    match crate::evaluation::create_trial(&state.pool, &submission).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/trials` — the trials, newest first.
async fn list_evaluation_trials(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::evaluation::StoredTrial>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no trials",
        ));
    }
    Ok(Json(crate::evaluation::list_trials(&state.pool).await?))
}

/// `POST /v1/evaluations/trials/{id}/results` — append one per-arm results
/// row (append-only — the record's identity is its content).
async fn record_trial_results(
    State(state): State<Arc<ApiState>>,
    Path(trial_id): Path<String>,
    headers: HeaderMap,
    Json(results): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no results",
        ));
    }
    match crate::evaluation::record_trial_results(&state.pool, &trial_id, &results).await {
        Ok(()) => Ok(Json(json!({ "trial_id": trial_id, "appended": true }))),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/trials/{id}/results` — the recorded per-arm results.
async fn list_trial_results(
    State(state): State<Arc<ApiState>>,
    Path(trial_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no results",
        ));
    }
    Ok(Json(
        crate::evaluation::list_trial_results(&state.pool, &trial_id).await?,
    ))
}

/// `POST /v1/evaluations/calibrations` — record one calibration (`.4.4`):
/// the accumulation over the NAMED runs (each must be registered).
async fn record_evaluation_calibration(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::evaluation::CalibrationSubmission>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no calibration",
        ));
    }
    match crate::evaluation::record_calibration(&state.pool, &submission).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/calibrations` — the calibration rows, newest first.
async fn list_evaluation_calibrations(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no calibrations",
        ));
    }
    Ok(Json(
        crate::evaluation::list_calibrations(&state.pool).await?,
    ))
}

/// `POST /v1/evaluations/gates` — record the gate (the baseline + the
/// threshold). The gate only BLOCKS.
async fn record_evaluation_gate(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::evaluation::GateSubmission>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no gate",
        ));
    }
    match crate::evaluation::record_gate(&state.pool, &submission).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/gates` — the gate rows, newest first.
async fn list_evaluation_gates(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no gates",
        ));
    }
    Ok(Json(crate::evaluation::list_gates(&state.pool).await?))
}

/// `POST /v1/evaluations/gates/{id}/evaluations` — evaluate the gate (`.4.4`):
/// the measured scores against the baseline minus the threshold; the result
/// APPENDS (the gate never rewrites a result).
async fn evaluate_gate_endpoint(
    State(state): State<Arc<ApiState>>,
    Path(gate_id): Path<String>,
    headers: HeaderMap,
    Json(scores): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal evaluates no gate",
        ));
    }
    match crate::evaluation::evaluate_gate(&state.pool, &gate_id, &scores).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/evaluations/gates/{id}/evaluations` — the gate's results.
async fn list_gate_results(
    State(state): State<Arc<ApiState>>,
    Path(gate_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no gate results",
        ));
    }
    Ok(Json(
        crate::evaluation::list_gate_results(&state.pool, &gate_id).await?,
    ))
}

// ── The routing policy (PHASE-5.5.2; ADR-031) ───────────────────────────────────────

/// `GET /v1/routing/rules` — the deterministic rule table.
async fn list_routing_rules(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no routing rules",
        ));
    }
    Ok(Json(crate::routing::list_rules(&state.pool).await?))
}

/// `POST /v1/routing/resolve` — the deterministic lookup: the submitted
/// class → the arm + the rule id (the resolution rides the audit table).
async fn resolve_routing_class(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::routing::ResolvedRoute>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal resolves no route",
        ));
    }
    let case_class = body
        .get("case_class")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the case_class is required"))?;
    match crate::routing::resolve(&state.pool, case_class).await {
        Ok(route) => {
            crate::routing::record_resolution(
                &state.pool,
                &route,
                &principal.id_string(),
                "resolve_verb",
            )
            .await?;
            Ok(Json(route))
        }
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/routing/resolutions` — the audit rows, newest first.
async fn list_routing_resolutions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no resolutions",
        ));
    }
    Ok(Json(crate::routing::list_resolutions(&state.pool).await?))
}

/// `POST /v1/routing/recommendations` — record the shadow recommendation
/// (`.5.3`): the arm must be an EXISTING registered profile (never a raise);
/// the record is NEVER applied.
async fn record_routing_recommendation(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::routing::RecommendationSubmission>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no recommendation",
        ));
    }
    match crate::routing::record_recommendation(&state.pool, &submission).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/routing/recommendations` — the shadow records, newest first.
async fn list_routing_recommendations(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no recommendations",
        ));
    }
    Ok(Json(
        crate::routing::list_recommendations(&state.pool).await?,
    ))
}

// ── The policy registry (PHASE-6.1.2; ADR-019, backlog 38) ─────────────────────────

/// `POST /v1/policies` — register one policy version (the typed,
/// validated, digest-pinned document; the owning authority must be an
/// ACTIVE grant — the label grants nothing).
async fn register_policy(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::policy::PolicyVersionInput>,
) -> Result<Json<crate::policy::RegisteredPolicy>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no policy",
        ));
    }
    match crate::policy::register(&state.pool, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policies` — the registered policy versions, newest first.
async fn list_policies(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::policy::RegisteredPolicy>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no policies",
        ));
    }
    Ok(Json(crate::policy::list(&state.pool).await?))
}

/// `POST /v1/policies/resolve` — the seven-step layered resolution (`.1.3`):
/// the issuer authority, the applicability, the dependencies/conflicts, the
/// precedence, the exceptions, the FAIL-CLOSED binding conflict, and the
/// explanation tree.
async fn resolve_policies(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<crate::policy::ResolutionRequest>,
) -> Result<Json<crate::policy::Resolution>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal resolves no policy set",
        ));
    }
    match crate::policy::resolve(&state.pool, &request).await {
        Ok(resolution) => Ok(Json(resolution)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policies/{id}/{version}/impact` — the impact map (`.1.3`): the
/// derivable coverage — the clauses × the declared applicability (never an
/// achievement claim).
async fn policy_impact(
    State(state): State<Arc<ApiState>>,
    Path((policy_id, version)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no impact map",
        ));
    }
    match crate::policy::impact(&state.pool, &policy_id, &version).await {
        Ok(map) => Ok(Json(map)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

// ── The policy lifecycle (PHASE-6.2.2; ADR-032) ─────────────────────────────────────

/// `POST /v1/policy-proposals` — register one proposal (the draft stage;
/// the reference to the policy version + the deliberation thread).
async fn register_policy_proposal(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::ProposalInput>,
) -> Result<Json<crate::lifecycle::StoredProposal>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant_id) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no proposal",
        ));
    };
    match crate::lifecycle::register_proposal(&state.pool, &tenant_id, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-proposals` — the proposals, newest first.
async fn list_policy_proposals(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredProposal>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no proposals",
        ));
    }
    Ok(Json(crate::lifecycle::list_proposals(&state.pool).await?))
}

/// `POST /v1/policy-decisions` — record one decision (the draft → decided
/// transition; the frozen electorate snapshot + the verdict reference).
async fn record_policy_decision(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::DecisionInput>,
) -> Result<Json<crate::lifecycle::StoredDecision>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant_id) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no decision",
        ));
    };
    match crate::lifecycle::record_decision(&state.pool, &tenant_id, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-decisions` — the decisions, newest first.
async fn list_policy_decisions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredDecision>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no decisions",
        ));
    }
    Ok(Json(crate::lifecycle::list_decisions(&state.pool).await?))
}

/// `POST /v1/policy-approvals` — record one approval (`.2.3`): the decided →
/// approved transition with the AUTHORITY PROOF (the grant re-check at the
/// approval boundary).
async fn record_policy_approval(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::ApprovalInput>,
) -> Result<Json<crate::lifecycle::StoredApproval>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no approval",
        ));
    }
    match crate::lifecycle::record_approval(&state.pool, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-approvals` — the approvals, newest first.
async fn list_policy_approvals(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredApproval>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no approvals",
        ));
    }
    Ok(Json(crate::lifecycle::list_approvals(&state.pool).await?))
}

/// `POST /v1/policy-projections` — project one resolved set (`.3.2`): the
/// server resolves, the hermetic compiler renders, the artifact records
/// with its digest + its declared unrepresentables.
async fn project_policies(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<crate::projections::ProjectionRequest>,
) -> Result<Json<crate::projections::StoredProjection>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal projects no policy",
        ));
    }
    match crate::projections::project(&state.pool, &request).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-projections` — the projection records, newest first.
async fn list_policy_projections(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::projections::StoredProjection>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no projections",
        ));
    }
    Ok(Json(crate::projections::list(&state.pool).await?))
}

/// `POST /v1/policy-publications` — stage one publication (`.4.2`): the
/// §15.7 steps 1–4's record half (the references verified, the manifest
/// digest, the staged state).
async fn stage_publication(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::publications::PublicationInput>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal stages no publication",
        ));
    }
    match crate::publications::stage(&state.pool, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-publications` — the publications, newest first.
async fn list_publications(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::publications::StoredPublication>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no publications",
        ));
    }
    Ok(Json(crate::publications::list(&state.pool).await?))
}

/// `POST /v1/policy-publications/{id}/effective` — the staged → effective
/// transition with the Git object ids (the §15.7 step 8's record half).
async fn mark_publication_effective(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing effective",
        ));
    }
    let git_object_ids: Vec<String> = body
        .get("git_object_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ControlApiError::invalid_command("the git_object_ids are required"))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    match crate::publications::mark_effective(&state.pool, &publication_id, git_object_ids).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `POST /v1/policy-publications/{id}/failed` — the typed failure (never a
/// skip) with the reason.
async fn mark_publication_failed(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing failed",
        ));
    }
    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the reason is required"))?;
    match crate::publications::mark_failed(&state.pool, &publication_id, reason).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `POST /v1/policy-publications/{id}/publish` — the Git publication half
/// (`.4.3.2`): the staged publication's bundle + the composed manifest
/// ride the publisher (the staging branch, the fetch-back verification,
/// the immutable ref, the effective channel via the CAS), then the record
/// marks effective. The repo path is the DECLARED store (the dev-trusted
/// operator surface — the `.5` deployment lane tightens it).
async fn publish_publication(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal publishes nothing",
        ));
    }
    let repo_path = body
        .get("repo_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the repo_path is required"))?;
    let expected_effective = body
        .get("expected_effective")
        .and_then(|v| v.as_str())
        .map(|hex| {
            hex.parse::<gix::ObjectId>().map_err(|_| {
                ControlApiError::invalid_command(format!(
                    "the expected_effective `{hex}` is malformed"
                ))
            })
        })
        .transpose()?;
    let publication = crate::publications::load(&state.pool, &publication_id)
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    if publication.state != "staged" {
        return Err(ControlApiError::invalid_command(format!(
            "publication `{publication_id}` is at stage `{}` — the publish rides a staged publication",
            publication.state
        )));
    }
    let projection = crate::projections::load(&state.pool, &publication.projection_id)
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let manifest = serde_json::to_string(&serde_json::json!({
        "publication_id": publication.publication_id,
        "proposal_id": publication.proposal_id,
        "decision_id": publication.decision_id,
        "approval_id": publication.approval_id,
        "projection_id": publication.projection_id,
        "projection_digest": projection.digest,
    }))
    .expect("the manifest serializes");
    let refs = crate::publisher::publish(
        std::path::Path::new(repo_path),
        &publication_id,
        &manifest,
        &projection.bytes,
        expected_effective,
    )
    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let git_object_ids = vec![refs.publication_ref_id, refs.effective_ref_id];
    let row = crate::publications::mark_effective(&state.pool, &publication_id, git_object_ids)
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    Ok(Json(row))
}

/// `POST /v1/deployment-targets` — register one target (`.5.2`): the id +
/// the type + the OWNING AUTHORITY (the grant check).
async fn register_deployment_target(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::deployments::TargetInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal registers no target",
        ));
    }
    match crate::deployments::register_target(&state.pool, &input).await {
        Ok(()) => Ok(Json(json!({ "target_id": input.target_id }))),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/deployment-targets` — the targets.
async fn list_deployment_targets(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no targets",
        ));
    }
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT target_id, target_type, owning_authority FROM deployment_targets ORDER BY target_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|(target_id, target_type, owning_authority)| {
                json!({
                    "target_id": target_id,
                    "target_type": target_type,
                    "owning_authority": owning_authority,
                })
            })
            .collect(),
    ))
}

/// `POST /v1/deployments` — assign one publication to one target (`.5.2`):
/// the canary wave + the DESIRED pair.
async fn assign_deployment(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::deployments::AssignmentInput>,
) -> Result<Json<crate::deployments::StoredAssignment>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal assigns nothing",
        ));
    }
    match crate::deployments::assign(&state.pool, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/deployments` — the assignments with the desired/observed pair.
async fn list_deployments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::deployments::StoredAssignment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no deployments",
        ));
    }
    Ok(Json(
        crate::deployments::list_assignments(&state.pool).await?,
    ))
}

/// `POST /v1/deployments/{target}/{publication}/receipt` — the RECEIPT
/// (`.5.2`): the OBSERVED digest + the state (the attestation).
async fn record_deployment_receipt(
    State(state): State<Arc<ApiState>>,
    Path((target_id, publication_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<crate::deployments::ReceiptInput>,
) -> Result<Json<crate::deployments::StoredAssignment>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no receipt",
        ));
    }
    match crate::deployments::record_receipt(&state.pool, &target_id, &publication_id, &input).await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `POST /v1/policy-drift` — record one drift observation (`.5.3`): the
/// categorized desired/observed pair.
async fn record_policy_drift(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::corrections::DriftInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no drift",
        ));
    }
    match crate::corrections::record_drift(&state.pool, &input).await {
        Ok(()) => Ok(Json(json!({ "drift_id": input.drift_id }))),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-drift` — the drift records, newest first.
async fn list_policy_drift(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no drift",
        ));
    }
    Ok(Json(crate::corrections::list_drift(&state.pool).await?))
}

/// `POST /v1/policy-corrections` — record one correction (`.5.3`): the
/// §4.7 operation + the authority proof (the retraction never deletes).
async fn record_policy_correction(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::corrections::CorrectionInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no correction",
        ));
    }
    match crate::corrections::record_correction(&state.pool, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-corrections` — the corrections, newest first.
async fn list_policy_corrections(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no corrections",
        ));
    }
    Ok(Json(
        crate::corrections::list_corrections(&state.pool).await?,
    ))
}

/// `POST /v1/policy-outcomes` — record one outcome (`.5.3`): the §15.11
/// link with the kind + the review trigger.
async fn record_policy_outcome(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::corrections::OutcomeInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no outcome",
        ));
    }
    match crate::corrections::record_outcome(&state.pool, &input).await {
        Ok(()) => Ok(Json(json!({ "outcome_id": input.outcome_id }))),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-outcomes` — the outcomes, newest first.
async fn list_policy_outcomes(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no outcomes",
        ));
    }
    Ok(Json(crate::corrections::list_outcomes(&state.pool).await?))
}

/// `POST /v1/policy-reviews/schedule` — evaluate the DUE reviews (`.6`):
/// the outcomes' named triggers + the drift + the repeated waivers, one
/// due review per (publication, trigger).
async fn schedule_policy_reviews(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::reviews::StoredReview>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal schedules no reviews",
        ));
    }
    Ok(Json(crate::reviews::schedule_reviews(&state.pool).await?))
}

/// `GET /v1/policy-reviews` — the reviews, newest first.
async fn list_policy_reviews(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::reviews::StoredReview>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no reviews",
        ));
    }
    Ok(Json(crate::reviews::list_reviews(&state.pool).await?))
}

/// `POST /v1/policy-reviews/{id}/done` — the due → done transition.
async fn mark_policy_review_done(
    State(state): State<Arc<ApiState>>,
    Path(review_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<crate::reviews::StoredReview>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing done",
        ));
    }
    match crate::reviews::mark_done(&state.pool, &review_id).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

// ── The claim-evidence graph (PHASE-4.6.3; backlog 35) ──────────────────────────────

/// `POST /v1/assessments` — submit the assessment (any enrolled principal;
/// the citation is VALIDATED: the excerpt must appear in the snapshot's
/// raw bytes — citation existence alone never satisfies an evidence gate).
async fn submit_assessment(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::claims::AssessmentSubmission>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no assessment",
        ));
    }
    match crate::claims::submit(&state.pool, &submission).await {
        Ok(assessment_id) => Ok(Json(json!({ "assessment_id": assessment_id }))),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::claims::AssessmentError::Storage(cause)) => Err(
            ControlApiError::internal_with_log(format!("claim assessment storage failed: {cause}")),
        ),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}/assessments` — the snapshot's assessments.
async fn list_snapshot_assessments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<Vec<crate::claims::StoredAssessment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no assessments",
        ));
    }
    Ok(Json(
        crate::claims::assessments_for_snapshot(&state.pool, &snapshot_id).await?,
    ))
}

/// `GET /v1/claims/{claim_id}/assessments` — the claim's assessments.
async fn list_claim_assessments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(claim_id): Path<String>,
) -> Result<Json<Vec<crate::claims::StoredAssessment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no assessments",
        ));
    }
    Ok(Json(
        crate::claims::assessments_of_claim(&state.pool, &claim_id).await?,
    ))
}

// ── The derivation graph (PHASE-4.6.2; backlog 35) ──────────────────────────────────

/// `POST /v1/derivations` — submit a derivation edge (any enrolled
/// principal; the content MUST hash to the declared digest; the same
/// parent + kind + digest is the replay).
async fn submit_derivation(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(submission): Json<crate::derivations::DerivationSubmission>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no derivation",
        ));
    }
    match crate::derivations::submit(&state.pool, &submission).await {
        Ok(derivation_id) => Ok(Json(json!({ "derivation_id": derivation_id }))),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::derivations::DerivationError::Storage(cause)) => Err(
            ControlApiError::internal_with_log(format!("derivation storage failed: {cause}")),
        ),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}/derivations` — the parent/derived traversal
/// (the snapshot's children, oldest first).
async fn list_derivations(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<Vec<crate::derivations::StoredDerivation>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no derivations",
        ));
    }
    Ok(Json(
        crate::derivations::children_of(&state.pool, &snapshot_id).await?,
    ))
}

/// `POST /v1/snapshots/expire-due` — the retention enforcement: the
/// live snapshots whose class TTL passed are TOMBSTONED with the reason
/// (the optional `at` overrides the clock — the tests drive the expiry).
async fn expire_due_snapshots(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal enforces no retention",
        ));
    }
    let at = body
        .and_then(|Json(value)| {
            value
                .get("at")
                .and_then(|v| v.as_str())
                .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .unwrap_or_else(chrono::Utc::now);
    let tombstoned = crate::snapshots::expire_due(&state.pool, at).await?;
    Ok(Json(json!({ "tombstoned": tombstoned })))
}

/// `GET /v1/snapshots/stale` — the staleness surface (the LIVE snapshots
/// whose freshness horizon passed — the assessments read this).
async fn list_stale_snapshots(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::snapshots::StoredSnapshot>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no staleness",
        ));
    }
    Ok(Json(
        crate::snapshots::stale(&state.pool, chrono::Utc::now()).await?,
    ))
}

// ── The evidence snapshot store (PHASE-4.6.1; backlog 35) ───────────────────────────

/// The snapshot request: the §12.6 facts + the raw bytes (base64 — the
/// content-addressing is VERIFIED: the bytes must hash to the declared
/// digest).
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotRequest {
    #[serde(flatten)]
    submission: crate::snapshots::SnapshotSubmission,
    bytes_base64: String,
}

/// `POST /v1/snapshots` — submit the snapshot (any enrolled principal; the
/// same reference + digest is the replay).
async fn submit_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<SnapshotRequest>,
) -> Result<Json<crate::snapshots::SnapshotOutcome>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no snapshot",
        ));
    }
    let bytes = base64_decode(&request.bytes_base64).ok_or_else(|| {
        ControlApiError::invalid_command("the bytes_base64 field is not valid base64")
    })?;
    match crate::snapshots::submit(&state.pool, &request.submission, &bytes, chrono::Utc::now())
        .await
    {
        Ok(outcome) => Ok(Json(outcome)),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::snapshots::SnapshotError::Storage(cause)) => Err(
            ControlApiError::internal_with_log(format!("snapshot storage failed: {cause}")),
        ),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}` — the read (the tombstone state rides the row).
async fn get_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<crate::snapshots::StoredSnapshot>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no snapshot",
        ));
    }
    match crate::snapshots::get(&state.pool, &snapshot_id).await? {
        Some(snapshot) => Ok(Json(snapshot)),
        None => Err(ControlApiError::not_found(format!(
            "no snapshot `{snapshot_id}`"
        ))),
    }
}

/// `DELETE /v1/snapshots/{id}` — the tombstone: the deletion records the
/// reason + the time (the row stays — never a silent disappearance).
async fn tombstone_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal tombstones no snapshot",
        ));
    }
    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the reason is required"))?;
    let updated = crate::snapshots::tombstone(&state.pool, &snapshot_id, reason).await?;
    Ok(Json(
        json!({ "snapshot_id": snapshot_id, "tombstoned": updated }),
    ))
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input).ok()
}

// ── The universal resource reference (PHASE-4.1.2; backlog 31) ─────────────────────

/// `POST /v1/resources` — submit the typed §12.1 reference (any enrolled
/// principal; the reference is declarative — the resolution is `.1.3`'s).
/// The locator is IMMUTABLE: the same locator + digest is the replay, the
/// same locator with a DIFFERENT digest is the typed conflict.
async fn submit_resource(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(reference): Json<crate::resources::ResourceReference>,
) -> Result<Json<crate::resources::SubmitOutcome>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no reference",
        ));
    }
    if let Some(error) = reference.digest_error() {
        return Err(ControlApiError::invalid_command(error));
    }
    let submitted_by = actor_handle_for_subject(&principal).to_string();
    match crate::resources::submit(&state.pool, &reference, &submitted_by).await {
        Ok(outcome) => Ok(Json(outcome)),
        Err(e) if e.as_database_error().is_some_and(|d| d.is_unique_violation()) => {
            Err(ControlApiError {
                status: StatusCode::CONFLICT,
                code: "locator_digest_conflict",
                message: "the locator's digest is immutable — the same locator with a different digest conflicts"
                    .to_string(),
            })
        }
        Err(e) if e
            .to_string()
            .contains("locator_digest_conflict") =>
        {
            Err(ControlApiError {
                status: StatusCode::CONFLICT,
                code: "locator_digest_conflict",
                message: "the locator's digest is immutable — the same locator with a different digest conflicts"
                    .to_string(),
            })
        }
        Err(e) => Err(ControlApiError::internal_with_log(format!(
            "the reference submit failed: {e}"
        ))),
    }
}

/// `GET /v1/resources/{resource_id}` — the inspection (the submitted shape +
/// the submitter + the creation time).
async fn get_resource(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(resource_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let enrolled = reader_tenant(&state.pool, &principal).await?.is_some();
    if !enrolled {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no reference",
        ));
    }
    let Some((id, reference, submitted_by, created_at)) =
        crate::resources::get(&state.pool, &resource_id).await?
    else {
        return Err(ControlApiError::not_found(format!(
            "no reference `{resource_id}`"
        )));
    };
    Ok(Json(json!({
        "resource_id": id,
        "reference": reference,
        "submitted_by": submitted_by,
        "created_at": created_at.to_rfc3339(),
    })))
}

// ── The open-call artifact + the typed responses (PHASE-3.4.2) ────────────────────

/// The node-initiated thread body (§11.5): the initiation declares its
/// topics + the confidentiality class + the spend bound.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AutoCreateRequest {
    tenant_id: String,
    subject: String,
    objective: String,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    confidentiality_class: Option<String>,
    #[serde(default)]
    budget_amount: Option<f64>,
}

/// `POST /v1/threads/auto` — the node-initiated thread creation (`.3.5.3`):
/// the role needs the EXPLICIT `thread:create:auto` grant, and the §11.5
/// wake checklist evaluates server-side BEFORE the initiation lands — the
/// topic gate (the declared interests cover the topics), the confidentiality
/// match, the concurrency gate, and the grant's spend bound. Replies do NOT
/// inherit the permission (a child thread needs its own grant).
async fn create_thread_auto(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<AutoCreateRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let GrantSubject::Role(role) = &principal else {
        return Err(ControlApiError::unauthorized(
            "only an enrolled role initiates a thread autonomously",
        ));
    };
    let tenant_id: TenantId = req
        .tenant_id
        .parse()
        .map_err(|_| ControlApiError::invalid_command("tenant_id is malformed"))?;

    // 1. THE grant: the explicit `thread:create:auto` authority (audited).
    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::ThreadCreateAuto,
        target: ResourceTarget::Tenant { tenant_id },
    };
    match authority::authorize_guarded(&state.pool, &authz).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            return Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )));
        }
        AuthorizationOutcome::Allowed { .. } => {}
    }

    // 2. THE §11.5 checklist (server-side).
    let profile_row: Option<(Value, Option<i64>)> = sqlx::query_as(
        "SELECT v.profile, (v.profile->'availability'->>'concurrency')::bigint \
         FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
         WHERE v.role_id = $1 AND v.version = p.current_version",
    )
    .bind(role.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((profile_json, concurrency)) = profile_row else {
        return Err(ControlApiError::unauthorized(
            "the role declares no profile — the auto-wake cannot be evaluated",
        ));
    };
    let profile: crate::profiles::AgentProfile = serde_json::from_value(profile_json)
        .map_err(|e| ControlApiError::internal_with_log(format!("profile unreadable: {e}")))?;

    // a. The topic gate: every initiation topic must ride the DECLARED
    //    interests (the auto-wake-for-topic rule).
    for topic in &req.topics {
        if !profile.interests.contains(topic) {
            return Err(ControlApiError::unauthorized(format!(
                "the auto-wake topic gate refuses: `{topic}` is not among the role's declared interests"
            )));
        }
    }
    // b. The confidentiality match.
    if let Some(class) = &req.confidentiality_class {
        if !profile.confidentiality_classes.contains(class) {
            return Err(ControlApiError::unauthorized(format!(
                "the confidentiality class `{class}` does not match the role's declared classes"
            )));
        }
    }
    // c. The concurrency gate (the `.5.2` sibling).
    if concurrency == Some(0) {
        return Err(ControlApiError::unauthorized(
            "the role's declared concurrency is zero — the auto-wake is held",
        ));
    }
    // d. The spend bound: the grant's spend limits cover the declared budget.
    if let Some(budget) = req.budget_amount {
        let max_spend: Option<f64> = sqlx::query_scalar(
            "SELECT max((spend_limits->>'amount')::float) FROM authority_grants \
             WHERE subject_id = $1 AND status = 'active' AND actions @> '[\"thread_create_auto\"]'::jsonb",
        )
        .bind(role.to_string())
        .fetch_one(&state.pool)
        .await?;
        match max_spend {
            Some(limit) if limit >= budget => {}
            _ => {
                return Err(ControlApiError::unauthorized(format!(
                    "the declared budget {budget} exceeds the grant's spend bound"
                )));
            }
        }
    }

    // 3. The initiation rides the SAME create flow (the auto grant authorizes
    //    it — the command machinery does the rest). The idempotency key is the
    //    server-assigned creation key (the auto-initiation is NOT a client
    //    replay surface).
    let body_value = serde_json::json!({
        "tenant_id": tenant_id.to_string(),
        "subject": req.subject,
        "objective": req.objective,
    });
    let mut body: threads::CreateBody = serde_json::from_value(body_value.clone())
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let hash = request_hash(threads::OP_CREATE, &principal, &body_value);
    // `.5.2` (ADR-031): the explicit profile always wins; the routing class
    // resolves through the rule table ONLY when no profile is named (the
    // human authority outranks the rule).
    let profile_id = match body.workflow_profile.as_deref() {
        Some(explicit) => Some(explicit.to_owned()),
        None => match body.routing_class.as_deref() {
            Some(class) => {
                let route = crate::routing::resolve(&state.pool, class)
                    .await
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
                crate::routing::record_resolution(
                    &state.pool,
                    &route,
                    &principal.id_string(),
                    "create_boundary",
                )
                .await?;
                Some(route.arm)
            }
            None => None,
        },
    };
    // The same `.3.3` catch as `create_thread`: the resolve ALWAYS runs —
    // the bare thread defaults to `quick_advice` (never empty steps).
    let resolved = crate::workflows::resolve(&state.pool, profile_id.as_deref())
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let workflow_steps = resolved.steps;
    body.workflow_profile = Some(resolved.profile_id);
    let key = format!("auto_{}_{}", role, tenant_id);
    let response = run_thread_command(
        &state.pool,
        &tenant_id,
        &principal,
        &authz,
        &key,
        &hash,
        CommandTarget::Create {
            body: &body,
            workflow_steps,
        },
    )
    .await?;
    Ok(json_response(response.0, response.1))
}

/// The call-open body (the §10.5 spec; the expression rides typed).
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenCallRequest {
    tenant_id: String,
    thread_id: String,
    expression: crate::matching::EligibilityExpression,
    #[serde(default = "default_min_participants")]
    min_participants: i32,
    #[serde(default = "default_max_participants")]
    max_participants: i32,
    #[serde(default)]
    recommendations_allowed: bool,
    join_deadline: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

fn default_min_participants() -> i32 {
    1
}
fn default_max_participants() -> i32 {
    4
}

/// `POST /v1/calls` — the initiator opens a call on a thread (the gate: the
/// caller holds `ThreadInvite` for the thread — the call rides the SAME
/// invitation machinery per ADR-015).
async fn open_recruitment_call(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<OpenCallRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id: TenantId = req
        .tenant_id
        .parse()
        .map_err(|_| ControlApiError::invalid_command("tenant_id is malformed"))?;
    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::ThreadInvite,
        target: ResourceTarget::Thread {
            tenant_id,
            thread_id: req
                .thread_id
                .parse()
                .map_err(|_| ControlApiError::invalid_command("thread_id is malformed"))?,
        },
    };
    match authority::authorize_guarded(&state.pool, &authz).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            return Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )));
        }
        AuthorizationOutcome::Allowed { .. } => {}
    }
    if req.min_participants < 1 || req.max_participants < req.min_participants {
        return Err(ControlApiError::invalid_command(
            "min_participants must be ≥ 1 and max_participants ≥ min_participants",
        ));
    }
    // The dev-scale storm controls (`.4.3`): the per-tenant + per-initiator
    // open-call fan-out caps — the typed 429 names the limit.
    let initiator = actor_handle_for_subject(&principal).to_string();
    let tenant_open =
        crate::recruitment::open_calls_by(&state.pool, Some(&req.tenant_id), None).await?;
    if tenant_open >= crate::recruitment::MAX_OPEN_CALLS_PER_TENANT {
        return Err(ControlApiError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "storm_control",
            message: format!(
                "the tenant's open-call fan-out limit ({}) is reached",
                crate::recruitment::MAX_OPEN_CALLS_PER_TENANT
            ),
        });
    }
    let initiator_open =
        crate::recruitment::open_calls_by(&state.pool, None, Some(&initiator)).await?;
    if initiator_open >= crate::recruitment::MAX_OPEN_CALLS_PER_INITIATOR {
        return Err(ControlApiError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "storm_control",
            message: format!(
                "the initiator's open-call fan-out limit ({}) is reached",
                crate::recruitment::MAX_OPEN_CALLS_PER_INITIATOR
            ),
        });
    }
    let expression = serde_json::to_value(&req.expression)
        .map_err(|e| ControlApiError::invalid_command(format!("expression: {e}")))?;
    let call_id = crate::recruitment::open_call(
        &state.pool,
        crate::recruitment::OpenCallParams {
            tenant_id: &req.tenant_id,
            thread_id: &req.thread_id,
            initiator: &initiator,
            expression: &expression,
            min_participants: req.min_participants,
            max_participants: req.max_participants,
            recommendations_allowed: req.recommendations_allowed,
            join_deadline: req.join_deadline,
            expires_at: req.expires_at,
        },
    )
    .await?;

    // The advertisement (`.5.2`): the call's topic tags (the expression's
    // interests) MATCH the subscribers' declared interests — the server
    // records the offer (the §10.5 advertisement window's durable trace).
    let subscribers: Vec<String> = sqlx::query_scalar(
        "SELECT p.role_id \
         FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
         JOIN agent_roles r ON r.role_id = p.role_id \
         WHERE v.version = p.current_version AND r.tenant_id = $1 \
           AND v.profile->'interests' ?| $2",
    )
    .bind(&req.tenant_id)
    .bind(req.expression.interests.clone())
    .fetch_all(&state.pool)
    .await?;
    let offered_count = subscribers.len();
    for role_id in subscribers {
        sqlx::query(
            "INSERT INTO recruitment_offers (offer_id, call_id, role_id) \
             VALUES ('ofr_' || gen_random_uuid()::text, $1, $2)",
        )
        .bind(&call_id)
        .bind(&role_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(Json(json!({
        "call_id": call_id,
        "thread_id": req.thread_id,
        "status": "open",
        "offered_to": offered_count,
    })))
}

/// The respondent's current facts for the eligibility gate (the same shape
/// the match surface loads).
async fn respondent_candidate(
    pool: &PgPool,
    role_id: &str,
) -> Option<(
    crate::matching::EligibilityCandidate,
    crate::presence::PresenceState,
)> {
    let row: Option<(bool, bool, Option<i64>, Option<Value>)> = sqlx::query_as(
        "SELECT np.online, np.suspended, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS concurrency, \
                (SELECT v.profile \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS profile \
         FROM node_presence np WHERE np.node_id = $1",
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await
    .ok()?;
    let (online, suspended, concurrency, profile) = row?;
    let state = crate::presence::presence_state(true, suspended, online, concurrency);
    let parsed =
        profile.and_then(|p| serde_json::from_value::<crate::profiles::AgentProfile>(p).ok());
    Some((
        crate::matching::EligibilityCandidate {
            role_id: role_id.to_string(),
            profile: parsed,
            presence_state: state,
            concurrency,
            available_budget: None,
        },
        state,
    ))
}

/// `POST /v1/calls/{call_id}/respond` — the respondent submits a TYPED §10.5
/// response. The gate: the call is open, the deadline has not passed, the
/// respondent is an enrolled role, and the server re-resolves the call's
/// eligibility expression against the respondent's CURRENT facts — an
/// ineligible response refuses with the stage-1 reasons.
async fn respond_to_call(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(call_id): Path<String>,
    Json(response): Json<crate::recruitment::RecruitmentResponse>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let value = respond_to_call_core(&state.pool, &principal, &call_id, &response).await?;
    Ok(Json(value))
}

/// The call-respond core — the SAME handler the MCP `join_call` tool rides
/// (the `.3.5.1` write seam calls this after its qualified gate: the
/// enrolled-ROLE check + the eligibility re-resolution + the response
/// record — never a new authority path).
pub(crate) async fn respond_to_call_core(
    pool: &PgPool,
    principal: &GrantSubject,
    call_id: &str,
    response: &crate::recruitment::RecruitmentResponse,
) -> Result<Value, ControlApiError> {
    let GrantSubject::Role(role) = principal else {
        return Err(ControlApiError::unauthorized(
            "only an enrolled role responds to a call",
        ));
    };
    let respondent = role.to_string();
    let Some(call) = crate::recruitment::call(pool, call_id).await? else {
        return Err(ControlApiError::not_found(format!("no call `{call_id}`")));
    };
    if call.status != "open" {
        return Err(ControlApiError::invalid_transition(format!(
            "the call is {}",
            call.status
        )));
    }
    if Utc::now() > call.join_deadline {
        return Err(ControlApiError::invalid_transition(
            "the join deadline has passed",
        ));
    }
    if Utc::now() > call.expires_at {
        return Err(ControlApiError::invalid_transition("the call has expired"));
    }
    let expression: crate::matching::EligibilityExpression =
        serde_json::from_value(call.expression).map_err(|e| {
            ControlApiError::internal_with_log(format!("stored expression unreadable: {e}"))
        })?;
    // The eligibility gate applies to the PARTICIPATION claims (join /
    // conditional_join) — a decline/recommend/recuse is exactly the
    // ineligible (or unwilling) declaring why, and must not be refused.
    let participation = matches!(response.kind(), "join" | "conditional_join");
    if participation {
        let Some((candidate, _state)) = respondent_candidate(pool, &respondent).await else {
            return Err(ControlApiError::unauthorized(
                "the respondent has no enrolled node/profile",
            ));
        };
        let verdict = crate::matching::eligible(&expression, &candidate);
        if !verdict.eligible {
            return Err(ControlApiError::unauthorized(format!(
                "the respondent is ineligible: {}",
                verdict.reasons.join("; ")
            )));
        }
    }
    crate::recruitment::record_response(pool, call_id, &respondent, response).await?;
    Ok(json!({
        "call_id": call_id,
        "respondent": respondent,
        "response": response.kind(),
    }))
}

/// `POST /v1/calls/{call_id}/close` — the initiator (or the tenant owner)
/// closes the call: the server snapshots the SELECTED panel (the joiners,
/// ranked, capped at the max) + the selection explanation (each panelist's
/// stage-1 reasons + the stage-2 features).
async fn close_call(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(call_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(call) = crate::recruitment::call(&state.pool, &call_id).await? else {
        return Err(ControlApiError::not_found(format!("no call `{call_id}`")));
    };
    if call.status != "open" {
        return Err(ControlApiError::invalid_transition(format!(
            "the call is {}",
            call.status
        )));
    }
    // The gate: the initiator or the tenant owner (audited).
    let is_initiator = call.initiator == actor_handle_for_subject(&principal).to_string();
    let is_owner = if let Ok(tenant) = call.tenant_id.parse() {
        authorize_tenant_admin(&state.pool, &principal, tenant)
            .await
            .is_ok()
    } else {
        false
    };
    if !is_initiator && !is_owner {
        return Err(ControlApiError::unauthorized(
            "only the initiator or the tenant owner closes the call",
        ));
    }

    let expression: crate::matching::EligibilityExpression =
        serde_json::from_value(call.expression.clone()).map_err(|e| {
            ControlApiError::internal_with_log(format!("stored expression unreadable: {e}"))
        })?;
    let responses = crate::recruitment::responses(&state.pool, &call_id).await?;
    let joiners: Vec<String> = responses
        .iter()
        .filter(|(_, kind, _, _)| kind == "join")
        .map(|(respondent, _, _, _)| respondent.clone())
        .collect();
    if (joiners.len() as i32) < call.min_participants {
        return Err(ControlApiError::invalid_transition(format!(
            "the panel needs at least {} joiners, {} responded",
            call.min_participants,
            joiners.len()
        )));
    }
    // Rank the joiners (the default preferences) and cap at the max.
    let mut candidates: Vec<(
        crate::matching::EligibilityCandidate,
        crate::matching::EligibilityVerdict,
    )> = Vec::new();
    for joiner in &joiners {
        if let Some((candidate, _)) = respondent_candidate(&state.pool, joiner).await {
            let verdict = crate::matching::eligible(&expression, &candidate);
            candidates.push((candidate, verdict));
        }
    }
    // The dependence facts (`.6.2`): each joiner's LATEST incarnation lineage
    // + the tenant as the owner — the panel's indicator inputs.
    let mut facts: std::collections::HashMap<String, crate::dependence::MemberFacts> =
        std::collections::HashMap::new();
    for joiner in &joiners {
        let lineage: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT provider, model, harness FROM incarnations \
                 WHERE role_id = $1 ORDER BY valid_from DESC LIMIT 1",
        )
        .bind(joiner)
        .fetch_optional(&state.pool)
        .await?;
        let (provider, model_family, harness) = lineage.unwrap_or((None, None, None));
        facts.insert(
            joiner.clone(),
            crate::dependence::MemberFacts {
                role_id: joiner.clone(),
                provider,
                model_family,
                harness,
                lineage: None,
                owner: Some(call.tenant_id.clone()),
            },
        );
    }
    let mut ranked = crate::matching::rank_with_dependence(
        &expression,
        &candidates,
        &Default::default(),
        Some(&facts),
    );
    ranked.truncate(call.max_participants as usize);
    let indicators: Vec<crate::dependence::DependenceIndicator> =
        crate::dependence::dependence_indicators(
            &ranked
                .iter()
                .filter_map(|r| facts.get(&r.role_id).cloned())
                .collect::<Vec<_>>(),
        );
    crate::recruitment::snapshot_panel(&state.pool, &call_id, &ranked, &indicators).await?;
    Ok(Json(json!({
        "call_id": call_id,
        "status": "closed",
        "panel": ranked
            .iter()
            .map(|r| r.role_id.clone())
            .collect::<Vec<_>>(),
    })))
}

/// `GET /v1/calls/{call_id}` — the inspection: the spec + the responses + the
/// snapshot (the initiator/owner view; the `.5` lane adds the wider views).
async fn inspect_call(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(call_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(call) = crate::recruitment::call(&state.pool, &call_id).await? else {
        return Err(ControlApiError::not_found(format!("no call `{call_id}`")));
    };
    let is_initiator = call.initiator == actor_handle_for_subject(&principal).to_string();
    let is_owner = if let Ok(tenant) = call.tenant_id.parse() {
        authorize_tenant_admin(&state.pool, &principal, tenant)
            .await
            .is_ok()
    } else {
        false
    };
    if !is_initiator && !is_owner {
        return Err(ControlApiError::unauthorized(
            "only the initiator or the tenant owner inspects the call",
        ));
    }
    let responses = crate::recruitment::responses(&state.pool, &call_id).await?;
    let offers: Vec<String> = sqlx::query_scalar(
        "SELECT role_id FROM recruitment_offers WHERE call_id = $1 ORDER BY role_id",
    )
    .bind(&call_id)
    .fetch_all(&state.pool)
    .await?;
    let snapshot: Option<(Value, Value, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT panel, explanation, snapshotted_at FROM recruitment_panels WHERE call_id = $1",
    )
    .bind(&call_id)
    .fetch_optional(&state.pool)
    .await?;
    Ok(Json(json!({
        "call_id": call.call_id,
        "thread_id": call.thread_id,
        "status": call.status,
        "min_participants": call.min_participants,
        "max_participants": call.max_participants,
        "offers": offers,
        "responses": responses
            .into_iter()
            .map(|(respondent, kind, payload, at)| json!({
                "respondent": respondent,
                "kind": kind,
                "payload": payload,
                "at": at.to_rfc3339(),
            }))
            .collect::<Vec<_>>(),
        "panel": snapshot.as_ref().map(|(panel, _, _)| panel.clone()),
        "explanation": snapshot.as_ref().map(|(_, explanation, _)| explanation.clone()),
    })))
}

// ── The matching query surface (PHASE-3.3.3; backlog 28/29) ─────────────────────

/// The match request: the initiator's eligibility expression + the stage-2
/// preferences (the defaults apply when absent).
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchRequest {
    expression: crate::matching::EligibilityExpression,
    #[serde(default)]
    preferences: crate::matching::RankingPreferences,
}

fn scope_rank(class: crate::profiles::ReaderClass) -> u8 {
    match class {
        crate::profiles::ReaderClass::Full => 2,
        crate::profiles::ReaderClass::Tenant => 1,
        crate::profiles::ReaderClass::Network => 0,
    }
}

/// `POST /v1/directory/match` — the initiator's expression resolves
/// SERVER-side (§10.2: the caller never enumerates the network). The
/// expression's scope must not exceed the reader's classification (a network
/// reader cannot demand the tenant view — the typed 403). The response carries
/// ONLY the eligible candidates, ranked, each with the stage-1 reasons + the
/// stage-2 explanations + the profile fields VISIBLE to the reader.
async fn directory_match(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<MatchRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(reader_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal matches nothing",
        ));
    };
    let owner = if let Ok(tenant) = reader_tenant.parse() {
        authorize_tenant_admin(&state.pool, &principal, tenant)
            .await
            .is_ok()
    } else {
        false
    };
    let reader_class = if owner {
        crate::profiles::ReaderClass::Full
    } else if crate::profiles::ReaderClass::Tenant == req.expression.scope
        || crate::profiles::ReaderClass::Network == req.expression.scope
    {
        // A same-tenant member may search at most the tenant scope; anyone
        // else at most the network scope.
        crate::profiles::ReaderClass::Tenant
    } else {
        crate::profiles::ReaderClass::Network
    };
    // The scope clamp: the expression must not exceed the reader's class.
    if scope_rank(req.expression.scope) > scope_rank(reader_class) {
        return Err(ControlApiError::unauthorized(
            "the expression's scope exceeds the reader's classification".to_string(),
        ));
    }

    type Row = (
        String,
        bool,
        bool,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<i64>,
        Option<Value>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT np.node_id, np.online, np.suspended, np.last_seen_at, np.lease_expires_at, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS concurrency, \
                (SELECT v.profile \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS profile \
         FROM node_presence np ORDER BY np.node_id",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut candidates: Vec<(
        crate::matching::EligibilityCandidate,
        crate::matching::EligibilityVerdict,
    )> = Vec::new();
    for (node_id, online, suspended, _last_seen, _lease_expiry, concurrency, profile) in rows {
        let Some(stored) = profile else {
            continue; // a role node without a profile declares nothing
        };
        let Ok(parsed) = serde_json::from_value::<crate::profiles::AgentProfile>(stored) else {
            continue;
        };
        let state = crate::presence::presence_state(true, suspended, online, concurrency);
        let candidate = crate::matching::EligibilityCandidate {
            role_id: node_id.clone(),
            profile: Some(parsed),
            presence_state: state,
            concurrency,
            // The dev profile has no per-role budget facts: UNKNOWN, never
            // zero (§14.5) — a budget requirement therefore cannot be proven.
            available_budget: None,
        };
        let verdict = crate::matching::eligible(&req.expression, &candidate);
        candidates.push((candidate, verdict));
    }

    let ranked = crate::matching::rank(&req.expression, &candidates, &req.preferences);
    let out: Vec<Value> = ranked
        .into_iter()
        .map(|r| {
            let profile = candidates
                .iter()
                .find(|(c, _)| c.role_id == r.role_id)
                .and_then(|(c, _)| c.profile.as_ref())
                .map(|p| crate::profiles::filter_profile(p, reader_class))
                .unwrap_or_else(|| json!({}));
            json!({
                "role_id": r.role_id,
                "stage1_reasons": r.stage1_reasons,
                "features": r.features,
                "total": r.total,
                "profile": profile,
            })
        })
        .collect();
    Ok(Json(json!({
        "scope": format!("{:?}", reader_class).to_lowercase(),
        "candidates": out,
    })))
}

// ── The privacy-filtered directory views (PHASE-3.2.3; backlog 27) ──────────────

/// The reader's directory scope: the OWNER (tenant_admin) reads the FULL fields
/// of their own tenant's nodes; a tenant member reads the TENANT-filtered
/// fields; every enrolled principal reads the network pseudonyms of the other
/// tenants (a profile whose network view is empty contributes NOTHING — not
/// even a count). The `.1.3` filter is the field-level engine.
async fn directory_presence(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(reader_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no directory",
        ));
    };
    let owner = if let Ok(tenant) = reader_tenant.parse() {
        authorize_tenant_admin(&state.pool, &principal, tenant)
            .await
            .is_ok()
    } else {
        false
    };
    let own_class = if owner {
        crate::profiles::ReaderClass::Full
    } else {
        crate::profiles::ReaderClass::Tenant
    };

    type Row = (
        String,
        String,
        bool,
        bool,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<i64>,
        Option<Value>,
    );
    // Every enrolled node with its derived presence + its CURRENT profile
    // (when the node id is the role wire id it serves — the dev wiring).
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT np.node_id, np.tenant_id, np.online, np.suspended, np.last_seen_at, np.lease_expires_at, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS concurrency, \
                (SELECT v.profile \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version) AS profile \
         FROM node_presence np ORDER BY np.tenant_id, np.node_id",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut own_nodes = Vec::new();
    let mut network_nodes = Vec::new();
    for (
        node_id,
        tenant,
        online,
        suspended,
        last_seen_at,
        lease_expires_at,
        concurrency,
        profile,
    ) in rows
    {
        let state = crate::presence::presence_state(true, suspended, online, concurrency)
            .as_str()
            .to_string();
        let (target, fields) = if tenant == reader_tenant {
            (true, profile)
        } else {
            (false, profile)
        };
        let entry = match (target, fields) {
            (true, Some(stored)) => {
                let parsed: crate::profiles::AgentProfile = match serde_json::from_value(stored) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let filtered = crate::profiles::filter_profile(&parsed, own_class);
                Some(json!({
                    "node_id": node_id,
                    "state": state,
                    "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                    "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
                    "profile": filtered,
                }))
            }
            (true, None) => Some(json!({
                "node_id": node_id,
                "state": state,
                "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
            })),
            (false, Some(stored)) => {
                let parsed: crate::profiles::AgentProfile = match serde_json::from_value(stored) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let filtered =
                    crate::profiles::filter_profile(&parsed, crate::profiles::ReaderClass::Network);
                // The zero-visibility rule: a profile whose network view is
                // EMPTY contributes nothing — not even a count.
                if filtered.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    continue;
                }
                Some(json!({
                    "node_id": node_id,
                    "state": state,
                    "profile": filtered,
                }))
            }
            (false, None) => continue,
        };
        if let Some(entry) = entry {
            if target {
                own_nodes.push(entry);
            } else {
                network_nodes.push(entry);
            }
        }
    }

    Ok(Json(json!({
        "own_tenant": {
            "tenant_id": reader_tenant,
            "nodes": own_nodes,
        },
        "network": {
            "nodes": network_nodes,
        },
    })))
}

// ── Directory profiles (PHASE-3.1.2; backlog 26) ──────────────────────────────

/// The profile write gate: ONLY the role itself may write its profile (a
/// self-declaration — §10.1 allows self-asserted claims, the provenance is
/// shown). The owner's path is the attestation verb below.
fn is_self(principal: &GrantSubject, role_id: &str) -> bool {
    matches!(principal, GrantSubject::Role(r) if r.to_string() == role_id)
}

async fn role_tenant(pool: &PgPool, role_id: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT tenant_id FROM agent_roles WHERE role_id = $1")
        .bind(role_id)
        .fetch_optional(pool)
        .await
}

/// The same lookup on a caller-owned transaction, so a route that gates on the
/// role's existence and then writes reads ONE snapshot (`SIGNOFF-REPAIR.3.3.4.11.1`).
async fn role_tenant_in_tx(
    tx: &mut sqlx::PgConnection,
    role_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT tenant_id FROM agent_roles WHERE role_id = $1")
        .bind(role_id)
        .fetch_optional(&mut *tx)
        .await
}

/// The shared read gate for `.1.2` (the `.1.3` leaf adds the per-reader
/// filtering): the role itself or its tenant owner sees the full profile.
async fn authorize_profile_read(
    pool: &PgPool,
    principal: &GrantSubject,
    role_id: &str,
) -> Result<(), ControlApiError> {
    if is_self(principal, role_id) {
        return Ok(());
    }
    let Some(tenant) = role_tenant(pool, role_id).await? else {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    };
    authorize_tenant_admin(
        pool,
        principal,
        tenant.parse().map_err(|_| ControlApiError::internal())?,
    )
    .await
}

/// `PUT /v1/profiles/{role_id}` — the role declares its own profile. A NEW
/// content-addressed version every write; the writer is the actor handle.
async fn put_profile(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
    Json(profile): Json<crate::profiles::AgentProfile>,
) -> Result<Json<crate::profiles::CurrentProfile>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if !is_self(&principal, &role_id) {
        return Err(ControlApiError::unauthorized(
            "only the role itself may write its profile (the owner attests via /attest)",
        ));
    }
    // The provenance gate: a role's own write may declare only SELF-ASSERTED
    // claims — the owner_attested/benchmarked/certified upgrades ride the
    // audited attest verb (§10.1: the provenance is shown, never self-granted).
    if let Some(claim) = profile
        .capabilities
        .iter()
        .find(|c| c.confidence != crate::profiles::ClaimConfidence::SelfAsserted)
    {
        return Err(ControlApiError::invalid_command(format!(
            "capability `{}` declares {} — a role's own write may declare only \
             self_asserted (the owner attests the upgrade via /attest)",
            claim.taxonomy_id,
            claim.confidence.rank_name()
        )));
    }
    let writer = actor_handle_for_subject(&principal).to_string();
    // ONE transaction (`SIGNOFF-REPAIR.3.3.4.11.1`): the role's existence, the
    // lineage check and the version write share a single snapshot and a single
    // commit, so a refusal cannot leave a version row or an advanced anchor
    // behind and the check can no longer read a different snapshot from the
    // write. It takes no tenant authority guard, and deliberately so: this route
    // is gated on identity — only the role itself writes its own profile — and
    // evaluates no grant, so there is no authority decision for a guard to order
    // it against. `.11.2` and `.11.3` DO take one, because theirs are admitted.
    let mut tx = state.pool.begin().await?;
    let at = crate::authority::transaction::database_now_in_tx(&mut tx).await?;
    if role_tenant_in_tx(&mut tx, &role_id).await?.is_none() {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    }
    // The lineage link, when present, must name a real incarnation of THIS role.
    if let Some(incarnation_id) = &profile.incarnation_id {
        let exists: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM incarnations WHERE incarnation_id = $1 AND role_id = $2)",
        )
        .bind(incarnation_id)
        .bind(&role_id)
        .fetch_optional(&mut *tx)
        .await?;
        if !exists.unwrap_or(false) {
            return Err(ControlApiError::invalid_command(format!(
                "incarnation `{incarnation_id}` is not an incarnation of `{role_id}`"
            )));
        }
    }
    let written = crate::profiles::write_profile_in_tx(&mut tx, &role_id, &writer, &profile, at)
        .await
        .map_err(|e| profile_error(e, &role_id))?;
    tx.commit().await?;
    Ok(Json(written))
}

fn profile_error(e: sqlx::Error, role_id: &str) -> ControlApiError {
    if e.as_database_error()
        .is_some_and(|d| d.is_foreign_key_violation())
    {
        ControlApiError::not_found(format!("no role `{role_id}`"))
    } else {
        ControlApiError::internal_with_log(format!("profile write failed: {e}"))
    }
}

/// The reader's tenant (their identity row), when enrolled.
pub(crate) async fn reader_tenant(
    pool: &PgPool,
    principal: &GrantSubject,
) -> Result<Option<String>, sqlx::Error> {
    let id = principal.id_string();
    match principal {
        GrantSubject::Human(_) => {
            sqlx::query_scalar("SELECT tenant_id FROM human_principals WHERE principal_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
        }
        GrantSubject::Role(_) => {
            sqlx::query_scalar("SELECT tenant_id FROM agent_roles WHERE role_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
        }
    }
}

/// The per-reader classification (`.1.3`): the role itself or its tenant owner
/// reads FULL; a same-tenant principal reads the TENANT view; any other
/// enrolled principal reads the NETWORK view.
async fn classify_reader(
    pool: &PgPool,
    principal: &GrantSubject,
    role_id: &str,
) -> Result<Option<crate::profiles::ReaderClass>, ControlApiError> {
    if is_self(principal, role_id) {
        return Ok(Some(crate::profiles::ReaderClass::Full));
    }
    let Some(role_tenant) = role_tenant(pool, role_id).await? else {
        return Ok(None);
    };
    let Some(reader_tenant) = reader_tenant(pool, principal).await? else {
        return Ok(None); // an unenrolled principal reads nothing
    };
    if reader_tenant == role_tenant {
        // The owner (tenant_admin) reads FULL — the decision is audited.
        if let Ok(tenant) = role_tenant.parse() {
            if authorize_tenant_admin(pool, principal, tenant)
                .await
                .is_ok()
            {
                return Ok(Some(crate::profiles::ReaderClass::Full));
            }
        }
        return Ok(Some(crate::profiles::ReaderClass::Tenant));
    }
    // The federation agreement (`.1.2`, ADR-026): a network reader whose
    // tenant holds the EFFECTIVE (both-sides accepted) directory-visibility
    // agreement with the profile's tenant reads the TENANT view — the
    // explicit opt-in; no agreement (or a revoked/one-sided one) stays the
    // network pseudonym. The widening never widens beyond the tenant view.
    if crate::federation::has_effective_directory_agreement(pool, &reader_tenant, &role_tenant)
        .await?
    {
        return Ok(Some(crate::profiles::ReaderClass::Tenant));
    }
    Ok(Some(crate::profiles::ReaderClass::Network))
}

/// `GET /v1/profiles/{role_id}` — the per-reader filtered profile (`.1.3`):
/// the role/owner reads the full profile; a tenant sibling reads the tenant
/// view; a stranger reads the network view. A hidden field is ABSENT, never
/// nulled; the response names the applied class.
async fn get_profile(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(class) = classify_reader(&state.pool, &principal, &role_id).await? else {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    };
    let Some(current) = crate::profiles::current_profile(&state.pool, &role_id).await? else {
        return Err(ControlApiError::not_found(format!(
            "no profile for `{role_id}`"
        )));
    };
    let (profile, visibility) = match class {
        crate::profiles::ReaderClass::Full => {
            let profile: crate::profiles::AgentProfile =
                serde_json::from_value(current.profile.clone()).map_err(|e| {
                    ControlApiError::internal_with_log(format!(
                        "stored profile no longer parses: {e}"
                    ))
                })?;
            (crate::profiles::filter_profile(&profile, class), "full")
        }
        other => {
            let profile: crate::profiles::AgentProfile =
                serde_json::from_value(current.profile.clone()).map_err(|e| {
                    ControlApiError::internal_with_log(format!(
                        "stored profile no longer parses: {e}"
                    ))
                })?;
            let name = match other {
                crate::profiles::ReaderClass::Tenant => "tenant",
                crate::profiles::ReaderClass::Network => "network",
                crate::profiles::ReaderClass::Full => unreachable!(),
            };
            (crate::profiles::filter_profile(&profile, other), name)
        }
    };
    Ok(Json(json!({
        "role_id": role_id,
        "version": current.version,
        "content_hash": current.content_hash,
        "visibility": visibility,
        "written_by": current.written_by,
        "written_at": current.written_at.to_rfc3339(),
        "profile": profile,
    })))
}

// ── The portable agent cards (PHASE-8.1.3; ADR-026/027) ────────────────────────

/// `GET /v1/profiles/{role_id}/card` — mint the digest-pinned portable
/// card (the FULL profile + the origin identity). Only the role itself or
/// its tenant admin (the Full class) exports the portable form.
async fn get_profile_card(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(class) = classify_reader(&state.pool, &principal, &role_id).await? else {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    };
    if class != crate::profiles::ReaderClass::Full {
        return Err(ControlApiError::unauthorized(
            "only the role itself or its tenant admin exports the portable card",
        ));
    }
    let Some(current) = crate::profiles::current_profile(&state.pool, &role_id).await? else {
        return Err(ControlApiError::not_found(format!(
            "no profile for `{role_id}`"
        )));
    };
    let profile: crate::profiles::AgentProfile =
        serde_json::from_value(current.profile).map_err(|e| {
            ControlApiError::internal_with_log(format!("stored profile no longer parses: {e}"))
        })?;
    let origin_tenant = role_tenant(&state.pool, &role_id)
        .await?
        .ok_or_else(|| ControlApiError::not_found(format!("no role `{role_id}`")))?;
    let card = crate::cards::AgentCard {
        schema_version: crate::cards::CARD_SCHEMA_VERSION.to_string(),
        origin_tenant_id: origin_tenant,
        origin_role_id: role_id,
        profile,
        exported_at: Utc::now().to_rfc3339(),
    };
    let digest = crate::cards::digest_of(&card)
        .map_err(|e| ControlApiError::internal_with_log(format!("the card digests: {e}")))?;
    Ok(Json(json!({ "card": card, "digest": digest })))
}

/// The card import body: the importing tenant + the card + the digest the
/// card claims (the re-derivation rung).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportCardRequest {
    pub tenant_id: TenantId,
    pub card: crate::cards::AgentCard,
    pub digest: String,
}

/// `POST /v1/profiles/cards/import` — the ADR-027 ladder over the card:
/// the digest rung (the re-derivation), the compatibility rung (the
/// schema), the allowlist rung (the EFFECTIVE recruitment agreement with
/// the origin), and the capability rung (the fresh local role under the
/// importing boundary's default grant — the card's self-asserted
/// capabilities NEVER confer authority; the local grant is the only
/// authority that acts, the ADR-026 invariant).
async fn import_profile_card(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ImportCardRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    // The pure rungs first — no storage before the card proves itself.
    crate::cards::verify_pure_rungs(&req.card, &req.digest)
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    // The allowlist rung: the effective recruitment agreement with the origin.
    if !crate::federation::has_effective_recruitment_agreement(
        &state.pool,
        &req.tenant_id.to_string(),
        &req.card.origin_tenant_id,
    )
    .await?
    {
        return Err(ControlApiError::unauthorized(format!(
            "no effective federation agreement with the origin tenant `{}` — the import refuses",
            req.card.origin_tenant_id
        )));
    }
    // The capability rung + the local identity: the fresh role, the default
    // grant under the importing boundary (the boundary-checked evaluation),
    // and the enrollment row — the enroll's role-branch shape.
    let boundary = authority::load_active_boundary_for_tenant(&state.pool, &req.tenant_id)
        .await?
        .ok_or_else(|| {
            ControlApiError::invalid_command(
                "the tenant has no active enrollment boundary — the import refuses",
            )
        })?;
    let role = GrantSubject::Role(AgentRoleId::new());
    let role_id = role.id_string();
    let grant = dev_grant(
        &boundary,
        HumanPrincipalId::new(),
        role.clone(),
        vec![
            GrantAction::ThreadContribute,
            GrantAction::ThreadInvitationRespond,
        ],
    );
    let mut tx = state.pool.begin().await?;
    authority::create_grant_unordered_in_tx(&mut *tx, &grant)
        .await
        .map_err(|error| {
            ControlApiError::grant_creation(
                error,
                "the imported role's grant exceeds the importing boundary",
            )
        })?;
    sqlx::query("INSERT INTO agent_roles (role_id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(&role_id)
        .bind(req.tenant_id.to_string())
        .bind(req.card.profile.display_label.clone())
        .execute(&mut *tx)
        .await?;
    // The imported role's per-principal quota (`.3.5.1`) — the identity row
    // implies its quota row (the fail-closed write gate).
    crate::quota::insert_principal_default_in_tx(&mut *tx, &req.tenant_id.to_string(), &role_id)
        .await?;
    sqlx::query(
        "INSERT INTO enrollments (principal_id, tenant_id, kind, name) VALUES ($1, $2, 'role', $3)",
    )
    .bind(&role_id)
    .bind(req.tenant_id.to_string())
    .bind(req.card.profile.display_label.clone())
    .execute(&mut *tx)
    .await?;
    // The cross-domain receipt (`.1.4`, ADR-026): the remote reference is
    // the card's digest (the remote domain's own re-derivable record);
    // the local reference is the fresh role — the receipt CROSS-REFERENCES,
    // it never merges the chains.
    crate::receipts::record_in_tx(
        &mut *tx,
        &req.tenant_id.to_string(),
        &req.card.origin_tenant_id,
        crate::receipts::KIND_CARD_IMPORT,
        &req.digest,
        &role_id,
    )
    .await?;
    tx.commit().await?;
    // The profile from the card (the content-addressed write path).
    crate::profiles::write_profile(&state.pool, &role_id, &role_id, &req.card.profile)
        .await
        .map_err(|e| {
            ControlApiError::internal_with_log(format!("the imported profile fails to write: {e}"))
        })?;
    Ok(Json(json!({
        "role_id": role_id,
        "origin_tenant_id": req.card.origin_tenant_id,
        "origin_role_id": req.card.origin_role_id,
    })))
}

/// `GET /v1/profiles/{role_id}/versions` — the content-addressed history.
async fn list_profile_versions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_profile_read(&state.pool, &principal, &role_id).await?;
    let rows = crate::profiles::version_list(&state.pool, &role_id).await?;
    Ok(Json(json!({
        "role_id": role_id,
        "versions": rows
            .into_iter()
            .map(|(version, content_hash, written_by, written_at)| json!({
                "version": version,
                "content_hash": content_hash,
                "written_by": written_by,
                "written_at": written_at.to_rfc3339(),
            }))
            .collect::<Vec<_>>(),
    })))
}

/// `GET /v1/profiles/{role_id}/versions/{version}` — one historical version.
async fn get_profile_version(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path((role_id, version)): Path<(String, i32)>,
) -> Result<Json<crate::profiles::ProfileVersionRow>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_profile_read(&state.pool, &principal, &role_id).await?;
    let Some(row) = crate::profiles::version_at(&state.pool, &role_id, version).await? else {
        return Err(ControlApiError::not_found(format!(
            "no version {version} of `{role_id}`'s profile"
        )));
    };
    Ok(Json(row))
}

/// The attestation body: the owner upgrades ONE named capability claim's
/// provenance to `owner_attested` with the evidence reference.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AttestRequest {
    taxonomy_id: String,
    evidence_ref: String,
}

/// `POST /v1/profiles/{role_id}/attest` — tenant_admin-gated (audited); a NEW
/// version whose writer is the attesting owner.
async fn attest_capability_claim(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
    Json(req): Json<AttestRequest>,
) -> Result<Json<crate::profiles::CurrentProfile>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = role_tenant(&state.pool, &role_id).await? else {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    };
    authorize_tenant_admin(
        &state.pool,
        &principal,
        tenant.parse().map_err(|_| ControlApiError::internal())?,
    )
    .await?;
    let writer = actor_handle_for_subject(&principal).to_string();
    let Some(written) = crate::profiles::attest_capability(
        &state.pool,
        &role_id,
        &writer,
        &req.taxonomy_id,
        &req.evidence_ref,
    )
    .await?
    else {
        return Err(ControlApiError::not_found(format!(
            "no profile for `{role_id}` or no capability `{}` in it",
            req.taxonomy_id
        )));
    };
    Ok(Json(written))
}

// ── Grant/boundary revocation + inspection (`.1.3.2`) ─────────────────────────

/// The revoke bodies: the acting human names the tenant they administer (the
/// target id rides the path); the reason is required.
#[derive(Debug, Deserialize)]
pub struct AdminListQuery {
    pub tenant_id: TenantId,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeAuthorityRequest {
    pub tenant_id: TenantId,
    pub reason: String,
}

/// Both revocation routes run ONE guarded transaction (`SIGNOFF-REPAIR.3.3.4.8`):
/// the admission, the tenant-bound target selection, the status change, the
/// epoch bump and the final effect record share a single commit under the
/// tenant's exclusive authority guard. Before this, the handler ran TWO guarded
/// transactions — a shared-guard admission and then an exclusive-guard service
/// call — so the admission was a fact about authority that could already have
/// stopped holding by the time the mutation ran.
///
/// Every answer below, including the refusals, carries the
/// `x-reasonbraid-authorization` receipt naming the admission this request
/// committed — which is also the effect record's id when one was written.
/// Without it the effect record would be unreachable: there is no list endpoint.
/// This follows the inspection routes' established shape, where a denial names
/// its record too.
async fn run_revocation(
    state: &ApiState,
    headers: &HeaderMap,
    req: &RevokeAuthorityRequest,
    target: authority::RevocationTarget,
    target_id: &str,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(headers)?;
    let revocation = authority::revoke_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        target,
        target_id,
        &req.reason,
    )
    .await?;
    let noun = match target {
        authority::RevocationTarget::Grant => "grant",
        authority::RevocationTarget::Boundary => "boundary",
    };
    let receipt = revocation.record_id;
    let response = match revocation.result {
        authority::RevocationResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::RevocationResult::InvalidReason(detail) => {
            ControlApiError::invalid_command(format!("the revocation reason is required: {detail}"))
                .into_response()
        }
        // Missing and foreign are ONE answer, so a caller learns nothing about
        // another tenant's ids (`SIGNOFF-REPAIR.3.1`).
        authority::RevocationResult::NotFound => {
            ControlApiError::not_found(format!("no {noun} `{target_id}` in this tenant"))
                .into_response()
        }
        authority::RevocationResult::AlreadyRevoked => {
            ControlApiError::invalid_transition(format!("{noun} `{target_id}` is already revoked"))
                .into_response()
        }
        authority::RevocationResult::Applied => {
            let key = match target {
                authority::RevocationTarget::Grant => "grant_id",
                authority::RevocationTarget::Boundary => "boundary_id",
            };
            Json(json!({
                key: target_id,
                "tenant_id": req.tenant_id.to_string(),
                // Database time from inside the transaction, not a process clock
                // read after it: this is the instant the decision, the mutation
                // and the effect record actually share.
                "revoked_at": revocation.effected_at.to_rfc3339(),
            }))
            .into_response()
        }
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

async fn revoke_grant(
    State(state): State<Arc<ApiState>>,
    Path(grant_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RevokeAuthorityRequest>,
) -> Result<Response, ControlApiError> {
    run_revocation(
        &state,
        &headers,
        &req,
        authority::RevocationTarget::Grant,
        &grant_id,
    )
    .await
}

async fn revoke_boundary(
    State(state): State<Arc<ApiState>>,
    Path(boundary_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RevokeAuthorityRequest>,
) -> Result<Response, ControlApiError> {
    run_revocation(
        &state,
        &headers,
        &req,
        authority::RevocationTarget::Boundary,
        &boundary_id,
    )
    .await
}

/// Admit and record a named own-tenant inspection before running its query.
/// Both allow and deny responses name a committed record. If the query later
/// fails, its response retains the real admission receipt; failed authority/audit
/// storage cannot advertise an unconfirmed receipt.
async fn inspect_tenant_admin<F, Fut>(
    state: &ApiState,
    principal: &GrantSubject,
    tenant_id: TenantId,
    inspection: reasonbraid_core::TenantAdminInspection,
    read: F,
) -> Result<Response, ControlApiError>
where
    F: FnOnce(PgPool) -> Fut,
    Fut: std::future::Future<Output = Result<Json<Value>, ControlApiError>>,
{
    let outcome =
        authority::authorize_tenant_admin_inspection(&state.pool, principal, tenant_id, inspection)
            .await?;
    let (record_id, response) = match outcome {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            let response = ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            ))
            .into_response();
            (record_id, response)
        }
        AuthorizationOutcome::Allowed { record_id, .. } => {
            let response = match read(state.pool.clone()).await {
                Ok(body) => body.into_response(),
                Err(error) => error.into_response(),
            };
            (record_id, response)
        }
    };
    Ok(([("x-reasonbraid-authorization", record_id)], response).into_response())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationRecordQuery {
    tenant_id: TenantId,
}

/// One known record in the admitted tenant. Its response receipt names a separate
/// inspection admission, never a recursive retrieval of that new admission.
async fn inspect_authorization_record(
    State(state): State<Arc<ApiState>>,
    Path(record_id): Path<reasonbraid_core::AuthorizationRecordId>,
    Query(q): Query<AuthorizationRecordQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::AuthorizationRecord { record_id },
        |pool| async move {
            let record = authority::load_tenant_authorization_record(&pool, q.tenant_id, record_id)
                .await?
                .ok_or_else(|| ControlApiError::not_found("authorization record not found"))?;
            let authorization = serde_json::to_value(record).map_err(|error| {
                ControlApiError::internal_with_log(format!(
                    "authorization record encoding: {error}"
                ))
            })?;
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "authorization": authorization,
            })))
        },
    )
    .await
}

/// The tenant's grants, newest first — the inspection surface the revocation
/// state rides (`tenant_admin`-gated).
async fn list_grants(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Grants {},
        |pool| async move {
            type Row = (
                String,
                String,
                String,
                Value,
                String,
                DateTime<Utc>,
                DateTime<Utc>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT grant_id, subject_kind, subject_id, actions, status, valid_from, expires_at \
         FROM authority_grants WHERE tenant_id = $1 ORDER BY valid_from DESC",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&pool)
    .await?;
            let grants: Vec<Value> = rows
                .into_iter()
                .map(
                    |(
                        grant_id,
                        subject_kind,
                        subject_id,
                        actions,
                        status,
                        valid_from,
                        expires_at,
                    )| {
                        json!({
                            "grant_id": grant_id,
                            "subject_kind": subject_kind,
                            "subject_id": subject_id,
                            "actions": actions,
                            "status": status,
                            "valid_from": valid_from.to_rfc3339(),
                            "expires_at": expires_at.to_rfc3339(),
                        })
                    },
                )
                .collect();
            Ok(Json(
                json!({ "tenant_id": q.tenant_id.to_string(), "grants": grants }),
            ))
        },
    )
    .await
}

/// The tenant's enrollment boundaries — the same inspection contract.
async fn list_boundaries(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Boundaries {},
        |pool| async move {
            type Row = (String, String, String, DateTime<Utc>, DateTime<Utc>);
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT boundary_id, status, target_owner, valid_from, expires_at \
         FROM enrollment_boundaries WHERE tenant_id = $1 ORDER BY valid_from DESC",
            )
            .bind(q.tenant_id.to_string())
            .fetch_all(&pool)
            .await?;
            let boundaries: Vec<Value> = rows
                .into_iter()
                .map(
                    |(boundary_id, status, target_owner, valid_from, expires_at)| {
                        json!({
                            "boundary_id": boundary_id,
                            "status": status,
                            "target_owner": target_owner,
                            "valid_from": valid_from.to_rfc3339(),
                            "expires_at": expires_at.to_rfc3339(),
                        })
                    },
                )
                .collect();
            Ok(Json(
                json!({ "tenant_id": q.tenant_id.to_string(), "boundaries": boundaries }),
            ))
        },
    )
    .await
}

/// The tenant's incarnations (`.1.6.1`; deferral #4's first half) — the §8.1
/// facts each enrolled role node declared, the inspection surface the run
/// writer (`.1.6.2`) links against.
async fn list_incarnations(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Incarnations {},
        |pool| async move {
            type Row = (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<Value>,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT incarnation_id, role_id, provider, model, harness, config, valid_from, valid_to \
         FROM incarnations WHERE tenant_id = $1 ORDER BY valid_from DESC",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&pool)
    .await?;
            let incarnations: Vec<Value> = rows
                .into_iter()
                .map(
                    |(
                        incarnation_id,
                        role_id,
                        provider,
                        model,
                        harness,
                        config,
                        valid_from,
                        valid_to,
                    )| {
                        json!({
                            "incarnation_id": incarnation_id,
                            "role_id": role_id,
                            "provider": provider,
                            "model": model,
                            "harness": harness,
                            "config": config,
                            "valid_from": valid_from.map(|d| d.to_rfc3339()),
                            "valid_to": valid_to.map(|d| d.to_rfc3339()),
                        })
                    },
                )
                .collect();
            Ok(Json(
                json!({ "tenant_id": q.tenant_id.to_string(), "incarnations": incarnations }),
            ))
        },
    )
    .await
}

/// The tenant's runs (`.1.6.2`; deferral #4's second half) — each run links its
/// attempt to the incarnation that ran it, the inspection surface the chain
/// rides.
async fn list_runs(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Runs {},
        |pool| async move {
            type Row = (String, String, String, Option<String>, DateTime<Utc>);
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT r.run_id, r.incarnation_id, i.role_id, r.attempt_id, r.created_at \
         FROM runs r JOIN incarnations i ON i.incarnation_id = r.incarnation_id \
         WHERE r.tenant_id = $1 ORDER BY r.created_at DESC",
            )
            .bind(q.tenant_id.to_string())
            .fetch_all(&pool)
            .await?;
            let runs: Vec<Value> = rows
                .into_iter()
                .map(
                    |(run_id, incarnation_id, role_id, attempt_id, created_at)| {
                        json!({
                            "run_id": run_id,
                            "incarnation_id": incarnation_id,
                            "role_id": role_id,
                            "attempt_id": attempt_id,
                            "created_at": created_at.to_rfc3339(),
                        })
                    },
                )
                .collect();
            Ok(Json(
                json!({ "tenant_id": q.tenant_id.to_string(), "runs": runs }),
            ))
        },
    )
    .await
}

/// The spend-breaker verbs (`.3.2`, backlog 23; tenant_admin): arm (declare
/// the threshold — re-arming clears any trip), reset (clear the trip), and
/// inspect (the armed/tripped state with the reason).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmBreakerRequest {
    pub tenant_id: TenantId,
    /// The spend threshold (BudgetDimensions JSON) the latch compares against.
    pub threshold: reasonbraid_core::BudgetDimensions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreakerTenantRequest {
    pub tenant_id: TenantId,
}

/// Both breaker verbs run ONE guarded transaction (`SIGNOFF-REPAIR.3.3.4.9`):
/// the admission, the tenant-bound breaker selection, the mutation and the final
/// effect record share a single commit under the tenant's exclusive authority
/// guard. Before this, each handler admitted the caller in its own shared-guard
/// transaction and then ran a bare statement ON THE POOL — no transaction, no
/// guard, and no record of what the operation finally did.
///
/// Every answer, including the refusals, carries the
/// `x-reasonbraid-authorization` receipt naming the admission this request
/// committed, which is also the effect record's id when one was written. Without
/// it the effect record would be unreachable: there is no list endpoint.
///
/// The status codes, messages and success bodies below are byte-unchanged. The
/// two states the `409` deliberately collapses — a breaker that is armed but not
/// tripped, and no breaker at all — are distinguished in the effect record
/// instead, which is the fact an admission cannot carry.
async fn run_breaker_administration(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: TenantId,
    command: authority::BreakerCommand,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(headers)?;
    let administration = authority::administer_breaker_in_one_transaction(
        &state.pool,
        &principal,
        tenant_id,
        command,
    )
    .await?;
    let receipt = administration.record_id;
    let response = match administration.result {
        authority::BreakerResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // An already-armed breaker answers exactly as a fresh arm does: the
        // caller asked for a state and has it.
        authority::BreakerResult::Armed | authority::BreakerResult::AlreadyArmed => {
            Json(json!({ "tenant_id": tenant_id.to_string(), "armed": true })).into_response()
        }
        authority::BreakerResult::Reset => {
            Json(json!({ "tenant_id": tenant_id.to_string(), "reset": true })).into_response()
        }
        authority::BreakerResult::NotTripped | authority::BreakerResult::NotArmed => {
            ControlApiError::invalid_transition(
                "no tripped breaker to reset (none armed, or none tripped)",
            )
            .into_response()
        }
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

async fn arm_breaker(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ArmBreakerRequest>,
) -> Result<Response, ControlApiError> {
    run_breaker_administration(
        &state,
        &headers,
        req.tenant_id,
        authority::BreakerCommand::Arm {
            threshold: req.threshold,
        },
    )
    .await
}

async fn reset_breaker(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<BreakerTenantRequest>,
) -> Result<Response, ControlApiError> {
    run_breaker_administration(
        &state,
        &headers,
        req.tenant_id,
        authority::BreakerCommand::Reset,
    )
    .await
}

async fn inspect_breakers(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Breakers {},
        |pool| async move {
            type BreakerRow = (
                serde_json::Value,
                Option<DateTime<Utc>>,
                Option<String>,
                DateTime<Utc>,
            );
            let row: Option<BreakerRow> = sqlx::query_as(
                "SELECT threshold, tripped_at, tripped_reason, armed_at \
             FROM spend_breakers WHERE tenant_id = $1",
            )
            .bind(q.tenant_id.to_string())
            .fetch_optional(&pool)
            .await?;
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "breaker": row.map(
                    |(threshold, tripped_at, tripped_reason, armed_at)| json!({
                        "threshold": threshold,
                        "tripped": tripped_at.is_some(),
                        "tripped_at": tripped_at.map(|d| d.to_rfc3339()),
                        "tripped_reason": tripped_reason,
                        "armed_at": armed_at.to_rfc3339(),
                    })
                ),
            })))
        },
    )
    .await
}

// ── Thread commands ──────────────────────────────────────────────────────────────

/// One prepared command's execution target inside the shared transaction flow.
pub(crate) enum CommandTarget<'a> {
    /// `thread.create` — pure preparation; the thread id is server-assigned.
    Create {
        body: &'a CreateBody,
        /// The resolved profile steps (the ADR-016 composition) — the
        /// create boundary resolved them against the registry.
        workflow_steps: Vec<String>,
    },
    /// A command against an existing thread — validated against the locked
    /// projection inside the transaction.
    Existing {
        thread_id: ThreadId,
        operation: &'a str,
        body: &'a Value,
    },
}

/// The one-transaction command flow, shared by create and thread commands:
/// claim → authorize → prepare → apply (+ ceiling for create). Every step commits
/// or rolls back together; a rejection stores its failure result in the idempotency
/// row so a replay reproduces the ORIGINAL status and body.
pub(crate) async fn run_thread_command(
    pool: &PgPool,
    tenant_id: &TenantId,
    principal: &GrantSubject,
    authz: &CommandAuthz,
    idempotency_key: &str,
    request_hash: &str,
    target: CommandTarget<'_>,
) -> Result<(StatusCode, Value), ControlApiError> {
    let mut tx = pool.begin().await?;

    // 0. The tenant authority guard, FIRST — before the idempotency key, the
    //    aggregate row, the quota rows and the inbox (`SIGNOFF-REPAIR.3.3.4.4`).
    //    Shared, because commands read authority and must run concurrently with
    //    each other; a revocation takes the exclusive mode and therefore fences
    //    every command that has not already reached this point.
    //
    //    Ordering here is not a narrower race window — before this, the command
    //    path took no guard at all, so it had no ordering against revocation to
    //    narrow. A control holds the exclusive guard and watches a command run
    //    to completion underneath it.
    crate::authority::transaction::acquire_in_tx(
        &mut tx,
        *tenant_id,
        crate::authority::transaction::GuardMode::Shared,
    )
    .await?;

    // 1. Idempotency claim: a replay returns the ORIGINAL stored result verbatim —
    //    the domain is not re-validated against state the original may have changed.
    match tx::claim_idempotency_in_tx(
        &mut *tx,
        &tenant_id.to_string(),
        idempotency_key,
        request_hash,
    )
    .await?
    {
        ClaimOutcome::Replay { result } => {
            crate::telemetry::metrics().incr("idempotency_replays");
            tx.commit().await?;
            let mut body = result;
            let status = if body.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                StatusCode::OK
            } else {
                let code = body
                    .get("error")
                    .and_then(|e| e.get("code"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("dependency_unavailable");
                ControlApiError::status_for_code(code)
            };
            if let Some(obj) = body.as_object_mut() {
                obj.insert("replayed".to_string(), json!(true));
            }
            return Ok((status, body));
        }
        ClaimOutcome::Fresh => {}
    }

    // 2. Authorization — the audit record commits with whatever happens next.
    //    An allowance captures what the delivery needs (`.1.5.2`, ADR-008): the
    //    record id + digest + decision time, plus the tenant's epoch AT DECISION
    //    TIME (read in the same transaction — the dispatch hook carries them).
    //    The decision time is DATABASE time sampled here, after the guard and the
    //    idempotency claim have both waited — not the process clock read before
    //    them. A grant that expired during those waits must not be evaluated as
    //    though it were still live at the instant the request arrived.
    let now = crate::authority::transaction::database_now_in_tx(&mut tx).await?;
    let admission = match authorize_in_tx(&mut *tx, authz, now).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            let message = format!("authorization denied ({record_id}): {reason}");
            let err = ControlApiError::unauthorized(message.clone());
            store_rejection(&mut *tx, tenant_id, idempotency_key, &err.failure_result()).await?;
            tx.commit().await?;
            return Err(err);
        }
        AuthorizationOutcome::Allowed {
            record_id,
            policy_digest,
            decided_at,
        } => {
            let revocation_epoch: i64 =
                sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
                    .bind(tenant_id.to_string())
                    .fetch_one(&mut *tx)
                    .await?;
            Some(AdmissionDecision {
                authz_ref: record_id,
                policy_digest,
                decided_at,
                revocation_epoch,
            })
        }
    };

    // 3. Domain validation (the create path is pure; existing-thread commands read
    //    the LOCKED projection inside this transaction).
    let (thread_id, prepared) = match &target {
        CommandTarget::Create {
            body,
            workflow_steps,
        } => {
            let thread_id = ThreadId::new();
            let prepared = threads::prepare_create(
                tenant_id,
                &thread_id,
                &principal.id_string(),
                body,
                workflow_steps.clone(),
            );
            (thread_id, prepared)
        }
        CommandTarget::Existing {
            thread_id,
            operation,
            body,
        } => {
            // The quota refusal (`.1.3.2`) is a RECORDED denial: the event
            // row must COMMIT even though the command is refused — the
            // authorization-denied pattern (store the rejection + commit),
            // never a silent rollback. Other domain refusals keep their
            // rollback semantics (the tx drops; the redelivery re-validates).
            let prepared = match threads::prepare_thread_command(
                &mut *tx,
                tenant_id,
                thread_id,
                operation,
                &principal.id_string(),
                body,
            )
            .await
            {
                Ok(p) => p,
                Err(threads::ThreadError::QuotaRefused(q)) => {
                    let err = ControlApiError::from(threads::ThreadError::QuotaRefused(q));
                    store_rejection(&mut *tx, tenant_id, idempotency_key, &err.failure_result())
                        .await?;
                    tx.commit().await?;
                    return Err(err);
                }
                Err(e) => return Err(e.into()),
            };
            (*thread_id, prepared)
        }
    };

    // 4. The WP2 durability writes + the ceiling (create) in the SAME transaction.
    //    The projection is re-parsed here (from the state `prepare` just built) so
    //    the `.6.2` dispatch hook can read the subject/objective/ceiling without a
    //    second query.
    let event_id = prepared.event_id.to_string();
    let projection: threads::ThreadProjection = serde_json::from_value(prepared.next_state.clone())
        .map_err(|e| {
            eprintln!("control api: freshly prepared projection does not parse: {e}");
            ControlApiError::internal()
        })?;
    let cmd = Command {
        tenant_id: tenant_id.to_string(),
        aggregate_type: threads::AGGREGATE_TYPE.to_string(),
        aggregate_id: thread_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        request_hash: request_hash.to_string(),
        event_id: event_id.clone(),
        event_type: prepared.event_type.to_string(),
        body: prepared.event_body,
        next_state: prepared.next_state,
        result: prepared.result.clone(),
    };
    tx::apply_fresh_in_tx(&mut *tx, &cmd).await?;
    if let Some((ceiling_id, dims)) = &prepared.ceiling {
        budget::create_ceiling_in_tx(
            &mut *tx,
            ceiling_id,
            &tenant_id.to_string(),
            &thread_id.to_string(),
            dims,
        )
        .await?;
    }

    // 5. Inbox dispatch (`.6.2`): an accepted invite/challenge hands work to the
    //    target role's node in the SAME transaction — the reservation (or its
    //    denial row) and the inbox row commit with the thread event, so an
    //    invitation exists iff its work does.
    if let CommandTarget::Existing {
        operation, body, ..
    } = &target
    {
        match *operation {
            // `.1.3.1`: the invite records the invitation ONLY — the work item
            // rides the ACCEPT transaction (an accepted invitation exists iff
            // its work does). A pending invitation enqueues nothing.
            threads::OP_ACCEPT_INVITATION => {
                match &principal {
                    GrantSubject::Role(role) => {
                        dispatch_work_in_tx(
                            &mut *tx,
                            &DispatchSpec {
                                tenant_id,
                                thread_id: &thread_id,
                                trigger_event_id: &event_id,
                                kind: threads::WORK_CONTRIBUTE,
                                agent_role: &role.to_string(),
                                target_event_id: None,
                                projection: &projection,
                                admission: admission
                                    .as_ref()
                                    .expect("the dispatch path runs after the authorization step"),
                            },
                        )
                        .await?;
                    }
                    // A human cannot be invited (invites target agent roles), so
                    // this arm is unreachable by construction — but dispatch
                    // nothing rather than ever guessing a target.
                    GrantSubject::Human(_) => {}
                }
            }
            threads::OP_CHALLENGE => {
                let challenge: threads::ChallengeBody = serde_json::from_value((*body).clone())
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
                let author = threads::event_author_in_thread(
                    &mut *tx,
                    tenant_id,
                    &thread_id,
                    &challenge.target_event_id,
                )
                .await?;
                // A role's contribution is revised by that role's node; a human
                // author revises through the CLI (no node, no dispatch). The
                // revise work targets THIS challenge's event (the domain's
                // `thread.revise` targets a challenge, not the contribution).
                if let Some(author) = author {
                    if author.parse::<AgentRoleId>().is_ok() {
                        dispatch_work_in_tx(
                            &mut *tx,
                            &DispatchSpec {
                                tenant_id,
                                thread_id: &thread_id,
                                trigger_event_id: &event_id,
                                kind: threads::WORK_REVISE,
                                agent_role: &author,
                                target_event_id: Some(event_id.as_str()),
                                projection: &projection,
                                admission: admission
                                    .as_ref()
                                    .expect("the dispatch path runs after the authorization step"),
                            },
                        )
                        .await?;
                    }
                }
            }
            _ => {}
        }
    }
    tx.commit().await?;

    Ok((StatusCode::OK, prepared.result))
}

/// The admission decision metadata a dispatched work item carries (`.1.5.2`,
/// ADR-008): the node caches exactly this and evaluates it at the dispatch
/// boundary against the freshness TTL + the tenant's current revocation epoch.
#[derive(Debug, Clone)]
pub struct AdmissionDecision {
    pub authz_ref: String,
    pub policy_digest: String,
    pub decided_at: chrono::DateTime<chrono::Utc>,
    pub revocation_epoch: i64,
}

/// The dispatch parameters for one work item (the `.6.2` hook's argument bundle).
struct DispatchSpec<'a> {
    tenant_id: &'a TenantId,
    thread_id: &'a ThreadId,
    trigger_event_id: &'a str,
    kind: &'a str,
    agent_role: &'a str,
    target_event_id: Option<&'a str>,
    projection: &'a threads::ThreadProjection,
    /// The admission decision the work item carries (always present — the
    /// dispatch hook runs after the authorization step of the same transaction).
    admission: &'a AdmissionDecision,
}

/// The `.6.2` dispatch body: hand one work item to the target role's node in the
/// caller's transaction. The reservation is best-effort — when the ceiling refuses,
/// the work item is STILL enqueued, without a reservation and with the denial
/// reason, so the node's budget gate journals `failed_before_dispatch` instead of
/// contacting a provider: the denial is visible and bounded at both boundaries.
///
/// Dev rule (recorded in `docs/decisions/2026-09-07_node-channel-wiring.md`): a
/// node id IS the agent role wire id it serves — one node, one role — until the
/// Phase 1 directory exists. The work item's command id correlates it with the
/// thread event that produced it (`work_{event_id}`).
async fn dispatch_work_in_tx<'e, E>(
    mut tx: E,
    spec: &DispatchSpec<'_>,
) -> Result<(), ControlApiError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    // The evaluator-access control (`.1.4.3`, the `.1.4.1` contract): the
    // dispatch is the DECISION POINT — a confidential thread's work delivery
    // requires a confidential-qualified evaluator profile, and the dev
    // registry qualifies none. The typed refusal aborts the delivery in this
    // transaction (the accept/challenge rolls back cleanly) — never a silent
    // general (ADR-034).
    let classification = spec.projection.classification;
    if !classification.has_qualified_evaluator() {
        return Err(ControlApiError::classification_unqualified(format!(
            "the thread's `{}` classification has no qualified evaluator \
             profile in this deployment — the dispatch refuses until one is registered",
            classification.as_str()
        )));
    }
    let (reservation, denial) = match budget::create_reservation_in_tx(
        &mut *tx,
        &spec.projection.ceiling_id,
        &spec.tenant_id.to_string(),
        &spec.thread_id.to_string(),
        &threads::WORK_RESERVATION,
        chrono::Duration::minutes(10),
        Utc::now(),
    )
    .await
    {
        Ok(r) => (Some(r.reference), None),
        Err(BudgetError::Unavailable { detail }) => {
            eprintln!(
                "control api: budget denied a {} dispatch: {detail}",
                spec.kind
            );
            (None, Some(format!("budget denied the dispatch: {detail}")))
        }
        // The dev reservation engine only produces `Unavailable`; any other typed
        // refusal is treated the same way (deny the dispatch, keep the work item
        // visible) rather than ever dispatching without a proof of allowance.
        Err(other) => {
            eprintln!(
                "control api: budget refused a {} dispatch: {other}",
                spec.kind
            );
            (None, Some(format!("budget refused the dispatch: {other}")))
        }
    };
    let payload = threads::work_payload(
        spec.kind,
        spec.agent_role,
        &spec.projection.subject,
        &spec.projection.objective,
        spec.target_event_id,
        reservation.as_ref(),
        denial.as_deref(),
    );
    node_channel::enqueue_in_tx(
        &mut *tx,
        spec.agent_role,
        &format!("work_{}", spec.trigger_event_id),
        &spec.tenant_id.to_string(),
        &spec.thread_id.to_string(),
        &payload,
        &spec.admission.authz_ref,
        &spec.admission.policy_digest,
        spec.admission.decided_at,
        spec.admission.revocation_epoch,
    )
    .await?;
    Ok(())
}

/// The inbox command whose row binds a node event's effect to a tenant, when the
/// payload has one at all. `None` is an ordinary channel receipt: it records the
/// event and changes nothing else, so there is nothing to order and no guard to
/// take.
///
/// ONE classifier, used by `node_channel::events` to resolve the tenant BEFORE
/// any lock is taken and by [`apply_node_result_in_tx`] to apply the effect under
/// that guard — so the guard that is held and the effect that is applied can
/// never disagree about which row binds them (`SIGNOFF-REPAIR.3.3.4.5`).
pub(crate) fn node_result_command(payload: &Value) -> Option<(&str, &str)> {
    let kind = payload.get("kind").and_then(|v| v.as_str())?;
    if kind != "work_result" && kind != "work_dead_lettered" {
        return None;
    }
    let command_id = payload.get("command_id").and_then(|v| v.as_str())?;
    Some((kind, command_id))
}

/// The `.6.2` node-result path (called from the node channel's `events` handler):
/// fold a node-emitted `work_result` into its thread through the SAME
/// claim → authorize → validate → apply flow a CLI command rides. The idempotency
/// key is the work item's inbox command id, so a duplicated transport produces a
/// replay — never a second domain effect. The caller owns the transaction; a
/// rejection is stored as the command's idempotent result and returned as the
/// error (the receipt still commits — the node DID emit this event).
///
/// `guarded_tenant` is the tenant whose authority guard the caller took BEFORE
/// the node's lease row (`SIGNOFF-REPAIR.3.3.4.5`). Every effect below binds its
/// SQL to it, which is the guard module's own contract: a row that changed
/// between the pre-lock resolution and this guarded read matches nothing, so the
/// fold cannot happen under the wrong tenant's guard.
pub(crate) async fn apply_node_result_in_tx(
    tx: &mut sqlx::PgConnection,
    node_id: &str,
    payload: &Value,
    guarded_tenant: TenantId,
) -> Result<(), ControlApiError> {
    let Some((kind, command_id)) = node_result_command(payload) else {
        // Ordinary channel events (WP3) are receipts only — nothing to fold.
        return Ok(());
    };
    // A node's dead-letter report (`.2.4`): auto-quarantine the inbox row WITH
    // the refusal reason — the terminal fact the operator later replays.
    if kind == "work_dead_lettered" {
        let Some(reason) = payload.get("reason").and_then(|v| v.as_str()) else {
            return Ok(()); // malformed report — the receipt stands, no domain effect
        };
        crate::telemetry::metrics().incr("dead_letters");
        sqlx::query(
            "UPDATE node_inbox SET quarantined_at = $4, quarantine_reason = $5 \
             WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3 \
               AND quarantined_at IS NULL",
        )
        .bind(node_id)
        .bind(command_id)
        .bind(guarded_tenant.to_string())
        .bind(Utc::now())
        .bind(format!("dead-lettered: {reason}"))
        .execute(&mut *tx)
        .await?;
        return Ok(());
    }

    // The inbox row binds the result to its tenant/thread scope and work kind —
    // selected BY the guarded tenant, so the row this fold uses is the row whose
    // guard is held.
    let Some((tenant_raw, thread_raw, work)) = node_channel::load_command_for_tenant_in_tx(
        &mut *tx,
        node_id,
        command_id,
        &guarded_tenant.to_string(),
    )
    .await?
    else {
        // Unknown to this node's ledger under this tenant: the receipt stands,
        // no domain effect.
        return Ok(());
    };
    let tenant_id: TenantId = tenant_raw
        .parse()
        .map_err(|_| ControlApiError::internal())?;
    let thread_id: ThreadId = thread_raw
        .parse()
        .map_err(|_| ControlApiError::internal())?;

    let operation = match work.get("kind").and_then(|v| v.as_str()) {
        Some(threads::WORK_CONTRIBUTE) => threads::OP_CONTRIBUTE,
        Some(threads::WORK_REVISE) => threads::OP_REVISE,
        _ => return Ok(()), // not thread work — receipt only
    };

    // Dev rule: the node id IS the agent role wire id it serves.
    let Ok(role) = node_id.parse::<AgentRoleId>() else {
        return Ok(());
    };
    let principal = GrantSubject::Role(role);

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let body = if operation == threads::OP_REVISE {
        let target = work
            .get("target_event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        json!({
            "tenant_id": tenant_id.to_string(),
            "target_event_id": target,
            "content": content,
        })
    } else {
        json!({ "tenant_id": tenant_id.to_string(), "content": content })
    };
    let hash = request_hash(operation, &principal, &body);
    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::ThreadContribute,
        target: ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
    };

    // Claim FIRST: a re-emitted result (original event id, same payload) replays
    // the ORIGINAL stored outcome — the first receipt already applied the work
    // and settled its reservation.
    match tx::claim_idempotency_in_tx(&mut *tx, &tenant_id.to_string(), command_id, &hash).await? {
        ClaimOutcome::Replay { .. } => return Ok(()),
        ClaimOutcome::Fresh => {}
    }

    // DATABASE time, sampled here — after the tenant guard and the idempotency
    // claim have both waited — not the process clock read before them. Authority
    // that ended while this result queued is evaluated as it stands now, which is
    // the same rule the thread-command path adopted in `.3.3.4.4`.
    let now = crate::authority::transaction::database_now_in_tx(&mut *tx).await?;
    match authorize_in_tx(&mut *tx, &authz, now).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            let err = ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            ));
            store_rejection(&mut *tx, &tenant_id, command_id, &err.failure_result()).await?;
            return Err(err);
        }
        AuthorizationOutcome::Allowed { .. } => {}
    }

    // The run writer (`.1.6.2`; deferral #4's second half): the attempt id rides
    // the result payload (the node's local journal fact); the run links it to the
    // role's CURRENT incarnation. The idempotency claim above dedupes redelivery
    // BEFORE this write — one result = one run, ever. A result without an
    // attempt id (or a role with no incarnation) still folds: the run row is
    // best-effort linkage, not a gate.
    if let Some(attempt_id) = payload.get("attempt_id").and_then(|v| v.as_str()) {
        let current_incarnation: Option<String> = sqlx::query_scalar(
            "SELECT incarnation_id FROM incarnations \
             WHERE role_id = $1 AND valid_to IS NULL \
             ORDER BY valid_from DESC NULLS LAST LIMIT 1",
        )
        .bind(node_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(incarnation_id) = current_incarnation {
            sqlx::query(
                "INSERT INTO runs (run_id, incarnation_id, tenant_id, attempt_id) \
                 VALUES ('run_' || gen_random_uuid()::text, $1, $2, $3)",
            )
            .bind(&incarnation_id)
            .bind(tenant_id.to_string())
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    let prepared = match threads::prepare_thread_command(
        &mut *tx,
        &tenant_id,
        &thread_id,
        operation,
        &principal.id_string(),
        &body,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            let err: ControlApiError = e.into();
            store_rejection(&mut *tx, &tenant_id, command_id, &err.failure_result()).await?;
            return Err(err);
        }
    };

    let cmd = Command {
        tenant_id: tenant_id.to_string(),
        aggregate_type: threads::AGGREGATE_TYPE.to_string(),
        aggregate_id: thread_id.to_string(),
        idempotency_key: command_id.to_string(),
        request_hash: hash,
        event_id: prepared.event_id.to_string(),
        event_type: prepared.event_type.to_string(),
        body: prepared.event_body,
        next_state: prepared.next_state,
        result: prepared.result.clone(),
    };
    tx::apply_fresh_in_tx(&mut *tx, &cmd).await?;

    // Settle the reservation with the reported usage (idempotent — a replay or a
    // duplicate event settles nothing twice).
    if let Some(reservation_id) = payload.get("reservation_id").and_then(|v| v.as_str()) {
        if !reservation_id.is_empty() {
            let usage = BudgetDimensions::attempt_usage(
                payload
                    .get("usage")
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(|v| v.as_u64()),
                payload
                    .get("usage")
                    .and_then(|u| u.get("output_tokens"))
                    .and_then(|v| v.as_u64()),
            );
            budget::settle_reservation_in_tx(&mut *tx, reservation_id, &usage, Utc::now()).await?;
        }
    }

    crate::telemetry::metrics().incr("results_folded");
    Ok(())
}

/// Store a rejection as the command's semantic result (idempotent replay of the
/// rejection reproduces the original status + body). The write rides the aggregate
/// library's step 6 (`agg::store_result_in_tx`, `PHASE-1.1.1`).
async fn store_rejection<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    idempotency_key: &str,
    failure: &Value,
) -> Result<(), ControlApiError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    crate::agg::store_result_in_tx(&mut *tx, &tenant_id.to_string(), idempotency_key, failure)
        .await
        .map_err(|e| {
            eprintln!("control api: storing the idempotent rejection failed: {e}");
            ControlApiError::internal()
        })?;
    Ok(())
}

async fn create_thread(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(envelope): Json<CommandEnvelope>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if envelope.protocol_version != PROTOCOL_VERSION {
        return Err(ControlApiError::protocol_incompatible(
            &envelope.protocol_version,
        ));
    }
    if envelope.operation != threads::OP_CREATE {
        return Err(ControlApiError::invalid_command(format!(
            "operation `{}` is not `{}` on the thread-create surface",
            envelope.operation,
            threads::OP_CREATE
        )));
    }
    let mut body: CreateBody = serde_json::from_value(envelope.body.clone())
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    // `.5.2` (ADR-031): the explicit profile always wins; the routing class
    // resolves through the rule table ONLY when no profile is named (the
    // human authority outranks the rule).
    let profile_id = match body.workflow_profile.as_deref() {
        Some(explicit) => Some(explicit.to_owned()),
        None => match body.routing_class.as_deref() {
            Some(class) => {
                let route = crate::routing::resolve(&state.pool, class)
                    .await
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
                crate::routing::record_resolution(
                    &state.pool,
                    &route,
                    &principal.id_string(),
                    "create_boundary",
                )
                .await?;
                Some(route.arm)
            }
            None => None,
        },
    };
    // The ADR-016 boundary: the workflow profile is a VALIDATED reference —
    // the unknown id is the typed refusal (never a stored string), and the
    // canonical id + the resolved steps ride the create onward. The resolve
    // ALWAYS runs: a bare thread defaults to `quick_advice` (the `.3.3`
    // catch — the earlier Some-only call left the bare thread's steps
    // EMPTY, so its step gates read `none`).
    let resolved = crate::workflows::resolve(&state.pool, profile_id.as_deref())
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let workflow_steps = resolved.steps;
    body.workflow_profile = Some(resolved.profile_id);
    let tenant_id = body.tenant_id;
    let hash = request_hash(threads::OP_CREATE, &principal, &envelope.body);
    let (delegate_subject, delegation_scope) = delegation_from_envelope(&envelope)?;
    let authz = CommandAuthz {
        delegation_scope,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject,
        action: GrantAction::ThreadCreate,
        target: ResourceTarget::Tenant { tenant_id },
    };
    let response = run_thread_command(
        &state.pool,
        &tenant_id,
        &principal,
        &authz,
        &envelope.idempotency_key,
        &hash,
        CommandTarget::Create {
            body: &body,
            workflow_steps,
        },
    )
    .await?;
    Ok(json_response(response.0, response.1))
}

#[derive(Debug, Deserialize)]
struct ThreadQuery {
    #[serde(default)]
    tenant_id: Option<String>,
}

async fn thread_command(
    State(state): State<Arc<ApiState>>,
    Path(thread_id_raw): Path<String>,
    headers: HeaderMap,
    Json(envelope): Json<CommandEnvelope>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if envelope.protocol_version != PROTOCOL_VERSION {
        return Err(ControlApiError::protocol_incompatible(
            &envelope.protocol_version,
        ));
    }
    let thread_id: ThreadId = thread_id_raw.parse().map_err(|_| {
        ControlApiError::invalid_command(format!("thread_id `{thread_id_raw}` is malformed"))
    })?;

    // Map the operation to its typed body, authorization action, and tenant scope.
    let (tenant_id, authz_action, hash) = match envelope.operation.as_str() {
        threads::OP_INVITE => {
            let body: threads::InviteBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadInvite,
                request_hash(threads::OP_INVITE, &principal, &envelope.body),
            )
        }
        threads::OP_CONTRIBUTE => {
            let body: threads::ContributeBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(threads::OP_CONTRIBUTE, &principal, &envelope.body),
            )
        }
        threads::OP_ADVANCE_ROUND => {
            let body: threads::AdvanceRoundBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadAdvanceRound,
                request_hash(threads::OP_ADVANCE_ROUND, &principal, &envelope.body),
            )
        }
        threads::OP_CHALLENGE => {
            let body: threads::ChallengeBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(threads::OP_CHALLENGE, &principal, &envelope.body),
            )
        }
        threads::OP_REVISE => {
            let body: threads::ReviseBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(threads::OP_REVISE, &principal, &envelope.body),
            )
        }
        threads::OP_CLOSE => {
            let body: threads::CloseBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadClose,
                request_hash(threads::OP_CLOSE, &principal, &envelope.body),
            )
        }
        threads::OP_CANCEL => {
            let body: threads::CancelBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadCancel,
                request_hash(threads::OP_CANCEL, &principal, &envelope.body),
            )
        }
        threads::OP_ACCEPT_INVITATION => {
            let body: threads::AcceptInvitationBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadInvitationRespond,
                request_hash(threads::OP_ACCEPT_INVITATION, &principal, &envelope.body),
            )
        }
        threads::OP_DECLINE_INVITATION => {
            let body: threads::DeclineInvitationBody =
                serde_json::from_value(envelope.body.clone())
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadInvitationRespond,
                request_hash(threads::OP_DECLINE_INVITATION, &principal, &envelope.body),
            )
        }
        threads::OP_JOIN => {
            let body: threads::JoinBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(threads::OP_JOIN, &principal, &envelope.body),
            )
        }
        threads::OP_REMOVE_PARTICIPANT => {
            let body: threads::RemoveParticipantBody =
                serde_json::from_value(envelope.body.clone())
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::TenantAdmin,
                request_hash(threads::OP_REMOVE_PARTICIPANT, &principal, &envelope.body),
            )
        }
        other => {
            return Err(ControlApiError::invalid_command(format!(
                "unknown thread operation `{other}`"
            )))
        }
    };

    let (delegate_subject, delegation_scope) = delegation_from_envelope(&envelope)?;
    let authz = CommandAuthz {
        delegation_scope,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject,
        action: authz_action,
        // Participant removal requires tenant administration. The domain
        // executor still locks and selects the thread within this same tenant.
        target: match authz_action {
            GrantAction::TenantAdmin => ResourceTarget::Tenant { tenant_id },
            _ => ResourceTarget::Thread {
                tenant_id,
                thread_id,
            },
        },
    };
    let response = run_thread_command(
        &state.pool,
        &tenant_id,
        &principal,
        &authz,
        &envelope.idempotency_key,
        &hash,
        CommandTarget::Existing {
            thread_id,
            operation: &envelope.operation,
            body: &envelope.body,
        },
    )
    .await?;
    Ok(json_response(response.0, response.1))
}

// ── Inspection (the `.6.1` acceptance: no database surgery) ─────────────────────

/// Authorize an inspection read (the audit row is recorded for reads too — the
/// audit view is complete, `.5.1`), then run the read.
async fn inspect<F, Fut>(
    state: &ApiState,
    principal: &GrantSubject,
    target: ResourceTarget,
    read: F,
) -> Result<Response, ControlApiError>
where
    F: FnOnce(PgPool) -> Fut,
    Fut: std::future::Future<Output = Result<Value, ControlApiError>>,
{
    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::ThreadInspect,
        target,
    };
    match authority::authorize_guarded(&state.pool, &authz).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )))
        }
        AuthorizationOutcome::Allowed { .. } => {
            let body = read(state.pool.clone()).await?;
            Ok(json_response(StatusCode::OK, body))
        }
    }
}

fn tenant_from_query(q: &ThreadQuery) -> Result<TenantId, ControlApiError> {
    match &q.tenant_id {
        Some(raw) => raw.parse().map_err(|_| {
            ControlApiError::invalid_command(format!("tenant_id `{raw}` is malformed"))
        }),
        None => Err(ControlApiError::invalid_command(
            "tenant_id query parameter is required",
        )),
    }
}

async fn get_thread(
    State(state): State<Arc<ApiState>>,
    Path(thread_id_raw): Path<String>,
    Query(q): Query<ThreadQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id = tenant_from_query(&q)?;
    let thread_id: ThreadId = thread_id_raw.parse().map_err(|_| {
        ControlApiError::invalid_command(format!("thread_id `{thread_id_raw}` is malformed"))
    })?;

    inspect(
        &state,
        &principal,
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
        |pool| async move {
            let tenant = tenant_id.to_string();
            let thread = thread_id.to_string();
            let claim = tenant.clone();
            let row: Option<(String, Value)> = crate::rls::with_tenant_claim(&pool, &claim, |tx| {
                Box::pin(async move {
                    sqlx::query_as(
                        "SELECT aggregate_id, state FROM aggregate_state \
                     WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_type = 'thread'",
                    )
                    .bind(tenant)
                    .bind(thread)
                    .fetch_optional(&mut *tx)
                    .await
                })
            })
            .await?;
            match row {
                Some((_, state_json)) => {
                    // The inspection view (`.1.3.1`): expiry is derived at read —
                    // `invited` entries past their `expires_at` read as `expired`,
                    // exactly like the channel's lease presence. The stored
                    // projection is untouched.
                    let stored: threads::ThreadProjection = serde_json::from_value(state_json)
                        .map_err(|e| {
                            ControlApiError::internal_with_log(format!(
                                "corrupt stored thread state: {e}"
                            ))
                        })?;
                    let viewed = threads::derived_view(stored);
                    Ok(json!({
                        "thread_id": thread_id.to_string(),
                        "tenant_id": tenant_id.to_string(),
                        "state": serde_json::to_value(&viewed)
                            .expect("the projection serializes"),
                    }))
                }
                None => Err(ControlApiError::scope_hidden()),
            }
        },
    )
    .await
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    #[serde(default)]
    tenant_id: Option<String>,
    #[serde(default)]
    after: Option<i64>,
}

async fn get_events(
    State(state): State<Arc<ApiState>>,
    Path(thread_id_raw): Path<String>,
    Query(q): Query<EventsQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id = tenant_from_query(&ThreadQuery {
        tenant_id: q.tenant_id.clone(),
    })?;
    let thread_id: ThreadId = thread_id_raw.parse().map_err(|_| {
        ControlApiError::invalid_command(format!("thread_id `{thread_id_raw}` is malformed"))
    })?;
    let reader = principal.id_string();

    inspect(
        &state,
        &principal,
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
        |pool| async move {
            let after = q.after.unwrap_or(0);
            let tenant = tenant_id.to_string();
            let thread = thread_id.to_string();
            let claim = tenant.clone();
            type Row = (String, String, i64, DateTime<Utc>, Value);
            let rows: Vec<Row> = crate::rls::with_tenant_claim(&pool, &claim, |tx| {
                Box::pin(async move {
                    sqlx::query_as(
                        "SELECT event_id, event_type, aggregate_version, committed_at, body \
                 FROM event_log \
                 WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_version > $3 \
                 ORDER BY aggregate_version",
                    )
                    .bind(tenant)
                    .bind(thread)
                    .bind(after)
                    .fetch_all(&mut *tx)
                    .await
                })
            })
            .await?;
            let mut events: Vec<Value> = rows
                .into_iter()
                .map(|(event_id, event_type, version, committed_at, body)| {
                    json!({
                        "event_id": event_id,
                        "event_type": event_type,
                        "aggregate_version": version,
                        "committed_at": committed_at,
                        "body": body,
                    })
                })
                .collect();
            // `.2.3` (ADR-029): the blind read-surface rule — a blind
            // contribution reads as digest + marker for any reader who is NOT
            // its author, until the commitment point (a LATER round advance
            // carrying `blind_committed`, or the close/cancel). A read rule,
            // never a store rewrite: the ledger keeps the full body.
            for i in 0..events.len() {
                let event = &events[i];
                let is_blind = event["event_type"] == json!("thread.contribution_submitted")
                    && event["body"]["blind"].as_bool() == Some(true);
                if !is_blind {
                    continue;
                }
                let author = event["body"]["author"].as_str();
                if author == Some(reader.as_str()) {
                    continue;
                }
                let committed = events[i + 1..].iter().any(|later| {
                    later["event_type"] == json!("thread.closed")
                        || later["event_type"] == json!("thread.cancelled")
                        || (later["event_type"] == json!("thread.round_advanced")
                            && later["body"]["blind_committed"].as_bool() == Some(true))
                });
                if committed {
                    continue;
                }
                let content = event["body"]["content"].as_str().unwrap_or_default();
                events[i]["body"] = json!({
                    "blind": true,
                    "blind_until": "round_advance",
                    "content_digest": crate::fetcher::digest_sha256_hex(content.as_bytes()),
                    "author": author,
                    "round": event["body"]["round"].clone(),
                });
            }
            let next_cursor = events
                .last()
                .and_then(|e| e.get("aggregate_version"))
                .and_then(|v| v.as_i64());
            Ok(json!({
                "thread_id": thread_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "events": events,
                "next_cursor": next_cursor,
            }))
        },
    )
    .await
}

/// `GET /v1/admin/metrics` — the operational metrics surface (`.5.2`; ADR-023:
/// OPERATIONAL counters, never audit facts — the audit record is the durable
/// table row). tenant_admin-gated, read-only.
async fn admin_metrics(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // The process-global registry has no single tenant: the gate is "the
    // caller HOLDS the tenant_admin action in ANY of their active grants"
    // (the dev root trust — a tenant admin sees the process's counters).
    let holds_admin: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS( \
             SELECT 1 FROM authority_grants \
             WHERE subject_id = $1 AND status = 'active' \
               AND actions ? 'tenant_admin' \
               AND valid_from <= now() AND (expires_at IS NULL OR expires_at > now()))",
    )
    .bind(principal.id_string())
    .fetch_one(&state.pool)
    .await?;
    if !holds_admin.unwrap_or(false) {
        return Err(ControlApiError::unauthorized(
            "the metrics surface is tenant_admin-gated",
        ));
    }
    let snapshot: serde_json::Map<String, Value> = crate::telemetry::metrics()
        .snapshot()
        .into_iter()
        .map(|(k, v)| (k, json!(v)))
        .collect();
    Ok(Json(Value::Object(snapshot)))
}

/// Preserve the registry response bodies while exposing the committed receipt.
/// All authorization, registry effects and audit persistence happen in the service.
async fn site_registry_response(
    state: &ApiState,
    principal: &GrantSubject,
    command: RegistryCommand,
) -> Result<Response, ControlApiError> {
    match site::execute(&state.pool, principal, &command).await {
        Ok(receipt) => Ok((
            [("x-reasonbraid-site-audit", receipt.audit_id)],
            Json(receipt.result),
        )
            .into_response()),
        Err(site::Error::Refused { reason, audit_id }) => {
            let (status, message) = if reason == "undeclared_region" {
                (
                    StatusCode::BAD_REQUEST,
                    "both regions must be declared before pairing",
                )
            } else {
                crate::telemetry::metrics().incr("authorization_denials");
                (
                    StatusCode::FORBIDDEN,
                    "a current site grant for this action and its actual boundary are required",
                )
            };
            Ok((
                status,
                Json(json!({"code": reason, "message": message, "audit_id": audit_id})),
            )
                .into_response())
        }
        Err(site::Error::InvalidInput(reason)) => Err(ControlApiError::invalid_command(reason)),
        Err(site::Error::OperatorRequired) => {
            Err(ControlApiError::unauthorized("site authority required"))
        }
        Err(site::Error::Sql(error)) => Err(error.into()),
    }
}

// Do not echo malformed caller input or driver diagnostics. Keep body-size and
// media-type refusal statuses; normalize JSON syntax/schema errors to typed 400.
fn site_request<T>(
    request: Result<Json<T>, axum::extract::rejection::JsonRejection>,
) -> Result<T, ControlApiError> {
    request.map(|Json(value)| value).map_err(|rejection| {
        let status = match rejection.status() {
            StatusCode::PAYLOAD_TOO_LARGE => StatusCode::PAYLOAD_TOO_LARGE,
            StatusCode::UNSUPPORTED_MEDIA_TYPE => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::BAD_REQUEST,
        };
        ControlApiError {
            status,
            code: "invalid_command",
            message: "registry request requires the documented JSON fields, bounded names and a nonblank reason".into(),
        }
    })
}

fn site_path<T>(
    path: Result<Path<T>, axum::extract::rejection::PathRejection>,
) -> Result<T, ControlApiError> {
    path.map(|Path(value)| value).map_err(|_| {
        ControlApiError::invalid_command("registry path names must be valid UTF-8 URL components")
    })
}

fn site_name(value: String) -> Result<RegistryName, ControlApiError> {
    RegistryName::new(value).map_err(|error| ControlApiError::invalid_command(error.to_string()))
}

/// `GET /v1/admin/adapters` — site registry inspection, audited atomically.
async fn list_adapters(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    site_registry_response(&state, &principal, RegistryCommand::ListAdapters).await
}

/// `POST /v1/admin/adapters` — the first reason stays on the registry row;
/// every admitted retry records its own reason and no-op receipt.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllowAdapterRequest {
    adapter_id: RegistryName,
    reason: Reason,
}

async fn allow_adapter(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<AllowAdapterRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    site_registry_response(
        &state,
        &principal,
        RegistryCommand::AllowAdapter {
            adapter_id: req.adapter_id,
            reason: req.reason,
        },
    )
    .await
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteReasonRequest {
    reason: Reason,
}

/// `POST /v1/admin/adapters/{adapter_id}/revoke` — remove with a reason.
async fn revoke_adapter(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    path: Result<Path<String>, axum::extract::rejection::PathRejection>,
    request: Result<Json<SiteReasonRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    site_registry_response(
        &state,
        &principal,
        RegistryCommand::RevokeAdapter {
            adapter_id: site_name(site_path(path)?)?,
            reason: req.reason,
        },
    )
    .await
}

/// `GET /v1/admin/regions` — site registry inspection, audited atomically.
async fn list_regions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    site_registry_response(&state, &principal, RegistryCommand::ListRegions).await
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclareRegionRequest {
    region: RegistryName,
    reason: Reason,
}

/// `POST /v1/admin/regions` — declare one region with an attributable reason.
async fn declare_region(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<DeclareRegionRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    site_registry_response(
        &state,
        &principal,
        RegistryCommand::DeclareRegion {
            region: req.region,
            reason: req.reason,
        },
    )
    .await
}

/// `POST /v1/admin/regions/{from}/pair/{to}` — both regions must be declared.
async fn pair_regions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    path: Result<Path<(String, String)>, axum::extract::rejection::PathRejection>,
    request: Result<Json<SiteReasonRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    let (from, to) = site_path(path)?;
    site_registry_response(
        &state,
        &principal,
        RegistryCommand::PairRegions {
            from: site_name(from)?,
            to: site_name(to)?,
            reason: req.reason,
        },
    )
    .await
}

/// `POST /v1/admin/regions/{from}/unpair/{to}` — remove with a reason.
async fn unpair_regions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    path: Result<Path<(String, String)>, axum::extract::rejection::PathRejection>,
    request: Result<Json<SiteReasonRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    let (from, to) = site_path(path)?;
    site_registry_response(
        &state,
        &principal,
        RegistryCommand::UnpairRegions {
            from: site_name(from)?,
            to: site_name(to)?,
            reason: req.reason,
        },
    )
    .await
}

/// `GET /v1/admin/usage?tenant_id=…` — the usage-reconciliation surface
/// (`.3.3`, backlog 25's dev slice): the estimates-vs-receipts picture per
/// tenant, SUMMED over the ledger rows (the same rows the budget engine
/// enforces against — nothing computed or invented here): held (active
/// unexpired reservations), settled (actual usage), overrun (settled minus
/// reserved, per dimension, floored), and the denials with their reasons —
/// plus the per-thread breakdown. tenant_admin-gated (the read carve-out's
/// surface), read-only.
/// `GET /v1/audit/receipts?tenant_id=…` — the tenant's cross-domain
/// receipts (the read surface, tenant_admin-gated): each receipt names
/// the remote domain's digest-pinned reference + the local record it
/// attached to — the cross-reference, never the merged chain.
async fn list_cross_domain_receipts(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, q.tenant_id).await?;
    let receipts = crate::receipts::list(&state.pool, &q.tenant_id.to_string()).await?;
    Ok(Json(
        json!({ "tenant_id": q.tenant_id.to_string(), "receipts": receipts }),
    ))
}

async fn admin_usage(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Usage {},
        |pool| async move {
            type Row = (
                String,
                Value,
                Option<Value>,
                String,
                Option<String>,
                Option<DateTime<Utc>>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT thread_id, dimensions, usage, status, reason, expires_at \
         FROM budget_reservations WHERE tenant_id = $1 ORDER BY created_at",
            )
            .bind(q.tenant_id.to_string())
            .fetch_all(&pool)
            .await?;

            let mut tenant_held = BudgetDimensions::default();
            let mut tenant_settled = BudgetDimensions::default();
            let mut tenant_overrun = BudgetDimensions::default();
            let mut tenant_denied = 0usize;
            let mut denial_reasons: Vec<&str> = Vec::new();
            let mut threads: std::collections::BTreeMap<
                String,
                (BudgetDimensions, BudgetDimensions, BudgetDimensions, usize),
            > = std::collections::BTreeMap::new();

            for (thread_id, dimensions, usage, status, reason, expires_at) in &rows {
                let reserved: BudgetDimensions =
                    serde_json::from_value(dimensions.clone()).expect("stored dims parse");
                let (held, settled, overrun, denied) =
                    threads.entry(thread_id.clone()).or_insert_with(|| {
                        (
                            BudgetDimensions::default(),
                            BudgetDimensions::default(),
                            BudgetDimensions::default(),
                            0,
                        )
                    });
                match status.as_str() {
                    "active" if expires_at.is_some_and(|e| e > Utc::now()) => {
                        *held = held.add(&reserved);
                        tenant_held = tenant_held.add(&reserved);
                    }
                    "settled" => {
                        let used: BudgetDimensions = usage
                            .as_ref()
                            .map(|u| {
                                serde_json::from_value(u.clone()).expect("stored usage parses")
                            })
                            .unwrap_or_default();
                        *settled = settled.add(&used);
                        tenant_settled = tenant_settled.add(&used);
                        // The overrun per dimension = used minus reserved, floored at
                        // None (an unused remainder is NOT a negative overrun).
                        let over = BudgetDimensions {
                            calls: used
                                .calls
                                .zip(reserved.calls)
                                .map(|(u, r)| u.saturating_sub(r)),
                            input_tokens: used
                                .input_tokens
                                .zip(reserved.input_tokens)
                                .map(|(u, r)| u.saturating_sub(r)),
                            output_tokens: used
                                .output_tokens
                                .zip(reserved.output_tokens)
                                .map(|(u, r)| u.saturating_sub(r)),
                            wall_clock_seconds: used
                                .wall_clock_seconds
                                .zip(reserved.wall_clock_seconds)
                                .map(|(u, r)| u.saturating_sub(r)),
                        };
                        *overrun = overrun.add(&over);
                        tenant_overrun = tenant_overrun.add(&over);
                    }
                    "denied" => {
                        *denied += 1;
                        tenant_denied += 1;
                        if let Some(reason) = reason.as_deref() {
                            denial_reasons.push(reason);
                        }
                    }
                    _ => {} // released/expired: nothing held, nothing settled
                }
            }

            let dims_to_json = |d: BudgetDimensions| {
                json!({
                    "calls": d.calls,
                    "input_tokens": d.input_tokens,
                    "output_tokens": d.output_tokens,
                    "wall_clock_seconds": d.wall_clock_seconds,
                })
            };
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "aggregate": {
                    "held": dims_to_json(tenant_held),
                    "settled": dims_to_json(tenant_settled),
                    "overrun": dims_to_json(tenant_overrun),
                    "denied": tenant_denied,
                    "denial_reasons": denial_reasons,
                },
                "threads": threads
                    .into_iter()
                    .map(|(thread_id, (held, settled, overrun, denied))| {
                        json!({
                            "thread_id": thread_id,
                            "held": dims_to_json(held),
                            "settled": dims_to_json(settled),
                            "overrun": dims_to_json(overrun),
                            "denied": denied,
                        })
                    })
                    .collect::<Vec<_>>(),
            })))
        },
    )
    .await
}

/// `GET /v1/threads/{thread_id}/budget` — the budget read surface (`PHASE-1.6.1`,
/// the `.1.6` census finding: budgets had NO read surface anywhere). A READ-ONLY
/// view of the ledger facts for one thread: the ceiling (dimensions, policy
/// version, created time) plus every reservation row — held vs settled usage,
/// denials with their reasons, release facts. The rows are the §14.3 ledger;
/// nothing is computed or invented here, so "spend and uncertainty are visible"
/// (`ROADMAP.md` §26.1) follows the same rows the budget engine enforces against.
/// Gated by the same `thread_inspect` path as `get_thread`: a role without the
/// grant is a typed 403 with the audit row.
async fn get_thread_budget(
    State(state): State<Arc<ApiState>>,
    Path(thread_id_raw): Path<String>,
    Query(q): Query<ThreadQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id = tenant_from_query(&q)?;
    let thread_id: ThreadId = thread_id_raw.parse().map_err(|_| {
        ControlApiError::invalid_command(format!("thread_id `{thread_id_raw}` is malformed"))
    })?;

    inspect(
        &state,
        &principal,
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
        |pool| async move {
            let ceiling: Option<(String, Value, String, DateTime<Utc>)> = sqlx::query_as(
                "SELECT ceiling_id, dimensions, policy_version, created_at \
                 FROM budget_ceilings WHERE tenant_id = $1 AND thread_id = $2",
            )
            .bind(tenant_id.to_string())
            .bind(thread_id.to_string())
            .fetch_optional(&pool)
            .await?;
            // BUDGET-003: a thread exists with its ceiling — a thread row without
            // one is corrupt, not an empty budget; refusing hides the anomaly
            // rather than presenting a fabricated `{}` budget.
            let Some((ceiling_id, dimensions, policy_version, created_at)) = ceiling else {
                return Err(ControlApiError::scope_hidden());
            };

            type Row = (
                String,
                Value,
                Option<Value>,
                String,
                Option<String>,
                Option<DateTime<Utc>>,
                DateTime<Utc>,
                Option<DateTime<Utc>>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT reservation_id, dimensions, usage, status, reason, expires_at, \
                 created_at, settled_at FROM budget_reservations \
                 WHERE ceiling_id = $1 ORDER BY created_at",
            )
            .bind(&ceiling_id)
            .fetch_all(&pool)
            .await?;
            // Absent optional facts are OMITTED on the wire (the `.1.5.1` shape
            // discipline) — never `null`.
            let reservations: Vec<Value> = rows
                .into_iter()
                .map(
                    |(
                        reservation_id,
                        dimensions,
                        usage,
                        status,
                        reason,
                        expires_at,
                        created_at,
                        settled_at,
                    )| {
                        let mut row = serde_json::Map::new();
                        row.insert("reservation_id".into(), json!(reservation_id));
                        row.insert("dimensions".into(), dimensions);
                        if let Some(usage) = usage {
                            row.insert("usage".into(), usage);
                        }
                        row.insert("status".into(), json!(status));
                        if let Some(reason) = reason {
                            row.insert("reason".into(), json!(reason));
                        }
                        if let Some(expires_at) = expires_at {
                            row.insert("expires_at".into(), json!(expires_at.to_rfc3339()));
                        }
                        row.insert("created_at".into(), json!(created_at.to_rfc3339()));
                        if let Some(settled_at) = settled_at {
                            row.insert("settled_at".into(), json!(settled_at.to_rfc3339()));
                        }
                        Value::Object(row)
                    },
                )
                .collect();

            Ok(json!({
                "thread_id": thread_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "ceiling": {
                    "ceiling_id": ceiling_id,
                    "dimensions": dimensions,
                    "policy_version": policy_version,
                    "created_at": created_at.to_rfc3339(),
                },
                "reservations": reservations,
            }))
        },
    )
    .await
}

async fn get_audit(
    State(state): State<Arc<ApiState>>,
    Path(thread_id_raw): Path<String>,
    Query(q): Query<ThreadQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id = tenant_from_query(&q)?;
    let thread_id: ThreadId = thread_id_raw.parse().map_err(|_| {
        ControlApiError::invalid_command(format!("thread_id `{thread_id_raw}` is malformed"))
    })?;

    inspect(
        &state,
        &principal,
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
        |pool| async move {
            let rows =
                authority::load_thread_authorization_records(&pool, tenant_id, thread_id).await?;
            let records: Vec<Value> = rows
                .into_iter()
                .map(|record| {
                    let reason = match &record.decision {
                        reasonbraid_core::Decision::Allowed => None,
                        reasonbraid_core::Decision::Denied { reason } => Some(reason),
                    };
                    json!({
                        "record_id": record.record_id,
                        "actor": record.actor,
                        "action": record.action,
                        "decision": record.decision.as_str(),
                        "reason": reason,
                        "policy_digest": record.policy_digest,
                        "decided_at": record.decided_at,
                        "evaluation": record.evaluation,
                    })
                })
                .collect();
            Ok(json!({
                "thread_id": thread_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "records": records,
            }))
        },
    )
    .await
}

async fn list_threads(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<ThreadQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let tenant_id = tenant_from_query(&q)?;

    inspect(
        &state,
        &principal,
        ResourceTarget::Tenant { tenant_id },
        |pool| async move {
            let tenant = tenant_id.to_string();
            let claim = tenant.clone();
            let rows: Vec<(String, Value)> = crate::rls::with_tenant_claim(&pool, &claim, |tx| {
                Box::pin(async move {
                    sqlx::query_as(
                        "SELECT aggregate_id, state FROM aggregate_state \
                     WHERE tenant_id = $1 AND aggregate_type = 'thread' ORDER BY aggregate_id",
                    )
                    .bind(tenant)
                    .fetch_all(&mut *tx)
                    .await
                })
            })
            .await?;
            let threads: Vec<Value> = rows
                .into_iter()
                .map(|(thread_id, state_json)| {
                    let subject = state_json
                        .get("subject")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let state = state_json
                        .get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    json!({
                        "thread_id": thread_id,
                        "subject": subject,
                        "state": state,
                    })
                })
                .collect();
            Ok(json!({
                "tenant_id": tenant_id.to_string(),
                "threads": threads,
            }))
        },
    )
    .await
}
