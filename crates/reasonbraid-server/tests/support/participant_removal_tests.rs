use super::*;
use reasonbraid_core::{AuthorityContext, TargetSelector};

struct RemovalFixture {
    pool: PgPool,
    server: TestServer,
    client: reqwest::Client,
    tenant: String,
    human: String,
    role: String,
    thread: String,
}

impl RemovalFixture {
    async fn start() -> Option<Self> {
        let pool = pool().await?;
        let server = TestServer::start(&pool).await;
        let client = reqwest::Client::new();
        let (tenant, human, role, thread) = bootstrap(&client, &server.base(), "removal").await;
        let (status, body) = invite(
            &client,
            &server.base(),
            &InviteSpec {
                thread: &thread,
                human: &human,
                tenant: &tenant,
                role: &role,
                key: "removal-invite",
                ttl: None,
            },
        )
        .await;
        assert_eq!(status, 200, "invite before removal: {body}");
        Some(Self {
            pool,
            server,
            client,
            tenant,
            human,
            role,
            thread,
        })
    }

    fn envelope(&self, key: &str) -> CommandEnvelope {
        envelope(
            "thread.remove_participant",
            key,
            json!({ "tenant_id": self.tenant, "participant": self.role }),
        )
    }

    async fn audit_ids(&self) -> Vec<String> {
        sqlx::query_scalar("SELECT record_id FROM authorization_records ORDER BY record_id")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn new_audits(&self, before: &[String]) -> Vec<Value> {
        sqlx::query_scalar(
            "SELECT to_jsonb(r) FROM authorization_records r
             WHERE NOT (record_id = ANY($1)) ORDER BY record_id",
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    // Exclude the explicitly committed admission/rejection records. Compare all
    // domain rows, including both tenants in the cross-tenant controls.
    async fn domain_snapshot(&self) -> Vec<Value> {
        let mut snapshot = Vec::new();
        for table in [
            "aggregate_state",
            "event_log",
            "outbox",
            "outbox_delivery",
            "node_inbox",
            "budget_reservations",
            "quota_events",
        ] {
            let rows: Value = sqlx::query_scalar(&format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(r) ORDER BY to_jsonb(r)::text), '[]'::jsonb)
                 FROM public.\"{table}\" r"
            ))
            .fetch_one(&self.pool)
            .await
            .unwrap();
            snapshot.push(rows);
        }
        snapshot
    }

    async fn refused(
        &self,
        principal: &str,
        thread: &str,
        request: &CommandEnvelope,
        expected_status: u16,
        audit_tenant: Option<&str>,
    ) -> Value {
        let before = self.domain_snapshot().await;
        let ids = self.audit_ids().await;
        let (status, body) = command(
            &self.client,
            &self.server.base(),
            &format!("/v1/threads/{thread}/commands"),
            principal,
            request,
        )
        .await;
        assert_eq!(status, expected_status, "removal refusal: {body}");
        assert_eq!(
            self.domain_snapshot().await,
            before,
            "refusal changed domain rows"
        );
        let audits = self.new_audits(&ids).await;
        if let Some(tenant) = audit_tenant {
            assert_eq!(audits.len(), 1, "one committed denial: {audits:?}");
            assert_audit(&audits[0], tenant, "denied");
        } else {
            assert!(
                audits.is_empty(),
                "request must add no admission: {audits:?}"
            );
        }
        body
    }

    async fn remove_successfully(&self) {
        let ids = self.audit_ids().await;
        let (status, body) = command(
            &self.client,
            &self.server.base(),
            &format!("/v1/threads/{}/commands", self.thread),
            &self.human,
            &self.envelope("removal-valid"),
        )
        .await;
        assert_eq!(status, 200, "valid tenant administrator removes: {body}");
        assert_eq!(body["event_type"], json!("thread.participant_removed"));
        let audits = self.new_audits(&ids).await;
        assert_eq!(audits.len(), 1, "one committed allowance: {audits:?}");
        assert_audit(&audits[0], &self.tenant, "allowed");
        let grant: String = sqlx::query_scalar(
            "SELECT grant_id FROM authority_grants WHERE tenant_id = $1 AND subject_id = $2",
        )
        .bind(&self.tenant)
        .bind(&self.human)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        assert_eq!(audits[0]["grant_id"], json!(grant));
        let state: Value = sqlx::query_scalar(
            "SELECT state FROM aggregate_state WHERE tenant_id = $1 AND aggregate_id = $2",
        )
        .bind(&self.tenant)
        .bind(&self.thread)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        assert_eq!(state["participants"][&self.role], json!("revoked"));
    }
}

fn assert_audit(audit: &Value, tenant: &str, decision: &str) {
    assert_eq!(audit["tenant_id"], json!(tenant));
    assert_eq!(audit["action"], json!("tenant_admin"));
    assert_eq!(audit["target_kind"], json!("tenant"));
    assert_eq!(audit["target_tenant"], json!(tenant));
    assert_eq!(audit["target_thread"], Value::Null);
    assert_eq!(audit["decision"], json!(decision));
    assert_eq!(audit["evaluation"], json!({ "kind": "boundary_checked" }));
}

#[tokio::test]
async fn removal_requires_live_tenant_wide_administration() {
    let _guard = guard().await;
    let Some(fixture) = RemovalFixture::start().await else {
        return;
    };
    let (grant, boundary, actions, selector, valid_from, expires_at): (
        String,
        String,
        Value,
        Value,
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
    ) = sqlx::query_as(
        "SELECT grant_id, boundary_id, actions, selector, valid_from, expires_at
         FROM authority_grants WHERE tenant_id = $1 AND subject_id = $2",
    )
    .bind(&fixture.tenant)
    .bind(&fixture.human)
    .fetch_one(&fixture.pool)
    .await
    .unwrap();

    for case in [
        "thread_selector",
        "missing_action",
        "revoked_grant",
        "expired_grant",
        "future_grant",
        "suspended_boundary",
        "revoked_boundary",
        "valid",
    ] {
        let now: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
        let expired_at = valid_from + chrono::Duration::microseconds(1);
        assert!(expired_at < now && now + chrono::Duration::hours(1) < expires_at);
        sqlx::query(
            "UPDATE authority_grants SET actions=$1, selector=$2, status=$3, valid_from=$4, expires_at=$5
             WHERE grant_id=$6",
        )
        .bind(if case == "missing_action" { json!(["thread_inspect"]) } else { actions.clone() })
        .bind(if case == "thread_selector" {
            json!({ "kind": "threads", "threads": [fixture.thread] })
        } else { selector.clone() })
        .bind(if case == "revoked_grant" { "revoked" } else { "active" })
        .bind(if case == "future_grant" { now + chrono::Duration::hours(1) } else { valid_from })
        .bind(if case == "expired_grant" { expired_at } else { expires_at })
        .bind(&grant)
        .execute(&fixture.pool).await.unwrap();
        sqlx::query("UPDATE enrollment_boundaries SET status=$1 WHERE boundary_id=$2")
            .bind(match case {
                "suspended_boundary" => "suspended",
                "revoked_boundary" => "revoked",
                _ => "active",
            })
            .bind(&boundary)
            .execute(&fixture.pool)
            .await
            .unwrap();
        if case != "valid" {
            fixture
                .refused(
                    &fixture.human,
                    &fixture.thread,
                    &fixture.envelope(case),
                    403,
                    Some(&fixture.tenant),
                )
                .await;
        }
    }
    // The restored authority applies to new intent, not an already committed
    // denial. A replay keeps the original result and creates no new admission.
    let replay = fixture
        .refused(
            &fixture.human,
            &fixture.thread,
            &fixture.envelope("thread_selector"),
            403,
            None,
        )
        .await;
    assert_eq!(replay["replayed"], json!(true));
    assert_eq!(replay["error"]["code"], json!("unauthorized"));
    fixture.remove_successfully().await;
}

#[tokio::test]
async fn removal_keeps_tenant_binding_and_delegation_attenuation() {
    let _guard = guard().await;
    let Some(fixture) = RemovalFixture::start().await else {
        return;
    };
    let (_, foreign_human, _, foreign_thread) =
        bootstrap(&fixture.client, &fixture.server.base(), "foreign-removal").await;
    fixture
        .refused(
            &foreign_human,
            &fixture.thread,
            &fixture.envelope("foreign-administrator"),
            403,
            Some(&fixture.tenant),
        )
        .await;
    // Valid authority in tenant A cannot select tenant B's aggregate. This
    // domain refusal rolls the provisional admission/idempotency claim back.
    fixture
        .refused(
            &fixture.human,
            &foreign_thread,
            &fixture.envelope("foreign-thread"),
            404,
            None,
        )
        .await;

    let mut borrowed = fixture.envelope("borrow-administrator");
    borrowed.authority_context = Some(AuthorityContext {
        on_behalf_of: fixture.human.clone(),
        purpose: None,
        scope: TargetSelector::TenantWide,
    });
    let refusal = fixture
        .refused(
            &fixture.role,
            &fixture.thread,
            &borrowed,
            403,
            Some(&fixture.tenant),
        )
        .await;
    assert!(refusal["message"]
        .as_str()
        .unwrap()
        .contains("caller's own authority failed"));

    // Even with both explicit administrative grants, a thread-only delegation
    // does not cover tenant administration. Consent/depth policy is separate.
    sqlx::query(
        "UPDATE authority_grants SET actions = actions || '[\"tenant_admin\"]'::jsonb
         WHERE tenant_id=$1 AND subject_id=$2",
    )
    .bind(&fixture.tenant)
    .bind(&fixture.role)
    .execute(&fixture.pool)
    .await
    .unwrap();
    let mut narrowed = fixture.envelope("thread-only-delegation");
    narrowed.authority_context = Some(AuthorityContext {
        on_behalf_of: fixture.role.clone(),
        purpose: None,
        scope: TargetSelector::Threads {
            threads: vec![fixture.thread.parse().unwrap()],
        },
    });
    let refusal = fixture
        .refused(
            &fixture.human,
            &fixture.thread,
            &narrowed,
            403,
            Some(&fixture.tenant),
        )
        .await;
    assert!(refusal["message"]
        .as_str()
        .unwrap()
        .contains("delegation scope"));
    fixture.remove_successfully().await;
}
