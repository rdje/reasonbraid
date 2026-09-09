//! Explicit site authority. Tenant enrollment is not an issuer of these grants.
//!
//! All supported site-configuration operations lock the same guard row before
//! reading authority or changing state. Transactions contain database work only.
//! This intentionally serializes the small administrative registry, including
//! revocation, without depending on a particular grant-selection query plan.

mod operator;
mod registry;

pub use operator::{disable_boundary, disable_grant, issue_boundary, issue_grant};
pub use registry::{execute, RegistryCommand};

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
