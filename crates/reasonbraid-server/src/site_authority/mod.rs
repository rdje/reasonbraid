//! Explicit site authority. Tenant enrollment is not an issuer of these grants.
//!
//! All supported site-configuration operations lock the same guard row before
//! reading authority or changing state. Transactions contain database work only.
//! This intentionally serializes the small administrative registry, including
//! revocation, without depending on a particular grant-selection query plan.

mod operator;
mod registry;
mod retention;

pub use operator::{
    disable_boundary, disable_grant, inspect, issue_boundary, issue_grant, Collection,
};
pub use registry::{execute, RegistryCommand};
pub use retention::{expire_evidence, tombstone_evidence};

use std::collections::BTreeSet;
use std::fmt;

use chrono::{DateTime, Utc};
use reasonbraid_core::GrantSubject;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Debug)]
pub enum Error {
    InvalidInput(&'static str),
    OperatorRequired,
    Refused {
        reason: &'static str,
        audit_id: String,
    },
    Sql(sqlx::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(reason) => f.write_str(reason),
            Self::OperatorRequired => f.write_str("explicit database operator authority required"),
            Self::Refused { reason, audit_id } => write!(f, "{reason} (audit {audit_id})"),
            // Connection details belong in controlled diagnostics, not caller responses.
            Self::Sql(_) => f.write_str("site authority database operation failed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sql(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sql(error)
    }
}

/// An opaque registry name. Preserve spelling, including Unicode and punctuation.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct RegistryName(String);

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
            return Err(Error::InvalidInput(
                "registry name must be nonblank, at most 256 bytes, without control characters",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RegistryName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Reason(String);

impl Reason {
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 1024 || value.chars().any(char::is_control) {
            return Err(Error::InvalidInput(
                "reason must be nonblank, at most 1024 bytes, without control characters",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Reason {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum Action {
    RegistryInspect,
    AdapterAllow,
    AdapterRevoke,
    RegionDeclare,
    RegionPair,
    RegionUnpair,
    /// The evidence retention sweep (`SIGNOFF-REPAIR.7.4.3`). Which snapshots
    /// are due is a property of the SHARED row's `retention_class`, not of any
    /// one tenant's citation, so enforcing retention is a site act.
    EvidenceExpire,
}

impl Action {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryInspect => "registry_inspect",
            Self::AdapterAllow => "adapter_allow",
            Self::AdapterRevoke => "adapter_revoke",
            Self::RegionDeclare => "region_declare",
            Self::RegionPair => "region_pair",
            Self::RegionUnpair => "region_unpair",
            Self::EvidenceExpire => "evidence_expire",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub actions: Vec<Action>,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Scope {
    fn checked(&self) -> Result<CheckedScope, Error> {
        // PostgreSQL stores microseconds. Normalize before subset comparisons so
        // an identical parent/child scope does not fail on discarded nanoseconds.
        let valid_from = DateTime::from_timestamp_micros(self.valid_from.timestamp_micros())
            .ok_or(Error::InvalidInput("invalid validity timestamp"))?;
        let expires_at = DateTime::from_timestamp_micros(self.expires_at.timestamp_micros())
            .ok_or(Error::InvalidInput("invalid expiry timestamp"))?;
        if self.actions.is_empty() || expires_at <= valid_from {
            return Err(Error::InvalidInput(
                "scope requires actions and an increasing validity window",
            ));
        }
        let actions = self
            .actions
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|action| action.as_str().to_owned())
            .collect();
        Ok(CheckedScope {
            actions,
            valid_from,
            expires_at,
        })
    }
}

#[derive(Serialize)]
struct CheckedScope {
    actions: Vec<String>,
    valid_from: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// Disable-only transitions. Restoring authority requires a new issuance.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Disable {
    Suspend,
    Revoke,
}

impl Disable {
    fn status(self) -> &'static str {
        match self {
            Self::Suspend => "suspended",
            Self::Revoke => "revoked",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Receipt {
    pub audit_id: String,
    pub result: Value,
}

type Tx<'a> = Transaction<'a, Postgres>;

async fn begin(pool: &PgPool) -> Result<Tx<'_>, Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL lock_timeout = '5s'")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL statement_timeout = '10s'")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

async fn lock(tx: &mut Tx<'_>) -> Result<DateTime<Utc>, Error> {
    sqlx::query("SELECT guard_id FROM public.site_authority_guard WHERE guard_id = 1 FOR UPDATE")
        .fetch_one(&mut **tx)
        .await?;
    // now() would retain the transaction's time from BEFORE a lock wait.
    Ok(sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await?)
}

fn subject_parts(subject: &GrantSubject) -> (&'static str, String) {
    let kind = match subject {
        GrantSubject::Human(_) => "human",
        GrantSubject::Role(_) => "role",
    };
    (kind, subject.id_string())
}

fn identifier(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::now_v7())
}

fn check_id(id: &str, prefix: &str) -> Result<(), Error> {
    let suffix = id
        .strip_prefix(prefix)
        .and_then(|value| value.strip_prefix('_'));
    if !suffix.is_some_and(|value| value.len() == 36 && uuid::Uuid::parse_str(value).is_ok()) {
        return Err(Error::InvalidInput("invalid site authority identifier"));
    }
    Ok(())
}

struct Intent {
    actor_kind: &'static str,
    actor: String,
    action: &'static str,
    target: Value,
    requested_reason: String,
}

struct Outcome {
    grant_id: Option<String>,
    boundary_id: Option<String>,
    outcome: &'static str,
    reason: &'static str,
    evaluation: Value,
    at: DateTime<Utc>,
}

/// What an authorized site act produced: the receipt body and the audit's
/// outcome word (`applied`, `noop` or `inspected`).
struct Effect {
    result: Value,
    outcome: &'static str,
}

impl Effect {
    fn write(result: Value, changed: u64) -> Self {
        Self {
            result,
            outcome: if changed == 0 { "noop" } else { "applied" },
        }
    }
}

/// One ordered site transaction, shared by every site act
/// (`SIGNOFF-REPAIR.7.4.5`).
///
/// Take the guard, read the clock AFTER the lock wait, evaluate the subject's
/// grants for `action`, and then either commit an audited denial or run
/// `effect` and audit what it produced. The effect, the denial and the audit
/// record all commit together: an audit failure rolls back an otherwise
/// allowed write, and a denial still commits its own record.
///
/// `effect` runs ONLY once a live grant on its actual boundary has been found.
/// It returns `Err(reason)` for a DOMAIN refusal — the caller held the grant
/// and asked for something the subsystem cannot do — which is audited as
/// `denied` WITH the grant and boundary attached, and surfaced as a typed
/// `Refused` the HTTP layer maps to its own status.
///
/// ⛔ The clock passed to `effect` is the database's own `clock_timestamp()`
/// read after the guard lock, never `now()` and never a caller's: a site act
/// that stamps rows with a time is making a factual claim about them
/// (`SIGNOFF-REPAIR.7.4.3`).
///
/// This was two copies until `.7.4.5`. `.7.4.3` created the second one
/// deliberately rather than refactor an authorization path inside the commit
/// that repaired a hole in it, and opened the leaf that merged them.
async fn authorized<F>(
    pool: &PgPool,
    subject: &GrantSubject,
    action: Action,
    target: Value,
    requested_reason: &str,
    effect: F,
) -> Result<Receipt, Error>
where
    F: for<'c> FnOnce(
        &'c mut sqlx::PgConnection,
        DateTime<Utc>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Result<Effect, &'static str>, Error>>
                + Send
                + 'c,
        >,
    >,
{
    let mut tx = begin(pool).await?;
    let at = lock(&mut tx).await?;
    let (actor_kind, actor) = subject_parts(subject);
    let intent = Intent {
        actor_kind,
        actor,
        action: action.as_str(),
        target,
        requested_reason: requested_reason.to_owned(),
    };
    let evaluation = evaluate(&mut tx, subject, action, at).await?;
    let Some(grant_id) = evaluation.grant_id else {
        let audit_id = audit(
            &mut tx,
            &intent,
            Outcome {
                grant_id: None,
                boundary_id: None,
                outcome: "denied",
                reason: "site_authority_required",
                evaluation: evaluation.checks,
                at,
            },
        )
        .await?;
        tx.commit().await?;
        return Err(Error::Refused {
            reason: "site_authority_required",
            audit_id,
        });
    };
    // A domain refusal is separate from a SQL error: an unavailable database
    // must never masquerade as one, so the closure returns the two distinctly.
    let produced = effect(&mut tx, at).await?;
    let (outcome, reason) = match &produced {
        Ok(effect) => (effect.outcome, effect.outcome),
        Err(refusal) => ("denied", *refusal),
    };
    let audit_id = audit(
        &mut tx,
        &intent,
        Outcome {
            grant_id: Some(grant_id),
            boundary_id: evaluation.boundary_id,
            outcome,
            reason,
            evaluation: evaluation.checks,
            at,
        },
    )
    .await?;
    tx.commit().await?;
    match produced {
        Ok(effect) => Ok(Receipt {
            audit_id,
            result: effect.result,
        }),
        Err(reason) => Err(Error::Refused { reason, audit_id }),
    }
}

async fn audit(tx: &mut Tx<'_>, intent: &Intent, outcome: Outcome) -> Result<String, Error> {
    let id = identifier("sau");
    sqlx::query(
        "INSERT INTO public.site_audit \
         (audit_id, actor_kind, actor, action, target, grant_id, boundary_id, requested_reason, \
          decision, outcome, reason, evaluation, decided_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
    )
    .bind(&id)
    .bind(intent.actor_kind)
    .bind(&intent.actor)
    .bind(intent.action)
    .bind(&intent.target)
    .bind(outcome.grant_id)
    .bind(outcome.boundary_id)
    .bind(&intent.requested_reason)
    .bind(if outcome.outcome == "denied" {
        "denied"
    } else {
        "allowed"
    })
    .bind(outcome.outcome)
    .bind(outcome.reason)
    .bind(outcome.evaluation)
    .bind(outcome.at)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

#[derive(sqlx::FromRow)]
struct Candidate {
    grant_id: String,
    boundary_id: String,
    actions: Vec<String>,
    valid_from: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    status: String,
    boundary_actions: Vec<String>,
    boundary_valid_from: DateTime<Utc>,
    boundary_expires_at: DateTime<Utc>,
    boundary_status: String,
}

impl Candidate {
    fn refusal(&self, action: Action, at: DateTime<Utc>) -> Option<&'static str> {
        if self.status != "active" || at < self.valid_from || at >= self.expires_at {
            Some("grant_not_current")
        } else if self.boundary_status != "active"
            || at < self.boundary_valid_from
            || at >= self.boundary_expires_at
        {
            Some("boundary_not_current")
        } else if self.valid_from < self.boundary_valid_from
            || self.expires_at > self.boundary_expires_at
            || !self
                .actions
                .iter()
                .all(|action| self.boundary_actions.contains(action))
        {
            Some("grant_exceeds_boundary")
        } else if !self
            .actions
            .iter()
            .any(|allowed| allowed == action.as_str())
        {
            Some("action_not_granted")
        } else {
            None
        }
    }
}

struct Evaluation {
    grant_id: Option<String>,
    boundary_id: Option<String>,
    checks: Value,
}

async fn evaluate(
    tx: &mut Tx<'_>,
    subject: &GrantSubject,
    action: Action,
    at: DateTime<Utc>,
) -> Result<Evaluation, Error> {
    let (kind, id) = subject_parts(subject);
    let candidates: Vec<Candidate> = sqlx::query_as(
        "SELECT g.grant_id, g.boundary_id, g.actions, g.valid_from, g.expires_at, g.status, \
                b.actions AS boundary_actions, b.valid_from AS boundary_valid_from, \
                b.expires_at AS boundary_expires_at, b.status AS boundary_status \
         FROM public.site_grants g JOIN public.site_boundaries b ON b.boundary_id = g.boundary_id \
         WHERE g.subject_kind = $1 AND g.subject_id = $2 ORDER BY g.grant_id",
    )
    .bind(kind)
    .bind(id)
    .fetch_all(&mut **tx)
    .await?;
    let mut checks = Vec::new();
    for candidate in candidates {
        let refusal = candidate.refusal(action, at);
        checks.push(json!({"grant_id": candidate.grant_id, "boundary_id": candidate.boundary_id, "result": refusal.unwrap_or("allowed")}));
        if refusal.is_none() {
            return Ok(Evaluation {
                grant_id: Some(candidate.grant_id),
                boundary_id: Some(candidate.boundary_id),
                checks: json!(checks),
            });
        }
    }
    Ok(Evaluation {
        grant_id: None,
        boundary_id: None,
        checks: json!(checks),
    })
}
