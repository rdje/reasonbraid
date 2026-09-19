//! The MCP write-half's qualified gate (`PHASE-8.3.5.1`, ADR-024 §9.6):
//! the three write tools (`respond`/`join_call`/`propose_policy_change`)
//! ride the SAME domain handlers the HTTP verbs run. The seam adds ONLY
//! the qualified-profile controls ADR-024 names — the ENROLLMENT binding
//! (the tool's principal must be the tenant's recorded principal) and the
//! PER-PRINCIPAL QUOTA (the `.1.3.2` `principal` scope, re-opened here:
//! the call-volume bound — the use/denial event commits in the gate's own
//! transaction, so the quota counts the ADMITTED CALLS, never the domain
//! effects). The seam never widens an authority path.
//!
//! ⛔ **What each handler brings is NOT uniform, and this header used to
//! say it was** (`SIGNOFF-REPAIR.6.1.2`). It claimed "the per-verb LOCAL
//! grants + the audit ride the handlers themselves" for all three. As
//! measured:
//!
//! - `respond` — TRUE. `run_thread_command` runs the idempotency, the
//!   `thread_contribute` grant and the audit record.
//! - `join_call` — no per-verb grant: the core checks the enrolled ROLE,
//!   the call's state and, for the PARTICIPATION kinds only, eligibility.
//!   `.6.1.2` added the tenant binding it was missing; there is still no
//!   `GrantAction` for answering a call and no audit row.
//! - `propose_policy_change` — NEITHER. `lifecycle::register_proposal`
//!   inserts the row after an enrolment check and writes no audit record,
//!   and the HTTP verb behind `POST /v1/policy-proposals` does exactly the
//!   same. The tool is not weaker than the HTTP surface; the sentence was
//!   simply describing a control neither of them has.
//!
//! ⭐ The tenant each handler receives is the gate's, not the caller's
//! claim: `gate` refuses unless the argument EQUALS the principal's
//! recorded tenant, so by the time a handler sees it the two cannot differ.

use reasonbraid_core::actor_handle_for_subject;
use reasonbraid_core::{GrantAction, GrantSubject, ResourceTarget, TenantId, ThreadId};
use sqlx::PgPool;

/// The qualified gate's refusal — the typed family + the message (the
/// `.3.5.2` tools surface these as the tool errors).
#[derive(Debug)]
pub struct WriteRefused {
    /// `enrollment` | `quota_unconfigured` | `quota_exceeded` |
    /// `handler:<code>` (the same handler refused, with its typed code).
    pub family: String,
    /// The human-readable message.
    pub message: String,
}

impl WriteRefused {
    fn enrollment(message: impl Into<String>) -> Self {
        Self {
            family: "enrollment".into(),
            message: message.into(),
        }
    }

    fn handler(e: crate::api::ControlApiError) -> Self {
        Self {
            family: format!("handler:{}", e.code),
            message: e.message,
        }
    }

    fn storage(what: &str, e: sqlx::Error) -> Self {
        Self {
            family: "enrollment".into(),
            message: format!("{what} failed: {e}"),
        }
    }
}

/// The qualified gate: the enrollment binding → the per-principal quota.
/// Every write tool runs this BEFORE its handler (the `.3.5.2` delegation
/// calls the gate first — the handlers are never reached unqualified).
pub async fn gate(
    pool: &PgPool,
    tenant_id: &str,
    principal: &GrantSubject,
) -> Result<(), WriteRefused> {
    // 1. The enrollment binding: the principal's recorded tenant must be the
    //    tool's tenant — the SAME `reader_tenant` check the HTTP handlers
    //    run (the dev-profile trust shape: the principal rides the tool's
    //    argument, exactly like the HTTP header).
    let recorded = crate::api::reader_tenant(pool, principal)
        .await
        .map_err(|e| WriteRefused::storage("the enrollment read", e))?;
    let Some(recorded) = recorded else {
        return Err(WriteRefused::enrollment(
            "the principal is not enrolled — the write gate refuses",
        ));
    };
    if recorded != tenant_id {
        return Err(WriteRefused::enrollment(format!(
            "the principal's tenant `{recorded}` does not match the call's tenant `{tenant_id}`"
        )));
    }

    // 2. The per-principal quota (the `.1.3.2` re-open): the fail-closed
    //    check + the recorded use/denial. A refusal at the ceiling COMMITS
    //    its denial row (a refusal is a recorded fact, never silent); the
    //    unconfigured scope records nothing (nothing was written).
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| WriteRefused::storage("the gate transaction begin", e))?;
    match crate::quota::check_in_tx(
        &mut *tx,
        tenant_id,
        crate::quota::SCOPE_PRINCIPAL,
        &principal.id_string(),
        chrono::Utc::now(),
    )
    .await
    {
        Ok(()) => {
            tx.commit()
                .await
                .map_err(|e| WriteRefused::storage("the gate transaction commit", e))?;
            Ok(())
        }
        Err(crate::quota::QuotaError::Exceeded {
            ceiling,
            window_seconds,
        }) => {
            // The denial row must survive the refusal — commit it.
            tx.commit()
                .await
                .map_err(|e| WriteRefused::storage("the denial commit", e))?;
            Err(WriteRefused {
                family: "quota_exceeded".into(),
                message: format!(
                    "the principal's write quota is exhausted: {ceiling} calls within \
                     {window_seconds}s — the denial is recorded"
                ),
            })
        }
        Err(crate::quota::QuotaError::Unconfigured { .. }) => Err(WriteRefused {
            family: "quota_unconfigured".into(),
            message: "no quota is configured for the principal — the write gate refuses \
                      fail-closed"
                .into(),
        }),
        Err(crate::quota::QuotaError::Storage(e)) => Err(WriteRefused::storage("the quota", e)),
    }
}

/// `respond` — the thread contribution: the gate → the SAME thread-command
/// pipeline (the OP_CONTRIBUTE over `run_thread_command`: the idempotency →
/// the `thread_contribute` grant → the domain → the audit). The tool's
/// payload is the ContributeBody WITHOUT the tenant (the tenant rides the
/// tool's argument — the seam injects it; the dev-profile trust shape).
///
/// 🔴 **The body is CHECKED before it is indexed (`SIGNOFF-REPAIR.6.1.3`).**
/// This function is `pub` through `crate::mcp_write_internal` and takes a
/// `serde_json::Value`, and it used to write `body["tenant_id"] = …` straight
/// into it. `IndexMut<&str>` for `Value` PANICS on a string, a number, a bool
/// or an array, so a caller that kept the signature's promise and handed it a
/// `Value` could unwind the task rather than be refused. ⚠️ Not reachable
/// through the MCP tool, which serializes a typed `ContributePayload` — but a
/// signature is a promise the body must keep (`SIGNOFF-REPAIR.4.2.7`), and the
/// guard belongs where the promise is made, not at one of its callers.
pub async fn respond(
    pool: &PgPool,
    tenant_id: &str,
    principal: &GrantSubject,
    thread_id: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, WriteRefused> {
    gate(pool, tenant_id, principal).await?;
    let tenant: TenantId = tenant_id.parse().map_err(|e| {
        WriteRefused::handler(crate::api::ControlApiError::invalid_command(format!(
            "tenant_id is malformed: {e}"
        )))
    })?;
    let thread: ThreadId = thread_id.parse().map_err(|e| {
        WriteRefused::handler(crate::api::ControlApiError::invalid_command(format!(
            "thread_id is malformed: {e}"
        )))
    })?;
    // ⛔ BEFORE the index, and it must stay a check rather than a conversion.
    // The body feeds `request_hash` three lines down, which derives the
    // idempotency KEY — so round-tripping it through a typed struct could change
    // the bytes, change the key, and make an old call contribute AGAIN instead
    // of replaying. That is the failure the comment below already guards against
    // for a different reason, and it is why this leaf refuses the shape rather
    // than retyping the body: `join_call` and `propose_policy_change` do parse
    // into their typed inputs, and they can, because neither feeds a hash.
    //
    // ⚠️ `null` is refused with the rest, deliberately. It was the one non-object
    // that did NOT panic — serde_json silently replaces a `Null` with an empty
    // object — so it reached the downstream parse and refused there naming the
    // missing fields rather than the real problem. Fabricating an object out of
    // a caller's `null` is not a service to anybody.
    let mut body = body;
    if !body.is_object() {
        return Err(WriteRefused::handler(
            crate::api::ControlApiError::invalid_command(
                "the contribution body must be a JSON object carrying the contribution's fields",
            ),
        ));
    }
    body["tenant_id"] = serde_json::json!(tenant_id);
    // The idempotency key is DETERMINISTIC over (thread, principal, body) —
    // the same call replays the original result (the tool is a replay
    // surface exactly like the HTTP envelope).
    // `None` twice: the MCP tool sets no delegate (`delegation: None` below), and
    // it binds no target because its KEY already does — `mcp_respond_{thread}_…`
    // fixes the thread, so the cross-target replay `SIGNOFF-REPAIR.3.4.6` closes
    // is structurally impossible here. ⛔ Binding it would also change the key,
    // since the key is derived from the hash: an old call repeated after the
    // upgrade would find no row and CONTRIBUTE AGAIN rather than replay. A
    // duplicate effect is a worse answer than a property this surface already has.
    let hash =
        crate::api::request_hash(crate::threads::OP_CONTRIBUTE, principal, &body, None, None);
    let key = format!("mcp_respond_{}_{hash}", thread);
    let authz = crate::authority::CommandAuthz {
        actor: actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::ThreadContribute,
        target: ResourceTarget::Thread {
            tenant_id: tenant,
            thread_id: thread,
        },
    };
    let (status, value) = crate::api::run_thread_command(
        pool,
        &tenant,
        principal,
        &authz,
        &key,
        &hash,
        crate::api::CommandTarget::Existing {
            thread_id: thread,
            operation: crate::threads::OP_CONTRIBUTE,
            body: &body,
        },
    )
    .await
    .map_err(WriteRefused::handler)?;
    Ok(serde_json::json!({ "status": status.as_u16(), "result": value }))
}

/// `join_call` — the call response: the gate → the SAME call-respond core
/// (the enrolled-ROLE check + the eligibility re-resolution + the response
/// record).
pub async fn join_call(
    pool: &PgPool,
    tenant_id: &str,
    principal: &GrantSubject,
    call_id: &str,
    response: serde_json::Value,
) -> Result<serde_json::Value, WriteRefused> {
    gate(pool, tenant_id, principal).await?;
    let response: crate::recruitment::RecruitmentResponse = serde_json::from_value(response)
        .map_err(|e| {
            WriteRefused::handler(crate::api::ControlApiError::invalid_command(format!(
                "the call response is malformed: {e}"
            )))
        })?;
    crate::api::respond_to_call_core(pool, principal, call_id, &response)
        .await
        .map_err(WriteRefused::handler)
}

/// `propose_policy_change` — the policy proposal: the gate → the SAME
/// lifecycle registration (the enrolled-principal gate + the proposal
/// record).
pub async fn propose_policy_change(
    pool: &PgPool,
    tenant_id: &str,
    principal: &GrantSubject,
    input: serde_json::Value,
) -> Result<serde_json::Value, WriteRefused> {
    gate(pool, tenant_id, principal).await?;
    let input: crate::lifecycle::ProposalInput = serde_json::from_value(input).map_err(|e| {
        WriteRefused::handler(crate::api::ControlApiError::invalid_command(format!(
            "the proposal input is malformed: {e}"
        )))
    })?;
    let row = crate::lifecycle::register_proposal(pool, tenant_id, &input)
        .await
        .map_err(|e| {
            WriteRefused::handler(crate::api::ControlApiError::invalid_command(e.to_string()))
        })?;
    serde_json::to_value(&row).map_err(|e| WriteRefused {
        family: "enrollment".into(),
        message: format!("the stored proposal failed to serialize: {e}"),
    })
}
