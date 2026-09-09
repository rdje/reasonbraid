//! Real protected operator binary, owned database, attributable receipts and pages.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::process::{Output, Stdio};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_core::{GrantSubject, HumanPrincipalId};
use reasonbraid_server::site_authority::{
    self as site, Action, Reason, RegistryCommand, RegistryName, Scope,
};
use serde_json::Value;
use sqlx::{ConnectOptions, PgPool};
use tokio::io::AsyncReadExt;

static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    let fixture =
        sqlx::postgres::PgConnectOptions::from_str(&std::env::var("DATABASE_URL").unwrap())
            .unwrap();
    let configured = fixture.to_url_lossy();
    let selected = configured
        .password()
        .map(|value| percent_encoding::percent_decode_str(value).decode_utf8_lossy());
    assert!(
        selected.as_deref() == Some("reasonbraid-owned-fixture"),
        "the owned passfile must match before any home fallback"
    );
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    sqlx::raw_sql("DELETE FROM public.site_audit; DELETE FROM public.site_grants; DELETE FROM public.site_boundaries; DELETE FROM public.site_regions WHERE region_id LIKE 'site-cli-%'")
        .execute(&pool).await.unwrap();
    Some(pool)
}

async fn cli(url: Option<&str>, args: &[&str], poison_environment: bool) -> Output {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_rb-site"));
    command
        .args(args)
        .env_remove("RB_SITE_DATABASE_URL")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(url) = url {
        command.env("RB_SITE_DATABASE_URL", url);
    }
    if poison_environment {
        // If these influenced the selected connection, the positive operation
        // would fail or use a different identity. Passfile stays repository-local.
        command
            .env("PGHOST", "192.0.2.1")
            .env("PGPORT", "1")
            .env("PGUSER", "wrong_identity")
            .env("PGDATABASE", "wrong_database")
            .env("PGPASSWORD", "wrong_password")
            .env("PGSSLMODE", "require")
            .env("PGOPTIONS", "-c default_transaction_read_only=on")
            .env(
                "PGPASSFILE",
                pg_test_support::repository_root()
                    .unwrap()
                    .join("target/site-cli-passfile-must-not-be-read"),
            );
    }
    let mut child = command.spawn().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).await.unwrap();
        bytes
    });
    let err = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).await.unwrap();
        bytes
    });
    let result = tokio::time::timeout(Duration::from_secs(60), child.wait()).await;
    let timed_out = result.is_err();
    let status = match result {
        Ok(result) => result.unwrap(),
        Err(_) => {
            child.kill().await.unwrap();
            child.wait().await.unwrap()
        }
    };
    let output = Output {
        status,
        stdout: out.await.unwrap(),
        stderr: err.await.unwrap(),
    };
    assert!(!timed_out, "operator child timed out and was killed/reaped");
    output
}

fn json(output: &Output, expected: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

async fn run(url: &str, args: &[&str]) -> Value {
    json(&cli(Some(url), args, false).await, 0)
}

fn scope() -> Scope {
    Scope {
        actions: vec![Action::RegionDeclare],
        valid_from: Utc::now() - chrono::Duration::hours(1),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    }
}

#[tokio::test]
async fn parser_and_target_refusals_do_not_disclose_database_credentials() {
    let _guard = guard().await;
    const SECRET: &str = "credential_marker_never_print";
    let valid = format!("postgres://operator:{SECRET}@127.0.0.1:9/database?sslmode=disable");
    for args in [
        vec!["--help"],
        vec!["boundary", "issue", "--reason", "missing scope"],
        vec!["grant", "issue", "--subject", "tenant:bad"],
        vec![
            "boundary",
            "list",
            "--limit",
            "101",
            "--reason",
            "invalid page",
        ],
        vec!["audit", "list", "--reason", "\n"],
    ] {
        let output = cli(Some(&valid), &args, false).await;
        assert_eq!(
            output.status.code(),
            Some(if args[0] == "--help" { 0 } else { 2 })
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
    }
    let missing = cli(None, &["audit", "list", "--reason", "no endpoint"], false).await;
    assert_eq!(missing.status.code(), Some(2));
    for target in [
        format!("postgres://operator:{SECRET}@192.0.2.1:5432/database"),
        format!("postgres://operator:{SECRET}@localhost:5432/database"),
        format!("postgres://operator:{SECRET}@127.0.0.1/database"),
        format!("postgres://operator:{SECRET}@127.0.0.1:5432/"),
        format!("postgres://operator:{SECRET}@127.0.0.1:5432/database?host=192.0.2.1"),
        format!("postgres://operator:{SECRET}@127.0.0.1:5432/database?sslmode=require"),
        format!(
            "postgres://operator:{SECRET}@127.0.0.1:5432/database?sslmode=disable&sslmode=disable"
        ),
    ] {
        let output = cli(
            Some(&target),
            &["audit", "list", "--reason", "invalid endpoint"],
            false,
        )
        .await;
        assert_eq!(json(&output, 2)["error"]["code"], "invalid_target");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
    }
}

#[tokio::test]
async fn protected_cli_issues_uses_disables_and_audits_exact_authority() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let url = std::env::var("DATABASE_URL").unwrap();
    let scope = scope();
    let from = scope.valid_from.to_rfc3339();
    let until = scope.expires_at.to_rfc3339();
    let actor = GrantSubject::Human(HumanPrincipalId::new());
    let subject = format!("human:{}", actor.id_string());
    let issued = cli(
        Some(&url),
        &[
            "boundary",
            "issue",
            "--action",
            "region_declare",
            "--valid-from",
            &from,
            "--expires-at",
            &until,
            "--reason",
            "operator runbook ceiling",
        ],
        true,
    )
    .await;
    let boundary = json(&issued, 0);
    assert_eq!(boundary["result"]["issued_by"], "postgres");
    let boundary_id = boundary["result"]["boundary_id"].as_str().unwrap();
    let grant = run(
        &url,
        &[
            "grant",
            "issue",
            "--boundary",
            boundary_id,
            "--subject",
            &subject,
            "--action",
            "region_declare",
            "--valid-from",
            &from,
            "--expires-at",
            &until,
            "--reason",
            "operator runbook grant",
        ],
    )
    .await;
    let grant_id = grant["result"]["grant_id"].as_str().unwrap();
    assert_eq!(grant["result"]["subject"]["kind"], "human");
    assert_eq!(grant["result"]["subject"]["id"], actor.id_string());
    let command = |region: &str| RegistryCommand::DeclareRegion {
        region: RegistryName::new(region).unwrap(),
        reason: Reason::new("operator CLI proof").unwrap(),
    };
    site::execute(&pool, &actor, &command("site-cli-before-disable"))
        .await
        .unwrap();
    let inventory = run(&url, &["grant", "list", "--reason", "review grants"]).await;
    assert_eq!(inventory["result"]["items"][0]["boundary_id"], boundary_id);
    assert!(inventory["result"]["next_before"].is_null());
    for (verb, status, changed) in [
        ("suspend", "suspended", true),
        ("revoke", "revoked", true),
        ("revoke", "revoked", false),
        ("suspend", "revoked", false),
    ] {
        let receipt = run(
            &url,
            &["grant", verb, grant_id, "--reason", "retire access"],
        )
        .await;
        assert_eq!(receipt["result"]["status"], status);
        assert_eq!(receipt["result"]["changed"], changed);
        let record: Value =
            sqlx::query_scalar("SELECT to_jsonb(a) FROM public.site_audit a WHERE audit_id=$1")
                .bind(receipt["audit_id"].as_str().unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(record["requested_reason"], "retire access");
        assert_eq!(record["outcome"], if changed { "applied" } else { "noop" });
        assert!(matches!(
            site::execute(&pool, &actor, &command("site-cli-after-disable")).await,
            Err(site::Error::Refused { .. })
        ));
    }
    let suspended = run(
        &url,
        &[
            "boundary",
            "suspend",
            boundary_id,
            "--reason",
            "retire ceiling",
        ],
    )
    .await;
    assert_eq!(suspended["result"]["status"], "suspended");
    let denied = cli(
        Some(&url),
        &[
            "grant",
            "issue",
            "--boundary",
            boundary_id,
            "--subject",
            &subject,
            "--action",
            "region_declare",
            "--valid-from",
            &from,
            "--expires-at",
            &until,
            "--reason",
            "disabled ceiling attempt",
        ],
        false,
    )
    .await;
    assert_eq!(json(&denied, 3)["error"]["code"], "boundary_unavailable");
    run(
        &url,
        &[
            "boundary",
            "revoke",
            boundary_id,
            "--reason",
            "final ceiling retirement",
        ],
    )
    .await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.site_regions WHERE region_id LIKE 'site-cli-%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);

    let outsider = format!("site_cli_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(&format!("CREATE ROLE {outsider} LOGIN"))
        .execute(&pool)
        .await
        .unwrap();
    let mut outsider_url = url::Url::parse(&url).unwrap();
    outsider_url.set_username(&outsider).unwrap();
    let unverified = cli(
        Some(outsider_url.as_str()),
        &[
            "audit",
            "list",
            "--reason",
            "storage must be verified first",
        ],
        false,
    )
    .await;
    assert_eq!(json(&unverified, 1)["error"]["code"], "storage_unverified");
    sqlx::query(&format!("GRANT pg_read_all_settings TO {outsider}"))
        .execute(&pool)
        .await
        .unwrap();
    let before: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM public.site_boundaries),(SELECT COUNT(*) FROM public.site_grants),(SELECT COUNT(*) FROM public.site_audit)").fetch_one(&pool).await.unwrap();
    let refusal = cli(
        Some(outsider_url.as_str()),
        &["audit", "list", "--reason", "outsider inspection"],
        false,
    )
    .await;
    assert_eq!(json(&refusal, 3)["error"]["code"], "operator_required");
    let issuance_args = [
        "boundary",
        "issue",
        "--action",
        "region_declare",
        "--valid-from",
        &from,
        "--expires-at",
        &until,
        "--reason",
        "explicit database role control",
    ];
    let refused_issue = cli(Some(outsider_url.as_str()), &issuance_args, false).await;
    assert_eq!(
        json(&refused_issue, 3)["error"]["code"],
        "operator_required"
    );
    let after: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM public.site_boundaries),(SELECT COUNT(*) FROM public.site_grants),(SELECT COUNT(*) FROM public.site_audit)").fetch_one(&pool).await.unwrap();
    assert_eq!(before, after);
    sqlx::query(&format!("GRANT INSERT ON public.site_audit TO {outsider}"))
        .execute(&pool)
        .await
        .unwrap();
    let auditable_refusal = cli(Some(outsider_url.as_str()), &issuance_args, false).await;
    let refusal = json(&auditable_refusal, 3);
    let recorded: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.site_audit WHERE audit_id=$1 AND actor=$2 AND decision='denied')")
        .bind(refusal["error"]["audit_id"].as_str().unwrap()).bind(&outsider).fetch_one(&pool).await.unwrap();
    assert!(recorded);
    sqlx::raw_sql(&format!("REVOKE pg_read_all_settings FROM {outsider}; REVOKE INSERT ON public.site_audit FROM {outsider}; DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='reasonbraid_site_operator') THEN CREATE ROLE reasonbraid_site_operator NOLOGIN; END IF; END; $$; GRANT reasonbraid_site_operator TO {outsider}; GRANT pg_read_all_settings TO reasonbraid_site_operator; GRANT USAGE ON SCHEMA public TO reasonbraid_site_operator; GRANT SELECT,UPDATE ON public.site_authority_guard TO reasonbraid_site_operator; GRANT SELECT,INSERT ON public.site_boundaries,public.site_grants,public.site_audit TO reasonbraid_site_operator; GRANT UPDATE (status) ON public.site_boundaries,public.site_grants TO reasonbraid_site_operator"))
        .execute(&pool).await.unwrap();
    let member_issue = run(outsider_url.as_str(), &issuance_args).await;
    assert_eq!(member_issue["result"]["issued_by"], outsider);
    let member_boundary = member_issue["result"]["boundary_id"].as_str().unwrap();
    let disabled = run(
        outsider_url.as_str(),
        &[
            "boundary",
            "revoke",
            member_boundary,
            "--reason",
            "column privilege control",
        ],
    )
    .await;
    assert_eq!(disabled["result"]["status"], "revoked");
    let inspected = run(
        outsider_url.as_str(),
        &[
            "audit",
            "list",
            "--limit",
            "1",
            "--reason",
            "inherited inspection control",
        ],
    )
    .await;
    assert_eq!(inspected["result"]["items"].as_array().unwrap().len(), 1);
    let can_rewrite_audit: bool =
        sqlx::query_scalar("SELECT has_table_privilege($1::NAME,'public.site_audit','UPDATE')")
            .bind(&outsider)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        !can_rewrite_audit,
        "the documented operator group does not need audit UPDATE"
    );
    sqlx::raw_sql(&format!("DROP OWNED BY {outsider}; DROP ROLE {outsider}"))
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn paginated_audit_walk_excludes_its_own_new_inspection_records() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let url = std::env::var("DATABASE_URL").unwrap();
    for _ in 0..5 {
        site::issue_boundary(&pool, &scope(), &Reason::new("page seed").unwrap())
            .await
            .unwrap();
    }
    let expected: Vec<String> =
        sqlx::query_scalar("SELECT audit_id FROM public.site_audit ORDER BY audit_id DESC")
            .fetch_all(&pool)
            .await
            .unwrap();
    let mut seen = Vec::new();
    let mut cursor = None::<String>;
    for _ in 0..4 {
        let mut args = vec![
            "audit",
            "list",
            "--limit",
            "2",
            "--reason",
            "bounded history review",
        ];
        if let Some(before) = &cursor {
            args.extend(["--before", before]);
        }
        let page = run(&url, &args).await;
        let items = page["result"]["items"].as_array().unwrap();
        assert!(items.len() <= 2);
        seen.extend(
            items
                .iter()
                .map(|item| item["audit_id"].as_str().unwrap().to_owned()),
        );
        cursor = page["result"]["next_before"].as_str().map(str::to_owned);
        if cursor.is_none() {
            break;
        }
    }
    assert!(cursor.is_none(), "the bounded history walk terminates");
    assert_eq!(seen, expected);
    let boundaries = run(
        &url,
        &[
            "boundary",
            "list",
            "--limit",
            "2",
            "--reason",
            "review ceilings",
        ],
    )
    .await;
    assert_eq!(boundaries["result"]["items"].as_array().unwrap().len(), 2);
    let wrong_cursor = boundaries["result"]["next_before"].as_str().unwrap();
    let refusal = cli(
        Some(&url),
        &[
            "audit",
            "list",
            "--before",
            wrong_cursor,
            "--reason",
            "wrong collection",
        ],
        false,
    )
    .await;
    assert_eq!(json(&refusal, 2)["error"]["code"], "invalid_input");
}
