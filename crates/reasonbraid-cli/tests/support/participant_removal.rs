use super::*;
use serde_json::json;

struct RemovalFixture {
    pool: PgPool,
    server: TestServer,
    _state: CliStateFixture,
    rb: Rb,
    tenant: String,
    source: String,
    source_grant: String,
    human_grant: String,
    participant: String,
}

impl RemovalFixture {
    async fn start() -> Option<Self> {
        let pool = pool().await?;
        let server = TestServer::start(&pool).await;
        let state = CliStateFixture::new("removal");
        let rb = Rb::new(&format!("http://{}", server.addr), state.0.clone());
        let human = rb.json(&["enroll", "human", "organizer", "--json"]).await;
        let tenant = human["tenant_id"].as_str().unwrap().to_string();
        let source = rb
            .json(&[
                "enroll",
                "role",
                "administrator",
                "--tenant",
                &tenant,
                "--actions",
                "tenant_admin,thread_invite",
                "--json",
            ])
            .await;
        let participant = rb
            .json(&["enroll", "role", "reviewer", "--tenant", &tenant, "--json"])
            .await;
        // Explicit fixture authority, not a grant-issuance or consent test. Keep
        // delegation policy eligible without relying on the dev defaults' gap.
        sqlx::query("UPDATE enrollment_boundaries SET delegable=true, max_delegation_depth=1 WHERE tenant_id=$1")
            .bind(&tenant).execute(&pool).await.unwrap();
        sqlx::query("UPDATE authority_grants SET delegable=true WHERE grant_id=$1")
            .bind(source["grant_id"].as_str().unwrap())
            .execute(&pool)
            .await
            .unwrap();
        Some(Self {
            pool,
            server,
            _state: state,
            rb,
            tenant,
            source: source["principal_id"].as_str().unwrap().to_string(),
            source_grant: source["grant_id"].as_str().unwrap().to_string(),
            human_grant: human["grant_id"].as_str().unwrap().to_string(),
            participant: participant["principal_id"].as_str().unwrap().to_string(),
        })
    }

    async fn thread(&self, actor: &str) -> String {
        let result = self
            .rb
            .json(&[
                "thread",
                "create",
                "--subject",
                "removal scope",
                "--objective",
                "qualify administrative delegation",
                "--as",
                actor,
                "--json",
            ])
            .await;
        result["thread_id"].as_str().unwrap().to_string()
    }

    async fn invite(&self, thread: &str, delegated: bool) {
        let mut args = vec![
            "thread",
            "invite",
            "--thread",
            thread,
            "--agent",
            "reviewer",
            "--as",
            "organizer",
            "--json",
        ];
        if delegated {
            args.extend(["--on-behalf-of", &self.source]);
        }
        let result = self.rb.json(&args).await;
        assert_eq!(result["event_type"], "thread.participant_invited");
    }

    async fn remove(
        &self,
        thread: &str,
        actor: &str,
        source: Option<&str>,
    ) -> (bool, String, String) {
        let mut args = vec![
            "thread",
            "remove-participant",
            "--thread",
            thread,
            "--participant",
            &self.participant,
            "--tenant",
            &self.tenant,
            "--as",
            actor,
            "--json",
        ];
        if let Some(source) = source {
            args.extend(["--on-behalf-of", source, "--purpose", "review complete"]);
        }
        self.rb.run(&args).await
    }

    async fn audit_ids(&self) -> Vec<String> {
        sqlx::query_scalar("SELECT record_id FROM authorization_records ORDER BY record_id")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn new_audits(&self, ids: &[String]) -> Vec<Value> {
        sqlx::query_scalar("SELECT to_jsonb(r) FROM authorization_records r WHERE NOT(record_id=ANY($1)) ORDER BY record_id")
            .bind(ids).fetch_all(&self.pool).await.unwrap()
    }

    async fn domain_snapshot(&self) -> Vec<Value> {
        let mut rows = Vec::new();
        for table in [
            "aggregate_state",
            "event_log",
            "outbox",
            "outbox_delivery",
            "node_inbox",
            "budget_reservations",
            "quota_events",
        ] {
            let snapshot: Value = sqlx::query_scalar(&format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(r) ORDER BY to_jsonb(r)::text),'[]'::jsonb) FROM public.\"{table}\" r"))
                .fetch_one(&self.pool).await.unwrap();
            rows.push(snapshot);
        }
        rows
    }

    async fn removed(&self, thread: &str, delegated: bool) {
        let ids = self.audit_ids().await;
        let (ok, out, err) = self
            .remove(
                thread,
                "organizer",
                delegated.then_some(self.source.as_str()),
            )
            .await;
        assert!(
            ok,
            "authorized CLI removal failed: stdout={out}; stderr={err}"
        );
        let reply: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(reply["event_type"], "thread.participant_removed");
        let audits = self.new_audits(&ids).await;
        assert_eq!(audits.len(), 1, "one admission: {audits:?}");
        let audit = &audits[0];
        assert_eq!(audit["tenant_id"], self.tenant);
        assert_eq!(audit["target_kind"], "tenant");
        assert_eq!(audit["target_tenant"], self.tenant);
        assert_eq!(audit["target_thread"], Value::Null);
        assert_eq!(audit["action"], "tenant_admin");
        assert_eq!(audit["decision"], "allowed");
        assert_eq!(
            audit["subject_id"],
            if delegated {
                json!(self.source)
            } else {
                Value::Null
            }
        );
        assert_eq!(
            audit["grant_id"],
            if delegated {
                self.source_grant.as_str()
            } else {
                self.human_grant.as_str()
            }
        );
        let state: Value = sqlx::query_scalar(
            "SELECT state FROM aggregate_state WHERE tenant_id=$1 AND aggregate_id=$2",
        )
        .bind(&self.tenant)
        .bind(thread)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        assert_eq!(state["participants"][&self.participant], "revoked");
        let inspection = self
            .rb
            .json(&["inspect", "thread", thread, "--as", "organizer", "--json"])
            .await;
        assert_eq!(
            inspection["thread"]["state"]["participants"][&self.participant],
            "revoked"
        );
    }

    async fn set_source(&self, actions: Value, selector: Value, status: &str) {
        let result = sqlx::query(
            "UPDATE authority_grants SET actions=$1, selector=$2, status=$3 WHERE grant_id=$4",
        )
        .bind(actions)
        .bind(selector)
        .bind(status)
        .bind(&self.source_grant)
        .execute(&self.pool)
        .await
        .unwrap();
        assert_eq!(result.rows_affected(), 1);
    }

    async fn finish(self) {
        self.server.finish().await;
        self.pool.close().await;
    }
}

#[tokio::test]
async fn real_cli_removal_uses_administrative_scope_and_preserves_refusals() {
    let _guard = e2e_guard().await;
    let Some(fixture) = RemovalFixture::start().await else {
        return;
    };
    let thread = fixture.thread("organizer").await;
    fixture.invite(&thread, false).await;
    // This actual rb invocation reproduces the old thread-only scope failure.
    fixture.removed(&thread, true).await;

    let thread = fixture.thread("organizer").await;
    fixture.invite(&thread, false).await;
    let foreign = fixture
        .rb
        .json(&["enroll", "human", "foreign", "--json"])
        .await;
    let foreign_thread = fixture.thread("foreign").await;
    let foreign_source = foreign["principal_id"].as_str().unwrap();
    for case in [
        "caller",
        "source_action",
        "source_scope",
        "revoked_source",
        "foreign_source",
        "foreign_thread",
    ] {
        fixture
            .set_source(
                if case == "source_action" {
                    json!(["thread_invite"])
                } else {
                    json!(["tenant_admin", "thread_invite"])
                },
                if case == "source_scope" {
                    json!({"kind":"threads", "threads":[thread]})
                } else {
                    json!({"kind":"tenant_wide"})
                },
                if case == "revoked_source" {
                    "revoked"
                } else {
                    "active"
                },
            )
            .await;
        let before = fixture.domain_snapshot().await;
        let state_before = reasonbraid_cli::StateFile::load(&fixture.rb.state_dir).unwrap();
        let ids = fixture.audit_ids().await;
        let (ok, out, err) = fixture
            .remove(
                if case == "foreign_thread" {
                    &foreign_thread
                } else {
                    &thread
                },
                if case == "caller" {
                    "reviewer"
                } else {
                    "organizer"
                },
                Some(if case == "foreign_source" {
                    foreign_source
                } else {
                    &fixture.source
                }),
            )
            .await;
        assert!(!ok, "{case}: removal unexpectedly succeeded: {out}");
        assert_eq!(
            fixture.domain_snapshot().await,
            before,
            "{case}: domain changes"
        );
        assert_eq!(
            serde_json::to_value(reasonbraid_cli::StateFile::load(&fixture.rb.state_dir).unwrap())
                .unwrap(),
            serde_json::to_value(state_before).unwrap(),
            "{case}: CLI state changes"
        );
        let audits = fixture.new_audits(&ids).await;
        if case == "foreign_thread" {
            assert!(err.contains("404"), "{case}: {err}");
            assert!(
                audits.is_empty(),
                "domain refusal must roll admission back: {audits:?}"
            );
        } else {
            assert!(
                err.contains("403") && err.contains("unauthorized"),
                "{case}: {err}"
            );
            assert_eq!(audits.len(), 1, "{case}: {audits:?}");
            assert_eq!(audits[0]["decision"], "denied");
            assert_eq!(audits[0]["action"], "tenant_admin");
            assert_eq!(audits[0]["target_kind"], "tenant");
            assert_eq!(audits[0]["target_tenant"], fixture.tenant);
            assert_eq!(audits[0]["target_thread"], Value::Null);
        }
    }
    fixture.removed(&thread, false).await;
    fixture.finish().await;
}

#[tokio::test]
async fn ordinary_cli_delegation_stays_within_one_thread() {
    let _guard = e2e_guard().await;
    let Some(fixture) = RemovalFixture::start().await else {
        return;
    };
    let thread = fixture.thread("organizer").await;
    let other = fixture.thread("organizer").await;
    fixture
        .set_source(
            json!(["thread_invite"]),
            json!({"kind":"threads", "threads":[thread]}),
            "active",
        )
        .await;
    let ids = fixture.audit_ids().await;
    fixture.invite(&thread, true).await;
    let audits = fixture.new_audits(&ids).await;
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0]["target_kind"], "thread");
    assert_eq!(audits[0]["target_thread"], thread);
    assert_eq!(audits[0]["subject_id"], fixture.source);
    assert_eq!(audits[0]["grant_id"], fixture.source_grant);
    assert_eq!(audits[0]["decision"], "allowed");
    let before = fixture.domain_snapshot().await;
    let ids = fixture.audit_ids().await;
    let (ok, out, err) = fixture
        .rb
        .run(&[
            "thread",
            "invite",
            "--thread",
            &other,
            "--agent",
            "reviewer",
            "--as",
            "organizer",
            "--on-behalf-of",
            &fixture.source,
            "--json",
        ])
        .await;
    assert!(!ok, "foreign scope succeeded: {out}");
    assert!(err.contains("403") && err.contains("unauthorized"), "{err}");
    assert_eq!(fixture.domain_snapshot().await, before);
    let audits = fixture.new_audits(&ids).await;
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0]["target_thread"], other);
    assert_eq!(audits[0]["decision"], "denied");
    fixture.finish().await;
}
