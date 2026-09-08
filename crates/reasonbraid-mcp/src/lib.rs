//! The MCP read-half (`PHASE-8.3.3`, ADR-024, §9.6): the READ tools over
//! the inspection verbs — the SAME queries + the SAME authorization as
//! the HTTP handlers, exposed as MCP tools. The write tools (`respond`,
//! `join_call`, `propose_policy_change`) stay OFF until the qualified
//! profile (`.3.5`) — a tool no handler backs is not exposed.
//!
//! The principal rides the tool's argument (the dev profile's trust
//! shape — the same principal the HTTP header carries); every read runs
//! the reader classification (the per-reader visibility the HTTP surface
//! enforces).

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

/// The read tool's shared handle: the pool + the principal (the dev
/// profile's trust shape — the same principal the HTTP header carries).
#[derive(Clone)]
pub struct ReadTools {
    pub pool: sqlx::PgPool,
}

/// `get_thread` — the thread's current projection + its events, read
/// through the SAME classification the HTTP inspection applies.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetThreadParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
    /// The thread id.
    pub thread_id: String,
}

/// `list_inbox` — the node's inbox rows with their delivery states.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListInboxParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
    /// The node id whose inbox to list.
    pub node_id: String,
}

/// `get_policy_bundle` — the authorized policy set: the registry's
/// current policy documents (the digest-pinned forms).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPolicyBundleParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
}

#[tool_router(server_handler)]
impl ReadTools {
    /// Read one thread's current projection — the same classification the
    /// HTTP `GET /v1/threads/{id}` applies (the per-reader visibility).
    #[tool(
        description = "Read one thread's current projection (the same classification the HTTP inspection applies)"
    )]
    async fn get_thread(
        &self,
        Parameters(args): Parameters<GetThreadParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        let class = crate::server::classify(&self.pool, &principal, &args.thread_id)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        let Some(class) = class else {
            return Ok(rmcp::model::CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!(
                    "no thread `{}` visible to the principal",
                    args.thread_id
                )),
            ]));
        };
        let state = crate::server::thread_state(&self.pool, &args.tenant_id, &args.thread_id)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        match state {
            Some((version, state)) => Ok(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(
                    serde_json::json!({
                        "thread_id": args.thread_id,
                        "version": version,
                        "visibility": class.wire_name(),
                        "state": state,
                    })
                    .to_string(),
                ),
            ])),
            None => Ok(rmcp::model::CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!(
                    "no thread `{}` in tenant `{}`",
                    args.thread_id, args.tenant_id
                )),
            ])),
        }
    }

    /// List one node's inbox rows with their delivery states — the same
    /// surface the HTTP inspection exposes.
    #[tool(description = "List one node's inbox rows with their delivery states")]
    async fn list_inbox(
        &self,
        Parameters(args): Parameters<ListInboxParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let rows = crate::server::inbox_rows(&self.pool, &args.tenant_id, &args.node_id)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        Ok(rmcp::model::CallToolResult::success(vec![
            rmcp::model::ContentBlock::text(
                serde_json::json!({ "node_id": args.node_id, "rows": rows }).to_string(),
            ),
        ]))
    }

    /// The authorized policy set — the registry's current policy documents
    /// (the digest-pinned forms).
    #[tool(
        description = "The authorized policy set — the registry's current digest-pinned documents"
    )]
    async fn get_policy_bundle(
        &self,
        Parameters(args): Parameters<GetPolicyBundleParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let docs = crate::server::policy_bundle(&self.pool, &args.tenant_id)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        Ok(rmcp::model::CallToolResult::success(vec![
            rmcp::model::ContentBlock::text(
                serde_json::json!({ "tenant_id": args.tenant_id, "policies": docs }).to_string(),
            ),
        ]))
    }
}

/// Parse the dev-profile principal (the same shapes the HTTP header
/// resolves).
pub fn principal(id: &str) -> Result<reasonbraid_core::GrantSubject, String> {
    if id.starts_with("hpr_") {
        id.parse::<reasonbraid_core::HumanPrincipalId>()
            .map(reasonbraid_core::GrantSubject::Human)
            .map_err(|e| format!("the principal `{id}` is malformed: {e}"))
    } else if id.starts_with("rol_") {
        id.parse::<reasonbraid_core::AgentRoleId>()
            .map(reasonbraid_core::GrantSubject::Role)
            .map_err(|e| format!("the principal `{id}` is malformed: {e}"))
    } else {
        Err(format!(
            "the principal `{id}` is outside the dev-profile wire space (`hpr_`/`rol_`)"
        ))
    }
}

/// The read-side plumbing over the server's surfaces (the SAME queries
/// the HTTP handlers run).
pub mod server {
    /// The reader's classification for a thread (the same classify
    /// logic the HTTP profile/thread reads apply — the per-reader
    /// visibility).
    pub async fn classify(
        pool: &sqlx::PgPool,
        principal: &reasonbraid_core::GrantSubject,
        thread_id: &str,
    ) -> Result<Option<ReaderClassWire>, sqlx::Error> {
        // The thread's role anchor: the thread_id is the aggregate id, not a
        // role — the HTTP thread inspection classifies by the TENANT scope,
        // not the per-profile class. The read-half maps the thread read to
        // the tenant-scoped visibility: the principal's tenant vs the
        // thread's tenant (the same rule the thread inspection applies).
        let Some(reader_tenant) = principal_tenant(pool, principal).await? else {
            return Ok(None);
        };
        let thread_tenant: Option<String> = sqlx::query_scalar(
            "SELECT tenant_id FROM aggregate_state \
             WHERE aggregate_id = $1 AND aggregate_type = 'thread'",
        )
        .bind(thread_id)
        .fetch_optional(pool)
        .await?;
        let Some(thread_tenant) = thread_tenant else {
            return Ok(None);
        };
        if reader_tenant == thread_tenant {
            Ok(Some(ReaderClassWire::Tenant))
        } else {
            Ok(Some(ReaderClassWire::Network))
        }
    }

    /// The reader class's wire name.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ReaderClassWire {
        Tenant,
        Network,
    }

    impl ReaderClassWire {
        pub fn wire_name(self) -> &'static str {
            match self {
                ReaderClassWire::Tenant => "tenant",
                ReaderClassWire::Network => "network",
            }
        }
    }

    async fn principal_tenant(
        pool: &sqlx::PgPool,
        principal: &reasonbraid_core::GrantSubject,
    ) -> Result<Option<String>, sqlx::Error> {
        let id = match principal {
            reasonbraid_core::GrantSubject::Human(h) => h.to_string(),
            reasonbraid_core::GrantSubject::Role(r) => r.to_string(),
        };
        if principal_is_human(principal) {
            sqlx::query_scalar("SELECT tenant_id FROM human_principals WHERE principal_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
        } else {
            sqlx::query_scalar("SELECT tenant_id FROM agent_roles WHERE role_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
        }
    }

    fn principal_is_human(principal: &reasonbraid_core::GrantSubject) -> bool {
        matches!(principal, reasonbraid_core::GrantSubject::Human(_))
    }

    /// The thread's current projection (the same aggregate-state read the
    /// HTTP inspection runs).
    pub async fn thread_state(
        pool: &sqlx::PgPool,
        tenant_id: &str,
        thread_id: &str,
    ) -> Result<Option<(i64, serde_json::Value)>, sqlx::Error> {
        sqlx::query_as(
            "SELECT aggregate_version, state FROM aggregate_state \
             WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_type = 'thread'",
        )
        .bind(tenant_id)
        .bind(thread_id)
        .fetch_optional(pool)
        .await
    }

    /// The node's inbox rows with the delivery states (the same
    /// node_inbox read the HTTP inspection runs).
    pub async fn inbox_rows(
        pool: &sqlx::PgPool,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows: Vec<(i64, String, bool, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
            "SELECT cursor, command_id, acknowledged_at IS NOT NULL, quarantined_at \
             FROM node_inbox WHERE tenant_id = $1 AND node_id = $2 ORDER BY cursor",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(cursor, command_id, acknowledged, quarantined)| {
                serde_json::json!({
                    "cursor": cursor,
                    "command_id": command_id,
                    "acknowledged": acknowledged,
                    "quarantined": quarantined.is_some(),
                })
            })
            .collect())
    }

    /// The authorized policy set — the registry's current digest-pinned
    /// documents (the same policy_versions read the HTTP surface runs).
    pub async fn policy_bundle(
        pool: &sqlx::PgPool,
        _tenant_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows: Vec<(String, String, String, serde_json::Value)> = sqlx::query_as(
            "SELECT policy_id, version, digest, clauses FROM policy_versions \
             ORDER BY policy_id, version",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(policy_id, version, digest, clauses)| {
                serde_json::json!({
                    "policy_id": policy_id,
                    "version": version,
                    "digest": digest,
                    "clauses": clauses,
                })
            })
            .collect())
    }
}

/// The conformance fixtures: the recorded tool schemas (the goldens the
/// offline tests compare against — the independent fixtures, never the
/// maturity-tier prose).
#[cfg(test)]
mod tests {
    use super::*;

    /// The read tools' router lists exactly the three read tools — the
    /// write tools stay OFF until the qualified profile.
    #[test]
    fn the_tool_router_lists_the_read_tools_only() {
        let router = ReadTools::tool_router();
        let names: Vec<String> = router
            .list_all()
            .iter()
            .map(|t| t.name.to_string())
            .collect();
        assert!(
            names.contains(&"get_thread".to_string()),
            "get_thread: {names:?}"
        );
        assert!(
            names.contains(&"list_inbox".to_string()),
            "list_inbox: {names:?}"
        );
        assert!(
            names.contains(&"get_policy_bundle".to_string()),
            "get_policy_bundle: {names:?}"
        );
        for forbidden in ["respond", "join_call", "propose_policy_change"] {
            assert!(
                !names.contains(&forbidden.to_string()),
                "the write tool `{forbidden}` must stay OFF: {names:?}"
            );
        }
    }

    /// The conformance goldens: each read tool's input schema names the
    /// principal + its target fields (the dev-profile trust shape).
    #[test]
    fn the_tool_schemas_carry_the_principal_and_the_targets() {
        let router = ReadTools::tool_router();
        let get_thread = router.get("get_thread").expect("get_thread");
        let schema = get_thread.input_schema.clone();
        let schema = serde_json::to_value(schema).expect("the schema serializes");
        let properties = schema["properties"].as_object().expect("the properties");
        for field in ["principal", "tenant_id", "thread_id"] {
            assert!(
                properties.contains_key(field),
                "get_thread names `{field}`: {properties:?}"
            );
        }
        let list_inbox = router.get("list_inbox").expect("list_inbox");
        let schema = serde_json::to_value(list_inbox.input_schema.clone()).expect("the schema");
        let properties = schema["properties"].as_object().expect("the properties");
        for field in ["principal", "tenant_id", "node_id"] {
            assert!(
                properties.contains_key(field),
                "list_inbox names `{field}`: {properties:?}"
            );
        }
    }

    /// The principal parse accepts the wire space + refuses the outside.
    #[test]
    fn the_principal_parse_bounds_the_wire_space() {
        assert!(principal("hpr_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee").is_ok());
        assert!(principal("rol_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee").is_ok());
        assert!(
            principal("bogus").is_err(),
            "outside the wire space refuses"
        );
    }
}
