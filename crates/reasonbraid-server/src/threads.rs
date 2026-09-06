//! The thread command domain for the WP6 control API (`PHASE-0.6.1`).
//!
//! # What this module owns
//!
//! The operation catalogue the CLI drives — `thread.create`, `thread.invite`,
//! `thread.contribute`, `thread.challenge`, `thread.revise`, `thread.close` — with
//! typed, `deny_unknown_fields` bodies, a minimal [`ThreadProjection`] stored in
//! `aggregate_state.state`, and deterministic validation against the core state
//! machines (`reasonbraid_core::state`). The HTTP layer (`crate::api`) resolves the
//! actor, authorizes, and commits; this module decides WHAT a command means and
//! whether the aggregate accepts it.
//!
//! # Dev-profile rules (recorded in `docs/decisions/2026-09-06_control-api-cli.md`)
//!
//! - **Challenge and revise are `thread_contribute` grants** — the WP5 action registry
//!   stays frozen; the three content verbs differ in message kind and event type, not
//!   in authority (`THREAD-004`: typed messages/contributions).
//! - **Auto-accept on first contribution** — `invited → accepted` when the invited
//!   role first contributes; the demo has no explicit accept verb.
//! - **Close folds `open → closing → closed`** into one command: both core-machine
//!   edges are validated, one `thread.closed` event records the terminal state.
//! - **The creator is a participant** from creation, so the organizer can contribute
//!   and challenge.
//! - **Content verbs require the thread to be `open`** and the actor to be a
//!   participant in `invited` or `accepted` state.
//!
//! # The projection
//!
//! The projection keeps only what validation and inspection need (state,
//! participants, counters, close reason, ceiling id, budget); the event log carries
//! the full content. `inspect` reconstructs the timeline from the log — never from
//! SQLite/psql surgery (the `.6.1` acceptance).

use std::collections::BTreeMap;

use reasonbraid_core::{
    AgentRoleId, BudgetDimensions, EventId, ParticipationState, ParticipationTransition, TenantId,
    ThreadId, ThreadState, ThreadTransition, TransitionError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Postgres;

pub const AGGREGATE_TYPE: &str = "thread";

/// Operation names (the `CommandEnvelope.operation` values this module accepts).
pub const OP_CREATE: &str = "thread.create";
pub const OP_INVITE: &str = "thread.invite";
pub const OP_CONTRIBUTE: &str = "thread.contribute";
pub const OP_CHALLENGE: &str = "thread.challenge";
pub const OP_REVISE: &str = "thread.revise";
pub const OP_CLOSE: &str = "thread.close";

/// The event types committed for the operations above.
pub const EVENT_CREATED: &str = "thread.created";
pub const EVENT_INVITED: &str = "thread.participant_invited";
pub const EVENT_CONTRIBUTED: &str = "thread.contribution_submitted";
pub const EVENT_CHALLENGED: &str = "thread.challenge_posted";
pub const EVENT_REVISED: &str = "thread.revision_submitted";
pub const EVENT_CLOSED: &str = "thread.closed";

/// The thread-work kinds an inbox payload carries (`PHASE-0.6.2`): an invitation
/// dispatches a `contribute` work item to the invited role's node; a challenge of a
/// role's contribution dispatches a `revise` work item to that role's node. The
/// node's journal treats exactly these kinds as thread work.
pub const WORK_CONTRIBUTE: &str = "contribute";
pub const WORK_REVISE: &str = "revise";

/// The reservation dimensions one work item requests (a single provider call with
/// a modest token and wall-clock allowance — covered by [`DEFAULT_BUDGET`]).
pub const WORK_RESERVATION: BudgetDimensions = BudgetDimensions {
    calls: Some(1),
    input_tokens: Some(2_000),
    output_tokens: Some(2_000),
    wall_clock_seconds: Some(120),
};

/// The dev-profile default thread budget when a create body names none: every
/// dimension metered (the fail-closed `BudgetDimensions::covers` refuses requests for
/// dimensions a ceiling does not meter).
pub const DEFAULT_BUDGET: BudgetDimensions = BudgetDimensions {
    calls: Some(10),
    input_tokens: Some(10_000),
    output_tokens: Some(10_000),
    wall_clock_seconds: Some(600),
};

// ── Operation bodies (client-supplied intent) ────────────────────────────────────

/// `thread.create` body: the tenant the thread belongs to, the subject/objective, and
/// an optional budget. The thread id itself is server-assigned.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
    pub tenant_id: TenantId,
    pub subject: String,
    pub objective: String,
    #[serde(default)]
    pub budget: Option<BudgetSpec>,
}

/// A budget specification for a new thread; unspecified dimensions take the
/// [`DEFAULT_BUDGET`] value (all dimensions metered in the dev profile).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetSpec {
    #[serde(default)]
    pub calls: Option<u64>,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub wall_clock_seconds: Option<u64>,
}

/// `thread.invite` body: the target tenant + thread scope and the invited agent role.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InviteBody {
    pub tenant_id: TenantId,
    pub agent_role: String,
}

/// `thread.contribute` body: the scope and the contribution content.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributeBody {
    pub tenant_id: TenantId,
    pub content: String,
}

/// `thread.challenge` body: the scope, the challenged contribution event, and the
/// challenge text.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeBody {
    pub tenant_id: TenantId,
    pub target_event_id: String,
    pub content: String,
}

/// `thread.revise` body: the scope, the challenge being answered, and the revision.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviseBody {
    pub tenant_id: TenantId,
    pub target_event_id: String,
    pub content: String,
}

/// `thread.close` body: the scope and the stop reason (preserved for the audit view).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloseBody {
    pub tenant_id: TenantId,
    pub reason: String,
}

// ── Projection ────────────────────────────────────────────────────────────────────

/// The minimal thread projection stored in `aggregate_state.state`. It keeps only
/// what validation and inspection need; content lives in the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadProjection {
    pub schema: u64,
    pub thread_id: ThreadId,
    pub subject: String,
    pub objective: String,
    pub state: ThreadState,
    /// The creating principal's wire id (`hpr_…`/`rol_…`).
    pub created_by: String,
    /// Principal wire id → participation state. The creator is `accepted` from day one.
    pub participants: BTreeMap<String, ParticipationState>,
    pub contributions: u64,
    pub revisions: u64,
    /// Challenges that no revision has answered (the unresolved register).
    pub open_challenges: u64,
    pub close_reason: Option<String>,
    pub ceiling_id: String,
    pub budget: BudgetDimensions,
}

// ── Errors ────────────────────────────────────────────────────────────────────────

/// A deterministic domain rejection. The HTTP layer maps each variant to a typed
/// §9.8 reason code and status.
#[derive(Debug)]
pub enum ThreadError {
    /// The body/operation is not a command this module accepts (or is malformed).
    InvalidCommand(String),
    /// The core state machine refused the move (or the thread is not `open` for a
    /// content verb).
    InvalidTransition(TransitionError),
    /// The aggregate has no row for this (tenant, thread) — reported as `scope_hidden`,
    /// never as a confirmed cross-tenant existence fact.
    ThreadNotFound,
    /// The actor has a grant but is not a participant of this thread.
    NotAParticipant { principal: String },
    /// The invite names a role that already has a participation record.
    AlreadyParticipant { principal: String },
    /// The stored projection does not parse (the database was modified outside the
    /// supported surface — the "no database surgery" acceptance's failure mode).
    CorruptState(String),
}

impl std::fmt::Display for ThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadError::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
            ThreadError::InvalidTransition(e) => write!(f, "{e}"),
            ThreadError::ThreadNotFound => write!(f, "thread not found"),
            ThreadError::NotAParticipant { principal } => {
                write!(
                    f,
                    "principal `{principal}` is not a participant of this thread"
                )
            }
            ThreadError::AlreadyParticipant { principal } => {
                write!(
                    f,
                    "principal `{principal}` already participates in this thread"
                )
            }
            ThreadError::CorruptState(detail) => {
                write!(f, "the stored thread state is corrupt: {detail}")
            }
        }
    }
}

impl std::error::Error for ThreadError {}

// ── Preparation ───────────────────────────────────────────────────────────────────

/// The fully validated result of preparing one command: everything the transaction
/// needs to write, and the semantic result stored for idempotent replay.
pub struct PreparedCommand {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub event_body: Value,
    pub next_state: Value,
    /// The success result (`{"ok": true, …}`); failures are stored by the caller.
    pub result: Value,
    /// `thread.create` only: the budget ceiling to write in the SAME transaction.
    pub ceiling: Option<(String, BudgetDimensions)>,
}

/// The semantic result every accepted command stores (and a replay returns verbatim).
fn success_result(
    thread_id: &ThreadId,
    event_id: &EventId,
    event_type: &str,
    state: ThreadState,
) -> Value {
    json!({
        "ok": true,
        "thread_id": thread_id.to_string(),
        "event_id": event_id.to_string(),
        "event_type": event_type,
        "thread_state": state.as_str(),
    })
}

fn ceiling_for(spec: Option<&BudgetSpec>) -> BudgetDimensions {
    match spec {
        None => DEFAULT_BUDGET,
        Some(spec) => BudgetDimensions {
            calls: spec.calls.or(DEFAULT_BUDGET.calls),
            input_tokens: spec.input_tokens.or(DEFAULT_BUDGET.input_tokens),
            output_tokens: spec.output_tokens.or(DEFAULT_BUDGET.output_tokens),
            wall_clock_seconds: spec
                .wall_clock_seconds
                .or(DEFAULT_BUDGET.wall_clock_seconds),
        },
    }
}

/// Prepare a `thread.create` — pure: a new thread has no prior state. The thread id
/// is server-assigned (the client proposes intent, never aggregate identity).
pub fn prepare_create(
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    principal: &str,
    body: &CreateBody,
) -> PreparedCommand {
    let budget = ceiling_for(body.budget.as_ref());
    let ceiling_id = format!("ceil_{thread_id}");
    let projection = ThreadProjection {
        schema: 1,
        thread_id: *thread_id,
        subject: body.subject.clone(),
        objective: body.objective.clone(),
        state: ThreadState::Open,
        created_by: principal.to_string(),
        participants: BTreeMap::from([(principal.to_string(), ParticipationState::Accepted)]),
        contributions: 0,
        revisions: 0,
        open_challenges: 0,
        close_reason: None,
        ceiling_id: ceiling_id.clone(),
        budget,
    };
    let event_id = EventId::new();
    let event_body = json!({
        "operation": OP_CREATE,
        "thread_id": thread_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "actor_principal_id": principal,
        "subject": body.subject,
        "objective": body.objective,
        "budget": budget,
    });
    PreparedCommand {
        event_id,
        event_type: EVENT_CREATED,
        event_body,
        next_state: serde_json::to_value(&projection).expect("projection serializes"),
        result: success_result(thread_id, &event_id, EVENT_CREATED, ThreadState::Open),
        ceiling: Some((ceiling_id, budget)),
    }
}

/// Require the thread to be `open` for a content/lifecycle verb; the rejection is a
/// core-machine-shaped [`TransitionError`] (the §9.8 `invalid_transition` path).
fn require_open(projection: &ThreadProjection, verb: &'static str) -> Result<(), ThreadError> {
    if projection.state != ThreadState::Open {
        return Err(ThreadError::InvalidTransition(TransitionError {
            aggregate: "Thread",
            from: projection.state.as_str(),
            event: verb,
        }));
    }
    Ok(())
}

/// The actor may act when it is an `invited` or `accepted` participant (an `invited`
/// role becomes `accepted` on its first contribution).
fn ensure_participant(
    projection: &mut ThreadProjection,
    principal: &str,
    accept_invited: bool,
) -> Result<(), ThreadError> {
    match projection.participants.get_mut(principal) {
        None => Err(ThreadError::NotAParticipant {
            principal: principal.to_string(),
        }),
        Some(state) => match *state {
            ParticipationState::Invited if accept_invited => {
                // Dev rule: the first contribution is the accept.
                *state = ParticipationState::Invited
                    .apply(ParticipationTransition::Accept)
                    .expect("invited accepts deterministically");
                Ok(())
            }
            ParticipationState::Invited | ParticipationState::Accepted => Ok(()),
            _ => Err(ThreadError::NotAParticipant {
                principal: principal.to_string(),
            }),
        },
    }
}

/// The event type of one event in this thread, if it exists (the challenge/revise
/// target check).
async fn event_type_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "SELECT event_type FROM event_log \
         WHERE event_id = $1 AND tenant_id = $2 AND aggregate_id = $3",
    )
    .bind(event_id)
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await
}

/// The `author` of one contribution/challenge event in this thread, if the event
/// exists and carries one (`PHASE-0.6.2`: the challenge dispatch sends its revise
/// work to the challenged contribution's author — a role's node — while a human
/// author revises through the CLI, so no dispatch happens).
pub(crate) async fn event_author_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    let body: Option<Value> = sqlx::query_scalar(
        "SELECT body FROM event_log \
         WHERE event_id = $1 AND tenant_id = $2 AND aggregate_id = $3",
    )
    .bind(event_id)
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    Ok(body.and_then(|b| b.get("author").and_then(|a| a.as_str()).map(str::to_string)))
}

/// Build the inbox payload for one thread work item (`PHASE-0.6.2`): the work
/// kind, the role it is for, the thread context the node's request carries, the
/// revise target when present, and the reservation (or the denial reason when the
/// ceiling refused to reserve).
pub fn work_payload(
    kind: &str,
    agent_role: &str,
    subject: &str,
    objective: &str,
    target_event_id: Option<&str>,
    reservation: Option<&reasonbraid_core::ReservationReference>,
    reservation_reason: Option<&str>,
) -> Value {
    let mut payload = json!({
        "kind": kind,
        "agent_role": agent_role,
        "subject": subject,
        "objective": objective,
        "reservation": reservation,
        "reservation_reason": reservation_reason,
    });
    if let Some(target) = target_event_id {
        payload["target_event_id"] = json!(target);
    }
    payload
}

/// Prepare a command against an EXISTING thread: read the locked projection, validate
/// the operation against the core state machines and the dev rules, and build the
/// next projection + event. The caller holds the transaction; the `FOR UPDATE` read
/// here is the same row the aggregate library's fresh apply re-reads
/// (`agg::apply_fresh_in_tx`, `PHASE-1.1.1`), so the version derived there matches
/// the state validated here.
pub(crate) async fn prepare_thread_command<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    operation: &str,
    principal: &str,
    body: &Value,
) -> Result<PreparedCommand, ThreadError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    let row: Option<(i64, Value)> = sqlx::query_as(
        "SELECT aggregate_version, state FROM aggregate_state \
         WHERE tenant_id = $1 AND aggregate_id = $2 FOR UPDATE",
    )
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ThreadError::CorruptState(e.to_string()))?;
    let Some((_, state_json)) = row else {
        return Err(ThreadError::ThreadNotFound);
    };
    let mut projection: ThreadProjection =
        serde_json::from_value(state_json).map_err(|e| ThreadError::CorruptState(e.to_string()))?;

    let event_id = EventId::new();
    let (event_type, event_body, next_state) = match operation {
        OP_INVITE => {
            let body: InviteBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "invite")?;
            let role: AgentRoleId = body.agent_role.parse().map_err(|_| {
                ThreadError::InvalidCommand(format!(
                    "agent_role `{}` is not a valid role identifier",
                    body.agent_role
                ))
            })?;
            if projection
                .participants
                .contains_key(body.agent_role.as_str())
            {
                return Err(ThreadError::AlreadyParticipant {
                    principal: body.agent_role.clone(),
                });
            }
            projection
                .participants
                .insert(body.agent_role.clone(), ParticipationState::Invited);
            (
                EVENT_INVITED,
                json!({
                    "operation": OP_INVITE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "agent_role": role.to_string(),
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CONTRIBUTE => {
            let body: ContributeBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "contribute")?;
            ensure_participant(&mut projection, principal, true)?;
            projection.contributions += 1;
            (
                EVENT_CONTRIBUTED,
                json!({
                    "operation": OP_CONTRIBUTE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "content": body.content,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CHALLENGE => {
            let body: ChallengeBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "challenge")?;
            ensure_participant(&mut projection, principal, false)?;
            match event_type_in_thread(&mut *tx, tenant_id, thread_id, &body.target_event_id)
                .await
                .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            {
                Some(t) if t == EVENT_CONTRIBUTED => {}
                Some(t) => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "challenge target `{}` is a `{t}` event, not a contribution",
                        body.target_event_id
                    )))
                }
                None => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "challenge target `{}` does not exist in this thread",
                        body.target_event_id
                    )))
                }
            }
            projection.open_challenges += 1;
            (
                EVENT_CHALLENGED,
                json!({
                    "operation": OP_CHALLENGE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "target_event_id": body.target_event_id,
                    "content": body.content,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_REVISE => {
            let body: ReviseBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "revise")?;
            ensure_participant(&mut projection, principal, false)?;
            match event_type_in_thread(&mut *tx, tenant_id, thread_id, &body.target_event_id)
                .await
                .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            {
                Some(t) if t == EVENT_CHALLENGED => {}
                Some(t) => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "revision target `{}` is a `{t}` event, not a challenge",
                        body.target_event_id
                    )))
                }
                None => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "revision target `{}` does not exist in this thread",
                        body.target_event_id
                    )))
                }
            }
            projection.open_challenges = projection.open_challenges.saturating_sub(1);
            projection.revisions += 1;
            (
                EVENT_REVISED,
                json!({
                    "operation": OP_REVISE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "target_event_id": body.target_event_id,
                    "content": body.content,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CLOSE => {
            let body: CloseBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            // Fold open → closing → closed (both core edges validated; one terminal
            // event records the result). A thread already `closing` only finalizes.
            let closed = match projection.state {
                ThreadState::Open => {
                    ThreadState::Open
                        .apply(ThreadTransition::BeginClose)
                        .map_err(ThreadError::InvalidTransition)?;
                    ThreadState::Closing
                        .apply(ThreadTransition::FinalizeClose)
                        .expect("finalize_close from closing is deterministic")
                }
                ThreadState::Closing => ThreadState::Closing
                    .apply(ThreadTransition::FinalizeClose)
                    .expect("finalize_close from closing is deterministic"),
                other => {
                    return Err(ThreadError::InvalidTransition(TransitionError {
                        aggregate: "Thread",
                        from: other.as_str(),
                        event: "close",
                    }))
                }
            };
            projection.state = closed;
            projection.close_reason = Some(body.reason.clone());
            (
                EVENT_CLOSED,
                json!({
                    "operation": OP_CLOSE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "reason": body.reason,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        other => {
            return Err(ThreadError::InvalidCommand(format!(
                "unknown thread operation `{other}`"
            )))
        }
    };

    Ok(PreparedCommand {
        event_id,
        event_type,
        event_body,
        next_state,
        result: success_result(thread_id, &event_id, event_type, projection.state),
        ceiling: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The close fold validates BOTH core edges: a closed thread rejects a second
    /// close, and the projection lands on `closed` with the reason preserved.
    #[test]
    fn close_fold_is_deterministic_and_terminal() {
        // A pure check that the fold is the documented one (the DB-backed path is
        // covered by the live-PG `command_api` suite).
        let from_open = ThreadState::Open
            .apply(ThreadTransition::BeginClose)
            .unwrap()
            .apply(ThreadTransition::FinalizeClose)
            .unwrap();
        assert_eq!(from_open, ThreadState::Closed);
        assert!(ThreadState::Closed
            .apply(ThreadTransition::BeginClose)
            .is_err());
    }

    /// Budget specs fill unspecified dimensions from the defaults — every dimension
    /// of a dev thread's ceiling is metered (fail-closed reservation coverage).
    #[test]
    fn budget_spec_defaults_every_dimension() {
        let dims = ceiling_for(Some(&BudgetSpec {
            calls: Some(1),
            input_tokens: None,
            output_tokens: None,
            wall_clock_seconds: None,
        }));
        assert_eq!(dims.calls, Some(1));
        assert_eq!(dims.input_tokens, DEFAULT_BUDGET.input_tokens);
        assert_eq!(dims.output_tokens, DEFAULT_BUDGET.output_tokens);
        assert_eq!(dims.wall_clock_seconds, DEFAULT_BUDGET.wall_clock_seconds);

        let all_default = ceiling_for(None);
        assert_eq!(all_default, DEFAULT_BUDGET);
    }

    /// The create projection seats the creator as an accepted participant — the
    /// organizer can contribute and challenge without a separate accept verb.
    #[test]
    fn create_projection_seats_the_creator() {
        let tenant: TenantId = "ten_00000000-0000-7000-8000-000000000001".parse().unwrap();
        let thread: ThreadId = "thr_00000000-0000-7000-8000-000000000001".parse().unwrap();
        let prepared = prepare_create(
            &tenant,
            &thread,
            "hpr_00000000-0000-7000-8000-000000000001",
            &CreateBody {
                tenant_id: tenant,
                subject: "subject".to_string(),
                objective: "objective".to_string(),
                budget: None,
            },
        );
        let projection: ThreadProjection =
            serde_json::from_value(prepared.next_state).expect("projection parses");
        assert_eq!(projection.state, ThreadState::Open);
        assert_eq!(
            projection.participants["hpr_00000000-0000-7000-8000-000000000001"],
            ParticipationState::Accepted
        );
        assert_eq!(projection.ceiling_id, format!("ceil_{thread}"));
        assert_eq!(
            prepared.ceiling.as_ref().map(|(id, _)| id.as_str()),
            Some(format!("ceil_{thread}").as_str())
        );
        assert_eq!(prepared.event_type, EVENT_CREATED);
    }
}
