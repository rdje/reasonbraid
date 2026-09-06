//! WP5 budget tests, server side (`PHASE-0.5.2`): the ceiling holds — reservations are
//! atomic against the currently-held amount, settlements free the unused remainder
//! (and record overruns, never clamp), releases return everything, expired
//! reservations stop holding, and denials are recorded rows. Run via
//! `scripts/run_pg_tests.sh` / the `pg-tests` CI job; skip offline.

use std::sync::OnceLock;

use chrono::{Duration, Utc};
use reasonbraid_core::{BudgetDimensions, BudgetError};
use reasonbraid_server::{
    create_ceiling, create_reservation, release_reservation, settle_reservation,
};
use sqlx::PgPool;

static BUDGET_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn budget_guard() -> tokio::sync::MutexGuard<'static, ()> {
    BUDGET_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real budget proof"
            );
            return None;
        }
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // The budget tables belong exclusively to this binary: purge at start.
    sqlx::query("DELETE FROM budget_reservations")
        .execute(&pool)
        .await
        .expect("purge budget_reservations");
    sqlx::query("DELETE FROM budget_ceilings")
        .execute(&pool)
        .await
        .expect("purge budget_ceilings");
    Some(pool)
}

fn dims(calls: Option<u64>, tokens: Option<u64>) -> BudgetDimensions {
    BudgetDimensions {
        calls,
        input_tokens: tokens,
        output_tokens: tokens,
        wall_clock_seconds: None,
    }
}

async fn ceiling(pool: &PgPool, tenant: &str, thread: &str, calls: u64, tokens: u64) -> String {
    let id = format!("ceil_{}", &tenant[4..]);
    create_ceiling(pool, &id, tenant, thread, &dims(Some(calls), Some(tokens)))
        .await
        .unwrap();
    id
}

/// A reservation within the ceiling holds; one beyond it is denied AND recorded.
#[tokio::test]
async fn reserve_within_the_ceiling_holds_and_beyond_it_is_denied_and_recorded() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000201";
    let ceiling_id = ceiling(
        &pool,
        tenant,
        "thr_00000000-0000-7000-8000-000000000201",
        10,
        1000,
    )
    .await;

    let ok = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000201",
        &dims(Some(2), Some(100)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .expect("within the ceiling");
    assert_eq!(ok.reference.dimensions.calls, Some(2));

    // 9 more calls would exceed the 10-call ceiling (2 held).
    let err = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000201",
        &dims(Some(9), Some(100)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, BudgetError::Unavailable { .. }), "got: {err}");

    let denied: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM budget_reservations WHERE ceiling_id = $1 AND status = 'denied'",
    )
    .bind(&ceiling_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(denied, 1, "the denial is itself a recorded row");
}

/// Settlement with LOWER usage frees the difference; the actual usage is what holds.
#[tokio::test]
async fn settlement_frees_the_unused_remainder() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000202";
    let ceiling_id = ceiling(
        &pool,
        tenant,
        "thr_00000000-0000-7000-8000-000000000202",
        10,
        1000,
    )
    .await;

    let reservation = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000202",
        &dims(Some(5), Some(500)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .unwrap();

    let settlement = settle_reservation(
        &pool,
        &reservation.reference.reservation_id,
        &dims(Some(1), Some(50)),
        Utc::now(),
    )
    .await
    .unwrap()
    .expect("active reservation settles");
    assert!(!settlement.overrun);

    // 1 call + 50 tokens now hold: 9 more calls and 950 tokens must fit exactly.
    let again = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000202",
        &dims(Some(9), Some(950)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .expect("the unused remainder was freed");
    assert_eq!(again.reference.dimensions.calls, Some(9));
}

/// Release returns everything the reservation held.
#[tokio::test]
async fn release_frees_everything() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000203";
    let ceiling_id = ceiling(
        &pool,
        tenant,
        "thr_00000000-0000-7000-8000-000000000203",
        2,
        100,
    )
    .await;

    let reservation = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000203",
        &dims(Some(2), Some(100)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .unwrap();
    release_reservation(&pool, &reservation.reference.reservation_id, Utc::now())
        .await
        .unwrap();

    let again = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000203",
        &dims(Some(2), Some(100)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .expect("the full ceiling is available again");
    assert_eq!(again.reference.dimensions.calls, Some(2));
}

/// An expired reservation stops holding (the caller-supplied clock decides).
#[tokio::test]
async fn expired_reservations_stop_holding() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000204";
    let ceiling_id = ceiling(
        &pool,
        tenant,
        "thr_00000000-0000-7000-8000-000000000204",
        2,
        100,
    )
    .await;
    let at = Utc::now();

    create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000204",
        &dims(Some(2), Some(100)),
        Duration::seconds(60),
        at,
    )
    .await
    .unwrap();

    // 61 seconds later the hold has lapsed: the full ceiling is available.
    let again = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000204",
        &dims(Some(2), Some(100)),
        Duration::minutes(5),
        at + Duration::seconds(61),
    )
    .await
    .expect("an expired reservation must not hold");
    assert_eq!(again.reference.dimensions.calls, Some(2));
}

/// Settlement with usage ABOVE the reservation records the overrun — never clamps.
#[tokio::test]
async fn settlement_overrun_is_recorded_never_clamped() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000205";
    let ceiling_id = ceiling(
        &pool,
        tenant,
        "thr_00000000-0000-7000-8000-000000000205",
        10,
        1000,
    )
    .await;

    let reservation = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        "thr_00000000-0000-7000-8000-000000000205",
        &dims(Some(1), Some(100)),
        Duration::minutes(5),
        Utc::now(),
    )
    .await
    .unwrap();

    let settlement = settle_reservation(
        &pool,
        &reservation.reference.reservation_id,
        &dims(Some(2), Some(150)),
        Utc::now(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(settlement.overrun, "the estimate error is reported");
    assert_eq!(settlement.overrun_dimensions.calls, Some(1));
    assert_eq!(settlement.overrun_dimensions.input_tokens, Some(50));

    // Settling twice is a no-op (terminal row), never a double-charge.
    let again = settle_reservation(
        &pool,
        &reservation.reference.reservation_id,
        &dims(Some(2), Some(150)),
        Utc::now(),
    )
    .await
    .unwrap();
    assert!(again.is_none(), "a settled row is terminal");
}
