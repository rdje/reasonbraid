//! Backup + restore exercise (`PHASE-2.4.1`; ROADMAP §17.5's last line: a
//! backup that has never been restored is not a recovery control). The test
//! seeds rows, takes a REAL pg_dump of the live database, MUTATES it, restores
//! the dump into an ISOLATED database, and asserts the restored state matches
//! the pre-mutation state — the restore is the proof, run in the guard.
//! Skips offline (no DATABASE_URL) or when the pg_dump/pg_restore binaries are
//! absent.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::path::PathBuf;
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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/backup-restore-controls")
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
