//! The evidence snapshot store (PHASE-4.6.1, ROADMAP §12.6 + §12.9): the
//! typed submission surface the `.2`–`.5` receipts land in, the
//! content-addressed object store (the raw bytes persist UNDER their
//! ADR-011 digest — identical bytes = one object), and the tombstone rule
//! (the deletion records the reason and the time — never a silent
//! disappearance).
//!
//! A snapshot names the SAME `original_locator` as the reference it is filed
//! against (`SIGNOFF-REPAIR.11.14.3.13`) — it is an acquisition OF that
//! reference, so the two cannot disagree. `final_locator` stays free, because a
//! redirect legitimately ends somewhere else.
//!
//! A snapshot is filed against a reference THIS TENANT REGISTERED
//! (`SIGNOFF-REPAIR.11.14.3.11`). A reference it did not register answers the
//! same `ReferenceMissing` an absent id gets, so the write surface stops being
//! an existence oracle over `res_…` ids — the binding the READ surfaces already
//! carry.
//!
//! A reference's `expected_digest` is ENFORCED here since
//! `SIGNOFF-REPAIR.11.14.3.6`: a pinned reference accepts only the bytes it
//! names, while an unpinned one still holds every version §12.6's changing page
//! produces. The two compose because `.11.14.3.2` made `(locator, digest)` the
//! reference's identity, so a changed page is a SECOND reference.
//!
//! The row is SHARED and the read is TENANT-BOUND (`SIGNOFF-REPAIR.11.14.1`,
//! `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`).
//! Content-addressing makes one row serve every tenant that cites the same
//! bytes, so ROADMAP §16.8's tenant is carried by the disclosure DECISION
//! rather than by a column: `evidence_citations` records which tenants cited
//! a snapshot, `submit` writes that citation on the fresh insert AND on the
//! replay, and every read surface here is bound to it. There is deliberately
//! no unbound read in this module — a caller cannot ask for a snapshot
//! without naming the tenant the answer is for.

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

/// Who cited a snapshot: the tenant whose evidence reads will show the row,
/// and the actor handle that asked for the acquisition.
///
/// This is the §16.8 authorization input the evidence chain was missing. It
/// is a citation and not an owner: the same row is legitimately cited by
/// several tenants, because `resource_references` is UNIQUE on
/// `(original_locator, expected_digest)` and `snapshot_objects` is keyed by
/// digest alone — two tenants acquiring the same bytes share one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citer {
    /// The citing tenant — the read surfaces compare against this.
    pub tenant_id: String,
    /// The actor handle that asked (recorded for the audit trail; the
    /// disclosure decision is the TENANT's).
    pub principal: String,
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
    /// The submission names a different `original_locator` from the reference
    /// it is filed against (`SIGNOFF-REPAIR.11.14.3.13`).
    ///
    /// Both values are carried: this check runs AFTER the registration
    /// predicate, so the caller has already proved it registered the reference
    /// and may read that locator — quoting it discloses nothing it does not
    /// hold, and the diagnosis is worth more than the symmetry.
    LocatorMismatch {
        submitted: String,
        reference: String,
    },
    /// The reference declares an `expected_digest` and these bytes are not it
    /// (`SIGNOFF-REPAIR.11.14.3.6`).
    ///
    /// ⛔ Neither digest is carried. The caller already holds the ACTUAL one —
    /// it hashed the bytes it sent — and the PINNED one belongs to a reference
    /// this route does not check the caller may read, so quoting it would make
    /// the refusal an oracle over a `res_…` id. The message says what to do
    /// instead, which is the part a legitimate caller does not already have.
    PinMismatch,
    /// The store itself failed. A database fault does not prove anything
    /// about the caller's input, and must never be reported as though it did.
    Storage(sqlx::Error),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(_) => write!(f, "the evidence store is unavailable"),
            Self::InvalidDigest(reason) => write!(f, "the raw digest is invalid: {reason}"),
            // ⛔ One sentence for two cases — absent, and registered by someone
            // else — because separating them is the oracle
            // (`SIGNOFF-REPAIR.11.14.3.11`).
            Self::ReferenceMissing => write!(
                f,
                "the reference does not exist, or this tenant did not register it"
            ),
            Self::DigestMismatch { declared, actual } => write!(
                f,
                "the bytes hash to `{actual}`, not the declared `{declared}`"
            ),
            Self::LocatorMismatch {
                submitted,
                reference,
            } => write!(
                f,
                "this snapshot names `{submitted}` and its reference names \
                 `{reference}` — a snapshot is an acquisition OF its reference, \
                 so the two cannot disagree (`final_locator` is where a redirect \
                 ended and is free)"
            ),
            Self::PinMismatch => write!(
                f,
                "the bytes do not match the digest this reference is pinned to — \
                 a page that changed is a SECOND reference (§12.6), so register \
                 the locator at the new digest and acquire against that"
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Submit a snapshot: the bytes MUST hash to the declared digest (the
/// content-addressing is verified, not trusted); the same reference +
/// digest is the REPLAY (the same id). The object store upserts the bytes.
///
/// The citation is recorded on BOTH outcomes. A replay is a genuine second
/// acquisition by this tenant, and it is the only record that the shared row
/// belongs on that tenant's read surfaces — skipping it there would hide a
/// row from one of its own authors, which is the trap `SIGNOFF-REPAIR.6.1.5`
/// names and the reason a read-side predicate alone was rejected.
///
/// A store fault between the snapshot write and the citation write leaves the
/// row cited by nobody, which is fail-CLOSED: it discloses nothing, the caller
/// is told the store failed, and the next acquisition of the same bytes takes
/// the replay path and records the citation.
pub async fn submit(
    pool: &PgPool,
    submission: &SnapshotSubmission,
    bytes: &[u8],
    retrieved_at: chrono::DateTime<chrono::Utc>,
    citer: &Citer,
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
    // The reference must exist, THIS TENANT must have registered it, AND its
    // pin must hold. One query answers all three: the outer `Option` is the
    // first two together, the inner one is the pin.
    //
    // ⛔ The registration predicate is in the SAME statement as the lookup
    // (`SIGNOFF-REPAIR.11.14.3.11`), so a reference the caller did not register
    // is indistinguishable from one that does not exist — both are
    // `ReferenceMissing`, and no new error exists for the distinction to leak
    // through. Before it, `POST /v1/snapshots` admitted on enrolment alone and a
    // caller holding a `res_…` id could tell the two apart.
    //
    // ⛔ And the pin (`SIGNOFF-REPAIR.11.14.3.6`): before it, the lookup asked
    // `EXISTS` and the §12.1 field a caller supplied to say "these are the bytes
    // I expect" constrained nothing — a reference pinned to one digest accepted
    // a snapshot of entirely different bytes.
    let pin: Option<(Option<String>, String)> = sqlx::query_as(
        "SELECT r.expected_digest, r.original_locator FROM resource_references r \
         JOIN reference_registrations g \
           ON g.resource_id = r.resource_id AND g.tenant_id = $2 \
         WHERE r.resource_id = $1",
    )
    .bind(&submission.reference_id)
    .bind(&citer.tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(SnapshotError::Storage)?;
    let Some((pin, reference_locator)) = pin else {
        return Err(SnapshotError::ReferenceMissing);
    };
    // ⛔ A snapshot is an acquisition OF its reference, so it cannot name a
    // different document (`SIGNOFF-REPAIR.11.14.3.13`). The comparison is BYTE
    // equality on purpose: §12.1 keeps the original locator immutable and
    // canonicalization separate and scheme-specific, so normalising either side
    // here would BE a canonicalization decision rather than a check. Refusing a
    // disagreement erases nothing.
    //
    // ⚠️ `final_locator` is deliberately untouched — a redirect legitimately
    // ends somewhere else, which is why the receipt records both.
    if submission.original_locator != reference_locator {
        return Err(SnapshotError::LocatorMismatch {
            submitted: submission.original_locator.clone(),
            reference: reference_locator,
        });
    }
    // ⭐ An UNPINNED reference is unaffected, and that is the whole shape of the
    // rule rather than an exemption. `evidence_snapshots` replays on
    // `(reference_id, raw_digest)`, so one reference is designed to hold many
    // versions — which is what §12.6's changing page needs. A PIN says the
    // opposite about its own reference: these bytes, this row. The two compose
    // because `SIGNOFF-REPAIR.11.14.3.2` made `(locator, digest)` the identity,
    // so the changed page is a second reference rather than a second version of
    // the pinned one.
    if let Some(pinned) = pin {
        if pinned != submission.raw_digest {
            return Err(SnapshotError::PinMismatch);
        }
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
        record_citation(pool, &existing, citer)
            .await
            .map_err(SnapshotError::Storage)?;
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
    record_citation(pool, &snapshot_id, citer)
        .await
        .map_err(SnapshotError::Storage)?;
    Ok(SnapshotOutcome {
        snapshot_id,
        replay: false,
    })
}

/// Record one citation — idempotent, so a re-acquisition by the same tenant
/// keeps the original time and actor.
///
/// A tenant that WITHDREW (`SIGNOFF-REPAIR.7.4.4`) and cites again is restored
/// rather than refused: the upsert clears the withdrawal and deliberately
/// leaves `cited_at`/`cited_by` alone. The original citation time and actor are
/// the durable fact; a withdrawal is an episode in that row's life, not a new
/// row. Clearing three already-NULL columns on an ordinary replay is a no-op.
async fn record_citation(
    pool: &PgPool,
    snapshot_id: &str,
    citer: &Citer,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO evidence_citations (snapshot_id, tenant_id, cited_by) \
         VALUES ($1, $2, $3) ON CONFLICT (snapshot_id, tenant_id) DO UPDATE \
         SET withdrawn_at = NULL, withdrawn_by = NULL, withdrawal_reason = NULL",
    )
    .bind(snapshot_id)
    .bind(&citer.tenant_id)
    .bind(&citer.principal)
    .execute(pool)
    .await?;
    Ok(())
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

impl From<SnapshotRow> for StoredSnapshot {
    fn from(row: SnapshotRow) -> Self {
        Self {
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
        }
    }
}

/// The stored snapshot's columns — one definition, so the two bound reads
/// cannot drift into different row shapes.
const SNAPSHOT_COLUMNS: &str =
    "snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
     resolver_id, resolver_version, network_class, auth_class, provider_receipt, \
     immutable_source_version, raw_digest, byte_length, media_type, storage_class, \
     retention_class, extraction_version, quarantine_status, redactions, \
     disclosure_policy, deleted_at, deletion_reason, license, fresh_until, refreshed_at";

/// The disclosure predicate (§16.8): a snapshot is readable by a tenant that
/// CITED it and by no other. `$2` is the tenant in both bound reads, so this
/// fragment carries its own parameter index.
///
/// A WITHDRAWN citation is not a citation (`SIGNOFF-REPAIR.7.4.4`). The row is
/// kept so the withdrawal stays auditable, so every disclosure surface must say
/// `withdrawn_at IS NULL` — omitting it on one surface would leave a tenant
/// reading evidence it has declared it no longer relies on.
const CITED_BY_TENANT: &str = "EXISTS (SELECT 1 FROM evidence_citations c \
     WHERE c.snapshot_id = evidence_snapshots.snapshot_id AND c.tenant_id = $2 \
       AND c.withdrawn_at IS NULL)";

/// Read a snapshot this tenant cited (the tombstone state rides the same row).
///
/// A row the tenant did not cite reads as ABSENT rather than forbidden: a
/// refusal that distinguishes the two would confirm that an identifier the
/// caller may not read exists, which is the enumeration this binding closes.
pub async fn get_for_tenant(
    pool: &PgPool,
    snapshot_id: &str,
    tenant_id: &str,
) -> Result<Option<StoredSnapshot>, sqlx::Error> {
    let row: Option<SnapshotRow> = sqlx::query_as::<_, SnapshotRow>(&format!(
        "SELECT {SNAPSHOT_COLUMNS} FROM evidence_snapshots \
         WHERE snapshot_id = $1 AND {CITED_BY_TENANT}"
    ))
    .bind(snapshot_id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(StoredSnapshot::from))
}

/// Whether this tenant cited the snapshot — the gate the CHILD reads
/// (`/derivations`, `/assessments`) apply to their parent before disclosing
/// anything about it, including whether it exists.
/// Generic over the executor so a deliberation can ask this question INSIDE
/// its own transaction (`SIGNOFF-REPAIR.11.14.3.1`): an `assess` contribution
/// and the assessment row it records commit together, so the citation it was
/// admitted on cannot be withdrawn between the check and the write.
pub async fn is_cited_by<'e, E>(
    mut executor: E,
    snapshot_id: &str,
    tenant_id: &str,
) -> Result<bool, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evidence_citations \
         WHERE snapshot_id = $1 AND tenant_id = $2 AND withdrawn_at IS NULL)",
    )
    .bind(snapshot_id)
    .bind(tenant_id)
    .fetch_one(&mut *executor)
    .await
}

/// Withdraw THIS tenant's citation: it stops relying on the snapshot, and the
/// shared row is untouched (`SIGNOFF-REPAIR.7.4.4`).
///
/// ⛔ This is the act `DELETE /v1/snapshots/{id}` performs, and the one a tenant
/// owns. Tombstoning the row says something about bytes other tenants may cite,
/// which is a site-operator act — `tombstone` below, and the `expire_due` sweep.
///
/// Recorded rather than deleted, per §12.9: a `DELETE FROM evidence_citations`
/// would leave no answer to *who stopped relying on this, when, and why*.
/// Idempotent — the first withdrawal's reason and time win, so a repeated call
/// returns `false` and rewrites nothing.
///
/// ⭐ The last citer withdrawing does NOT tombstone the row, deliberately. An
/// uncited snapshot is simply read by nobody until it is cited again — exactly
/// the state `migrations/0062` describes for every row written before citations
/// existed — and the retention sweep reaps it on its own class TTL. Tombstoning
/// on the last withdrawal would hand any tenant the shared-row authority this
/// leaf has just taken away, by the back door of being the only citer.
pub async fn withdraw_citation(
    pool: &PgPool,
    snapshot_id: &str,
    tenant_id: &str,
    withdrawn_by: &str,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE evidence_citations \
         SET withdrawn_at = now(), withdrawn_by = $3, withdrawal_reason = $4 \
         WHERE snapshot_id = $1 AND tenant_id = $2 AND withdrawn_at IS NULL",
    )
    .bind(snapshot_id)
    .bind(tenant_id)
    .bind(withdrawn_by)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// The tombstone: the deletion records the reason + the time — the row
/// stays (never a silent disappearance). Idempotent: the first reason wins.
///
/// ⛔ NO TENANT-BOUND CALLER (`SIGNOFF-REPAIR.7.4.4`). This writes the SHARED
/// row, so every citer sees it; a tenant reaching it could remove evidence
/// another tenant relies on and stamp that tenant's receipt with its own
/// reason. Its callers are the site-operator acts: `expire_due`'s sweep and
/// `site_authority::retention::tombstone_evidence`'s named row.
pub async fn tombstone(
    pool: &PgPool,
    snapshot_id: &str,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    tombstone_in(&mut *pool.acquire().await?, snapshot_id, reason).await
}

/// The same tombstone inside a caller's transaction, so a site act's tombstone,
/// its authorization and its audit record commit together — a tombstone that
/// committed without its audit record would be an unattributable deletion
/// (the argument `expire_due` already makes for the sweep).
pub async fn tombstone_in(
    conn: &mut sqlx::PgConnection,
    snapshot_id: &str,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $2 \
         WHERE snapshot_id = $1 AND deleted_at IS NULL",
    )
    .bind(snapshot_id)
    .bind(reason)
    .execute(&mut *conn)
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
///
/// ⛔ This sweep carries no tenant predicate, and cannot: `retention_class`
/// is a column on the SHARED row, so which snapshots are due is a site-wide
/// fact rather than any one tenant's. That is why the authority to invoke it
/// is a site-operator capability and the caller does not choose `now`
/// (`SIGNOFF-REPAIR.7.4.3`; `site_authority::expire_evidence` passes the
/// database's own `clock_timestamp()`, read after the guard lock).
///
/// Takes a connection rather than the pool so the tombstones, the
/// authorization and its audit commit as ONE transaction — a sweep that
/// committed without its audit record would be an unattributable deletion.
pub async fn expire_due(
    conn: &mut sqlx::PgConnection,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $1 \
         WHERE deleted_at IS NULL AND retention_class = 'standard' AND created_at < $2",
    )
    .bind("the retention expired")
    .bind(now - chrono::Duration::days(30))
    .execute(&mut *conn)
    .await?;
    let standard = result.rows_affected();
    let result = sqlx::query(
        "UPDATE evidence_snapshots SET deleted_at = now(), deletion_reason = $1 \
         WHERE deleted_at IS NULL AND retention_class = 'temporary' AND created_at < $2",
    )
    .bind("the retention expired")
    .bind(now - chrono::Duration::days(1))
    .execute(&mut *conn)
    .await?;
    Ok(standard + result.rows_affected())
}

/// The staleness surface: the LIVE snapshots THIS TENANT CITED whose
/// freshness horizon has passed (the assessments read this — the re-fetch is
/// the caller's).
///
/// This is the surface the enumeration ran through. It is a list rather than
/// a lookup by identifier, so an unbound version hands any enrolled principal
/// the whole site's research trail — every other tenant's locators, resolvers
/// and credential classes — without needing to guess a single id.
///
/// ⛔ It stays a TENANT read rather than moving behind a site-operator grant.
/// Nothing operator-shaped consumes it: `git grep -n "snapshots::stale" -- crates`
/// and `git grep -n "snapshots/stale" -- crates` together return the route,
/// this function, its own doc lines and one test — no CLI, no MCP tool, no
/// worker. A freshness horizon is a decision about a tenant's own
/// re-acquisition; the operator-shaped verb over these rows is `expire_due`.
pub async fn stale_for_tenant(
    pool: &PgPool,
    now: chrono::DateTime<chrono::Utc>,
    tenant_id: &str,
) -> Result<Vec<StoredSnapshot>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SnapshotRow>(&format!(
        "SELECT {SNAPSHOT_COLUMNS} FROM evidence_snapshots \
         WHERE deleted_at IS NULL AND fresh_until IS NOT NULL AND fresh_until < $1 \
           AND {CITED_BY_TENANT} \
         ORDER BY fresh_until"
    ))
    .bind(now)
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(StoredSnapshot::from).collect())
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
