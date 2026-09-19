//! The MCP tools (`PHASE-8.3.3` + `PHASE-8.3.5.2`, ADR-024, §9.6): the
//! READ tools over the inspection verbs, through
//! `reasonbraid_server::mcp_read_internal`, and the WRITE tools
//! (`respond`, `join_call`, `propose_policy_change`) over the `.3.5.1`
//! qualified gate, through `mcp_write_internal`: a tool no handler backs
//! is not exposed, and the remote MCP metadata never grants authority.
//!
//! The principal rides the tool's argument (the dev profile's trust
//! shape — the same principal the HTTP header carries). Every tool is a
//! thin translation: parse the principal, call the seam, render the
//! seam's typed refusal as a tool error. The tool payloads EXCLUDE the
//! token fields — the tokens never enter the thread content.
//!
//! ⛔ This header used to claim "the SAME queries + the SAME
//! authorization as the HTTP handlers" and "every read runs the reader
//! classification" over a private re-implementation that ran NEITHER:
//! `list_inbox` declared a `principal` it never read, `get_policy_bundle`
//! took its tenant as an unused parameter, and `get_thread` computed the
//! foreign-reader class and returned the full projection beside it
//! (`SIGNOFF-REPAIR.6.1.1`). Measured before the repair, a principal in
//! one tenant received another tenant's entire thread projection. The
//! sameness is now a CALL rather than a sentence — which is the only
//! form of it a reader can check.

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

/// `get_policy_bundle` — the site's registered policy documents (the
/// digest-pinned forms).
///
/// ⛔ No tenant: the registry is site-global, so a tenant argument here
/// could only be ignored or checked against something it does not scope.
/// The conformance golden moved with it, deliberately
/// (`SIGNOFF-REPAIR.6.1.1`).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPolicyBundleParams {
    /// The caller's principal id (the dev-profile trust header's value).
    pub principal: String,
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
    /// Read one thread's current projection — the SAME `thread_inspect`
    /// authorization and the SAME tenant-bound select the HTTP
    /// `GET /v1/threads/{id}` runs.
    #[tool(
        description = "Read one thread's current projection (the same authorization and query the HTTP inspection runs)"
    )]
    async fn get_thread(
        &self,
        Parameters(args): Parameters<GetThreadParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        Ok(render(
            reasonbraid_server::mcp_read_internal::thread(
                &self.pool,
                &principal,
                &args.tenant_id,
                &args.thread_id,
            )
            .await,
        ))
    }

    /// List one node's inbox rows with their delivery states — the SAME
    /// `tenant_admin` authorization and the SAME select the HTTP
    /// `GET /v1/nodes/inbox` runs.
    #[tool(description = "List one node's inbox rows with their delivery states")]
    async fn list_inbox(
        &self,
        Parameters(args): Parameters<ListInboxParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        Ok(render(
            reasonbraid_server::mcp_read_internal::inbox(
                &self.pool,
                &principal,
                &args.tenant_id,
                &args.node_id,
            )
            .await,
        ))
    }

    /// The registered policy set — the SAME enrolment gate and the SAME
    /// `policy::list` read the HTTP `GET /v1/policies` runs.
    ///
    /// ⛔ The registry is SITE-GLOBAL, and the response carries no tenant
    /// because no tenant scoped it: `policy_versions` has no tenant column
    /// and none of its sites filters by one. The tool used to take a tenant
    /// and label the bundle with it, which told the caller these were their
    /// tenant's policies. Whether the registry SHOULD be tenant-scoped is a
    /// schema decision, owned by `SIGNOFF-REPAIR.6.1.5`.
    #[tool(
        description = "The site's registered policy set — the registry's current digest-pinned documents"
    )]
    async fn get_policy_bundle(
        &self,
        Parameters(args): Parameters<GetPolicyBundleParams>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let principal = crate::principal(&args.principal)
            .map_err(|e| rmcp::ErrorData::invalid_params(e, None))?;
        Ok(render(
            reasonbraid_server::mcp_read_internal::policy_bundle(&self.pool, &principal).await,
        ))
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

/// Render a read seam's result as a tool result: the value on success, the
/// seam's typed `family: message` on refusal — the same shape the write
/// tools already use for `WriteRefused`.
fn render(
    result: Result<serde_json::Value, reasonbraid_server::mcp_read_internal::ReadRefused>,
) -> rmcp::model::CallToolResult {
    match result {
        Ok(value) => rmcp::model::CallToolResult::success(vec![rmcp::model::ContentBlock::text(
            value.to_string(),
        )]),
        Err(refused) => rmcp::model::CallToolResult::error(vec![rmcp::model::ContentBlock::text(
            format!("{}: {}", refused.family, refused.message),
        )]),
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

/// The conformance fixtures: the recorded tool schemas (the goldens the
/// offline tests compare against — the independent fixtures, never the
/// maturity-tier prose).
#[cfg(test)]
#[path = "../../reasonbraid-server/tests/support/mod.rs"]
mod pg_test_support;

#[cfg(test)]
#[path = "../../reasonbraid-server/tests/support/cleanup.rs"]
mod pg_cleanup;

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
    ///
    /// ⛔ `get_policy_bundle`'s golden MOVED at `SIGNOFF-REPAIR.6.1.1` and the
    /// move is asserted rather than relaxed: the tool no longer takes a
    /// tenant, because the registry is site-global and a tenant argument
    /// there could only be ignored. A golden that merely stopped requiring
    /// the field would pass again the day someone re-added it.
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
        let bundle = router.get("get_policy_bundle").expect("get_policy_bundle");
        let schema = serde_json::to_value(bundle.input_schema.clone()).expect("the schema");
        let properties = schema["properties"].as_object().expect("the properties");
        assert!(
            properties.contains_key("principal"),
            "get_policy_bundle names `principal`: {properties:?}"
        );
        assert!(
            !properties.contains_key("tenant_id"),
            "get_policy_bundle names NO tenant — the registry is site-global: {properties:?}"
        );
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

    // ── The live tool-path roundtrip (`.3.5.3`) ─────────────────────────

    /// The live pool + the purge (the DATABASE_URL gate — the guard runs
    /// this; the offline sweep skips it).
    async fn live_pool() -> Option<sqlx::PgPool> {
        let pool = crate::pg_test_support::pool().await?;
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("apply migrations");
        crate::pg_cleanup::delete_tables(
            &pool,
            &[
                "policy_outcomes",
                "policy_corrections",
                "policy_drift",
                "deployment_assignments",
                "deployment_targets",
                "policy_publications",
                "policy_projections",
                "policy_approvals",
                "policy_decisions",
                "policy_proposals",
                "policy_versions",
                "routing_resolutions",
                "evaluation_runs",
                "evaluation_corpora",
                "profile_versions",
                "agent_profiles",
                "outbox_delivery",
                "outbox",
                "node_events",
                "node_inbox",
                "budget_reservations",
                "budget_ceilings",
                "spend_breakers",
                "administrative_effects",
                "node_enrollment_tokens",
                "authorization_records",
                "authority_grants",
                "enrollments",
                "enrollment_boundaries",
                "node_enroll_audit",
                "node_keys",
                "node_certificates",
                "server_ca",
                "node_leases",
                "runs",
                "incarnations",
                "node_proof_nonces",
                "nodes",
                "hosts",
                "recruitment_panels",
                "recruitment_responses",
                "recruitment_offers",
                "recruitment_calls",
                "agent_roles",
                "human_principals",
                "evidence_citations",
                "claim_assessments",
                "derivations",
                "evidence_snapshots",
                "reference_registrations",
                "resource_references",
                "quota_events",
                "usage_quotas",
                "federation_agreements",
                "cross_domain_receipts",
                "mcp_listen_state",
                "tenant_bootstrap_requests",
                "routing_recommendations",
                "tenants",
                "idempotency",
                "event_log",
                "aggregate_state",
            ],
        )
        .await
        .expect("purge checked fixture plan");
        Some(pool)
    }

    struct LiveServer {
        addr: std::net::SocketAddr,
        _handle: tokio::task::JoinHandle<()>,
    }

    impl LiveServer {
        async fn start(pool: &sqlx::PgPool) -> Self {
            let router = reasonbraid_server::api_router(pool.clone());
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind the ephemeral port");
            let addr = listener.local_addr().unwrap();
            let handle = tokio::spawn(async move {
                axum::serve(listener, router).await.expect("serve");
            });
            Self {
                addr,
                _handle: handle,
            }
        }

        fn base(&self) -> String {
            format!("http://{}", self.addr)
        }
    }

    async fn enroll(
        client: &reqwest::Client,
        base: &str,
        body: serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let response = client
            .post(format!("{base}/v1/enrollments"))
            .json(&body)
            .send()
            .await
            .expect("the enroll request");
        let status = response.status().as_u16();
        (status, response.json().await.expect("the enroll json"))
    }

    async fn command(
        client: &reqwest::Client,
        base: &str,
        path: &str,
        principal_id: &str,
        envelope: &reasonbraid_core::CommandEnvelope,
    ) -> (u16, serde_json::Value) {
        let response = client
            .post(format!("{base}{path}"))
            .header(reasonbraid_server::PRINCIPAL_HEADER, principal_id)
            .json(envelope)
            .send()
            .await
            .expect("the command request");
        let status = response.status().as_u16();
        let text = response.text().await.expect("the command body");
        let body =
            serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));
        (status, body)
    }

    async fn post(
        client: &reqwest::Client,
        base: &str,
        path: &str,
        principal_id: &str,
        body: &serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let response = client
            .post(format!("{base}{path}"))
            .header(reasonbraid_server::PRINCIPAL_HEADER, principal_id)
            .json(body)
            .send()
            .await
            .expect("the post request");
        let status = response.status().as_u16();
        let text = response.text().await.expect("the post body");
        let body =
            serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));
        (status, body)
    }

    fn envelope(
        operation: &str,
        key: &str,
        body: serde_json::Value,
    ) -> reasonbraid_core::CommandEnvelope {
        reasonbraid_core::CommandEnvelope {
            protocol_version: reasonbraid_core::PROTOCOL_VERSION.to_string(),
            operation: operation.to_string(),
            request_id: reasonbraid_core::RequestId::new(),
            idempotency_key: key.to_string(),
            expected_aggregate_version: None,
            body,
            authority_context: None,
            client_context: reasonbraid_core::ClientContext::default(),
        }
    }

    fn text_of(result: &rmcp::model::CallToolResult) -> String {
        match &result.content[0] {
            rmcp::model::ContentBlock::Text(t) => t.text.clone(),
            other => panic!("the tool result is not text: {other:?}"),
        }
    }

    /// The live roundtrip: the granted `respond` through the TOOL handler
    /// lands the effect + the quota use; the ungranted + the unconfigured
    /// refusals surface as the typed tool errors; the `join_call` + the
    /// `propose_policy_change` ride the same handlers.
    #[tokio::test]
    async fn the_write_tools_roundtrip_the_qualified_gate_live() {
        let Some(pool) = live_pool().await else {
            return;
        };
        let server = LiveServer::start(&pool).await;
        let client = reqwest::Client::new();
        let base = server.base();

        let (status, human) = enroll(
            &client,
            &base,
            serde_json::json!({ "kind": "human", "name": "mcp-tool-human" }),
        )
        .await;
        assert_eq!(status, 200, "the human enrolls: {human}");
        let human_id = human["principal_id"].as_str().unwrap().to_string();
        let tenant = human["tenant_id"].as_str().unwrap().to_string();
        let (status, role) = enroll(
            &client,
            &base,
            serde_json::json!({
                "kind": "role",
                "name": "mcp-tool-writer",
                "tenant_id": tenant,
                "actions": ["thread_contribute", "thread_invitation_respond"],
            }),
        )
        .await;
        assert_eq!(status, 200, "the writer enrolls: {role}");
        let role_id = role["principal_id"].as_str().unwrap().to_string();

        let (status, created) = command(
            &client,
            &base,
            "/v1/threads",
            &human_id,
            &envelope(
                "thread.create",
                "k-create",
                serde_json::json!({
                    "tenant_id": tenant,
                    "subject": "the mcp tool roundtrip",
                    "objective": "the same handlers through the tools",
                }),
            ),
        )
        .await;
        assert_eq!(status, 200, "create: {created}");
        let thread_id = created["thread_id"].as_str().unwrap().to_string();
        let (status, invited) = command(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}/commands"),
            &human_id,
            &envelope(
                "thread.invite",
                "k-invite",
                serde_json::json!({ "tenant_id": tenant, "agent_role": role_id }),
            ),
        )
        .await;
        assert_eq!(status, 200, "invite: {invited}");
        let (status, accepted) = command(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}/commands"),
            &role_id,
            &envelope(
                "thread.accept_invitation",
                "k-accept",
                serde_json::json!({ "tenant_id": tenant }),
            ),
        )
        .await;
        assert_eq!(status, 200, "accept: {accepted}");

        // 1. The granted respond through the TOOL handler.
        let tools = McpTools { pool: pool.clone() };
        let result = tools
            .respond(Parameters(RespondParams {
                principal: role_id.clone(),
                tenant_id: tenant.clone(),
                thread_id: thread_id.clone(),
                payload: ContributePayload {
                    content: "the tool path lands".into(),
                    kind: "claim".into(),
                    evidence_refs: vec![],
                },
            }))
            .await
            .expect("the tool handler runs");
        assert_ne!(
            result.is_error,
            Some(true),
            "the granted write is not an error: {result:?}"
        );
        let text = text_of(&result);
        assert!(
            text.contains("\"status\":200"),
            "the respond reports the effect: {text}"
        );
        let uses: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM quota_events q \
             JOIN usage_quotas u ON u.quota_id = q.quota_id \
             WHERE u.tenant_id = $1 AND u.scope_kind = 'principal' AND u.scope_id = $2 \
             AND q.kind = 'use'",
        )
        .bind(&tenant)
        .bind(&role_id)
        .fetch_one(&pool)
        .await
        .expect("the use count");
        assert!(uses >= 1, "the tool-path write counted the quota use");

        // 2. The ungranted role → the handler's OWN authz refusal, surfaced
        //    as the typed tool error.
        let (status, ghost) = enroll(
            &client,
            &base,
            serde_json::json!({ "kind": "role", "name": "mcp-tool-ghost", "tenant_id": tenant, "actions": [] }),
        )
        .await;
        assert_eq!(status, 200, "the ghost enrolls: {ghost}");
        let ghost_id = ghost["principal_id"].as_str().unwrap().to_string();
        let result = tools
            .respond(Parameters(RespondParams {
                principal: ghost_id,
                tenant_id: tenant.clone(),
                thread_id: thread_id.clone(),
                payload: ContributePayload {
                    content: "the ungranted tool write".into(),
                    kind: "claim".into(),
                    evidence_refs: vec![],
                },
            }))
            .await
            .expect("the ungranted handler runs");
        assert_eq!(result.is_error, Some(true), "the ungranted is an error");
        assert!(
            text_of(&result).contains("handler:unauthorized"),
            "the ungranted surfaces the typed refusal: {}",
            text_of(&result)
        );

        // 3. The unconfigured quota → the fail-closed tool error.
        sqlx::query(
            "DELETE FROM quota_events USING usage_quotas \
             WHERE quota_events.quota_id = usage_quotas.quota_id \
             AND usage_quotas.tenant_id = $1 AND usage_quotas.scope_kind = 'principal' \
             AND usage_quotas.scope_id = $2",
        )
        .bind(&tenant)
        .bind(&role_id)
        .execute(&pool)
        .await
        .expect("clear the writer's events");
        sqlx::query(
            "DELETE FROM usage_quotas \
             WHERE tenant_id = $1 AND scope_kind = 'principal' AND scope_id = $2",
        )
        .bind(&tenant)
        .bind(&role_id)
        .execute(&pool)
        .await
        .expect("remove the writer's quota");
        let result = tools
            .respond(Parameters(RespondParams {
                principal: role_id.clone(),
                tenant_id: tenant.clone(),
                thread_id: thread_id.clone(),
                payload: ContributePayload {
                    content: "the unconfigured tool write".into(),
                    kind: "claim".into(),
                    evidence_refs: vec![],
                },
            }))
            .await
            .expect("the unconfigured handler runs");
        assert_eq!(result.is_error, Some(true), "the unconfigured is an error");
        assert!(
            text_of(&result).contains("quota_unconfigured"),
            "the unconfigured surfaces the fail-closed refusal: {}",
            text_of(&result)
        );

        // 4. The join_call + the propose_policy_change ride the same
        //    handlers through the tools. The writer's quota row returns
        //    first (the binding restored).
        sqlx::query(
            "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
             VALUES ($1, $2, 'principal', $3, 1000, 3600)",
        )
        .bind(format!("quo_{role_id}_writes"))
        .bind(&tenant)
        .bind(&role_id)
        .execute(&pool)
        .await
        .expect("restore the writer's quota");

        let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
        let (status, opened) = post(
            &client,
            &base,
            "/v1/calls",
            &human_id,
            &serde_json::json!({
                "tenant_id": tenant,
                "thread_id": thread_id,
                "expression": { "scope": "tenant", "capabilities": [], "presence_states": ["available"] },
                "min_participants": 1,
                "max_participants": 2,
                "join_deadline": deadline,
                "expires_at": expiry,
            }),
        )
        .await;
        assert_eq!(status, 200, "the call opens: {opened}");
        let call_id = opened["call_id"].as_str().unwrap().to_string();
        let result = tools
            .join_call(Parameters(JoinCallParams {
                principal: role_id.clone(),
                tenant_id: tenant.clone(),
                call_id,
                kind: "decline".into(),
                reason: Some("the tool declines".into()),
            }))
            .await
            .expect("the join_call handler runs");
        assert_ne!(result.is_error, Some(true), "the decline is not an error");
        assert!(
            text_of(&result).contains("\"decline\""),
            "the decline rides the handler: {}",
            text_of(&result)
        );

        let grant_id = format!("grt_{human_id}");
        let (status, registered) = post(
            &client,
            &base,
            "/v1/policies",
            &human_id,
            &serde_json::json!({
                "policy_id": "mcp-tool-pol",
                "version": "1.0.0",
                "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "lifecycle": "draft",
                "title": "the mcp tool policy",
                "intent": "the proposal surface",
                "domain": "deliberation",
                "risk_class": "low",
                "owning_authority": grant_id,
                "clauses": [ { "id": "c1", "statement": "every write rides a local grant" } ],
                "applicability": [ { "layer": "organization", "target": "*" } ],
                "exceptions": [],
            }),
        )
        .await;
        assert_eq!(status, 200, "the policy registers: {registered}");
        let result = tools
            .propose_policy_change(Parameters(ProposePolicyChangeParams {
                principal: role_id,
                tenant_id: tenant,
                proposal_id: "prp_mcp_tool_1".into(),
                policy_id: "mcp-tool-pol".into(),
                policy_version: "1.0.0".into(),
                thread_id,
            }))
            .await
            .expect("the propose handler runs");
        assert_ne!(result.is_error, Some(true), "the proposal is not an error");
        assert!(
            text_of(&result).contains("prp_mcp_tool_1"),
            "the proposal rides the handler: {}",
            text_of(&result)
        );
    }

    // ── The entitlement control (`SIGNOFF-REPAIR.6.1.1`) ─────────────────

    /// Every read leg records its own verdict and the run asserts at the end,
    /// rather than stopping at the first breach. A fail-fast control over six
    /// legs proves one of them RED and says nothing about the other five,
    /// which is the wrong shape when the claim under test is *"all three read
    /// tools"*.
    fn record(breaches: &mut Vec<String>, leg: &str, held: bool, evidence: &str) {
        if !held {
            breaches.push(format!("{leg}: {evidence}"));
        }
    }

    /// The three READ tools must refuse a caller who is not entitled to the
    /// target, and must keep serving one who is.
    ///
    /// ⛔ The policy leg is NOT a tenant leg, and the finding that opened this
    /// leaf read it as one. `policy_versions` carries no tenant column and
    /// none of its six SQL sites filters by one — the registry is SITE-GLOBAL
    /// by construction, the same set the HTTP `GET /v1/policies` returns to
    /// any enrolled caller. What the tool owed that surface and never paid is
    /// its ENROLMENT gate; what it additionally got wrong is labelling a
    /// site-global bundle with the caller's own tenant.
    ///
    /// ⭐ The thread leg runs TWICE from the same principal — once naming the
    /// foreign tenant, once naming her own — because a repair that only
    /// compares the caller's tenant to the ARGUMENT passes the first and
    /// fails the second. The two identifiers must be BOUND.
    #[tokio::test]
    async fn the_read_tools_refuse_the_unentitled_caller_live() {
        let Some(pool) = live_pool().await else {
            return;
        };
        let server = LiveServer::start(&pool).await;
        let client = reqwest::Client::new();
        let base = server.base();

        // Two tenants, each with its own bootstrap human (the dev admin set —
        // `thread_inspect` + `tenant_admin`, scoped to that tenant).
        let (status, alice) = enroll(
            &client,
            &base,
            serde_json::json!({ "kind": "human", "name": "mcp-read-alice" }),
        )
        .await;
        assert_eq!(status, 200, "alice enrolls: {alice}");
        let alice_id = alice["principal_id"].as_str().unwrap().to_string();
        let tenant_a = alice["tenant_id"].as_str().unwrap().to_string();

        let (status, bob) = enroll(
            &client,
            &base,
            serde_json::json!({ "kind": "human", "name": "mcp-read-bob" }),
        )
        .await;
        assert_eq!(status, 200, "bob enrolls: {bob}");
        let bob_id = bob["principal_id"].as_str().unwrap().to_string();
        let tenant_b = bob["tenant_id"].as_str().unwrap().to_string();
        assert_ne!(
            tenant_a, tenant_b,
            "the two enrolments mint distinct tenants"
        );

        // Bob's thread, in tenant B, with a subject a leak would reveal.
        let (status, created) = command(
            &client,
            &base,
            "/v1/threads",
            &bob_id,
            &envelope(
                "thread.create",
                "k-read-control-create",
                serde_json::json!({
                    "tenant_id": tenant_b,
                    "subject": "bobs-unshared-subject",
                    "objective": "the foreign-read control",
                }),
            ),
        )
        .await;
        assert_eq!(status, 200, "bob creates his thread: {created}");
        let bob_thread = created["thread_id"].as_str().unwrap().to_string();

        // Bob's inbox row, in tenant B, with a command id a leak would reveal.
        sqlx::query(
            "INSERT INTO node_inbox (node_id, cursor, command_id, tenant_id, thread_id, payload) \
             VALUES ($1, 1, $2, $3, $4, $5)",
        )
        .bind("nod-mcp-read-bob")
        .bind("cmd-bobs-unshared-command")
        .bind(&tenant_b)
        .bind(&bob_thread)
        .bind(serde_json::json!({ "kind": "control" }))
        .execute(&pool)
        .await
        .expect("seed bob's inbox row");

        let tools = McpTools { pool: pool.clone() };
        let mut breaches: Vec<String> = Vec::new();

        // Leg A — the thread, from a principal entitled to neither the thread
        // nor (in the first shape) the tenant she names.
        for (named_tenant, shape) in [
            (tenant_b.clone(), "naming the foreign tenant"),
            (tenant_a.clone(), "naming her own tenant"),
        ] {
            let result = tools
                .get_thread(Parameters(GetThreadParams {
                    principal: alice_id.clone(),
                    tenant_id: named_tenant,
                    thread_id: bob_thread.clone(),
                }))
                .await
                .expect("the thread tool runs");
            let text = text_of(&result);
            record(
                &mut breaches,
                &format!("A/{shape}: refuses"),
                result.is_error == Some(true),
                &text,
            );
            record(
                &mut breaches,
                &format!("A/{shape}: no subject"),
                !text.contains("bobs-unshared-subject"),
                &text,
            );
            record(
                &mut breaches,
                &format!("A/{shape}: no state field"),
                !text.contains("\"state\""),
                &text,
            );
        }

        // Leg B — the inbox: alice receives none of bob's rows, and the
        // refusal is TYPED rather than an empty list.
        let result = tools
            .list_inbox(Parameters(ListInboxParams {
                principal: alice_id.clone(),
                tenant_id: tenant_b.clone(),
                node_id: "nod-mcp-read-bob".into(),
            }))
            .await
            .expect("the inbox tool runs");
        let text = text_of(&result);
        record(
            &mut breaches,
            "B: refuses",
            result.is_error == Some(true),
            &text,
        );
        record(
            &mut breaches,
            "B: no command id",
            !text.contains("cmd-bobs-unshared-command"),
            &text,
        );

        // Leg C — the policy bundle: the ENROLMENT gate the HTTP surface
        // applies, on both shapes the tool never checked.
        //
        // ⛔ The leg asserts that NO BUNDLE came back, not that a particular
        // channel carried the refusal. A tool has two: an `Err(ErrorData)` at
        // the protocol level and a `CallToolResult::error` in the result. An
        // unparseable argument belongs in the first — which is where the
        // write tools already put it — and an authorization refusal in the
        // second. A control keyed on one channel would have called the
        // repaired malformed-principal case a failure while it was refusing
        // correctly.
        for (principal, shape) in [
            ("bogus", "C/malformed principal"),
            (
                "hpr_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee",
                "C/unenrolled principal",
            ),
        ] {
            let (refused, evidence) = match tools
                .get_policy_bundle(Parameters(GetPolicyBundleParams {
                    principal: principal.into(),
                }))
                .await
            {
                Err(protocol) => (true, format!("protocol refusal: {protocol}")),
                Ok(result) => (result.is_error == Some(true), text_of(&result)),
            };
            record(
                &mut breaches,
                &format!("{shape}: no bundle returned"),
                refused,
                &evidence,
            );
        }

        // Leg D — the positive half, so the repair cannot be a blanket
        // refusal: each principal still reads their OWN tenant, and the
        // bundle an enrolled caller receives carries no tenant label,
        // because no tenant scoped it.
        let result = tools
            .get_thread(Parameters(GetThreadParams {
                principal: bob_id.clone(),
                tenant_id: tenant_b.clone(),
                thread_id: bob_thread.clone(),
            }))
            .await
            .expect("the thread tool runs");
        let text = text_of(&result);
        record(
            &mut breaches,
            "D: bob reads his own thread",
            result.is_error != Some(true) && text.contains("bobs-unshared-subject"),
            &text,
        );

        let result = tools
            .list_inbox(Parameters(ListInboxParams {
                principal: bob_id.clone(),
                tenant_id: tenant_b.clone(),
                node_id: "nod-mcp-read-bob".into(),
            }))
            .await
            .expect("the inbox tool runs");
        let text = text_of(&result);
        record(
            &mut breaches,
            "D: bob reads his own tenant's inbox",
            result.is_error != Some(true) && text.contains("cmd-bobs-unshared-command"),
            &text,
        );

        // BOB registers a policy; ALICE, in the other tenant, must see it.
        // ⛔ That is not a leak this leaf failed to close: `policy_versions`
        // has no tenant column, and the HTTP `GET /v1/policies` returns the
        // same row to the same caller. Asserting it here makes the
        // site-global property OBSERVABLE, so `SIGNOFF-REPAIR.6.1.5` inherits
        // a demonstration rather than a description.
        let (status, registered) = post(
            &client,
            &base,
            "/v1/policies",
            &bob_id,
            &serde_json::json!({
                "policy_id": "mcp-read-site-pol",
                "version": "1.0.0",
                "digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "lifecycle": "draft",
                "title": "the site registry control",
                "intent": "the site-global read",
                "domain": "deliberation",
                "risk_class": "low",
                "owning_authority": format!("grt_{bob_id}"),
                "clauses": [ { "id": "c1", "statement": "the registry is site-wide" } ],
                "applicability": [ { "layer": "organization", "target": "*" } ],
                "exceptions": [],
            }),
        )
        .await;
        assert_eq!(status, 200, "bob registers a policy: {registered}");

        let result = tools
            .get_policy_bundle(Parameters(GetPolicyBundleParams {
                principal: alice_id.clone(),
            }))
            .await
            .expect("the bundle tool runs");
        let text = text_of(&result);
        record(
            &mut breaches,
            "D: an enrolled principal reads the site registry",
            result.is_error != Some(true) && text.contains("mcp-read-site-pol"),
            &text,
        );
        record(
            &mut breaches,
            "D: the site-global bundle carries no tenant label",
            !text.contains("tenant_id"),
            &text,
        );

        assert!(
            breaches.is_empty(),
            "{} of the entitlement legs breach:\n{}",
            breaches.len(),
            breaches.join("\n")
        );
    }
}
