//! Tenant coordination and transaction lifetime, not authorization policy.
//!
//! Declare every tenant whose authority or domain state will be used, and the
//! strongest required mode, before accessing that state. A minimal foreign-ID
//! existence probe may identify a binding refusal; it must not decode/evaluate
//! foreign policy or mutate foreign state. The callback must validate authority
//! and bind effect SQL to the guarded tenant. It must not issue transaction-control SQL, alter the
//! local limits, delete anchors, or perform external work. The borrowed connection
//! supports existing SQLx helpers; it cannot mechanically inspect their SQL.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::future::BoxFuture;
use reasonbraid_core::TenantId;
use sqlx::pool::PoolConnection;
use sqlx::{Connection, PgConnection, PgPool, Postgres, Transaction};

const MAX_GUARDS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GuardMode {
    Shared,
    Exclusive,
}

/// Failure of a tenant authority transaction. A commit error or acknowledgment
/// deadline leaves the outcome unconfirmed; callers must not infer rollback or
/// automatically retry. This is exported as `AuthorityTransactionError` while
/// the transaction/guard construction APIs remain private.
#[derive(Debug)]
#[non_exhaustive]
pub enum GuardError {
    InvalidGuards,
    InvalidLimits,
    ScopeMismatch,
    Storage(sqlx::Error),
    Deadline,
    /// Commit was attempted. The caller must not infer rollback or retry.
    Commit(sqlx::Error),
    /// The commit acknowledgment did not arrive within the operation deadline.
    CommitDeadline,
}

impl fmt::Display for GuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidGuards => "invalid tenant authority guard set",
            Self::InvalidLimits => "invalid tenant authority transaction limits",
            Self::ScopeMismatch => "tenant authority guard scope or mode mismatch",
            Self::Storage(_) => "tenant authority transaction storage failure",
            Self::Deadline => "tenant authority transaction deadline exceeded",
            Self::Commit(_) => "tenant authority commit outcome unconfirmed",
            Self::CommitDeadline => "tenant authority commit acknowledgment deadline exceeded",
        })
    }
}

impl std::error::Error for GuardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) | Self::Commit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for GuardError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    lock: Duration,
    statement: Duration,
    total: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            lock: Duration::from_secs(5),
            statement: Duration::from_secs(10),
            total: Duration::from_secs(15),
        }
    }
}

impl Limits {
    /// Internal policies may shorten, but never enlarge, the production ceilings.
    pub(crate) fn shortened(
        lock: Duration,
        statement: Duration,
        total: Duration,
    ) -> Result<Self, GuardError> {
        let caps = Self::default();
        if lock < Duration::from_millis(1)
            || lock > statement
            || statement > total
            || lock > caps.lock
            || statement > caps.statement
            || total > caps.total
            || [lock, statement, total]
                .iter()
                .any(|duration| duration.subsec_nanos() % 1_000_000 != 0)
        {
            return Err(GuardError::InvalidLimits);
        }
        Ok(Self {
            lock,
            statement,
            total,
        })
    }
}

/// Own the connection before awaiting BEGIN. SQLx 0.8.6's transaction Drop guard
/// is not yet published during that await; its internal rollback depth is zero.
/// A cancelled setup must therefore close the connection instead of pooling an
/// unacknowledged server transaction. Only a confirmed commit permits reuse.
struct ConnectionLease {
    connection: PoolConnection<Postgres>,
    reusable: bool,
}

impl Drop for ConnectionLease {
    fn drop(&mut self) {
        if !self.reusable {
            self.connection.close_on_drop();
        }
    }
}

pub(crate) struct TenantTransaction<'connection> {
    transaction: Transaction<'connection, Postgres>,
    guards: BTreeMap<TenantId, GuardMode>,
}

impl TenantTransaction<'_> {
    /// Check both the complete operation key and its strongest required mode.
    /// There is deliberately no acquisition/upgrade API on an existing context.
    pub(crate) fn require_scope(
        &self,
        tenant: TenantId,
        mode: GuardMode,
    ) -> Result<(), GuardError> {
        if self.guards.get(&tenant).is_some_and(|held| *held >= mode) {
            Ok(())
        } else {
            Err(GuardError::ScopeMismatch)
        }
    }

    pub(crate) fn connection(
        &mut self,
        tenant: TenantId,
        mode: GuardMode,
    ) -> Result<&mut PgConnection, GuardError> {
        self.require_scope(tenant, mode)?;
        Ok(&mut self.transaction)
    }

    /// Sample at evaluation, after preceding waits (including idempotency), not
    /// at construction. PostgreSQL now()/CURRENT_TIMESTAMP would cache BEGIN time.
    pub(crate) async fn database_now(&mut self) -> Result<DateTime<Utc>, GuardError> {
        Ok(sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *self.transaction)
            .await?)
    }
}

pub(crate) async fn transact<T, F>(
    pool: &PgPool,
    guards: &[(TenantId, GuardMode)],
    body: F,
) -> Result<T, GuardError>
where
    T: Send,
    F: for<'tx, 'connection> FnOnce(
            &'tx mut TenantTransaction<'connection>,
        ) -> BoxFuture<'tx, Result<T, GuardError>>
        + Send,
{
    let caps = Limits::default();
    let limits = Limits::shortened(caps.lock, caps.statement, caps.total)?;
    transact_with_limits(pool, guards, limits, body).await
}

pub(crate) async fn transact_with_limits<T, F>(
    pool: &PgPool,
    guards: &[(TenantId, GuardMode)],
    limits: Limits,
    body: F,
) -> Result<T, GuardError>
where
    T: Send,
    F: for<'tx, 'connection> FnOnce(
            &'tx mut TenantTransaction<'connection>,
        ) -> BoxFuture<'tx, Result<T, GuardError>>
        + Send,
{
    transact_with_error(pool, guards, limits, body).await
}

/// An error aborts all provisional work, including a newly inserted anchor.
/// Return a successful value only when that value and its local effects should
/// commit. Typed errors preserve domain refusals without disguising them as SQL
/// failures; From<GuardError> must retain storage causes and commit uncertainty.
/// Limits can only be constructed with the bounded defaults or shortened policy.
pub(crate) async fn transact_with_error<T, E, F>(
    pool: &PgPool,
    guards: &[(TenantId, GuardMode)],
    limits: Limits,
    body: F,
) -> Result<T, E>
where
    T: Send,
    E: From<GuardError> + Send,
    F: for<'tx, 'connection> FnOnce(
            &'tx mut TenantTransaction<'connection>,
        ) -> BoxFuture<'tx, Result<T, E>>
        + Send,
{
    run(
        pool,
        guards,
        limits,
        "BEGIN ISOLATION LEVEL READ COMMITTED",
        body,
    )
    .await
}

// A fixed test-only fault holds BEGIN readiness after the server opened the
// transaction, before SQLx publishes its transaction object. Production callers
// cannot change the BEGIN statement or enable the injected delay.
#[cfg(test)]
#[allow(
    dead_code,
    reason = "The exact-source PostgreSQL integration suite calls this fixed fault; library unit-test compilation does not."
)]
pub(crate) async fn transact_with_delayed_begin(
    pool: &PgPool,
    tenant: TenantId,
) -> Result<(), GuardError> {
    run(
        pool,
        &[(tenant, GuardMode::Shared)],
        Limits::shortened(
            Duration::from_millis(10),
            Duration::from_millis(50),
            Duration::from_millis(100),
        )?,
        "BEGIN ISOLATION LEVEL READ COMMITTED; SELECT pg_sleep(0.5)",
        |_| Box::pin(async { panic!("cancelled BEGIN must not reach the callback") }),
    )
    .await
}

async fn run<T, E, F>(
    pool: &PgPool,
    guards: &[(TenantId, GuardMode)],
    limits: Limits,
    begin: &'static str,
    body: F,
) -> Result<T, E>
where
    T: Send,
    E: From<GuardError> + Send,
    F: for<'tx, 'connection> FnOnce(
            &'tx mut TenantTransaction<'connection>,
        ) -> BoxFuture<'tx, Result<T, E>>
        + Send,
{
    if guards.is_empty() || guards.len() > MAX_GUARDS {
        return Err(GuardError::InvalidGuards.into());
    }
    let mut normalized = BTreeMap::new();
    for &(tenant, mode) in guards {
        normalized
            .entry(tenant)
            .and_modify(|held| *held = std::cmp::max(*held, mode))
            .or_insert(mode);
    }
    // Includes pool acquisition, all waits, callback work and COMMIT. Dropping a
    // pending future drops its transaction and closes the leased connection.
    // Cleanup is asynchronous, not an immediate release receipt. In particular,
    // closing the connection does not prove an already-sent COMMIT was rolled back.
    let mut committing = false;
    let operation = async {
        let mut lease = ConnectionLease {
            connection: pool.acquire().await.map_err(GuardError::Storage)?,
            reusable: false,
        };
        let mut transaction = lease
            .connection
            .begin_with(begin)
            .await
            .map_err(GuardError::Storage)?;
        sqlx::query(
            "SELECT set_config('lock_timeout', $1, true), \
             set_config('statement_timeout', $2, true)",
        )
        .bind(format!("{}ms", limits.lock.as_millis()))
        .bind(format!("{}ms", limits.statement.as_millis()))
        .execute(&mut *transaction)
        .await
        .map_err(GuardError::Storage)?;
        // Typed UUID order equals byte order of canonical tenant keys. Acquire
        // each full key, including first-use insertion, in that one stable order.
        for (tenant, mode) in &normalized {
            sqlx::query(
                "INSERT INTO tenant_authority_guards (tenant_id) VALUES ($1) \
                 ON CONFLICT (tenant_id) DO NOTHING",
            )
            .bind(tenant.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(GuardError::Storage)?;
            let query = match mode {
                GuardMode::Shared => {
                    "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR SHARE"
                }
                GuardMode::Exclusive => {
                    "SELECT tenant_id FROM tenant_authority_guards \
                     WHERE tenant_id = $1 FOR NO KEY UPDATE"
                }
            };
            sqlx::query_scalar::<_, String>(query)
                .bind(tenant.to_string())
                .fetch_one(&mut *transaction)
                .await
                .map_err(GuardError::Storage)?;
        }
        let mut context = TenantTransaction {
            transaction,
            guards: normalized,
        };
        let value = body(&mut context).await?;
        // A callback that accidentally swallowed a SQL error must not turn an
        // aborted transaction's COMMIT-as-ROLLBACK into a reported success.
        sqlx::query("SELECT 1")
            .execute(&mut *context.transaction)
            .await
            .map_err(GuardError::Storage)?;
        committing = true;
        context
            .transaction
            .commit()
            .await
            .map_err(GuardError::Commit)?;
        lease.reusable = true;
        Ok(value)
    };
    let result = tokio::time::timeout(limits.total, operation).await;
    match result {
        Ok(result) => result,
        Err(_) if committing => Err(GuardError::CommitDeadline.into()),
        Err(_) => Err(GuardError::Deadline.into()),
    }
}
