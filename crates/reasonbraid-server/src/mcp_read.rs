//! The MCP read-half's authorization seam (`PHASE-8.3.3`, ADR-024 §9.6;
//! repaired by `SIGNOFF-REPAIR.6.1.1`): the three read tools
//! (`get_thread`/`list_inbox`/`get_policy_bundle`) run the SAME queries and
//! the SAME authorization as the HTTP inspection handlers, because they now
//! CALL them — [`crate::api::authorize_inspection`],
//! [`crate::api::thread_inspection`], [`crate::api::inbox_inspection`] and
//! [`crate::policy::list`] are the single definition each surface shares.
//!
//! ⛔ This module exists because the sentence came first and the code never
//! did. `reasonbraid-mcp` opened by claiming "the SAME queries + the SAME
//! authorization as the HTTP handlers" and "every read runs the reader
//! classification" over a private re-implementation that ran neither: the
//! inbox tool declared a `principal` argument it never read, the bundle tool
//! took its tenant as an unused parameter, and the thread tool computed the
//! foreign-reader class and returned the full projection beside it. A claim
//! of sameness is only worth the call graph that enforces it — so the seam
//! is a call, never a copy.
//!
//! ⭐ The tenant binding follows `inspect_node_inbox`'s adjudicated shape
//! (`SIGNOFF-REPAIR.3.5.3`), not `inspect_call`'s. Where the target carries
//! the only tenant, `inspect_call` derives it from the target; here the
//! caller names a tenant, so the named tenant is what authority is checked
//! against AND what the select is bound to — the two identifiers cannot
//! disagree without the read returning nothing. Deriving instead would
//! answer a question the caller did not ask, and refusing on a mismatch
//! before authorizing would tell an unauthorized caller which tenant owns
//! the target.

use reasonbraid_core::{GrantSubject, ResourceTarget, TenantId, ThreadId};
use sqlx::PgPool;

/// The read seam's typed refusal — the family + the message (the `.3.5.2`
/// tools surface these as the tool errors).
///
/// The family is the HTTP surface's own error CODE (`unauthorized`,
/// `scope_hidden`, `invalid_command`, …). Unlike [`crate::mcp_write`]'s
/// `WriteRefused`, nothing is prefixed `handler:`: the read seam adds no
/// gate of its own, so every refusal already IS the handler's.
#[derive(Debug)]
pub struct ReadRefused {
    pub family: String,
    pub message: String,
}

impl ReadRefused {
    fn from_api(error: crate::api::ControlApiError) -> Self {
        Self {
            family: error.code.to_string(),
            message: error.message,
        }
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self {
            family: "invalid_command".into(),
            message: message.into(),
        }
    }

    fn storage(what: &str, error: sqlx::Error) -> Self {
        Self {
            family: "internal".into(),
            message: format!("{what} failed: {error}"),
        }
    }
}

/// `get_thread` — one thread's current projection, through the SAME
/// `thread_inspect` authorization and the SAME tenant-bound select the HTTP
/// `GET /v1/threads/{id}` runs, and therefore with the same derived view
/// (an invitation past its expiry reads `expired`, not `invited`).
pub async fn thread(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: &str,
    thread_id: &str,
) -> Result<serde_json::Value, ReadRefused> {
    let tenant: TenantId = tenant_id
        .parse()
        .map_err(|e| ReadRefused::invalid(format!("tenant_id `{tenant_id}` is malformed: {e}")))?;
    let thread: ThreadId = thread_id
        .parse()
        .map_err(|e| ReadRefused::invalid(format!("thread_id `{thread_id}` is malformed: {e}")))?;
    crate::api::authorize_inspection(
        pool,
        principal,
        ResourceTarget::Thread {
            tenant_id: tenant,
            thread_id: thread,
        },
    )
    .await
    .map_err(ReadRefused::from_api)?;
    crate::api::thread_inspection(pool, tenant, thread)
        .await
        .map_err(ReadRefused::from_api)
}

/// `list_inbox` — one node's inbox rows, through the SAME `tenant_admin`
/// authorization and the SAME `node_inbox_state` select the HTTP
/// `GET /v1/nodes/inbox` runs (so the delivery state and the quarantine
/// facts travel with the row, exactly as the operator surface reports them).
pub async fn inbox(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: &str,
    node_id: &str,
) -> Result<serde_json::Value, ReadRefused> {
    let tenant: TenantId = tenant_id
        .parse()
        .map_err(|e| ReadRefused::invalid(format!("tenant_id `{tenant_id}` is malformed: {e}")))?;
    crate::api::authorize_tenant_admin(pool, principal, tenant)
        .await
        .map_err(ReadRefused::from_api)?;
    let inspection = crate::api::inbox_inspection(pool, tenant, node_id)
        .await
        .map_err(ReadRefused::from_api)?;
    serde_json::to_value(&inspection)
        .map_err(|e| ReadRefused::invalid(format!("the inbox inspection failed to serialize: {e}")))
}

/// `get_policy_bundle` — the registered policy documents, through the SAME
/// enrolment gate and the SAME `policy::list` read the HTTP
/// `GET /v1/policies` runs.
///
/// ⛔ The registry is SITE-GLOBAL and the response says so by carrying no
/// tenant. `policy_versions` has no tenant column, no site filters on one,
/// and `PolicyVersionInput` cannot supply one — so the tenant the tool used
/// to take was neither a filter nor a fact about the rows, and labelling the
/// bundle with it told the caller these were THEIR tenant's policies. The
/// question of whether the registry SHOULD be tenant-scoped is a schema
/// decision, owned by `SIGNOFF-REPAIR.6.1.5`; it is not something a read
/// tool may imply by adding a label.
pub async fn policy_bundle(
    pool: &PgPool,
    principal: &GrantSubject,
) -> Result<serde_json::Value, ReadRefused> {
    let enrolled = crate::api::reader_tenant(pool, principal)
        .await
        .map_err(|e| ReadRefused::storage("the enrolment read", e))?;
    if enrolled.is_none() {
        return Err(ReadRefused {
            family: "unauthorized".into(),
            message: "an unenrolled principal reads no policies".into(),
        });
    }
    let policies = crate::policy::list(pool)
        .await
        .map_err(|e| ReadRefused::storage("the policy read", e))?;
    Ok(serde_json::json!({ "policies": policies }))
}
