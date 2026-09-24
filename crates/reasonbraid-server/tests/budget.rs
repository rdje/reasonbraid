//! WP5 budget tests, server side (`PHASE-0.5.2`): the ceiling holds — reservations are
//! atomic against the currently-held amount, settlements free the unused remainder
//! (and record overruns, never clamp), releases return everything, expired
//! reservations stop holding, and denials are recorded rows. Run via
//! `scripts/run_pg_tests.sh` / the `pg-tests` CI job; skip offline.

#[path = "support/mod.rs"]
mod pg_test_support;

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
    let pool = pg_test_support::pool().await?;
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
    sqlx::query("DELETE FROM spend_breakers")
        .execute(&pool)
        .await
        .expect("purge spend_breakers");
    // The breaker's tenant FK needs a real tenant row for the breaker tests.
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name) \
         VALUES ('ten_00000000-0000-7000-8000-000000000000', 'budget-seed') \
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .expect("seed tenant");
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

/// THE `.3.2` acceptance: the per-tenant spend latch — the projected spend
/// (recorded + requested) crossing the threshold trips the breaker IN the
/// reservation transaction and refuses with the typed reason; while tripped
/// every NEW reservation is refused (the latch, not a recomputation); the
/// reset re-opens it; the threshold persists.
#[tokio::test]
async fn a_spend_breaker_trips_refuses_and_resets() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000000";
    let ceiling_id = ceiling(&pool, tenant, thread, 100, 0).await;

    // Arm: one call of recorded spend is the declared threshold.
    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::to_value(dims(Some(1), None)).expect("threshold serializes"))
        .execute(&pool)
        .await
        .expect("arm");

    // Below the threshold: 0 + 1 = 1 call — covered.
    create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), None),
        Duration::minutes(10),
        Utc::now(),
    )
    .await
    .expect("the first call holds");

    // Crossing: the held 1 + the requested 1 = 2 calls — the breaker trips
    // IN this transaction and refuses with the typed reason.
    let refused = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), None),
        Duration::minutes(10),
        Utc::now(),
    )
    .await;
    match refused {
        Err(BudgetError::Unavailable { detail }) => {
            assert!(
                detail.contains("circuit breaker tripped"),
                "the refusal names the breaker: {detail}"
            )
        }
        other => panic!("the crossing reservation must be refused: {other:?}"),
    }

    // The latch: tripped with the reason, refusing EVERYTHING new — even
    // though the recorded spend alone (1 call) is still under the threshold.
    let (tripped_at, reason): (Option<chrono::DateTime<Utc>>, Option<String>) = sqlx::query_as(
        "SELECT tripped_at, tripped_reason FROM spend_breakers WHERE tenant_id = $1",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .expect("breaker row");
    assert!(tripped_at.is_some(), "the breaker tripped");
    assert!(
        reason.as_deref().is_some_and(|r| r.contains("crossed")),
        "the trip reason names the crossing: {reason:?}"
    );
    let while_tripped = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), None),
        Duration::minutes(10),
        Utc::now(),
    )
    .await;
    match while_tripped {
        Err(BudgetError::Unavailable { detail }) => {
            assert!(
                detail.contains("tripped"),
                "a tripped breaker refuses everything new: {detail}"
            )
        }
        other => panic!("a tripped breaker refuses everything new: {other:?}"),
    }

    // The operator reset: the latch clears, the threshold persists. (The held
    // call is released first so the post-reset request stays under it — the
    // next test proves a reset breaker still re-trips on a crossing.)
    let held: String = sqlx::query_scalar(
        "SELECT reservation_id FROM budget_reservations WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .expect("the held reservation");
    release_reservation(&pool, &held, Utc::now())
        .await
        .expect("release");
    sqlx::query(
        "UPDATE spend_breakers SET tripped_at = NULL, tripped_reason = NULL WHERE tenant_id = $1",
    )
    .bind(tenant)
    .execute(&pool)
    .await
    .expect("reset");
    create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), None),
        Duration::minutes(10),
        Utc::now(),
    )
    .await
    .expect("after the reset a within-threshold reservation holds again");
}

/// After a reset, the threshold STILL gates: the crossing refusal trips again
/// (the latch is not a one-shot) — the second trip carries a fresh reason.
#[tokio::test]
async fn a_reset_breaker_trips_again_on_the_next_crossing() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000000";
    let ceiling_id = ceiling(&pool, tenant, thread, 100, 0).await;

    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::to_value(dims(Some(1), None)).expect("threshold serializes"))
        .execute(&pool)
        .await
        .expect("arm");

    // First trip: 0 + 2 calls crosses the 1-call threshold.
    assert!(
        matches!(
            create_reservation(
                &pool,
                &ceiling_id,
                tenant,
                thread,
                &dims(Some(2), None),
                Duration::minutes(10),
                Utc::now(),
            )
            .await,
            Err(BudgetError::Unavailable { .. })
        ),
        "the first crossing trips"
    );
    sqlx::query(
        "UPDATE spend_breakers SET tripped_at = NULL, tripped_reason = NULL WHERE tenant_id = $1",
    )
    .bind(tenant)
    .execute(&pool)
    .await
    .expect("reset");

    // Second trip after the reset: the latch re-arms.
    assert!(
        matches!(
            create_reservation(
                &pool,
                &ceiling_id,
                tenant,
                thread,
                &dims(Some(2), None),
                Duration::minutes(10),
                Utc::now(),
            )
            .await,
            Err(BudgetError::Unavailable { .. })
        ),
        "the reset breaker trips again on the next crossing"
    );
}

/// ⭐ `SIGNOFF-REPAIR.11.30` — THE PROOF CARRIES THE LEDGER'S OWN BYTES, and this
/// control can FAIL ON ANY HOST. `node_work`'s equivalent assertion compares two
/// instants that both come from `Utc::now()`, so on a host whose `CLOCK_REALTIME`
/// is microsecond-granular (macOS: 20,000 samples, zero nonzero sub-microsecond
/// digits) the truncation it is meant to catch cannot occur and the check is
/// unfalsifiable. Here `at` is CHOSEN with nanosecond digits, so `TIMESTAMPTZ`'s
/// microsecond truncation is guaranteed and the equality is a real question
/// everywhere. The window matters because the held-amount query stops counting an
/// active row at `expires_at > $2` against the STORED column: a reference carrying
/// the pre-truncation instant outlives the capacity it proves.
#[tokio::test]
async fn the_reference_carries_the_stored_instants_not_the_ones_bound_to_the_insert() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let tenant = "ten_00000000-0000-7000-8000-0000000011f0";
    let thread = "thr_00000000-0000-7000-8000-0000000011f0";
    let ceiling_id = ceiling(&pool, tenant, thread, 10, 1000).await;

    // A deliberate 310 ns tail — the exact remainder the runner reported — so the
    // row MUST differ from the bound value on every platform.
    let at = "2026-09-20T08:55:40.449715310Z"
        .parse::<chrono::DateTime<Utc>>()
        .expect("a nanosecond-precision instant");
    assert_ne!(
        at.timestamp_subsec_nanos() % 1_000,
        0,
        "the fixture must carry sub-microsecond digits or it proves nothing"
    );

    let reservation = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), Some(10)),
        Duration::minutes(10),
        at,
    )
    .await
    .expect("the ceiling covers one call");

    let (row_expires_at, row_created_at): (chrono::DateTime<Utc>, chrono::DateTime<Utc>) =
        sqlx::query_as(
            "SELECT expires_at, created_at FROM budget_reservations WHERE reservation_id = $1",
        )
        .bind(&reservation.reference.reservation_id)
        .fetch_one(&pool)
        .await
        .expect("the issued reservation row");

    assert_eq!(
        reservation.reference.expires_at, row_expires_at,
        "the reference's window is the ledger's own, not the instant bound to the insert"
    );
    assert_eq!(
        reservation.reference.issued_at, row_created_at,
        "the reference's issue instant is the ledger's own too"
    );
    assert_eq!(
        reservation.expires_at, row_expires_at,
        "the reservation's own window agrees with the row it wrote"
    );

    // THE POSITIVE CONTROL for the control: the fixture really is truncated by the
    // store, so the two assertions above had something to catch.
    assert_ne!(
        row_expires_at,
        at + Duration::minutes(10),
        "the store must truncate the bound instant, or this test cannot fail"
    );
    assert_eq!(
        row_expires_at.timestamp_subsec_nanos() % 1_000,
        0,
        "a stored TIMESTAMPTZ carries no sub-microsecond digits"
    );
}

/// Two settled reservations whose reported usage sums past `u64`
/// (`SIGNOFF-REPAIR.4.5.2`). Settlement takes the node's reported usage as it
/// is and records it in full (§14.3: never clamped), so the ledger's sums must
/// not wrap: a wrapped held sum reads SMALL and the ceiling lends again.
async fn two_settlements_past_u64(pool: &PgPool, tenant: &str, thread: &str) -> String {
    let ceiling_id = ceiling(pool, tenant, thread, 10, 100).await;
    // Both are held BEFORE either settles: once one settlement is recorded,
    // the ceiling (rightly) has no room for another reservation.
    let mut held = Vec::new();
    for _ in 0..2 {
        held.push(
            create_reservation(
                pool,
                &ceiling_id,
                tenant,
                thread,
                &dims(Some(1), Some(10)),
                Duration::minutes(10),
                Utc::now(),
            )
            .await
            .expect("room for the call"),
        );
    }
    let half = 1u64 << 63;
    for reservation in &held {
        settle_reservation(
            pool,
            &reservation.reference.reservation_id,
            &dims(Some(1), Some(half)),
            Utc::now(),
        )
        .await
        .expect("settled")
        .expect("the reservation was active");
    }
    ceiling_id
}

/// The ceiling's held sum overflows: admission refuses, naming the overflow,
/// rather than panicking (debug) or wrapping to a small sum and lending
/// (release).
#[tokio::test]
async fn a_held_sum_past_u64_refuses_rather_than_wrapping() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000206";
    let thread = "thr_00000000-0000-7000-8000-000000000206";
    let ceiling_id = two_settlements_past_u64(&pool, tenant, thread).await;

    let refused = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), Some(10)),
        Duration::minutes(10),
        Utc::now(),
    )
    .await;
    match refused {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("overflow") && detail.contains("ceiling"),
            "the refusal is the ceiling sum's overflow: {detail}"
        ),
        other => panic!("expected the overflow refusal, got {other:?}"),
    }
}

/// The breaker's tenant-wide spend overflows the same way, and it is summed
/// FIRST, so it is refused before the ceiling is read.
#[tokio::test]
async fn a_spend_sum_past_u64_refuses_rather_than_wrapping() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    // The seeded tenant: a breaker row references `tenants`.
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000207";
    let ceiling_id = two_settlements_past_u64(&pool, tenant, thread).await;
    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::to_value(dims(Some(1_000), Some(u64::MAX))).expect("threshold"))
        .execute(&pool)
        .await
        .expect("arm");

    let refused = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), Some(10)),
        Duration::minutes(10),
        Utc::now(),
    )
    .await;
    match refused {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("overflow") && detail.contains("spend"),
            "the refusal is the spend sum's overflow: {detail}"
        ),
        other => panic!("expected the overflow refusal, got {other:?}"),
    }
}

/// The breaker's PROJECTION overflows even when its spend alone does not: one
/// settlement of `u64::MAX` tokens is countable, and the request on top of it
/// is not.
#[tokio::test]
async fn a_projected_spend_past_u64_refuses_rather_than_wrapping() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000208";
    let ceiling_id = ceiling(&pool, tenant, thread, 10, 100).await;
    let held = create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), Some(10)),
        Duration::minutes(10),
        Utc::now(),
    )
    .await
    .expect("room for the call");
    settle_reservation(
        &pool,
        &held.reference.reservation_id,
        &dims(Some(1), Some(u64::MAX)),
        Utc::now(),
    )
    .await
    .expect("settled")
    .expect("the reservation was active");
    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::to_value(dims(Some(1_000), Some(u64::MAX))).expect("threshold"))
        .execute(&pool)
        .await
        .expect("arm");

    match create_reservation(
        &pool,
        &ceiling_id,
        tenant,
        thread,
        &dims(Some(1), Some(10)),
        Duration::minutes(10),
        Utc::now(),
    )
    .await
    {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("overflow") && detail.contains("spend"),
            "the refusal is the projection's overflow: {detail}"
        ),
        other => panic!("expected the overflow refusal, got {other:?}"),
    }
}

/// The backend of `job` when PostgreSQL reports it waiting on a lock `holder`
/// holds, running a statement containing `query`. A job that FINISHED, or a
/// timeout, is not evidence of a lock and answers `None` (`.4.3.4`'s method:
/// the wait is observed, never inferred from a sleep).
async fn waiter<T>(
    pool: &PgPool,
    holder: i32,
    query: &str,
    job: &tokio::task::JoinHandle<T>,
) -> Option<i32> {
    tokio::time::timeout(std::time::Duration::from_secs(4), async {
        loop {
            if job.is_finished() {
                return None;
            }
            let pid: Option<i32> = sqlx::query_scalar(
                "SELECT pid FROM pg_stat_activity WHERE datname = current_database() \
                 AND wait_event_type = 'Lock' AND query LIKE $1 \
                 AND $2 = ANY(pg_blocking_pids(pid)) ORDER BY pid LIMIT 1",
            )
            .bind(format!("%{query}%"))
            .bind(holder)
            .fetch_optional(pool)
            .await
            .expect("pg_stat_activity");
            if pid.is_some() {
                return pid;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap_or(None)
}

/// A transaction standing in for an admission in flight: it holds `lock` (a
/// `FOR UPDATE` on the row that admission decides against) and reports its
/// backend.
async fn in_flight(
    pool: &PgPool,
    lock: &str,
    key: &str,
) -> (sqlx::Transaction<'static, sqlx::Postgres>, i32) {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query(lock)
        .bind(key)
        .execute(&mut *tx)
        .await
        .expect("the in-flight lock");
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *tx)
        .await
        .expect("pid");
    (tx, pid)
}

/// `SIGNOFF-REPAIR.4.5.3` — admission serializes on its CEILING. While another
/// admission holds the ceiling, a second one WAITS, and once the first commits
/// its hold, the second sums it and is denied. Before this leaf the second read
/// the ceiling unlocked and summed the rows committed so far, so two concurrent
/// admissions could each see room for one call and both hold it.
#[tokio::test]
async fn a_second_admission_waits_for_the_ceiling_and_sees_the_first_hold() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000209";
    let thread = "thr_00000000-0000-7000-8000-000000000209";
    let ceiling_id = ceiling(&pool, tenant, thread, 1, 100).await;

    let (mut first, holder) = in_flight(
        &pool,
        "SELECT 1 FROM budget_ceilings WHERE ceiling_id = $1 FOR UPDATE",
        &ceiling_id,
    )
    .await;
    let second = {
        let (pool, ceiling_id) = (pool.clone(), ceiling_id.clone());
        tokio::spawn(async move {
            create_reservation(
                &pool,
                &ceiling_id,
                tenant,
                thread,
                &dims(Some(1), Some(10)),
                Duration::minutes(10),
                Utc::now(),
            )
            .await
        })
    };
    assert!(
        waiter(&pool, holder, "budget_ceilings", &second)
            .await
            .is_some(),
        "the second admission waits on the ceiling the first holds"
    );

    // The first admission's hold commits: the room for one call is taken.
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, expires_at, created_at) \
         VALUES ('res_first_in_flight', $1, $2, $3, $4, 'active', now() + interval '10 minutes', now())",
    )
    .bind(&ceiling_id)
    .bind(tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(1), Some(10))).expect("dims"))
    .execute(&mut *first)
    .await
    .expect("the first hold");
    first.commit().await.expect("the first admission commits");

    match second.await.expect("the second admission ran") {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("does not cover"),
            "the second is denied on the first's hold: {detail}"
        ),
        other => panic!("the second admission must see the first hold, got {other:?}"),
    }
}

/// The tenant's spend BREAKER serializes the same way: it sums every ceiling
/// of the tenant, so two admissions against DIFFERENT ceilings, which do not
/// share a ceiling lock, must still queue on the breaker.
#[tokio::test]
async fn a_second_admission_waits_for_the_breaker_and_sees_the_first_spend() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000210";
    let ceiling_id = ceiling(&pool, tenant, thread, 10, 1_000).await;
    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::to_value(dims(Some(1), Some(1_000))).expect("threshold"))
        .execute(&pool)
        .await
        .expect("arm");

    let (mut first, holder) = in_flight(
        &pool,
        "SELECT 1 FROM spend_breakers WHERE tenant_id = $1 FOR UPDATE",
        tenant,
    )
    .await;
    let second = {
        let (pool, ceiling_id) = (pool.clone(), ceiling_id.clone());
        tokio::spawn(async move {
            create_reservation(
                &pool,
                &ceiling_id,
                tenant,
                thread,
                &dims(Some(1), Some(10)),
                Duration::minutes(10),
                Utc::now(),
            )
            .await
        })
    };
    assert!(
        waiter(&pool, holder, "spend_breakers", &second)
            .await
            .is_some(),
        "the second admission waits on the breaker the first holds"
    );

    // The first admission's hold, on ANOTHER of the tenant's ceilings,
    // commits: the breaker's one call is spent.
    let other_thread = "thr_00000000-0000-7000-8000-000000000211";
    let other = "ceil_breaker_other";
    create_ceiling(
        &pool,
        other,
        tenant,
        other_thread,
        &dims(Some(10), Some(1_000)),
    )
    .await
    .expect("the tenant's other ceiling");
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, expires_at, created_at) \
         VALUES ('res_first_spend', $1, $2, $3, $4, 'active', now() + interval '10 minutes', now())",
    )
    .bind(other)
    .bind(tenant)
    .bind(other_thread)
    .bind(serde_json::to_value(dims(Some(1), Some(10))).expect("dims"))
    .execute(&mut *first)
    .await
    .expect("the first hold");
    first.commit().await.expect("the first admission commits");

    match second.await.expect("the second admission ran") {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("circuit breaker"),
            "the second trips the breaker on the first's spend: {detail}"
        ),
        other => panic!("the second admission must see the first spend, got {other:?}"),
    }
}

/// `SIGNOFF-REPAIR.4.5.6` — a breaker constrains only the dimensions its
/// threshold meters. Armed at one CALL, it lets a first request through even
/// though that request also asks for tokens the threshold does not name, and
/// trips on the second call. Before this leaf it refused the first request:
/// `covers` fails closed on an unmetered dimension, which is right for a
/// ceiling and made a partial breaker trip on any work item at all.
#[tokio::test]
async fn a_breaker_constrains_only_the_dimensions_it_meters() {
    let _guard = budget_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000000";
    let thread = "thr_00000000-0000-7000-8000-000000000212";
    let ceiling_id = ceiling(&pool, tenant, thread, 10, 1_000).await;
    sqlx::query("INSERT INTO spend_breakers (tenant_id, threshold) VALUES ($1, $2)")
        .bind(tenant)
        .bind(serde_json::json!({ "calls": 1 }))
        .execute(&pool)
        .await
        .expect("arm on calls alone");

    let one_call = dims(Some(1), Some(10));
    let reserve = || {
        create_reservation(
            &pool,
            &ceiling_id,
            tenant,
            thread,
            &one_call,
            Duration::minutes(10),
            Utc::now(),
        )
    };
    reserve()
        .await
        .expect("one call is within a one-call threshold, whatever its tokens");
    match reserve().await {
        Err(BudgetError::Unavailable { detail }) => assert!(
            detail.contains("circuit breaker"),
            "the second call crosses the threshold: {detail}"
        ),
        other => panic!("the second call must trip the breaker, got {other:?}"),
    }
}
