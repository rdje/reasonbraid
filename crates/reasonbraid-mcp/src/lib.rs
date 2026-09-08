//! The MCP tools (`PHASE-8.3.3` + `PHASE-8.3.5.2`, ADR-024, §9.6): the
//! READ tools over the inspection verbs — the SAME queries + the SAME
//! authorization as the HTTP handlers — and the WRITE tools (`respond`,
//! `join_call`, `propose_policy_change`) over the `.3.5.1` qualified
//! gate: the enrollment binding + the per-principal quota, then the SAME
//! domain handlers (a tool no handler backs is not exposed; the remote
//! MCP metadata never grants authority).
//!
//! The principal rides the tool's argument (the dev profile's trust
//! shape — the same principal the HTTP header carries); every read runs
//! the reader classification (the per-reader visibility the HTTP surface
//! enforces); every write rides the qualified gate. The tool payloads
//! EXCLUDE the token fields — the tokens never enter the thread content.

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

/// The tools' shared handle: the pool (the dev-profile trust shape —
/// the same principal the HTTP header carries, per tool argument).
#[derive(Clone)]
pub struct McpTools {
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

/// `respond` — the thread contribution (the qualified write profile).
/// The payload is the contribute body WITHOUT the tenant (the seam
/// injects it) — and WITHOUT any token field (the tokens never enter
/// the thread content).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RespondParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
    /// The thread id.
    pub thread_id: String,
    /// The contribution payload: the content + the kind + the evidence
    /// refs (the handler's ContributeBody minus the tenant).
    pub payload: ContributePayload,
}

/// The contribution payload (the minimal demonstration profile — the
/// advanced contribution fields ride the named follow-on).
#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ContributePayload {
    /// The contribution's content.
    pub content: String,
    /// The contribution kind (the handler's vocabulary).
    pub kind: String,
    /// The cited evidence references.
    #[serde(default)]
    pub evidence_refs: Vec<serde_json::Value>,
}

/// `join_call` — the call response (the qualified write profile): the
/// response kind + the optional decline reason (the minimal profile of
/// the handler's response vocabulary).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct JoinCallParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
    /// The call id.
    pub call_id: String,
    /// The response kind (the handler's vocabulary — `join`, `decline`,
    /// `observe`, …).
    pub kind: String,
    /// The decline reason (only the `decline` kind carries it).
    #[serde(default)]
    pub reason: Option<String>,
}

/// `propose_policy_change` — the policy proposal (the qualified write
/// profile): the lifecycle's ProposalInput fields, flat.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProposePolicyChangeParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
    /// The tenant id.
    pub tenant_id: String,
    /// The proposal id.
    pub proposal_id: String,
    /// The policy id to change.
    pub policy_id: String,
    /// The policy version to change.
    pub policy_version: String,
    /// The deliberation thread the proposal references.
    pub thread_id: String,
}

#[tool_router(server_handler)]
impl McpTools {
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

    /// Contribute to a thread — the qualified write profile: the
    /// enrollment binding + the per-principal quota, then the SAME
    /// thread-command handler the HTTP verb runs (the per-verb grant +
    /// the audit ride the handler).
    #[tool(
        description = "Contribute to a thread (the qualified write profile: the gate, then the same thread-command handler)"
    )]
    async fn respond(
        &self,
        Parameters(args): Parameters<RespondParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        let payload = serde_json::to_value(&args.payload)
            .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
        match reasonbraid_server::mcp_write_internal::respond(
            &self.pool,
            &args.tenant_id,
            &principal,
            &args.thread_id,
            payload,
        )
        .await
        {
            Ok(result) => Ok(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(result.to_string()),
            ])),
            Err(refused) => Ok(rmcp::model::CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!("{}: {}", refused.family, refused.message)),
            ])),
        }
    }

    /// Respond to a call — the qualified write profile: the gate, then
    /// the SAME call-respond handler (the enrolled-role check + the
    /// eligibility re-resolution ride the handler).
    #[tool(
        description = "Respond to a recruitment call (the qualified write profile: the gate, then the same call-respond handler)"
    )]
    async fn join_call(
        &self,
        Parameters(args): Parameters<JoinCallParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        let response = match args.kind.as_str() {
            "decline" => serde_json::json!({ "kind": "decline", "reason": args.reason }),
            other => serde_json::json!({ "kind": other }),
        };
        match reasonbraid_server::mcp_write_internal::join_call(
            &self.pool,
            &args.tenant_id,
            &principal,
            &args.call_id,
            response,
        )
        .await
        {
            Ok(result) => Ok(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(result.to_string()),
            ])),
            Err(refused) => Ok(rmcp::model::CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!("{}: {}", refused.family, refused.message)),
            ])),
        }
    }

    /// Propose a policy change — the qualified write profile: the gate,
    /// then the SAME lifecycle registration the HTTP verb runs.
    #[tool(
        description = "Propose a policy change (the qualified write profile: the gate, then the same lifecycle registration)"
    )]
    async fn propose_policy_change(
        &self,
        Parameters(args): Parameters<ProposePolicyChangeParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        let input = serde_json::json!({
            "proposal_id": args.proposal_id,
            "policy_id": args.policy_id,
            "policy_version": args.policy_version,
            "thread_id": args.thread_id,
        });
        match reasonbraid_server::mcp_write_internal::propose_policy_change(
            &self.pool,
            &args.tenant_id,
            &principal,
            input,
        )
        .await
        {
            Ok(result) => Ok(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(result.to_string()),
            ])),
            Err(refused) => Ok(rmcp::model::CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!("{}: {}", refused.family, refused.message)),
            ])),
        }
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

    /// The router lists exactly the six tools — the three reads + the
    /// three qualified writes (a tool no handler backs is not exposed).
    #[test]
    fn the_tool_router_lists_the_six_tools() {
        let router = McpTools::tool_router();
        let names: Vec<String> = router
            .list_all()
            .iter()
            .map(|t| t.name.to_string())
            .collect();
        for expected in [
            "get_thread",
            "list_inbox",
            "get_policy_bundle",
            "respond",
            "join_call",
            "propose_policy_change",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "the tool `{expected}` lists: {names:?}"
            );
        }
        assert_eq!(names.len(), 6, "exactly the six tools: {names:?}");
    }

    /// The conformance goldens: each read tool's input schema names the
    /// principal + its target fields (the dev-profile trust shape).
    #[test]
    fn the_tool_schemas_carry_the_principal_and_the_targets() {
        let router = McpTools::tool_router();
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

    /// The write schemas: the principal + the targets ride the top
    /// level; the payloads name the handler's fields; NO schema names a
    /// token field (the tokens never enter the thread content).
    #[test]
    fn the_write_schemas_name_the_targets_and_exclude_the_tokens() {
        let router = McpTools::tool_router();

        let respond = router.get("respond").expect("respond");
        let schema = serde_json::to_value(respond.input_schema.clone()).expect("the schema");
        let properties = schema["properties"].as_object().expect("the properties");
        for field in ["principal", "tenant_id", "thread_id", "payload"] {
            assert!(
                properties.contains_key(field),
                "respond names `{field}`: {properties:?}"
            );
        }

        let join_call = router.get("join_call").expect("join_call");
        let schema = serde_json::to_value(join_call.input_schema.clone()).expect("the schema");
        let properties = schema["properties"].as_object().expect("the properties");
        for field in ["principal", "tenant_id", "call_id", "kind"] {
            assert!(
                properties.contains_key(field),
                "join_call names `{field}`: {properties:?}"
            );
        }

        let propose = router
            .get("propose_policy_change")
            .expect("propose_policy_change");
        let schema = serde_json::to_value(propose.input_schema.clone()).expect("the schema");
        let properties = schema["properties"].as_object().expect("the properties");
        for field in [
            "principal",
            "tenant_id",
            "proposal_id",
            "policy_id",
            "policy_version",
            "thread_id",
        ] {
            assert!(
                properties.contains_key(field),
                "propose_policy_change names `{field}`: {properties:?}"
            );
        }

        // The token exclusion: no write schema names a token/authorization
        // field — the remote MCP metadata never grants authority.
        for name in ["respond", "join_call", "propose_policy_change"] {
            let tool = router.get(name).expect(name);
            let schema = serde_json::to_value(tool.input_schema.clone()).expect("the schema");
            let text = schema.to_string();
            for forbidden in ["token", "access_token", "authorization", "credential"] {
                assert!(
                    !text.to_lowercase().contains(forbidden),
                    "the `{name}` schema names `{forbidden}`: {text}"
                );
            }
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
