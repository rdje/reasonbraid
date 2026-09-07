//! Migration upgrade test (`PHASE-2.4.2`; ROADMAP §17.6): the
//! upgrade-an-EXISTING-database path — every other suite migrates a FRESH
//! database, this one applies all but the LAST migration, seeds data through
//! the real API, applies the rest, and asserts the data + the behavior
//! survive. Skips offline (no DATABASE_URL).

use std::sync::OnceLock;

use serde_json::{json, Value};
use sqlx::migrate::Migrator;
use sqlx::PgPool;

static UPGRADE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    UPGRADE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enroll request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("enroll json"))
}

/// THE §17.6 upgrade leg: an existing database upgrades — the seeded rows and
/// the API behavior survive the last migration.
#[tokio::test]
async fn an_existing_database_upgrades_and_its_data_survives() {
    let _g = guard().await;
    let Some(url) = std::env::var("DATABASE_URL").ok() else {
        eprintln!("SKIP: DATABASE_URL is unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .expect("migrator");

    // 1. All but the LAST migration (the pre-upgrade database).
    let all_but_last: Vec<_> = migrator
        .migrations
        .iter()
        .filter(|m| m.version != migrator.migrations.last().unwrap().version)
        .cloned()
        .collect();
    let prefix = Migrator {
        migrations: std::borrow::Cow::Owned(all_but_last),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    // The upgrade path starts from a CLEAN pre-upgrade database.
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&pool)
        .await
        .expect("drop schema");
    sqlx::query("CREATE SCHEMA public")
        .execute(&pool)
        .await
        .expect("create schema");
    prefix
        .run(&pool)
        .await
        .expect("apply all but the last migration");

    // 2. Seed data through the REAL API (the enroll path writes tenants,
    //    boundaries, grants — the rows the last migration must not disturb).
    let server = {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().unwrap();
        let pool = pool.clone();
        tokio::spawn(async move {
            let router = reasonbraid_server::api_router(pool);
            axum::serve(listener, router).await.expect("serve");
        });
        format!("http://{addr}")
    };
    let client = reqwest::Client::new();
    let (status, alice) = enroll(
        &client,
        &server,
        json!({ "kind": "human", "name": "upgrade-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let seeded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(seeded, 1, "the API seeded the tenant");

    // 3. THE upgrade: the remaining migration(s) over the existing data.
    migrator
        .run(&pool)
        .await
        .expect("apply the remaining migrations");

    // 4. The data survived.
    let tenants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count after upgrade");
    assert_eq!(tenants, 1, "the tenant row survived the upgrade");
    let boundaries: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM enrollment_boundaries WHERE tenant_id = $1")
            .bind(&tenant)
            .fetch_one(&pool)
            .await
            .expect("boundaries after upgrade");
    assert_eq!(boundaries, 1, "the boundary row survived the upgrade");

    // 5. The API behavior survives (the post-upgrade surface works over the
    //    upgraded database): the role enroll path still answers.
    let (status, role) = enroll(
        &client,
        &server,
        json!({ "kind": "role", "name": "upgrade-role", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls after the upgrade: {role}");
    assert!(role["principal_id"].as_str().unwrap().starts_with("rol_"));
}
