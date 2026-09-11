//! The WP3 node journal (`PHASE-0.3.1`): SQLite in WAL mode with an explicit,
//! inspectable durability profile — and honest handling of a crash after a possible
//! provider dispatch.
//!
//! # Durability profile
//!
//! [`Journal::open`] applies WAL + `synchronous=FULL` to every connection, verifies
//! both took effect, and records the profile in the `journal_meta` table
//! (`ROADMAP.md` §11.4: "Select and document synchronous mode…"; WAL alone is not a
//! power-loss guarantee, so the conservative `FULL` setting is the Phase 0 default).
//! Pragmas are per-connection, so the meta row — not a pragma read — is what the
//! read-only inspection CLI reports.
//!
//! # Boundary-before-boundary ordering
//!
//! `ROADMAP.md` §17.4: the node persists a fact before advancing the corresponding
//! boundary. The critical boundary here is the provider dispatch:
//!
//! 1. [`record_dispatch`] writes `prepared → dispatched` (with the
//!    `provider_request_id` when known) and commits **before** the caller invokes the
//!    adapter. The boundary record is durable and visible to other connections first.
//! 2. A crash after that commit but before a result is recorded leaves the attempt
//!    `dispatched`. [`recover`] reclassifies it as `outcome_unknown` — the node may or
//!    may not have dispatched; it cannot know, so it does not guess and does not retry
//!    silently (`KICKOFF.md` WP3, kill-risk Q4).
//! 3. A crash before `record_dispatch` leaves the attempt `prepared`, which `recover`
//!    reports as `safe_to_redeliver` — the boundary was never crossed.
//! 4. The only path out of ambiguity is proof or adjudication: [`prove_result`] lands
//!    `outcome_unknown → completed|failed_known` with the adapter's evidence (the
//!    §11.3 provider-lookup recovery edges), and [`reconcile`] records an authorized
//!    `reconciled` adjudication.
//!
//! # The state machine stays in `reasonbraid-core`
//!
//! Every status change is validated against
//! [`reasonbraid_core::ProviderAttemptState::apply`] before it is written, so the
//! journal can never reach a state the domain machine forbids — the honest-ambiguity
//! edges (`outcome_unknown → completed|failed_known`, the `failed_known` state) live
//! in the core machine (extended in this leaf), not in ad-hoc journal logic.
//!
//! # Clock
//!
//! Every writer takes the caller's `now` (`chrono::DateTime<Utc>`), stored as RFC 3339
//! TEXT. Tests advance time deterministically; the journal never reads a clock itself.

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use reasonbraid_core::{ProviderAttemptState, ProviderAttemptTransition, TransitionError};
use serde::Serialize;
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use uuid::Uuid;

/// The journal's durability journal mode (`ROADMAP.md` §11.4).
pub const DURABILITY_JOURNAL_MODE: &str = "wal";
/// The journal's durability synchronous mode — `FULL`, the conservative §11.4 default.
pub const DURABILITY_SYNCHRONOUS: &str = "FULL";
/// SQLite's numeric value for `synchronous=FULL`.
const SQLITE_SYNCHRONOUS_FULL: i64 = 2;

/// A typed journal error.
#[derive(Debug)]
pub enum JournalError {
    /// A driver-level failure (including `SQLITE_READONLY` on the read-only CLI handle).
    Sql(sqlx::Error),
    /// The embedded schema migrations failed to apply.
    Migration(sqlx::migrate::MigrateError),
    /// The domain state machine refused the move (carries aggregate/from/event).
    InvalidTransition(TransitionError),
    /// The file is not a ReasonBraid node journal (missing meta, wrong mode, or not a database).
    NotAJournal(String),
    /// A record that must exist (attempt/command/operation) does not.
    NotFound { what: &'static str, id: String },
    /// A record that must not exist yet already does.
    AlreadyExists { what: &'static str, id: String },
    /// A persisted attempt status is not a wire name this build's registry knows.
    UnknownStatus(String),
    /// A row changed between read and guarded write (should be unreachable under the
    /// single-writer pool, but never silently overwritten).
    ConcurrentChange { what: &'static str, id: String },
    /// A persisted `channel_state` value is not what its key expects (corrupt state).
    CorruptState { key: &'static str, value: String },
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JournalError::Sql(e) => write!(f, "journal database error: {e}"),
            JournalError::Migration(e) => write!(f, "journal migration error: {e}"),
            JournalError::InvalidTransition(e) => write!(f, "invalid attempt transition: {e}"),
            JournalError::NotAJournal(reason) => {
                write!(f, "not a ReasonBraid node journal: {reason}")
            }
            JournalError::NotFound { what, id } => write!(f, "{what} `{id}` not found"),
            JournalError::AlreadyExists { what, id } => write!(f, "{what} `{id}` already exists"),
            JournalError::UnknownStatus(status) => {
                write!(
                    f,
                    "unknown attempt status `{status}` recorded in the journal"
                )
            }
            JournalError::ConcurrentChange { what, id } => {
                write!(
                    f,
                    "{what} `{id}` changed while the transition was being applied"
                )
            }
            JournalError::CorruptState { key, value } => {
                write!(
                    f,
                    "channel state key `{key}` holds an unreadable value `{value}`"
                )
            }
        }
    }
}

impl std::error::Error for JournalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JournalError::Sql(e) => Some(e),
            JournalError::Migration(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for JournalError {
    fn from(e: sqlx::Error) -> Self {
        JournalError::Sql(e)
    }
}

/// A command received from the control plane, to persist.
pub struct CommandInput<'a> {
    pub command_id: &'a str,
    pub tenant_id: &'a str,
    pub thread_id: &'a str,
    pub payload: &'a Value,
    /// The admitting authorization record id (`.1.5.2`, ADR-008 — the delivery
    /// carries the admission decision; a command without one has no cached
    /// decision and is refused at the dispatch boundary, fail-closed).
    pub authz_ref: Option<&'a str>,
    /// The policy digest the record bound.
    pub policy_digest: Option<&'a str>,
    /// The decision time (RFC 3339) — the freshness TTL runs from it.
    pub decided_at: Option<&'a str>,
    /// The tenant's revocation epoch AT DECISION TIME.
    pub revocation_epoch: Option<i64>,
    /// The server cursor (sequence) this command arrived under — reconnect evidence for `.3.2`.
    pub server_cursor: &'a str,
}

/// The outcome of [`Journal::record_command`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandRecorded {
    /// `true` when the command id was already durable — a transport redelivery.
    pub already_known: bool,
}

/// The outcome of [`Journal::ensure_operation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationRecorded {
    pub operation_id: String,
    /// `false` when the command already had a local operation (deduplicated).
    pub created: bool,
}

/// A proven terminal status for [`Journal::prove_result`]: the adapter supplied proof
/// (a definitive runtime result or a provider status-lookup hit), never a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenStatus {
    Completed,
    FailedKnown,
}

/// One attempt row as the inspection surfaces see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttemptSummary {
    pub attempt_id: String,
    pub operation_id: String,
    pub status: String,
    pub provider_request_id: Option<String>,
    pub evidence: Option<String>,
    pub updated_at: String,
}

/// One row of an attempt's before/after boundary ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransitionRow {
    pub seq: i64,
    pub from_status: String,
    pub to_status: String,
    pub at: String,
}

/// One outgoing event (with its body, so a pending event can be re-emitted after a
/// crash with its ORIGINAL id — `ROADMAP.md` §17.4 step 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventSummary {
    pub event_id: String,
    pub operation_id: String,
    pub emitted_at: String,
    /// The event body (JSON text), carried verbatim for re-emission.
    pub payload: String,
}

/// One inbox command that is thread work (`PHASE-0.6.2`): its payload carries a
/// `kind` of `contribute` or `revise`. The worker's execution/skip decision reads
/// [`WorkItem::latest_attempt_status`]: `None` or `prepared` is safe to execute
/// (the dispatch boundary was never crossed); anything else is skipped —
/// `outcome_unknown` needs proof or adjudication, the terminal states are done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkItem {
    pub command_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    /// The inbox payload (JSON text), carried verbatim.
    pub payload: String,
    /// The local operation this command created, once it has one.
    pub operation_id: Option<String>,
    /// The latest attempt status for that operation, if any attempt exists.
    pub latest_attempt_status: Option<String>,
}

/// The classification produced by [`Journal::recover`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryReport {
    /// Attempts still `prepared`: the dispatch boundary was never crossed, so the
    /// command is safe to redeliver (classification only — the rows are unchanged).
    pub safe_to_redeliver: Vec<AttemptSummary>,
    /// Attempts that were `dispatched` with no recorded result: the node may have
    /// dispatched, so they are now `outcome_unknown` and must be proven or adjudicated.
    pub now_ambiguous: Vec<AttemptSummary>,
}

/// The journal's inspectable health: the recorded durability profile, connection-level
/// settings, the schema version, and a `quick_check` integrity result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalHealth {
    pub journal_mode: String,
    pub synchronous: String,
    pub foreign_keys: i64,
    pub busy_timeout_ms: i64,
    pub user_version: i64,
    pub quick_check: String,
}

/// Row counts by journal section, for the inspection CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalCounts {
    pub commands: i64,
    pub operations: i64,
    pub attempts_prepared: i64,
    pub attempts_dispatched: i64,
    pub attempts_completed: i64,
    pub attempts_failed_before_dispatch: i64,
    pub attempts_failed_known: i64,
    pub attempts_outcome_unknown: i64,
    pub attempts_reconciled: i64,
    pub events_emitted: i64,
    pub events_acked: i64,
}

/// A node-local journal handle. The pool is capped at ONE connection: the journal is a
/// single-writer local store, and serializing writers makes the state-machine guards
/// (`WHERE status = ?`) authoritative rather than a race-detection net. `Clone` shares
/// the same pool (tests poll from one handle while a task drives another).
#[derive(Debug, Clone)]
pub struct Journal {
    pool: SqlitePool,
    path: PathBuf,
}

impl Journal {
    /// Open (or create) the journal at `path`, apply migrations, and enforce the
    /// durability profile: WAL + `synchronous=FULL`, verified on the live connection
    /// and recorded in `journal_meta` so the read-only CLI can report it.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let path = path.as_ref().to_path_buf();
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(std::time::Duration::from_secs(5))
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(JournalError::Migration)?;
        let journal = Self { pool, path };
        journal.verify_durability_profile().await?;
        Ok(journal)
    }

    /// Open the journal READ-ONLY for inspection (the `rb-journal` CLI). Never runs
    /// migrations and can never write: SQLite itself refuses every mutating statement.
    /// Concurrent use beside the writer is safe because the journal is in WAL mode.
    pub async fn open_readonly(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let path = path.as_ref().to_path_buf();
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .read_only(true)
            .busy_timeout(std::time::Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let journal = Self { pool, path };
        match sqlx::query_scalar::<_, String>(
            "SELECT value FROM journal_meta WHERE key = 'durability_synchronous'",
        )
        .fetch_one(&journal.pool)
        .await
        {
            Ok(_) => Ok(journal),
            Err(sqlx::Error::RowNotFound) => Err(JournalError::NotAJournal(
                "journal_meta lacks the durability profile".to_string(),
            )),
            Err(e) if e.to_string().contains("no such table") => Err(JournalError::NotAJournal(
                "no journal_meta table".to_string(),
            )),
            Err(e) => Err(JournalError::Sql(e)),
        }
    }

    /// The journal file path this handle was opened with.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Verify the durability profile actually took effect on the live connection.
    async fn verify_durability_profile(&self) -> Result<(), JournalError> {
        let mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&self.pool)
            .await?;
        if mode != DURABILITY_JOURNAL_MODE {
            return Err(JournalError::NotAJournal(format!(
                "journal_mode is `{mode}`, expected `{DURABILITY_JOURNAL_MODE}`"
            )));
        }
        let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
            .fetch_one(&self.pool)
            .await?;
        if synchronous != SQLITE_SYNCHRONOUS_FULL {
            return Err(JournalError::NotAJournal(format!(
                "synchronous is {synchronous}, expected {SQLITE_SYNCHRONOUS_FULL} (FULL)"
            )));
        }
        Ok(())
    }

    /// The one place a status change happens: validate the move against the core
    /// machine, then write the new status, optional evidence columns, and the
    /// before/after transition row in ONE transaction. Nothing else mutates `status`.
    async fn apply_transition(
        &self,
        attempt_id: &str,
        transition: ProviderAttemptTransition,
        provider_request_id: Option<&str>,
        evidence: Option<&Value>,
        at: DateTime<Utc>,
    ) -> Result<ProviderAttemptState, JournalError> {
        let mut tx = self.pool.begin().await?;

        let current_s: Option<String> =
            sqlx::query_scalar("SELECT status FROM attempts WHERE attempt_id = ?")
                .bind(attempt_id)
                .fetch_optional(&mut *tx)
                .await?;
        let current_s = current_s.ok_or_else(|| JournalError::NotFound {
            what: "attempt",
            id: attempt_id.to_string(),
        })?;
        let current = current_s.parse::<ProviderAttemptState>().map_err(
            |e: reasonbraid_core::UnknownProviderAttemptState| JournalError::UnknownStatus(e.0),
        )?;
        let next = current
            .apply(transition)
            .map_err(JournalError::InvalidTransition)?;

        let at_s = at.to_rfc3339();
        let res = sqlx::query(
            "UPDATE attempts \
             SET status = ?, \
                 provider_request_id = COALESCE(?, provider_request_id), \
                 evidence = COALESCE(?, evidence), \
                 updated_at = ? \
             WHERE attempt_id = ? AND status = ?",
        )
        .bind(next.as_str())
        .bind(provider_request_id)
        .bind(evidence)
        .bind(&at_s)
        .bind(attempt_id)
        .bind(current.as_str())
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() != 1 {
            return Err(JournalError::ConcurrentChange {
                what: "attempt",
                id: attempt_id.to_string(),
            });
        }

        let seq: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM attempt_transitions WHERE attempt_id = ?",
        )
        .bind(attempt_id)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO attempt_transitions (attempt_id, seq, from_status, to_status, at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(attempt_id)
        .bind(seq)
        .bind(current.as_str())
        .bind(next.as_str())
        .bind(&at_s)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(next)
    }

    /// Persist a received command (idempotent by command id — transport redelivery
    /// records the same row and reports [`CommandRecorded::already_known`]).
    pub async fn record_command(
        &self,
        cmd: &CommandInput<'_>,
        at: DateTime<Utc>,
    ) -> Result<CommandRecorded, JournalError> {
        let res = sqlx::query(
            "INSERT OR IGNORE INTO commands \
             (command_id, tenant_id, thread_id, payload, authz_ref, policy_digest, \
              decided_at, revocation_epoch, server_cursor, received_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(cmd.command_id)
        .bind(cmd.tenant_id)
        .bind(cmd.thread_id)
        .bind(cmd.payload)
        .bind(cmd.authz_ref)
        .bind(cmd.policy_digest)
        .bind(cmd.decided_at)
        .bind(cmd.revocation_epoch)
        .bind(cmd.server_cursor)
        .bind(at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        let already_known = res.rows_affected() == 0;
        // A REPLAY re-delivery (`.2.4`) carries a FRESH admission decision over
        // a command the journal already holds: refresh the decision facts so the
        // dispatch gate evaluates the replayed admission, not the dead one. A
        // plain redelivery (no decision) keeps the original row untouched.
        if already_known && cmd.decided_at.is_some() {
            sqlx::query(
                "UPDATE commands SET authz_ref = ?, policy_digest = ?, decided_at = ?, \
                 revocation_epoch = ? WHERE command_id = ?",
            )
            .bind(cmd.authz_ref)
            .bind(cmd.policy_digest)
            .bind(cmd.decided_at)
            .bind(cmd.revocation_epoch)
            .bind(cmd.command_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(CommandRecorded { already_known })
    }

    /// The local operation for a command, created exactly once: `command_id` is UNIQUE
    /// in `operations`, so a duplicated command never creates a second local
    /// operation (`KICKOFF.md` WP3 acceptance).
    pub async fn ensure_operation(
        &self,
        command_id: &str,
        at: DateTime<Utc>,
    ) -> Result<OperationRecorded, JournalError> {
        let exists: Option<String> =
            sqlx::query_scalar("SELECT command_id FROM commands WHERE command_id = ?")
                .bind(command_id)
                .fetch_optional(&self.pool)
                .await?;
        if exists.is_none() {
            return Err(JournalError::NotFound {
                what: "command",
                id: command_id.to_string(),
            });
        }

        if let Some(operation_id) = sqlx::query_scalar::<_, String>(
            "SELECT operation_id FROM operations WHERE command_id = ?",
        )
        .bind(command_id)
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(OperationRecorded {
                operation_id,
                created: false,
            });
        }

        let operation_id = format!("op_{}", Uuid::now_v7());
        let res = sqlx::query(
            "INSERT OR IGNORE INTO operations (operation_id, command_id, created_at) \
             VALUES (?, ?, ?)",
        )
        .bind(&operation_id)
        .bind(command_id)
        .bind(at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            // Lost a race to another writer: the existing row is authoritative.
            let winner: String =
                sqlx::query_scalar("SELECT operation_id FROM operations WHERE command_id = ?")
                    .bind(command_id)
                    .fetch_one(&self.pool)
                    .await?;
            return Ok(OperationRecorded {
                operation_id: winner,
                created: false,
            });
        }
        Ok(OperationRecorded {
            operation_id,
            created: true,
        })
    }

    /// Register an attempt in `prepared` — before any dispatch boundary.
    pub async fn prepare_attempt(
        &self,
        attempt_id: &str,
        operation_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        let exists: Option<String> =
            sqlx::query_scalar("SELECT operation_id FROM operations WHERE operation_id = ?")
                .bind(operation_id)
                .fetch_optional(&self.pool)
                .await?;
        if exists.is_none() {
            return Err(JournalError::NotFound {
                what: "operation",
                id: operation_id.to_string(),
            });
        }
        let at_s = at.to_rfc3339();
        let res = sqlx::query(
            "INSERT OR IGNORE INTO attempts \
             (attempt_id, operation_id, status, created_at, updated_at) \
             VALUES (?, ?, 'prepared', ?, ?)",
        )
        .bind(attempt_id)
        .bind(operation_id)
        .bind(&at_s)
        .bind(&at_s)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(JournalError::AlreadyExists {
                what: "attempt",
                id: attempt_id.to_string(),
            });
        }
        Ok(())
    }

    /// THE dispatch boundary record: `prepared → dispatched`, committed BEFORE the
    /// caller invokes the adapter. `provider_request_id` is recorded here when known
    /// (the proof handle a later status lookup needs). After this returns, a crash
    /// means the dispatch *may* have happened — recovery reports `outcome_unknown`.
    pub async fn record_dispatch(
        &self,
        attempt_id: &str,
        provider_request_id: Option<&str>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::Dispatch,
            provider_request_id,
            None,
            at,
        )
        .await?;
        Ok(())
    }

    /// A deterministic failure BEFORE the dispatch boundary was crossed.
    pub async fn record_failed_before_dispatch(
        &self,
        attempt_id: &str,
        reason: Option<&str>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        let evidence = reason.map(|r| serde_json::json!({ "failure_reason": r }));
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::FailBeforeDispatch,
            None,
            evidence.as_ref(),
            at,
        )
        .await?;
        Ok(())
    }

    /// A proven successful result (the adapter returned it definitively).
    pub async fn record_completed(
        &self,
        attempt_id: &str,
        result: Option<&Value>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::Complete,
            None,
            result,
            at,
        )
        .await?;
        Ok(())
    }

    /// A proven failure (a definitive provider rejection at runtime) — never a guess.
    pub async fn record_failed_known(
        &self,
        attempt_id: &str,
        failure: Option<&Value>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::FailKnown,
            None,
            failure,
            at,
        )
        .await?;
        Ok(())
    }

    /// The result of the attempt is indeterminate (e.g. a timeout after dispatch):
    /// the journal records the honest `outcome_unknown` instead of guessing.
    pub async fn record_outcome_unknown(
        &self,
        attempt_id: &str,
        reason: Option<&str>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        let evidence = reason.map(|r| serde_json::json!({ "reason": r }));
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::MarkOutcomeUnknown,
            None,
            evidence.as_ref(),
            at,
        )
        .await?;
        Ok(())
    }

    /// Record the provider's request id for an attempt — the proof handle a later
    /// status lookup needs (`§11.3`). It arrives with the dispatch acknowledgement,
    /// which follows the boundary record, so it is attached afterwards; the FIRST id
    /// recorded wins (provider request ids are stable for one attempt).
    pub async fn attach_provider_request_id(
        &self,
        attempt_id: &str,
        provider_request_id: &str,
    ) -> Result<(), JournalError> {
        let res = sqlx::query(
            "UPDATE attempts SET provider_request_id = COALESCE(provider_request_id, ?) \
             WHERE attempt_id = ?",
        )
        .bind(provider_request_id)
        .bind(attempt_id)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(JournalError::NotFound {
                what: "attempt",
                id: attempt_id.to_string(),
            });
        }
        Ok(())
    }

    /// The ONLY way an ambiguous attempt leaves `outcome_unknown` with a result: the
    /// adapter PROVED it (a definitive runtime answer or a provider status-lookup hit,
    /// §11.3). Land on the proven terminal state with the evidence attached.
    pub async fn prove_result(
        &self,
        attempt_id: &str,
        status: ProvenStatus,
        provider_request_id: Option<&str>,
        evidence: Option<&Value>,
        at: DateTime<Utc>,
    ) -> Result<(), JournalError> {
        let transition = match status {
            ProvenStatus::Completed => ProviderAttemptTransition::Complete,
            ProvenStatus::FailedKnown => ProviderAttemptTransition::FailKnown,
        };
        self.apply_transition(attempt_id, transition, provider_request_id, evidence, at)
            .await?;
        Ok(())
    }

    /// An authorized adjudication of an ambiguous attempt: `outcome_unknown →
    /// reconciled` (terminal). The machine refuses to reconcile anything else.
    pub async fn reconcile(&self, attempt_id: &str, at: DateTime<Utc>) -> Result<(), JournalError> {
        self.apply_transition(
            attempt_id,
            ProviderAttemptTransition::Reconcile,
            None,
            None,
            at,
        )
        .await?;
        Ok(())
    }

    /// Crash recovery: reclassify every attempt still `dispatched` (the boundary was
    /// durably crossed with no recorded result) as `outcome_unknown`, and report every
    /// `prepared` attempt as `safe_to_redeliver` (unchanged — the boundary was never
    /// crossed). Idempotent: a second run has nothing left to reclassify.
    pub async fn recover(&self, at: DateTime<Utc>) -> Result<RecoveryReport, JournalError> {
        let safe_to_redeliver = self
            .attempts_where("status = 'prepared'", "created_at")
            .await?;

        let dispatched = sqlx::query_scalar::<_, String>(
            "SELECT attempt_id FROM attempts WHERE status = 'dispatched'",
        )
        .fetch_all(&self.pool)
        .await?;
        for attempt_id in dispatched {
            self.apply_transition(
                &attempt_id,
                ProviderAttemptTransition::MarkOutcomeUnknown,
                None,
                Some(&serde_json::json!({ "reason": "crash recovery: dispatched with no recorded result" })),
                at,
            )
            .await?;
        }
        let now_ambiguous = self
            .attempts_where("status = 'outcome_unknown'", "updated_at")
            .await?;
        Ok(RecoveryReport {
            safe_to_redeliver,
            now_ambiguous,
        })
    }

    /// Record an event the node emitted toward the control plane (idempotent by
    /// event id — a redelivery of the same emission records nothing new).
    pub async fn record_outgoing_event(
        &self,
        event_id: &str,
        operation_id: &str,
        payload: &Value,
        at: DateTime<Utc>,
    ) -> Result<CommandRecorded, JournalError> {
        let exists: Option<String> =
            sqlx::query_scalar("SELECT operation_id FROM operations WHERE operation_id = ?")
                .bind(operation_id)
                .fetch_optional(&self.pool)
                .await?;
        if exists.is_none() {
            return Err(JournalError::NotFound {
                what: "operation",
                id: operation_id.to_string(),
            });
        }
        let res = sqlx::query(
            "INSERT OR IGNORE INTO outgoing_events (event_id, operation_id, payload, emitted_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(event_id)
        .bind(operation_id)
        .bind(payload)
        .bind(at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(CommandRecorded {
            already_known: res.rows_affected() == 0,
        })
    }

    /// Mark an emitted event acknowledged with the server cursor at acknowledgement.
    /// Returns `true` when THIS call set the acknowledgement (idempotent on redelivery).
    pub async fn acknowledge_event(
        &self,
        event_id: &str,
        ack_cursor: &str,
        at: DateTime<Utc>,
    ) -> Result<bool, JournalError> {
        let res = sqlx::query(
            "UPDATE outgoing_events SET acked_at = ?, ack_cursor = ? \
             WHERE event_id = ? AND acked_at IS NULL",
        )
        .bind(at.to_rfc3339())
        .bind(ack_cursor)
        .bind(event_id)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 1 {
            return Ok(true);
        }
        let exists: Option<String> =
            sqlx::query_scalar("SELECT event_id FROM outgoing_events WHERE event_id = ?")
                .bind(event_id)
                .fetch_optional(&self.pool)
                .await?;
        if exists.is_none() {
            return Err(JournalError::NotFound {
                what: "event",
                id: event_id.to_string(),
            });
        }
        Ok(false)
    }

    /// In-flight attempts: `prepared` (boundary not crossed) and `dispatched`
    /// (boundary crossed, result not yet recorded) — the "pending" inspection view.
    pub async fn pending_attempts(&self) -> Result<Vec<AttemptSummary>, JournalError> {
        self.attempts_where("status IN ('prepared', 'dispatched')", "created_at")
            .await
    }

    /// Ambiguous attempts awaiting proof or adjudication.
    pub async fn ambiguous_attempts(&self) -> Result<Vec<AttemptSummary>, JournalError> {
        self.attempts_where("status = 'outcome_unknown'", "updated_at")
            .await
    }

    /// One attempt row by id, in any status (the supervisor's tests and operators use
    /// it to see the attached provider handle on a TERMINAL attempt).
    pub async fn attempt_summary(&self, attempt_id: &str) -> Result<AttemptSummary, JournalError> {
        let (attempt_id, operation_id, status, provider_request_id, evidence, updated_at) =
            sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String)>(
                "SELECT attempt_id, operation_id, status, provider_request_id, evidence, updated_at \
                 FROM attempts WHERE attempt_id = ?",
            )
            .bind(attempt_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => JournalError::NotFound {
                    what: "attempt",
                    id: attempt_id.to_string(),
                },
                other => JournalError::Sql(other),
            })?;
        Ok(AttemptSummary {
            attempt_id,
            operation_id,
            status,
            provider_request_id,
            evidence,
            updated_at,
        })
    }

    /// The before/after boundary ledger of one attempt, in order.
    pub async fn attempt_history(
        &self,
        attempt_id: &str,
    ) -> Result<Vec<TransitionRow>, JournalError> {
        let rows = sqlx::query_as::<_, (i64, String, String, String)>(
            "SELECT seq, from_status, to_status, at FROM attempt_transitions \
             WHERE attempt_id = ? ORDER BY seq",
        )
        .bind(attempt_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(seq, from_status, to_status, at)| TransitionRow {
                seq,
                from_status,
                to_status,
                at,
            })
            .collect())
    }

    /// Outgoing events the server has not acknowledged yet, with their bodies — the
    /// reconnect path re-emits these with their ORIGINAL ids (`ROADMAP.md` §17.4 step 5).
    pub async fn pending_events(&self) -> Result<Vec<EventSummary>, JournalError> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT event_id, operation_id, emitted_at, payload FROM outgoing_events \
             WHERE acked_at IS NULL ORDER BY emitted_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(event_id, operation_id, emitted_at, payload)| EventSummary {
                    event_id,
                    operation_id,
                    emitted_at,
                    payload,
                },
            )
            .collect())
    }

    /// ALL outgoing events, acknowledged or not, oldest first (`PHASE-0.6.2`) —
    /// the evidence surface a node's emissions are inspected from, and the source
    /// of the ORIGINAL ids a duplicate-transport demonstration re-sends.
    pub async fn emitted_events(&self) -> Result<Vec<EventSummary>, JournalError> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT event_id, operation_id, emitted_at, payload FROM outgoing_events \
             ORDER BY emitted_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(event_id, operation_id, emitted_at, payload)| EventSummary {
                    event_id,
                    operation_id,
                    emitted_at,
                    payload,
                },
            )
            .collect())
    }

    /// The node's acknowledgement cursor: the highest server command cursor the node has
    /// durably recorded (0 on a fresh journal). Reported in the reconnect handshake;
    /// the server replays everything after it and this journal deduplicates.
    pub async fn last_acked_cursor(&self) -> Result<i64, JournalError> {
        let value: Option<String> =
            sqlx::query_scalar("SELECT value FROM channel_state WHERE key = 'last_acked_cursor'")
                .fetch_optional(&self.pool)
                .await?;
        match value {
            None => Ok(0),
            Some(v) => v.parse::<i64>().map_err(|_| JournalError::CorruptState {
                key: "last_acked_cursor",
                value: v,
            }),
        }
    }

    /// Record the node's acknowledgement cursor (idempotent overwrite).
    pub async fn set_last_acked_cursor(&self, cursor: i64) -> Result<(), JournalError> {
        sqlx::query(
            "INSERT INTO channel_state (key, value) VALUES ('last_acked_cursor', ?) \
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        )
        .bind(cursor.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The tenant's revocation epoch as of the LATEST handshake/poll the node has
    /// seen (`.1.5.2`, ADR-008): the freshness reference every cached admission
    /// decision is evaluated against at the dispatch boundary. `None` until the
    /// first handshake/poll — a dispatch gate with no epoch reference fails
    /// closed (the cache cannot be validated without one).
    pub async fn revocation_epoch(&self) -> Result<Option<i64>, JournalError> {
        let value: Option<String> =
            sqlx::query_scalar("SELECT value FROM channel_state WHERE key = 'revocation_epoch'")
                .fetch_optional(&self.pool)
                .await?;
        match value {
            None => Ok(None),
            Some(v) => v
                .parse::<i64>()
                .map(Some)
                .map_err(|_| JournalError::CorruptState {
                    key: "revocation_epoch",
                    value: v,
                }),
        }
    }

    /// Record the latest seen tenant revocation epoch (idempotent overwrite).
    pub async fn set_revocation_epoch(&self, epoch: i64) -> Result<(), JournalError> {
        sqlx::query(
            "INSERT INTO channel_state (key, value) VALUES ('revocation_epoch', ?) \
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        )
        .bind(epoch.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The cached ADMISSION decision for one command (`.1.5.2`, ADR-008) — the
    /// server's decision metadata journaled with the delivery. `None` when the
    /// command carries no decision (pre-0003 row or plain traffic): the dispatch
    /// boundary treats that as fail-closed. The cached kind is always `Allow`
    /// (the server only DELIVERS allowed work; a re-ask denial is journaled as a
    /// refusal, not a cached row).
    pub async fn cached_decision(
        &self,
        command_id: &str,
    ) -> Result<Option<reasonbraid_core::CachedDecision>, JournalError> {
        let row: Option<(Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT policy_digest, decided_at, revocation_epoch FROM commands \
             WHERE command_id = ?",
        )
        .bind(command_id)
        .fetch_optional(&self.pool)
        .await?;
        let Some((Some(policy_digest), Some(decided_at), Some(revocation_epoch))) = row else {
            return Ok(None);
        };
        let decided_at = chrono::DateTime::parse_from_rfc3339(&decided_at)
            .map(|d| d.with_timezone(&chrono::Utc))
            .map_err(|_| JournalError::CorruptState {
                key: "commands.decided_at",
                value: decided_at,
            })?;
        Ok(Some(reasonbraid_core::CachedDecision::allow(
            decided_at,
            revocation_epoch as u64,
            policy_digest,
        )))
    }

    /// The local operations that have not reached a terminal state (no attempt yet, or
    /// every attempt still in flight/ambiguous) — the "pending local operation IDs" the
    /// reconnect handshake reports (`§17.4` step 2).
    pub async fn pending_operations(&self) -> Result<Vec<String>, JournalError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT operation_id FROM operations \
             WHERE operation_id NOT IN ( \
                 SELECT operation_id FROM attempts \
                 WHERE status IN ('completed', 'failed_known', 'failed_before_dispatch', 'reconciled') \
             ) \
             ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// All local operation ids, oldest first (an inspection/assertion surface).
    pub async fn operation_ids(&self) -> Result<Vec<String>, JournalError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT operation_id FROM operations ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// The thread-work items this journal holds (`PHASE-0.6.2`): every command whose
    /// payload declares a work kind, joined with its local operation and the latest
    /// attempt status. Non-work commands (plain WP3 channel traffic) are excluded.
    pub async fn work_items(&self) -> Result<Vec<WorkItem>, JournalError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
            ),
        >(
            "SELECT c.command_id, c.tenant_id, c.thread_id, c.payload, o.operation_id, \
                    (SELECT a.status FROM attempts a \
                     WHERE a.operation_id = o.operation_id \
                     ORDER BY a.updated_at DESC LIMIT 1) \
             FROM commands c LEFT JOIN operations o ON o.command_id = c.command_id \
             ORDER BY c.received_at",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for (command_id, tenant_id, thread_id, payload, operation_id, latest_attempt_status) in rows
        {
            let value: Value =
                serde_json::from_str(&payload).map_err(|e| JournalError::CorruptState {
                    key: "command payload",
                    value: e.to_string(),
                })?;
            match value.get("kind").and_then(|v| v.as_str()) {
                Some("contribute") | Some("revise") => {}
                _ => continue, // plain channel command — not thread work
            }
            items.push(WorkItem {
                command_id,
                tenant_id,
                thread_id,
                payload,
                operation_id,
                latest_attempt_status,
            });
        }
        Ok(items)
    }

    /// Mark a locally-emitted event acknowledged because the SERVER reports holding it
    /// (the `known_events` handshake path: the server accepted the event, the ack was
    /// lost). Only a pending event with the matching ids is marked; anything else is a
    /// no-op — never an overwrite.
    pub async fn acknowledge_known_event(
        &self,
        operation_id: &str,
        event_id: &str,
        ack_cursor: i64,
        at: DateTime<Utc>,
    ) -> Result<bool, JournalError> {
        let res = sqlx::query(
            "UPDATE outgoing_events SET acked_at = ?, ack_cursor = ? \
             WHERE operation_id = ? AND event_id = ? AND acked_at IS NULL",
        )
        .bind(at.to_rfc3339())
        .bind(ack_cursor.to_string())
        .bind(operation_id)
        .bind(event_id)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() == 1)
    }

    async fn attempts_where(
        &self,
        where_clause: &str,
        order_by: &str,
    ) -> Result<Vec<AttemptSummary>, JournalError> {
        let sql = format!(
            "SELECT attempt_id, operation_id, status, provider_request_id, evidence, updated_at \
             FROM attempts WHERE {where_clause} ORDER BY {order_by}"
        );
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
            ),
        >(&sql)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(attempt_id, operation_id, status, provider_request_id, evidence, updated_at)| {
                    AttemptSummary {
                        attempt_id,
                        operation_id,
                        status,
                        provider_request_id,
                        evidence,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    /// Every attempt of one operation, oldest first — the inspection surface the
    /// `.1.5.2` refusal tests (and operators) read the dispatch-boundary verdict
    /// from (a refused cached decision is a `failed_before_dispatch` with the
    /// staleness in its evidence).
    pub async fn attempts_for_operation(
        &self,
        operation_id: &str,
    ) -> Result<Vec<AttemptSummary>, JournalError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            "SELECT attempt_id, operation_id, status, provider_request_id, evidence, updated_at \
             FROM attempts WHERE operation_id = ? ORDER BY created_at",
        )
        .bind(operation_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(attempt_id, operation_id, status, provider_request_id, evidence, updated_at)| {
                    AttemptSummary {
                        attempt_id,
                        operation_id,
                        status,
                        provider_request_id,
                        evidence,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    /// Has the operation already reported a dead letter (`.2.4`)? The dedup
    /// rides the durable outgoing-event journal: the report is emitted ONCE
    /// per operation, and a transport failure re-emits with the original id.
    pub async fn has_dead_letter(&self, operation_id: &str) -> Result<bool, JournalError> {
        let found: Option<String> = sqlx::query_scalar(
            "SELECT event_id FROM outgoing_events \
             WHERE operation_id = ? AND json_extract(payload, '$.kind') = 'work_dead_lettered' \
             LIMIT 1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(found.is_some())
    }

    /// Inspectable health: the recorded durability profile, live connection settings,
    /// the schema version, and a `quick_check` integrity result.
    pub async fn health(&self) -> Result<JournalHealth, JournalError> {
        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&self.pool)
            .await?;
        let synchronous: String = sqlx::query_scalar(
            "SELECT value FROM journal_meta WHERE key = 'durability_synchronous'",
        )
        .fetch_one(&self.pool)
        .await?;
        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&self.pool)
            .await?;
        let busy_timeout_ms: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&self.pool)
            .await?;
        let user_version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&self.pool)
            .await?;
        let quick_check: String = sqlx::query_scalar("PRAGMA quick_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(JournalHealth {
            journal_mode,
            synchronous,
            foreign_keys,
            busy_timeout_ms,
            user_version,
            quick_check,
        })
    }

    /// Row counts by journal section.
    pub async fn counts(&self) -> Result<JournalCounts, JournalError> {
        let (
            commands,
            operations,
            attempts_prepared,
            attempts_dispatched,
            attempts_completed,
            attempts_failed_before_dispatch,
            attempts_failed_known,
            attempts_outcome_unknown,
            attempts_reconciled,
            events_emitted,
            events_acked,
        ): (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT \
             (SELECT COUNT(*) FROM commands), \
             (SELECT COUNT(*) FROM operations), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'prepared'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'dispatched'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'completed'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'failed_before_dispatch'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'failed_known'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'outcome_unknown'), \
             (SELECT COUNT(*) FROM attempts WHERE status = 'reconciled'), \
             (SELECT COUNT(*) FROM outgoing_events), \
             (SELECT COUNT(*) FROM outgoing_events WHERE acked_at IS NOT NULL)",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(JournalCounts {
            commands,
            operations,
            attempts_prepared,
            attempts_dispatched,
            attempts_completed,
            attempts_failed_before_dispatch,
            attempts_failed_known,
            attempts_outcome_unknown,
            attempts_reconciled,
            events_emitted,
            events_acked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique journal path under the repo's `target/` (same volume as the repo, per
    /// the data-locality policy; `target/` is gitignored and cleaned with `cargo clean`).
    fn test_path(name: &str) -> PathBuf {
        let unique = uuid::Uuid::now_v7();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/journal-tests")
            .join(format!("{name}-{unique}"));
        std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
        // Exclusive: an existing directory belongs to another fixture or an
        // earlier run, and must never be adopted.
        std::fs::DirBuilder::new()
            .create(&dir)
            .expect("the fixture directory is new");
        dir.join("node.db")
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn command<'a>(command_id: &'a str, cursor: &'a str, payload: &'a Value) -> CommandInput<'a> {
        CommandInput {
            command_id,
            tenant_id: "ten_00000000-0000-7000-8000-000000000000",
            thread_id: "thr_00000000-0000-7000-8000-000000000000",
            payload,
            authz_ref: None,
            policy_digest: None,
            decided_at: None,
            revocation_epoch: None,
            server_cursor: cursor,
        }
    }

    /// Seed the full pre-dispatch chain and return the attempt/operation ids.
    async fn seed_attempt(journal: &Journal, tag: &str) -> (String, String) {
        let cmd_id = format!("cmd_{tag}");
        let payload = serde_json::json!({ "operation": "contribute" });
        journal
            .record_command(&command(&cmd_id, "c1", &payload), now())
            .await
            .expect("record command");
        let op = journal
            .ensure_operation(&cmd_id, now())
            .await
            .expect("ensure operation");
        assert!(op.created);
        let attempt_id = format!("patt_{tag}");
        journal
            .prepare_attempt(&attempt_id, &op.operation_id, now())
            .await
            .expect("prepare attempt");
        (attempt_id, op.operation_id)
    }

    /// The durability profile is applied AND recorded: pragmas verify on the live
    /// connection and the meta row carries the profile the read-only CLI reports.
    #[tokio::test]
    async fn open_applies_and_records_the_durability_profile() {
        let journal = Journal::open(test_path("profile")).await.unwrap();
        let health = journal.health().await.unwrap();
        assert_eq!(health.journal_mode, "wal");
        assert_eq!(health.synchronous, "FULL");
        assert_eq!(health.foreign_keys, 1);
        assert_eq!(health.user_version, 3, "migrations set the schema version");
        assert_eq!(health.quick_check, "ok");
        assert!(health.busy_timeout_ms > 0);
    }

    /// A transport redelivery of the same command records exactly one row and reports
    /// `already_known` on the replay.
    #[tokio::test]
    async fn record_command_is_idempotent_by_command_id() {
        let journal = Journal::open(test_path("cmd")).await.unwrap();
        let payload = serde_json::json!({ "operation": "contribute" });
        let cmd = command("cmd_x", "c7", &payload);
        let first = journal.record_command(&cmd, now()).await.unwrap();
        assert!(!first.already_known);
        let replay = journal.record_command(&cmd, now()).await.unwrap();
        assert!(replay.already_known, "redelivery must be reported as known");
        let counts = journal.counts().await.unwrap();
        assert_eq!(counts.commands, 1, "one command row despite redelivery");
    }

    /// A duplicated command never creates a second local operation: the operation is
    /// keyed 1:1 on the command id (KICKOFF WP3 acceptance).
    #[tokio::test]
    async fn ensure_operation_creates_exactly_once() {
        let journal = Journal::open(test_path("op")).await.unwrap();
        let payload = serde_json::json!({ "operation": "contribute" });
        journal
            .record_command(&command("cmd_y", "c9", &payload), now())
            .await
            .unwrap();
        let first = journal.ensure_operation("cmd_y", now()).await.unwrap();
        assert!(first.created);
        let replay = journal.ensure_operation("cmd_y", now()).await.unwrap();
        assert!(
            !replay.created,
            "duplicate command must not create a second operation"
        );
        assert_eq!(first.operation_id, replay.operation_id);
        assert_eq!(journal.counts().await.unwrap().operations, 1);

        let missing = journal.ensure_operation("cmd_absent", now()).await;
        assert!(matches!(
            missing,
            Err(JournalError::NotFound {
                what: "command",
                ..
            })
        ));
    }

    /// The happy path writes the before/after boundary ledger, and moves the core
    /// machine forbids are rejected — the journal cannot reach a state the domain
    /// machine does not allow.
    #[tokio::test]
    async fn attempt_lifecycle_writes_the_boundary_ledger() {
        let journal = Journal::open(test_path("lifecycle")).await.unwrap();
        let (attempt_id, _op) = seed_attempt(&journal, "life").await;
        let t0 = now();

        journal
            .record_dispatch(&attempt_id, Some("prv_abc"), t0)
            .await
            .unwrap();
        journal
            .record_completed(&attempt_id, Some(&serde_json::json!({ "ok": true })), t0)
            .await
            .unwrap();

        let history = journal.attempt_history(&attempt_id).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].from_status, "prepared");
        assert_eq!(history[0].to_status, "dispatched");
        assert_eq!(history[1].from_status, "dispatched");
        assert_eq!(history[1].to_status, "completed");
        assert_eq!(history[0].seq + 1, history[1].seq);

        // A second completion is invalid: the machine rejects, nothing is written.
        let again = journal.record_completed(&attempt_id, None, now()).await;
        assert!(matches!(again, Err(JournalError::InvalidTransition(_))));
        assert_eq!(
            journal.attempt_history(&attempt_id).await.unwrap().len(),
            2,
            "the rejected move wrote nothing"
        );

        // Unknown attempt ids are NotFound, never a silent no-op.
        let missing = journal.record_dispatch("patt_nope", None, now()).await;
        assert!(matches!(
            missing,
            Err(JournalError::NotFound {
                what: "attempt",
                ..
            })
        ));
    }

    /// THE acceptance: a crash after the dispatch boundary record but before a result
    /// is recovered as `outcome_unknown` — never retried silently, never guessed.
    #[tokio::test]
    async fn crash_after_dispatch_before_result_recovers_to_outcome_unknown() {
        let path = test_path("crash-dispatch");
        let (attempt_id, _op) = {
            let journal = Journal::open(&path).await.unwrap();
            let ids = seed_attempt(&journal, "kpd").await;
            journal
                .record_dispatch(&ids.0, Some("prv_kpd"), now())
                .await
                .unwrap();
            ids
        }; // "crash": the handle is dropped with the dispatch committed, no result.

        let journal = Journal::open(&path).await.unwrap();
        let report = journal.recover(now()).await.unwrap();
        assert!(report.safe_to_redeliver.is_empty());
        assert_eq!(report.now_ambiguous.len(), 1);
        let ambiguous = &report.now_ambiguous[0];
        assert_eq!(ambiguous.attempt_id, attempt_id);
        assert_eq!(ambiguous.status, "outcome_unknown");
        assert_eq!(
            ambiguous.provider_request_id.as_deref(),
            Some("prv_kpd"),
            "the proof handle recorded at the boundary survives the crash"
        );
        let history = journal.attempt_history(&attempt_id).await.unwrap();
        assert_eq!(history.last().unwrap().to_status, "outcome_unknown");

        // Recovery is idempotent: a second run finds nothing left to reclassify.
        let again = journal.recover(now()).await.unwrap();
        assert_eq!(again.now_ambiguous.len(), 1);
        assert_eq!(journal.attempt_history(&attempt_id).await.unwrap().len(), 2);
    }

    /// A crash BEFORE the dispatch boundary is `safe_to_redeliver`: the boundary was
    /// never crossed, so redelivery cannot duplicate a provider effect.
    #[tokio::test]
    async fn crash_before_dispatch_is_safe_to_redeliver() {
        let path = test_path("crash-prepared");
        let (attempt_id, _op) = {
            let journal = Journal::open(&path).await.unwrap();
            seed_attempt(&journal, "kpp").await
        }; // "crash" right after prepare, before any dispatch.

        let journal = Journal::open(&path).await.unwrap();
        let report = journal.recover(now()).await.unwrap();
        assert!(report.now_ambiguous.is_empty());
        assert_eq!(report.safe_to_redeliver.len(), 1);
        assert_eq!(report.safe_to_redeliver[0].attempt_id, attempt_id);
        assert_eq!(report.safe_to_redeliver[0].status, "prepared");
        // The prepared row is unchanged — classification only.
        let pending = journal.pending_attempts().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].status, "prepared");
    }

    /// A crash AFTER a result was recorded leaves the terminal state untouched.
    #[tokio::test]
    async fn crash_after_result_is_untouched_by_recover() {
        let path = test_path("crash-completed");
        let (attempt_id, _op) = {
            let journal = Journal::open(&path).await.unwrap();
            let ids = seed_attempt(&journal, "kpc").await;
            journal.record_dispatch(&ids.0, None, now()).await.unwrap();
            journal
                .record_completed(&ids.0, Some(&serde_json::json!({ "ok": true })), now())
                .await
                .unwrap();
            ids
        }; // "crash" after the completed record.

        let journal = Journal::open(&path).await.unwrap();
        let report = journal.recover(now()).await.unwrap();
        assert!(report.safe_to_redeliver.is_empty());
        assert!(report.now_ambiguous.is_empty(), "completed is terminal");
        assert_eq!(
            journal
                .attempt_history(&attempt_id)
                .await
                .unwrap()
                .last()
                .unwrap()
                .to_status,
            "completed"
        );
    }

    /// The acceptance's escape hatch: an ambiguous attempt whose result the adapter
    /// can PROVE lands on the proven terminal state (§11.3 provider lookup). Without
    /// proof, nothing but adjudication can move it.
    #[tokio::test]
    async fn prove_result_lands_ambiguous_on_proven_terminal_states() {
        let journal = Journal::open(test_path("prove")).await.unwrap();

        // Proved success.
        let (a1, _) = seed_attempt(&journal, "prv1").await;
        journal
            .record_dispatch(&a1, Some("prv_1"), now())
            .await
            .unwrap();
        journal
            .record_outcome_unknown(&a1, Some("timeout"), now())
            .await
            .unwrap();
        journal
            .prove_result(
                &a1,
                ProvenStatus::Completed,
                None,
                Some(&serde_json::json!({ "status_lookup": "completed" })),
                now(),
            )
            .await
            .unwrap();
        let history = journal.attempt_history(&a1).await.unwrap();
        assert_eq!(history.last().unwrap().to_status, "completed");

        // Proved failure.
        let (a2, _) = seed_attempt(&journal, "prv2").await;
        journal.record_dispatch(&a2, None, now()).await.unwrap();
        journal
            .record_outcome_unknown(&a2, None, now())
            .await
            .unwrap();
        journal
            .prove_result(
                &a2,
                ProvenStatus::FailedKnown,
                Some("prv_2"),
                Some(&serde_json::json!({ "status_lookup": "failed" })),
                now(),
            )
            .await
            .unwrap();
        let summary = journal.ambiguous_attempts().await.unwrap();
        assert!(
            summary.iter().all(|a| a.attempt_id != a2),
            "a proven attempt is no longer ambiguous"
        );
        let history = journal.attempt_history(&a2).await.unwrap();
        assert_eq!(history.last().unwrap().to_status, "failed_known");

        // Proving on a non-ambiguous attempt is refused by the core machine.
        let (a3, _) = seed_attempt(&journal, "prv3").await;
        let refused = journal
            .prove_result(&a3, ProvenStatus::Completed, None, None, now())
            .await;
        assert!(matches!(refused, Err(JournalError::InvalidTransition(_))));
    }

    /// An authorized adjudication ends ambiguity: `outcome_unknown → reconciled`.
    #[tokio::test]
    async fn reconcile_lands_ambiguous_on_reconciled() {
        let journal = Journal::open(test_path("reconcile")).await.unwrap();
        let (attempt_id, _) = seed_attempt(&journal, "rec").await;
        journal
            .record_dispatch(&attempt_id, None, now())
            .await
            .unwrap();
        journal
            .record_outcome_unknown(&attempt_id, None, now())
            .await
            .unwrap();
        journal.reconcile(&attempt_id, now()).await.unwrap();
        assert!(journal.ambiguous_attempts().await.unwrap().is_empty());
        assert_eq!(
            journal
                .attempt_history(&attempt_id)
                .await
                .unwrap()
                .last()
                .unwrap()
                .to_status,
            "reconciled"
        );
        // Reconciling something that was never ambiguous is refused.
        let (a2, _) = seed_attempt(&journal, "rec2").await;
        let refused = journal.reconcile(&a2, now()).await;
        assert!(matches!(refused, Err(JournalError::InvalidTransition(_))));
    }

    /// Boundary-before-boundary ordering: the dispatch record is durable AND visible
    /// to a second connection before the adapter would be invoked — a crash at the
    /// adapter-call seam cannot lose the boundary fact.
    #[tokio::test]
    async fn boundary_record_is_durable_and_visible_before_the_adapter_runs() {
        let path = test_path("boundary");
        let journal = Journal::open(&path).await.unwrap();
        let (attempt_id, _op) = seed_attempt(&journal, "bnd").await;

        journal
            .record_dispatch(&attempt_id, Some("prv_bnd"), now())
            .await
            .unwrap();

        // A SEPARATE read-only handle — the equivalent of the operator's CLI running
        // beside the node — already sees the boundary record.
        let reader = Journal::open_readonly(&path).await.unwrap();
        let visible = reader.pending_attempts().await.unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].attempt_id, attempt_id);
        assert_eq!(visible[0].status, "dispatched");
        assert_eq!(visible[0].provider_request_id.as_deref(), Some("prv_bnd"));
    }

    /// Emitted events deduplicate by event id, and acknowledgement is idempotent —
    /// the ack survives redelivery and the event leaves the pending view exactly once.
    #[tokio::test]
    async fn outgoing_events_and_acknowledgement() {
        let journal = Journal::open(test_path("events")).await.unwrap();
        let (_attempt_id, op) = seed_attempt(&journal, "evt").await;

        let payload = serde_json::json!({ "event_type": "contribution.ready" });
        let first = journal
            .record_outgoing_event("evt_1", &op, &payload, now())
            .await
            .unwrap();
        assert!(!first.already_known);
        let replay = journal
            .record_outgoing_event("evt_1", &op, &payload, now())
            .await
            .unwrap();
        assert!(replay.already_known);

        let pending = journal.pending_events().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event_id, "evt_1");

        assert!(journal
            .acknowledge_event("evt_1", "c42", now())
            .await
            .unwrap());
        assert!(journal.pending_events().await.unwrap().is_empty());
        let counts = journal.counts().await.unwrap();
        assert_eq!(counts.events_emitted, 1);
        assert_eq!(counts.events_acked, 1);
        // Redelivered ack: idempotent, not an error.
        assert!(!journal
            .acknowledge_event("evt_1", "c42", now())
            .await
            .unwrap());
        // Acking an unknown event is NotFound.
        let missing = journal.acknowledge_event("evt_nope", "c42", now()).await;
        assert!(matches!(
            missing,
            Err(JournalError::NotFound { what: "event", .. })
        ));
    }

    /// Deterministic post-dispatch failures are recorded as proven states, never
    /// laundered into ambiguity: `failed_known` from a definitive runtime failure,
    /// `failed_before_dispatch` when the boundary was never crossed.
    #[tokio::test]
    async fn deterministic_failures_are_proven_not_ambiguous() {
        let journal = Journal::open(test_path("failures")).await.unwrap();

        let (a1, _) = seed_attempt(&journal, "fbd").await;
        journal
            .record_failed_before_dispatch(&a1, Some("adapter spawn refused"), now())
            .await
            .unwrap();
        let summary = journal.pending_attempts().await.unwrap();
        assert!(
            summary.iter().all(|a| a.attempt_id != a1),
            "failed_before_dispatch is terminal, not pending"
        );

        let (a2, _) = seed_attempt(&journal, "fk").await;
        journal.record_dispatch(&a2, None, now()).await.unwrap();
        journal
            .record_failed_known(&a2, Some(&serde_json::json!({ "http_status": 429 })), now())
            .await
            .unwrap();
        assert_eq!(
            journal
                .attempt_history(&a2)
                .await
                .unwrap()
                .last()
                .unwrap()
                .to_status,
            "failed_known"
        );
    }

    /// Garbage files and missing files fail cleanly: the journal refuses to open what
    /// is not a journal, and the read-only handle refuses a file with no meta.
    #[tokio::test]
    async fn garbage_and_missing_files_fail_cleanly() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/journal-tests")
            .join(format!("garbage-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
        // Exclusive: an existing directory belongs to another fixture or an
        // earlier run, and must never be adopted.
        std::fs::DirBuilder::new()
            .create(&dir)
            .expect("the fixture directory is new");

        let garbage = dir.join("garbage.db");
        std::fs::write(&garbage, b"this is not a sqlite database").unwrap();
        let err = Journal::open(&garbage).await.unwrap_err();
        assert!(err.to_string().contains("not a database"), "got: {err}");

        let missing = dir.join("absent.db");
        let err = Journal::open_readonly(&missing).await.unwrap_err();
        assert!(matches!(err, JournalError::Sql(_)), "got: {err}");

        // A real journal (with meta) opens read-only fine.
        let journal = Journal::open(dir.join("real.db")).await.unwrap();
        let reader = Journal::open_readonly(journal.path()).await.unwrap();
        assert_eq!(reader.health().await.unwrap().synchronous, "FULL");
    }

    /// The node's acknowledgement cursor: 0 on a fresh journal, a round trip once set,
    /// and an overwrite stays idempotent (the handshake reports the LATEST value).
    #[tokio::test]
    async fn acknowledgement_cursor_round_trips() {
        let journal = Journal::open(test_path("cursor")).await.unwrap();
        assert_eq!(journal.last_acked_cursor().await.unwrap(), 0);
        journal.set_last_acked_cursor(7).await.unwrap();
        assert_eq!(journal.last_acked_cursor().await.unwrap(), 7);
        journal.set_last_acked_cursor(9).await.unwrap();
        assert_eq!(journal.last_acked_cursor().await.unwrap(), 9);
    }

    /// Pending operations are exactly those without a terminal attempt — the "pending
    /// local operation IDs" the reconnect handshake reports.
    #[tokio::test]
    async fn pending_operations_are_those_without_a_terminal_attempt() {
        let journal = Journal::open(test_path("pending-ops")).await.unwrap();
        // op1: no attempt at all → pending.
        let (_attempt_id, op1) = {
            let (attempt_id, op) = seed_attempt(&journal, "p1").await;
            // leave it prepared (non-terminal)
            (attempt_id, op)
        };
        // op2: attempt completed → NOT pending.
        let (_a2, _op2) = {
            let (attempt_id, op) = seed_attempt(&journal, "p2").await;
            journal
                .record_dispatch(&attempt_id, None, now())
                .await
                .unwrap();
            journal
                .record_completed(&attempt_id, None, now())
                .await
                .unwrap();
            (attempt_id, op)
        };
        // op3: ambiguous → still pending (no terminal state).
        let (_a3, _op3) = {
            let (attempt_id, op) = seed_attempt(&journal, "p3").await;
            journal
                .record_dispatch(&attempt_id, None, now())
                .await
                .unwrap();
            journal
                .record_outcome_unknown(&attempt_id, None, now())
                .await
                .unwrap();
            (attempt_id, op)
        };

        let pending = journal.pending_operations().await.unwrap();
        assert_eq!(pending.len(), 2, "got {pending:?}");
        assert!(pending.contains(&op1));
        let ids = journal.operation_ids().await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    /// The `known_events` handshake path: a pending event whose acknowledgement was
    /// lost is marked acknowledged ONLY when both ids match — never an overwrite.
    #[tokio::test]
    async fn known_event_acknowledgement_matches_both_ids() {
        let journal = Journal::open(test_path("known-event")).await.unwrap();
        let (_attempt_id, op) = seed_attempt(&journal, "ke").await;
        let payload = json_for_test();
        journal
            .record_outgoing_event("evt_ke", &op, &payload, now())
            .await
            .unwrap();

        // A known event with a WRONG event id is a no-op (never overwrites).
        let wrong = journal
            .acknowledge_known_event(&op, "evt_other", 5, now())
            .await
            .unwrap();
        assert!(!wrong);
        assert_eq!(journal.pending_events().await.unwrap().len(), 1);

        // The matching pair marks it acknowledged with the cursor.
        let ok = journal
            .acknowledge_known_event(&op, "evt_ke", 5, now())
            .await
            .unwrap();
        assert!(ok);
        assert!(journal.pending_events().await.unwrap().is_empty());

        // Re-applying is a no-op, not an error.
        let again = journal
            .acknowledge_known_event(&op, "evt_ke", 5, now())
            .await
            .unwrap();
        assert!(!again);
    }

    /// Pending events carry their bodies, so the reconnect path can re-emit them with
    /// their original ids and payloads.
    #[tokio::test]
    async fn pending_events_carry_their_payloads() {
        let journal = Journal::open(test_path("event-payload")).await.unwrap();
        let (_attempt_id, op) = seed_attempt(&journal, "ep").await;
        let payload = json_for_test();
        journal
            .record_outgoing_event("evt_ep", &op, &payload, now())
            .await
            .unwrap();
        let pending = journal.pending_events().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event_id, "evt_ep");
        let back: Value = serde_json::from_str(&pending[0].payload).unwrap();
        assert_eq!(back, payload);
    }

    fn json_for_test() -> Value {
        serde_json::json!({ "event_type": "contribution.ready", "note": "re-emit me" })
    }
}
