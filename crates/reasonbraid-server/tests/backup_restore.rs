//! Backup + restore exercise (`PHASE-2.4.1`; ROADMAP §17.5's last line: a
//! backup that has never been restored is not a recovery control). The test
//! seeds rows, takes a REAL pg_dump of the live database, MUTATES it, restores
//! the dump into an ISOLATED database, and asserts the restored state matches
//! the pre-mutation state — the restore is the proof, run in the guard.
//! Skips offline (no DATABASE_URL) or when the pg_dump/pg_restore binaries are
//! absent.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::process::Command;
use std::sync::OnceLock;

use serde_json::json;
use sqlx::PgPool;

static RESTORE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    RESTORE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

fn have_pg_tools() -> bool {
    Command::new("pg_dump")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        && Command::new("pg_restore")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

/// Split a postgres URL into its components; returns (user, host, port, dbname)
/// for a URL like postgres://user@host:port/dbname.
fn url_parts(url: &str) -> (Option<String>, String, Option<String>, String) {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .expect("a postgres URL");
    let slash = rest.rfind('/').expect("a dbname component");
    let mut authority = &rest[..slash];
    let user = match authority.rfind('@') {
        Some(at) => {
            let u = authority[..at].to_string();
            authority = &authority[at + 1..];
            Some(u)
        }
        None => None,
    };
    // Split host[:port].
    let (host, port) = match authority.rfind(':') {
        Some(colon) => {
            let port: u16 = authority[colon + 1..].parse().expect("a numeric port");
            (&authority[..colon], Some(port.to_string()))
        }
        None => (authority, None),
    };
    (user, host.to_string(), port, rest[slash + 1..].to_string())
}

/// THE restore exercise: seed → dump → mutate → restore → assert.
#[tokio::test]
async fn a_backup_restores_into_an_isolated_database() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let url = std::env::var("DATABASE_URL").expect("validated DATABASE_URL remains present");
    if !have_pg_tools() {
        eprintln!("SKIP: pg_dump/pg_restore not on PATH");
        return;
    }
    let (user, host, port, dbname) = url_parts(&url);
    let _ = dbname;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("migrate");

    // 1. Seed: a tenant row + a budget row the restore must bring back.
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name) \
         VALUES ('ten_00000000-0000-7000-8000-0000000000aa', 'restore-seed') \
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .expect("seed tenant");
    sqlx::query(
        "INSERT INTO budget_ceilings (ceiling_id, tenant_id, thread_id, dimensions, policy_version) \
         VALUES ('ceil_restore', 'ten_00000000-0000-7000-8000-0000000000aa', \
                 'thr_00000000-0000-7000-8000-0000000000aa', $1, 'dev-budget-1')",
    )
    .bind(json!({"calls": 10, "input_tokens": null, "output_tokens": null, "wall_clock_seconds": null}))
    .execute(&pool)
    .await
    .expect("seed budget");

    // 2. The backup (the real pg_dump, the same command scripts/backup.sh runs).
    // §13: project data lives on the repository's own volume, never in an
    // ambient temporary directory. The directory name was also FIXED, so two
    // runs shared it, and a clock supplied the only uniqueness in the file
    // name.
    let run = uuid::Uuid::now_v7().simple().to_string();
    let dir = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target/backup-restore-controls")
        .join(format!("exercise-{run}"));
    std::fs::create_dir_all(dir.parent().expect("the control parent")).expect("backup parent");
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the backup directory is new");
    let file = dir.join("exercise.dump");
    let dump = Command::new("pg_dump")
        .args(["--format=custom", "--no-owner", "--file"])
        .arg(&file)
        .arg(&url)
        .output()
        .expect("pg_dump runs");
    assert!(
        dump.status.success(),
        "pg_dump failed: {}",
        String::from_utf8_lossy(&dump.stderr)
    );

    // 3. MUTATE the live database: the seeded rows go away.
    sqlx::query("DELETE FROM budget_ceilings WHERE ceiling_id = 'ceil_restore'")
        .execute(&pool)
        .await
        .expect("mutate");
    let gone: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM budget_ceilings WHERE ceiling_id = 'ceil_restore'",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(gone, 0, "the mutation removed the seeded row");

    // 4. Restore into an ISOLATED database.
    let target = format!("reasonbraid_restore_{run}");
    let target_url = match (&user, &port) {
        (Some(user), Some(port)) => format!("postgres://{user}@{host}:{port}/{target}"),
        (Some(user), None) => format!("postgres://{user}@{host}/{target}"),
        (None, Some(port)) => format!("postgres://{host}:{port}/{target}"),
        (None, None) => format!("postgres://{host}/{target}"),
    };
    // CREATE/DROP run on the verified pool connection, so a reused TCP port
    // cannot redirect these administrative mutations to another cluster.
    // `target` contains only this fixed prefix and a generated integer.
    sqlx::query(&format!("CREATE DATABASE {target}"))
        .execute(&pool)
        .await
        .expect("create the isolated restore database on the verified server");
    let restore = Command::new("pg_restore")
        .args([
            "--clean",
            "--if-exists",
            "--no-owner",
            "--exit-on-error",
            "--dbname",
            &target_url,
        ])
        .arg(&file)
        .output()
        .expect("pg_restore runs");
    assert!(
        restore.status.success(),
        "pg_restore failed: {}",
        String::from_utf8_lossy(&restore.stderr)
    );

    // 5. THE assertion: the restored database has the pre-mutation state.
    let restored = PgPool::connect(&target_url)
        .await
        .expect("connect to the restored database");
    let ceiling: (String, String) = sqlx::query_as(
        "SELECT ceiling_id, tenant_id FROM budget_ceilings WHERE ceiling_id = 'ceil_restore'",
    )
    .fetch_one(&restored)
    .await
    .expect("the restored row");
    assert_eq!(ceiling.1, "ten_00000000-0000-7000-8000-0000000000aa");
    restored.close().await;
    // The isolated database is disposable — drop it after the assertion.
    sqlx::query(&format!("DROP DATABASE {target}"))
        .execute(&pool)
        .await
        .expect("drop the isolated restore database on the verified server");
    std::fs::remove_file(&file).ok();
}

/// Run one of the backup scripts from the repository, returning (success, output).
fn run_script(script: &str, env: &[(&str, &str)]) -> (bool, String) {
    let root = pg_test_support::repository_root().expect("repository root");
    let output = Command::new("bash")
        .arg(root.join("scripts").join(script))
        .envs(env.iter().copied())
        .output()
        .expect("the script runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

/// THE `SIGNOFF-REPAIR.4.6.1.5.2` acceptance: §18.5's backup/restore status is
/// reported from what RUNNING the two scripts leaves behind — never from a
/// receipt written by hand — and §17.5's rule decides the verdict.
///
/// The sequence is the procedure itself: back up, see the backup listed but
/// NOT accepted (it has never been restored); restore-test it, see it
/// accepted; then damage the dump and see both the restore test refuse it and
/// the report stop counting it.
#[tokio::test]
async fn the_backup_status_is_reported_from_what_the_scripts_leave_behind() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let url = std::env::var("DATABASE_URL").expect("validated DATABASE_URL remains present");
    if !have_pg_tools() {
        eprintln!("SKIP: pg_dump/pg_restore not on PATH");
        return;
    }
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("migrate");
    let fixture = reasonbraid_core::fixture::Fixture::create("backup-restore-tests", "receipts")
        .expect("fixture");
    let backups = fixture.join("backups");
    let backup_dir = backups.to_str().expect("a UTF-8 fixture path");

    // A real server: enrolment mints the administrator the report requires.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let router = reasonbraid_server::api_router(pool.clone()).merge(
        reasonbraid_server::backup_router(pool.clone(), Some(backups.clone())),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    let client = reqwest::Client::new();
    let admin: serde_json::Value = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({ "kind": "human", "name": "backup-admin" }))
        .send()
        .await
        .expect("enrol")
        .json()
        .await
        .expect("enrol json");
    let admin = admin["principal_id"].as_str().unwrap().to_string();
    let status = || {
        let client = client.clone();
        let base = base.clone();
        let admin = admin.clone();
        async move {
            let response = client
                .get(format!("{base}/v1/admin/backups"))
                .header(reasonbraid_server::PRINCIPAL_HEADER, &admin)
                .send()
                .await
                .expect("status request");
            assert_eq!(response.status().as_u16(), 200);
            assert!(response
                .headers()
                .contains_key("x-reasonbraid-authorization"));
            response
                .json::<serde_json::Value>()
                .await
                .expect("status json")
        }
    };

    // (1) Back up. Listed, intact — and NOT a recovery control yet.
    let (ok, out) = run_script(
        "backup.sh",
        &[("DATABASE_URL", &url), ("BACKUP_DIR", backup_dir)],
    );
    assert!(ok, "backup.sh: {out}");
    let body = status().await;
    assert_eq!(body["declared"], json!(true));
    let listed = body["backups"].as_array().expect("the backup list");
    assert_eq!(listed.len(), 1, "{body}");
    let dump = listed[0]["dump"].as_str().unwrap().to_string();
    assert_eq!(listed[0]["file"], json!("intact"));
    assert_eq!(listed[0]["restore_test"], json!(null));
    assert_eq!(
        body["recovery_control"],
        json!("not_accepted"),
        "a backup never restored is not a recovery control: {body}"
    );

    // (2) Restore-test it into an ISOLATED database. Now it counts.
    let target = format!("reasonbraid_restore_receipt_{}", std::process::id());
    sqlx::query(&format!("CREATE DATABASE {target}"))
        .execute(&pool)
        .await
        .expect("create the isolated restore database on the verified server");
    let options = pool.connect_options();
    let target_url = format!(
        "postgres://{}@{}:{}/{target}",
        options.get_username(),
        options.get_host(),
        options.get_port()
    );
    let dump_path = backups.join(&dump);
    let dump_arg = dump_path.to_str().unwrap();
    // `SIGNOFF-REPAIR.11.3.2`: the connection rides libpq's environment, so an
    // AMBIENT libpq variable meant for another server must not ride along —
    // this test server speaks neither TLS nor a password, and both are asked
    // for here. The script clears them before it exports the target's own.
    let (ok, out) = run_script(
        "restore.sh",
        &[
            ("BACKUP_FILE", dump_arg),
            ("RESTORE_DATABASE_URL", &target_url),
            ("PGSSLMODE", "require"),
            ("PGPASSWORD", "meant-for-another-server"),
        ],
    );
    assert!(ok, "restore.sh: {out}");
    let body = status().await;
    assert_eq!(body["recovery_control"], json!("accepted"), "{body}");
    let restore = &body["backups"][0]["restore_test"];
    assert_eq!(restore["matches_backup"], json!(true), "{body}");
    assert_eq!(restore["target_database"], json!(target));
    assert!(restore["migrations"].as_u64().unwrap() > 0, "{body}");

    // (2b) `SIGNOFF-REPAIR.11.3.2`: the restore test runs `pg_restore --clean`, so
    // its target must be EMPTY. The target just restored into now carries this
    // product's schema; restoring into it again is refused before anything is
    // dropped, and the receipt it already earned is untouched.
    let (ok, out) = run_script(
        "restore.sh",
        &[
            ("BACKUP_FILE", dump_arg),
            ("RESTORE_DATABASE_URL", &target_url),
        ],
    );
    assert!(!ok, "a populated target must not be restored into: {out}");
    assert!(out.contains("is not empty"), "the refusal says why: {out}");
    // …and the live database is refused BY NAME, before its contents are even
    // looked at: `DATABASE_URL` is the database this run is serving from.
    let (ok, out) = run_script(
        "restore.sh",
        &[
            ("BACKUP_FILE", dump_arg),
            ("RESTORE_DATABASE_URL", &url),
            ("DATABASE_URL", &url),
        ],
    );
    assert!(!ok, "the live database must not be restored into: {out}");
    assert!(
        out.contains("names the live database"),
        "the refusal says why: {out}"
    );
    assert!(
        !out.contains("pg_restore"),
        "nothing ran against the live database: {out}"
    );

    // (3) Damage the dump. The restore test refuses it before restoring
    // anything, and the report stops counting it.
    let bytes = std::fs::read(&dump_path).unwrap();
    std::fs::write(&dump_path, &bytes[..bytes.len() / 2]).unwrap();
    let (ok, out) = run_script(
        "restore.sh",
        &[
            ("BACKUP_FILE", dump_arg),
            ("RESTORE_DATABASE_URL", &target_url),
        ],
    );
    assert!(!ok, "a truncated dump must not restore-test: {out}");
    assert!(out.contains("its receipt recorded"), "{out}");
    let body = status().await;
    assert_eq!(body["backups"][0]["file"], json!("size_mismatch"), "{body}");
    assert_eq!(body["recovery_control"], json!("not_accepted"), "{body}");

    server.abort();
    sqlx::query(&format!("DROP DATABASE {target} WITH (FORCE)"))
        .execute(&pool)
        .await
        .expect("drop the isolated restore database");
}
