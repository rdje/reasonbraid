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

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
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
    self, authorize, authorize_in_tx, AuthorizationOutcome, CommandAuthz, GrantRefused,
};
use crate::budget;
use crate::node_channel;
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
fn request_hash(operation: &str, principal: &GrantSubject, body: &Value) -> String {
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
}

impl ApiState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// The control API router (`§9.4` Phase 0 subset).
pub fn api_router(pool: PgPool) -> Router {
    let state = Arc::new(ApiState::new(pool));
    Router::new()
        .route("/v1/enrollments", post(enroll))
        .route("/v1/nodes/enroll-tokens", post(issue_node_enroll_token))
        .route("/v1/nodes/quarantine", post(quarantine_command))
        .route("/v1/nodes/replay", post(replay_command))
        .route("/v1/nodes/inbox", get(inspect_node_inbox))
        .route("/v1/nodes/inbox/prune", post(prune_node_inbox))
        .route("/v1/nodes/revoke", post(revoke_node))
        .route("/v1/admin/grants/{grant_id}/revoke", post(revoke_grant))
        .route(
            "/v1/admin/boundaries/{boundary_id}/revoke",
            post(revoke_boundary),
        )
        .route("/v1/admin/grants", get(list_grants))
        .route("/v1/admin/boundaries", get(list_boundaries))
        .route("/v1/admin/incarnations", get(list_incarnations))
        .route("/v1/admin/runs", get(list_runs))
        .route(
            "/v1/admin/breakers",
            post(arm_breaker).get(inspect_breakers),
        )
        .route("/v1/admin/breakers/reset", post(reset_breaker))
        .route("/v1/admin/usage", get(admin_usage))
        .route("/v1/threads", post(create_thread))
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
    /// `"human"` or `"role"`.
    pub kind: String,
    pub name: String,
    /// For a role: the granted actions (wire names). Default: `[thread_contribute]`.
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
    /// `true` when (tenant, kind, name) already exists: the ORIGINAL principal id is
    /// returned; nothing new was created.
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
    let now = Utc::now();

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

    // Replay: the same (tenant, kind, name) returns the ORIGINAL principal id.
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT principal_id, kind FROM enrollments \
         WHERE tenant_id = $1 AND kind = $2 AND name = $3",
    )
    .bind(tenant_id.to_string())
    .bind(kind)
    .bind(&req.name)
    .fetch_optional(&state.pool)
    .await?;
    if let Some((principal_id, stored_kind)) = existing {
        return Ok(Json(EnrollResponse {
            tenant_id: tenant_id.to_string(),
            principal_id,
            kind: stored_kind,
            name: req.name,
            boundary_id: None,
            grant_id: None,
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
    // known for a role — the dev profile uses the role's principal as a stand-in
    // issuer handle for audit purposes (recorded limitation).
    let issuer = match principal {
        GrantSubject::Human(h) => h,
        GrantSubject::Role(_) => HumanPrincipalId::new(),
    };

    // ONE transaction: boundary (new tenants), grant, enrollment row — all or none.
    let mut tx = state.pool.begin().await?;
    let mut boundary = None;
    if req.tenant_id.is_none() {
        let b = dev_boundary(&tenant_id, now);
        // The tenant's identity row FIRST: the human_principals insert below
        // references it (PHASE-1.1.2, migrations/0007).
        sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
            .bind(tenant_id.to_string())
            .execute(&mut *tx)
            .await?;
        authority::insert_boundary_in_tx(&mut *tx, &b).await?;
        boundary = Some(b);
    }

    let boundary_ref = match &boundary {
        Some(b) => b.clone(),
        None => authority::load_active_boundary_for_tenant(&state.pool, &tenant_id)
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
    authority::create_grant_in_tx(&mut *tx, &grant)
        .await
        .map_err(|GrantRefused { violations }| {
            ControlApiError::invalid_command(format!(
                "the dev grant exceeds its boundary: {}",
                violations
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ))
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
            .execute(&mut *tx)
            .await?;
        }
        GrantSubject::Role(r) => {
            sqlx::query("INSERT INTO agent_roles (role_id, tenant_id, name) VALUES ($1, $2, $3)")
                .bind(r.to_string())
                .bind(tenant_id.to_string())
                .bind(&req.name)
                .execute(&mut *tx)
                .await?;
        }
    }

    sqlx::query(
        "INSERT INTO enrollments (principal_id, tenant_id, kind, name) VALUES ($1, $2, $3, $4)",
    )
    .bind(principal.id_string())
    .bind(tenant_id.to_string())
    .bind(kind)
    .bind(&req.name)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(EnrollResponse {
        tenant_id: tenant_id.to_string(),
        principal_id: principal.id_string(),
        kind: kind.to_string(),
        name: req.name,
        boundary_id: boundary.as_ref().map(|b| b.boundary_id.clone()),
        grant_id: Some(grant.grant_id),
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

/// Issue a one-time node enrollment token. `tenant_admin` authority only — the
/// decision is audited by [`authorize`], and the token row is the issuance record.
async fn issue_node_enroll_token(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<IssueNodeTokenRequest>,
) -> Result<Json<IssueNodeTokenResponse>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if !crate::node_channel::is_valid_node_identity(&req.node_id) {
        return Err(ControlApiError::invalid_command(format!(
            "node_id `{}` is not a valid node identity (a `nod_…` node id or the `rol_…` \
             role wire id the dev profile serves)",
            req.node_id
        )));
    }

    let authz = CommandAuthz {
        delegation_scope: None,
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegate_subject: None,
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant {
            tenant_id: req.tenant_id,
        },
    };
    match authorize(&state.pool, &authz, Utc::now()).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            return Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )))
        }
        AuthorizationOutcome::Allowed { .. } => {}
    }

    let now = Utc::now();
    let ttl = chrono::Duration::seconds(req.ttl_seconds.unwrap_or(3600));
    // The token id and nonce are server-generated, opaque, and unguessable
    // (`gen_random_uuid()`; no client-chosen fields). One UNUSED token per node
    // (the 0008 unique index): a re-issue while one is outstanding is a TYPED
    // refusal, never a database error on the wire.
    let issued: Result<(String, String, chrono::DateTime<Utc>), sqlx::Error> = sqlx::query_as(
        "INSERT INTO node_enrollment_tokens \
         (token_id, tenant_id, node_id, host_claim, nonce, expires_at) \
         VALUES ('ntk_' || gen_random_uuid()::text, $1, $2, $3, gen_random_uuid()::text, $4) \
         RETURNING token_id, nonce, expires_at",
    )
    .bind(req.tenant_id.to_string())
    .bind(&req.node_id)
    .bind(&req.host_claim)
    .bind(now + ttl)
    .fetch_one(&state.pool)
    .await;
    let (token_id, nonce, expires_at) = match issued {
        Ok(row) => row,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(ControlApiError {
                status: StatusCode::CONFLICT,
                code: "invalid_command",
                message: format!(
                    "an unused enrollment token for node `{}` already exists — \
                     consume or expire it before issuing another",
                    req.node_id
                ),
            })
        }
        Err(e) => return Err(e.into()),
    };

    Ok(Json(IssueNodeTokenResponse {
        token_id,
        nonce,
        expires_at: expires_at.to_rfc3339(),
    }))
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
    match authorize(pool, &authz, Utc::now()).await? {
        AuthorizationOutcome::Denied { reason, record_id } => Err(ControlApiError::unauthorized(
            format!("authorization denied ({record_id}): {reason}"),
        )),
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

async fn replay_command(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ReplayRequest>,
) -> Result<Json<ReplayResponse>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;

    let mut tx = state.pool.begin().await?;
    // Only a DEAD-LETTERED command replays: quarantine is the terminal the
    // replay reverses; a live command re-delivered twice would double-dispatch.
    let quarantined: Option<Option<DateTime<Utc>>> = sqlx::query_scalar(
        "SELECT quarantined_at FROM node_inbox WHERE node_id = $1 AND command_id = $2",
    )
    .bind(&req.node_id)
    .bind(&req.command_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(existing) = quarantined else {
        return Err(ControlApiError::not_found(format!(
            "no command `{}` in node `{}`'s inbox",
            req.command_id, req.node_id
        )));
    };
    if existing.is_none() {
        return Err(ControlApiError::invalid_transition(
            "the command is not dead-lettered — replay only reverses a quarantine",
        ));
    }

    // The fresh admission decision (`.1.5.2` shape): the CURRENT revocation
    // epoch at replay time, the decision clock restarted. The record + digest
    // stay bound to the original admission.
    let revocation_epoch: i64 =
        sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
            .bind(req.tenant_id.to_string())
            .fetch_one(&mut *tx)
            .await?;

    let affected = sqlx::query(
        "UPDATE node_inbox SET \
           quarantined_at = NULL, \
           quarantine_reason = NULL, \
           decided_at = $3, \
           revocation_epoch = $4, \
           cursor = (SELECT COALESCE(MAX(cursor), 0) + 1 FROM node_inbox WHERE node_id = $1) \
         WHERE node_id = $1 AND command_id = $2 AND quarantined_at IS NOT NULL",
    )
    .bind(&req.node_id)
    .bind(&req.command_id)
    .bind(Utc::now())
    .bind(revocation_epoch)
    .execute(&mut *tx)
    .await?;
    if affected.rows_affected() != 1 {
        return Err(ControlApiError::invalid_transition(
            "the command is already replayed (a concurrent replay won)",
        ));
    }

    tx.commit().await?;
    Ok(Json(ReplayResponse {
        node_id: req.node_id,
        command_id: req.command_id,
        replayed_at: Utc::now().to_rfc3339(),
    }))
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

async fn quarantine_command(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<QuarantineRequest>,
) -> Result<Json<QuarantineResponse>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    if req.reason.trim().is_empty() {
        return Err(ControlApiError::invalid_command(
            "the quarantine reason is required (a quarantine without a reason is a silent skip)",
        ));
    }

    // The row is the serialization point: quarantine only a command that exists
    // in THIS node's inbox, and only once (a re-quarantine is a typed refusal,
    // like the token re-issue).
    let existing: Option<Option<DateTime<Utc>>> = sqlx::query_scalar(
        "SELECT quarantined_at FROM node_inbox WHERE node_id = $1 AND command_id = $2",
    )
    .bind(&req.node_id)
    .bind(&req.command_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(existing) = existing else {
        return Err(ControlApiError::invalid_command(format!(
            "no command `{}` in node `{}`'s inbox",
            req.command_id, req.node_id
        )));
    };
    if let Some(at) = existing {
        return Err(ControlApiError::invalid_transition(format!(
            "the command is already quarantined ({at})"
        )));
    }

    let quarantined: Option<DateTime<Utc>> = sqlx::query_scalar(
        "UPDATE node_inbox SET quarantined_at = $3, quarantine_reason = $4 \
         WHERE node_id = $1 AND command_id = $2 AND quarantined_at IS NULL \
         RETURNING quarantined_at",
    )
    .bind(&req.node_id)
    .bind(&req.command_id)
    .bind(Utc::now())
    .bind(req.reason.trim())
    .fetch_optional(&state.pool)
    .await?;
    let Some(at) = quarantined else {
        // A concurrent quarantine won the race.
        return Err(ControlApiError::invalid_transition(
            "the command is already quarantined",
        ));
    };

    Ok(Json(QuarantineResponse {
        node_id: req.node_id,
        command_id: req.command_id,
        quarantined_at: at.to_rfc3339(),
    }))
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
    }
    let rows: Vec<InboxRowRow> = sqlx::query_as(
        "SELECT cursor, command_id, thread_id, payload, acknowledged_at, quarantined_at, quarantine_reason \
         FROM node_inbox WHERE node_id = $1 ORDER BY cursor",
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
            })
            .collect(),
    }))
}

/// The `POST /v1/nodes/inbox/prune` body: the retention window — DELIVERED rows
/// (acknowledged by the node) at least this old are deleted. Cleanup is an
/// explicit, measured operator action; nothing sweeps on its own.
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

async fn prune_node_inbox(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<PruneInboxRequest>,
) -> Result<Json<PruneInboxResponse>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    if req.min_age_seconds < 0 {
        return Err(ControlApiError::invalid_command(
            "min_age_seconds must be >= 0",
        ));
    }
    // The measured before/after rides ONE transaction: the count, the delete,
    // and the recount see a consistent ledger, and the response is the
    // operator's receipt for exactly what was removed.
    let mut tx = state.pool.begin().await?;
    let cutoff = Utc::now() - chrono::Duration::seconds(req.min_age_seconds);
    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_inbox WHERE node_id = $1")
        .bind(&req.node_id)
        .fetch_one(&mut *tx)
        .await?;
    let deleted = sqlx::query(
        "DELETE FROM node_inbox \
         WHERE node_id = $1 AND acknowledged_at IS NOT NULL AND acknowledged_at <= $2",
    )
    .bind(&req.node_id)
    .bind(cutoff)
    .execute(&mut *tx)
    .await?
    .rows_affected() as i64;
    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_inbox WHERE node_id = $1")
        .bind(&req.node_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(PruneInboxResponse {
        deleted,
        before,
        after,
        cutoff_at: cutoff.to_rfc3339(),
    }))
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

async fn revoke_node(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<RevokeNodeRequest>,
) -> Result<Json<RevokeNodeResponse>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    if req.reason.trim().is_empty() {
        return Err(ControlApiError::invalid_command(
            "the revocation reason is required (a revocation without a reason is a silent skip)",
        ));
    }

    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE node_id = $1 AND tenant_id = $2)",
    )
    .bind(&req.node_id)
    .bind(req.tenant_id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    if !exists.unwrap_or(false) {
        return Err(ControlApiError::not_found(format!(
            "no enrolled node `{}` in this tenant",
            req.node_id
        )));
    }

    let revoked_at = Utc::now();
    // The cert revocation + the epoch bump commit together (`.1.5.2`, ADR-008):
    // a node-side cached decision is invalidated the moment the revocation is
    // durable — the handshake ladder refuses the certs, the epoch refuses the
    // cache, and neither can observe a window where one landed without the other.
    let mut tx = state.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE node_certificates SET revoked_at = $2 \
         WHERE node_id = $1 AND revoked_at IS NULL",
    )
    .bind(&req.node_id)
    .bind(revoked_at)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ControlApiError::invalid_transition(format!(
            "node `{}` has no active certificate to revoke",
            req.node_id
        )));
    }
    authority::bump_revocation_epoch(&mut tx, &req.tenant_id.to_string()).await?;
    tx.commit().await?;

    Ok(Json(RevokeNodeResponse {
        node_id: req.node_id,
        revoked_certificates: result.rows_affected() as i64,
        revoked_at: revoked_at.to_rfc3339(),
    }))
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

async fn revoke_grant(
    State(state): State<Arc<ApiState>>,
    Path(grant_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RevokeAuthorityRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    if req.reason.trim().is_empty() {
        return Err(ControlApiError::invalid_command(
            "the revocation reason is required (a revocation without a reason is a silent skip)",
        ));
    }
    let Some((tenant, previous)) = authority::revoke_grant(&state.pool, &grant_id).await? else {
        return Err(ControlApiError::not_found(format!(
            "no grant `{grant_id}` in this tenant"
        )));
    };
    if tenant != req.tenant_id.to_string() {
        return Err(ControlApiError::not_found(format!(
            "no grant `{grant_id}` in this tenant"
        )));
    }
    if previous == Some(GrantStatus::Revoked) {
        return Err(ControlApiError::invalid_transition(format!(
            "grant `{grant_id}` is already revoked"
        )));
    }
    Ok(Json(json!({
        "grant_id": grant_id,
        "tenant_id": tenant,
        "revoked_at": Utc::now().to_rfc3339(),
    })))
}

async fn revoke_boundary(
    State(state): State<Arc<ApiState>>,
    Path(boundary_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RevokeAuthorityRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    if req.reason.trim().is_empty() {
        return Err(ControlApiError::invalid_command(
            "the revocation reason is required (a revocation without a reason is a silent skip)",
        ));
    }
    let Some((tenant, previous)) = authority::revoke_boundary(&state.pool, &boundary_id).await?
    else {
        return Err(ControlApiError::not_found(format!(
            "no boundary `{boundary_id}` in this tenant"
        )));
    };
    if tenant != req.tenant_id.to_string() {
        return Err(ControlApiError::not_found(format!(
            "no boundary `{boundary_id}` in this tenant"
        )));
    }
    if previous == Some(BoundaryStatus::Revoked) {
        return Err(ControlApiError::invalid_transition(format!(
            "boundary `{boundary_id}` is already revoked"
        )));
    }
    Ok(Json(json!({
        "boundary_id": boundary_id,
        "tenant_id": tenant,
        "revoked_at": Utc::now().to_rfc3339(),
    })))
}

/// The admin READ authorization (the `.1.3.2` lists): the principal's OWN
/// grant must carry `tenant_admin` and be within its window — the boundary
/// ceiling is deliberately NOT required, so inspection survives a boundary
/// revocation (the freeze stops writes, never the operator's eyes). The
/// dev-profile root trust is the bootstrap human's grant.
async fn authorize_tenant_admin_read(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
) -> Result<(), ControlApiError> {
    let (subject_kind, subject_id) = match principal {
        GrantSubject::Human(h) => ("human", h.to_string()),
        GrantSubject::Role(r) => ("role", r.to_string()),
    };
    let row: Option<(Value, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT actions, valid_from, expires_at FROM authority_grants \
         WHERE tenant_id = $1 AND subject_kind = $2 AND subject_id = $3 AND status = 'active' \
         ORDER BY valid_from DESC LIMIT 1",
    )
    .bind(tenant_id.to_string())
    .bind(subject_kind)
    .bind(subject_id)
    .fetch_optional(pool)
    .await?;
    let Some((actions, valid_from, expires_at)) = row else {
        return Err(ControlApiError::unauthorized(
            "authorization denied: no applicable grant".to_string(),
        ));
    };
    let now = Utc::now();
    let in_window = valid_from <= now && now <= expires_at;
    let has_admin = actions
        .as_array()
        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("tenant_admin")));
    if in_window && has_admin {
        Ok(())
    } else {
        Err(ControlApiError::unauthorized(
            "authorization denied: the tenant_admin grant is revoked or outside its window"
                .to_string(),
        ))
    }
}

/// The tenant's grants, newest first — the inspection surface the revocation
/// state rides (`tenant_admin`-gated).
async fn list_grants(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;
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
    .fetch_all(&state.pool)
    .await?;
    let grants: Vec<Value> = rows
        .into_iter()
        .map(
            |(grant_id, subject_kind, subject_id, actions, status, valid_from, expires_at)| {
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
}

/// The tenant's enrollment boundaries — the same inspection contract.
async fn list_boundaries(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;
    type Row = (String, String, String, DateTime<Utc>, DateTime<Utc>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT boundary_id, status, target_owner, valid_from, expires_at \
         FROM enrollment_boundaries WHERE tenant_id = $1 ORDER BY valid_from DESC",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&state.pool)
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
}

/// The tenant's incarnations (`.1.6.1`; deferral #4's first half) — the §8.1
/// facts each enrolled role node declared, the inspection surface the run
/// writer (`.1.6.2`) links against.
async fn list_incarnations(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;
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
    .fetch_all(&state.pool)
    .await?;
    let incarnations: Vec<Value> = rows
        .into_iter()
        .map(
            |(incarnation_id, role_id, provider, model, harness, config, valid_from, valid_to)| {
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
}

/// The tenant's runs (`.1.6.2`; deferral #4's second half) — each run links its
/// attempt to the incarnation that ran it, the inspection surface the chain
/// rides.
async fn list_runs(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;
    type Row = (String, String, String, Option<String>, DateTime<Utc>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT r.run_id, r.incarnation_id, i.role_id, r.attempt_id, r.created_at \
         FROM runs r JOIN incarnations i ON i.incarnation_id = r.incarnation_id \
         WHERE r.tenant_id = $1 ORDER BY r.created_at DESC",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&state.pool)
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

async fn arm_breaker(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ArmBreakerRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    sqlx::query(
        "INSERT INTO spend_breakers (tenant_id, threshold, tripped_at, tripped_reason) \
         VALUES ($1, $2, NULL, NULL) \
         ON CONFLICT (tenant_id) DO UPDATE SET \
           threshold = EXCLUDED.threshold, tripped_at = NULL, tripped_reason = NULL",
    )
    .bind(req.tenant_id.to_string())
    .bind(serde_json::to_value(req.threshold).expect("threshold serializes"))
    .execute(&state.pool)
    .await?;
    Ok(Json(
        json!({ "tenant_id": req.tenant_id.to_string(), "armed": true }),
    ))
}

async fn reset_breaker(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<BreakerTenantRequest>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, req.tenant_id).await?;
    let reset = sqlx::query(
        "UPDATE spend_breakers SET tripped_at = NULL, tripped_reason = NULL \
         WHERE tenant_id = $1 AND tripped_at IS NOT NULL",
    )
    .bind(req.tenant_id.to_string())
    .execute(&state.pool)
    .await?;
    if reset.rows_affected() == 0 {
        return Err(ControlApiError::invalid_transition(
            "no tripped breaker to reset (none armed, or none tripped)",
        ));
    }
    Ok(Json(
        json!({ "tenant_id": req.tenant_id.to_string(), "reset": true }),
    ))
}

async fn inspect_breakers(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;
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
    .fetch_optional(&state.pool)
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
}

// ── Thread commands ──────────────────────────────────────────────────────────────

/// One prepared command's execution target inside the shared transaction flow.
enum CommandTarget<'a> {
    /// `thread.create` — pure preparation; the thread id is server-assigned.
    Create { body: &'a CreateBody },
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
async fn run_thread_command(
    pool: &PgPool,
    tenant_id: &TenantId,
    principal: &GrantSubject,
    authz: &CommandAuthz,
    idempotency_key: &str,
    request_hash: &str,
    target: CommandTarget<'_>,
) -> Result<Response, ControlApiError> {
    let mut tx = pool.begin().await?;

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
            return Ok(json_response(status, body));
        }
        ClaimOutcome::Fresh => {}
    }

    // 2. Authorization — the audit record commits with whatever happens next.
    //    An allowance captures what the delivery needs (`.1.5.2`, ADR-008): the
    //    record id + digest + decision time, plus the tenant's epoch AT DECISION
    //    TIME (read in the same transaction — the dispatch hook carries them).
    let now = Utc::now();
    let admission = match authorize_in_tx(&mut *tx, authz, now).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
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
    let (thread_id, prepared) = match target {
        CommandTarget::Create { body } => {
            let thread_id = ThreadId::new();
            let prepared =
                threads::prepare_create(tenant_id, &thread_id, &principal.id_string(), body);
            (thread_id, prepared)
        }
        CommandTarget::Existing {
            thread_id,
            operation,
            body,
        } => {
            let prepared = threads::prepare_thread_command(
                &mut *tx,
                tenant_id,
                &thread_id,
                operation,
                &principal.id_string(),
                body,
            )
            .await?;
            (thread_id, prepared)
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

    Ok(json_response(StatusCode::OK, prepared.result))
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

/// The `.6.2` node-result path (called from the node channel's `events` handler):
/// fold a node-emitted `work_result` into its thread through the SAME
/// claim → authorize → validate → apply flow a CLI command rides. The idempotency
/// key is the work item's inbox command id, so a duplicated transport produces a
/// replay — never a second domain effect. The caller owns the transaction; a
/// rejection is stored as the command's idempotent result and returned as the
/// error (the receipt still commits — the node DID emit this event).
pub(crate) async fn apply_node_result_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    payload: &Value,
) -> Result<(), ControlApiError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    // A node's dead-letter report (`.2.4`): auto-quarantine the inbox row WITH
    // the refusal reason — the terminal fact the operator later replays.
    if payload.get("kind").and_then(|v| v.as_str()) == Some("work_dead_lettered") {
        let (Some(command_id), Some(reason)) = (
            payload.get("command_id").and_then(|v| v.as_str()),
            payload.get("reason").and_then(|v| v.as_str()),
        ) else {
            return Ok(()); // malformed report — the receipt stands, no domain effect
        };
        sqlx::query(
            "UPDATE node_inbox SET quarantined_at = $3, quarantine_reason = $4 \
             WHERE node_id = $1 AND command_id = $2 AND quarantined_at IS NULL",
        )
        .bind(node_id)
        .bind(command_id)
        .bind(Utc::now())
        .bind(format!("dead-lettered: {reason}"))
        .execute(&mut *tx)
        .await?;
        return Ok(());
    }
    // Ordinary channel events (WP3) are receipts only — nothing to fold.
    if payload.get("kind").and_then(|v| v.as_str()) != Some("work_result") {
        return Ok(());
    }
    let Some(command_id) = payload.get("command_id").and_then(|v| v.as_str()) else {
        return Ok(());
    };

    // The inbox row binds the result to its tenant/thread scope and work kind.
    let Some((tenant_raw, thread_raw, work)) =
        node_channel::load_command_in_tx(&mut *tx, node_id, command_id).await?
    else {
        // Unknown to this node's ledger: the receipt stands, no domain effect.
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

    let now = Utc::now();
    match authorize_in_tx(&mut *tx, &authz, now).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
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
    let body: CreateBody = serde_json::from_value(envelope.body.clone())
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
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
    run_thread_command(
        &state.pool,
        &tenant_id,
        &principal,
        &authz,
        &envelope.idempotency_key,
        &hash,
        CommandTarget::Create { body: &body },
    )
    .await
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
        target: ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
    };
    run_thread_command(
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
    .await
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
    match authorize(&state.pool, &authz, Utc::now()).await? {
        AuthorizationOutcome::Denied { reason, record_id } => Err(ControlApiError::unauthorized(
            format!("authorization denied ({record_id}): {reason}"),
        )),
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
            let row: Option<(String, Value)> = sqlx::query_as(
                "SELECT aggregate_id, state FROM aggregate_state \
             WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_type = 'thread'",
            )
            .bind(tenant_id.to_string())
            .bind(thread_id.to_string())
            .fetch_optional(&pool)
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

    inspect(
        &state,
        &principal,
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
        |pool| async move {
            let after = q.after.unwrap_or(0);
            type Row = (String, String, i64, DateTime<Utc>, Value);
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT event_id, event_type, aggregate_version, committed_at, body \
             FROM event_log \
             WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_version > $3 \
             ORDER BY aggregate_version",
            )
            .bind(tenant_id.to_string())
            .bind(thread_id.to_string())
            .bind(after)
            .fetch_all(&pool)
            .await?;
            let events: Vec<Value> = rows
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

/// `GET /v1/admin/usage?tenant_id=…` — the usage-reconciliation surface
/// (`.3.3`, backlog 25's dev slice): the estimates-vs-receipts picture per
/// tenant, SUMMED over the ledger rows (the same rows the budget engine
/// enforces against — nothing computed or invented here): held (active
/// unexpired reservations), settled (actual usage), overrun (settled minus
/// reserved, per dimension, floored), and the denials with their reasons —
/// plus the per-thread breakdown. tenant_admin-gated (the read carve-out's
/// surface), read-only.
async fn admin_usage(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin_read(&state.pool, &principal, q.tenant_id).await?;

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
    .fetch_all(&state.pool)
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
                    .map(|u| serde_json::from_value(u.clone()).expect("stored usage parses"))
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
            type Row = (
                String,
                String,
                String,
                String,
                Option<String>,
                String,
                DateTime<Utc>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT record_id, actor, action, decision, reason, policy_digest, decided_at \
             FROM authorization_records \
             WHERE tenant_id = $1 AND target_kind = 'thread' AND target_thread = $2 \
             ORDER BY decided_at",
            )
            .bind(tenant_id.to_string())
            .bind(thread_id.to_string())
            .fetch_all(&pool)
            .await?;
            let records: Vec<Value> = rows
                .into_iter()
                .map(
                    |(record_id, actor, action, decision, reason, policy_digest, decided_at)| {
                        json!({
                            "record_id": record_id,
                            "actor": actor,
                            "action": action,
                            "decision": decision,
                            "reason": reason,
                            "policy_digest": policy_digest,
                            "decided_at": decided_at,
                        })
                    },
                )
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
            let rows: Vec<(String, Value)> = sqlx::query_as(
                "SELECT aggregate_id, state FROM aggregate_state \
             WHERE tenant_id = $1 AND aggregate_type = 'thread' ORDER BY aggregate_id",
            )
            .bind(tenant_id.to_string())
            .fetch_all(&pool)
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
