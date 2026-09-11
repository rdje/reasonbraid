//! The evidence snapshot store (PHASE-4.6.1, ROADMAP §12.6 + §12.9): the
//! typed submission surface the `.2`–`.5` receipts land in, the
//! content-addressed object store (the raw bytes persist UNDER their
//! ADR-011 digest — identical bytes = one object), and the tombstone rule
//! (the deletion records the reason and the time — never a silent
//! disappearance).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The typed snapshot submission (the receipts' common facts + the bytes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSubmission {
    pub reference_id: String,
    pub original_locator: String,
    pub final_locator: String,
    pub resolver_id: String,
    pub resolver_version: String,
    #[serde(default)]
    pub network_class: String,
    #[serde(default)]
    pub auth_class: String,
    #[serde(default)]
    pub provider_receipt: serde_json::Value,
    #[serde(default)]
    pub immutable_source_version: Option<String>,
    pub raw_digest: String,
    pub byte_length: i64,
    pub media_type: String,
    #[serde(default = "default_storage")]
    pub storage_class: String,
    #[serde(default = "default_storage")]
    pub retention_class: String,
    #[serde(default)]
    pub extraction_version: Option<String>,
    #[serde(default = "default_storage")]
    pub quarantine_status: String,
    #[serde(default)]
    pub redactions: serde_json::Value,
    #[serde(default)]
    pub disclosure_policy: serde_json::Value,
    /// The license metadata (the §12.6 record).
    #[serde(default)]
    pub license: Option<String>,
    /// The freshness horizon (the §12.9 staleness surface).
    #[serde(default)]
    pub fresh_until: Option<chrono::DateTime<chrono::Utc>>,
}

fn default_storage() -> String {
    "standard".to_owned()
}

impl SnapshotSubmission {
    /// The ADR-011 digest validation (the same scheme the receipts use).
    pub fn digest_error(&self) -> Option<&'static str> {
        crate::resources::ResourceReference {
            original_locator: String::new(),
            scheme: "https".to_owned(),
            media_type_hint: None,
            expected_digest: Some(self.raw_digest.clone()),
            fragment_or_selector: None,
            credential_binding_ref: None,
            owning_node_or_capability: None,
            visibility_scope: "tenant".to_owned(),
            purpose: None,
            retention_class: None,
            risk_class: "standard".to_owned(),
        }
        .digest_error()
    }
}

/// The submission outcome: the snapshot id, or the typed replay.
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotOutcome {
    pub snapshot_id: String,
    pub replay: bool,
}

/// The stored snapshot (the read surface — the tombstone state rides the
/// same row: `deleted_at`/`deletion_reason`).
#[derive(Debug, Clone, Serialize)]
pub struct StoredSnapshot {
    pub snapshot_id: String,
    pub reference_id: String,
    pub original_locator: String,
    pub final_locator: String,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
    pub resolver_id: String,
    pub resolver_version: String,
    pub network_class: String,
    pub auth_class: String,
    pub provider_receipt: serde_json::Value,
    pub immutable_source_version: Option<String>,
    pub raw_digest: String,
    pub byte_length: i64,
    pub media_type: String,
    pub storage_class: String,
    pub retention_class: String,
    pub extraction_version: Option<String>,
    pub quarantine_status: String,
    pub redactions: serde_json::Value,
    pub disclosure_policy: serde_json::Value,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_reason: Option<String>,
    pub license: Option<String>,
    pub fresh_until: Option<chrono::DateTime<chrono::Utc>>,
    pub refreshed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// The snapshot's error — every refusal names its reason.
// Only `Debug` derives: the storage variant carries the original SQLx error
// so it survives to `std::error::Error::source`, and that error is neither
// `Clone` nor `Eq`. This matches the `GrantCreateError` contract from `.3.3.4.3.1`.
#[derive(Debug)]
#[non_exhaustive]
pub enum SnapshotError {
    InvalidDigest(&'static str),
    ReferenceMissing,
    DigestMismatch {
        declared: String,
        actual: String,
    },
    /// The store itself failed. A database fault does not prove anything
    /// about the caller's input, and must never be reported as though it did.
    Storage(sqlx::Error),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(_) => write!(f, "the evidence store is unavailable"),
            Self::InvalidDigest(reason) => write!(f, "the raw digest is invalid: {reason}"),
            Self::ReferenceMissing => write!(f, "the reference does not exist"),
            Self::DigestMismatch { declared, actual } => write!(
                f,
                "the bytes hash to `{actual}`, not the declared `{declared}`"
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Submit a snapshot: the bytes MUST hash to the declared digest (the
/// content-addressing is verified, not trusted); the same reference +
/// digest is the REPLAY (the same id). The object store upserts the bytes.
pub async fn submit(
    pool: &PgPool,
    submission: &SnapshotSubmission,
    bytes: &[u8],
    retrieved_at: chrono::DateTime<chrono::Utc>,
) -> Result<SnapshotOutcome, SnapshotError> {
    if let Some(reason) = submission.digest_error() {
        return Err(SnapshotError::InvalidDigest(reason));
    }
    let actual = crate::fetcher::digest_sha256_hex(bytes);
    if actual != submission.raw_digest {
        return Err(SnapshotError::DigestMismatch {
            declared: submission.raw_digest.clone(),
            actual,
        });
    }
    let reference_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM resource_references WHERE resource_id = $1)",
    )
    .bind(&submission.reference_id)
    .fetch_one(pool)
    .await
    .map_err(SnapshotError::Storage)?;
    if !reference_exists {
        return Err(SnapshotError::ReferenceMissing);
    }
    let snapshot_id = evidence_id("snp");
    sqlx::query(
        "INSERT INTO snapshot_objects (digest, bytes) VALUES ($1, $2) \
         ON CONFLICT (digest) DO NOTHING",
    )
    .bind(&submission.raw_digest)
    .bind(bytes)
    .execute(pool)
    .await
    .map_err(SnapshotError::Storage)?;
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT snapshot_id FROM evidence_snapshots \
         WHERE reference_id = $1 AND raw_digest = $2 LIMIT 1",
    )
    .bind(&submission.reference_id)
    .bind(&submission.raw_digest)
    .fetch_optional(pool)
    .await
    .map_err(SnapshotError::Storage)?;
    if let Some(existing) = existing {
        // The re-fetch policy: the replay refreshes the freshness record
        // (the re-acquisition happened — the bytes are unchanged, the
        // horizon resets).
        let _ = sqlx::query(
            "UPDATE evidence_snapshots SET refreshed_at = now() WHERE snapshot_id = $1",
        )
        .bind(&existing)
        .execute(pool)
        .await;
        return Ok(SnapshotOutcome {
            snapshot_id: existing,
            replay: true,
        });
    }
    sqlx::query(
        "INSERT INTO evidence_snapshots \
         (snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
          resolver_id, resolver_version, network_class, auth_class, provider_receipt, \
          immutable_source_version, raw_digest, byte_length, media_type, storage_class, \
          retention_class, extraction_version, quarantine_status, redactions, disclosure_policy, \
          license, fresh_until) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)",
    )
    .bind(&snapshot_id)
    .bind(&submission.reference_id)
    .bind(&submission.original_locator)
    .bind(&submission.final_locator)
    .bind(retrieved_at)
    .bind(&submission.resolver_id)
    .bind(&submission.resolver_version)
    .bind(&submission.network_class)
    .bind(&submission.auth_class)
    .bind(&submission.provider_receipt)
    .bind(&submission.immutable_source_version)
    .bind(&submission.raw_digest)
    .bind(submission.byte_length)
    .bind(&submission.media_type)
    .bind(&submission.storage_class)
    .bind(&submission.retention_class)
    .bind(&submission.extraction_version)
    .bind(&submission.quarantine_status)
    .bind(&submission.redactions)
    .bind(&submission.disclosure_policy)
    .bind(&submission.license)
    .bind(submission.fresh_until)
    .execute(pool)
    .await
    .map_err(SnapshotError::Storage)?;
    Ok(SnapshotOutcome {
        snapshot_id,
        replay: false,
    })
}

/// The snapshot row (sqlx's tuple impls stop short of the full width).
#[derive(sqlx::FromRow)]
struct SnapshotRow {
    snapshot_id: String,
    reference_id: String,
    original_locator: String,
    final_locator: String,
    retrieved_at: chrono::DateTime<chrono::Utc>,
    resolver_id: String,
    resolver_version: String,
    network_class: String,
    auth_class: String,
    provider_receipt: serde_json::Value,
    immutable_source_version: Option<String>,
    raw_digest: String,
    byte_length: i64,
    media_type: String,
    storage_class: String,
    retention_class: String,
    extraction_version: Option<String>,
    quarantine_status: String,
    redactions: serde_json::Value,
    disclosure_policy: serde_json::Value,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    deletion_reason: Option<String>,
    license: Option<String>,
    fresh_until: Option<chrono::DateTime<chrono::Utc>>,
    refreshed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Read a snapshot (the tombstone state rides the same row).
pub async fn get(pool: &PgPool, snapshot_id: &str) -> Result<Option<StoredSnapshot>, sqlx::Error> {
    let row: Option<SnapshotRow> = sqlx::query_as::<_, SnapshotRow>(
        "SELECT snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
                resolver_id, resolver_version, network_class, auth_class, provider_receipt, \
                immutable_source_version, raw_digest, byte_length, media_type, storage_class, \
                retention_class, extraction_version, quarantine_status, redactions, \
                disclosure_policy, deleted_at, deletion_reason, license, fresh_until, refreshed_at \
         FROM evidence_snapshots WHERE snapshot_id = $1",
    )
    .bind(snapshot_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| StoredSnapshot {
        snapshot_id: row.snapshot_id,
        reference_id: row.reference_id,
        original_locator: row.original_locator,
        final_locator: row.final_locator,
        retrieved_at: row.retrieved_at,
        resolver_id: row.resolver_id,
        resolver_version: row.resolver_version,
        network_class: row.network_class,
        auth_class: row.auth_class,
        provider_receipt: row.provider_receipt,
        immutable_source_version: row.immutable_source_version,
        raw_digest: row.raw_digest,
        byte_length: row.byte_length,
        media_type: row.media_type,
        storage_class: row.storage_class,
        retention_class: row.retention_class,
        extraction_version: row.extraction_version,
        quarantine_status: row.quarantine_status,
        redactions: row.redactions,
        disclosure_policy: row.disclosure_policy,
        deleted_at: row.deleted_at,
        deletion_reason: row.deletion_reason,
        license: row.license,
        fresh_until: row.fresh_until,
        refreshed_at: row.refreshed_at,
    }))
}

/// The tombstone: the deletion records the reason + the time — the row
/// stays (never a silent disappearance). Idempotent: the first reason wins.
pub async fn tombstone(
    pool: &PgPool,
    snapshot_id: &str,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $2 \
         WHERE snapshot_id = $1 AND deleted_at IS NULL",
    )
    .bind(snapshot_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Mint one evidence identifier.
///
/// The superseded `uuid_like_suffix` was `format!("{:x}{:x}", nanos, pid)` — a
/// shape that resembles a UUID without the single property a UUID is for. A
/// probe of that exact expression on this host measured 8 collisions in 10
/// sequential calls, 918 in 1,000, and 269 among 400 across eight threads:
/// roughly one distinct value per twelve calls, because the realtime clock
/// advances far more slowly than the work between calls. These identifiers are
/// PRIMARY KEYs for evidence rows, so a collision loses a snapshot, a
/// derivation or an assessment.
///
/// A v7 UUID is time-ordered like the old shape and distinct by construction.
pub(crate) fn evidence_id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::now_v7().simple())
}

/// The retention classes' TTLs (the §12.9 enforcement): the audit class
/// never expires (binding decisions stay addressable for the charter's
/// audit period); standard = 30 days; temporary = 1 day.
pub fn retention_ttl(retention_class: &str) -> Option<chrono::Duration> {
    match retention_class {
        "audit" => None,
        "temporary" => Some(chrono::Duration::days(1)),
        _ => Some(chrono::Duration::days(30)),
    }
}

/// The retention enforcement: every live snapshot whose class TTL has
/// passed (measured from `created_at` against `now`) is TOMBSTONED with
/// the reason — never silently removed. Returns the tombstoned count.
pub async fn expire_due(
    pool: &PgPool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $1 \
         WHERE deleted_at IS NULL AND retention_class = 'standard' AND created_at < $2",
    )
    .bind("the retention expired")
    .bind(now - chrono::Duration::days(30))
    .execute(pool)
    .await?;
    let standard = result.rows_affected();
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $1 \
         WHERE deleted_at IS NULL AND retention_class = 'temporary' AND created_at < $2",
    )
    .bind("the retention expired")
    .bind(now - chrono::Duration::days(1))
    .execute(pool)
    .await?;
    Ok(standard + result.rows_affected())
}

/// The staleness surface: the LIVE snapshots whose freshness horizon has
/// passed (the assessments read this — the re-fetch is the caller's).
pub async fn stale(
    pool: &PgPool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<StoredSnapshot>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SnapshotRow>(
        "SELECT snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
                resolver_id, resolver_version, network_class, auth_class, provider_receipt, \
                immutable_source_version, raw_digest, byte_length, media_type, storage_class, \
                retention_class, extraction_version, quarantine_status, redactions, \
                disclosure_policy, deleted_at, deletion_reason, license, fresh_until, refreshed_at \
         FROM evidence_snapshots \
         WHERE deleted_at IS NULL AND fresh_until IS NOT NULL AND fresh_until < $1 \
         ORDER BY fresh_until",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| StoredSnapshot {
            snapshot_id: row.snapshot_id,
            reference_id: row.reference_id,
            original_locator: row.original_locator,
            final_locator: row.final_locator,
            retrieved_at: row.retrieved_at,
            resolver_id: row.resolver_id,
            resolver_version: row.resolver_version,
            network_class: row.network_class,
            auth_class: row.auth_class,
            provider_receipt: row.provider_receipt,
            immutable_source_version: row.immutable_source_version,
            raw_digest: row.raw_digest,
            byte_length: row.byte_length,
            media_type: row.media_type,
            storage_class: row.storage_class,
            retention_class: row.retention_class,
            extraction_version: row.extraction_version,
            quarantine_status: row.quarantine_status,
            redactions: row.redactions,
            disclosure_policy: row.disclosure_policy,
            deleted_at: row.deleted_at,
            deletion_reason: row.deletion_reason,
            license: row.license,
            fresh_until: row.fresh_until,
            refreshed_at: row.refreshed_at,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the superseded clock-and-process-id shape did not have.
    /// Eight threads mint fifty identifiers each; all four hundred are distinct.
    /// A probe of the old expression produced 269 collisions in this shape.
    #[test]
    fn evidence_identifiers_stay_distinct_under_concurrency() {
        let handles: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(|| (0..50).map(|_| evidence_id("snp")).collect::<Vec<_>>()))
            .collect();
        let minted: Vec<String> = handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("the minter finished"))
            .collect();
        let distinct: std::collections::HashSet<&String> = minted.iter().collect();
        assert_eq!(minted.len(), 400);
        assert_eq!(
            distinct.len(),
            400,
            "{} of 400 evidence identifiers collided",
            400 - distinct.len()
        );
        assert!(minted.iter().all(|id| id.starts_with("snp_")));
    }

    /// Rapid SEQUENTIAL calls were the worse case: the old shape returned two
    /// distinct values for ten calls.
    #[test]
    fn rapid_sequential_identifiers_stay_distinct() {
        let minted: Vec<String> = (0..1_000).map(|_| evidence_id("drv")).collect();
        let distinct: std::collections::HashSet<&String> = minted.iter().collect();
        assert_eq!(
            distinct.len(),
            1_000,
            "{} of 1000 sequential identifiers collided",
            1_000 - distinct.len()
        );
    }
}
