//! Qualification of the exact private guard source before production integration.
//! All database fixtures require the supervised runner's disposable ownership proof.

#[path = "support/mod.rs"]
mod pg_test_support;
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use reasonbraid_core::TenantId;
use sqlx::PgPool;
use tenant_transaction::{
    transact, transact_with_error, transact_with_limits, GuardError, GuardMode, Limits,
};
use tokio::sync::{oneshot, Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::timeout;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

// Deliberately no From<sqlx::Error>: the generic runner must preserve the
// transaction phase through GuardError instead of requiring unrelated conversions.
#[derive(Debug)]
enum TypedFailure {
    Refused { reason: &'static str, backend: i32 },
    Transaction(GuardError),
}

impl From<GuardError> for TypedFailure {
    fn from(error: GuardError) -> Self {
        Self::Transaction(error)
    }
}

async fn fixture() -> Option<(PgPool, MutexGuard<'static, ()>)> {
    let guard = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guard_probe (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    Some((pool, guard))
}

fn limits(lock_ms: u64, statement_ms: u64, total_ms: u64) -> Limits {
    Limits::shortened(
        Duration::from_millis(lock_ms),
        Duration::from_millis(statement_ms),
        Duration::from_millis(total_ms),
    )
    .unwrap()
}

async fn seed(pool: &PgPool, tenant: TenantId) {
    transact(pool, &[(tenant, GuardMode::Shared)], |_| {
        Box::pin(async { Ok(()) })
    })
    .await
    .unwrap();
}

struct Holder {
    entered: oneshot::Receiver<i32>,
    release: oneshot::Sender<()>,
    job: JoinHandle<Result<(), GuardError>>,
}

fn start_holder(pool: &PgPool, guards: Vec<(TenantId, GuardMode)>) -> Holder {
    let pool = pool.clone();
    let (entered_tx, entered) = oneshot::channel();
    let (release, release_rx) = oneshot::channel();
    let required = guards.clone();
    let job = tokio::spawn(async move {
        transact(&pool, &guards, move |tx| {
            Box::pin(async move {
                for &(tenant, mode) in &required {
                    tx.require_scope(tenant, mode)?;
                }
                let (tenant, mode) = required[0];
                let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(tx.connection(tenant, mode)?)
                    .await?;
                entered_tx.send(pid).unwrap();
                release_rx.await.unwrap();
                Ok(())
            })
        })
        .await
    });
    Holder {
        entered,
        release,
        job,
    }
}

async fn enter(holder: &mut Holder) -> i32 {
    timeout(Duration::from_secs(4), &mut holder.entered)
        .await
        .unwrap()
        .unwrap()
}

async fn release(holder: Holder) {
    holder.release.send(()).unwrap();
    timeout(Duration::from_secs(4), holder.job)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

/// Observe real lock waits in this owned database. An uncompleted task or sleep
/// alone is never evidence that the requested database lock was reached.
async fn blocked(pool: &PgPool, holder: i32, count: usize) -> Vec<i32> {
    blocked_by_any(pool, &[holder], count).await
}

async fn blocked_by_any(pool: &PgPool, holders: &[i32], count: usize) -> Vec<i32> {
    let observed = timeout(Duration::from_secs(4), async {
        loop {
            let rows: Vec<i32> = sqlx::query_scalar(
                "WITH RECURSIVE waiting AS MATERIALIZED (\
                     SELECT pid, pg_blocking_pids(pid) AS blockers FROM pg_stat_activity \
                     WHERE datname = current_database() AND wait_event_type = 'Lock' \
                     AND query LIKE '%tenant_authority_guards%'\
                 ), blocked(pid) AS (\
                     SELECT pid FROM waiting WHERE blockers && $1::int[] \
                     UNION SELECT w.pid FROM waiting w JOIN blocked b ON b.pid = ANY(w.blockers)\
                 ) SELECT pid FROM blocked ORDER BY pid",
            )
            .bind(holders)
            .fetch_all(pool)
            .await
            .unwrap();
            if rows.len() >= count {
                return rows;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    match observed {
        Ok(rows) => rows,
        Err(_) => {
            let graph: serde_json::Value = sqlx::query_scalar(
                "SELECT coalesce(jsonb_agg(jsonb_build_object('pid', pid, 'state', state, \
                 'wait_type', wait_event_type, 'wait', wait_event, 'blockers', pg_blocking_pids(pid), \
                 'query', query)), '[]'::jsonb) FROM pg_stat_activity \
                 WHERE datname = current_database() AND pid <> pg_backend_pid()"
            ).fetch_one(pool).await.unwrap();
            panic!("expected {count} guard waiters depending on {holders:?}; graph={graph}");
        }
    }
}

async fn probe_count(pool: &PgPool, tenant: TenantId) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM guard_probe WHERE key = $1")
        .bind(tenant.to_string())
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn backend_exited(pool: &PgPool, pid: i32) {
    timeout(Duration::from_secs(4), async {
        loop {
            let absent: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid = $1)",
            )
            .bind(pid)
            .fetch_one(pool)
            .await
            .unwrap();
            if absent {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("discarded backend exits before cleanup is claimed");
}

#[test]
fn internal_limits_refuse_unbounded_inverted_or_submillisecond_values() {
    for (lock, statement, total) in [
        (0, 1, 1),
        (2, 1, 2),
        (1, 3, 2),
        (5001, 6000, 7000),
        (1, 10001, 12000),
        (1, 2, 15001),
    ] {
        assert!(matches!(
            Limits::shortened(
                Duration::from_millis(lock),
                Duration::from_millis(statement),
                Duration::from_millis(total)
            ),
            Err(GuardError::InvalidLimits)
        ));
    }
    assert!(matches!(
        Limits::shortened(
            Duration::from_nanos(1_000_001),
            Duration::from_millis(2),
            Duration::from_millis(3)
        ),
        Err(GuardError::InvalidLimits)
    ));
    limits(5000, 10000, 15000);
    limits(1, 1, 1);
}

#[tokio::test]
async fn compatible_shared_holders_both_exclude_an_exclusive_waiter() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    seed(&pool, tenant).await;
    let mut first = start_holder(&pool, vec![(tenant, GuardMode::Shared)]);
    let first_pid = enter(&mut first).await;
    let mut second = start_holder(&pool, vec![(tenant, GuardMode::Shared)]);
    let second_pid = enter(&mut second).await;
    assert_ne!(first_pid, second_pid);
    let mut writer = start_holder(&pool, vec![(tenant, GuardMode::Exclusive)]);
    let waiter = blocked_by_any(&pool, &[first_pid, second_pid], 1).await[0];
    let blockers: Vec<i32> = sqlx::query_scalar("SELECT pg_blocking_pids($1)")
        .bind(waiter)
        .fetch_one(&pool)
        .await
        .unwrap();
    // PostgreSQL may wait on one MultiXact member at a time. Release the
    // actually reported member first, then observe the remaining exclusion.
    if blockers.contains(&first_pid) {
        release(first).await;
        blocked(&pool, second_pid, 1).await;
        assert!(!writer.job.is_finished());
        release(second).await;
    } else {
        assert!(blockers.contains(&second_pid));
        release(second).await;
        blocked(&pool, first_pid, 1).await;
        assert!(!writer.job.is_finished());
        release(first).await;
    }
    enter(&mut writer).await;
    release(writer).await;
    pool.close().await;
}

#[tokio::test]
async fn exclusive_holder_blocks_both_modes_while_another_tenant_progresses() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    seed(&pool, tenant).await;
    let mut holder = start_holder(&pool, vec![(tenant, GuardMode::Exclusive)]);
    let pid = enter(&mut holder).await;
    let mut shared = start_holder(&pool, vec![(tenant, GuardMode::Shared)]);
    let mut exclusive = start_holder(&pool, vec![(tenant, GuardMode::Exclusive)]);
    blocked(&pool, pid, 2).await;
    timeout(Duration::from_secs(2), seed(&pool, TenantId::new()))
        .await
        .unwrap();
    assert!(!shared.job.is_finished() && !exclusive.job.is_finished());
    release(holder).await;
    // Either waiter may be first in PostgreSQL's queue. Consume that admission
    // before releasing it, then consume the other; do not assume spawn order.
    tokio::select! {
        result = &mut shared.entered => {
            result.unwrap(); release(shared).await;
            enter(&mut exclusive).await; release(exclusive).await;
        }
        result = &mut exclusive.entered => {
            result.unwrap(); release(exclusive).await;
            enter(&mut shared).await; release(shared).await;
        }
    }
    pool.close().await;
}

#[tokio::test]
async fn full_key_order_and_duplicate_strongest_mode_prevent_lock_inversion() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let mut keys = [TenantId::new(), TenantId::new()];
    keys.sort();
    let [a, b] = keys;
    seed(&pool, a).await;
    seed(&pool, b).await;
    let mut b_holder = start_holder(&pool, vec![(b, GuardMode::Exclusive)]);
    let b_pid = enter(&mut b_holder).await;
    let mut multi = start_holder(
        &pool,
        vec![
            (b, GuardMode::Shared),
            (a, GuardMode::Shared),
            (a, GuardMode::Exclusive),
        ],
    );
    let multi_pid = blocked(&pool, b_pid, 1).await[0];
    // The opposite input order must already hold A exclusively while waiting on
    // B. A missing sort or failure to strengthen A would let this reader enter.
    let mut a_reader = start_holder(&pool, vec![(a, GuardMode::Shared)]);
    blocked(&pool, multi_pid, 1).await;
    release(b_holder).await;
    assert_eq!(enter(&mut multi).await, multi_pid);
    blocked(&pool, multi_pid, 1).await;
    release(multi).await;
    enter(&mut a_reader).await;
    release(a_reader).await;
    pool.close().await;
}

#[tokio::test]
async fn concurrent_first_use_commits_one_anchor_without_creating_authority() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    let mut first = start_holder(&pool, vec![(tenant, GuardMode::Shared)]);
    let pid = enter(&mut first).await;
    let mut second = start_holder(&pool, vec![(tenant, GuardMode::Shared)]);
    blocked(&pool, pid, 1).await; // unique-key insertion waits on uncommitted first use
    release(first).await;
    enter(&mut second).await;
    release(second).await;
    for (table, expected) in [
        ("tenant_authority_guards", 1_i64),
        ("tenants", 0),
        ("enrollment_boundaries", 0),
        ("authority_grants", 0),
        ("authorization_records", 0),
    ] {
        let count: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {table} WHERE tenant_id = $1"
        ))
        .bind(tenant.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, expected, "{table}");
    }
    pool.close().await;
}

#[tokio::test]
async fn scope_errors_and_callback_sql_errors_rollback_but_domain_refusals_commit() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    let foreign = TenantId::new();
    for guards in [vec![], vec![(tenant, GuardMode::Shared); 9]] {
        let result = transact(&pool, &guards, |_| {
            Box::pin(async { panic!("invalid scope reached body") })
        })
        .await;
        assert!(matches!(result, Err::<(), _>(GuardError::InvalidGuards)));
    }
    let result = transact(&pool, &[(tenant, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            assert!(matches!(
                tx.connection(foreign, GuardMode::Shared),
                Err(GuardError::ScopeMismatch)
            ));
            assert!(matches!(
                tx.connection(tenant, GuardMode::Exclusive),
                Err(GuardError::ScopeMismatch)
            ));
            sqlx::query("INSERT INTO guard_probe VALUES ($1, 'uncommitted')")
                .bind(tenant.to_string())
                .execute(tx.connection(tenant, GuardMode::Shared)?)
                .await?;
            Err::<(), _>(sqlx::Error::Protocol("original callback error".into()).into())
        })
    })
    .await;
    assert!(
        matches!(result, Err(GuardError::Storage(sqlx::Error::Protocol(ref message))) if message == "original callback error")
    );
    seed(&pool, tenant).await; // observes release/rollback before the next acquisition
    assert_eq!(probe_count(&pool, tenant).await, 0);
    let result = transact(&pool, &[(tenant, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            sqlx::query("INSERT INTO guard_probe VALUES ($1, 'recorded refusal')")
                .bind(tenant.to_string())
                .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                .await?;
            Ok(Err::<(), _>("domain denied"))
        })
    })
    .await
    .unwrap();
    assert_eq!(result, Err("domain denied"));
    assert_eq!(probe_count(&pool, tenant).await, 1);
    let poisoned = TenantId::new();
    let result = transact(&pool, &[(poisoned, GuardMode::Shared)], move |tx| {
        Box::pin(async move {
            sqlx::query("INSERT INTO guard_probe VALUES ($1, 'must roll back')")
                .bind(poisoned.to_string())
                .execute(tx.connection(poisoned, GuardMode::Shared)?)
                .await?;
            let failure = sqlx::query("SELECT 1 / 0")
                .execute(tx.connection(poisoned, GuardMode::Shared)?)
                .await;
            assert!(failure.is_err());
            Ok(()) // the pre-commit health check must reject swallowed SQL failure
        })
    })
    .await;
    assert!(
        matches!(result, Err(GuardError::Storage(sqlx::Error::Database(ref error))) if error.code().as_deref() == Some("25P02"))
    );
    seed(&pool, poisoned).await;
    assert_eq!(probe_count(&pool, poisoned).await, 0);
    pool.close().await;
}

#[tokio::test]
async fn cancellation_rolls_back_first_use_and_protected_rows_before_reacquisition() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    let (entered_tx, entered) = oneshot::channel();
    let worker_pool = pool.clone();
    let job = tokio::spawn(async move {
        transact(&worker_pool, &[(tenant, GuardMode::Exclusive)], move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'cancelled')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                entered_tx.send(pid).unwrap();
                std::future::pending::<()>().await;
                Ok(())
            })
        })
        .await
    });
    let pid = timeout(Duration::from_secs(4), entered)
        .await
        .unwrap()
        .unwrap();
    job.abort();
    assert!(job.await.unwrap_err().is_cancelled());
    backend_exited(&pool, pid).await;
    let anchors: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tenant_authority_guards WHERE tenant_id = $1")
            .bind(tenant.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(anchors, 0, "cancelled first-use anchor rolls back too");
    transact(&pool, &[(tenant, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM guard_probe WHERE key = $1")
                .bind(tenant.to_string())
                .fetch_one(tx.connection(tenant, GuardMode::Exclusive)?)
                .await?;
            assert_eq!(count, 0);
            Ok(())
        })
    })
    .await
    .unwrap();
    assert_eq!(probe_count(&pool, tenant).await, 0);
    pool.close().await;
}

#[tokio::test]
async fn read_committed_and_database_clock_follow_guard_and_later_statement_waits() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    seed(&pool, tenant).await;
    sqlx::query("INSERT INTO guard_probe VALUES ($1, 'old')")
        .bind(tenant.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let (ready_tx, ready) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let writer_pool = pool.clone();
    let writer = tokio::spawn(async move {
        transact(&writer_pool, &[(tenant, GuardMode::Exclusive)], move |tx| {
            Box::pin(async move {
                sqlx::query("UPDATE guard_probe SET value = 'committed' WHERE key = $1")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                ready_tx.send(pid).unwrap();
                release_rx.await.unwrap();
                Ok(())
            })
        })
        .await
    });
    let pid = timeout(Duration::from_secs(4), ready)
        .await
        .unwrap()
        .unwrap();
    let reader_pool = pool.clone();
    let reader = tokio::spawn(async move {
        transact(&reader_pool, &[(tenant, GuardMode::Shared)], move |tx| Box::pin(async move {
            let actual: (String, String, DateTime<Utc>) = sqlx::query_as(
                "SELECT value, current_setting('transaction_isolation'), transaction_timestamp() FROM guard_probe WHERE key = $1"
            ).bind(tenant.to_string()).fetch_one(tx.connection(tenant, GuardMode::Shared)?).await?;
            sqlx::query("SELECT pg_sleep(0.025)").execute(tx.connection(tenant, GuardMode::Shared)?).await?;
            let before_evaluation: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(tx.connection(tenant, GuardMode::Shared)?).await?;
            let evaluated_at = tx.database_now().await?;
            Ok((actual, before_evaluation, evaluated_at))
        })).await
    });
    blocked(&pool, pid, 1).await;
    let release_at: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    release_tx.send(()).unwrap();
    writer.await.unwrap().unwrap();
    let ((value, isolation, began), before_evaluation, evaluated_at) =
        reader.await.unwrap().unwrap();
    assert_eq!(value, "committed");
    assert_eq!(isolation, "read committed");
    assert!(
        began < release_at && release_at < before_evaluation && before_evaluation <= evaluated_at
    );
    pool.close().await;
}

#[tokio::test]
async fn lock_and_statement_timeouts_preserve_sqlstate_and_release_guards() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    seed(&pool, tenant).await;
    let mut holder = start_holder(&pool, vec![(tenant, GuardMode::Exclusive)]);
    let pid = enter(&mut holder).await;
    let waiter_pool = pool.clone();
    let entered = Arc::new(AtomicBool::new(false));
    let called = entered.clone();
    let waiter = tokio::spawn(async move {
        transact_with_limits(
            &waiter_pool,
            &[(tenant, GuardMode::Shared)],
            limits(500, 1000, 2000),
            move |_| {
                Box::pin(async move {
                    called.store(true, Ordering::SeqCst);
                    Ok(())
                })
            },
        )
        .await
    });
    blocked(&pool, pid, 1).await;
    let result = waiter.await.unwrap();
    assert!(
        matches!(result, Err(GuardError::Storage(sqlx::Error::Database(ref error))) if error.code().as_deref() == Some("55P03"))
    );
    assert!(!entered.load(Ordering::SeqCst));
    release(holder).await;
    let result = transact_with_limits(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        limits(100, 200, 1000),
        move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'statement timeout')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                sqlx::query("SELECT pg_sleep(2)")
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                Ok(())
            })
        },
    )
    .await;
    assert!(
        matches!(result, Err(GuardError::Storage(sqlx::Error::Database(ref error))) if error.code().as_deref() == Some("57014"))
    );
    seed(&pool, tenant).await;
    assert_eq!(probe_count(&pool, tenant).await, 0);
    pool.close().await;
}

#[tokio::test]
async fn whole_deadline_bounds_pool_acquisition_and_callback_work() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let mut occupied = Vec::new();
    for _ in 0..10 {
        occupied.push(pool.acquire().await.unwrap());
    }
    let tenant = TenantId::new();
    let result = transact_with_limits(
        &pool,
        &[(tenant, GuardMode::Shared)],
        limits(10, 50, 100),
        |_| Box::pin(async { panic!("pool exhausted: callback must not run") }),
    )
    .await;
    assert!(matches!(result, Err::<(), _>(GuardError::Deadline)));
    drop(occupied);
    let result = transact_with_limits(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        limits(10, 50, 100),
        move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'body deadline')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await?;
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            })
        },
    )
    .await;
    assert!(matches!(result, Err(GuardError::Deadline)));
    seed(&pool, tenant).await;
    assert_eq!(probe_count(&pool, tenant).await, 0);
    pool.close().await;
}

#[tokio::test]
async fn local_limits_reset_on_the_same_physical_connection() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let mut occupied = Vec::new();
    for _ in 0..9 {
        occupied.push(pool.acquire().await.unwrap());
    }
    let mut connection = pool.acquire().await.unwrap();
    let before: (i32, String, String) = sqlx::query_as(
        "SELECT pg_backend_pid(), current_setting('lock_timeout'), current_setting('statement_timeout')"
    ).fetch_one(&mut *connection).await.unwrap();
    drop(connection);
    let tenant = TenantId::new();
    let during: (i32, String, String) = transact_with_limits(&pool, &[(tenant, GuardMode::Shared)], limits(100, 200, 1000), move |tx| Box::pin(async move {
        Ok(sqlx::query_as("SELECT pg_backend_pid(), current_setting('lock_timeout'), current_setting('statement_timeout')")
            .fetch_one(tx.connection(tenant, GuardMode::Shared)?).await?)
    })).await.unwrap();
    assert_eq!(during, (before.0, "100ms".into(), "200ms".into()));
    let mut connection = pool.acquire().await.unwrap();
    let after: (i32, String, String) = sqlx::query_as(
        "SELECT pg_backend_pid(), current_setting('lock_timeout'), current_setting('statement_timeout')"
    ).fetch_one(&mut *connection).await.unwrap();
    assert_eq!(before, after);
    drop(connection);
    drop(occupied);
    pool.close().await;
}

#[tokio::test]
async fn cancelled_begin_never_returns_an_open_transaction_to_the_pool() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let mut occupied = Vec::new();
    for _ in 0..9 {
        occupied.push(pool.acquire().await.unwrap());
    }
    let mut connection = pool.acquire().await.unwrap();
    let old_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    drop(connection);
    let result = tenant_transaction::transact_with_delayed_begin(&pool, TenantId::new()).await;
    assert!(matches!(result, Err(GuardError::Deadline)));
    let mut connection = pool.acquire().await.unwrap();
    let new_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM pg_stat_activity WHERE pid = $1")
        .bind(new_pid)
        .fetch_one(&mut *occupied[0])
        .await
        .unwrap();
    // Restore before asserting even on the pre-repair baseline.
    sqlx::query("ROLLBACK")
        .execute(&mut *connection)
        .await
        .unwrap();
    drop(connection);
    drop(occupied);
    backend_exited(&pool, old_pid).await;
    pool.close().await;
    assert_eq!(
        state, "idle",
        "cancelled BEGIN reused backend {old_pid} as {new_pid}"
    );
    assert_ne!(
        new_pid, old_pid,
        "unconfirmed transaction setup is not reusable"
    );
}

#[tokio::test]
async fn typed_deferred_commit_failure_never_returns_a_success_value_or_partial_effect() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    sqlx::raw_sql("CREATE TABLE guard_commit_parent (key TEXT PRIMARY KEY); \
        CREATE TABLE guard_commit_child (key TEXT PRIMARY KEY REFERENCES guard_commit_parent(key) DEFERRABLE INITIALLY DEFERRED)")
        .execute(&pool).await.unwrap();
    let tenant = TenantId::new();
    let result: Result<_, TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        Limits::default(),
        move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_commit_child VALUES ($1)")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await
                    .map_err(GuardError::Storage)?;
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'uncommitted')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await
                    .map_err(GuardError::Storage)?;
                Ok("must not return this receipt")
            })
        },
    )
    .await;
    assert!(
        matches!(result, Err(TypedFailure::Transaction(GuardError::Commit(sqlx::Error::Database(ref error)))) if error.code().as_deref() == Some("23503"))
    );
    seed(&pool, tenant).await;
    assert_eq!(probe_count(&pool, tenant).await, 0);
    let children: i64 = sqlx::query_scalar("SELECT count(*) FROM guard_commit_child")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(children, 0);
    pool.close().await;
}

#[tokio::test]
async fn typed_commit_deadline_reports_uncertainty_even_when_the_database_later_commits() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    sqlx::raw_sql(
        "CREATE TABLE guard_commit_wait (key TEXT PRIMARY KEY); \
        CREATE FUNCTION guard_commit_pause() RETURNS trigger LANGUAGE plpgsql AS $$ \
        BEGIN PERFORM pg_sleep(1.5); RETURN NEW; END $$; \
        CREATE CONSTRAINT TRIGGER guard_commit_pause AFTER INSERT ON guard_commit_wait \
        DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION guard_commit_pause()",
    )
    .execute(&pool)
    .await
    .unwrap();
    let tenant = TenantId::new();
    let worker_pool = pool.clone();
    let (ready_tx, ready) = oneshot::channel();
    let job = tokio::spawn(async move {
        transact_with_error(
            &worker_pool,
            &[(tenant, GuardMode::Exclusive)],
            limits(500, 2000, 2000),
            move |tx| {
                Box::pin(async move {
                    sqlx::query("INSERT INTO guard_commit_wait VALUES ($1)")
                        .bind(tenant.to_string())
                        .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                        .await
                        .map_err(GuardError::Storage)?;
                    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                        .fetch_one(tx.connection(tenant, GuardMode::Exclusive)?)
                        .await
                        .map_err(GuardError::Storage)?;
                    ready_tx.send(pid).unwrap();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Ok::<_, TypedFailure>("unconfirmed receipt")
                })
            },
        )
        .await
    });
    let pid = timeout(Duration::from_secs(4), ready)
        .await
        .unwrap()
        .unwrap();
    timeout(Duration::from_secs(4), async {
        loop {
            let committing: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid = $1 AND query = 'COMMIT' AND wait_event = 'PgSleep')"
            ).bind(pid).fetch_one(&pool).await.unwrap();
            if committing { break; }
            assert!(!job.is_finished(), "must observe the actual deferred COMMIT wait");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.unwrap();
    assert!(matches!(
        job.await.unwrap(),
        Err(TypedFailure::Transaction(GuardError::CommitDeadline))
    ));
    // Resolve this test's uncertainty through authoritative readback, never an
    // automatic retry. The original COMMIT can succeed after the runner returns.
    timeout(Duration::from_secs(4), async {
        loop {
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM guard_commit_wait WHERE key = $1")
                    .bind(tenant.to_string())
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            if count == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    seed(&pool, tenant).await;
    pool.close().await;
}

#[tokio::test]
async fn typed_domain_error_rolls_back_protected_work_and_first_use_before_recovery() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    let result: Result<(), TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        Limits::default(),
        move |tx| {
            Box::pin(async move {
                let conn = tx.connection(tenant, GuardMode::Exclusive)?;
                let backend = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(GuardError::Storage)?;
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'provisional domain effect')")
                    .bind(tenant.to_string())
                    .execute(&mut *conn)
                    .await
                    .map_err(GuardError::Storage)?;
                Err(TypedFailure::Refused {
                    reason: "policy refuses staged effect",
                    backend,
                })
            })
        },
    )
    .await;
    let backend = match result {
        Err(TypedFailure::Refused {
            reason: "policy refuses staged effect",
            backend,
        }) => backend,
        other => panic!("typed refusal payload was not preserved: {other:?}"),
    };
    backend_exited(&pool, backend).await;
    assert_eq!(probe_count(&pool, tenant).await, 0);
    let anchors: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tenant_authority_guards WHERE tenant_id = $1")
            .bind(tenant.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(anchors, 0);
    let recovered: Result<_, TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        Limits::default(),
        move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'confirmed recovery')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await
                    .map_err(GuardError::Storage)?;
                Ok("confirmed recovery receipt")
            })
        },
    )
    .await;
    assert_eq!(recovered.unwrap(), "confirmed recovery receipt");
    assert_eq!(probe_count(&pool, tenant).await, 1);
    pool.close().await;
}

#[tokio::test]
async fn typed_transaction_errors_preserve_invalid_scope_sql_cause_and_deadlines() {
    let Some((pool, _guard)) = fixture().await else {
        return;
    };
    let tenant = TenantId::new();
    let invalid: Result<(), TypedFailure> =
        transact_with_error(&pool, &[], Limits::default(), |_| {
            Box::pin(async { panic!("invalid guards reached body") })
        })
        .await;
    assert!(matches!(
        invalid,
        Err(TypedFailure::Transaction(GuardError::InvalidGuards))
    ));
    let scope: Result<(), TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Shared)],
        Limits::default(),
        move |tx| {
            Box::pin(async move {
                tx.require_scope(tenant, GuardMode::Exclusive)?;
                Ok(())
            })
        },
    )
    .await;
    assert!(matches!(
        scope,
        Err(TypedFailure::Transaction(GuardError::ScopeMismatch))
    ));
    for swallowed in [false, true] {
        let sql: Result<(), TypedFailure> = transact_with_error(
            &pool,
            &[(tenant, GuardMode::Exclusive)],
            Limits::default(),
            move |tx| {
                Box::pin(async move {
                    let error = sqlx::query("SELECT 1 / 0")
                        .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                        .await
                        .expect_err("fixed division fault");
                    if swallowed {
                        Ok(())
                    } else {
                        Err(GuardError::Storage(error).into())
                    }
                })
            },
        )
        .await;
        let expected = if swallowed { "25P02" } else { "22012" };
        assert!(
            matches!(sql, Err(TypedFailure::Transaction(GuardError::Storage(sqlx::Error::Database(ref error)))) if error.code().as_deref() == Some(expected)),
            "{sql:?}"
        );
    }
    let mut occupied = Vec::new();
    for _ in 0..10 {
        occupied.push(pool.acquire().await.unwrap());
    }
    let acquisition: Result<(), TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Shared)],
        limits(10, 50, 100),
        |_| Box::pin(async { panic!("exhausted pool reached body") }),
    )
    .await;
    drop(occupied);
    assert!(matches!(
        acquisition,
        Err(TypedFailure::Transaction(GuardError::Deadline))
    ));
    let body: Result<(), TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Exclusive)],
        limits(10, 50, 100),
        move |tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO guard_probe VALUES ($1, 'typed body deadline')")
                    .bind(tenant.to_string())
                    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
                    .await
                    .map_err(GuardError::Storage)?;
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            })
        },
    )
    .await;
    assert!(matches!(
        body,
        Err(TypedFailure::Transaction(GuardError::Deadline))
    ));
    seed(&pool, tenant).await;
    assert_eq!(probe_count(&pool, tenant).await, 0);
    pool.close().await;
    let closed: Result<(), TypedFailure> = transact_with_error(
        &pool,
        &[(tenant, GuardMode::Shared)],
        Limits::default(),
        |_| Box::pin(async { panic!("closed pool reached body") }),
    )
    .await;
    assert!(matches!(
        closed,
        Err(TypedFailure::Transaction(GuardError::Storage(
            sqlx::Error::PoolClosed
        )))
    ));
}
