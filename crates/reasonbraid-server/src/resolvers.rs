//! The resolver capability registry (`PHASE-4.1.3`, backlog 31): the §12.2
//! advertise shape, durable. The resolution order is the §12.2 contract —
//! the authorization + the risk filters FIRST (the scheme + the ADR-018
//! isolation classes), then the rank of the eligible resolvers (the latency
//! midpoint at the dev scale); the absence of a resolver yields the explicit
//! `resource_unresolvable_now` — the reference is PRESERVED for later, never
//! fabricated into evidence. The dev profile's resolvers are the future
//! packs (`.2`–`.4`): the registry ships the SHAPE with the explicit result
//! measured.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// The ADR-018 sandbox ladder (the registry's allowed values).
pub const SANDBOX_LEVELS: [&str; 4] = ["none", "process", "constrained_process", "vm_container"];

/// The ADR-018 egress classes.
pub const EGRESS_CLASSES: [&str; 4] = ["none", "loopback", "listed", "any"];

/// The typed §12.2 advertise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResolverAdvertise {
    pub resolver_id: String,
    pub schemes: Vec<String>,
    #[serde(default)]
    pub locator_patterns: Vec<String>,
    #[serde(default)]
    pub media_types: Vec<String>,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: i64,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[serde(default)]
    pub authentication_classes: Vec<String>,
    pub egress_class: String,
    pub sandbox_level: String,
    #[serde(default = "default_deny")]
    pub redirect_policy: String,
    #[serde(default = "default_deny")]
    pub archive_policy: String,
    #[serde(default = "default_deny")]
    pub subresource_policy: String,
    #[serde(default = "default_deny")]
    pub javascript_policy: String,
    #[serde(default)]
    pub snapshot_formats: Vec<String>,
    #[serde(default)]
    pub derivation_formats: Vec<String>,
    #[serde(default = "default_latency")]
    pub latency_range_ms: Value,
    pub version: String,
    #[serde(default)]
    pub security_evidence: Value,
}

fn default_max_bytes() -> i64 {
    10 * 1024 * 1024
}
fn default_deny() -> String {
    "deny".to_string()
}
fn default_latency() -> Value {
    serde_json::json!({ "min": 1000, "max": 60000 })
}

impl ResolverAdvertise {
    /// The ADR-018 vocabulary validation.
    pub fn isolation_error(&self) -> Option<&'static str> {
        if !SANDBOX_LEVELS.contains(&self.sandbox_level.as_str()) {
            return Some("the sandbox level is outside the ADR-018 ladder");
        }
        if !EGRESS_CLASSES.contains(&self.egress_class.as_str()) {
            return Some("the egress class is outside the ADR-018 vocabulary");
        }
        None
    }
}

/// Register (or replace) one resolver's advertise — the operator's verb (the
/// future packs call it at their install).
pub async fn register(pool: &PgPool, advertise: &ResolverAdvertise) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO resolver_capabilities \
         (resolver_id, schemes, locator_patterns, media_types, max_bytes, abilities, \
          authentication_classes, egress_class, sandbox_level, redirect_policy, \
          archive_policy, subresource_policy, javascript_policy, snapshot_formats, \
          derivation_formats, latency_range_ms, version, security_evidence) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18) \
         ON CONFLICT (resolver_id) DO UPDATE SET schemes = $2, egress_class = $8, \
           sandbox_level = $9, version = $17, security_evidence = $18, max_bytes = $5",
    )
    .bind(&advertise.resolver_id)
    .bind(serde_json::to_value(&advertise.schemes).expect("schemes serialize"))
    .bind(serde_json::to_value(&advertise.locator_patterns).expect("patterns serialize"))
    .bind(serde_json::to_value(&advertise.media_types).expect("media types serialize"))
    .bind(advertise.max_bytes)
    .bind(serde_json::to_value(&advertise.abilities).expect("abilities serialize"))
    .bind(serde_json::to_value(&advertise.authentication_classes).expect("auth classes serialize"))
    .bind(&advertise.egress_class)
    .bind(&advertise.sandbox_level)
    .bind(&advertise.redirect_policy)
    .bind(&advertise.archive_policy)
    .bind(&advertise.subresource_policy)
    .bind(&advertise.javascript_policy)
    .bind(serde_json::to_value(&advertise.snapshot_formats).expect("snapshot formats serialize"))
    .bind(
        serde_json::to_value(&advertise.derivation_formats).expect("derivation formats serialize"),
    )
    .bind(&advertise.latency_range_ms)
    .bind(&advertise.version)
    .bind(&advertise.security_evidence)
    .execute(pool)
    .await?;
    Ok(())
}

/// The built-in R0 pack's registry id (the migration 0025 install record).
pub const R0_RESOLVER_ID: &str = "r0-https-fetcher";

/// The built-in R0's NAMED acquisition refusal (the `.2.2` fetcher's typed
/// error) — the reference stays submitted, never fabricated.
#[derive(Debug, Clone, Serialize)]
pub struct AcquisitionError {
    pub kind: String,
    pub message: String,
}

/// The resolution outcome: the ranked eligible resolvers, or the explicit
/// unresolvable-now (the reference stays submitted — never fabricated).
/// When the built-in R0 resolver ranks FIRST, the resolution path executes
/// the acquisition and carries either the receipt or the named refusal.
#[derive(Debug, Clone, Serialize)]
pub struct ResolutionOutcome {
    pub resolvers: Vec<String>,
    pub unresolvable_now: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition: Option<crate::fetcher::AcquisitionReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_error: Option<AcquisitionError>,
}

/// Resolve a reference: the scheme + the ADR-018 isolation filters FIRST
/// (the required sandbox/egress classes ride the request), then the rank by
/// the latency midpoint.
pub async fn resolve(
    pool: &PgPool,
    scheme: &str,
    required_sandbox: &str,
    required_egress: &str,
) -> Result<ResolutionOutcome, sqlx::Error> {
    let sandbox_rank = SANDBOX_LEVELS.iter().position(|s| *s == required_sandbox);
    let egress_rank = EGRESS_CLASSES.iter().position(|e| *e == required_egress);
    let rows: Vec<(String, String, String, Value)> = sqlx::query_as(
        "SELECT resolver_id, sandbox_level, egress_class, latency_range_ms \
         FROM resolver_capabilities WHERE schemes @> $1::jsonb",
    )
    .bind(serde_json::json!([scheme]))
    .fetch_all(pool)
    .await?;

    // The filters: the resolver's declared classes must MEET the required
    // ones (the ADR-018 ladder order — the claim is the maximum, so a
    // resolver claiming LESS than required is ineligible).
    let mut eligible: Vec<(String, f64)> = Vec::new();
    for (resolver_id, sandbox, egress, latency) in rows {
        let ok_sandbox = match (
            SANDBOX_LEVELS.iter().position(|s| s == &sandbox),
            sandbox_rank,
        ) {
            (Some(declared), Some(required)) => declared >= required,
            _ => false,
        };
        let ok_egress = match (
            EGRESS_CLASSES.iter().position(|e| e == &egress),
            egress_rank,
        ) {
            (Some(declared), Some(required)) => declared >= required,
            _ => false,
        };
        if ok_sandbox && ok_egress {
            let min = latency
                .get("min")
                .and_then(|v| v.as_f64())
                .unwrap_or(1000.0);
            let max = latency
                .get("max")
                .and_then(|v| v.as_f64())
                .unwrap_or(60000.0);
            eligible.push((resolver_id, (min + max) / 2.0));
        }
    }
    eligible.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let unresolvable_now = eligible.is_empty();
    Ok(ResolutionOutcome {
        resolvers: eligible.into_iter().map(|(id, _)| id).collect(),
        unresolvable_now,
        acquisition: None,
        acquisition_error: None,
    })
}
