//! Destructive fixtures require a live runner receipt and connection-bound proof.

#![cfg(unix)] // The disposable runner uses POSIX process groups and volume IDs.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup_tests.rs"]
mod cleanup_tests;
#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::fs;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use pg_test_support::Ownership;
use serde_json::json;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = pg_test_support::repository_root().unwrap();
        let volume = fs::metadata(&root).unwrap().dev();
        let mut parent = root;
        for part in ["target", "pg-guard-controls"] {
            parent.push(part);
            // Creating only when the path is ABSENT is a check-then-act: these
            // tests run in parallel threads, so several see the same missing
            // parent and all but one lose the create. It never fired locally
            // because `target/` stays warm between runs and the racing branch
            // is then dead code; a fresh checkout runs it every time, which is
            // why only CI saw it. An existing directory is the outcome this
            // wants, so it is accepted rather than raced for.
            match fs::DirBuilder::new().mode(0o700).create(&parent) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => panic!(
                    "guard fixture storage {} refused: {error}",
                    parent.display()
                ),
            }
            let metadata = fs::symlink_metadata(&parent).unwrap();
            assert!(metadata.is_dir() && metadata.dev() == volume);
        }
        // A process id and a clock reading do not make a name unique: these
        // tests run in parallel threads of ONE process, and this host returns
        // byte-identical `time_ns()` for consecutive calls, so two fixtures
        // could propose the same directory and the loser's exclusive create
        // panicked. The counter is unique within the process, the process id
        // across processes, and the bounded retry covers anything else — a
        // candidate that is already taken is skipped, never adopted.
        for _ in 0..64 {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("{}-{sequence}", std::process::id()));
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => panic!("guard fixture creation failed: {error}"),
            }
        }
        panic!("guard fixture names exhausted; existing directories stay untouched");
    }

    fn prepare(&self, relative: &str, database: &str, port: u16, pid: u32) {
        let workspace = self.0.join(relative);
        fs::create_dir_all(workspace.join("data")).unwrap();
        fs::write(workspace.join("data/postmaster.pid"), format!("{pid}\n")).unwrap();
        fs::write(
            workspace.join("runner.json"),
            json!({
                "workspace": relative, "database": database, "port": port,
                "postgres_pid": pid, "state": "running",
            })
            .to_string(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0)
            .expect("remove metadata-only fixture; no server was started there");
    }
}

#[test]
fn preflight_refuses_unsafe_targets_and_nonlive_receipts_before_connecting() {
    let fixture = Fixture::new();
    let relative = "target/pg-tests/run-control";
    let database = "rb_test_0123456789abcdef01234567";
    let token = "0123456789abcdef0123456789abcdef0123456789abcdef";
    let url = format!("postgres://postgres@127.0.0.1:54321/{database}?sslmode=disable");
    fixture.prepare(relative, database, 54321, 12345);
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_ok());
    for bad_url in [
        "postgres://production.invalid/important".to_owned(),
        format!("{url}&options=-csearch_path=foreign"),
        url.replace("54321", "54322"),
        url.replace(database, "other_database"),
    ] {
        assert!(Ownership::read(&fixture.0, &bad_url, relative, token, database).is_err());
    }
    for bad_path in [
        "../outside",
        "/absolute",
        "target/pg-tests/../run-control",
        "target/other/run-control",
    ] {
        assert!(Ownership::read(&fixture.0, &url, bad_path, token, database).is_err());
    }
    assert!(Ownership::read(&fixture.0, &url, relative, "", database).is_err());
    let receipt = fixture.0.join(relative).join("runner.json");
    let original = fs::read_to_string(&receipt).unwrap();
    fs::write(&receipt, original.replace("running", "stopped")).unwrap();
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_err());
    fs::write(&receipt, &original).unwrap();
    fs::write(
        fixture.0.join(relative).join("data/postmaster.pid"),
        "54321\n",
    )
    .unwrap();
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_err());
    fs::remove_file(receipt).unwrap();
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_workspace_or_receipt_is_not_ownership() {
    let fixture = Fixture::new();
    let relative = "target/pg-tests/run-control";
    let database = "rb_test_0123456789abcdef01234567";
    let token = "0123456789abcdef0123456789abcdef0123456789abcdef";
    let url = format!("postgres://postgres@127.0.0.1:54321/{database}?sslmode=disable");
    fixture.prepare(relative, database, 54321, 12345);
    let workspace = fixture.0.join(relative);
    let receipt = workspace.join("runner.json");
    fs::rename(&receipt, workspace.join("saved.json")).unwrap();
    std::os::unix::fs::symlink("saved.json", &receipt).unwrap();
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_err());
    fs::remove_file(receipt).unwrap();
    fs::rename(workspace.join("saved.json"), workspace.join("runner.json")).unwrap();
    let moved = fixture.0.join("moved");
    fs::rename(&workspace, &moved).unwrap();
    std::os::unix::fs::symlink(&moved, &workspace).unwrap();
    assert!(Ownership::read(&fixture.0, &url, relative, token, database).is_err());
}

#[tokio::test]
async fn forged_proof_cannot_open_a_pool_and_every_new_connection_is_checked() {
    let Some(admin) = pg_test_support::pool().await else {
        return;
    };
    let url = std::env::var("DATABASE_URL").unwrap();
    let relative = std::env::var("RB_TEST_CLUSTER").unwrap();
    let token = std::env::var("RB_TEST_OWNER").unwrap();
    let database = std::env::var("RB_TEST_DATABASE").unwrap();
    let root = pg_test_support::repository_root().unwrap();
    sqlx::query("CREATE TABLE pg_guard_witness (value TEXT NOT NULL)")
        .execute(&admin)
        .await
        .unwrap();
    sqlx::query("INSERT INTO pg_guard_witness VALUES ('unchanged')")
        .execute(&admin)
        .await
        .unwrap();

    let mut wrong_token = token.clone();
    wrong_token.replace_range(..1, if token.starts_with('0') { "1" } else { "0" });
    let forged = Ownership::read(&root, &url, &relative, &wrong_token, &database).unwrap();
    assert!(
        forged.connect().await.is_err(),
        "forged owner must fail before fixture SQL"
    );
    let fixture = Fixture::new();
    let receipt: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(&relative).join("runner.json")).unwrap(),
    )
    .unwrap();
    fixture.prepare(
        &relative,
        &database,
        receipt["port"].as_u64().unwrap() as u16,
        receipt["postgres_pid"].as_u64().unwrap() as u32,
    );
    let wrong_directory = Ownership::read(&fixture.0, &url, &relative, &token, &database).unwrap();
    assert!(
        wrong_directory.connect().await.is_err(),
        "a forged local receipt cannot attest a different server directory"
    );

    let ownership = Ownership::read(&root, &url, &relative, &token, &database).unwrap();
    let checked = ownership.connect().await.unwrap();
    let held = checked.acquire().await.unwrap();
    // A role default changes the marker only for NEW sessions. Hold the original
    // connection so the next acquisition must exercise after_connect again.
    sqlx::query("ALTER ROLE postgres SET reasonbraid.test_owner = 'wrong-new-session'")
        .execute(&admin)
        .await
        .unwrap();
    let refused = checked.acquire().await.is_err();
    // Restore through the already verified admin connection before asserting,
    // so a failing control does not strand the rest of the owned cluster.
    sqlx::query("ALTER ROLE postgres RESET reasonbraid.test_owner")
        .execute(&admin)
        .await
        .unwrap();
    assert!(
        refused,
        "a pool must not admit an unchecked replacement connection"
    );
    drop(held);
    let value: String = sqlx::query_scalar("SELECT value FROM pg_guard_witness")
        .fetch_one(&checked)
        .await
        .unwrap();
    assert_eq!(value, "unchanged");
    sqlx::query("DROP TABLE pg_guard_witness")
        .execute(&admin)
        .await
        .unwrap();
    checked.close().await;
    admin.close().await;
}
