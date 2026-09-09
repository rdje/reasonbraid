//! End-to-end CLI proof (`PHASE-0.6.1`): the REAL `rb` binary drives the whole WP6
//! flow — enroll, create thread, invite, contribute, challenge, revise, close,
//! inspect — against an in-process control API on live PostgreSQL. The acceptance is
//! exercised mechanically: every inspection happens through CLI output, never
//! through database access.
//!
//! Run with `scripts/run_pg_tests.sh` (DATABASE_URL-gated; skips offline so
//! `make check` stays green).

#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "../../reasonbraid-server/tests/support/mod.rs"]
mod pg_test_support;

use std::net::SocketAddr;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::timeout;

use reasonbraid_server::api_router;
use serde_json::Value;
use sqlx::PgPool;

static E2E_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn e2e_guard() -> tokio::sync::MutexGuard<'static, ()> {
    E2E_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // This suite exclusively owns these tables for its duration (FK order).
    for table in [
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "budget_reservations",
        "budget_ceilings",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "node_enroll_audit",
        "node_keys",
        "node_certificates",
        "server_ca",
        "node_leases",
        "node_enrollment_tokens",
        "runs",
        "incarnations",
        "nodes",
        "hosts",
        "profile_versions",
        "agent_profiles",
        "recruitment_panels",
        "recruitment_responses",
        "recruitment_offers",
        "recruitment_calls",
        "agent_roles",
        "human_principals",
        "resource_references",
        "quota_events",
        "usage_quotas",
        "federation_agreements",
        "cross_domain_receipts",
        "tenant_bootstrap_requests",
        "tenants",
        "idempotency",
        "event_log",
        "aggregate_state",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let router = api_router(pool.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self {
            addr,
            handle: Some(handle),
        }
    }
    async fn finish(mut self) {
        let handle = self.handle.take().unwrap();
        handle.abort();
        let result = handle.await;
        assert!(
            matches!(result, Err(ref error) if error.is_cancelled()),
            "owned server shutdown: {result:?}"
        );
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
}

/// One invocation of the REAL rb binary.
struct Rb {
    server: String,
    state_dir: PathBuf,
}

impl Rb {
    fn new(server: &str, state_dir: PathBuf) -> Self {
        Self {
            server: server.to_string(),
            state_dir,
        }
    }

    async fn run(&self, args: &[&str]) -> (bool, String, String) {
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_rb"))
            .args(args)
            .env("REASONBRAID_SERVER", &self.server)
            .env("REASONBRAID_CLI_STATE", &self.state_dir)
            .env("NO_PROXY", "127.0.0.1,localhost")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn rb");
        let mut stdout = child.stdout.take().unwrap().take(1_048_577);
        let mut stderr = child.stderr.take().unwrap().take(1_048_577);
        let mut out = Vec::new();
        let mut err = Vec::new();
        let observed = timeout(Duration::from_secs(90), async {
            tokio::try_join!(
                stdout.read_to_end(&mut out),
                stderr.read_to_end(&mut err),
                child.wait()
            )
        })
        .await;
        let status = match observed {
            Ok(Ok((_, _, status))) => status,
            failure => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                panic!(
                    "owned rb output/wait failed: {failure:?}; stderr={}",
                    String::from_utf8_lossy(&err)
                );
            }
        };
        assert!(
            out.len() <= 1_048_576 && err.len() <= 1_048_576,
            "owned rb output exceeds fixture limit"
        );
        (
            status.success(),
            String::from_utf8_lossy(&out).into_owned(),
            String::from_utf8_lossy(&err).into_owned(),
        )
    }

    /// Run expecting success; return the parsed --json response.
    async fn json(&self, args: &[&str]) -> Value {
        let (ok, stdout, stderr) = self.run(args).await;
        assert!(ok, "rb {args:?} failed\nstdout: {stdout}\nstderr: {stderr}");
        serde_json::from_str(stdout.trim())
            .unwrap_or_else(|e| panic!("rb {args:?} output is not JSON ({e}): {stdout}"))
    }
}

/// Each test owns exactly one new directory; never delete a prior fixed-name
/// workspace on startup. The repository root is derived at runtime.
struct CliStateFixture(PathBuf);

impl CliStateFixture {
    fn new(name: &str) -> Self {
        let cwd = std::env::current_dir().unwrap();
        let root = cwd
            .ancestors()
            .find(|p| p.join("crates/reasonbraid-cli/Cargo.toml").is_file())
            .unwrap();
        let device = std::fs::metadata(root).unwrap().dev();
        let target = root.join("target");
        let meta = std::fs::symlink_metadata(&target).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        let base = target.join("cli-writer-controls");
        match std::fs::create_dir(&base) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => panic!("owned CLI fixture base: {error}"),
        }
        let meta = std::fs::symlink_metadata(&base).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        let path = base.join(format!("e2e-{name}-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for CliStateFixture {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if std::thread::panicking() {
                eprintln!("owned CLI fixture cleanup failed: {error}");
            } else {
                panic!("owned CLI fixture cleanup failed: {error}");
            }
        }
    }
}

/// The full WP6 flow, driven only by the real binary's stdout.
#[tokio::test]
async fn the_real_cli_drives_the_whole_flow() {
    let _guard = e2e_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = format!("http://{}", server.addr);
    let fixture = CliStateFixture::new("full-flow");
    let rb = Rb::new(&base, fixture.0.clone());

    // 1. Bootstrap + role enroll (--json for machine-readable ids).
    let alice = rb.json(&["enroll", "human", "alice", "--json"]).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    assert!(alice_id.starts_with("hpr_"));
    assert!(alice["boundary_id"].is_string(), "bootstrap boundary");

    let reviewer = rb
        .json(&["enroll", "role", "reviewer", "--tenant", &tenant, "--json"])
        .await;
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();
    assert!(reviewer_id.starts_with("rol_"));

    // Re-enrolling the same name is an idempotent replay.
    let (ok, stdout, _) = rb
        .run(&["enroll", "role", "reviewer", "--tenant", &tenant])
        .await;
    assert!(ok);
    assert!(
        stdout.contains("already enrolled"),
        "re-enroll is a replay: {stdout}"
    );

    // 2. Create the thread.
    let created = rb
        .json(&[
            "thread",
            "create",
            "--subject",
            "should we ship?",
            "--objective",
            "decide with evidence",
            "--as",
            "alice",
            "--json",
        ])
        .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    assert_eq!(created["thread_state"], serde_json::json!("open"));

    // 3. Invite (PENDING) → the reviewer ACCEPTS (`.1.3.1` explicit participants)
    //    → contribute → challenge → revise → close (human-readable output).
    let (ok, stdout, stderr) = rb
        .run(&[
            "thread", "invite", "--thread", &thread_id, "--agent", "reviewer", "--as", "alice",
        ])
        .await;
    assert!(ok, "invite failed: {stderr}");
    assert!(stdout.contains("thread.participant_invited"), "{stdout}");

    let (ok, stdout, stderr) = rb
        .run(&[
            "thread", "accept", "--thread", &thread_id, "--as", "reviewer",
        ])
        .await;
    assert!(ok, "accept failed: {stderr}");
    assert!(stdout.contains("thread.invitation_accepted"), "{stdout}");

    let contributed = rb
        .json(&[
            "thread",
            "contribute",
            "--thread",
            &thread_id,
            "--text",
            "Ship it: the kill-risk experiments are green.",
            "--kind",
            "evidence-reference",
            "--evidence-uri",
            "https://example.org/kill-risk",
            "--as",
            "reviewer",
            "--json",
        ])
        .await;
    assert_eq!(
        contributed["event_type"],
        serde_json::json!("thread.contribution_submitted")
    );
    let contribution_event = contributed["event_id"].as_str().unwrap().to_string();

    // `.1.5.1`: the CLI's kind + evidence flags land on the wire and render in the
    // inspection view — the kebab `--kind evidence-reference` normalizes to the
    // wire's snake_case (`evidence_reference`), the `.1.1.3` profile precedent.
    let inspected = rb
        .json(&["inspect", "thread", &thread_id, "--as", "alice", "--json"])
        .await;
    let contribution = inspected["events"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == serde_json::json!("thread.contribution_submitted"))
        .expect("the contribution event exists");
    assert_eq!(
        contribution["body"]["kind"],
        serde_json::json!("evidence_reference")
    );
    assert_eq!(
        contribution["body"]["evidence_refs"],
        serde_json::json!([{ "uri": "https://example.org/kill-risk" }])
    );

    // `.1.5.2`: the human advances the round — the REAL binary drives the verb.
    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "advance-round",
            "--thread",
            &thread_id,
            "--as",
            "alice",
        ])
        .await;
    assert!(ok, "advance failed: {stderr}");
    assert!(stdout.contains("thread.round_advanced"), "{stdout}");

    // `.1.6.1`: the budget read surface — the real binary drives `inspect budget`;
    // the reviewer's accept dispatched a reservation, so the ledger shows a hold.
    let budget = rb
        .json(&["inspect", "budget", &thread_id, "--as", "alice", "--json"])
        .await;
    assert!(
        budget["ceiling"]["ceiling_id"].is_string(),
        "ceiling: {budget}"
    );
    assert!(
        budget["ceiling"]["dimensions"]["calls"].is_number(),
        "the default budget meters calls: {budget}"
    );
    let holds = budget["reservations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["status"] == serde_json::json!("active"))
        .count();
    assert_eq!(
        holds, 1,
        "the accept's dispatch holds one reservation: {budget}"
    );
    let (ok, stdout, stderr) = rb
        .run(&["inspect", "budget", &thread_id, "--as", "alice"])
        .await;
    assert!(ok, "human budget inspect failed: {stderr}");
    assert!(stdout.contains("reservations"), "{stdout}");
    // The inspect gate holds for a role without `thread_inspect` — same gate the
    // page inherits.
    let (ok, _stdout, _stderr) = rb
        .run(&["inspect", "budget", &thread_id, "--as", "reviewer"])
        .await;
    assert!(!ok, "the role has no thread_inspect grant");

    let challenged = rb
        .json(&[
            "thread",
            "challenge",
            "--thread",
            &thread_id,
            "--target",
            &contribution_event,
            "--text",
            "Which experiments, exactly?",
            "--as",
            "alice",
            "--json",
        ])
        .await;
    let challenge_event = challenged["event_id"].as_str().unwrap().to_string();
    assert_eq!(
        challenged["event_type"],
        serde_json::json!("thread.challenge_posted")
    );

    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "revise",
            "--thread",
            &thread_id,
            "--target",
            &challenge_event,
            "--text",
            "The SQLite kill-point sweep and the Codex qualification.",
            "--as",
            "reviewer",
        ])
        .await;
    assert!(ok, "revise failed: {stderr}");
    assert!(stdout.contains("thread.revision_submitted"), "{stdout}");

    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "close",
            "--thread",
            &thread_id,
            "--reason",
            "decision reached",
            "--as",
            "alice",
        ])
        .await;
    assert!(ok, "close failed: {stderr}");
    assert!(stdout.contains("thread.closed"), "{stdout}");
    assert!(stdout.contains("now closed"), "{stdout}");

    // 4. Inspect — through the CLI only (the acceptance).
    let (ok, stdout, stderr) = rb
        .run(&["inspect", "thread", &thread_id, "--as", "alice"])
        .await;
    assert!(ok, "inspect failed: {stderr}");
    assert!(stdout.contains("state: closed"), "{stdout}");
    assert!(stdout.contains("reason: decision reached"), "{stdout}");
    assert!(stdout.contains("thread.created"), "{stdout}");
    assert!(stdout.contains("thread.participant_invited"), "{stdout}");
    assert!(stdout.contains("thread.invitation_accepted"), "{stdout}");
    assert!(stdout.contains("thread.contribution_submitted"), "{stdout}");
    assert!(stdout.contains("thread.challenge_posted"), "{stdout}");
    assert!(stdout.contains("thread.revision_submitted"), "{stdout}");
    assert!(stdout.contains("thread.closed"), "{stdout}");
    assert!(stdout.contains("audit:"), "{stdout}");
    assert!(stdout.contains("contributions=1"), "{stdout}");
    assert!(stdout.contains("revisions=1"), "{stdout}");
    assert!(stdout.contains("open_challenges=0"), "{stdout}");
    assert!(
        stdout.contains(&reviewer_id),
        "participants listed: {stdout}"
    );
    assert!(stdout.contains("accepted"), "{stdout}");

    // `.1.5.3`: the honest close — a second thread ends INCONCLUSIVELY with its
    // unresolved register, driven by the REAL binary.
    let honest = rb
        .json(&[
            "thread",
            "create",
            "--subject",
            "honest",
            "--objective",
            "no decision",
            "--as",
            "alice",
            "--json",
        ])
        .await;
    let honest_id = honest["thread_id"].as_str().unwrap().to_string();
    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "close",
            "--thread",
            &honest_id,
            "--reason",
            "did not converge",
            "--outcome",
            "inconclusive",
            "--unresolved",
            "the objection stands",
            "--as",
            "alice",
        ])
        .await;
    assert!(ok, "inconclusive close failed: {stderr}");
    assert!(stdout.contains("thread.closed"), "{stdout}");
    let inspected = rb
        .json(&["inspect", "thread", &honest_id, "--as", "alice", "--json"])
        .await;
    assert_eq!(
        inspected["thread"]["state"]["state"],
        serde_json::json!("inconclusive")
    );
    let close_event = inspected["events"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == serde_json::json!("thread.closed"))
        .expect("the close event exists");
    // `.2.4.1`: the legacy wire word is the accepted alias — the event
    // persists the CANONICAL terminal (the aliases never persist).
    assert_eq!(
        close_event["body"]["outcome"],
        serde_json::json!("deadlocked")
    );
    assert_eq!(
        close_event["body"]["unresolved"],
        serde_json::json!(["the objection stands"])
    );

    let (ok, stdout, stderr) = rb.run(&["inspect", "threads", "--as", "alice"]).await;
    assert!(ok, "inspect threads failed: {stderr}");
    assert!(stdout.contains(&thread_id), "{stdout}");

    // 5. The thread stays inspectable through the API for the OTHER enrolled
    //    principal too (thread_inspect is granted to the role? no — the role's
    //    grant is thread_contribute only; inspection by the role is DENIED).
    let (ok, _, stderr) = rb
        .run(&["inspect", "thread", &thread_id, "--as", "reviewer"])
        .await;
    assert!(!ok, "the role has no thread_inspect grant");
    assert!(stderr.contains("unauthorized"), "{stderr}");

    // 6. `PHASE-1.1.3`: the typed create flags and the cancel terminal, driven by
    //    the REAL binary — cancel is inspectable through the CLI only.
    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "create",
            "--subject",
            "typed and doomed",
            "--objective",
            "profile and cancel",
            "--classification",
            "confidential",
            "--workflow-profile",
            "critique",
            "--as",
            "alice",
        ])
        .await;
    assert!(ok, "typed create failed: {stderr}");
    assert!(stdout.contains("created thread"), "{stdout}");
    let second = stdout
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(2))
        .expect("created thread id")
        .to_string();

    let (ok, stdout, stderr) = rb
        .run(&["inspect", "thread", &second, "--as", "alice"])
        .await;
    assert!(ok, "inspect typed thread failed: {stderr}");
    assert!(stdout.contains("state: open"), "{stdout}");

    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "cancel",
            "--thread",
            &second,
            "--reason",
            "no longer needed",
            "--as",
            "alice",
        ])
        .await;
    assert!(ok, "cancel failed: {stderr}");
    assert!(stdout.contains("thread.cancelled"), "{stdout}");
    assert!(stdout.contains("now cancelled"), "{stdout}");

    let (ok, stdout, stderr) = rb
        .run(&["inspect", "thread", &second, "--as", "alice"])
        .await;
    assert!(ok, "inspect cancelled failed: {stderr}");
    assert!(stdout.contains("state: cancelled"), "{stdout}");
    assert!(stdout.contains("cancelled: no longer needed"), "{stdout}");
    assert!(stdout.contains("thread.cancelled"), "{stdout}");
    server.finish().await;
    pool.close().await;
}

/// Deny-by-default at the CLI: a role without `thread_create` gets a typed refusal,
/// and nothing is created.
#[tokio::test]
async fn the_cli_surfaces_typed_denials() {
    let _guard = e2e_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = format!("http://{}", server.addr);
    let fixture = CliStateFixture::new("denials");
    let rb = Rb::new(&base, fixture.0.clone());

    let alice = rb.json(&["enroll", "human", "alice", "--json"]).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    rb.run(&["enroll", "role", "reviewer", "--tenant", &tenant])
        .await;

    let (ok, stdout, stderr) = rb
        .run(&[
            "thread",
            "create",
            "--subject",
            "s",
            "--objective",
            "o",
            "--as",
            "reviewer",
        ])
        .await;
    assert!(!ok, "the reviewer cannot create threads\nstdout: {stdout}");
    assert!(stderr.contains("unauthorized"), "typed denial: {stderr}");
    assert!(
        stderr.contains("denied"),
        "the denial names its audit record: {stderr}"
    );

    // The CLI also fails loudly on a bad verb target (invalid transition), and the
    // state dir stays consistent for the next invocation.
    let (ok, _, stderr) = rb
        .run(&[
            "inspect",
            "thread",
            "thr_00000000-0000-7000-8000-000000000099",
            "--as",
            "alice",
        ])
        .await;
    assert!(!ok);
    assert!(stderr.contains("scope_hidden"), "{stderr}");
    server.finish().await;
    pool.close().await;
}
