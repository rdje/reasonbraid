//! The directory profile service (`PHASE-3.1.2`, backlog 26): the §10.1
//! registration profile as TYPED boundary data — a role declares its own profile
//! (self-asserted claims are allowed; the provenance is shown, never flattened),
//! every write is a NEW content-addressed version (the old ones stay readable),
//! and the profile REFERENCES grants — it never creates authority (the
//! evaluation reads the grants table only).
//!
//! The visibility policy is stored here and ENFORCED by the `.1.3` read surface;
//! this module owns the schema + the write path + the content addressing.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};

use chrono::{DateTime, Utc};

/// The §10.1 registration profile. `deny_unknown_fields`: an unknown field is a
/// typed rejection, never silently dropped.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentProfile {
    pub display_label: String,
    pub purpose: String,
    #[serde(default)]
    pub conversation_modes: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityClaim>,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub structured_output_formats: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub confidentiality_classes: Vec<String>,
    #[serde(default)]
    pub availability: Option<Availability>,
    #[serde(default)]
    pub resolver_tool_capabilities: Vec<String>,
    #[serde(default)]
    pub cost_latency_class: Option<String>,
    #[serde(default)]
    pub resource_ceilings: Option<Value>,
    /// The per-field visibility policy (`.1.3` enforces it per reader).
    #[serde(default)]
    pub visibility: VisibilityPolicy,
    /// Grant references ONLY — the profile never grants authority.
    #[serde(default)]
    pub grants_by_reference: Vec<String>,
    /// The incarnation lineage link (the `.1.6.1` row), when known.
    #[serde(default)]
    pub incarnation_id: Option<String>,
}

/// One capability claim: the §10.1 rule — a high self-declared score is never
/// equivalent to verified competence, so the provenance rides every claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityClaim {
    pub taxonomy_id: String,
    #[serde(default = "default_confidence")]
    pub confidence: ClaimConfidence,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_confidence() -> ClaimConfidence {
    ClaimConfidence::SelfAsserted
}

impl ClaimConfidence {
    /// The wire name (the `.3.1` reasons + the profile surface share it).
    pub fn rank_name(&self) -> &'static str {
        match self {
            ClaimConfidence::SelfAsserted => "self_asserted",
            ClaimConfidence::OwnerAttested => "owner_attested",
            ClaimConfidence::Benchmarked => "benchmarked",
            ClaimConfidence::Certified => "certified",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimConfidence {
    SelfAsserted,
    OwnerAttested,
    Benchmarked,
    Certified,
}

/// Availability / operating-hours / concurrency / wake policy (§10.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    #[serde(default)]
    pub operating_hours: Option<String>,
    #[serde(default)]
    pub concurrency: Option<i64>,
    #[serde(default)]
    pub wake_policy: Option<String>,
}

/// How far a profile field travels (§10.1's per-field visibility).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityClass {
    /// The role itself only.
    #[default]
    SelfOnly,
    /// The role's tenant (the `.1.3` tenant-scoped view).
    Tenant,
    /// Any enrolled principal (the network view).
    Network,
    /// Unrestricted.
    Public,
}

/// The per-field visibility policy; every named field defaults to `self_only`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct VisibilityPolicy {
    pub display_label: VisibilityClass,
    pub purpose: VisibilityClass,
    pub conversation_modes: VisibilityClass,
    pub capabilities: VisibilityClass,
    pub interests: VisibilityClass,
    pub languages: VisibilityClass,
    pub structured_output_formats: VisibilityClass,
    pub scopes: VisibilityClass,
    pub confidentiality_classes: VisibilityClass,
    pub availability: VisibilityClass,
    pub resolver_tool_capabilities: VisibilityClass,
    pub cost_latency_class: VisibilityClass,
    pub resource_ceilings: VisibilityClass,
    pub grants_by_reference: VisibilityClass,
}

impl Default for VisibilityPolicy {
    fn default() -> Self {
        Self {
            display_label: VisibilityClass::Network,
            purpose: VisibilityClass::Network,
            conversation_modes: VisibilityClass::Tenant,
            capabilities: VisibilityClass::Tenant,
            interests: VisibilityClass::Network,
            languages: VisibilityClass::Network,
            structured_output_formats: VisibilityClass::Tenant,
            scopes: VisibilityClass::Tenant,
            confidentiality_classes: VisibilityClass::SelfOnly,
            availability: VisibilityClass::Tenant,
            resolver_tool_capabilities: VisibilityClass::Tenant,
            cost_latency_class: VisibilityClass::Tenant,
            resource_ceilings: VisibilityClass::SelfOnly,
            grants_by_reference: VisibilityClass::SelfOnly,
        }
    }
}

/// The reader's visibility class (the per-reader classification the `.1.3`
/// read surface applies). `Self` sees everything; the ladder is
/// Public < Network < Tenant < Self.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderClass {
    /// The role itself (or its accountable owner): the full profile.
    Full,
    /// A principal enrolled in the role's tenant.
    Tenant,
    /// Any other enrolled principal (the network view).
    Network,
}

fn visibility_rank(class: VisibilityClass) -> u8 {
    match class {
        VisibilityClass::Public => 0,
        VisibilityClass::Network => 1,
        VisibilityClass::Tenant => 2,
        VisibilityClass::SelfOnly => 3,
    }
}

/// A reader of the given class sees a field when the field's visibility class
/// is AT LEAST as wide as the reader's class (a `tenant` field is visible to
/// the tenant, the network, and the public). `Self` sees everything.
fn field_visible(field: VisibilityClass, reader: ReaderClass) -> bool {
    if reader == ReaderClass::Full {
        return true;
    }
    visibility_rank(field)
        <= match reader {
            ReaderClass::Tenant => 2,
            ReaderClass::Network => 1,
            ReaderClass::Full => unreachable!(),
        }
}

/// The per-reader filtered profile: a hidden field is ABSENT, never nulled.
/// The provenance of every visible capability claim rides along (the
/// self-asserted vs attested distinction is shown, never flattened).
pub fn filter_profile(profile: &AgentProfile, reader: ReaderClass) -> Value {
    let v = &profile.visibility;
    let mut out = serde_json::Map::new();
    fn put(out: &mut serde_json::Map<String, Value>, key: &str, visible: bool, value: Value) {
        if visible {
            out.insert(key.to_string(), value);
        }
    }
    fn ser<T: Serialize>(x: &T) -> Value {
        serde_json::to_value(x).expect("serializes")
    }
    put(
        &mut out,
        "display_label",
        field_visible(v.display_label, reader),
        ser(&profile.display_label),
    );
    put(
        &mut out,
        "purpose",
        field_visible(v.purpose, reader),
        ser(&profile.purpose),
    );
    put(
        &mut out,
        "conversation_modes",
        field_visible(v.conversation_modes, reader),
        ser(&profile.conversation_modes),
    );
    put(
        &mut out,
        "capabilities",
        field_visible(v.capabilities, reader),
        ser(&profile.capabilities),
    );
    put(
        &mut out,
        "interests",
        field_visible(v.interests, reader),
        ser(&profile.interests),
    );
    put(
        &mut out,
        "languages",
        field_visible(v.languages, reader),
        ser(&profile.languages),
    );
    put(
        &mut out,
        "structured_output_formats",
        field_visible(v.structured_output_formats, reader),
        ser(&profile.structured_output_formats),
    );
    put(
        &mut out,
        "scopes",
        field_visible(v.scopes, reader),
        ser(&profile.scopes),
    );
    put(
        &mut out,
        "confidentiality_classes",
        field_visible(v.confidentiality_classes, reader),
        ser(&profile.confidentiality_classes),
    );
    put(
        &mut out,
        "availability",
        field_visible(v.availability, reader),
        ser(&profile.availability),
    );
    put(
        &mut out,
        "resolver_tool_capabilities",
        field_visible(v.resolver_tool_capabilities, reader),
        ser(&profile.resolver_tool_capabilities),
    );
    put(
        &mut out,
        "cost_latency_class",
        field_visible(v.cost_latency_class, reader),
        ser(&profile.cost_latency_class),
    );
    put(
        &mut out,
        "resource_ceilings",
        field_visible(v.resource_ceilings, reader),
        ser(&profile.resource_ceilings),
    );
    put(
        &mut out,
        "grants_by_reference",
        field_visible(v.grants_by_reference, reader),
        ser(&profile.grants_by_reference),
    );
    Value::Object(out)
}

/// The canonical content hash: the SHA-256 of the typed profile's serialized
/// form (field order fixed by the struct — deterministic for the same input).
pub fn content_hash(profile: &AgentProfile) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(profile)?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// One profile version as stored.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileVersionRow {
    pub version: i32,
    pub content_hash: String,
    pub profile: Value,
    pub written_by: String,
    pub written_at: DateTime<Utc>,
}

/// The current profile + its version metadata.
#[derive(Debug, Clone, Serialize)]
pub struct CurrentProfile {
    pub role_id: String,
    pub version: i32,
    pub content_hash: String,
    pub profile: Value,
    pub written_by: String,
    pub written_at: DateTime<Utc>,
}

/// Write (or replace) the profile on the caller's OWN transaction
/// (`SIGNOFF-REPAIR.3.3.4.11.1`): a NEW version row, the current pointer
/// advanced, the writer recorded. The content hash is server-computed.
///
/// ⛔ The caller supplies a connection that is already inside a transaction, so
/// the version this writes and whatever else that transaction does share ONE
/// commit. That is what `.11.2`'s attestation and `.11.3`'s card import need and
/// what the pool-taking form below cannot give them: a function that opens its
/// own transaction can only ever be a second commit, and it would reach for a
/// second pool connection while the caller holds rows it needs — two
/// connections blocking each other through the pool, where no deadlock detector
/// sees a cycle and only `lock_timeout` ends it.
///
/// `at` is the caller's own evaluation time, so a caller that sampled database
/// time after its guard wait records that instant here rather than a second,
/// later reading.
pub async fn write_profile_in_tx(
    tx: &mut PgConnection,
    role_id: &str,
    writer: &str,
    profile: &AgentProfile,
    at: DateTime<Utc>,
) -> Result<CurrentProfile, sqlx::Error> {
    let hash = content_hash(profile).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        )))
    })?;
    let profile_json = serde_json::to_value(profile).expect("the typed profile serializes");

    // The role must exist (the `agent_roles` row is the identity anchor). This
    // runs BEFORE the anchor insert on purpose: the anchor's foreign key would
    // otherwise RAISE for an unknown role, and a raised violation aborts the
    // caller's whole transaction rather than returning a typed refusal
    // (`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`).
    let role_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agent_roles WHERE role_id = $1)")
            .bind(role_id)
            .fetch_optional(&mut *tx)
            .await?;
    if !role_exists.unwrap_or(false) {
        return Err(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no role `{role_id}`"),
        ))));
    }

    // ⛔ Acquire the role's version anchor, creating it INSIDE the acquisition.
    // The superseded writer read `current_version`, added one and inserted under
    // `UNIQUE (role_id, version)`, so two concurrent writers both read N, both
    // wrote N+1, and the constraint turned the second into a raised violation
    // the route could only answer `500`. A row lock alone does not fix that:
    // `SELECT … FOR UPDATE` over a row that does not exist yet matches nothing,
    // locks nothing and returns immediately, so it would protect every write
    // except a role's first. Insert-then-lock is the same two-step
    // `transaction::acquire_in_tx` uses for the tenant guards, for the same
    // reason (`docs/knowledge/serializing-writers-at-a-row-that-may-not-exist.md`).
    sqlx::query(
        "INSERT INTO agent_profiles (role_id, current_version, updated_at) \
         VALUES ($1, 0, $2) ON CONFLICT (role_id) DO NOTHING",
    )
    .bind(role_id)
    .bind(at)
    .execute(&mut *tx)
    .await?;
    let current: i32 = sqlx::query_scalar(
        "SELECT current_version FROM agent_profiles WHERE role_id = $1 FOR UPDATE",
    )
    .bind(role_id)
    .fetch_one(&mut *tx)
    .await?;
    let next = current + 1;

    sqlx::query(
        "INSERT INTO profile_versions (version_id, role_id, version, content_hash, profile, written_by, written_at) \
         VALUES ('pver_' || gen_random_uuid()::text, $1, $2, $3, $4, $5, $6)",
    )
    .bind(role_id)
    .bind(next)
    .bind(&hash)
    .bind(&profile_json)
    .bind(writer)
    .bind(at)
    .execute(&mut *tx)
    .await?;

    // The anchor exists and is locked, so this is a plain UPDATE rather than the
    // upsert the superseded writer needed.
    sqlx::query(
        "UPDATE agent_profiles SET current_version = $2, updated_at = $3 WHERE role_id = $1",
    )
    .bind(role_id)
    .bind(next)
    .bind(at)
    .execute(&mut *tx)
    .await?;

    Ok(CurrentProfile {
        role_id: role_id.to_string(),
        version: next,
        content_hash: hash,
        profile: profile_json,
        written_by: writer.to_string(),
        written_at: at,
    })
}

/// The pool-taking form: one transaction around [`write_profile_in_tx`], holding
/// database time sampled inside it.
///
/// There is ONE writer, not two — this is a thin wrapper, kept for the callers
/// `.11.2` and `.11.3` have not yet moved onto their own guarded transactions.
/// It takes no tenant authority guard, and deliberately: the routes that reach
/// it here are gated on identity rather than on a grant, so there is no
/// authority decision for a guard to order them against.
pub async fn write_profile(
    pool: &PgPool,
    role_id: &str,
    writer: &str,
    profile: &AgentProfile,
) -> Result<CurrentProfile, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let at = crate::authority::transaction::database_now_in_tx(&mut tx).await?;
    let written = write_profile_in_tx(&mut tx, role_id, writer, profile, at).await?;
    tx.commit().await?;
    Ok(written)
}

/// Read the CURRENT profile (the `.1.3` leaf adds the per-reader filtering).
pub async fn current_profile(
    pool: &PgPool,
    role_id: &str,
) -> Result<Option<CurrentProfile>, sqlx::Error> {
    let row: Option<(i32, String, Value, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT v.version, v.content_hash, v.profile, v.written_by, v.written_at \
         FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
         WHERE v.role_id = $1 AND v.version = p.current_version",
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(version, content_hash, profile, written_by, written_at)| CurrentProfile {
            role_id: role_id.to_string(),
            version,
            content_hash,
            profile,
            written_by,
            written_at,
        },
    ))
}

/// Read ONE historical version (the content-addressed history stays readable).
pub async fn version_at(
    pool: &PgPool,
    role_id: &str,
    version: i32,
) -> Result<Option<ProfileVersionRow>, sqlx::Error> {
    let row: Option<(i32, String, Value, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT version, content_hash, profile, written_by, written_at \
         FROM profile_versions WHERE role_id = $1 AND version = $2",
    )
    .bind(role_id)
    .bind(version)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(version, content_hash, profile, written_by, written_at)| ProfileVersionRow {
            version,
            content_hash,
            profile,
            written_by,
            written_at,
        },
    ))
}

/// The version list (the audit of every write).
pub async fn version_list(
    pool: &PgPool,
    role_id: &str,
) -> Result<Vec<(i32, String, String, DateTime<Utc>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT version, content_hash, written_by, written_at \
         FROM profile_versions WHERE role_id = $1 ORDER BY version",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
}

/// The owner's attestation: the named capability claim's provenance upgrades to
/// `owner_attested` with the evidence reference — a NEW version, writer recorded.
pub async fn attest_capability(
    pool: &PgPool,
    role_id: &str,
    writer: &str,
    taxonomy_id: &str,
    evidence_ref: &str,
) -> Result<Option<CurrentProfile>, sqlx::Error> {
    let Some(current) = current_profile(pool, role_id).await? else {
        return Ok(None);
    };
    let mut profile: AgentProfile =
        serde_json::from_value(current.profile.clone()).map_err(|e| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        })?;
    let Some(claim) = profile
        .capabilities
        .iter_mut()
        .find(|c| c.taxonomy_id == taxonomy_id)
    else {
        return Ok(None);
    };
    claim.confidence = ClaimConfidence::OwnerAttested;
    claim.evidence_ref = Some(evidence_ref.to_string());
    let written = write_profile(pool, role_id, writer, &profile).await?;
    Ok(Some(written))
}
