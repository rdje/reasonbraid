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
    actor_handle_for_subject, AgentRoleId, AuthorityContext, BoundaryStatus, BudgetDimensions,
    BudgetError, CommandEnvelope, EnrollmentAuthorityBoundary, GrantAction, GrantStatus,
    GrantSubject, HumanPrincipalId, ResourceTarget, RiskClass, TargetSelector, TenantId, ThreadId,
    PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::authority::{
    self, authorize_in_tx, AuthorizationOutcome, CommandAuthz, Delegation, GrantCreateError,
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

    /// Stored content that does not match its record (`SIGNOFF-REPAIR.9.3.5.2`):
    /// the §9.8 `publication_conflict` code. A server-side integrity failure —
    /// the request was well formed and the content is refused, not served.
    pub fn publication_conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "publication_conflict",
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

    /// The deployment declares NO publication repository root
    /// (`SIGNOFF-REPAIR.9.2.1.1`, fail-closed) — the publish verb refuses
    /// until an operator configures one. The same shape as
    /// [`Self::quota_unconfigured`], and for the same reason: this is a
    /// deployment gap, never the caller's request, so a client retrying with
    /// a different path gets nowhere.
    pub fn publication_repository_unconfigured(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "publication_repository_unconfigured",
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
        // The domain refusals' wording lives in ONE place, so the response and
        // the administrative effect record that describes the same refusal
        // cannot drift apart (`SIGNOFF-REPAIR.3.3.4.11.3`).
        match authority::grant_refusal_message(&error, refusal_context) {
            Some(message) => Self::invalid_command(message),
            None => match error {
                GrantCreateError::Storage(error) => error.into(),
                GrantCreateError::Transaction(error) => error.into(),
                refusal => Self::internal_with_log(format!(
                    "an unrendered grant refusal reached the HTTP surface: {refusal}"
                )),
            },
        }
    }
}

impl From<crate::evaluation::EvaluationError> for ControlApiError {
    /// ⛔ **A STORE FAULT IS THE SERVER'S, NOT THE CALLER'S**
    /// (`SIGNOFF-REPAIR.8.2.2`). Every one of the seven evaluation handlers used
    /// to write `invalid_command(error.to_string())` by hand, so a connection
    /// loss reached the submitter as HTTP 400 *already exists* or *the corpus is
    /// not registered*. ⭐ The mapping lives HERE, once: seven copies of a
    /// decision is how one of them ends up disagreeing with the other six.
    fn from(error: crate::evaluation::EvaluationError) -> Self {
        match error {
            crate::evaluation::EvaluationError::Storage(cause) => {
                storage_failure(cause, "the evaluation store failed")
            }
            refusal => Self::invalid_command(refusal.to_string()),
        }
    }
}

impl ControlApiError {
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
            "publication_conflict" => StatusCode::CONFLICT,
            UNREPRESENTABLE_INPUT => StatusCode::BAD_REQUEST,
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

/// Did this write fail because the INPUT holds a character the store cannot
/// represent (`SIGNOFF-REPAIR.4.4.10.1`)? PostgreSQL keeps no U+0000: `jsonb`
/// refuses the escape (`22P05`, *unsupported Unicode escape sequence*) and
/// `text` the byte (`22021`, *invalid byte sequence for encoding "UTF8": 0x00*).
/// That is a property of the caller's input, as permanent as the input is, so
/// it must never be answered as the store being unavailable: a node reads a
/// 500 as transient, retries the same bytes, and never recovers.
pub(crate) fn unrepresentable_input(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|code| code == "22P05" || code == "22021")
}

/// A domain store's failure as the caller hears it (`SIGNOFF-REPAIR.4.4.10.1.1`):
/// input the store cannot hold is the caller's `400 unrepresentable_input`, and
/// anything else is the server's `500`, with `context` and the cause logged.
/// The one decision for every storage error that reaches a handler as a
/// `sqlx::Error` inside a domain error, so no handler builds its own `500` and
/// loses the distinction again.
pub(crate) fn storage_failure(cause: sqlx::Error, context: &str) -> ControlApiError {
    if unrepresentable_input(&cause) {
        return cause.into();
    }
    ControlApiError::internal_with_log(format!("{context}: {cause}"))
}

/// The caller's `400 unrepresentable_input`, for an error type that classified
/// its store error where the SQLSTATE was still readable.
fn unrepresentable() -> ControlApiError {
    ControlApiError {
        status: StatusCode::BAD_REQUEST,
        code: UNREPRESENTABLE_INPUT,
        message: UNREPRESENTABLE_INPUT_MESSAGE.to_string(),
    }
}

/// The caller's answer when [`unrepresentable_input`] holds.
pub(crate) const UNREPRESENTABLE_INPUT: &str = "unrepresentable_input";

/// Its message: which character, and that resending cannot succeed.
pub(crate) const UNREPRESENTABLE_INPUT_MESSAGE: &str =
    "the input holds a character the store cannot represent (U+0000); \
     the same input will be refused again";

impl From<sqlx::Error> for ControlApiError {
    fn from(e: sqlx::Error) -> Self {
        if unrepresentable_input(&e) {
            return unrepresentable();
        }
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
            // Through the one classification, so input the store cannot hold is
            // the caller's refusal here too (`SIGNOFF-REPAIR.4.4.10.1`).
            ApplyError::Sql(e) => e.into(),
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
            e @ threads::ThreadError::ModeratorRestricted { .. } => {
                ControlApiError::unauthorized(e.to_string())
            }
            threads::ThreadError::QuotaRefused(q) => match q {
                crate::quota::QuotaError::Unconfigured { .. } => {
                    ControlApiError::quota_unconfigured(q.to_string())
                }
                crate::quota::QuotaError::Exceeded { .. } => {
                    ControlApiError::quota_exceeded(q.to_string())
                }
                crate::quota::QuotaError::Storage(cause) => {
                    storage_failure(cause, "quota storage failure")
                }
            },
        }
    }
}

// ── Principal resolution (dev profile) ───────────────────────────────────────────

/// Resolve the presented principal from the trusted dev header. A missing or
/// malformed value is `unauthenticated` — the dev profile trusts the header, but it
/// must still be WELL-FORMED and typed.
///
/// `pub(crate)` for `node_channel::presence` (`SIGNOFF-REPAIR.3.5.5`), which had
/// no principal at all. It reads the header the same way rather than parsing it a
/// second time: a second copy of this rule would drift from this one, and the two
/// routes disagreeing about who may read `node_presence` is the defect that leaf
/// exists to close.
pub(crate) fn resolve_principal(headers: &HeaderMap) -> Result<GrantSubject, ControlApiError> {
    let value = headers
        .get(PRINCIPAL_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ControlApiError::unauthenticated(format!(
                "missing `{PRINCIPAL_HEADER}` header (dev profile: hpr_… | rol_…)"
            ))
        })?;
    parse_principal(value).ok_or_else(|| {
        ControlApiError::unauthenticated(format!(
            "malformed `{PRINCIPAL_HEADER}` value `{value}` (expected hpr_… | rol_…)"
        ))
    })
}

/// A principal id as the control API spells one: `hpr_…` (a human) or `rol_…`
/// (an agent role). One definition for the header and for a body field that
/// names a principal (`deployments::register_target`'s reporter).
pub(crate) fn parse_principal(value: &str) -> Option<GrantSubject> {
    if let Ok(human) = value.parse::<HumanPrincipalId>() {
        return Some(GrantSubject::Human(human));
    }
    value.parse::<AgentRoleId>().ok().map(GrantSubject::Role)
}

/// Resolve the delegation from the envelope's `authority_context` (`.1.4.2`,
/// ADR-009 — chain-in-envelope): the subject (the authority source) and the
/// requested scope ride together in one [`Delegation`]. The actor keeps its own
/// identity (the caller check in the dual evaluation).
fn delegation_from_envelope(
    envelope: &CommandEnvelope,
) -> Result<Option<Delegation>, ControlApiError> {
    let Some(ctx) = &envelope.authority_context else {
        return Ok(None);
    };
    // `SIGNOFF-REPAIR.3.4.1.1`: the subject and the scope leave here as ONE
    // value. This was already the only producer and it always set both, so the
    // pairing changes no behaviour — it removes the option of forgetting.
    if let Ok(human) = ctx.on_behalf_of.parse::<HumanPrincipalId>() {
        return Ok(Some(Delegation {
            subject: GrantSubject::Human(human),
            scope: ctx.scope.clone(),
        }));
    }
    if let Ok(role) = ctx.on_behalf_of.parse::<AgentRoleId>() {
        return Ok(Some(Delegation {
            subject: GrantSubject::Role(role),
            scope: ctx.scope.clone(),
        }));
    }
    Err(ControlApiError::invalid_command(format!(
        "malformed `on_behalf_of` value `{}` (expected hpr_… | rol_…)",
        ctx.on_behalf_of
    )))
}

/// The canonical idempotency request hash: operation + presented principal +
/// canonical (struct-field-order) body JSON, and the authority context when the
/// request carries one. SHA-256 hex.
///
/// ⛔ The authority context is part of the hash because the idempotency claim is
/// made BEFORE authorization, and a replay returns the stored result without
/// evaluating anything (`SIGNOFF-REPAIR.3.4.2`). Measured on the superseded
/// shape: a first request delegating to a live role answered `200`, and a second
/// with the same key and body delegating to a REVOKED role answered
/// `200 replayed=true` and wrote **zero** authorization records — an
/// unauthorized delegation told it had succeeded, with no record that it was
/// ever attempted. `authority_context` is a sibling of `body` in the envelope,
/// so hashing the body alone could not see it.
///
/// ⚠️ A request with NO authority context hashes EXACTLY as before — the
/// delegated suffix is appended only when there is one. That is deliberate: the
/// hash is a STORED value, and every historical undelegated key must keep
/// replaying. A historical DELEGATED key now conflicts instead of replaying,
/// which is the safe direction (it refuses rather than returning someone else's
/// result) and is documented as the wire change it is.
/// The canonical request hash: a WIRE CONTRACT, because the value is stored and
/// a later request is compared against it.
///
/// It binds the operation, the actor, the body, the authority context the
/// request was made under (`SIGNOFF-REPAIR.3.4.2`) and the TARGET the command
/// acts on (`SIGNOFF-REPAIR.3.4.6`).
///
/// ⛔ `target` is `Some` exactly where the CALLER can vary the target
/// independently of the idempotency key — which is the eleven operations of
/// `thread_command`, whose thread arrives as a path segment while the typed
/// bodies carry only `tenant_id`. Every other caller passes `None` and says
/// why at its own call site: a creation has no thread yet, and the two
/// server-keyed surfaces already fix the thread inside the key itself.
///
/// Each optional part is appended only when present, so a request that has
/// neither hashes byte-identically to the original two-part shape and its
/// historical key still replays. The two suffixes cannot be confused: the
/// authority context serializes as a JSON object and the target line is
/// `target=` followed by the id.
pub(crate) fn request_hash(
    operation: &str,
    principal: &GrantSubject,
    body: &Value,
    authority: Option<&AuthorityContext>,
    target: Option<&str>,
) -> String {
    let mut input = format!(
        "{operation}\n{}\n{}",
        principal.describe(),
        serde_json::to_string(body).expect("canonical body serializes")
    );
    if let Some(context) = authority {
        input.push('\n');
        input.push_str(
            &serde_json::to_string(context).expect("the typed authority context serializes"),
        );
    }
    if let Some(target) = target {
        input.push_str("\ntarget=");
        input.push_str(target);
    }
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
    /// The publication repository ROOT this deployment declares
    /// (`SIGNOFF-REPAIR.9.2.1.1`): the one directory the publish verb may
    /// write inside. `None` is the fail-closed default — the verb refuses
    /// rather than opening whatever path the request body names.
    publication_repo_root: Option<std::path::PathBuf>,
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
            publication_repo_root: None,
        }
    }

    /// The publication seam (`SIGNOFF-REPAIR.9.2.1.1`): the deployment
    /// declares the ONE repository root the publish verb may write inside.
    ///
    /// Every other constructor leaves it unset, so a state built any other way
    /// refuses that verb instead of trusting the caller's path. That is the
    /// direction the default has to fail in: a deployment that never thought
    /// about publication storage must not be publishing anywhere.
    pub fn with_publication_root(
        pool: PgPool,
        publication_repo_root: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            publication_repo_root,
            ..Self::new(pool)
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

/// The publication seam's router (`SIGNOFF-REPAIR.9.2.1.1`): the same router
/// over a state that declares where a publication may be written. `api_router`
/// keeps its signature — every one of its callers is untouched, which is why
/// the root arrives through a seam rather than through its argument list — and
/// it declares no root, which CLOSES the publish verb rather than leaving it
/// open. Re-derive that caller set with
/// `git grep -n "api_router(" -- ':(glob)crates/**/*.rs'` rather than trusting
/// a count written here.
pub fn api_router_with_publication_root(
    pool: PgPool,
    publication_repo_root: Option<std::path::PathBuf>,
) -> Router {
    api_router_with_state(Arc::new(ApiState::with_publication_root(
        pool,
        publication_repo_root,
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
        .route("/v1/admin/storm-refusals", get(list_storm_refusals))
        .route(
            "/v1/admin/nodes/ambiguous-attempts/adjudicate",
            post(adjudicate_ambiguous_attempt),
        )
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
        .route(
            "/v1/admin/nodes/ambiguous-attempts",
            get(list_ambiguous_attempts),
        )
        .route(
            "/v1/admin/resolution-refusals",
            get(list_resolution_refusals),
        )
        .route("/v1/admin/incidents", get(list_active_incidents))
        .route("/v1/directory/presence", get(directory_presence))
        .route("/v1/directory/match", post(directory_match))
        .route("/v1/calls", post(open_recruitment_call))
        .route("/v1/calls/offered", get(list_offered_calls))
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
        .route("/v1/governance-charters", post(register_governance_charter))
        .route(
            "/v1/governance-charters/{charter_digest}",
            get(read_governance_charter),
        )
        .route("/v1/decision-rules/{rule}", get(tenant_decision_rule))
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
            "/v1/policy-bundles/{manifest_digest}",
            get(read_policy_bundle),
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
            "/v1/deployments/{target_id}/{publication_id}/receipts",
            get(list_deployment_receipts),
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
            get(get_snapshot).delete(withdraw_snapshot_citation),
        )
        .route(
            "/v1/snapshots/{snapshot_id}/tombstone",
            post(tombstone_snapshot),
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
    /// For a role: the grant's spend limit, `{"amount": N}`, checked against
    /// the boundary's spend ceiling at issuance. Refused for a human.
    #[serde(default)]
    pub spend_limits: Option<Value>,
    /// For a role holding `thread_create_auto`: the grant's typed bounds
    /// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1`). Refused for a human, and refused
    /// on a role without the action — bounds that bind nothing are not stored.
    #[serde(default)]
    pub auto_bounds: Option<reasonbraid_core::AutoBounds>,
    /// For a role that creates threads: the decision rules a thread it creates
    /// may declare, by charter wire name, narrower than the tenant's charter
    /// (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.1`). Refused for a human, and refused on
    /// a role without a create action.
    #[serde(default)]
    pub decision_rule_constraints: Option<Vec<String>>,
    /// §4.2's `conditions[]` (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`): the typed
    /// conditions every admission under this grant must satisfy — today
    /// `{"kind": "within_hours", "window": "HH:MM-HH:MM"}`. Refused for a
    /// human (the dev admin set carries no bound), refused empty, refused
    /// malformed; an unknown kind is refused by the wire type itself.
    #[serde(default)]
    pub conditions: Option<Vec<reasonbraid_core::GrantCondition>>,
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
///
/// ⭐ The five administrative verbs join in `SIGNOFF-REPAIR.9.3.4.1`, and they
/// are listed EXPLICITLY although `tenant_admin` already subsumes them. The
/// boundary is *visible to the enrolled target* (this module's authority
/// header), and a ceiling that relies on an implicit rule to explain what it
/// permits is less honest than one that lists it. Subsumption then does
/// exactly one job — the rows written before these names existed — which is
/// what it is for.
///
/// ⛔ `ThreadCreateAuto` is still excluded, and its absence is the reason this
/// is a DECISION and not a default: node-initiated thread creation is never
/// implied (§11.5), so the set is 14 of the 15 registered actions.
const ADMIN_ACTIONS: [GrantAction; 14] = [
    GrantAction::ThreadCreate,
    GrantAction::ThreadInvite,
    GrantAction::ThreadContribute,
    GrantAction::ThreadInspect,
    GrantAction::ThreadClose,
    GrantAction::ThreadCancel,
    GrantAction::ThreadInvitationRespond,
    GrantAction::ThreadAdvanceRound,
    GrantAction::TenantAdmin,
    GrantAction::PolicyVersionRegister,
    GrantAction::PolicyProposalApprove,
    GrantAction::PolicyPublicationWrite,
    GrantAction::PolicyCorrectionRecord,
    GrantAction::DeploymentTargetRegister,
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

// The eighth parameter is the fourth optional bound a grant may carry
// (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`); the two callers pass all four by name in
// the grant's own field order. A bundle struct is the shape to reach for when a
// fifth arrives, not a reason to split the builder in two now.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dev_grant(
    boundary: &EnrollmentAuthorityBoundary,
    issuer: HumanPrincipalId,
    subject: GrantSubject,
    actions: Vec<GrantAction>,
    spend_limits: Option<Value>,
    auto_bounds: Option<reasonbraid_core::AutoBounds>,
    decision_rule_constraints: Option<Vec<String>>,
    conditions: Option<Vec<reasonbraid_core::GrantCondition>>,
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
        spend_limits,
        auto_bounds,
        decision_rule_constraints,
        conditions,
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

    // The auto grant's bounds have a producer
    // (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1`): this body, the one dev-profile
    // issuer that already declares a role's actions. A human's dev grant is the
    // admin set and carries no bound. What the bounds may say is checked once,
    // in `create_grant_in_guard`, with the boundary's own checks.
    if kind == "human"
        && (req.spend_limits.is_some()
            || req.auto_bounds.is_some()
            || req.decision_rule_constraints.is_some()
            || req.conditions.is_some())
    {
        return Err(ControlApiError::invalid_command(
            "spend_limits, auto_bounds, decision_rule_constraints and conditions belong to a \
             role's grant — a human's dev grant carries the admin set and no bound",
        ));
    }
    let grant = dev_grant(
        &boundary_ref,
        issuer,
        principal.clone(),
        actions,
        req.spend_limits.clone(),
        req.auto_bounds.clone(),
        req.decision_rule_constraints.clone(),
        req.conditions.clone(),
    );
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
            // A role is also an INITIATOR (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1`):
            // its identity row implies its initiation bound, which
            // `POST /v1/threads/auto` checks fail-closed. Only a role gets the
            // row — a human cannot initiate autonomously.
            crate::quota::insert_initiator_default_in_tx(
                tx.connection(tenant_id, GuardMode::Exclusive)?,
                &tenant_id.to_string(),
                &r.to_string(),
            )
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

/// The `POST /v1/nodes/enroll-tokens` body: a principal holding `TenantAdmin`
/// issues a ONE-TIME enrollment token bound to tenant + expected node id + host
/// claim + expiry + nonce (`ROADMAP.md` §16.2). The node consumes it at
/// `POST /v1/nodes/enroll`.
///
/// # The issuer is a GRANT, not a kind of principal (`SIGNOFF-REPAIR.4.1.4`)
///
/// This said "an authorized **human**" and the code never checked. Measured
/// across the server's 117 `resolve_principal` call sites, authorization never
/// depends on the principal's kind: every branch on `GrantSubject::Human`
/// selects which identity TABLE to read. §16.4 specifies authorization over
/// typed actions and resources, deny-by-default, and §16.3 states outright
/// that "a human, service, or agent may delegate a strict subset of its own
/// authority" — so the code was consistent with the roadmap and this sentence
/// was the outlier.
///
/// ⚠️ The consequence is stated rather than left implied: **an agent role
/// granted `tenant_admin` can extend the node population.** That is the
/// grant's meaning, and it is asserted by
/// `an_agent_role_holding_tenant_admin_may_issue_an_enrollment_token`, which
/// also asserts a role WITHOUT the grant is refused `403` and writes no token.
/// Narrowing who may issue is a change to the GRANT model, not a kind check
/// bolted onto this route.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueNodeTokenRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub host_claim: String,
    /// Default [`DEFAULT_TOKEN_TTL_SECONDS`]; validated against
    /// [`MAX_TOKEN_TTL_SECONDS`] before anything computes with it
    /// (`SIGNOFF-REPAIR.4.1.2.1`).
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
}

/// The dev profile's default token lifetime: one hour. A consumed token has no
/// second use, so the lifetime bounds only the window in which an unredeemed
/// token is a live bearer credential.
pub const DEFAULT_TOKEN_TTL_SECONDS: i64 = 3_600;

/// The longest lifetime the route will issue (`SIGNOFF-REPAIR.4.1.2.1`).
///
/// One day, chosen so an operator can prepare a node ahead of a working day and
/// no further: beyond that the answer is to issue a fresh token, not to hold a
/// long-lived one. The value matters because the token is a BEARER credential —
/// `.4.1.2` reasons about its exposure from the premise that it expires, and an
/// unvalidated `i64` made that premise vacuous. Measured before the cap existed,
/// an admin asking for a century received `expires_at: 2126-09-14` and a `200`.
pub const MAX_TOKEN_TTL_SECONDS: i64 = 86_400;

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

    // `SIGNOFF-REPAIR.4.1.2.1`: the lifetime arrives as a caller-chosen `i64` and
    // is validated HERE, beside the node-id shape check and before anything
    // computes with it. Three measured behaviours made that placement the fix
    // rather than a tidy-up, and they are different failures:
    //
    //  * `i64::MAX` PANICKED the handler — `TimeDelta::seconds` is documented to
    //    panic above `i64::MAX / 1_000` — and it did so BEFORE any authorization,
    //    because `resolve_principal` only parses the header. The caller reached
    //    past the gate and dropped the connection.
    //  * `1_000_000_000_000_000` panicked later and worse: `at + ttl` leaves
    //    chrono's date range, and that addition runs INSIDE the tenant's
    //    authority guard.
    //  * A century was simply ISSUED, `200`, expiring in 2126 — a bearer
    //    credential outliving everyone who could reason about it.
    //
    // A range check answers all three the same way: a typed refusal, before the
    // arithmetic, so no value the caller chooses is ever handed to it.
    let ttl_seconds = req.ttl_seconds.unwrap_or(DEFAULT_TOKEN_TTL_SECONDS);
    if !(1..=MAX_TOKEN_TTL_SECONDS).contains(&ttl_seconds) {
        return Err(ControlApiError::invalid_command(format!(
            "ttl_seconds must be between 1 and {MAX_TOKEN_TTL_SECONDS} \
             (one day); got {ttl_seconds}"
        )));
    }

    // `SIGNOFF-REPAIR.4.1.6`: the host claim is checked HERE, beside the node-id
    // shape and the lifetime range, because this is where a human typed it.
    //
    // Measured before the placement was chosen: a claim the certificate library
    // refuses is accepted at issuance, and the PANIC lands on the node redeeming
    // the token — which returns a transport error rather than an answer, leaves
    // the token unconsumed, and so leaves that node id unenrollable until the
    // token lapses (`.4.1.1`'s supersede is what bounds it to the TTL rather
    // than for ever). Refusing at issuance costs the administrator one corrected
    // field and makes that state unreachable.
    //
    // ⛔ The rule is the library's OWN verdict (`ca::check_host_claim`), not a
    // grammar restated here: this moves the refusal earlier without narrowing
    // which claims are acceptable, so no deployment loses a claim it can use
    // today. Whether a STRICTER grammar should bind is a separate decision with
    // its own compatibility question, and `.4.1.6` records it rather than taking
    // it in a repair.
    if let Err(refusal) = crate::ca::check_host_claim(&req.host_claim) {
        return Err(ControlApiError::invalid_command(refusal.to_string()));
    }

    let issue = authority::issue_enrollment_token_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.host_claim,
        chrono::Duration::seconds(ttl_seconds),
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
pub(crate) async fn authorize_tenant_admin(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: TenantId,
) -> Result<(), ControlApiError> {
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegation: None,
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
    /// `SIGNOFF-REPAIR.4.4.7.2.2`: authorize the possible duplicate of an
    /// `outcome_unknown` attempt, for a row dead-lettered for want of exactly
    /// that. Absent or false is the plain replay.
    #[serde(default)]
    pub allow_possible_duplicate: bool,
    /// Why the duplicate risk is accepted; required with the authorization.
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayResponse {
    pub node_id: String,
    pub command_id: String,
    pub replayed_at: String,
    /// The fresh reservation a possible-duplicate replay runs under; absent for a
    /// plain replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_id: Option<String>,
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
    if req.allow_possible_duplicate {
        return replay_with_possible_duplicate(&state, &principal, &req).await;
    }
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
            reservation_id: None,
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

/// The replay that authorizes a possible duplicate (`SIGNOFF-REPAIR.4.4.7.2.2`).
async fn replay_with_possible_duplicate(
    state: &ApiState,
    principal: &GrantSubject,
    req: &ReplayRequest,
) -> Result<Response, ControlApiError> {
    let replay = authority::replay_command_with_possible_duplicate_in_one_transaction(
        &state.pool,
        principal,
        req.tenant_id,
        &req.node_id,
        &req.command_id,
        req.reason.as_deref().unwrap_or_default(),
    )
    .await?;
    let receipt = replay.record_id;
    let response = match replay.result {
        authority::DuplicateReplayResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::DuplicateReplayResult::InvalidReason(detail) => {
            ControlApiError::invalid_command(format!(
                "a possible-duplicate replay needs a reason: {detail}"
            ))
            .into_response()
        }
        authority::DuplicateReplayResult::NotInInbox => ControlApiError::not_found(format!(
            "no command `{}` in node `{}`'s inbox",
            req.command_id, req.node_id
        ))
        .into_response(),
        authority::DuplicateReplayResult::NotAwaitingAuthorization => {
            ControlApiError::invalid_transition(
                "the command is not dead-lettered for want of the possible-duplicate \
                 authorization: only an outcome_unknown the node refused to retry takes it",
            )
            .into_response()
        }
        authority::DuplicateReplayResult::BudgetDenied { detail } => ControlApiError {
            status: StatusCode::CONFLICT,
            code: "budget_unavailable",
            message: format!("the ceiling has no room for the possible duplicate: {detail}"),
        }
        .into_response(),
        authority::DuplicateReplayResult::Replayed { reservation_id } => Json(ReplayResponse {
            node_id: req.node_id.clone(),
            command_id: req.command_id.clone(),
            replayed_at: replay.effected_at.to_rfc3339(),
            reservation_id: Some(reservation_id),
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
    /// The derived §10.6 delivery state (`node_inbox_state`): `queued` |
    /// `offered` | `transport_received` | `consumed` | `revoked` | `expired` |
    /// `dead_lettered`.
    pub delivery_state: String,
    /// The fold's refusal of this row's result, `{code, message}`, when the
    /// control plane received the result and REFUSED to apply it (the thread
    /// closed, the authority was withdrawn, …); `null` otherwise
    /// (`SIGNOFF-REPAIR.4.4.2.1`). The row is still `consumed`: the ladder
    /// records that the agent's result came back, and this records what the
    /// domain did with it. Without it, a refused result and an applied one read
    /// the same.
    pub result_refusal: Option<Value>,
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
///
/// ⛔ The caller supplies TWO independent identifiers — the tenant and the node
/// — and until `SIGNOFF-REPAIR.3.5.3` this handler checked one of them. The
/// admission proves the caller administers the tenant it NAMED; the select then
/// asked only for the node id, so an administrator of any tenant read whatever
/// rows that node held. Measured before the repair: a foreign administrator
/// received both of another tenant's rows with their command ids, thread ids,
/// delivery state and payloads.
///
/// ⭐ Contrast `inspect_call`, which is the shape this is now equivalent to: it
/// loads the call FIRST and authorizes against the call's OWN tenant, so the two
/// identifiers cannot disagree. Here the tenant stays a caller-supplied
/// parameter — changing that is a wire change, and the three sibling mutations
/// (`.3.3.4.10.3`) already bind the admitted tenant into their statements — so
/// the predicate joins the select instead.
///
/// ⛔ No transaction and no guard, decided rather than defaulted: this is a read
/// with no check-then-act, so a slightly older snapshot costs nothing and cannot
/// produce an unauthorized effect. Locking the tenant for it would be the cost
/// `.3.3.4.10.3` declined for `poll`, for the same reason.
async fn inspect_node_inbox(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Query(params): Query<InboxInspectionParams>,
) -> Result<Json<InboxInspection>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    authorize_tenant_admin(&state.pool, &principal, params.tenant_id).await?;
    Ok(Json(
        inbox_inspection(&state.pool, params.tenant_id, &params.node_id).await?,
    ))
}

/// The inbox inspection's read half: the select whose predicate binds BOTH
/// caller-supplied identifiers, which is the `SIGNOFF-REPAIR.3.5.3` repair.
///
/// ⭐ Shared with the MCP read seam since `SIGNOFF-REPAIR.6.1.1`. The seam's
/// own read used to be a reduced re-implementation over the `node_inbox`
/// TABLE; it now runs this one over the `node_inbox_state` VIEW, so the
/// delivery state and the quarantine reason travel with the row on both
/// surfaces.
pub(crate) async fn inbox_inspection(
    pool: &PgPool,
    tenant_id: TenantId,
    node_id: &str,
) -> Result<InboxInspection, ControlApiError> {
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
        result_refusal: Option<Value>,
    }
    // The refusal is the fold's STORED result for this row: the idempotency row
    // keyed by `node_result_fold_key` (`<node_id>:<command_id>`, `0104`), when
    // it records `ok: false`.
    let rows: Vec<InboxRowRow> = sqlx::query_as(
        "SELECT s.cursor, s.command_id, s.thread_id, s.payload, s.acknowledged_at, \
                s.quarantined_at, s.quarantine_reason, s.delivery_state, \
                CASE WHEN i.response_result->>'ok' = 'false' \
                     THEN i.response_result->'error' END AS result_refusal \
         FROM node_inbox_state s \
         LEFT JOIN idempotency i \
           ON i.tenant_id = s.tenant_id \
          AND i.idempotency_key = s.node_id || ':' || s.command_id \
         WHERE s.node_id = $1 AND s.tenant_id = $2 ORDER BY s.cursor",
    )
    .bind(node_id)
    .bind(tenant_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(InboxInspection {
        node_id: node_id.to_string(),
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
                result_refusal: r.result_refusal,
            })
            .collect(),
    })
}

/// The `POST /v1/nodes/inbox/prune` body: the retention window. TWO classes of
/// row at least this old are deleted, and the response reports them separately
/// (`SIGNOFF-REPAIR.11.24.1.1.2.1.1`):
///
/// 1. **delivered** rows, acknowledged by the node, aged by `acknowledged_at`;
/// 2. rows that were **never delivered** and reached §10.6's `expired` — the
///    admitting grant passed its own `expires_at` — aged by that same instant,
///    which is exactly when the row entered the terminal;
/// 3. rows that were **never delivered** and reached §10.6's `revoked`, aged by
///    the instant of the revocation itself (`migrations/0077`).
///
/// A QUARANTINED row is never prunable (§16.11, `.1.3.3`): the preservation
/// survives the disposition. Cleanup is an explicit, measured operator action;
/// nothing sweeps on its own.
///
/// ⛔ A `revoked` row whose grant carries no recoverable instant is still
/// retained. `migrations/0058` is where the audit record the backfill derives
/// from begins, so a revocation applied before it cannot be dated — and a
/// window aged by any other column would delete work the operator was told
/// they could still see.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PruneInboxRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub min_age_seconds: i64,
}

/// ⚠️ **`deleted` is the TOTAL, and its meaning widened with the verb.** It used
/// to be reachable only by delivered rows, so a client reading it as *delivered
/// rows removed* was accidentally right; it is now the sum of both classes, and
/// the breakdown beside it is what keeps an operator's retention statement true.
/// Pre-1.0, development profile, and the alternative is a receipt that says
/// *delivered* about work that was never handed over.
#[derive(Debug, Clone, Serialize)]
pub struct PruneInboxResponse {
    pub deleted: i64,
    pub deleted_delivered: i64,
    pub deleted_expired: i64,
    pub deleted_revoked: i64,
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
            deleted_delivered,
            deleted_expired,
            deleted_revoked,
            before,
            after,
            cutoff,
        } => Json(PruneInboxResponse {
            deleted,
            deleted_delivered,
            deleted_expired,
            deleted_revoked,
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
            deleted_delivered: 0,
            deleted_expired: 0,
            deleted_revoked: 0,
            before,
            after,
            cutoff_at: cutoff.to_rfc3339(),
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

// ── Node revocation (`.1.3.1`) ──────────────────────────────────────────────

/// The `POST /v1/nodes/revoke` body: a principal holding `TenantAdmin` revokes
/// the node's ACTIVE workload certificates. ⚠️ Like issuance, that is a GRANT
/// and not a kind of principal — an agent role holding it may revoke a node
/// (`SIGNOFF-REPAIR.4.1.4`, which found this second site by censusing the
/// sentence rather than trusting the two it was opened on). The `.1.2.2` handshake ladder refuses a
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
    /// The direction's lifetime (`SIGNOFF-REPAIR.5.3.4`): absent is no
    /// lifetime; a past instant is refused before the admission, as a
    /// malformed body is.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// The three federation direction verbs each run ONE guarded transaction
/// (`SIGNOFF-REPAIR.3.3.4.12`): the admission, the direction mutation, the
/// acceptance's cross-domain receipt and the final effect record share a single
/// commit under the LOCAL tenant's exclusive authority guard. Before this, each
/// ran a shared-guard admission that had already committed and then mutated on
/// the connection pool, with nothing recording what the request finally did.
///
/// A direction is the local tenant's own record of a relationship, so the local
/// administrator is the right and only authority for it; the pairing is
/// both-sides precisely so that neither side mutates the other's row.
///
/// Every answer carries the `x-reasonbraid-authorization` receipt naming the
/// admission this request committed, which is also the effect record's id.
///
/// `POST /v1/federation-agreements` — propose one direction. A proposal widens
/// NOTHING by itself (the pairing needs the remote side's own row).
async fn propose_federation_agreement(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<FederationAgreementRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    if req.expires_at.is_some_and(|at| at <= Utc::now()) {
        return Err(ControlApiError::invalid_command(
            "expires_at must be in the future",
        ));
    }
    let proposal = authority::propose_direction_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        req.remote_tenant_id,
        req.directory_visibility,
        req.recruitment,
        req.expires_at,
    )
    .await?;
    let receipt = proposal.record_id;
    let response = match proposal.result {
        authority::ProposeResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // Previously a RAISED foreign-key violation and a `500`
        // (`SIGNOFF-REPAIR.3.3.4.12`). A proposal has to name a real
        // counterparty, so this answer is intrinsic to the operation.
        authority::ProposeResult::UnknownRemote => ControlApiError::not_found(format!(
            "no tenant `{}` to federate with",
            req.remote_tenant_id
        ))
        .into_response(),
        // One answer for both: a re-proposal on identical terms is reported the
        // same way it always was, and the effect record is where it reads `no_op`.
        authority::ProposeResult::Proposed { agreement_id }
        | authority::ProposeResult::Unchanged { agreement_id } => {
            Json(json!({ "agreement_id": agreement_id, "status": "proposed" })).into_response()
        }
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
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
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let acceptance = authority::accept_direction_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        req.remote_tenant_id,
    )
    .await?;
    let receipt = acceptance.record_id;
    let response = match acceptance.result {
        authority::AcceptResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // The two idle states keep their single message, unchanged; the effect
        // record distinguishes an already-accepted direction from one that was
        // never proposed.
        authority::AcceptResult::AlreadyAccepted | authority::AcceptResult::NothingProposed => {
            ControlApiError::invalid_transition(
                "no PROPOSED agreement in this direction to accept (the remote side must propose \
                 first)",
            )
            .into_response()
        }
        // `SIGNOFF-REPAIR.5.3.1`: an acceptance pins the counterparty's terms,
        // so the counterparty must have proposed — which the message above
        // always claimed and the code never required.
        authority::AcceptResult::NoCounterparty { remote_tenant_id } => {
            ControlApiError::invalid_transition(format!(
                "the counterparty `{remote_tenant_id}` has no live direction toward this tenant — \
                 an acceptance pins its terms, so it must propose first"
            ))
            .into_response()
        }
        authority::AcceptResult::Accepted => Json(json!({ "status": "accepted" })).into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

/// `POST /v1/federation-agreements/revoke` — revoke this tenant's
/// direction (the fallback: the network pseudonym).
async fn revoke_federation_agreement(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<FederationAgreementAction>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let revocation = authority::revoke_direction_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        req.remote_tenant_id,
    )
    .await?;
    let receipt = revocation.record_id;
    let response = match revocation.result {
        authority::RevokeDirectionResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // The existing `revoked` count is preserved exactly: 1 when a live
        // direction was revoked, 0 when there was nothing to revoke.
        authority::RevokeDirectionResult::Revoked => Json(json!({ "revoked": 1 })).into_response(),
        authority::RevokeDirectionResult::NothingToRevoke => {
            Json(json!({ "revoked": 0 })).into_response()
        }
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
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
                i64,
                i64,
                Option<Value>,
                DateTime<Utc>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT np.node_id, np.online, np.suspended, np.last_seen_at, np.lease_expires_at, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version), \
                np.in_flight, \
                (SELECT count(*) FROM node_inbox_state i \
                  WHERE i.node_id = np.node_id \
                    AND i.delivery_state IN ('queued', 'offered')), \
                (SELECT v.profile->'availability' \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = np.node_id AND v.version = p.current_version), \
                now() \
         FROM node_presence np WHERE np.tenant_id = $1 ORDER BY np.node_id",
    )
    .bind(q.tenant_id.to_string())
    .fetch_all(&pool)
    .await?;
            let nodes: Vec<Value> = rows
                .into_iter()
                .map(
                    |(
                        node_id,
                        online,
                        suspended,
                        last_seen_at,
                        lease_expires_at,
                        concurrency,
                        in_flight,
                        undelivered,
                        availability,
                        now,
                    )| {
                        let hold = crate::presence::hold_from_stored(availability.as_ref(), now);
                        json!({
                            "node_id": node_id,
                            "state": crate::presence::presence_state(
                                true, suspended, online, concurrency, in_flight, hold.as_ref(),
                            )
                            .as_str(),
                            "hold": hold.as_ref().map(|h| h.wire_name()),
                            "online": online,
                            "suspended": suspended,
                            "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                            "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
                            // §10.7's offline backlog, beside the state
                            // (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`): what the node
                            // holds undelivered against the cap that refuses more.
                            "backlog": {
                                "undelivered": undelivered,
                                "cap": node_channel::MAX_OFFLINE_BACKLOG,
                            },
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

/// `POST /v1/admin/nodes/ambiguous-attempts/adjudicate` — the body.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdjudicateAttemptRequest {
    pub tenant_id: TenantId,
    pub node_id: String,
    pub attempt_id: String,
    /// `completed` or `failed_known` (§11.3).
    pub verdict: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct AdjudicateAttemptResponse {
    pub node_id: String,
    pub attempt_id: String,
    pub verdict: String,
    pub adjudicated_at: String,
    /// The row closes when the node applies the verdict at its next handshake.
    pub applied_by_node: bool,
}

/// `POST /v1/admin/nodes/ambiguous-attempts/adjudicate` — §11.3's fourth
/// option (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`): a `tenant_admin` asserts what an
/// `outcome_unknown` attempt did, with a reason. One shared-guard transaction:
/// the admission, the tenant-bound row, the verdict, and the administrative
/// effect commit together; the node applies the verdict at its next handshake.
async fn adjudicate_ambiguous_attempt(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<AdjudicateAttemptRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let outcome = authority::adjudicate_ambiguous_attempt_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.node_id,
        &req.attempt_id,
        &req.verdict,
        &req.reason,
    )
    .await?;
    let receipt = outcome.record_id;
    let response = match outcome.result {
        authority::AdjudicationResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        authority::AdjudicationResult::InvalidVerdict(detail) => {
            ControlApiError::invalid_command(detail).into_response()
        }
        authority::AdjudicationResult::InvalidReason(detail) => {
            ControlApiError::invalid_command(format!(
                "the adjudication reason is required (a verdict without a reason cannot be \
                 reviewed): {detail}"
            ))
            .into_response()
        }
        authority::AdjudicationResult::NotOpen => ControlApiError::invalid_command(format!(
            "no open ambiguous attempt `{}` on node `{}` in this tenant",
            req.attempt_id, req.node_id
        ))
        .into_response(),
        authority::AdjudicationResult::AlreadyAdjudicated { verdict, at } => {
            ControlApiError::invalid_transition(format!(
                "the attempt already carries the verdict `{verdict}` ({at}), which the node has \
                 not applied yet"
            ))
            .into_response()
        }
        authority::AdjudicationResult::Adjudicated { at } => Json(AdjudicateAttemptResponse {
            node_id: req.node_id.clone(),
            attempt_id: req.attempt_id.clone(),
            verdict: req.verdict.clone(),
            adjudicated_at: at.to_rfc3339(),
            applied_by_node: false,
        })
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
}

/// The safe resolution actions for an ambiguous attempt, in the order an
/// operator should consider them — `docs/runbooks/provider-outage-ambiguous-charge.md`'s
/// *Recovery*, as a fixed vocabulary. The first three are taken where the
/// authority for them lives, which is why the list names who; the fourth is
/// this surface's own verb (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`).
///
/// ⛔ Each action states whether it can be taken TODAY (`SIGNOFF-REPAIR.4.4.7.1`).
/// The re-ask was listed for commits as if it could, while nothing could set
/// the flag it names; an operator following it would have found no verb. Since
/// `.4.4.7.2` it is the tenant administrator's possible-duplicate replay, and
/// every action on the list can be taken.
const AMBIGUOUS_ATTEMPT_ACTIONS: [AmbiguousAttemptAction; 4] = [
    AmbiguousAttemptAction {
        action: "provider_status_lookup",
        who: "the node's operator",
        effect: "a proven lookup lands the attempt completed or failed_known in the node's \
                 journal; the node's next handshake closes this row as resolved_by_node",
        unavailable_because: None,
    },
    AmbiguousAttemptAction {
        action: "reask_with_allow_possible_duplicate",
        who: "the tenant's administrator",
        effect: "once the node has dead-lettered the item as retry_requires_authorization, \
                 POST /v1/nodes/replay with allow_possible_duplicate and a reason \
                 (`rb node replay --allow-possible-duplicate --reason …`): a fresh reservation \
                 pays for the possible duplicate, the original stays held, and the node re-runs \
                 the item once; never a silent retry",
        unavailable_because: None,
    },
    AmbiguousAttemptAction {
        action: "close_with_unresolved_register",
        who: "the thread's human",
        effect: "closes the thread honestly with the attempt in its unresolved register",
        unavailable_because: None,
    },
    AmbiguousAttemptAction {
        action: "adjudicate",
        who: "the tenant's administrator",
        effect: "POST /v1/admin/nodes/ambiguous-attempts/adjudicate with `completed` or \
                 `failed_known` and a reason, when the provider's own records settle the \
                 question; the node applies it at its next handshake and the row closes \
                 adjudicated",
        unavailable_because: None,
    },
];

/// One safe resolution action for an ambiguous attempt, and whether it can be
/// taken today.
struct AmbiguousAttemptAction {
    action: &'static str,
    who: &'static str,
    effect: &'static str,
    /// `None` when the action can be taken; otherwise why not, and what will
    /// change that.
    unavailable_because: Option<&'static str>,
}

/// `GET /v1/admin/nodes/ambiguous-attempts?tenant_id=…` — the tenant's
/// ambiguous attempts that are still OPEN, with the safe actions for them
/// (`SIGNOFF-REPAIR.4.6.1.1`; ROADMAP §18.5).
///
/// A row is what a node reported at its handshake and the server could not
/// settle: it holds no receipt for the attempt's operation. Rows are written by
/// the handshake (`NodeChannelState::record_ambiguous_reports`); closed rows are
/// kept with how they ended and are not listed here. The tenant is the node's.
///
/// ⛔ `re-fire the call to check` is NOT among the actions, deliberately: the
/// runbook's first safe action is never to do it, because a re-fire can charge
/// twice. The response says so rather than leaving it to be inferred.
///
/// Uses the own-tenant inspection gate, like `list_node_presence` beside it:
/// both read node state an administrator needs while a revocation is in flight.
async fn list_ambiguous_attempts(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::AmbiguousAttempts {},
        |pool| async move {
            type Row = (
                String,
                String,
                String,
                String,
                DateTime<Utc>,
                DateTime<Utc>,
                i64,
                Option<String>,
                Option<String>,
                Option<DateTime<Utc>>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT a.node_id, a.attempt_id, a.operation_id, a.reason, \
                        a.first_reported_at, a.last_reported_at, a.report_count, \
                        a.operator_verdict, a.operator_reason, a.adjudicated_at \
                 FROM node_ambiguous_attempts a JOIN nodes n ON n.node_id = a.node_id \
                 WHERE n.tenant_id = $1 AND a.closed_at IS NULL \
                 ORDER BY a.first_reported_at, a.node_id, a.attempt_id",
            )
            .bind(q.tenant_id.to_string())
            .fetch_all(&pool)
            .await?;
            let attempts: Vec<Value> = rows
                .into_iter()
                .map(
                    |(
                        node_id,
                        attempt_id,
                        operation_id,
                        reason,
                        first,
                        last,
                        count,
                        verdict,
                        operator_reason,
                        adjudicated_at,
                    )| {
                        let mut row = json!({
                            "node_id": node_id,
                            "attempt_id": attempt_id,
                            "operation_id": operation_id,
                            "reason": reason,
                            "first_reported_at": first.to_rfc3339(),
                            "last_reported_at": last.to_rfc3339(),
                            "report_count": count,
                        });
                        // A verdict the node has not applied yet
                        // (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`); absent facts are omitted.
                        if let (Some(verdict), Some(reason), Some(at)) =
                            (verdict, operator_reason, adjudicated_at)
                        {
                            row["operator_verdict"] = json!(verdict);
                            row["operator_reason"] = json!(reason);
                            row["adjudicated_at"] = json!(at.to_rfc3339());
                        }
                        row
                    },
                )
                .collect();
            let safe_actions: Vec<Value> = AMBIGUOUS_ATTEMPT_ACTIONS
                .iter()
                .map(|a| {
                    let mut entry = json!({
                        "action": a.action,
                        "who": a.who,
                        "effect": a.effect,
                        "available": a.unavailable_because.is_none(),
                    });
                    if let Some(why) = a.unavailable_because {
                        entry["unavailable_because"] = json!(why);
                    }
                    entry
                })
                .collect();
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "attempts": attempts,
                "safe_actions": safe_actions,
                "never": "re-fire the provider call to check: it can charge twice",
                "runbook": "docs/runbooks/provider-outage-ambiguous-charge.md",
            })))
        },
    )
    .await
}

// ── The resolver capability registry (PHASE-4.1.3; backlog 31) ──────────────────────

/// `POST /v1/resolvers` — the operator registers one resolver's §12.2
/// advertise (the tenant_admin gate; the future packs call it at their
/// install).
///
/// ⛔ IT REGISTERS; IT DOES NOT REPLACE, and the doc comment used to say
/// "registers (or replaces)" while the underlying upsert wrote 6 of its 18
/// columns. That combination is the worst of the three available answers: a
/// caller narrowing a pack's `media_types` — the documented use of the field —
/// received `200 {"registered": true}` and the registry kept serving the wide
/// set (`SIGNOFF-REPAIR.7.3.6.2`).
///
/// ⚠️ COMPLETING THE REPLACE HERE WAS THE REJECTED OPTION, and the reason is a
/// finding this leaf did not make: `resolver_capabilities` has no tenant column
/// and a single-column `resolver_id` key, so ANY tenant's administrator can
/// address ANY row, the built-in `r0-https-fetcher` included
/// (`SIGNOFF-REPAIR.11.9.1.1.1`; binding that authority is `.7.1`'s). Widening
/// the upsert to satisfy this doc comment would have handed that unbound
/// principal eleven more columns — `media_types`, all four advertised
/// policies, the abilities and the authentication classes — on a site-global
/// row. A silent no-op is a defect; a loud refusal is not, and it is the only
/// one of the three that does not grow the surface `.7.1` has to bind.
async fn register_resolver(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<RegisterResolverRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    // Before the gate, as every site route bounds its input on extraction: the
    // ADR-018 vocabulary is a published constant, so naming the rule that
    // failed is an oracle over nothing.
    if let Some(error) = req.advertise.isolation_error() {
        return Err(ControlApiError::invalid_command(error));
    }
    site_receipt_response(
        site::resolvers::register_resolver(&state.pool, &principal, &req.advertise, &req.reason)
            .await,
        "a current site grant for this action and its actual boundary are required",
    )
}

/// `POST /v1/resolvers`'s body (`SIGNOFF-REPAIR.7.1.3.1`): the advertisement
/// and the site act's reason. The advertisement refuses unknown fields, so the
/// reason cannot ride inside it; it is NESTED rather than flattened, because
/// serde does not combine `flatten` with `deny_unknown_fields`.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisterResolverRequest {
    advertise: crate::resolvers::ResolverAdvertise,
    reason: site::Reason,
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
/// ⛔ `any` is the PERMISSIVE default, and it is the honest spelling of the
/// behaviour that shipped. The egress claim is a maximum, so a requirement is
/// a ceiling: `any` means *I place no bound on where this pack may dial*.
///
/// The field read `loopback`, which under the old `declared >= required` test
/// also admitted every pack — every one declares `listed` or `any`. So the
/// default's behaviour is unchanged by `SIGNOFF-REPAIR.7.3.6.4`; what changed
/// is that the word now says what it does. `loopback` under the corrected
/// comparison would mean *at most loopback*, which no pack declares, and every
/// default resolution would answer `unresolvable_now`.
///
/// ⚠️ Whether the default SHOULD be permissive is a separate decision this leaf
/// does not take: tightening it changes what every existing caller gets, and
/// `.7.3.6.4` owns the comparison, not the policy.
fn default_egress() -> String {
    "any".to_string()
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal resolves no reference",
        ));
    };
    // Each pack below may acquire bytes and persist a snapshot. The tenant
    // that asked for the resolution is the tenant whose evidence reads must
    // show that row (`.11.14.1`) — an acquisition nobody cites is an
    // acquisition nobody can read.
    let citer = citer(&principal, tenant.clone());
    // The same binding the detail read takes (`SIGNOFF-REPAIR.11.14.3.4`).
    // Without it a caller refused the READ could still drive an acquisition off
    // the reference — including, with the gated R5 pack on, off the
    // `credential_binding_ref` another tenant wrote into the shared row.
    let Some((_, reference, _, _)) =
        crate::resources::get_for_tenant(&state.pool, &resource_id, &tenant).await?
    else {
        return Err(ControlApiError::not_found(format!(
            "no reference `{resource_id}`"
        )));
    };
    // 🔴 AN OFF-LADDER REQUIREMENT IS NAMED, not answered as an absence
    // (`SIGNOFF-REPAIR.7.3.6.4`). `resolve` locates each required class with
    // `position()`; an unknown one yielded `None`, every row became ineligible,
    // and the caller received `unresolvable_now` — indistinguishable from "no
    // resolver available". Fail-closed, and silent about which.
    //
    // ⛔ It is validated HERE, after the tenant binding, deliberately. Checking
    // the body first would answer 400 for a resource this caller may not know
    // exists, which re-opens the existence oracle `.11.14.3.4` closed: a
    // foreign or absent id must keep giving one answer.
    for (field, value, vocabulary) in [
        (
            "required_sandbox",
            &req.required_sandbox,
            &crate::resolvers::SANDBOX_LEVELS[..],
        ),
        (
            "required_egress",
            &req.required_egress,
            &crate::resolvers::EGRESS_CLASSES[..],
        ),
    ] {
        if !vocabulary.contains(&value.as_str()) {
            return Err(ControlApiError::invalid_command(format!(
                "`{field}` must be one of {}; `{value}` is outside the ADR-018 vocabulary",
                vocabulary.join(", ")
            )));
        }
    }
    let mut outcome = crate::resolvers::resolve(
        &state.pool,
        &reference.scheme,
        reference.media_type_hint.as_deref(),
        reference.credential_binding_ref.as_deref(),
        &req.required_sandbox,
        &req.required_egress,
    )
    .await?;
    // The resolver that ACTS is the first ranked one this server can execute
    // (`SIGNOFF-REPAIR.7.1.3`); the bounds below and the execution count it.
    let executing = outcome.select_executable(state.r5r3rx_enabled);
    // §16.11's acquisition bounds (`SIGNOFF-REPAIR.11.14.3.14`). This is the
    // surface the section names as "resolver abuse" and "scraping", and until
    // this leaf it carried no quota, no storm control and no breaker while
    // performing a real network acquisition per call.
    //
    // Two scopes, both per tenant: the RANKED RESOLVER and the locator's HOST.
    // Each falls back to the tenant's `*` default row when it has no specific
    // one, which is what makes a fail-closed bound possible over a member space
    // the server does not control.
    //
    // ⚠️ Placed after the ranking and before the match rather than inside each
    // arm, so an attempt against a gated-off pack is counted too. That is
    // deliberate: the bound is on ATTEMPTS, and an attempt is what a caller can
    // repeat. ⚠️ A locator with no host takes the resolver bound only — there is
    // no destination to bound, and no pack can fetch such a locator anyway.
    if let Some(ranked) = executing.clone() {
        let host = url::Url::parse(&reference.original_locator)
            .ok()
            .and_then(|parsed| parsed.host_str().map(str::to_owned));
        let mut quota_tx = state.pool.begin().await?;
        let mut refusal = None;
        for (scope_kind, scope_id) in [
            (Some(crate::quota::SCOPE_RESOLVER), Some(ranked.clone())),
            (Some(crate::quota::SCOPE_DESTINATION), host),
        ] {
            let (Some(scope_kind), Some(scope_id)) = (scope_kind, scope_id) else {
                continue;
            };
            if let Err(error) = crate::quota::check_open_scope_in_tx(
                &mut *quota_tx,
                &tenant,
                scope_kind,
                &scope_id,
                chrono::Utc::now(),
            )
            .await
            {
                refusal = Some(error);
                break;
            }
        }
        // The operator's record of WHICH request was refused
        // (`SIGNOFF-REPAIR.4.6.1.2`): `quota_events` counts the denial per
        // quota and names neither the resource nor the caller. A storage
        // failure is not a refusal of the caller, so it records nothing here.
        let refused_as = match &refusal {
            Some(crate::quota::QuotaError::Exceeded { .. }) => Some("quota_exceeded"),
            Some(crate::quota::QuotaError::Unconfigured { .. }) => Some("quota_unconfigured"),
            Some(crate::quota::QuotaError::Storage(_)) | None => None,
        };
        if let (Some(kind), Some(error)) = (refused_as, &refusal) {
            record_resolution_refusal(
                &mut *quota_tx,
                &ResolutionRefusal {
                    tenant_id: &tenant,
                    resource_id: &resource_id,
                    requested_by: &principal.id_string(),
                    resolver_id: Some(&ranked),
                    kind,
                    message: &error.to_string(),
                },
            )
            .await?;
        }
        // The denial row is a recorded fact and must survive the refusal, so the
        // transaction commits either way — the `.3.5.1` shape.
        quota_tx.commit().await?;
        if let Some(error) = refusal {
            return Err(match error {
                crate::quota::QuotaError::Exceeded { .. } => {
                    ControlApiError::quota_exceeded(error.to_string())
                }
                crate::quota::QuotaError::Unconfigured { .. } => {
                    ControlApiError::quota_unconfigured(error.to_string())
                }
                crate::quota::QuotaError::Storage(cause) => {
                    storage_failure(cause, "the acquisition quota check failed")
                }
            });
        }
    }
    let outcome = acquire_ranked(&state, &citer, &resource_id, &reference, outcome).await?;
    // §18.5's *resolver denials* (`SIGNOFF-REPAIR.4.6.1.2`): the refusal this
    // answer carries is recorded before it is returned. Without the row the
    // refusal reached the caller alone and no operator route could list it.
    if let Some((resolver_id, kind, message)) = refusal_answered(&outcome, &reference.scheme, &req)
    {
        record_resolution_refusal(
            &state.pool,
            &ResolutionRefusal {
                tenant_id: &tenant,
                resource_id: &resource_id,
                requested_by: &principal.id_string(),
                resolver_id: resolver_id.as_deref(),
                kind: &kind,
                message: &message,
            },
        )
        .await?;
    }
    Ok(Json(outcome))
}

/// The workflow profile an incident is conducted under (§13's table:
/// *operational/security events*).
const INCIDENT_PROFILE: &str = "incident_review";

/// `GET /v1/admin/incidents?tenant_id=…` — the tenant's ACTIVE incidents
/// (`SIGNOFF-REPAIR.4.6.1.5.1`; ROADMAP §18.5).
///
/// An incident is a thread under the built-in `incident_review` profile, and
/// it is active while its state is not terminal
/// (`docs/decisions/2026-09-22_an-incident-is-an-open-incident-review-thread-and-a-backup-is-reported-by-its-receipts.md`).
/// Declaring one is `thread.create` with that profile and resolving one is the
/// thread's own close or cancel, so this route adds no verb: it is a view over
/// the projection the thread verbs already maintain, oldest incident first.
///
/// The active states are DERIVED from `ThreadState`'s transition table rather
/// than typed here, so a new state cannot be silently missing from the list.
async fn list_active_incidents(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::Incidents {},
        |pool| async move {
            let active: Vec<String> = reasonbraid_core::ThreadState::ALL
                .into_iter()
                .filter(|s| !s.is_terminal())
                .map(|s| s.as_str().to_owned())
                .collect();
            let tenant = q.tenant_id.to_string();
            let claim = tenant.clone();
            let rows: Vec<(String, Value, DateTime<Utc>)> =
                crate::rls::with_tenant_claim(&pool, &claim, |tx| {
                    Box::pin(async move {
                        sqlx::query_as(
                            "SELECT s.aggregate_id, s.state, \
                                    (SELECT min(e.committed_at) FROM event_log e \
                                     WHERE e.tenant_id = s.tenant_id \
                                       AND e.aggregate_id = s.aggregate_id) \
                             FROM aggregate_state s \
                             WHERE s.tenant_id = $1 AND s.aggregate_type = 'thread' \
                               AND s.state->>'workflow_profile' = $2 \
                               AND s.state->>'state' = ANY($3) \
                             ORDER BY 3, 1",
                        )
                        .bind(tenant)
                        .bind(INCIDENT_PROFILE)
                        .bind(active)
                        .fetch_all(&mut *tx)
                        .await
                    })
                })
                .await?;
            let incidents: Vec<Value> = rows
                .into_iter()
                .map(|(thread_id, projection, opened_at)| {
                    let step = projection["workflow_step"].as_u64().unwrap_or(0) as usize;
                    json!({
                        "thread_id": thread_id,
                        "subject": projection["subject"],
                        "state": projection["state"],
                        "opened_at": opened_at.to_rfc3339(),
                        "workflow_step": projection["workflow_steps"].get(step),
                        "open_challenges": projection["open_challenges"],
                    })
                })
                .collect();
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "incidents": incidents,
            })))
        },
    )
    .await
}

/// The most refusals one read returns, newest first. The response states the
/// bound, so a truncated list is never mistaken for the whole history.
const RESOLUTION_REFUSAL_PAGE: i64 = 500;

/// `GET /v1/admin/resolution-refusals?tenant_id=…` — the refusals the resolve
/// path answered the tenant's callers, newest first (`SIGNOFF-REPAIR.4.6.1.2`;
/// ROADMAP §18.5 *resolver denials*).
///
/// Each row is exactly what the caller was told: the refusal's own `kind`, its
/// message, the resolver that ranked first (none for `unresolvable_now`), the
/// resource, and who asked. Rows are written by `resolve_resource` itself, so
/// the list cannot disagree with what was answered.
///
/// Uses the own-tenant inspection gate, like the other `/v1/admin/*` reads.
async fn list_resolution_refusals(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::ResolutionRefusals {},
        |pool| async move {
            type Row = (
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                DateTime<Utc>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT refusal_id, resource_id, requested_by, resolver_id, kind, message, \
                        refused_at \
                 FROM resolution_refusals WHERE tenant_id = $1 \
                 ORDER BY refused_at DESC, refusal_id DESC LIMIT $2",
            )
            .bind(q.tenant_id.to_string())
            .bind(RESOLUTION_REFUSAL_PAGE)
            .fetch_all(&pool)
            .await?;
            let refusals: Vec<Value> = rows
                .into_iter()
                .map(
                    |(refusal_id, resource_id, requested_by, resolver_id, kind, message, at)| {
                        json!({
                            "refusal_id": refusal_id,
                            "resource_id": resource_id,
                            "requested_by": requested_by,
                            "resolver_id": resolver_id,
                            "kind": kind,
                            "message": message,
                            "refused_at": at.to_rfc3339(),
                        })
                    },
                )
                .collect();
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "refusals": refusals,
                "limit": RESOLUTION_REFUSAL_PAGE,
            })))
        },
    )
    .await
}

/// The refusal a resolution answer carries, if it carries one, as
/// `(resolver, kind, message)` — exactly the words the caller was given.
///
/// `unresolvable_now` has no resolver: none was eligible. Every other refusal
/// is the first-ranked pack's named `acquisition_error`. An answer carrying
/// neither is an acquisition, and records nothing here.
fn refusal_answered(
    outcome: &crate::resolvers::ResolutionOutcome,
    scheme: &str,
    req: &ResolveRequest,
) -> Option<(Option<String>, String, String)> {
    if outcome.unresolvable_now {
        return Some((
            None,
            "unresolvable_now".to_owned(),
            format!(
                "no registered resolver serves scheme `{scheme}` within required_sandbox \
                 `{}` and required_egress `{}`",
                req.required_sandbox, req.required_egress
            ),
        ));
    }
    outcome.acquisition_error.as_ref().map(|error| {
        // The refusing resolver is the one that ACTED: the first ranked one not
        // named unexecutable (`SIGNOFF-REPAIR.7.1.3`), or none when none could.
        (
            outcome
                .resolvers
                .iter()
                .find(|id| !outcome.unexecutable.contains(id))
                .cloned(),
            error.kind.clone(),
            error.message.clone(),
        )
    })
}

/// `GET /v1/admin/storm-refusals?tenant_id=…` — every `429 storm_control` this
/// tenant's callers were answered, newest first, with the control that refused,
/// its limit, the initiator, the thread named, and the words the caller was
/// given (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.4`). Rows are written by
/// `refuse_storm` itself, so the list cannot disagree with what was answered.
/// This is what makes the circuit breakers' trigger — *the first multi-tenant
/// storm observed* — a condition anyone can evaluate.
///
/// Uses the own-tenant inspection gate, like the other `/v1/admin/*` reads.
async fn list_storm_refusals(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<AdminListQuery>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    inspect_tenant_admin(
        &state,
        &principal,
        q.tenant_id,
        reasonbraid_core::TenantAdminInspection::StormRefusals {},
        |pool| async move {
            type Row = (
                String,
                String,
                String,
                Option<i64>,
                Option<String>,
                String,
                DateTime<Utc>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT refusal_id, initiator, control, limit_value, target, message, refused_at \
                 FROM storm_refusals WHERE tenant_id = $1 \
                 ORDER BY refused_at DESC, refusal_id DESC LIMIT $2",
            )
            .bind(q.tenant_id.to_string())
            .bind(RESOLUTION_REFUSAL_PAGE)
            .fetch_all(&pool)
            .await?;
            let refusals: Vec<Value> = rows
                .into_iter()
                .map(
                    |(refusal_id, initiator, control, limit_value, target, message, at)| {
                        let mut row = json!({
                            "refusal_id": refusal_id,
                            "initiator": initiator,
                            "control": control,
                            "message": message,
                            "refused_at": at.to_rfc3339(),
                        });
                        // Absent optional facts are OMITTED on the wire, never null.
                        if let Some(limit_value) = limit_value {
                            row["limit_value"] = json!(limit_value);
                        }
                        if let Some(target) = target {
                            row["target"] = json!(target);
                        }
                        row
                    },
                )
                .collect();
            Ok(Json(json!({
                "tenant_id": q.tenant_id.to_string(),
                "refusals": refusals,
                "limit": RESOLUTION_REFUSAL_PAGE,
            })))
        },
    )
    .await
}

/// One storm-control refusal, as `migrations/0091_storm_refusals.sql` stores it.
struct StormRefusal<'a> {
    tenant_id: &'a str,
    initiator: &'a str,
    /// The limit's own name (`open_calls_per_tenant`, `autonomous_depth`, …).
    control: &'a str,
    /// The numeric limit, when the control has one.
    limit_value: Option<i64>,
    /// The thread the refused request named, when it named one.
    target: Option<&'a str>,
}

/// Record a storm-control refusal and return the `429` that answers it
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.4`). ⛔ This is the ONLY place the storm
/// wire code is spelled as a value: a producer that built the error by hand
/// would be a refusal answered and recorded nowhere, which is the defect this
/// helper removes; the leaf carries the census that counts the spelling.
/// Recording is not optional and not best-effort: a storage failure here
/// is an internal error, because a storm the operator cannot see is worse than
/// a request refused twice.
async fn refuse_storm(
    pool: &PgPool,
    refusal: StormRefusal<'_>,
    message: String,
) -> ControlApiError {
    let recorded = sqlx::query(
        "INSERT INTO storm_refusals \
           (refusal_id, tenant_id, initiator, control, limit_value, target, message) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(format!("srf_{}", uuid::Uuid::now_v7()))
    .bind(refusal.tenant_id)
    .bind(refusal.initiator)
    .bind(refusal.control)
    .bind(refusal.limit_value)
    .bind(refusal.target)
    .bind(&message)
    .execute(pool)
    .await;
    match recorded {
        Ok(_) => ControlApiError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "storm_control",
            message,
        },
        Err(error) => ControlApiError::internal_with_log(format!(
            "a storm-control refusal could not be recorded: {error}"
        )),
    }
}

/// One refused resolution, as `migrations/0088_resolution_refusals.sql` stores it.
struct ResolutionRefusal<'a> {
    tenant_id: &'a str,
    resource_id: &'a str,
    requested_by: &'a str,
    resolver_id: Option<&'a str>,
    kind: &'a str,
    message: &'a str,
}

/// Record one refused resolution (`SIGNOFF-REPAIR.4.6.1.2`). Generic over the
/// executor so a quota refusal commits in the SAME transaction as its
/// `quota_events` denial.
async fn record_resolution_refusal<'e, E>(
    executor: E,
    refusal: &ResolutionRefusal<'_>,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO resolution_refusals \
           (refusal_id, tenant_id, resource_id, requested_by, resolver_id, kind, message) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(format!("rrf_{}", uuid::Uuid::now_v7()))
    .bind(refusal.tenant_id)
    .bind(refusal.resource_id)
    .bind(refusal.requested_by)
    .bind(refusal.resolver_id)
    .bind(refusal.kind)
    .bind(refusal.message)
    .execute(executor)
    .await?;
    Ok(())
}

/// Execute the first-ranked built-in pack, if one ranked (the body of
/// `resolve_resource` after its binding, validation, ranking and quota
/// admission). Returns the outcome with the acquisition or its NAMED refusal;
/// the caller records a refusal before answering, which is why this returns
/// the outcome rather than the response.
async fn acquire_ranked(
    state: &ApiState,
    citer: &crate::snapshots::Citer,
    resource_id: &str,
    reference: &crate::resources::ResourceReference,
    mut outcome: crate::resolvers::ResolutionOutcome,
) -> Result<crate::resolvers::ResolutionOutcome, ControlApiError> {
    // The built-in packs execute when they are the first EXECUTABLE ranked
    // resolver (`SIGNOFF-REPAIR.7.1.3`): a ranked row this server has no
    // executor for is skipped and named in `unexecutable`, never allowed to
    // turn the answer into an empty success. The acquisition runs under the
    // pack's own ceilings + the `.2.1` policy; a refusal is the NAMED error,
    // and the reference stays submitted either way.
    let executing = outcome
        .resolvers
        .iter()
        .find(|id| crate::resolvers::executable(id, state.r5r3rx_enabled))
        .cloned();
    match executing.as_deref() {
        Some(crate::resolvers::R0_RESOLVER_ID) => {
            match state.fetcher.fetch(&reference.original_locator).await {
                Ok(document) => {
                    let receipt = crate::fetcher::AcquisitionReceipt::from_document(
                        &reference.original_locator,
                        &document,
                        chrono::Utc::now(),
                    );
                    // The snapshot store (`.6.1`): the acquired bytes land
                    // under their digest. ⛔ A persistence failure is NOT a
                    // successful acquisition — this arm used to discard the
                    // error and still set `outcome.acquisition`, so a caller
                    // was told the document had been acquired while no evidence
                    // row existed. `.7.4.2` decided that for the R2 arm and this
                    // one was never migrated onto it (`SIGNOFF-REPAIR.11.14.3.12`).
                    let stored = crate::snapshots::submit(
                        &state.pool,
                        &crate::snapshots::SnapshotSubmission {
                            reference_id: resource_id.to_owned(),
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
                        citer,
                    )
                    .await;
                    if let Err(error) = stored {
                        crate::log_event!(
                            "acquisition_evidence_unstored",
                            "resource_id" => resource_id,
                            "reason" => error.to_string(),
                        );
                        outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                            kind: "evidence_unstored".to_owned(),
                            message: error.to_string(),
                        });
                        return Ok(outcome);
                    }
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
                            // ⛔ The disclosure names where the credential WENT.
                            // It used to name `final_url` — the LAST hop — so a
                            // credential the fetcher had sent to one host was
                            // disclosed against whichever host the redirect
                            // chain happened to end on. `credential_hosts` is
                            // recorded at the moment the header is attached
                            // (`SIGNOFF-REPAIR.7.2.6`).
                            let disclosure = state.broker.disclose(
                                &binding,
                                document
                                    .credential_hosts
                                    .first()
                                    .map(String::as_str)
                                    .unwrap_or("unknown"),
                                chrono::Utc::now(),
                            );
                            match disclosure {
                                Ok(disclosure) => {
                                    let receipt = crate::fetcher::AcquisitionReceipt::from_document(
                                        &reference.original_locator,
                                        &document,
                                        chrono::Utc::now(),
                                    );
                                    // The same rule the R0 arm and `.7.4.2`'s
                                    // R2 arm apply: evidence that did not
                                    // persist is not an acquisition
                                    // (`SIGNOFF-REPAIR.11.14.3.12`).
                                    let stored = crate::snapshots::submit(
                                        &state.pool,
                                        &crate::snapshots::SnapshotSubmission {
                                            reference_id: resource_id.to_owned(),
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
                                            disclosure_policy: serde_json::json!({ "credential_class": disclosure.credential_class, "host": disclosure.host, "credential_hosts": document.credential_hosts }),
                                            license: None,
                                            fresh_until: None,
                                        },
                                        &document.bytes,
                                        chrono::Utc::now(),
                                        citer,
                                    )
                                    .await;
                                    if let Err(error) = stored {
                                        crate::log_event!(
                                            "acquisition_evidence_unstored",
                                            "resource_id" => resource_id,
                                            "reason" => error.to_string(),
                                        );
                                        outcome.acquisition_error =
                                            Some(crate::resolvers::AcquisitionError {
                                                kind: "evidence_unstored".to_owned(),
                                                message: error.to_string(),
                                            });
                                        return Ok(outcome);
                                    }
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
                        // §12.6's evidence, for the artefact that section opens
                        // on (`SIGNOFF-REPAIR.11.24.1.3.1`): *a live Web page
                        // can change, so deliberation evidence points to an
                        // immutable `EvidenceSnapshot`*. The parent is the
                        // page's OWN bytes — `.11.24.1.3.1.1` made them exist;
                        // before it, `parent_digest` digested the chunk derived
                        // from them and there was nothing here to store.
                        let document = response.document.as_bytes();
                        // ⛔ THE WORKER'S CLAIM ABOUT ITS OWN BYTES MUST HOLD.
                        // R2 refuses a receipt whose `parent_digest` is not the
                        // digest of the bytes the REQUEST supplied (`.7.3.3.3`);
                        // here the worker supplies the bytes itself, so the
                        // comparison is against what it sent. A worker whose
                        // digest disagrees with its own document is
                        // malfunctioning, and §12.6's evidence must not be built
                        // on it — nor may the receipt and the snapshot disagree
                        // about which bytes were rendered, which is what
                        // silently re-deriving would have shipped.
                        let rendered_digest = crate::fetcher::digest_sha256_hex(document);
                        if rendered_digest != response.parent_digest {
                            outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                                kind: "render_source_mismatch".to_owned(),
                                message: format!(
                                    "the worker declared `{}` for a document whose \
                                         digest is `{rendered_digest}`",
                                    response.parent_digest
                                ),
                            });
                            return Ok(outcome);
                        }
                        let snapshot = crate::snapshots::submit(
                            &state.pool,
                            &crate::snapshots::SnapshotSubmission {
                                reference_id: resource_id.to_owned(),
                                original_locator: reference.original_locator.clone(),
                                final_locator: reference.original_locator.clone(),
                                resolver_id: crate::resolvers::R3_RESOLVER_ID.to_owned(),
                                resolver_version: "0.1.0".to_owned(),
                                network_class: "public".to_owned(),
                                auth_class: "none".to_owned(),
                                // The render's disclosure (§12.6): every request
                                // the page made, and every one the pack's
                                // advertised deny-policies refused.
                                provider_receipt: serde_json::json!({
                                    "network_log": response.network_log,
                                    "refused_requests": response.refused_requests,
                                    "page_title": response.page_title,
                                    "browser_version": response.browser_version,
                                }),
                                immutable_source_version: None,
                                // Re-derived above from the bytes this process
                                // holds, and equal to the worker's claim by the
                                // refusal that precedes this block — so the
                                // snapshot is addressed by a measurement rather
                                // than by a claim, and the receipt cannot
                                // disagree with it.
                                raw_digest: rendered_digest,
                                byte_length: document.len() as i64,
                                media_type: "text/html".to_owned(),
                                storage_class: "standard".to_owned(),
                                retention_class: "standard".to_owned(),
                                extraction_version: None,
                                quarantine_status: "none".to_owned(),
                                redactions: serde_json::json!([]),
                                disclosure_policy: serde_json::json!({}),
                                license: None,
                                fresh_until: None,
                            },
                            document,
                            chrono::Utc::now(),
                            citer,
                        )
                        .await;
                        // A failed snapshot is NOT a successful acquisition —
                        // `.7.4.2`'s rule, which the R0 arm had to be migrated
                        // onto later (`.11.14.3.12`) and which this arm takes
                        // from the start. The refusal IS the outcome.
                        let Ok(snapshot) = snapshot.inspect_err(|error| {
                            crate::log_event!(
                                "acquisition_evidence_unstored",
                                "resource_id" => resource_id,
                                "reason" => error.to_string(),
                            );
                            outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                                kind: "evidence_unstored".to_owned(),
                                message: error.to_string(),
                            });
                        }) else {
                            return Ok(outcome);
                        };
                        // One edge per chunk. The parent is the snapshot this
                        // render just wrote and cited, so the resolving tenant
                        // is a citer by construction (`.11.14.3.15`).
                        for chunk in &response.chunks {
                            let _ = crate::derivations::submit(
                                &state.pool,
                                &crate::derivations::DerivationSubmission {
                                    parent_snapshot_id: snapshot.snapshot_id.clone(),
                                    derived_kind: "chunk".to_owned(),
                                    derived_digest: chunk.digest.clone(),
                                    content: chunk.text.clone(),
                                    extraction_version: None,
                                    source_selector: None,
                                },
                                &citer.tenant_id,
                            )
                            .await;
                        }
                        outcome.acquisition = Some(crate::resolvers::Acquisition::Browse(
                            crate::browse::BrowserReceipt {
                                parent_digest: response.parent_digest,
                                chunks: response.chunks,
                                network_log: response.network_log,
                                refused_requests: response.refused_requests,
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
                                    reference_id: resource_id.to_owned(),
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
                                citer,
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
                                    "resource_id" => resource_id,
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
                                return Ok(outcome);
                            };
                            // The derivation graph: every derived chunk is a
                            // Derivation edge — the parent stays addressable
                            // (a chunk is NEVER the original).
                            {
                                for chunk in &response.chunks {
                                    // The parent is the snapshot this
                                    // acquisition just wrote and cited, so the
                                    // resolving tenant is a citer by
                                    // construction (`.11.14.3.15`).
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
                                        &citer.tenant_id,
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
            // ⭐ THIS ACQUISITION WRITES AN `EvidenceSnapshot` IN §12.9's SECOND
            // STORAGE CLASS, and the reason is recorded here rather than only in
            // the task tree (`SIGNOFF-REPAIR.11.24.1.3` + `.11.24.1.3.2`).
            //
            // ⭐ THE SNAPSHOT IS OWED AND THE AUTOMATIC EDGE IS NOT, and §12.6
            // says which is which rather than leaving it to taste: *a live Web
            // page or BRANCH can change*, so the acquired tree is evidence; and
            // *a quote, summary, OCR result, model-generated caption, or
            // REPOSITORY ANALYSIS is not the original source*, which makes the
            // repository the PARENT and an analysis of it the edge. Nothing
            // here analyses the tree, so there is no edge to write — a
            // derivation invented to fill a graph would be an edge with no
            // transformation behind it.
            //
            // ⛔ AND THE BYTES ARE NOT INLINED, BECAUSE THE MEASUREMENT SAYS
            // THEY ARE NOT AN IDENTITY. §12.9 permits a snapshot to *remain
            // addressable … or retain a verifiable external archival
            // reference*; `git::tests::the_acquired_object_database_is_not_a_
            // stable_identity` acquires one immutable commit twice across a
            // server-side repack and gets two different object databases, so the
            // first alternative would file a second evidence row every time an
            // upstream forge repacks. The commit id is the stable identity, git
            // makes it a commitment to the whole tree, and re-acquiring it is
            // the verification — which is the second alternative exactly.
            match state.git_fetcher.acquire(&reference.original_locator).await {
                Ok(acquisition) => {
                    let requested_ref = url::Url::parse(&reference.original_locator)
                        .ok()
                        .and_then(|u| u.fragment().map(str::to_owned))
                        .unwrap_or_else(|| "HEAD".to_owned());
                    let receipt = crate::git::GitReceipt::from_acquisition(
                        &reference.original_locator,
                        &requested_ref,
                        &acquisition,
                        chrono::Utc::now(),
                    );
                    // A failed snapshot is NOT a successful acquisition — the
                    // `.7.4.2` rule the R0 arm had to be migrated onto later
                    // (`.11.14.3.12`) and which this arm takes from the start.
                    if let Err(error) = crate::snapshots::submit_external(
                        &state.pool,
                        &crate::git::external_snapshot_submission(
                            resource_id,
                            &reference.original_locator,
                            &receipt,
                        ),
                        chrono::Utc::now(),
                        citer,
                    )
                    .await
                    {
                        crate::log_event!(
                            "acquisition_evidence_unstored",
                            "resource_id" => resource_id,
                            "reason" => error.to_string(),
                        );
                        outcome.acquisition_error = Some(crate::resolvers::AcquisitionError {
                            kind: "evidence_unstored".to_owned(),
                            message: error.to_string(),
                        });
                        return Ok(outcome);
                    }
                    outcome.acquisition = Some(crate::resolvers::Acquisition::Git(receipt));
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
                            crate::git::GitError::AmbiguousRefSelector { .. } => {
                                "ambiguous_ref_selector".to_owned()
                            }
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
    Ok(outcome)
}

// ── The workflow-profile registry (PHASE-5.1.2; backlog 36) ─────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisterWorkflowProfileRequest {
    profile_id: String,
    steps: Vec<String>,
    reason: Reason,
}

/// `POST /v1/governance-charters` — register a charter version
/// (`SIGNOFF-REPAIR.11.4.7.2.1.2.1`; ROADMAP §4.1).
///
/// ⭐ A SITE act, by derivation rather than caution: §4.1 makes the charter the
/// document that CONSTRAINS a tenant, and §4.4 makes the enrollment boundary
/// naming its digest a root/parent-granted ceiling. A tenant that could rewrite
/// its own allowed decision rules would hold the ceiling it is bound by.
///
/// ⛔ The digest is the SERVER's product. A body may assert one, and a
/// disagreement is refused naming both — the `publications::stage` rule
/// (`.9.2.1.3.1`) applied to a second content-addressed store.
///
/// ⚠️ Registration is IDEMPOTENT over byte-identical content: the row's
/// identity is its content, so a second write asserts nothing new.
async fn register_governance_charter(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<crate::charters::CharterInput>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let input = site_request(request)?;
    // Before the gate, deliberately (`SIGNOFF-REPAIR.8.2.5.2`): §13.3's seven
    // families are a published constant the book lists, so naming the rule that
    // failed is an oracle over nothing, and a caller that fails validation
    // learns nothing about authority.
    crate::charters::validate(&input)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::charters::register_charter(&state.pool, &principal, &input).await,
        "a current site grant for this action and its actual boundary are required",
    )
}

/// `GET /v1/governance-charters/{charter_digest}` — read one charter back.
///
/// ⛔ Tenant-bound: a charter is a tenant's governance document, so a principal
/// of another tenant gets the SAME `unknown` answer an absent digest gets. The
/// read surface is not an existence oracle over other tenants' charters.
async fn read_governance_charter(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(charter_digest): Path<String>,
) -> Result<Json<crate::charters::StoredCharter>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no charter",
        ));
    };
    match crate::charters::load(&state.pool, &charter_digest).await {
        Ok(charter) if charter.tenant_id == caller_tenant => Ok(Json(charter)),
        Ok(_) | Err(crate::charters::CharterError::Unknown(_)) => {
            Err(ControlApiError::invalid_command(
                crate::charters::CharterError::Unknown(charter_digest).to_string(),
            ))
        }
        // The caller's input, or the store's fault, told apart and never
        // blurred into `invalid_command` (`SIGNOFF-REPAIR.4.4.10.1.2`): a store
        // fault was answered as the caller's 400, with the store's own error
        // text in the message.
        Err(crate::charters::CharterError::UnrepresentableInput) => Err(unrepresentable()),
        Err(crate::charters::CharterError::Storage(detail)) => Err(
            ControlApiError::internal_with_log(format!("the charter store failed: {detail}")),
        ),
        Err(other) => Err(ControlApiError::invalid_command(other.to_string())),
    }
}

/// `GET /v1/decision-rules/{rule}` — §4.1's read path: *may this tenant decide
/// under this rule?*
///
/// Resolves the caller's tenant to its ACTIVE enrollment boundary, that
/// boundary to the charter digest it was ISSUED under, and answers from that
/// charter. ⛔ Through the boundary rather than by `tenant_id` directly,
/// because §4.4 makes the boundary the ceiling: the charter a tenant is bound
/// by is the one its issuer named, not the newest anybody registered.
///
/// ⛔ FAILS CLOSED. A boundary whose `charter_digest` resolves to nothing —
/// every boundary issued before `migrations/0084` carries a label — answers
/// *the question cannot be answered*, never *allowed*.
async fn tenant_decision_rule(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(rule): Path<String>,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal asks no charter",
        ));
    };
    match crate::charters::allows(&state.pool, &caller_tenant, &rule).await {
        Ok((resolved, threshold)) => Ok(Json(json!({
            "tenant_id": caller_tenant,
            "decision_rule": resolved.as_str(),
            "allowed": true,
            "approval_threshold": threshold,
        }))),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `POST /v1/workflow-profiles` — register a profile (the operator's verb):
/// the steps MUST pass the composition validation.
///
/// 🔴 **A SITE act, not a tenant one (`.7.1.2.1`), and the doc comment said so
/// before the code did.** This route admitted any enrolled principal and
/// appended `MAX(version) + 1` under ANY `profile_id`, the eight §13.1 built-ins
/// included, while `workflows::resolve` takes the highest version site-wide with
/// no `built_in` filter — and a bare thread resolves `quick_advice` through it.
/// So one enrolled principal chose the steps every other tenant's next bare
/// thread executed. It now takes the `workflow_register` site capability and is
/// audited like every other site-wide act.
///
/// ⚠️ The body gained a required `reason`, which is a wire change and is stated
/// in `docs/book/src/site-authority.md`. Every site act is attributable: a
/// registration that changes what the whole site deliberates is exactly the kind
/// of act an operator must be able to explain afterwards.
///
/// ⛔ `GET /v1/workflow-profiles` deliberately stays on enrolment. The registry
/// is site-wide configuration a tenant must be able to READ in order to name a
/// profile on a thread, and the rows carry no tenant data — the defect was the
/// write.
async fn register_workflow_profile(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<RegisterWorkflowProfileRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    // Before the gate, deliberately: the ADR-016 vocabulary is a published
    // constant, so naming the rule that failed is an oracle over nothing, and
    // every other site route bounds its input on extraction for the same reason.
    crate::workflows::validate_steps(&req.steps)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::register_profile(
            &state.pool,
            &principal,
            &req.profile_id,
            &req.steps,
            &req.reason,
        )
        .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<
        Json<crate::evaluation::CorpusRegistration>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let registration = site_request(request)?;
    // ⛔ BEFORE THE GATE (`SIGNOFF-REPAIR.8.2.5.2`): a site refusal renders as an
    // authorization reason, so validating inside the act turns a 400 about input
    // into a 403 about authority. The write re-validates regardless.
    crate::evaluation::validate_corpus(&registration)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::register_corpus(
            &state.pool,
            &principal,
            &registration,
            &registration.reason,
        )
        .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<Json<crate::evaluation::RunRecord>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let run = site_request(request)?;
    crate::evaluation::validate_run(&run)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::record_run(&state.pool, &principal, &run, &run.reason).await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<
        Json<crate::evaluation::TrialSubmission>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let submission = site_request(request)?;
    crate::evaluation::validate_trial(&submission)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::create_trial(&state.pool, &principal, &submission, &submission.reason)
            .await,
        "a current site grant for this action and its actual boundary are required",
    )
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

/// `POST /v1/evaluations/trials/{id}/results`' body (`SIGNOFF-REPAIR.8.2.5.3`).
///
/// ⚠️ **A DOCUMENTED WIRE CHANGE.** The route took a BARE results object, and a
/// site act requires a reason for its audit record — so the results move under
/// their own key rather than a field being added beside them. `deny_unknown_fields`
/// makes the old shape a typed 400 rather than a silently ignored body.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialResultsSubmission {
    pub results: serde_json::Value,
    pub reason: crate::site_authority::Reason,
}

/// `POST /v1/evaluations/gates/{id}/evaluations`' body (`SIGNOFF-REPAIR.8.2.5.3`).
/// The same documented wire change as [`TrialResultsSubmission`], for the scores.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateEvaluation {
    pub scores: serde_json::Value,
    pub reason: crate::site_authority::Reason,
}

/// `POST /v1/evaluations/trials/{id}/results` — append one per-arm results
/// row (append-only — the record's identity is its content).
async fn record_trial_results(
    State(state): State<Arc<ApiState>>,
    Path(trial_id): Path<String>,
    headers: HeaderMap,
    request: Result<Json<TrialResultsSubmission>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let body = site_request(request)?;
    // ⚠️ Nothing to validate ahead of the gate: this route's only check is that
    // the trial exists, which needs the database (`SIGNOFF-REPAIR.8.2.5.2`).
    site_receipt_response(
        site::evaluation::record_trial_results(
            &state.pool,
            &principal,
            &trial_id,
            &body.results,
            &body.reason,
        )
        .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<
        Json<crate::evaluation::CalibrationSubmission>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let submission = site_request(request)?;
    crate::evaluation::validate_calibration(&submission)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::record_calibration(
            &state.pool,
            &principal,
            &submission,
            &submission.reason,
        )
        .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<
        Json<crate::evaluation::GateSubmission>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let submission = site_request(request)?;
    crate::evaluation::validate_gate(&submission)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::record_gate(&state.pool, &principal, &submission, &submission.reason)
            .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    request: Result<Json<GateEvaluation>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let body = site_request(request)?;
    crate::evaluation::validate_gate_scores(&body.scores)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::evaluation::evaluate_gate(
            &state.pool,
            &principal,
            &gate_id,
            &body.scores,
            &body.reason,
        )
        .await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal resolves no route",
        ));
    };
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
                &tenant,
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no resolutions",
        ));
    };
    Ok(Json(
        crate::routing::list_resolutions(&state.pool, &tenant).await?,
    ))
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no recommendation",
        ));
    };
    match crate::routing::record_recommendation(&state.pool, &submission, &tenant).await {
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no recommendations",
        ));
    };
    Ok(Json(
        crate::routing::list_recommendations(&state.pool, &tenant).await?,
    ))
}

// ── The policy registry (PHASE-6.1.2; ADR-019, backlog 38) ─────────────────────────

/// `POST /v1/policies` — register one policy version (the typed,
/// validated, digest-pinned document; the owning authority must be an
/// ACTIVE grant — the label grants nothing).
///
/// 🔴 **A SITE act, not a tenant one (`.6.1.5.4`).** This route admitted any
/// enrolled principal into a FIRST-COME identifier namespace: `policy_versions`
/// is keyed `(policy_id, version)` with no tenant column, so the first caller to
/// name a coordinate owned it and every later caller — including the rightful
/// author — was refused as a duplicate, while the text the other tenants then
/// read under that governance id was the first caller's. It now takes the
/// `policy_register` site capability and is audited like every other site act.
///
/// ⚠️ The body gained a required `reason`, and two refusals moved from 400 to
/// 403 with an audit id — a ghost owning authority and a taken coordinate. Both
/// are wire changes and both are stated in `docs/book/src/site-authority.md`.
/// They moved because each is a question about the DATABASE, and answering
/// either before the gate would hand a caller with no site authority an
/// existence oracle over the site's grants and over a registry it may not write.
/// The five refusals that ask only about the SUBMISSION stay a typed 400.
///
/// ⛔ `GET /v1/policies`, `POST /v1/policies/resolve`, the impact map and the MCP
/// policy bundle deliberately stay on enrolment. The library is shared BY DESIGN
/// (DOC-0071) — a policy only its author can read is not governance — and
/// `.6.1.5.3`'s control asserts it, so a later repair that bound a read would
/// fail there rather than silently reverse a recorded decision. The defect was
/// the write.
async fn register_policy(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<
        Json<crate::policy::PolicyVersionInput>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let input = site_request(request)?;
    // Before the gate, deliberately: the ADR-011 digest shape, the semantic
    // version, the published `LIFECYCLES` vocabulary and the stable clause ids
    // are rules over published constants, so naming the one that failed is an
    // oracle over nothing — and every other site route bounds its input on
    // extraction for the same reason.
    crate::policy::validate(&input)
        .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
    site_receipt_response(
        site::register_policy(&state.pool, &principal, &input, &input.reason).await,
        "a current site grant for this action and its actual boundary are required",
    )
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
    // `SIGNOFF-REPAIR.9.1.7`: a refusal is the request's `400`; a store that
    // could not answer is the server's, never reported as a refusal.
    match crate::policy::resolve(&state.pool, &request).await {
        Ok(Ok(resolution)) => Ok(Json(resolution)),
        Ok(Err(refusal)) => Err(ControlApiError::invalid_command(refusal.to_string())),
        Err(cause) => Err(storage_failure(cause, "the policy resolution")),
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
        Ok(Ok(map)) => Ok(Json(map)),
        Ok(Err(refusal)) => Err(ControlApiError::invalid_command(refusal.to_string())),
        Err(cause) => Err(storage_failure(cause, "the policy impact map")),
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
        Err(error) => Err(lifecycle_refusal(error)),
    }
}

/// `GET /v1/policy-proposals` — the proposals, newest first.
async fn list_policy_proposals(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredProposal>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no proposals",
        ));
    };
    Ok(Json(
        crate::lifecycle::list_proposals(&state.pool, &caller_tenant).await?,
    ))
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
        Err(error) => Err(lifecycle_refusal(error)),
    }
}

/// `GET /v1/policy-decisions` — the decisions, newest first.
async fn list_policy_decisions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredDecision>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no decisions",
        ));
    };
    Ok(Json(
        crate::lifecycle::list_decisions(&state.pool, &caller_tenant).await?,
    ))
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
    let Some(tenant_id) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no approval",
        ));
    };
    match crate::lifecycle::record_approval(&state.pool, &principal, &tenant_id, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(lifecycle_refusal(error)),
    }
}

/// `GET /v1/policy-approvals` — the approvals, newest first.
async fn list_policy_approvals(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::lifecycle::StoredApproval>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no approvals",
        ));
    };
    Ok(Json(
        crate::lifecycle::list_approvals(&state.pool, &caller_tenant).await?,
    ))
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
    // `SIGNOFF-REPAIR.6.1.5.2`: the tenant is BOUND, not tested with `is_some()`
    // and dropped — the `.6.1.5` shape this repair programme has now found at a
    // dozen sites. The projection records its AUTHOR, because it has no other
    // tenant to record.
    let Some(tenant_id) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal projects no policy",
        ));
    };
    match crate::projections::project(&state.pool, &tenant_id, &request).await {
        Ok(row) => Ok(Json(row)),
        // `SIGNOFF-REPAIR.9.1.4`: the lock's registry read can fail, and a store
        // that did not answer is not a refusal.
        Err(crate::projections::ProjectionError::Storage(cause)) => {
            Err(storage_failure(cause, "the policy projection"))
        }
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/policy-projections` — the projection records, newest first.
async fn list_policy_projections(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::projections::StoredProjection>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no projections",
        ));
    };
    Ok(Json(
        crate::projections::list(&state.pool, &caller_tenant).await?,
    ))
}

/// `POST /v1/policy-publications` — stage one publication (`.4.2`): the
/// §15.7 steps 1–4's record half (the references verified, the manifest
/// digest, the staged state).
///
/// `.9.2.1.2.2`: the caller names an `owning_authority` it HOLDS, and the row
/// records it. ⛔ This was the last of the four publication verbs on enrolment
/// alone, and the reason it binds is not symmetry: `stage` never reads the
/// proposal's policy and checks the projection only for existence and tenant,
/// so the stager chooses the bytes the approval will publish (`.9.2.1.3.2`).
async fn stage_publication(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::publications::PublicationInput>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal stages no publication",
        ));
    };
    // `.9.2.1.2.2`: authorized BEFORE any record is looked up, in the order
    // the three transition verbs already use. ⛔ `.9.2.1.2.3` removed the
    // hand-graded absence that sat here: the field is required on a TYPED
    // input, so the strict wire boundary refuses a request without it and
    // this handler never runs — which is the convention, not an exception.
    held_publication_grant(&state, &principal, &input.owning_authority).await?;
    match crate::publications::stage(&state.pool, &caller_tenant, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(publication_refusal(error)),
    }
}

/// `GET /v1/policy-publications` — the publications, newest first.
async fn list_publications(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::publications::StoredPublication>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no publications",
        ));
    };
    Ok(Json(
        crate::publications::list(&state.pool, &caller_tenant).await?,
    ))
}

/// `POST /v1/policy-publications/{id}/effective` — the staged → effective
/// transition with the Git object ids (the §15.7 step 8's record half).
///
/// `.9.2.1.3`: the declared ids must EXIST. The body therefore names the
/// repository they are claimed to live in, `repo_path`, resolved inside the
/// deployment's configured root exactly as the publish verb's is — a required
/// field, and a wire-contract change to a shipped verb, because a transition
/// that cannot say which repository it is talking about cannot check anything.
/// `.9.2.1.2`: the caller also names an `owning_authority` it HOLDS.
async fn mark_publication_effective(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<crate::publications::MarkEffectiveInput>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing effective",
        ));
    };
    // `.9.2.1.2`: authorized BEFORE anything is looked up, so an unauthorized
    // caller learns nothing about the request they were not entitled to make.
    // ⛔ `.9.2.1.2.3`: the body's SHAPE is now graded before this handler runs
    // at all, so there is nothing left here to parse out by hand.
    held_publication_grant(&state, &principal, &input.owning_authority).await?;
    let git_object_ids = input.git_object_ids;
    let repository = crate::publisher::resolve_repository(
        state.publication_repo_root.as_deref(),
        &input.repo_path,
    )
    .map_err(publication_repository_refused)?;
    match crate::publications::mark_effective(
        &state.pool,
        &caller_tenant,
        &publication_id,
        git_object_ids,
        &repository,
    )
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(publication_refusal(error)),
    }
}

/// A governance record's error on the wire: a store fault is the server's
/// (`500`, logged), never a missing record (`SIGNOFF-REPAIR.9.2.3`).
fn lifecycle_refusal(error: crate::lifecycle::LifecycleError) -> ControlApiError {
    match error {
        crate::lifecycle::LifecycleError::Storage(detail) => {
            eprintln!("control api: the governance store failed: {detail}");
            ControlApiError::internal()
        }
        crate::lifecycle::LifecycleError::UnrepresentableInput => unrepresentable(),
        refusal => ControlApiError::invalid_command(refusal.to_string()),
    }
}

/// A deployment verb's error on the wire: a store fault is the server's (`500`,
/// logged), never a refusal of the caller's request; input the store cannot
/// represent is the caller's `400 unrepresentable_input`. ⚠️ The reporter lookup
/// (`SIGNOFF-REPAIR.9.3.3.2`), the receipt write and the receipt history
/// (`.9.3.3.3`) classify through `DeploymentError::storage`; the module's older
/// store sites still answer as refusals and are `.9.3.3.6`'s.
fn deployment_refusal(error: crate::deployments::DeploymentError) -> ControlApiError {
    match error {
        crate::deployments::DeploymentError::Storage(detail) => {
            eprintln!("control api: the deployment store failed: {detail}");
            ControlApiError::internal()
        }
        crate::deployments::DeploymentError::UnrepresentableInput => unrepresentable(),
        refusal => ControlApiError::invalid_command(refusal.to_string()),
    }
}

/// A publication transition's error on the wire: a store fault is the server's
/// (`500`, logged), never a refusal of the caller's request, which is how both
/// transition verbs used to answer it (`SIGNOFF-REPAIR.9.2.2`, `.7.4.2`'s rule).
fn publication_refusal(error: crate::publications::PublicationError) -> ControlApiError {
    match error {
        crate::publications::PublicationError::Storage(detail) => {
            eprintln!("control api: the publication store failed: {detail}");
            ControlApiError::internal()
        }
        crate::publications::PublicationError::UnrepresentableInput => unrepresentable(),
        refusal => ControlApiError::invalid_command(refusal.to_string()),
    }
}

/// `POST /v1/policy-publications/{id}/failed` — the typed failure (never a
/// skip) with the reason.
///
/// `.9.2.1.2.1`: the caller also names an `owning_authority` it HOLDS. ⛔ This
/// verb was left behind by `.9.2.1.2`, whose title and reproduce line named
/// the other two, so the one transition that is TERMINAL — `mark_effective`
/// and `publish` both refuse a publication that is no longer `staged` — was
/// the one reachable on enrolment alone.
async fn mark_publication_failed(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<crate::publications::MarkFailedInput>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing failed",
        ));
    };
    // `.9.2.1.2.1`: authorized first, in the order the two siblings already
    // use. ⛔ `.9.2.1.2.3`: the shape is graded at the wire boundary now.
    held_publication_grant(&state, &principal, &input.owning_authority).await?;
    let reason = input.reason.as_str();
    match crate::publications::mark_failed(&state.pool, &caller_tenant, &publication_id, reason)
        .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(publication_refusal(error)),
    }
}

/// All FOUR publication verbs are bound to a grant the caller HOLDS
/// (`SIGNOFF-REPAIR.9.2.1.2` for `effective` and `publish`, `.9.2.1.2.1` for
/// `failed`, `.9.2.1.2.2` for `stage`), in the shape
/// `deployments::register_target` already uses: the request names an
/// `owning_authority` and [`authority::grant_held_by`] decides.
///
/// ⛔ The body-reading wrapper this doc used to introduce is GONE
/// (`.9.2.1.2.3`). It existed only because three of the four verbs took an
/// untyped `serde_json::Value` and had to fish the field out; now all four are
/// typed, so every caller already holds the grant id and the shared thing is
/// the decision alone. ⚠️ Its own comment claimed *"a later change cannot move
/// one verb without the other"* — a property that was never true of a verb
/// that did not call it, which is how `failed` went unbound
/// (`.9.2.1.2.1`).
/// The DECISION itself, over a grant id this caller already has in hand.
///
/// ⛔ Split out by `SIGNOFF-REPAIR.9.2.1.2.2` so the staging verb can ask the
/// same question, and split rather than copied for the reason the wrapper
/// above records: this is THE predicate (`.9.3.1`), never a fourth spelling.
/// The three transition verbs take an untyped body and read the field out of
/// it; `stage_publication` takes a TYPED input and already has the value, so
/// the shared thing is the decision and not the extraction.
async fn held_publication_grant(
    state: &ApiState,
    principal: &GrantSubject,
    owning_authority: &str,
) -> Result<(), ControlApiError> {
    // `.9.3.4.2`: HELD **and COVERING**. `policy_publication_write` is the one
    // action for all four publication verbs — `.9.3.4.1` measured that
    // splitting staging from the transitions had no operator need behind it.
    let held = authority::grant_held_by(
        &state.pool,
        owning_authority,
        principal,
        GrantAction::PolicyPublicationWrite,
    )
    .await
    .map_err(|_| ControlApiError::internal())?;
    if !held {
        return Err(ControlApiError::unauthorized(
            "the publication verbs require an authority the caller HOLDS that COVERS \
             `policy_publication_write` — naming a grant is not holding one, and holding \
             one is not being authorized for this verb",
        ));
    }
    Ok(())
}

/// Map a publication-repository refusal onto the wire (`.9.2.1.1`).
///
/// The two DEPLOYMENT faults answer `publication_repository_unconfigured`,
/// because no request can fix them; the two CALLER faults answer
/// `invalid_command`. ⛔ The configured root is never echoed to the caller — it
/// is a server-side path, so it goes to the log the way
/// [`ControlApiError::internal_with_log`] sends its detail there.
fn publication_repository_refused(refusal: crate::publisher::RepositoryRefusal) -> ControlApiError {
    use crate::publisher::RepositoryRefusal as Refusal;
    match refusal {
        Refusal::Unconfigured => ControlApiError::publication_repository_unconfigured(
            "this deployment declares no publication repository root — the publish verb is closed until an operator configures one",
        ),
        Refusal::RootUnusable { root, reason } => {
            eprintln!(
                "control api: the configured publication repository root `{root}` is not usable: {reason}"
            );
            ControlApiError::publication_repository_unconfigured(
                "the deployment's publication repository root is not usable — an operator must repair it",
            )
        }
        refusal @ (Refusal::Unresolvable { .. } | Refusal::Outside { .. }) => {
            ControlApiError::invalid_command(refusal.to_string())
        }
    }
}

/// `POST /v1/policy-publications/{id}/publish` — the Git publication half
/// (`.4.3.2`): the staged publication's bundle + the composed manifest
/// ride the publisher (the staging branch, the fetch-back verification,
/// the immutable ref, the effective channel via the CAS), then the record
/// marks effective. The repository is the deployment's CONFIGURED root
/// (`.9.2.1.1`): `repo_path` names a location inside it and is resolved
/// against it, and a deployment that declares no root refuses the verb.
/// `.9.2.1.2`: the caller also names an `owning_authority` it HOLDS.
/// `GET /v1/policy-bundles/{manifest_digest}` (`SIGNOFF-REPAIR.9.3.5.2`): an
/// effective publication's bundle, served by its manifest digest and VERIFIED
/// on the way out — the stored manifest must hash to the digest it is asked
/// for, and the stored bundle to the projection digest that manifest names.
/// Content that does not is refused (`409 publication_conflict`), never served
/// with a warning. Tenant-bound: a foreign digest answers as an absent one.
async fn read_policy_bundle(
    State(state): State<Arc<ApiState>>,
    Path(manifest_digest): Path<String>,
    headers: HeaderMap,
) -> Result<Json<crate::publications::ServedBundle>, ControlApiError> {
    use crate::publications::PublicationError as E;
    let principal = resolve_principal(&headers)?;
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no bundle",
        ));
    };
    match crate::publications::serve_bundle(
        &state.pool,
        state.publication_repo_root.as_deref(),
        &caller_tenant,
        &manifest_digest,
    )
    .await
    {
        Ok(served) => Ok(Json(served)),
        Err(e @ E::MalformedDigest(_)) => Err(ControlApiError::invalid_command(e.to_string())),
        Err(e @ (E::NotPublished(_) | E::BundleUnlocatable(_))) => {
            Err(ControlApiError::not_found(e.to_string()))
        }
        Err(e @ E::BundleIntegrity { .. }) => {
            eprintln!("control api: a published bundle failed verification: {e}");
            Err(ControlApiError::publication_conflict(e.to_string()))
        }
        Err(e) => {
            eprintln!("control api: a bundle read failed: {e}");
            Err(ControlApiError::internal())
        }
    }
}

async fn publish_publication(
    State(state): State<Arc<ApiState>>,
    Path(publication_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<crate::publications::PublishInput>,
) -> Result<Json<crate::publications::StoredPublication>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal publishes nothing",
        ));
    };
    // `.9.2.1.2`: authorized BEFORE the path is resolved or the publication is
    // loaded — an unauthorized caller reaches neither the filesystem nor the
    // database. ⛔ `.9.2.1.2.3`: the shape is graded at the wire boundary now.
    held_publication_grant(&state, &principal, &input.owning_authority).await?;
    let requested = input.repo_path.as_str();
    // `.9.2.1.1`: the caller names a location INSIDE the deployment's
    // configured root, and it is resolved here — before the publication is
    // loaded and before anything is written. An untrusted path is refused
    // before it reaches the database, not after.
    let repo_path =
        crate::publisher::resolve_repository(state.publication_repo_root.as_deref(), requested)
            .map_err(publication_repository_refused)?;
    // ⚠️ A SUPPLIED value that will not parse stays a `400`: whether a string
    // is a Git object id is a semantic question the handler answers, not a
    // shape the deserializer can express (`.9.2.1.2.3`).
    let expected_effective = input
        .expected_effective
        .as_deref()
        .map(|hex| {
            hex.parse::<gix::ObjectId>().map_err(|_| {
                ControlApiError::invalid_command(format!(
                    "the expected_effective `{hex}` is malformed"
                ))
            })
        })
        .transpose()?;
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: the publication must be the caller's, and the
    // check sits HERE rather than beside the authority check above, because the
    // ordering is load-bearing in both directions. `.9.2.1.2` requires the
    // AUTHORITY to answer before a path is resolved; the containment control
    // requires a path escape to be reported as a path escape rather than as a
    // missing record — its positive arm is *an inside location reaches the
    // lookup*. So ownership answers after containment and before the publication
    // is read, which is the first point at which this verb learns anything about
    // the record. ⚠️ A non-owner reaches only path validation, which discloses
    // nothing about the publication and is already reachable by any authority
    // holder regardless of ownership.
    crate::publications::owned_by(&state.pool, &publication_id, &caller_tenant)
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
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
    // `.9.2.1.3.1`: ONE definition of the manifest, shared with `stage`, so
    // the bytes written here and the digest stored there cannot drift.
    let manifest = crate::publications::manifest(
        &publication.publication_id,
        &publication.proposal_id,
        &publication.decision_id,
        &publication.approval_id,
        &publication.projection_id,
        &projection.digest,
    );
    // ⛔ THE STORED DIGEST FINALLY HAS A READER (`.9.2.1.3.1`). Until here it
    // was written at staging and consulted by nothing, so a publication could
    // be published against a manifest that did not match the one it was staged
    // under. The comparison happens BEFORE the publisher is called: a
    // disagreement is a typed refusal, never a write followed by a complaint.
    let composed_digest = crate::publications::manifest_digest(&manifest);
    if composed_digest != publication.manifest_digest {
        return Err(ControlApiError::invalid_command(format!(
            "publication `{publication_id}` was staged under manifest digest `{}`, \
             and the manifest this publish composes digests to `{composed_digest}` — \
             the compiled inputs moved under the staged publication",
            publication.manifest_digest
        )));
    }
    // `SIGNOFF-REPAIR.9.3.5.1.1` (§15.7 step 4): the desired Git operation is
    // COMMITTED before the first Git object is written, so a publish that dies
    // anywhere after this line leaves a row naming where to look.
    let repository =
        crate::publisher::root_relative(state.publication_repo_root.as_deref(), &repo_path)
            .map_err(publication_repository_refused)?;
    crate::publisher::opens(&repo_path)
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    crate::publications::record_git_operation(
        &state.pool,
        &caller_tenant,
        &publication_id,
        &repository,
        input.expected_effective.as_deref(),
    )
    .await
    .map_err(publication_refusal)?;
    let refs = crate::publisher::publish(
        repo_path.path(),
        &publication_id,
        &manifest,
        &projection.bytes,
        expected_effective,
        publication.staged_at_seconds,
    )
    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let git_object_ids = vec![refs.publication_ref_id, refs.effective_ref_id];
    let row = crate::publications::mark_effective(
        &state.pool,
        &caller_tenant,
        &publication_id,
        git_object_ids,
        &repo_path,
    )
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
    match crate::deployments::register_target(&state.pool, &principal, &input).await {
        Ok(()) => Ok(Json(json!({ "target_id": input.target_id }))),
        Err(error) => Err(deployment_refusal(error)),
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
    // `SIGNOFF-REPAIR.9.3.3.2`: `reporter` is `null` for a target registered
    // before `migrations/0114`, which takes no receipt.
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT target_id, target_type, owning_authority, reporter \
         FROM deployment_targets ORDER BY target_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|(target_id, target_type, owning_authority, reporter)| {
                json!({
                    "target_id": target_id,
                    "target_type": target_type,
                    "owning_authority": owning_authority,
                    "reporter": reporter,
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
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal assigns nothing",
        ));
    };
    match crate::deployments::assign(&state.pool, &caller_tenant, &input).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(deployment_refusal(error)),
    }
}

/// `GET /v1/deployments` — the assignments with the desired/observed pair.
async fn list_deployments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::deployments::StoredAssignment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: an assignment is read by the tenant that owns
    // its publication — DOC-0071's ruling, implemented here because no child of
    // `.6.1.5` named this table and a decided-but-unbound read owned by nobody
    // is how a data model gets decided by accident.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no deployments",
        ));
    };
    Ok(Json(
        crate::deployments::list_assignments(&state.pool, &caller_tenant).await?,
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
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no receipt",
        ));
    };
    match crate::deployments::record_receipt(
        &state.pool,
        &principal,
        &caller_tenant,
        &target_id,
        &publication_id,
        &input,
    )
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(deployment_refusal(error)),
    }
}

/// `GET /v1/deployments/{target_id}/{publication_id}/receipts` — one
/// assignment's receipts in the order filed (`SIGNOFF-REPAIR.9.3.3.3`): read by
/// the tenant that owns the assignment's publication, as `GET /v1/deployments` is.
async fn list_deployment_receipts(
    State(state): State<Arc<ApiState>>,
    Path((target_id, publication_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::deployments::StoredReceipt>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no receipts",
        ));
    };
    crate::deployments::list_receipts(&state.pool, &caller_tenant, &target_id, &publication_id)
        .await
        .map(Json)
        .map_err(deployment_refusal)
}

/// `POST /v1/policy-drift` — record one drift observation (`.5.3`): the
/// categorized desired/observed pair.
async fn record_policy_drift(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::corrections::DriftInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no drift",
        ));
    };
    match crate::corrections::record_drift(&state.pool, &caller_tenant, &input).await {
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
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no drift",
        ));
    };
    Ok(Json(
        crate::corrections::list_drift(&state.pool, &caller_tenant).await?,
    ))
}

/// `POST /v1/policy-corrections` — record one correction (`.5.3`): the
/// §4.7 operation + the authority proof (the retraction never deletes).
async fn record_policy_correction(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(input): Json<crate::corrections::CorrectionInput>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no correction",
        ));
    };
    match crate::corrections::record_correction(&state.pool, &principal, &caller_tenant, &input)
        .await
    {
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
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no corrections",
        ));
    };
    Ok(Json(
        crate::corrections::list_corrections(&state.pool, &caller_tenant).await?,
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
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal records no outcome",
        ));
    };
    match crate::corrections::record_outcome(&state.pool, &caller_tenant, &input).await {
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
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no outcomes",
        ));
    };
    Ok(Json(
        crate::corrections::list_outcomes(&state.pool, &caller_tenant).await?,
    ))
}

/// `POST /v1/policy-reviews/schedule` — evaluate the DUE reviews (`.6`):
/// the outcomes' named triggers + the drift + the repeated waivers, one
/// due review per (publication, trigger).
async fn schedule_policy_reviews(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::reviews::StoredReview>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal schedules no reviews",
        ));
    };
    Ok(Json(
        crate::reviews::schedule_reviews(&state.pool, &caller_tenant).await?,
    ))
}

/// `GET /v1/policy-reviews` — the reviews, newest first.
async fn list_policy_reviews(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::reviews::StoredReview>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.3`: the tenant is BOUND and PASSED to the read.
    // `.6.1.5.2` recorded an owner on every lifecycle row and `.6.1.5.2.1` gated
    // every write; this is the third half — until now the list returned every
    // tenant's governance trail to any enrolled caller.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no reviews",
        ));
    };
    Ok(Json(
        crate::reviews::list_reviews(&state.pool, &caller_tenant).await?,
    ))
}

/// `POST /v1/policy-reviews/{id}/done` — the due → done transition.
async fn mark_policy_review_done(
    State(state): State<Arc<ApiState>>,
    Path(review_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<crate::reviews::StoredReview>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // `SIGNOFF-REPAIR.6.1.5.2.1`: the tenant is BOUND, not tested with
    // `is_some()` and dropped. `.6.1.5.2` gave every lifecycle row an owner and
    // deliberately gated nothing, so this verb still acted on another tenant's
    // record until here.
    let Some(caller_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal marks nothing done",
        ));
    };
    match crate::reviews::mark_done(&state.pool, &caller_tenant, &review_id).await {
        Ok(row) => Ok(Json(row)),
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

// ── The claim-evidence graph (PHASE-4.6.3; backlog 35) ──────────────────────────────

/// `POST /v1/assessments` — submit the assessment (a tenant that CITED the
/// snapshot; the citation is VALIDATED: the excerpt must appear in the
/// snapshot's raw bytes — citation existence alone never satisfies an evidence
/// gate).
///
/// This is the NON-DELIBERATION path, and its rows are recorded in the
/// `external` namespace (`SIGNOFF-REPAIR.11.14.3.3`): `claim_id` here is a
/// caller label rather than the digest the `assess` step mints and
/// membership-checks against a thread. The namespace is part of the row's
/// identity, so a caller naming a real thread's claim digest writes its own
/// row and reads its own id back instead of aliasing the deliberation's.
///
/// ⛔ The route used to admit any enrolled principal, which made it the one
/// surface naming a `snapshot_id` that was not citation-bound
/// (`SIGNOFF-REPAIR.11.14.3.8`). The gate is `claims::submit`'s, so it cannot
/// be reached around; the compatibility break is recorded in
/// `docs/decisions/2026-09-17_the-standalone-assessment-is-citation-bound.md`.
///
/// ⛔ The author is the authenticated principal, and the body names none
/// (`SIGNOFF-REPAIR.7.4.7`): it used to be a body string, so any principal
/// could attribute an assessment to anyone.
async fn submit_assessment(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<
        Json<crate::claims::AssessmentRequest>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let submission = json_body(request)?.authored_by(&principal.id_string());
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no assessment",
        ));
    };
    let mut conn = state.pool.acquire().await?;
    match crate::claims::submit(
        &mut *conn,
        &submission,
        &tenant,
        crate::claims::ClaimNamespace::External,
    )
    .await
    {
        Ok(assessment_id) => Ok(Json(json!({ "assessment_id": assessment_id }))),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::claims::AssessmentError::Storage(cause)) => {
            Err(storage_failure(cause, "claim assessment storage failed"))
        }
        // A reused key with a different payload: the command API's own rule
        // and code (`SIGNOFF-REPAIR.7.4.8`).
        Err(error @ crate::claims::AssessmentError::ReplayMismatch { .. }) => {
            Err(ControlApiError {
                status: StatusCode::CONFLICT,
                code: "idempotency_mismatch",
                message: error.to_string(),
            })
        }
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}/assessments` — the snapshot's assessments that
/// THIS TENANT AUTHORED, for a tenant that cited the snapshot.
///
/// Two gates, and both are needed. The citation gate (`.11.14.1`) keeps a
/// tenant from learning that a snapshot exists at all. The authoring gate
/// (`.11.14.2`) keeps two tenants that BOTH cite one shared row from reading
/// each other's analytical position on it — the residual `.11.14.1` measured
/// and could not close, because the citation both of them hold is exactly what
/// its own gate admits on.
async fn list_snapshot_assessments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<Vec<crate::claims::StoredAssessment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no assessments",
        ));
    };
    cited_snapshot(&state.pool, &snapshot_id, &tenant).await?;
    Ok(Json(
        crate::claims::assessments_for_snapshot(&state.pool, &snapshot_id, &tenant).await?,
    ))
}

/// `GET /v1/claims/{claim_id}/assessments` — the claim's assessments that THIS
/// TENANT AUTHORED (`.11.14.2`).
///
/// ⛔ There is no citation gate here and there cannot be: a claim spans
/// snapshots and this route names no snapshot. ⚠️ `claim_id` is caller-supplied
/// text the server never mints — `git grep -n "CREATE TABLE claims" --
/// migrations` returns nothing — so before this binding the route was an oracle
/// over guessable identifiers, which is strictly worse than `.11.14.1`'s
/// enumeration. The authoring gate is what makes a guessed identifier useless;
/// the namespace itself is `.11.14.3`'s.
async fn list_claim_assessments(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(claim_id): Path<String>,
) -> Result<Json<Vec<crate::claims::StoredAssessment>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no assessments",
        ));
    };
    Ok(Json(
        crate::claims::assessments_of_claim(&state.pool, &claim_id, &tenant).await?,
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no derivation",
        ));
    };
    // A derivation is filed against a snapshot THIS TENANT CITED
    // (`SIGNOFF-REPAIR.11.14.3.15`) — the binding every read of a snapshot
    // already carries, and the surface `.11.14.3.8`'s census did not see because
    // the parent is named in the BODY.
    match crate::derivations::submit(&state.pool, &submission, &tenant).await {
        Ok(derivation_id) => Ok(Json(json!({ "derivation_id": derivation_id }))),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::derivations::DerivationError::Storage(cause)) => {
            Err(storage_failure(cause, "derivation storage failed"))
        }
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}/derivations` — the parent/derived traversal
/// (the snapshot's children, oldest first), for a tenant that cited the
/// parent (`.11.14.1`).
async fn list_derivations(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<Vec<crate::derivations::StoredDerivation>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no derivations",
        ));
    };
    cited_snapshot(&state.pool, &snapshot_id, &tenant).await?;
    Ok(Json(
        crate::derivations::children_of(&state.pool, &snapshot_id).await?,
    ))
}

/// The child reads' shared gate (`SIGNOFF-REPAIR.11.14.1`): a snapshot this
/// tenant did not cite is reported ABSENT rather than forbidden, so the
/// refusal cannot be used to confirm that an identifier exists — which is the
/// enumeration the binding closes. The message is the same one `get_snapshot`
/// returns for an identifier that truly does not exist.
async fn cited_snapshot(
    pool: &PgPool,
    snapshot_id: &str,
    tenant: &str,
) -> Result<(), ControlApiError> {
    let mut conn = pool.acquire().await?;
    if crate::snapshots::is_cited_by(&mut *conn, snapshot_id, tenant).await? {
        return Ok(());
    }
    Err(ControlApiError::not_found(format!(
        "no snapshot `{snapshot_id}`"
    )))
}

/// `POST /v1/snapshots/expire-due` — the retention enforcement: the live
/// snapshots whose class TTL passed are TOMBSTONED with the reason.
///
/// A SITE act, not a tenant one (`.7.4.3`). Which snapshots are due is a
/// property of the shared row's `retention_class` rather than of any one
/// tenant's citation, so the sweep takes the `evidence_expire` site capability
/// and is audited like every other site-wide act.
///
/// ⛔ The caller supplies a reason and NOT a time. The route previously took an
/// unbounded `at` from the request body — a test affordance left on a
/// production route — so one enrolled principal naming a far-future instant
/// tombstoned every tenant's live evidence, stamping each row with "the
/// retention expired" when it had not. The cutoff is now the database's own
/// clock, read inside the transaction that writes the tombstones and their
/// audit record.
async fn expire_due_snapshots(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Result<Json<SiteReasonRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    site_receipt_response(
        site::expire_evidence(&state.pool, &principal, &req.reason).await,
        "a current site grant for this action and its actual boundary are required",
    )
}

/// `GET /v1/snapshots/stale` — the staleness surface (the LIVE snapshots
/// THIS TENANT CITED whose freshness horizon passed — the assessments read
/// this).
///
/// This list was the cross-tenant enumeration (`.11.14.1`): admitted on
/// enrolment and unfiltered, it handed any enrolled principal every other
/// tenant's locators, resolvers and credential classes without requiring a
/// single identifier to be guessed.
async fn list_stale_snapshots(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::snapshots::StoredSnapshot>>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no staleness",
        ));
    };
    Ok(Json(
        crate::snapshots::stale_for_tenant(&state.pool, chrono::Utc::now(), &tenant).await?,
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
///
/// The submitting tenant is recorded as a CITER (`.11.14.1`), on the replay
/// as well as on the fresh insert. The write stays open to any enrolled
/// principal — narrowing the read without recording the write is exactly the
/// shape that hides a row from its own author (`.6.1.5`).
async fn submit_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<SnapshotRequest>,
) -> Result<Json<crate::snapshots::SnapshotOutcome>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no snapshot",
        ));
    };
    let bytes = base64_decode(&request.bytes_base64).ok_or_else(|| {
        ControlApiError::invalid_command("the bytes_base64 field is not valid base64")
    })?;
    match crate::snapshots::submit(
        &state.pool,
        &request.submission,
        &bytes,
        chrono::Utc::now(),
        &citer(&principal, tenant),
    )
    .await
    {
        Ok(outcome) => Ok(Json(outcome)),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`). The cause is logged
        // server-side; the wire keeps the safe generic message.
        Err(crate::snapshots::SnapshotError::Storage(cause)) => {
            Err(storage_failure(cause, "snapshot storage failed"))
        }
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
    }
}

/// `GET /v1/snapshots/{id}` — the read, for a tenant that cited the snapshot
/// (the tombstone state rides the row).
///
/// A row this tenant did not cite is ABSENT, not forbidden (`.11.14.1`): the
/// two answers are indistinguishable on the wire by design.
async fn get_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
) -> Result<Json<crate::snapshots::StoredSnapshot>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no snapshot",
        ));
    };
    match crate::snapshots::get_for_tenant(&state.pool, &snapshot_id, &tenant).await? {
        Some(snapshot) => Ok(Json(snapshot)),
        None => Err(ControlApiError::not_found(format!(
            "no snapshot `{snapshot_id}`"
        ))),
    }
}

/// `DELETE /v1/snapshots/{id}` — WITHDRAW this tenant's citation: it stops
/// relying on the snapshot, and the shared row is untouched.
///
/// ⛔ This verb used to tombstone the row, and `SIGNOFF-REPAIR.7.4.4` measured
/// what that meant. A snapshot two tenants cite is ONE row — the pair key and
/// ADR-011's digest addressing make it so — and `.11.14.1`'s binding to a
/// CITING tenant does not separate two citers, because both are citers. So one
/// tenant's deletion removed the evidence from the other's surface and stamped
/// its receipt with a reason it never wrote, irreversibly: the `UPDATE` carries
/// `AND deleted_at IS NULL` and nothing in the codebase clears the column.
///
/// Two acts were conflated in one verb. *Withdrawing a citation* is a statement
/// about this tenant's own reliance and is plainly the tenant's to make.
/// *Tombstoning the row* is a statement about bytes other tenants cite, which
/// needs the authority the retention sweep needs — it is now
/// `POST /v1/snapshots/{id}/tombstone`, a site-operator act.
///
/// The withdrawal is RECORDED, not deleted (§12.9): the citation row keeps
/// `withdrawn_at`, `withdrawn_by` and `withdrawal_reason`, so *who stopped
/// relying on this, when and why* still has an answer. Re-citing restores it.
///
/// ⚠️ The reason stays REQUIRED, and the field keeps its name. A caller that
/// sent `{"reason": …}` to delete still sends it to withdraw; what changed is
/// the `tombstoned` field in the reply, which became `withdrawn` because
/// reporting a tombstone that did not happen would be the more damaging
/// compatibility choice.
async fn withdraw_snapshot_citation(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal withdraws no citation",
        ));
    };
    cited_snapshot(&state.pool, &snapshot_id, &tenant).await?;
    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ControlApiError::invalid_command("the reason is required"))?;
    let withdrawn = crate::snapshots::withdraw_citation(
        &state.pool,
        &snapshot_id,
        &tenant,
        &actor_handle_for_subject(&principal).to_string(),
        reason,
    )
    .await?;
    Ok(Json(
        json!({ "snapshot_id": snapshot_id, "withdrawn": withdrawn }),
    ))
}

/// `POST /v1/snapshots/{id}/tombstone` — the site-operator tombstone of one
/// named row (`SIGNOFF-REPAIR.7.4.4`).
///
/// Saying "this evidence must not be relied upon by anyone" is a statement
/// about a SHARED row, so it takes the same authority the retention sweep takes
/// and is audited the same way. A tenant withdraws its own citation instead
/// (`DELETE /v1/snapshots/{id}`).
async fn tombstone_snapshot(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
    request: Result<Json<SiteReasonRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let req = site_request(request)?;
    site_receipt_response(
        site::tombstone_evidence(&state.pool, &principal, &snapshot_id, &req.reason).await,
        "a current site grant for this action and its actual boundary are required",
    )
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input).ok()
}

// ── The universal resource reference (PHASE-4.1.2; backlog 31) ─────────────────────

/// `POST /v1/resources` — submit the typed §12.1 reference (any enrolled
/// principal; the reference is declarative — the resolution is `.1.3`'s).
///
/// The identity is the `(original_locator, expected_digest)` PAIR
/// (`SIGNOFF-REPAIR.11.14.3.2`): the same pair is the replay, and the same
/// locator at a DIFFERENT digest is a second reference, because §12.6's page
/// changes and §12.1 forbids erasing that distinction. ⛔ The former
/// `locator_digest_conflict` is retired — it leaked the cross-tenant existence
/// §9.8 forbids, and it let one principal make a locator uncitable by everyone
/// else.
async fn submit_resource(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(reference): Json<crate::resources::ResourceReference>,
) -> Result<Json<crate::resources::SubmitOutcome>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal submits no reference",
        ));
    };
    let submitted_by = actor_handle_for_subject(&principal).to_string();
    // The tenant that registered the reference is the tenant whose detail read
    // must show it (`SIGNOFF-REPAIR.11.14.3.4`) — a registration nobody records
    // is a reference nobody can read. Written on the replay too, so one tenant
    // naming a pair never makes it unreadable by another.
    let registrant = crate::resources::Registrant {
        tenant_id: tenant,
        principal: submitted_by.clone(),
        // ⛔ The credential selector goes on the REGISTRATION, not the shared
        // reference (`SIGNOFF-REPAIR.11.14.3.10`). On the shared row it was
        // inherited by every tenant that replayed the pair.
        credential_binding_ref: reference.credential_binding_ref.clone(),
    };
    let mut conn = state.pool.acquire().await?;
    match crate::resources::submit(&mut *conn, &reference, &submitted_by, &registrant).await {
        Ok(outcome) => Ok(Json(outcome)),
        // A store fault is the server's problem and must not be reported as
        // though the caller's input were wrong (`.7.4.2`).
        Err(crate::resources::ReferenceError::Storage(cause)) => {
            Err(storage_failure(cause, "the reference submit failed"))
        }
        // Every other variant IS about the input, and it lives in the store so
        // both writers — this route and a contribution's citation — apply one
        // rule: the ADR-011 digest (`.11.14.3.2`). ⛔ The declared `scheme` is
        // deliberately NOT among them; `.11.14.3.5` measured that it is the
        // resolver-selection key rather than the locator's URI scheme.
        Err(error) => Err(ControlApiError::invalid_command(error.to_string())),
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
    let Some(tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal reads no reference",
        ));
    };
    let Some((id, reference, submitted_by, created_at)) =
        crate::resources::get_for_tenant(&state.pool, &resource_id, &tenant).await?
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
    /// The thread whose activity caused this initiation, when there is one
    /// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.1`). It fixes the new thread's place in
    /// its causation chain: its depth, and the roles already on it.
    #[serde(default)]
    caused_by: Option<String>,
    /// The caller's key for THIS initiation (`SIGNOFF-REPAIR.5.2` clause 1):
    /// a transport redelivery carrying the same key replays the stored answer;
    /// a new initiation carries a new key. Required, and never empty. Until
    /// this field existed the key was `auto_{role}_{tenant}`, so a role could
    /// initiate exactly once per tenant, ever.
    idempotency_key: String,
}

/// `POST /v1/threads/auto` — the node-initiated thread creation (`.3.5.3`):
/// the role needs the EXPLICIT `thread:create:auto` grant, and the §11.5
/// wake checklist evaluates server-side BEFORE the initiation lands — the
/// topic gate (the declared interests cover the topics), the confidentiality
/// match, the concurrency gate, and the grant's spend bound. Replies do NOT
/// inherit the permission (a child thread needs its own grant). The RATE
/// bound (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1`) is the `initiator` quota, checked
/// inside the create transaction by `run_thread_command` where it can commit
/// with the thread it admits.
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
    // `SIGNOFF-REPAIR.5.2` clause 1: the key is the CALLER's, per initiation.
    // An empty one would collapse every initiation onto `auto_{role}_` — the
    // once-per-tenant defect in a new spelling — so it is refused outright.
    if req.idempotency_key.is_empty() {
        return Err(ControlApiError::invalid_command(
            "idempotency_key must name this initiation",
        ));
    }

    // 1. THE grant: the explicit `thread:create:auto` authority (audited).
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::ThreadCreateAuto,
        target: ResourceTarget::Tenant { tenant_id },
    };
    let admitting_record = match authority::authorize_guarded(&state.pool, &authz).await? {
        AuthorizationOutcome::Denied { reason, record_id } => {
            crate::telemetry::metrics().incr("authorization_denials");
            return Err(ControlApiError::unauthorized(format!(
                "authorization denied ({record_id}): {reason}"
            )));
        }
        AuthorizationOutcome::Allowed { record_id, .. } => record_id,
    };
    // The bounds are the ADMITTING grant's — the one the record names — never
    // `max()` over the subject's grants (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.2`,
    // discharging `.5.2` clause 2): an expired or not-yet-valid grant cannot
    // admit, so it cannot raise the ceiling either.
    let admitting: Option<(String, Option<Value>, Option<Value>)> = sqlx::query_as(
        "SELECT g.grant_id, g.spend_limits, g.auto_bounds FROM authorization_records r \
         JOIN authority_grants g ON g.grant_id = r.grant_id WHERE r.record_id = $1",
    )
    .bind(&admitting_record)
    .fetch_optional(&state.pool)
    .await?;
    let Some((admitting_grant, spend_limits, auto_bounds)) = admitting else {
        return Err(ControlApiError::internal_with_log(format!(
            "authorization record `{admitting_record}` admitted and names no grant"
        )));
    };
    let auto_bounds: Option<reasonbraid_core::AutoBounds> = auto_bounds
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| {
            ControlApiError::internal_with_log(format!("stored auto bounds unreadable: {e}"))
        })?;

    // 2. THE §11.5 checklist (server-side). The instant rides the same read,
    //    from the database's clock, for the operating-hours gate below.
    let profile_row: Option<(Value, DateTime<Utc>)> = sqlx::query_as(
        "SELECT v.profile, now() \
         FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
         WHERE v.role_id = $1 AND v.version = p.current_version",
    )
    .bind(role.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((profile_json, now)) = profile_row else {
        return Err(ControlApiError::unauthorized(
            "the role declares no profile — the auto-wake cannot be evaluated",
        ));
    };
    let profile: crate::profiles::AgentProfile = serde_json::from_value(profile_json)
        .map_err(|e| ControlApiError::internal_with_log(format!("profile unreadable: {e}")))?;

    // a. The topic gate: every initiation topic must ride the DECLARED
    //    interests (the auto-wake-for-topic rule) — the ROLE's own declaration.
    for topic in &req.topics {
        if !profile.interests.contains(topic) {
            return Err(ControlApiError::unauthorized(format!(
                "the auto-wake topic gate refuses: `{topic}` is not among the role's declared interests"
            )));
        }
    }
    // a'. And the ISSUER's topic bound, when the admitting grant declares one
    //     (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.2`): both apply, because they are
    //     two declarations by two parties.
    if let Some(allowed) = auto_bounds
        .as_ref()
        .and_then(|bounds| bounds.topics.as_ref())
    {
        for topic in &req.topics {
            if !allowed.contains(topic) {
                return Err(ControlApiError::unauthorized(format!(
                    "the admitting grant's topic bound refuses: `{topic}` is not among the topics it allows"
                )));
            }
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
    // c. The availability gates — the concurrency drain switch, the wake
    //    policy, the operating hours — by the ONE evaluator the delivery
    //    boundary uses (`crate::wake`, `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`): a
    //    role that may not be woken may not wake itself either.
    if let Some(hold) = crate::wake::hold(profile.availability.as_ref(), now) {
        return Err(ControlApiError::unauthorized(format!(
            "the auto-wake is held: {hold}"
        )));
    }
    // d. The spend bound: the ADMITTING grant's spend limit covers the declared
    //    budget. ⛔ It used to be `max(spend_limits)` over every `active` grant
    //    of the subject with no validity window, so an expired grant with a
    //    larger limit raised the ceiling a live smaller one set (`.5.2` clause 2).
    if let Some(budget) = req.budget_amount {
        let limit = spend_limits
            .as_ref()
            .and_then(|limits| limits.get("amount"))
            .and_then(Value::as_f64);
        match limit {
            Some(limit) if limit >= budget => {}
            _ => {
                return Err(ControlApiError::unauthorized(format!(
                    "the declared budget {budget} exceeds the admitting grant's spend bound"
                )));
            }
        }
    }

    // 3. The initiation rides the SAME create flow (the auto grant authorizes
    //    it — the command machinery does the rest). The idempotency key is
    //    the caller's, namespaced to the role (`SIGNOFF-REPAIR.5.2` clause 1):
    //    a redelivery replays, a new initiation lands. ⛔ It used to be
    //    `auto_{role}_{tenant}`, so a role could initiate once per tenant,
    //    ever — and that replay was the only bound on autonomous initiation
    //    until the `initiator` quota `run_thread_command` now checks.
    // §10.7's depth and cycle controls (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.1`),
    // decided from STORED facts before anything is written — and before the
    // create flow's idempotency path, so a refused chain is never answered
    // with a replay of an earlier thread.
    let parent = match &req.caused_by {
        None => None,
        Some(raw) => {
            let parent_id: ThreadId = raw
                .parse()
                .map_err(|_| ControlApiError::invalid_command("caused_by is not a thread id"))?;
            let Some(projection) =
                load_thread_projection(&state.pool, tenant_id, parent_id).await?
            else {
                return Err(ControlApiError::not_found(format!(
                    "no thread `{parent_id}` to name as the cause"
                )));
            };
            Some((parent_id, projection))
        }
    };
    // The depth ceiling in force: the admitting grant's `max_depth` when it
    // declares one, else the site constant; issuance refuses a grant above the
    // constant, and the `min` keeps that true even for a hand-inserted row.
    let ceiling = auto_bounds
        .as_ref()
        .and_then(|bounds| bounds.max_depth)
        .map_or(threads::MAX_AUTONOMOUS_DEPTH, |declared| {
            declared.min(threads::MAX_AUTONOMOUS_DEPTH)
        });
    let mut lineage = match threads::auto_lineage(
        &role.to_string(),
        parent.as_ref().map(|(id, projection)| (id, projection)),
        ceiling,
    ) {
        Ok(lineage) => lineage,
        Err(refusal @ threads::LineageRefusal::NotAParticipant) => {
            return Err(ControlApiError::unauthorized(refusal.to_string()));
        }
        Err(refusal) => {
            // §10.7's cycle and depth controls are storm controls, and each
            // refusal is recorded before it is answered (`.5.3.4`).
            let (control, limit_value) = match &refusal {
                threads::LineageRefusal::Cycle { .. } => ("autonomous_cycle", None),
                threads::LineageRefusal::TooDeep { ceiling, .. } => {
                    ("autonomous_depth", Some(i64::from(*ceiling)))
                }
                threads::LineageRefusal::NotAParticipant => unreachable!("matched above"),
            };
            return Err(refuse_storm(
                &state.pool,
                StormRefusal {
                    tenant_id: &tenant_id.to_string(),
                    initiator: &principal.id_string(),
                    control,
                    limit_value,
                    target: req.caused_by.as_deref(),
                },
                refusal.to_string(),
            )
            .await);
        }
    };

    let body_value = serde_json::json!({
        "tenant_id": tenant_id.to_string(),
        "subject": req.subject,
        "objective": req.objective,
    });
    // The thread remembers the grant that admitted it
    // (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.3`), so a call opened on it later can
    // read that grant's audience bound.
    lineage.initiating_grant = Some(admitting_grant);
    let mut body: threads::CreateBody = serde_json::from_value(body_value.clone())
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    body.lineage = Some(lineage);
    // `None` twice: no delegation, and no thread to bind — a creation's target is
    // the tenant, which the idempotency primary key already carries. The cause is
    // part of what was asked, so it is hashed too: the same key with a different
    // cause is a mismatch, never a replay.
    let hashed = serde_json::json!({ "body": body_value, "caused_by": req.caused_by });
    let hash = request_hash(threads::OP_CREATE, &principal, &hashed, None, None);
    // The autonomous request carries no profile and no routing class — its
    // body is built above from the tenant, subject and objective alone — so the
    // thread takes the default profile, as it always has. `SIGNOFF-REPAIR.8.2.7`
    // removed the create boundary's routing branch from here: it could never
    // run, and it wrote its audit row before the command's authorization.
    let resolved = crate::workflows::resolve(&state.pool, None)
        .await
        .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
    let workflow_steps = resolved.steps;
    body.workflow_profile = Some(resolved.profile_id);
    let key = format!("auto_{}_{}", role, req.idempotency_key);
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
            resolution: None,
        },
    )
    .await?;
    Ok(json_response(response.0, response.1))
}

/// Whether an audience bound admits a call's eligibility scope: `network`
/// admits any scope; `tenant` admits every scope narrower than the network.
fn audience_admits(
    audience: reasonbraid_core::Audience,
    scope: crate::profiles::ReaderClass,
) -> bool {
    match audience {
        reasonbraid_core::Audience::Network => true,
        reasonbraid_core::Audience::Tenant => scope != crate::profiles::ReaderClass::Network,
    }
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
    let thread_id: ThreadId = req
        .thread_id
        .parse()
        .map_err(|_| ControlApiError::invalid_command("thread_id is malformed"))?;
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::ThreadInvite,
        target: ResourceTarget::Thread {
            tenant_id,
            thread_id,
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
    // The call is bound to a REAL thread in the named tenant
    // (`SIGNOFF-REPAIR.5.2`'s attached clause): the same tenant-bound read every
    // thread view makes, and the same answer for a thread the caller cannot see.
    // ⛔ Until this check a call could be opened on any id at all — the two
    // storm-control fixtures went green on literal ids no thread ever had.
    let Some(projection) = load_thread_projection(&state.pool, tenant_id, thread_id).await? else {
        return Err(ControlApiError::scope_hidden());
    };
    // The audience bound (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.3`): a call on a
    // thread a role initiated on its own may not target a wider audience than
    // the initiating grant allows. Read from the grant the thread names.
    if let Some(grant_id) = &projection.initiating_grant {
        let audience: Option<Value> = sqlx::query_scalar(
            "SELECT auto_bounds->'audience' FROM authority_grants WHERE grant_id = $1",
        )
        .bind(grant_id)
        .fetch_optional(&state.pool)
        .await?
        .flatten();
        if let Some(audience) = audience {
            let audience: reasonbraid_core::Audience =
                serde_json::from_value(audience).map_err(|e| {
                    ControlApiError::internal_with_log(format!("stored audience unreadable: {e}"))
                })?;
            if !audience_admits(audience, req.expression.scope) {
                return Err(ControlApiError::unauthorized(format!(
                    "the initiating grant's audience bound refuses: the call's scope `{}` is wider than `{}`",
                    serde_json::to_value(req.expression.scope)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default(),
                    serde_json::to_value(audience)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default(),
                )));
            }
        }
    }
    if req.min_participants < 1 || req.max_participants < req.min_participants {
        return Err(ControlApiError::invalid_command(
            "min_participants must be ≥ 1 and max_participants ≥ min_participants",
        ));
    }
    // The dev-scale storm controls (`.4.3`): the per-tenant + per-initiator
    // open-call fan-out caps — the typed 429 names the limit.
    let initiator = actor_handle_for_subject(&principal).to_string();
    // The record names the PRINCIPAL (`hpr_…`/`rol_…`), which an operator can
    // resolve; the `agt_` handle above is the fan-out counter's key.
    let refused_by = principal.id_string();
    let expression = serde_json::to_value(&req.expression)
        .map_err(|e| ControlApiError::invalid_command(format!("expression: {e}")))?;
    // ONE transaction (`SIGNOFF-REPAIR.5.2.4`): the tenant's opens serialized,
    // the two caps decided under that serialization, the call and its offers
    // written together. A refusal rolls back before it is recorded, so the
    // record is written on its own commit and the lock is not held for it.
    let mut tx = state.pool.begin().await?;
    crate::recruitment::serialize_opens(&mut tx, &req.tenant_id).await?;
    let tenant_open =
        crate::recruitment::open_calls_by(&mut tx, Some(&req.tenant_id), None).await?;
    if tenant_open >= crate::recruitment::MAX_OPEN_CALLS_PER_TENANT {
        tx.rollback().await?;
        return Err(refuse_storm(
            &state.pool,
            StormRefusal {
                tenant_id: &req.tenant_id,
                initiator: &refused_by,
                control: "open_calls_per_tenant",
                limit_value: Some(crate::recruitment::MAX_OPEN_CALLS_PER_TENANT),
                target: Some(&req.thread_id),
            },
            format!(
                "the tenant's open-call fan-out limit ({}) is reached",
                crate::recruitment::MAX_OPEN_CALLS_PER_TENANT
            ),
        )
        .await);
    }
    let initiator_open = crate::recruitment::open_calls_by(&mut tx, None, Some(&initiator)).await?;
    if initiator_open >= crate::recruitment::MAX_OPEN_CALLS_PER_INITIATOR {
        tx.rollback().await?;
        return Err(refuse_storm(
            &state.pool,
            StormRefusal {
                tenant_id: &req.tenant_id,
                initiator: &refused_by,
                control: "open_calls_per_initiator",
                limit_value: Some(crate::recruitment::MAX_OPEN_CALLS_PER_INITIATOR),
                target: Some(&req.thread_id),
            },
            format!(
                "the initiator's open-call fan-out limit ({}) is reached",
                crate::recruitment::MAX_OPEN_CALLS_PER_INITIATOR
            ),
        )
        .await);
    }
    let call_id = crate::recruitment::open_call(
        &mut tx,
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
    // records the offer (the §10.5 advertisement window's durable trace),
    // on the same transaction as the call.
    let offered_count = crate::recruitment::offer_to_subscribers(
        &mut tx,
        &call_id,
        &req.tenant_id,
        &req.expression.interests,
        req.expression.scope == crate::profiles::ReaderClass::Network,
    )
    .await?;
    tx.commit().await?;

    Ok(Json(json!({
        "call_id": call_id,
        "thread_id": req.thread_id,
        "status": "open",
        "offered_to": offered_count,
    })))
}

/// The respondent's current facts for the eligibility gate (the same shape
/// the match surface loads).
///
/// The PROFILE is the role's own; the PRESENCE is the node its work runs on,
/// resolved through `role_execution` (`SIGNOFF-REPAIR.5.3.5.3.1`): the role's
/// own id under the dev rule, the origin node for an origin-bound import while
/// the recruitment agreement stands, and no node otherwise — `None` here, which
/// every caller refuses as *no enrolled node*.
async fn respondent_candidate<'e>(
    executor: impl sqlx::PgExecutor<'e>,
    role_id: &str,
    at: chrono::DateTime<Utc>,
) -> Option<(
    crate::matching::EligibilityCandidate,
    crate::presence::PresenceState,
)> {
    // online, suspended, declared concurrency, profile, in flight.
    type CandidateRow = (bool, bool, Option<i64>, Option<Value>, i64);
    let row: Option<CandidateRow> = sqlx::query_as(
        "SELECT np.online, np.suspended, \
                (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = re.role_id AND v.version = p.current_version) AS concurrency, \
                (SELECT v.profile \
                 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                 WHERE v.role_id = re.role_id AND v.version = p.current_version) AS profile, \
                np.in_flight \
         FROM role_execution re JOIN node_presence np ON np.node_id = re.node_id \
         WHERE re.role_id = $1",
    )
    .bind(role_id)
    .fetch_optional(executor)
    .await
    .ok()?;
    let (online, suspended, concurrency, profile, in_flight) = row?;
    let hold =
        crate::presence::hold_from_stored(profile.as_ref().and_then(|p| p.get("availability")), at);
    let state = crate::presence::presence_state(
        true,
        suspended,
        online,
        concurrency,
        in_flight,
        hold.as_ref(),
    );
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
    // ONE transaction (`SIGNOFF-REPAIR.5.2.4`): the call row is held SHARED
    // from here to the upsert, so a close in progress — which holds it
    // exclusively — makes this respond wait and then read the status the
    // close committed. A join that lost the race to a close is refused as a
    // response to a closed call, never recorded on a call whose panel it is
    // not on. Responses do not queue behind each other: shared locks coexist.
    let mut tx = pool.begin().await?;
    let Some(call) =
        crate::recruitment::call_locked(&mut tx, call_id, crate::recruitment::CallLock::Shared)
            .await?
    else {
        return Err(ControlApiError::not_found(format!("no call `{call_id}`")));
    };
    // The tenant binding (`SIGNOFF-REPAIR.6.1.2`), derived from the CALL —
    // `inspect_call`'s shape, and the reason it lives here rather than at the
    // MCP seam: the seam gates the caller against a tenant the CALLER
    // SUPPLIES and then hands this core a call id, while the HTTP verb takes
    // no tenant at all. Two surfaces, one core, and the core is where the
    // call's own tenant is in scope.
    //
    // ⛔ Measured before the repair, on BOTH surfaces: a role enrolled in one
    // tenant recorded a `decline` and a `recuse` against another tenant's
    // call. Those two kinds set `participation = false`, so the eligibility
    // gate below is skipped entirely and nothing else in the path ever looks
    // at the respondent.
    let Some(respondent_tenant) = reader_tenant(pool, principal).await? else {
        return Err(ControlApiError::unauthorized(
            "only a principal enrolled in the call's tenant responds to it",
        ));
    };
    let foreign = respondent_tenant != call.tenant_id;
    if foreign {
        // A federated subscriber (`SIGNOFF-REPAIR.5.3.5.2`): its `join` is a
        // REQUEST, never a join — it cannot act in this tenant without a local
        // grant (ADR-026), and the only path to one is the card import the
        // call's tenant performs. Three gates, and the first two answer with
        // the same words an un-offered foreign principal has always heard, so
        // a call's existence is not learned by probing its id:
        //   (1) only a `join` is a request — a foreign decline, observe or
        //       recommend has no local meaning and records nothing;
        //   (2) the call must have been OFFERED to this role, which happens
        //       only for a network-scope call under the directory agreement;
        //   (3) the request carries the role's card, and a card crosses only
        //       under the EFFECTIVE recruitment agreement — the operators'
        //       consent, on both sides, that ADR-026 asks for.
        let old_answer = "only a principal enrolled in the call's tenant responds to it";
        if !matches!(response, crate::recruitment::RecruitmentResponse::Join) {
            return Err(ControlApiError::unauthorized(old_answer));
        }
        let offered: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM recruitment_offers WHERE call_id = $1 AND role_id = $2)",
        )
        .bind(call_id)
        .bind(&respondent)
        .fetch_one(&mut *tx)
        .await?;
        if !offered {
            return Err(ControlApiError::unauthorized(old_answer));
        }
        if !crate::federation::has_effective_recruitment_agreement_in_tx(
            &mut tx,
            &call.tenant_id,
            &respondent_tenant,
        )
        .await?
        {
            return Err(ControlApiError::unauthorized(format!(
                "no effective recruitment agreement between the call's tenant and `{respondent_tenant}` — \
                 a join request carries the role's card, and a card crosses only under one"
            )));
        }
    }
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
        // ONE instant judges the respondent: its hold and its claims' expiry.
        let at = Utc::now();
        let Some((candidate, _state)) = respondent_candidate(&mut *tx, &respondent, at).await
        else {
            return Err(ControlApiError::unauthorized(
                "the respondent has no enrolled node/profile",
            ));
        };
        let verdict = crate::matching::eligible(&expression, &candidate, at);
        if !verdict.eligible {
            return Err(ControlApiError::unauthorized(format!(
                "the respondent is ineligible: {}",
                verdict.reasons.join("; ")
            )));
        }
    }
    if foreign {
        // The request IS the role's own export, attached for the call's tenant
        // to import: the same card `GET /v1/profiles/{role_id}/card` would
        // mint for it, with its digest, so the import's digest rung re-derives
        // it exactly. Recorded under `join_request`, which the close never
        // seats and the inspection shows.
        let Some((card, digest)) = mint_card(pool, &respondent).await? else {
            return Err(ControlApiError::unauthorized(
                "the respondent has no profile to export with its request",
            ));
        };
        crate::recruitment::record_join_request(
            &mut tx,
            call_id,
            &respondent,
            &json!({
                "origin_tenant_id": respondent_tenant,
                "card": card,
                "digest": digest,
            }),
        )
        .await?;
        tx.commit().await?;
        return Ok(json!({
            "call_id": call_id,
            "respondent": respondent,
            "response": "join_request",
        }));
    }
    crate::recruitment::record_response(&mut tx, call_id, &respondent, response).await?;
    tx.commit().await?;
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
    // ONE transaction (`SIGNOFF-REPAIR.5.2.4`): the call row is held
    // EXCLUSIVELY from here to the commit. A second close waits here, reads
    // `closed`, and refuses below — a `409`, never `recruitment_panels`'
    // primary key raised into a `500`. A respond waits here too, so no join
    // lands between the responses read and the snapshot. Until this
    // transaction, the close was a read on the pool, then two writes on the
    // pool, with both races open.
    let mut tx = state.pool.begin().await?;
    let Some(call) =
        crate::recruitment::call_locked(&mut tx, &call_id, crate::recruitment::CallLock::Exclusive)
            .await?
    else {
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
    let responses = crate::recruitment::responses(&mut *tx, &call_id).await?;
    let joiners: Vec<String> = responses
        .iter()
        .filter(|(_, kind, _, _)| kind == "join")
        .map(|(respondent, _, _, _)| respondent.clone())
        .collect();
    // The minimum is asked below, of the SELECTED panel — not here, of who
    // once said `join` (`SIGNOFF-REPAIR.5.2.5`).
    // Rank the joiners (the default preferences) and cap at the max.
    let mut candidates: Vec<(
        crate::matching::EligibilityCandidate,
        crate::matching::EligibilityVerdict,
    )> = Vec::new();
    // ONE instant judges the whole panel — every joiner's hold and every
    // claim's expiry — so two joiners are never judged at different times.
    let at = Utc::now();
    for joiner in &joiners {
        if let Some((candidate, _)) = respondent_candidate(&mut *tx, joiner, at).await {
            let verdict = crate::matching::eligible(&expression, &candidate, at);
            candidates.push((candidate, verdict));
        }
    }
    // The dependence facts (`.6.2`): the LATEST incarnation on the node each
    // joiner's work runs on (`role_execution`, `SIGNOFF-REPAIR.5.3.5.3.1` — an
    // origin-bound import's facts are the origin machine's) + the tenant as the
    // owner — the panel's indicator inputs. `incarnations.node_id` is the §8.1
    // binding fact (DOC-0139); under the dev rule it equals the role id.
    let mut facts: std::collections::HashMap<String, crate::dependence::MemberFacts> =
        std::collections::HashMap::new();
    for joiner in &joiners {
        let lineage: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT provider, model, harness FROM incarnations \
                 WHERE node_id = (SELECT node_id FROM role_execution WHERE role_id = $1) \
                 ORDER BY valid_from DESC LIMIT 1",
        )
        .bind(joiner)
        .fetch_optional(&mut *tx)
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
    // §10.5's minimum is on the SELECTED panel (`SIGNOFF-REPAIR.5.2.5`): it is
    // asked of the ELIGIBLE, ranked set — after the re-resolution above has
    // dropped a joiner whose profile or node no longer satisfies the
    // expression — and never of who once said `join`. Until this check moved
    // here, a call closed with a panel below its minimum, down to empty.
    if (ranked.len() as i32) < call.min_participants {
        return Err(ControlApiError::invalid_transition(format!(
            "the panel needs at least {} eligible joiners: {} remain eligible of {} who joined",
            call.min_participants,
            ranked.len(),
            joiners.len()
        )));
    }
    ranked.truncate(call.max_participants as usize);
    let indicators: Vec<crate::dependence::DependenceIndicator> =
        crate::dependence::dependence_indicators(
            &ranked
                .iter()
                .filter_map(|r| facts.get(&r.role_id).cloned())
                .collect::<Vec<_>>(),
        );
    crate::recruitment::snapshot_panel(&mut tx, &call_id, &ranked, &indicators).await?;
    tx.commit().await?;
    Ok(Json(json!({
        "call_id": call_id,
        "status": "closed",
        "panel": ranked
            .iter()
            .map(|r| r.role_id.clone())
            .collect::<Vec<_>>(),
    })))
}

/// `GET /v1/calls/offered` — the calls offered to the calling ROLE
/// (`SIGNOFF-REPAIR.5.3.5.1`): the durable half of §10.5's advertisement. An
/// offer row was written for every matching subscriber since `.5.2`, and until
/// this read nothing carried it further — a role learned of a call out of band
/// and answered by id. Only OPEN calls inside their join window are listed,
/// oldest offer first, each with the role's own response if it has made one.
/// A person has no offers (calls are offered to roles), and an unenrolled
/// principal is refused rather than shown an empty list it could mistake for
/// an answer.
async fn list_offered_calls(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let GrantSubject::Role(role) = &principal else {
        return Err(ControlApiError::unauthorized(
            "calls are offered to roles; a person reads a call by its id",
        ));
    };
    let Some(own_tenant) = reader_tenant(&state.pool, &principal).await? else {
        return Err(ControlApiError::unauthorized(
            "an unenrolled principal has no offers",
        ));
    };
    let role_id = role.to_string();
    type Row = (
        String,
        String,
        String,
        Value,
        i32,
        i32,
        bool,
        DateTime<Utc>,
        DateTime<Utc>,
        DateTime<Utc>,
        Option<String>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT c.call_id, c.tenant_id, c.thread_id, c.expression, c.min_participants, \
                c.max_participants, c.recommendations_allowed, c.join_deadline, c.expires_at, \
                o.offered_at, \
                (SELECT r.response_kind FROM recruitment_responses r \
                  WHERE r.call_id = c.call_id AND r.respondent = $1) AS responded \
         FROM recruitment_offers o JOIN recruitment_calls c ON c.call_id = o.call_id \
         WHERE o.role_id = $1 AND c.status = 'open' \
           AND c.join_deadline > now() AND c.expires_at > now() \
         ORDER BY o.offered_at, c.call_id",
    )
    .bind(&role_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({
        "role_id": role_id,
        "offered": rows
            .into_iter()
            .map(
                |(
                    call_id,
                    call_tenant_id,
                    thread_id,
                    expression,
                    min_participants,
                    max_participants,
                    recommendations_allowed,
                    join_deadline,
                    expires_at,
                    offered_at,
                    responded,
                )| {
                    // A federated offer (`SIGNOFF-REPAIR.5.3.5.1.2`) carries the
                    // call and its window and NOT the thread: the thread lives in
                    // another tenant, whose existence the read must not leak
                    // (ROADMAP §9.4: cross-tenant existence is not leaked) and
                    // which the role could not read anyway. It is named on a join.
                    let foreign = call_tenant_id != own_tenant;
                    json!({
                    "call_id": call_id,
                    "call_tenant_id": call_tenant_id,
                    "foreign": foreign,
                    "thread_id": if foreign { Value::Null } else { json!(thread_id) },
                    "expression": expression,
                    "min_participants": min_participants,
                    "max_participants": max_participants,
                    "recommendations_allowed": recommendations_allowed,
                    "join_deadline": join_deadline.to_rfc3339(),
                    "expires_at": expires_at.to_rfc3339(),
                    "offered_at": offered_at.to_rfc3339(),
                    "responded": responded,
                    })
                },
            )
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

/// The directory's rows, one per listed identity (`SIGNOFF-REPAIR.5.3.5.3.1.2`),
/// as `d (role_id, node_id, tenant_id, online, suspended, last_seen_at,
/// lease_expires_at, concurrency, profile, in_flight)`:
///
/// - every enrolled node, as the role of its own id — the dev rule, and the
///   only rows before this repair (a node with no profile still lists, as the
///   presence always showed it);
/// - every ORIGIN-bound identity, on the node `role_execution` resolves it to,
///   under its OWN tenant (the importing one, which decides the reader's
///   class) and with its OWN profile. An identity whose agreement no longer
///   stands resolves to no node and is not listed.
///
/// The profile and the declared concurrency are always the ROLE's; the
/// presence columns are always the NODE's.
const DIRECTORY_ROWS: &str = "(\
    SELECT np.node_id AS role_id, np.node_id, np.tenant_id, np.online, np.suspended, \
           np.last_seen_at, np.lease_expires_at, np.in_flight \
    FROM node_presence np \
    UNION ALL \
    SELECT re.role_id, np.node_id, re.tenant_id, np.online, np.suspended, \
           np.last_seen_at, np.lease_expires_at, np.in_flight \
    FROM role_execution re JOIN node_presence np ON np.node_id = re.node_id \
    WHERE re.node_id <> re.role_id\
) d";

/// The columns every directory read selects from [`DIRECTORY_ROWS`], in the
/// order both handlers' row types expect.
const DIRECTORY_COLUMNS: &str = "d.role_id, d.node_id, d.tenant_id, d.online, d.suspended, \
    d.last_seen_at, d.lease_expires_at, \
    (SELECT (v.profile->'availability'->>'concurrency')::bigint \
     FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
     WHERE v.role_id = d.role_id AND v.version = p.current_version) AS concurrency, \
    (SELECT v.profile \
     FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
     WHERE v.role_id = d.role_id AND v.version = p.current_version) AS profile, \
    d.in_flight";

/// One directory row, as [`DIRECTORY_COLUMNS`] selects it.
type DirectoryRow = (
    String,
    String,
    String,
    bool,
    bool,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<i64>,
    Option<Value>,
    i64,
);

/// The reader's class toward ANOTHER tenant's directory entries, memoised per
/// tenant for one request: `Tenant` when the effective directory-visibility
/// agreement stands between the two (both directions accepted, both carrying
/// `directory_visibility`, unexpired — the opt-in `classify_reader` honours),
/// `Network` otherwise. The match and the presence listing both ask it
/// (`SIGNOFF-REPAIR.5.1.1`, `.5.1.6`), so the two routes cannot classify one
/// tenant differently.
async fn class_toward_foreign_tenant(
    pool: &PgPool,
    reader_tenant: &str,
    tenant: &str,
    memo: &mut std::collections::HashMap<String, crate::profiles::ReaderClass>,
) -> Result<crate::profiles::ReaderClass, ControlApiError> {
    if let Some(class) = memo.get(tenant) {
        return Ok(*class);
    }
    let class = if crate::federation::has_effective_directory_agreement(pool, reader_tenant, tenant)
        .await?
    {
        crate::profiles::ReaderClass::Tenant
    } else {
        crate::profiles::ReaderClass::Network
    };
    memo.insert(tenant.to_string(), class);
    Ok(class)
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
/// expression's scope must not exceed the reader's classification toward its
/// OWN tenant (a member cannot demand the full view — the typed 403). The
/// response carries ONLY the eligible candidates, ranked, each with the
/// stage-1 reasons + the stage-2 explanations + the profile fields VISIBLE to
/// the reader.
///
/// ⛔ VISIBLE TO THE READER is decided PER CANDIDATE TENANT
/// (`SIGNOFF-REPAIR.5.1.1`), the way `directory_presence` always did and this
/// surface did not: the reader is `Full` or `Tenant` toward its own tenant's
/// candidates, `Tenant` toward a tenant that holds the effective directory
/// agreement with it, and `Network` toward everyone else. Each candidate is
/// judged, ranked and rendered at the LOWER of the expression's scope and that
/// class. Until this repair ONE class — the reader's own — was applied to
/// every tenant's candidate, so a member of one tenant read another tenant's
/// tenant-view fields and saw its tenant-only claims satisfy a requirement.
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
    // The weights are bounded before the directory is read (`SIGNOFF-REPAIR.5.1.4`).
    req.preferences
        .validate()
        .map_err(ControlApiError::invalid_command)?;
    let owner = if let Ok(tenant) = reader_tenant.parse() {
        authorize_tenant_admin(&state.pool, &principal, tenant)
            .await
            .is_ok()
    } else {
        false
    };
    // The reader's class toward its OWN tenant: the owner reads full, a member
    // reads the tenant view. It is the scope clamp's reference and the class
    // every own-tenant candidate is rendered at.
    let own_class = if owner {
        crate::profiles::ReaderClass::Full
    } else {
        crate::profiles::ReaderClass::Tenant
    };
    // The scope clamp: the expression must not exceed the reader's own class.
    if scope_rank(req.expression.scope) > scope_rank(own_class) {
        return Err(ControlApiError::unauthorized(
            "the expression's scope exceeds the reader's classification".to_string(),
        ));
    }

    let rows: Vec<DirectoryRow> = sqlx::query_as(&format!(
        "SELECT {DIRECTORY_COLUMNS} FROM {DIRECTORY_ROWS} ORDER BY d.role_id"
    ))
    .fetch_all(&state.pool)
    .await?;

    // The class toward each CANDIDATE's tenant, memoised per tenant: the
    // reader's own tenant reads at `own_class`; a tenant holding the effective
    // directory agreement with the reader's reads at the tenant view (the
    // opt-in `classify_reader` honours); every other tenant at the network
    // pseudonym. A candidate is judged, ranked and rendered at the LOWER of
    // the expression's scope and that class — so a foreign tenant-only claim
    // neither satisfies a requirement nor appears in the answer.
    let mut class_by_tenant: std::collections::HashMap<String, crate::profiles::ReaderClass> =
        std::collections::HashMap::new();
    // ONE instant judges every candidate — its hold and its claims' expiry.
    let at = Utc::now();
    let mut scope_by_role: std::collections::HashMap<String, crate::profiles::ReaderClass> =
        std::collections::HashMap::new();
    let mut candidates: Vec<(
        crate::matching::EligibilityCandidate,
        crate::matching::EligibilityVerdict,
    )> = Vec::new();
    for (
        role_id,
        _node_id,
        tenant,
        online,
        suspended,
        _last_seen,
        _lease_expiry,
        concurrency,
        profile,
        in_flight,
    ) in rows
    {
        let Some(stored) = profile else {
            continue; // a role node without a profile declares nothing
        };
        let Ok(parsed) = serde_json::from_value::<crate::profiles::AgentProfile>(stored) else {
            continue;
        };
        let class = if tenant == reader_tenant {
            own_class
        } else {
            class_toward_foreign_tenant(&state.pool, &reader_tenant, &tenant, &mut class_by_tenant)
                .await?
        };
        let effective = if scope_rank(class) < scope_rank(req.expression.scope) {
            class
        } else {
            req.expression.scope
        };
        let scoped = crate::matching::EligibilityExpression {
            scope: effective,
            ..req.expression.clone()
        };
        let hold = crate::wake::hold(parsed.availability.as_ref(), at);
        let state_now = crate::presence::presence_state(
            true,
            suspended,
            online,
            concurrency,
            in_flight,
            hold.as_ref(),
        );
        let candidate = crate::matching::EligibilityCandidate {
            role_id: role_id.clone(),
            profile: Some(parsed),
            presence_state: state_now,
            concurrency,
            // The dev profile has no per-role budget facts: UNKNOWN, never
            // zero (§14.5) — a budget requirement therefore cannot be proven.
            available_budget: None,
        };
        let verdict = crate::matching::eligible(&scoped, &candidate, at);
        scope_by_role.insert(role_id, effective);
        candidates.push((candidate, verdict));
    }

    // The ranking scores each candidate on its own visible profile, so ranking
    // each scope's group at that scope and merging by the same key the ranker
    // sorts on is the per-candidate rank; the dependence facts, which are
    // cross-candidate, are the close's and not this surface's.
    let mut ranked: Vec<crate::matching::RankedCandidate> = Vec::new();
    for scope in [
        crate::profiles::ReaderClass::Full,
        crate::profiles::ReaderClass::Tenant,
        crate::profiles::ReaderClass::Network,
    ] {
        let group: Vec<(
            crate::matching::EligibilityCandidate,
            crate::matching::EligibilityVerdict,
        )> = candidates
            .iter()
            .filter(|(c, _)| scope_by_role.get(&c.role_id) == Some(&scope))
            .cloned()
            .collect();
        if group.is_empty() {
            continue;
        }
        let scoped = crate::matching::EligibilityExpression {
            scope,
            ..req.expression.clone()
        };
        ranked.extend(crate::matching::rank(&scoped, &group, &req.preferences));
    }
    ranked.sort_by(crate::matching::by_rank);
    let out: Vec<Value> = ranked
        .into_iter()
        .map(|r| {
            let class = scope_by_role
                .get(&r.role_id)
                .copied()
                .unwrap_or(crate::profiles::ReaderClass::Network);
            let profile = candidates
                .iter()
                .find(|(c, _)| c.role_id == r.role_id)
                .and_then(|(c, _)| c.profile.as_ref())
                .map(|p| crate::profiles::filter_profile(p, class))
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
        "scope": format!("{:?}", own_class).to_lowercase(),
        "candidates": out,
    })))
}

// ── The privacy-filtered directory views (PHASE-3.2.3; backlog 27) ──────────────

/// The reader's directory scope: the OWNER (tenant_admin) reads the FULL fields
/// of their own tenant's nodes; a tenant member reads the TENANT-filtered
/// fields; every enrolled principal reads the other tenants at the class the
/// match reads them at — the tenant view under the effective directory
/// agreement, the network pseudonym otherwise (`SIGNOFF-REPAIR.5.1.6`) — and a
/// profile whose view at that class is empty contributes NOTHING, not even a
/// count. The `.1.3` filter is the field-level engine.
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

    // Every listed identity with its node's derived presence + its CURRENT
    // profile ([`DIRECTORY_ROWS`]: each enrolled node as the role of its own
    // id, and each origin-bound identity on the node it resolves to).
    let rows: Vec<DirectoryRow> = sqlx::query_as(&format!(
        "SELECT {DIRECTORY_COLUMNS} FROM {DIRECTORY_ROWS} ORDER BY d.tenant_id, d.role_id"
    ))
    .fetch_all(&state.pool)
    .await?;

    let mut own_nodes = Vec::new();
    let mut network_nodes = Vec::new();
    // Another tenant's entries are read at the class the match reads them at
    // (`SIGNOFF-REPAIR.5.1.6`): the tenant view under the effective directory
    // agreement, the network pseudonym otherwise. Until this repair every other
    // tenant was read at `Network`, agreement or not.
    let mut class_by_tenant: std::collections::HashMap<String, crate::profiles::ReaderClass> =
        std::collections::HashMap::new();
    // The loop below names each entry's presence `state`; the pool is taken first.
    let pool = &state.pool;
    for (
        role_id,
        node_id,
        tenant,
        online,
        suspended,
        last_seen_at,
        lease_expires_at,
        concurrency,
        profile,
        in_flight,
    ) in rows
    {
        let hold = crate::presence::hold_from_stored(
            profile.as_ref().and_then(|p| p.get("availability")),
            Utc::now(),
        );
        let state = crate::presence::presence_state(
            true,
            suspended,
            online,
            concurrency,
            in_flight,
            hold.as_ref(),
        )
        .as_str()
        .to_string();
        let hold = hold.as_ref().map(|h| h.wire_name());
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
                    "role_id": role_id,
                    "node_id": node_id,
                    "state": state,
                    "hold": hold,
                    "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                    "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
                    "profile": filtered,
                }))
            }
            (true, None) => Some(json!({
                "role_id": role_id,
                "node_id": node_id,
                "state": state,
                "hold": hold,
                "last_seen_at": last_seen_at.map(|t| t.to_rfc3339()),
                "lease_expires_at": lease_expires_at.map(|t| t.to_rfc3339()),
            })),
            (false, Some(stored)) => {
                let parsed: crate::profiles::AgentProfile = match serde_json::from_value(stored) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let class = class_toward_foreign_tenant(
                    pool,
                    &reader_tenant,
                    &tenant,
                    &mut class_by_tenant,
                )
                .await?;
                let filtered = crate::profiles::filter_profile(&parsed, class);
                // The zero-visibility rule: a profile whose view at that class
                // is EMPTY contributes nothing — not even a count.
                if filtered.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    continue;
                }
                // ⛔ An origin-bound identity's NODE is the origin tenant's:
                // naming it to a third tenant would disclose that the two
                // tenants federate. Only the identity's own tenant reads which
                // node it runs on (`SIGNOFF-REPAIR.5.3.5.3.1.2`).
                let mut entry = json!({
                    "role_id": role_id,
                    "state": state,
                    "hold": hold,
                    "profile": filtered,
                });
                if node_id == role_id {
                    entry["node_id"] = json!(node_id);
                }
                Some(entry)
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
    // The availability formats (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`): a value the
    // wake evaluator could not read is refused here, so it never reaches the
    // store — where it would hold the role fail-closed.
    if let Some(availability) = &profile.availability {
        crate::wake::validate(availability)
            .map_err(|error| ControlApiError::invalid_command(error.to_string()))?;
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
        storage_failure(e, "profile write failed")
    }
}

/// The reader's tenant (their identity row), when enrolled.
/// The evidence citer (`SIGNOFF-REPAIR.11.14.1`): the tenant whose reads the
/// acquisition will appear on, plus the actor handle that asked for it — the
/// same handle `submit_resource` records as a reference's `submitted_by`.
fn citer(principal: &GrantSubject, tenant: String) -> crate::snapshots::Citer {
    crate::snapshots::Citer {
        tenant_id: tenant,
        principal: actor_handle_for_subject(principal).to_string(),
    }
}

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
    let full = class == crate::profiles::ReaderClass::Full;
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
    let mut body = json!({
        "role_id": role_id,
        "version": current.version,
        "content_hash": current.content_hash,
        "visibility": visibility,
        "written_by": current.written_by,
        "written_at": current.written_at.to_rfc3339(),
        "profile": profile,
    });
    // The provenance of an imported role (`SIGNOFF-REPAIR.5.3.2`), on the full
    // class only: the origin is the importing tenant's own record of where its
    // role came from, not a directory fact for siblings or the network.
    if full {
        // `executes_on` is the binding as recorded; `runs_on` is where it
        // resolves NOW (`role_execution`, `SIGNOFF-REPAIR.5.3.5.3.1`) — the two
        // differ exactly when an origin binding's agreement no longer stands.
        type Provenance = (
            String,
            String,
            String,
            DateTime<Utc>,
            Option<String>,
            Option<String>,
        );
        let provenance: Option<Provenance> = sqlx::query_as(
            "SELECT ci.origin_tenant_id, ci.origin_role_id, ci.card_digest, ci.imported_at, \
                    ci.executes_on, re.node_id \
             FROM card_imports ci JOIN role_execution re ON re.role_id = ci.role_id \
             WHERE ci.role_id = $1",
        )
        .bind(&role_id)
        .fetch_optional(&state.pool)
        .await?;
        if let Some((
            origin_tenant_id,
            origin_role_id,
            card_digest,
            imported_at,
            executes_on,
            runs_on,
        )) = provenance
        {
            body["imported_from"] = json!({
                "origin_tenant_id": origin_tenant_id,
                "origin_role_id": origin_role_id,
                "card_digest": card_digest,
                "imported_at": imported_at.to_rfc3339(),
                "execution": if executes_on.is_some() { "origin" } else { "local" },
                "executes_on": executes_on,
                "runs_on": runs_on,
            });
        }
    }
    Ok(Json(body))
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
    let Some((card, digest)) = mint_card(&state.pool, &role_id).await? else {
        return Err(ControlApiError::not_found(format!(
            "no profile for `{role_id}`"
        )));
    };
    Ok(Json(json!({ "card": card, "digest": digest })))
}

/// The card a role exports, minted from its current profile: the export and
/// the federated join request (`SIGNOFF-REPAIR.5.3.5.2`) mint the same card,
/// because the request IS the role's own export, attached for the call's
/// tenant. `None` when the role has no profile to export.
async fn mint_card(
    pool: &PgPool,
    role_id: &str,
) -> Result<Option<(crate::cards::AgentCard, String)>, ControlApiError> {
    let Some(current) = crate::profiles::current_profile(pool, role_id).await? else {
        return Ok(None);
    };
    let profile: crate::profiles::AgentProfile =
        serde_json::from_value(current.profile).map_err(|e| {
            ControlApiError::internal_with_log(format!("stored profile no longer parses: {e}"))
        })?;
    let Some(origin_tenant) = role_tenant(pool, role_id).await? else {
        return Ok(None);
    };
    let card = crate::cards::AgentCard {
        schema_version: crate::cards::CARD_SCHEMA_VERSION.to_string(),
        origin_tenant_id: origin_tenant,
        origin_role_id: role_id.to_string(),
        profile,
        exported_at: Utc::now().to_rfc3339(),
    };
    let digest = crate::cards::digest_of(&card)
        .map_err(|e| ControlApiError::internal_with_log(format!("the card digests: {e}")))?;
    Ok(Some((card, digest)))
}

/// The card import body: the importing tenant + the card + the digest the
/// card claims (the re-derivation rung).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportCardRequest {
    pub tenant_id: TenantId,
    pub card: crate::cards::AgentCard,
    pub digest: String,
    /// Where the imported identity's work runs; `local` when absent
    /// (`SIGNOFF-REPAIR.5.3.5.3.1`).
    #[serde(default)]
    pub execution: crate::cards::CardExecution,
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
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    // The card's own digest, re-derived before the transaction because it is the
    // effect record's TARGET and must exist even when a rung refuses. It is pure
    // — no storage is touched and nothing is refused by computing it — so running
    // it before the admission leaks nothing.
    let card_digest = crate::cards::digest_of(&req.card).map_err(|e| {
        ControlApiError::internal_with_log(format!("the submitted card does not digest: {e}"))
    })?;
    let import = authority::import_card_in_one_transaction(
        &state.pool,
        &principal,
        req.tenant_id,
        &req.card,
        &req.digest,
        &card_digest,
        req.execution,
    )
    .await?;
    let receipt = import.record_id;
    let response = match import.result {
        authority::CardImportResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // The rung messages are `CardError`'s own, unchanged.
        authority::CardImportResult::CardRefused(detail) => {
            ControlApiError::invalid_command(detail).into_response()
        }
        authority::CardImportResult::NoAgreement { origin_tenant } => {
            ControlApiError::unauthorized(format!(
                "no effective federation agreement with the origin tenant `{origin_tenant}` — the \
                 import refuses"
            ))
            .into_response()
        }
        authority::CardImportResult::NoActiveBoundary => ControlApiError::invalid_command(
            "the tenant has no active enrollment boundary — the import refuses",
        )
        .into_response(),
        authority::CardImportResult::GrantRefused(detail) => {
            ControlApiError::invalid_command(detail).into_response()
        }
        // The label collision used to be a raised unique violation and a `500`
        // (`SIGNOFF-REPAIR.3.3.4.11.5`). It is a typed refusal, and since
        // `SIGNOFF-REPAIR.5.3.2` it is only ever a DIFFERENT origin's card: a
        // repeat of the same origin is the replay below.
        authority::CardImportResult::LabelTaken { label } => {
            ControlApiError::invalid_command(format!(
                "the importing tenant already holds an identity labelled `{label}` — the import \
                 refuses"
            ))
            .into_response()
        }
        authority::CardImportResult::NoOriginNode { origin_role } => {
            ControlApiError::invalid_command(format!(
                "the origin role `{origin_role}` has no enrolled node in this deployment — the \
                 `origin` binding refuses"
            ))
            .into_response()
        }
        authority::CardImportResult::Imported {
            role_id,
            executes_on,
        } => Json(json!({
            "role_id": role_id,
            "origin_tenant_id": req.card.origin_tenant_id,
            "origin_role_id": req.card.origin_role_id,
            "execution": req.execution,
            "executes_on": executes_on,
        }))
        .into_response(),
        // The enrolment route's answer to a repeat (`SIGNOFF-REPAIR.5.3.2`): the
        // original local role, flagged, with the digest of the card on file so a
        // caller holding a newer card can see it did not land.
        authority::CardImportResult::Replayed {
            role_id,
            digest_on_file,
        } => Json(json!({
            "role_id": role_id,
            "origin_tenant_id": req.card.origin_tenant_id,
            "origin_role_id": req.card.origin_role_id,
            "replayed": true,
            "digest_on_file": digest_on_file,
        }))
        .into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
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

/// `POST /v1/profiles/{role_id}/attest` — the owner upgrades one capability
/// claim's provenance, in ONE guarded transaction (`SIGNOFF-REPAIR.3.3.4.11.2`).
///
/// The superseded shape ran three unconnected pieces: the role's tenant on the
/// pool, an admission in its own transaction, then a read of the current profile
/// on the pool and a write in a third transaction. The read-modify-write split
/// across two transactions is what made a concurrent attestation vanish — two
/// administrators upgrading two different capabilities of one role each read
/// version N and each wrote a profile carrying only their own upgrade, with no
/// error on either side.
///
/// The role's tenant is still resolved before the transaction, because the guard
/// set must be declared before the transaction opens; it is re-read inside, under
/// the guard, and the answer used there is the one the mutation acts on.
///
/// Every answer that reached an admission carries the
/// `x-reasonbraid-authorization` receipt naming it, which is also the effect
/// record's id. The pre-admission `404` deliberately carries none: no record
/// exists yet to name.
async fn attest_capability_claim(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(role_id): Path<String>,
    Json(req): Json<AttestRequest>,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let Some(tenant) = role_tenant(&state.pool, &role_id).await? else {
        return Err(ControlApiError::not_found(format!("no role `{role_id}`")));
    };
    let attestation = authority::attest_capability_in_one_transaction(
        &state.pool,
        &principal,
        tenant.parse().map_err(|_| ControlApiError::internal())?,
        &role_id,
        &req.taxonomy_id,
        &req.evidence_ref,
    )
    .await?;
    let receipt = attestation.record_id;
    let response = match attestation.result {
        authority::AttestResult::Denied { reason } => {
            crate::telemetry::metrics().incr("authorization_denials");
            ControlApiError::unauthorized(format!("authorization denied ({receipt}): {reason}"))
                .into_response()
        }
        // Unchanged message: "no profile" and "no such capability" stay ONE
        // answer on the wire, and the effect record is where they stop being the
        // same fact.
        authority::AttestResult::NoSuchClaim => ControlApiError::not_found(format!(
            "no profile for `{role_id}` or no capability `{}` in it",
            req.taxonomy_id
        ))
        .into_response(),
        authority::AttestResult::Attested(written) => Json(*written).into_response(),
    };
    Ok(([("x-reasonbraid-authorization", receipt)], response).into_response())
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
                Option<Value>,
                Option<Value>,
                Option<Value>,
                Option<Value>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT grant_id, subject_kind, subject_id, actions, status, valid_from, expires_at, \
                spend_limits, auto_bounds, decision_rule_constraints, conditions \
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
                        spend_limits,
                        auto_bounds,
                        decision_rule_constraints,
                        conditions,
                    )| {
                        let mut grant = json!({
                            "grant_id": grant_id,
                            "subject_kind": subject_kind,
                            "subject_id": subject_id,
                            "actions": actions,
                            "status": status,
                            "valid_from": valid_from.to_rfc3339(),
                            "expires_at": expires_at.to_rfc3339(),
                        });
                        // The bounds an operator may read
                        // (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1`); absent
                        // optional facts are OMITTED on the wire, never `null`.
                        if let Some(spend_limits) = spend_limits {
                            grant["spend_limits"] = spend_limits;
                        }
                        if let Some(auto_bounds) = auto_bounds {
                            grant["auto_bounds"] = auto_bounds;
                        }
                        if let Some(rules) = decision_rule_constraints {
                            grant["decision_rule_constraints"] = rules;
                        }
                        if let Some(conditions) = conditions {
                            grant["conditions"] = conditions;
                        }
                        grant
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
                Option<String>,
            );
            let rows: Vec<Row> = sqlx::query_as(
        "SELECT incarnation_id, role_id, provider, model, harness, config, valid_from, valid_to, \
                node_id \
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
                        node_id,
                    )| {
                        json!({
                            "incarnation_id": incarnation_id,
                            "role_id": role_id,
                            // §8.1: which node instance ran it
                            // (`SIGNOFF-REPAIR.11.4.7.2.1.5.1`).
                            "node_id": node_id,
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
        authority::BreakerResult::ThresholdNamesNothing => ControlApiError::invalid_command(
            "the threshold names no dimension, so the breaker could never trip; \
             name at least one of calls, input_tokens, output_tokens, wall_clock_seconds",
        )
        .into_response(),
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
/// A routing resolution made at the create boundary (`SIGNOFF-REPAIR.8.2.7`).
///
/// ⛔ It used to be written to the POOL before the command's authorization, so
/// a create that was then refused left an audit row asserting a resolution for
/// a thread that never existed, and each idempotent replay of a successful
/// create added another. It now rides the command and is recorded in its
/// transaction after authorization, so it commits exactly when the thread does.
pub(crate) struct CreateResolution {
    pub route: crate::routing::ResolvedRoute,
    /// DERIVED from the authenticated caller, never from `body.tenant_id`
    /// (`SIGNOFF-REPAIR.7.1.2.2`); unchanged by the move into the transaction.
    pub journal_tenant: String,
}

pub(crate) enum CommandTarget<'a> {
    /// `thread.create` — pure preparation; the thread id is server-assigned.
    Create {
        body: &'a CreateBody,
        /// The resolved profile steps (the ADR-016 composition) — the
        /// create boundary resolved them against the registry.
        workflow_steps: Vec<String>,
        /// The routing resolution that chose the profile, when a routing class
        /// did; recorded inside the command's transaction.
        resolution: Option<CreateResolution>,
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
/// Record a full-backlog dispatch refusal and return the `429` that answers it
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`). Called by `run_thread_command` AFTER it
/// has rolled its transaction back — no event, no invitation, no work, so the
/// `.6.2` invariant holds — on the pool's own connection, so the record
/// survives the rollback and the operator can see which node is drowning. It
/// goes through the one helper every storm control uses. A retry re-validates,
/// as every domain refusal's does.
async fn backlog_refused(
    pool: &PgPool,
    tenant_id: &TenantId,
    principal: &GrantSubject,
    thread_id: &ThreadId,
    node: &str,
    undelivered: i64,
    cap: i64,
) -> ControlApiError {
    let message = format!(
        "the offline backlog of node `{node}` is at its cap ({undelivered} of {cap} \
         undelivered rows) — no more work is handed to it until it takes some"
    );
    refuse_storm(
        pool,
        StormRefusal {
            tenant_id: &tenant_id.to_string(),
            initiator: &principal.id_string(),
            control: "offline_backlog",
            limit_value: Some(cap),
            target: Some(&thread_id.to_string()),
        },
        message,
    )
    .await
}

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
            resolution,
        } => {
            // The INITIATOR quota (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1`, ROADMAP
            // §11.5's rate bound): an AUTONOMOUS creation — one carrying a
            // lineage, which only `POST /v1/threads/auto` can set — counts
            // against the role's windowed ceiling. It rides THIS transaction,
            // after the idempotency claim and the authorization, so a replay
            // consumes nothing, a denied caller consumes nothing, and a `use`
            // commits with the thread it admitted. A refusal follows the
            // recorded-denial pattern the invite quota established: the
            // rejection is stored and committed, never rolled back silently.
            if body.lineage.is_some() {
                if let Err(refusal) = crate::quota::check_in_tx(
                    &mut *tx,
                    &tenant_id.to_string(),
                    crate::quota::SCOPE_INITIATOR,
                    &principal.id_string(),
                    now,
                )
                .await
                {
                    let err = ControlApiError::from(threads::ThreadError::QuotaRefused(refusal));
                    store_rejection(&mut *tx, tenant_id, idempotency_key, &err.failure_result())
                        .await?;
                    tx.commit().await?;
                    return Err(err);
                }
            }
            // `SIGNOFF-REPAIR.8.1.1.2`: may this tenant decide under the
            // declared rule? Asked AFTER authorization, so a caller who may not
            // create here learns nothing about this tenant's charter, and on
            // this transaction, so the thread records the charter it was
            // actually created under. A boundary naming no registered charter
            // FAILS CLOSED (`charters::for_tenant`).
            // And may THIS SUBJECT declare it? §4.2's `decision_rule_constraints`
            // (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.1`): the ADMITTING grant — the one
            // the authorization record names — may narrow the charter for its
            // subject. Read on this transaction from the record the same
            // transaction wrote; a grant with no constraint leaves the charter
            // to decide, exactly as before.
            if let (Some(rule), Some(admitted)) = (body.decision_rule, admission.as_ref()) {
                let constraint: Option<Option<Value>> = sqlx::query_scalar(
                    "SELECT g.decision_rule_constraints FROM authorization_records r \
                     JOIN authority_grants g ON g.grant_id = r.grant_id WHERE r.record_id = $1",
                )
                .bind(&admitted.authz_ref)
                .fetch_optional(&mut *tx)
                .await?;
                if let Some(Some(constraint)) = constraint {
                    let allowed: Vec<String> = serde_json::from_value(constraint).map_err(|e| {
                        ControlApiError::internal_with_log(format!(
                            "stored decision-rule constraint unreadable: {e}"
                        ))
                    })?;
                    if !allowed.iter().any(|name| name == rule.as_str()) {
                        return Err(ControlApiError::invalid_command(format!(
                            "the admitting grant constrains the decision rules its subject may \
                             declare: `{}` is not among them",
                            rule.as_str()
                        )));
                    }
                }
            }
            let declared = match body.decision_rule {
                None => None,
                Some(rule) => {
                    let allowed = crate::charters::allows_on(&mut tx, &tenant_id.to_string(), rule)
                        .await
                        .map_err(|e| match e {
                            crate::charters::CharterError::Storage(detail) => {
                                eprintln!("control api: charter read failed: {detail}");
                                ControlApiError::internal()
                            }
                            crate::charters::CharterError::UnrepresentableInput => {
                                unrepresentable()
                            }
                            refusal => ControlApiError::invalid_command(refusal.to_string()),
                        })?;
                    Some(threads::DeclaredRule {
                        rule,
                        threshold: allowed.threshold,
                        charter_digest: allowed.charter_digest,
                    })
                }
            };
            let thread_id = ThreadId::new();
            let prepared = threads::prepare_create(
                tenant_id,
                &thread_id,
                &principal.id_string(),
                body,
                workflow_steps.clone(),
                declared,
            );
            // After the authorization and the quota, whose refusals COMMIT their
            // recorded denial, and on this transaction: the resolution commits
            // with the thread it routed and with nothing else.
            if let Some(resolution) = resolution {
                crate::routing::record_resolution_in(
                    &mut tx,
                    &resolution.route,
                    &principal.id_string(),
                    "create_boundary",
                    &resolution.journal_tenant,
                )
                .await?;
            }
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
                        let dispatched = dispatch_work_in_tx(
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
                        .await;
                        match dispatched {
                            Ok(()) => {}
                            Err(DispatchRefusal::Api(error)) => return Err(error),
                            Err(DispatchRefusal::OfflineBacklog {
                                node,
                                undelivered,
                                cap,
                            }) => {
                                tx.rollback().await?;
                                return Err(backlog_refused(
                                    pool,
                                    tenant_id,
                                    principal,
                                    &thread_id,
                                    &node,
                                    undelivered,
                                    cap,
                                )
                                .await);
                            }
                        }
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
                        let dispatched = dispatch_work_in_tx(
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
                        .await;
                        match dispatched {
                            Ok(()) => {}
                            Err(DispatchRefusal::Api(error)) => return Err(error),
                            Err(DispatchRefusal::OfflineBacklog {
                                node,
                                undelivered,
                                cap,
                            }) => {
                                tx.rollback().await?;
                                return Err(backlog_refused(
                                    pool,
                                    tenant_id,
                                    principal,
                                    &thread_id,
                                    &node,
                                    undelivered,
                                    cap,
                                )
                                .await);
                            }
                        }
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
/// The target NODE is the one the role's work runs on, resolved through
/// `role_execution` (`SIGNOFF-REPAIR.5.3.5.3.1.3`): the role's own id under the
/// dev rule (`docs/decisions/2026-09-07_node-channel-wiring.md` — one node, one
/// role), the ORIGIN node for an origin-bound import while its recruitment
/// agreement stands, and none otherwise — refused, never enqueued where no node
/// reads it. The work item keeps the dispatching tenant and its admission, so
/// an origin node judges it by THAT tenant's epoch (`SIGNOFF-REPAIR.5.3.5.3.2`).
/// The command id correlates it with the thread event that produced it
/// (`work_{event_id}`).
async fn dispatch_work_in_tx<'e, E>(
    mut tx: E,
    spec: &DispatchSpec<'_>,
) -> Result<(), DispatchRefusal>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    // The node the role's work runs on — asked before anything is reserved or
    // counted, because every later step is about THAT node.
    let resolved: Option<Option<String>> =
        sqlx::query_scalar("SELECT node_id FROM role_execution WHERE role_id = $1")
            .bind(spec.agent_role)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(node) = resolved.flatten() else {
        return Err(ControlApiError::invalid_transition(format!(
            "the role `{}` runs on no node: it is bound to its origin's node and the \
             recruitment agreement with the origin no longer stands",
            spec.agent_role
        ))
        .into());
    };
    // §10.7's maximum offline backlog (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`),
    // asked FIRST so a refused dispatch reserves no budget: a node already
    // holding the cap's worth of undelivered work is handed nothing more, and
    // the command that would have handed it rolls back — an invitation exists
    // iff its work does, so the work is refused rather than silently dropped.
    // Counted on the RESOLVED node, which is the one that would hold it.
    let undelivered = node_channel::undelivered_in_tx(&mut *tx, &node).await?;
    if undelivered >= node_channel::MAX_OFFLINE_BACKLOG {
        return Err(DispatchRefusal::OfflineBacklog {
            node,
            undelivered,
            cap: node_channel::MAX_OFFLINE_BACKLOG,
        });
    }
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
        ))
        .into());
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
        &node,
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

/// Why a dispatch did not hand the node its work. `Api` is any refusal or
/// failure the caller answers as it is; `OfflineBacklog` is the one the caller
/// must roll back AND record as a storm control before answering
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`).
enum DispatchRefusal {
    Api(ControlApiError),
    OfflineBacklog {
        node: String,
        undelivered: i64,
        cap: i64,
    },
}

impl From<ControlApiError> for DispatchRefusal {
    fn from(error: ControlApiError) -> Self {
        DispatchRefusal::Api(error)
    }
}

impl From<sqlx::Error> for DispatchRefusal {
    fn from(error: sqlx::Error) -> Self {
        DispatchRefusal::Api(error.into())
    }
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

/// The fold's idempotency key (`SIGNOFF-REPAIR.4.3.3`): the WORK ITEM's identity,
/// which is a node's inbox row — the node and its command id — never the command
/// id alone. Command ids are unique per node (`UNIQUE (node_id, command_id)`), so
/// two nodes holding one id in a tenant are two work items, each owed its own
/// fold; the same node re-emitting a result under a new event id still replays.
/// Migration `0104` re-keyed the stored rows.
pub(crate) fn node_result_fold_key(node_id: &str, command_id: &str) -> String {
    format!("{node_id}:{command_id}")
}

/// The `.6.2` node-result path (called from the node channel's `events` handler):
/// fold a node-emitted `work_result` into its thread through the SAME
/// claim → authorize → validate → apply flow a CLI command rides. The idempotency
/// key is the work item's identity — [`node_result_fold_key`], the node and its
/// inbox command id — so a duplicated transport produces a replay, never a second
/// domain effect, and another node's work item under the same id is its own. The
/// caller owns the transaction; a rejection is stored as the work item's
/// idempotent result and returned as the error (the receipt still commits — the
/// node DID emit this event).
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
    //
    // ⛔ NOT a row whose result this node already delivered
    // (`SIGNOFF-REPAIR.4.4.8`): its work is done, and a dead letter for it is a
    // contradiction. Such a report was sent for EVERY successful item, one tick
    // after its success, and the quarantine outranks every other delivery state,
    // so finished work read `dead_lettered`. The node no longer sends it; this
    // guard keeps any report, of any origin, from undoing a delivered result.
    // The result is found by the command it names (`payload->>'command_id'`,
    // what the fold itself reads), never by its operation id, which is the
    // node's own and never the command's.
    if kind == "work_dead_lettered" {
        let Some(reason) = payload.get("reason").and_then(|v| v.as_str()) else {
            return Ok(()); // malformed report — the receipt stands, no domain effect
        };
        crate::telemetry::metrics().incr("dead_letters");
        sqlx::query(
            "UPDATE node_inbox SET quarantined_at = $4, quarantine_reason = $5 \
             WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3 \
               AND quarantined_at IS NULL \
               AND NOT EXISTS (SELECT 1 FROM node_events e \
                               WHERE e.node_id = node_inbox.node_id \
                                 AND e.payload->>'kind' = 'work_result' \
                                 AND e.payload->>'command_id' = node_inbox.command_id)",
        )
        .bind(node_id)
        .bind(command_id)
        .bind(guarded_tenant.to_string())
        .bind(Utc::now())
        .bind(format!("dead-lettered: {reason}"))
        .execute(&mut *tx)
        .await?;
        // `retry_requires_authorization` is the node's report that the attempt's
        // provider response was LOST, so its outcome is unknown and the lost call
        // may have consumed the hold: keep it counted past its window
        // (`SIGNOFF-REPAIR.4.5.1`). The reservation is the one the server stored
        // on the inbox row, read under the guarded tenant, never one the node
        // names.
        if reason == reasonbraid_core::RETRY_REQUIRES_AUTHORIZATION {
            if let Some((_, _, work)) = node_channel::load_command_for_tenant_in_tx(
                &mut *tx,
                node_id,
                command_id,
                &guarded_tenant.to_string(),
            )
            .await?
            {
                if let Some(reservation_id) = work
                    .get("reservation")
                    .and_then(|r| r.get("reservation_id"))
                    .and_then(|v| v.as_str())
                {
                    let attempt = payload
                        .get("attempt_id")
                        .and_then(|v| v.as_str())
                        .map(|attempt| (node_id, attempt));
                    budget::hold_for_unknown_outcome_in_tx(
                        &mut *tx,
                        reservation_id,
                        &guarded_tenant.to_string(),
                        attempt,
                        Utc::now(),
                    )
                    .await?;
                }
            }
        }
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

    // The acting role is the one the WORK ITEM names — read from the server's
    // own inbox row, never from anything the node sends (`SIGNOFF-REPAIR.5.3.5.3.1.4`).
    // Under the dev rule it is the node's own id; for an origin-bound import it
    // is the imported identity, whose work the origin node ran. A row written
    // before the item named its role is the dev rule's, and names the node.
    let role_raw = work
        .get("agent_role")
        .and_then(|v| v.as_str())
        .unwrap_or(node_id)
        .to_string();
    let Ok(role) = role_raw.parse::<AgentRoleId>() else {
        return Ok(());
    };
    let principal = GrantSubject::Role(role);

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut body = if operation == threads::OP_REVISE {
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
    // Where the node replaced a U+0000 no store can hold (`SIGNOFF-REPAIR.4.4.10.3.1`):
    // forwarded as the node stated it, and validated by the command like any
    // author's, so a false claim is this result's rejection. Absent when the
    // node replaced nothing, which keeps every other result's hash unchanged.
    if let Some(runs) = payload.get("nul_positions") {
        body["nul_positions"] = runs.clone();
    }
    // `None` for the target: this path's idempotency key is the server-assigned
    // `command_id` (with its node), which belongs to exactly one thread, so no
    // caller can vary the target under a fixed key. Binding it here would change
    // every stored hash and turn an in-flight redelivery into a conflict for no
    // gain (`SIGNOFF-REPAIR.3.4.6`).
    let hash = request_hash(operation, &principal, &body, None, None);
    let fold_key = node_result_fold_key(node_id, command_id);
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::ThreadContribute,
        target: ResourceTarget::Thread {
            tenant_id,
            thread_id,
        },
    };

    // Claim FIRST: a re-emitted result (original event id, same payload) replays
    // the ORIGINAL stored outcome — the first receipt already applied the work
    // and settled its reservation.
    match tx::claim_idempotency_in_tx(&mut *tx, &tenant_id.to_string(), &fold_key, &hash).await? {
        ClaimOutcome::Replay { .. } => return Ok(()),
        ClaimOutcome::Fresh => {}
    }

    // The node may act for that role only while the role's work RUNS ON IT,
    // asked now (`role_execution`): an origin node whose binding lapsed after
    // the work was delivered no longer speaks for the identity, and its result
    // is refused and recorded like any other rejection. A node's own role
    // always resolves to the node, so the dev rule is unchanged.
    let runs_here: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM role_execution WHERE role_id = $1 AND node_id = $2)",
    )
    .bind(&role_raw)
    .bind(node_id)
    .fetch_one(&mut *tx)
    .await?;
    if !runs_here {
        let err = ControlApiError::unauthorized(format!(
            "the node `{node_id}` does not run role `{role_raw}`: its result is not the role's"
        ));
        store_rejection(&mut *tx, &tenant_id, &fold_key, &err.failure_result()).await?;
        settle_work_item_reservation(&mut *tx, &work, payload).await?;
        return Err(err);
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
            store_rejection(&mut *tx, &tenant_id, &fold_key, &err.failure_result()).await?;
            settle_work_item_reservation(&mut *tx, &work, payload).await?;
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
            store_rejection(&mut *tx, &tenant_id, &fold_key, &err.failure_result()).await?;
            settle_work_item_reservation(&mut *tx, &work, payload).await?;
            return Err(err);
        }
    };

    let cmd = Command {
        tenant_id: tenant_id.to_string(),
        aggregate_type: threads::AGGREGATE_TYPE.to_string(),
        aggregate_id: thread_id.to_string(),
        idempotency_key: fold_key,
        request_hash: hash,
        event_id: prepared.event_id.to_string(),
        event_type: prepared.event_type.to_string(),
        body: prepared.event_body,
        next_state: prepared.next_state,
        result: prepared.result.clone(),
    };
    tx::apply_fresh_in_tx(&mut *tx, &cmd).await?;

    settle_work_item_reservation(&mut *tx, &work, payload).await?;

    crate::telemetry::metrics().incr("results_folded");
    Ok(())
}

/// Settle a work item's reservation with the usage its node reports
/// (`SIGNOFF-REPAIR.4.4.2`) — on the fold's success AND on every refusal that
/// follows the idempotency claim, because the provider ran whatever the domain
/// then decided, and the usage is spend. Until this repair only the success path
/// settled: a refused result left its reservation `active` until it expired,
/// after which the ceiling read its NULL usage as nothing — the spend was
/// charged nowhere.
///
/// The reservation is the one the SERVER put on the work item (`work`, the
/// stored inbox payload), never an id the node names: the settlement used to
/// read `reservation_id` from the node's own payload, which let a node settle
/// any reservation it could name. Idempotent — a settled/released/denied row is
/// terminal, so a replay settles nothing twice.
async fn settle_work_item_reservation<'e, E>(
    mut tx: E,
    work: &Value,
    payload: &Value,
) -> Result<(), ControlApiError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let Some(reservation_id) = work
        .get("reservation")
        .and_then(|r| r.get("reservation_id"))
        .and_then(|v| v.as_str())
        .filter(|id| !id.is_empty())
    else {
        return Ok(()); // a budget-denied dispatch carries no reservation
    };
    let reported = |field: &str| {
        payload
            .get("usage")
            .and_then(|u| u.get(field))
            .and_then(|v| v.as_u64())
    };
    // The node measures the attempt's wall-clock seconds and reports them with
    // its tokens (`SIGNOFF-REPAIR.4.4.6.1`); settled, they are spent against the
    // ceiling's clock like the tokens are against its token dimensions.
    let usage = BudgetDimensions::attempt_usage(
        reported("input_tokens"),
        reported("output_tokens"),
        reported("wall_clock_seconds"),
    );
    budget::settle_reservation_in_tx(&mut *tx, reservation_id, &usage, Utc::now()).await?;
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
    let mut resolution = None;
    let profile_id = match body.workflow_profile.as_deref() {
        Some(explicit) => Some(explicit.to_owned()),
        None => match body.routing_class.as_deref() {
            Some(class) => {
                let route = crate::routing::resolve(&state.pool, class)
                    .await
                    .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
                // ⛔ The journal's tenant is DERIVED from the authenticated
                // caller, NOT taken from `body.tenant_id`, so a caller-supplied
                // tenant cannot inject rows into another tenant's audit trail
                // (`SIGNOFF-REPAIR.7.1.2.2`). The row itself is written by the
                // command, after its authorization (`SIGNOFF-REPAIR.8.2.7`).
                let Some(journal_tenant) = reader_tenant(&state.pool, &principal).await? else {
                    return Err(ControlApiError::unauthorized(
                        "an unenrolled principal resolves no route",
                    ));
                };
                let arm = route.arm.clone();
                resolution = Some(CreateResolution {
                    route,
                    journal_tenant,
                });
                Some(arm)
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
    // `SIGNOFF-REPAIR.8.1.1.2`: the declared rule's PURE checks run here, where
    // they reveal nothing about the tenant. Its charter is read inside the
    // command transaction, after authorization.
    if let Some(rule) = body.decision_rule {
        threads::validate_declared_rule(rule, &workflow_steps)?;
    }
    // `SIGNOFF-REPAIR.11.4.7.2.1.2.2`: pure, so it too runs before authorization.
    if let Some(artifact) = &body.expected_artifact {
        threads::validate_expected_artifact(artifact)?;
    }
    let tenant_id = body.tenant_id;
    // `None`: a creation has no thread to bind. Its target is the TENANT, which
    // is already the first column of `idempotency`'s primary key, so two creates
    // that differ only in target cannot share a row (`SIGNOFF-REPAIR.3.4.6`).
    let hash = request_hash(
        threads::OP_CREATE,
        &principal,
        &envelope.body,
        envelope.authority_context.as_ref(),
        None,
    );
    let delegation = delegation_from_envelope(&envelope)?;
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegation,
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
            resolution,
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

    // The TARGET the command acts on, hashed alongside the body
    // (`SIGNOFF-REPAIR.3.4.6`). It arrives as a path segment and no typed body
    // carries it, so without this the same actor, body and key against a
    // DIFFERENT thread in the same tenant hashed identically and replayed the
    // first thread's stored result.
    let target = thread_id.to_string();

    // Map the operation to its typed body, authorization action, and tenant scope.
    let (tenant_id, authz_action, hash) = match envelope.operation.as_str() {
        threads::OP_INVITE => {
            let body: threads::InviteBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadInvite,
                request_hash(
                    threads::OP_INVITE,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_CONTRIBUTE => {
            let body: threads::ContributeBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(
                    threads::OP_CONTRIBUTE,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_ADVANCE_ROUND => {
            let body: threads::AdvanceRoundBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadAdvanceRound,
                request_hash(
                    threads::OP_ADVANCE_ROUND,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_CHALLENGE => {
            let body: threads::ChallengeBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(
                    threads::OP_CHALLENGE,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_REVISE => {
            let body: threads::ReviseBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(
                    threads::OP_REVISE,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_CLOSE => {
            let body: threads::CloseBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadClose,
                request_hash(
                    threads::OP_CLOSE,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_CANCEL => {
            let body: threads::CancelBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadCancel,
                request_hash(
                    threads::OP_CANCEL,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_ACCEPT_INVITATION => {
            let body: threads::AcceptInvitationBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadInvitationRespond,
                request_hash(
                    threads::OP_ACCEPT_INVITATION,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
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
                request_hash(
                    threads::OP_DECLINE_INVITATION,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        threads::OP_JOIN => {
            let body: threads::JoinBody = serde_json::from_value(envelope.body.clone())
                .map_err(|e| ControlApiError::invalid_command(e.to_string()))?;
            let tenant = body.tenant_id;
            (
                tenant,
                GrantAction::ThreadContribute,
                request_hash(
                    threads::OP_JOIN,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
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
                request_hash(
                    threads::OP_REMOVE_PARTICIPANT,
                    &principal,
                    &envelope.body,
                    envelope.authority_context.as_ref(),
                    Some(target.as_str()),
                ),
            )
        }
        other => {
            return Err(ControlApiError::invalid_command(format!(
                "unknown thread operation `{other}`"
            )))
        }
    };

    let delegation = delegation_from_envelope(&envelope)?;
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal: principal.clone(),
        delegation,
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

/// Authorize an inspection read: the `thread_inspect` grant over the target,
/// through the guarded authorization that records the audit row for reads too
/// (the audit view is complete, `.5.1`).
///
/// ⭐ Shared with the MCP read seam since `SIGNOFF-REPAIR.6.1.1`
/// ([`crate::mcp_read`]). That surface's header claimed "the SAME
/// authorization as the HTTP handlers" over a private re-implementation that
/// ran none of it; the claim is now carried by this call rather than by a
/// sentence, so the two surfaces cannot drift apart without the compiler
/// noticing.
pub(crate) async fn authorize_inspection(
    pool: &PgPool,
    principal: &GrantSubject,
    target: ResourceTarget,
) -> Result<(), ControlApiError> {
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::ThreadInspect,
        target,
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

/// Authorize an inspection read, then run it.
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
    authorize_inspection(&state.pool, principal, target).await?;
    let body = read(state.pool.clone()).await?;
    Ok(json_response(StatusCode::OK, body))
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
        |pool| async move { thread_inspection(&pool, tenant_id, thread_id).await },
    )
    .await
}

/// One thread's stored projection, tenant-bound under the tenant's RLS claim —
/// the same select `thread_inspection` makes, without the derived view. `None`
/// for a thread id that does not exist in this tenant.
async fn load_thread_projection(
    pool: &PgPool,
    tenant_id: TenantId,
    thread_id: ThreadId,
) -> Result<Option<threads::ThreadProjection>, ControlApiError> {
    let tenant = tenant_id.to_string();
    let thread = thread_id.to_string();
    let claim = tenant.clone();
    let row: Option<(Value,)> = crate::rls::with_tenant_claim(pool, &claim, |tx| {
        Box::pin(async move {
            sqlx::query_as(
                "SELECT state FROM aggregate_state \
                 WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_type = 'thread'",
            )
            .bind(tenant)
            .bind(thread)
            .fetch_optional(&mut *tx)
            .await
        })
    })
    .await?;
    row.map(|(state,)| {
        serde_json::from_value(state).map_err(|e| {
            ControlApiError::internal_with_log(format!("corrupt stored thread state: {e}"))
        })
    })
    .transpose()
}

/// Whether a thread exists in this tenant — the tenant-BOUND select
/// [`thread_inspection`] makes, reduced to its predicate (`SIGNOFF-REPAIR.17`).
/// The sub-reads ask it first, so a thread the caller cannot see gets ONE
/// answer from every view — [`ControlApiError::scope_hidden`], whether the id
/// is another tenant's or nobody's. A row-less `event_log` or
/// `authorization_records` select cannot tell those apart from an existing
/// thread with nothing to show, which is how the timeline came to answer an
/// absent thread with `200 []` and the audit view with the caller's own
/// inspection records while the item and budget reads answered `404`.
async fn thread_exists(
    pool: &PgPool,
    tenant_id: TenantId,
    thread_id: ThreadId,
) -> Result<bool, ControlApiError> {
    let tenant = tenant_id.to_string();
    let thread = thread_id.to_string();
    let claim = tenant.clone();
    let exists: bool = crate::rls::with_tenant_claim(pool, &claim, |tx| {
        Box::pin(async move {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM aggregate_state \
                 WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_type = 'thread')",
            )
            .bind(tenant)
            .bind(thread)
            .fetch_one(&mut *tx)
            .await
        })
    })
    .await?;
    Ok(exists)
}

/// The thread inspection's read half: the tenant-BOUND `aggregate_state`
/// select under that tenant's RLS claim, plus the `.1.3.1` derived view.
///
/// The select's predicate is the binding: a thread id belonging to another
/// tenant selects no row and leaves as [`ControlApiError::scope_hidden`], so
/// the caller's named tenant and the requested thread cannot disagree and
/// still return data.
///
/// ⭐ Shared with the MCP read seam since `SIGNOFF-REPAIR.6.1.1`: the query
/// exists ONCE, so "the same queries as the HTTP handlers" is a call graph
/// rather than a claim. The seam therefore also inherits the derived view —
/// an `invited` entry past its `expires_at` reads `expired` on both surfaces,
/// which it did not before, the tool having returned the stored projection.
pub(crate) async fn thread_inspection(
    pool: &PgPool,
    tenant_id: TenantId,
    thread_id: ThreadId,
) -> Result<Value, ControlApiError> {
    let tenant = tenant_id.to_string();
    let thread = thread_id.to_string();
    let claim = tenant.clone();
    let row: Option<(String, Value)> = crate::rls::with_tenant_claim(pool, &claim, |tx| {
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
            let stored: threads::ThreadProjection =
                serde_json::from_value(state_json).map_err(|e| {
                    ControlApiError::internal_with_log(format!("corrupt stored thread state: {e}"))
                })?;
            let viewed = threads::derived_view(stored);
            Ok(json!({
                "thread_id": thread_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "state": serde_json::to_value(&viewed).expect("the projection serializes"),
            }))
        }
        None => Err(ControlApiError::scope_hidden()),
    }
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
            // `SIGNOFF-REPAIR.17`: the item read's answer for a thread the
            // caller cannot see, before a row-less select could say `[]`.
            if !thread_exists(&pool, tenant_id, thread_id).await? {
                return Err(ControlApiError::scope_hidden());
            }
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

/// Admit an administrator of the caller's OWN tenant to a process-wide read,
/// and return the committed authorization record's id (the receipt the
/// response carries). Shared by `GET /v1/admin/metrics` and
/// `GET /v1/admin/backups` (`SIGNOFF-REPAIR.4.6.1.5.2`): both report on the
/// process rather than on a tenant's aggregate, so neither takes a `tenant_id`.
async fn admit_own_tenant_admin(
    pool: &PgPool,
    principal: &GrantSubject,
    refusal: &'static str,
) -> Result<String, ControlApiError> {
    // ⚠️ The reasoning below was written for the metrics read, whose leaf
    // (`.3.5.2.1`) introduced this admission; it applies unchanged to every
    // process-wide read that shares it.
    // `SIGNOFF-REPAIR.3.5.2.1`: this read is AUDITED, and the tenant its record
    // binds to is the caller's OWN — not one the request names.
    //
    // ⭐ The route takes no `tenant_id`, and it does not need one. A principal
    // belongs to exactly ONE tenant, structurally: `migrations/0007` declares
    // `human_principals.principal_id` and `agent_roles.role_id` as PRIMARY KEY,
    // each with a single `tenant_id`. So the tenant is DERIVED from the
    // authenticated caller rather than supplied by it — the same shape
    // `inspect_call` uses, and the reason no wire change was needed to start
    // recording who took this read.
    //
    // ⛔ Deliberately the ORDINARY guarded admission, not
    // `authorize_tenant_admin_inspection`. That entry point applies the
    // frozen-boundary carve-out, which exists so an administrator can inspect
    // AUTHORITY state while a revocation is in flight; process counters are not
    // authority state, so `.3.5.2` ruled the carve-out is not this surface's to
    // inherit. `authorize_guarded` records `boundary_checked` like any other
    // ordinary read — which is also why no `TenantAdminInspection` variant is
    // added and no stored evaluation discriminant changes.
    //
    // ⚠️ The width NARROWS, declared rather than smuggled. The superseded gate
    // asked `subject_id = $1` with NO tenant predicate — any active
    // `tenant_admin` grant in ANY tenant. Selection filters on the tenant, so
    // the rule is now "an administrator of your own tenant". Nothing binds a
    // grant's tenant to its subject's tenant (that is `R-85-1` clause 2's shape,
    // owned at `.3.3`), but the only two producers — development enrolment and
    // card import — both create the principal in the grant's own tenant, so no
    // reachable caller loses access. The deliberate half of the width is intact:
    // an admin still sees ALL the process's counters, not a per-tenant slice.
    let Some(tenant) = reader_tenant(pool, principal).await? else {
        return Err(ControlApiError::unauthorized(refusal));
    };
    let tenant_id: TenantId = tenant.parse().map_err(|_| {
        ControlApiError::internal_with_log(format!(
            "the caller's stored tenant `{tenant}` is malformed"
        ))
    })?;
    let authz = CommandAuthz {
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegation: None,
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
        AuthorizationOutcome::Allowed { record_id, .. } => Ok(record_id),
    }
}

/// What `GET /v1/admin/backups` reads: the store its admission is recorded in,
/// and the declared backup directory (`None` when `rb-server` was started
/// without `--backup-dir`).
struct BackupState {
    pool: PgPool,
    backup_dir: Option<std::path::PathBuf>,
}

/// `GET /v1/admin/backups` — backup/restore status from the receipts the
/// backup scripts write (`SIGNOFF-REPAIR.4.6.1.5.2`; ROADMAP §18.5 and §17.5).
/// The report and its `recovery_control` verdict are `crate::backups::report`'s;
/// this handler admits the caller and reads the directory off the async
/// runtime. Process-wide, like the metrics read, so it shares that admission.
async fn admin_backups(
    State(state): State<Arc<BackupState>>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let receipt = admit_own_tenant_admin(
        &state.pool,
        &principal,
        "the backup status is tenant_admin-gated",
    )
    .await?;
    let dir = state.backup_dir.clone();
    let body =
        tokio::task::spawn_blocking(move || crate::backups::report(dir.as_deref(), Utc::now()))
            .await
            .map_err(|e| {
                ControlApiError::internal_with_log(format!("the backup report failed: {e}"))
            })?;
    Ok(([("x-reasonbraid-authorization", receipt)], Json(body)).into_response())
}

/// The backup-status route over `backup_dir` (`SIGNOFF-REPAIR.4.6.1.5.2`). A
/// router of its own, like `node_router`, so the directory reaches it without
/// widening every `ApiState` constructor.
pub fn backup_router(pool: PgPool, backup_dir: Option<std::path::PathBuf>) -> Router {
    Router::new()
        .route("/v1/admin/backups", get(admin_backups))
        .with_state(Arc::new(BackupState { pool, backup_dir }))
}

/// `GET /v1/admin/metrics` — the operational metrics surface (`.5.2`; ADR-023:
/// OPERATIONAL counters, never audit facts — the audit record is the durable
/// table row). tenant_admin-gated, read-only.
async fn admin_metrics(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Response, ControlApiError> {
    let principal = resolve_principal(&headers)?;
    let receipt = admit_own_tenant_admin(
        &state.pool,
        &principal,
        "the metrics surface is tenant_admin-gated",
    )
    .await?;
    let snapshot: serde_json::Map<String, Value> = crate::telemetry::metrics()
        .snapshot()
        .into_iter()
        .map(|(k, v)| (k, json!(v)))
        .collect();
    // The receipt names the admission this read committed. Without it the record
    // would be unreachable: there is no list endpoint.
    Ok((
        [("x-reasonbraid-authorization", receipt)],
        Json(Value::Object(snapshot)),
    )
        .into_response())
}

/// Preserve the registry response bodies while exposing the committed receipt.
/// All authorization, registry effects and audit persistence happen in the service.
async fn site_registry_response(
    state: &ApiState,
    principal: &GrantSubject,
    command: RegistryCommand,
) -> Result<Response, ControlApiError> {
    let outcome = site::execute(&state.pool, principal, &command).await;
    // ⭐ The hand-matched `undeclared_region` special case that used to sit here
    // is GONE, subsumed by `SIGNOFF-REPAIR.16`: every domain refusal now renders
    // as a bad request because the TYPE says it is one, so the rule no longer
    // depends on a reason string being remembered at one call site.
    site_receipt_response(
        outcome,
        "a current site grant for this action and its actual boundary are required",
    )
}

/// One site receipt on the wire: the audit id rides the response either way, so
/// an allowed act and a refused one are equally traceable.
fn site_receipt_response(
    outcome: Result<site::Receipt, site::Error>,
    denial: &'static str,
) -> Result<Response, ControlApiError> {
    match outcome {
        Ok(receipt) => Ok((
            [("x-reasonbraid-site-audit", receipt.audit_id)],
            Json(receipt.result),
        )
            .into_response()),
        // ⛔ THE CALLER DID NOT HOLD THE AUTHORITY. 403, and the only case that
        // belongs in the `authorization_denials` metric.
        Err(site::Error::Denied { reason, audit_id }) => {
            crate::telemetry::metrics().incr("authorization_denials");
            Ok((
                StatusCode::FORBIDDEN,
                Json(json!({"code": reason, "message": denial, "audit_id": audit_id})),
            )
                .into_response())
        }
        // ⛔ THE CALLER HELD THE AUTHORITY AND THE ACT WAS REFUSED ON ITS OWN
        // TERMS (`SIGNOFF-REPAIR.16`). Until it, this fell into the arm above
        // and answered an authorized caller `403` with *a current site grant
        // for this action and its actual boundary are required* — a sentence
        // that is FALSE of someone holding the grant — and counted them as an
        // authorization denial.
        //
        // ⭐ The distinction is already written down one function below, where
        // `site_registry_response` hand-matched ONE reason string to render it
        // as a bad request: *a domain refusal, not an authority one: the caller
        // held the grant and asked for something the registry cannot express*.
        // That one-off is why the principle reached one reason out of sixteen;
        // it is now the type's job, so the compiler reaches all of them.
        //
        // ⚠️ The audit is UNCHANGED: a refused act is recorded `denied` either
        // way, because the act did not take effect. What changes is only what
        // the CALLER is told, and the `audit_id` still resolves the full reason.
        Err(site::Error::Refused { reason, audit_id }) => Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"code": reason, "message": reason, "audit_id": audit_id})),
        )
            .into_response()),
        Err(site::Error::InvalidInput(reason)) => Err(ControlApiError::invalid_command(reason)),
        Err(site::Error::OperatorRequired) => {
            Err(ControlApiError::unauthorized("site authority required"))
        }
        Err(site::Error::Sql(error)) => Err(error.into()),
    }
}

// Do not echo malformed caller input or driver diagnostics. Keep body-size and
// media-type refusal statuses; normalize JSON syntax/schema errors to typed 400.
/// A JSON body, or the `400 invalid_command` refusal that says what was wrong
/// with it in serde's own words, which name an unknown or missing field
/// (`SIGNOFF-REPAIR.7.4.7`).
///
/// ⚠️ A route taking a bare `Json<T>` answers the same body with axum's default,
/// `422` and a plain-text body carrying no `code`, against the errors chapter's
/// promise that every refusal carries one. `SIGNOFF-REPAIR.11.36` owns the
/// routes still doing so.
fn json_body<T>(
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
            message: rejection.body_text(),
        }
    })
}

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
            message: "a site request requires the documented JSON fields, bounded names and a nonblank reason".into(),
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

/// The usage view's answer when a ledger sum exceeds `u64`
/// (`SIGNOFF-REPAIR.4.5.2`): a recorded usage too large to count, which only a
/// node's report can put there. Its own code, rather than a wrong total or a
/// bare `internal`, so the operator knows to look for that report.
fn ledger_overflow(error: BudgetError) -> ControlApiError {
    ControlApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: LEDGER_OVERFLOW,
        message: format!("the budget ledger cannot be summed: {error}"),
    }
}

/// The code [`ledger_overflow`] answers with.
pub(crate) const LEDGER_OVERFLOW: &str = "ledger_overflow";

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
            type Row = (String, Value, Option<Value>, String, Option<String>, bool);
            // Whether a row still holds is the ledger's own rule, asked of the
            // store, so this view cannot count what admission does not
            // (`SIGNOFF-REPAIR.4.5.1`).
            let rows: Vec<Row> = sqlx::query_as(concat!(
                "SELECT r.thread_id, r.dimensions, r.usage, r.status, r.reason, COALESCE(",
                budget::holding!(),
                ", false) FROM budget_reservations r WHERE r.tenant_id = $1 ORDER BY r.created_at",
            ))
            .bind(q.tenant_id.to_string())
            .bind(Utc::now())
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

            for (thread_id, dimensions, usage, status, reason, holding) in &rows {
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
                    "active" if *holding => {
                        *held = held.add(&reserved).map_err(ledger_overflow)?;
                        tenant_held = tenant_held.add(&reserved).map_err(ledger_overflow)?;
                    }
                    "settled" => {
                        let used: BudgetDimensions = usage
                            .as_ref()
                            .map(|u| {
                                serde_json::from_value(u.clone()).expect("stored usage parses")
                            })
                            .unwrap_or_default();
                        *settled = settled.add(&used).map_err(ledger_overflow)?;
                        tenant_settled = tenant_settled.add(&used).map_err(ledger_overflow)?;
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
                        *overrun = overrun.add(&over).map_err(ledger_overflow)?;
                        tenant_overrun = tenant_overrun.add(&over).map_err(ledger_overflow)?;
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
                Option<DateTime<Utc>>,
            );
            let rows: Vec<Row> = sqlx::query_as(
                "SELECT reservation_id, dimensions, usage, status, reason, expires_at, \
                 created_at, settled_at, outcome_unknown_at FROM budget_reservations \
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
                        outcome_unknown_at,
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
                        // The hold is counted past `expires_at` because its
                        // attempt's outcome is unknown (`SIGNOFF-REPAIR.4.5.1`).
                        if let Some(at) = outcome_unknown_at {
                            row.insert("outcome_unknown_at".into(), json!(at.to_rfc3339()));
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
            // `SIGNOFF-REPAIR.17`: the item read's answer for a thread the
            // caller cannot see — before the records select, which would
            // otherwise return the inspection record this very call wrote.
            if !thread_exists(&pool, tenant_id, thread_id).await? {
                return Err(ControlApiError::scope_hidden());
            }
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

#[cfg(test)]
mod storage_failures {
    use super::{storage_failure, UNREPRESENTABLE_INPUT};
    use std::borrow::Cow;

    /// A database error carrying exactly one SQLSTATE, for the classifier.
    #[derive(Debug)]
    struct Coded(&'static str);

    impl std::fmt::Display for Coded {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "sqlstate {}", self.0)
        }
    }

    impl std::error::Error for Coded {}

    impl sqlx::error::DatabaseError for Coded {
        fn message(&self) -> &str {
            self.0
        }
        fn code(&self) -> Option<Cow<'_, str>> {
            Some(Cow::Borrowed(self.0))
        }
        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }
        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }
        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }
        fn kind(&self) -> sqlx::error::ErrorKind {
            sqlx::error::ErrorKind::Other
        }
    }

    fn failure(code: &'static str) -> super::ControlApiError {
        storage_failure(sqlx::Error::Database(Box::new(Coded(code))), "a store")
    }

    /// The one decision every domain storage error takes
    /// (`SIGNOFF-REPAIR.4.4.10.1.1`): U+0000 in `jsonb` (22P05) or `text` (22021)
    /// is the caller's permanent `400`; any other failure, a unique violation or
    /// a lost connection alike, is the server's `500`.
    #[test]
    fn only_unrepresentable_input_is_the_callers() {
        for code in ["22P05", "22021"] {
            let answer = failure(code);
            assert_eq!(
                (answer.status.as_u16(), answer.code),
                (400, UNREPRESENTABLE_INPUT),
                "{code}"
            );
        }
        for code in ["23505", "08006", "40001", "XX000"] {
            let answer = failure(code);
            assert_eq!(
                (answer.status.as_u16(), answer.code),
                (500, "dependency_unavailable"),
                "{code}"
            );
        }
        let lost = storage_failure(sqlx::Error::PoolTimedOut, "a store");
        assert_eq!(lost.status.as_u16(), 500);
    }
}

#[cfg(test)]
mod json_bodies {
    //! `json_body` (`SIGNOFF-REPAIR.7.4.7`) over the rejections axum's `Json`
    //! extractor produces. Each is produced BY the extractor, so the statuses
    //! are axum's own. They are tested here rather than over the wire because an
    //! oversized body is answered before it is read, and the connection that
    //! carried it is closed under a client still writing: measured, a live run
    //! then failed its NEXT request with a broken pipe.
    use super::{json_body, ControlApiError};
    use crate::claims::AssessmentRequest;
    use axum::extract::{FromRequest, Request};
    use axum::Json;

    async fn extract(
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> Result<AssessmentRequest, ControlApiError> {
        let mut request = Request::builder().method("POST").uri("/v1/assessments");
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        let request = request
            .body(axum::body::Body::from(body))
            .expect("a request");
        json_body(Json::<AssessmentRequest>::from_request(request, &()).await)
    }

    #[tokio::test]
    async fn an_unknown_field_is_a_400_that_names_it() {
        let refused = extract(Some("application/json"), br#"{"author":"x"}"#.to_vec())
            .await
            .unwrap_err();
        assert_eq!(
            (refused.status.as_u16(), refused.code),
            (400, "invalid_command")
        );
        assert!(
            refused.message.contains("unknown field `author`"),
            "{}",
            refused.message
        );
    }

    #[tokio::test]
    async fn a_body_that_is_not_json_keeps_its_415() {
        let refused = extract(None, b"{}".to_vec()).await.unwrap_err();
        assert_eq!(
            (refused.status.as_u16(), refused.code),
            (415, "invalid_command")
        );
    }

    #[tokio::test]
    async fn an_oversized_body_keeps_its_413() {
        let refused = extract(Some("application/json"), vec![b' '; 3 * 1024 * 1024])
            .await
            .unwrap_err();
        assert_eq!(
            (refused.status.as_u16(), refused.code),
            (413, "invalid_command")
        );
    }
}

#[cfg(test)]
mod publication_refusals {
    //! `publication_refusal` (`SIGNOFF-REPAIR.9.2.2`): the transition verbs used
    //! to answer a store fault as the caller's `400`. No live control can make
    //! the store fail on cue, so the mapping is pinned here.
    use super::{lifecycle_refusal, publication_refusal};
    use crate::lifecycle::LifecycleError;
    use crate::publications::PublicationError;

    /// `SIGNOFF-REPAIR.9.2.3`: the governance records' mapping, and the
    /// decision named as a decision rather than as a proposal.
    #[test]
    fn a_governance_store_fault_is_the_servers_and_a_missing_decision_is_named() {
        let fault = lifecycle_refusal(LifecycleError::Storage("relation gone".into()));
        assert_eq!(fault.status.as_u16(), 500);
        assert!(
            !fault.message.contains("relation gone"),
            "{}",
            fault.message
        );
        let missing = lifecycle_refusal(LifecycleError::UnknownDecision("dec-1".into()));
        assert_eq!(
            (missing.status.as_u16(), missing.code),
            (400, "invalid_command")
        );
        assert_eq!(missing.message, "decision `dec-1` does not exist");
    }

    #[test]
    fn a_store_fault_is_the_servers_and_a_wrong_stage_is_the_callers() {
        let fault = publication_refusal(PublicationError::Storage("connection reset".into()));
        assert_eq!(fault.status.as_u16(), 500);
        assert!(
            !fault.message.contains("connection reset"),
            "{}",
            fault.message
        );
        let refused = publication_refusal(PublicationError::WrongStage {
            publication_id: "pb".into(),
            state: "effective".into(),
        });
        assert_eq!(
            (refused.status.as_u16(), refused.code),
            (400, "invalid_command")
        );
        assert!(refused.message.contains("is at stage `effective`"));
    }
}
