//! The resolver capability registry (`PHASE-4.1.3`, backlog 31): the §12.2
//! advertise shape, durable. The resolution order is the §12.2 contract —
//! the authorization + the risk filters FIRST (the scheme + the ADR-018
//! isolation classes), then the rank of the eligible resolvers (the latency
//! midpoint at the dev scale); the absence of a resolver yields the explicit
//! `resource_unresolvable_now` — the reference is PRESERVED for later, never
//! fabricated into evidence. The dev profile's resolvers are the future
//! packs (`.2`–`.4`): the registry ships the SHAPE with the explicit result
//! measured.
//!
//! The advertise SHAPE is the SDK surface (`.4.1`): it lives in
//! `reasonbraid-adapter::resolver` (the third-party dependency home) and is
//! re-exported here so the server's callers keep the single path.

use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;

pub use reasonbraid_adapter::resolver::{ResolverAdvertise, EGRESS_CLASSES, SANDBOX_LEVELS};

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
/// The built-in R1 pack's registry id (the migration 0026 install record).
pub const R1_RESOLVER_ID: &str = "r1-git-fetcher";
/// The built-in R2 pack's registry id (the migration 0027 install record).
pub const R2_RESOLVER_ID: &str = "r2-extract-worker";
/// The gated packs' registry ids (the `.5.3` wiring): NO migration seeds
/// them — the startup sync registers them ONLY when the gate is open, and
/// removes them when it is closed. The resolve never returns a disabled
/// pack because the disabled pack has no row.
pub const R3_RESOLVER_ID: &str = "r3-browser-worker";
pub const R5_RESOLVER_ID: &str = "r5-credential-broker";
pub const RX_RESOLVER_ID: &str = "rx-agent-mediated";

/// The acquisition result of a built-in pack (the `.2.3`/`.3.3` receipts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum Acquisition {
    Web(crate::fetcher::AcquisitionReceipt),
    Git(crate::git::GitReceipt),
    Extract(crate::extraction::ExtractionReceipt),
    Authenticated(Box<crate::broker::AuthenticatedReceipt>),
    Browse(crate::browse::BrowserReceipt),
}

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
    pub acquisition: Option<Acquisition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_error: Option<AcquisitionError>,
    /// The RX pack's capability-call publication (the §12.2/§12.8 shape)
    /// — present when the agent-mediated resolver ranks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_call: Option<crate::mediated::AcquisitionCall>,
}

/// Resolve a reference: the scheme + the ADR-018 isolation filters FIRST
/// (the required sandbox/egress classes ride the request), then the rank by
/// the latency midpoint.
pub async fn resolve(
    pool: &PgPool,
    scheme: &str,
    media_type: Option<&str>,
    credential_binding: Option<&str>,
    required_sandbox: &str,
    required_egress: &str,
) -> Result<ResolutionOutcome, sqlx::Error> {
    let sandbox_rank = SANDBOX_LEVELS.iter().position(|s| *s == required_sandbox);
    let egress_rank = EGRESS_CLASSES.iter().position(|e| *e == required_egress);
    // The routing filters (the `.4.1` + `.5.1` contracts):
    // - a media-type hint ranks only the resolvers whose advertised types
    //   include it (the extraction pack); a hintless reference keeps the
    //   acquisition-only path (the extraction ability is excluded);
    // - a credential binding ranks only the resolvers that declare the
    //   `credential` authentication class (the R5 broker); a binding-less
    //   reference ranks only the `none`-class packs.
    let base = "SELECT resolver_id, sandbox_level, egress_class, latency_range_ms FROM resolver_capabilities WHERE schemes @> $1::jsonb";
    let query: String = match (media_type, credential_binding) {
        (Some(_hint), Some(_binding)) => {
            format!("{base} AND media_types @> $2::jsonb AND authentication_classes @> $3::jsonb")
        }
        (Some(_hint), None) => {
            format!("{base} AND media_types @> $2::jsonb AND authentication_classes @> $3::jsonb")
        }
        (None, Some(_binding)) => format!("{base} AND authentication_classes @> $2::jsonb"),
        (None, None) => format!(
            "{base} AND NOT abilities @> '[\"extract\"]'::jsonb \
             AND (authentication_classes @> $2::jsonb OR authentication_classes = '[]'::jsonb)"
        ),
    };
    let mut query_builder = sqlx::query_as::<_, (String, String, String, Value)>(&query)
        .bind(serde_json::json!([scheme]));
    if let Some(hint) = media_type {
        query_builder = query_builder.bind(serde_json::json!([hint]));
    }
    let rows: Vec<(String, String, String, Value)> = match credential_binding {
        Some(_) => {
            query_builder
                .bind(serde_json::json!(["credential"]))
                .fetch_all(pool)
                .await?
        }
        None => {
            query_builder
                .bind(serde_json::json!(["none"]))
                .fetch_all(pool)
                .await?
        }
    };
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
        acquisition_call: None,
    })
}

/// The gate's startup sync: the R3/R5/RX rows exist ONLY while the gate is
/// open — opening registers them, closing REMOVES them (the resolve never
/// returns a disabled pack because the disabled pack has no row).
pub async fn sync_gated_entries(pool: &PgPool, enabled: bool) -> Result<(), sqlx::Error> {
    if enabled {
        for advertise in gated_advertises() {
            register(pool, &advertise).await?;
        }
    } else {
        sqlx::query("DELETE FROM resolver_capabilities WHERE resolver_id = ANY($1::text[])")
            .bind([R3_RESOLVER_ID, R5_RESOLVER_ID, RX_RESOLVER_ID])
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// The gated packs' advertises (the `.5.1` contract's claims).
fn gated_advertises() -> Vec<ResolverAdvertise> {
    vec![
        ResolverAdvertise {
            resolver_id: R5_RESOLVER_ID.to_owned(),
            schemes: vec!["https".to_owned()],
            locator_patterns: vec!["https://*".to_owned()],
            media_types: Vec::new(),
            max_bytes: 16 * 1024 * 1024,
            abilities: vec!["fetch-authenticated".to_owned()],
            authentication_classes: vec!["credential".to_owned()],
            egress_class: "listed".to_owned(),
            sandbox_level: "none".to_owned(),
            redirect_policy: "follow-classified".to_owned(),
            archive_policy: "deny".to_owned(),
            subresource_policy: "deny".to_owned(),
            javascript_policy: "deny".to_owned(),
            snapshot_formats: vec!["sha256:<hex>".to_owned()],
            derivation_formats: Vec::new(),
            latency_range_ms: serde_json::json!({ "min": 200, "max": 5000 }),
            version: "0.1.0".to_owned(),
            security_evidence: serde_json::json!({
                "broker": "local",
                "credential": "per-request, never ambient",
                "disclosure": "explicit",
            }),
        },
        ResolverAdvertise {
            resolver_id: R3_RESOLVER_ID.to_owned(),
            schemes: vec!["web+render".to_owned()],
            locator_patterns: vec!["https://*".to_owned()],
            media_types: Vec::new(),
            max_bytes: 16 * 1024 * 1024,
            abilities: vec!["render".to_owned()],
            authentication_classes: vec!["none".to_owned()],
            egress_class: "listed".to_owned(),
            sandbox_level: "vm_container".to_owned(),
            redirect_policy: "deny".to_owned(),
            archive_policy: "deny".to_owned(),
            subresource_policy: "deny".to_owned(),
            javascript_policy: "allow-bounded".to_owned(),
            snapshot_formats: vec!["sha256:<hex>".to_owned()],
            derivation_formats: vec!["text/chunks".to_owned()],
            latency_range_ms: serde_json::json!({ "min": 1000, "max": 60000 }),
            version: "0.1.0".to_owned(),
            security_evidence: serde_json::json!({
                "worker": "process-per-render",
                "network_log": true,
                "container_required": true,
            }),
        },
        ResolverAdvertise {
            resolver_id: RX_RESOLVER_ID.to_owned(),
            schemes: vec!["web+agent".to_owned()],
            locator_patterns: Vec::new(),
            media_types: Vec::new(),
            max_bytes: 0,
            abilities: vec!["agent-mediated".to_owned()],
            authentication_classes: vec!["none".to_owned()],
            egress_class: "any".to_owned(),
            sandbox_level: "none".to_owned(),
            redirect_policy: "deny".to_owned(),
            archive_policy: "deny".to_owned(),
            subresource_policy: "deny".to_owned(),
            javascript_policy: "deny".to_owned(),
            snapshot_formats: Vec::new(),
            derivation_formats: Vec::new(),
            latency_range_ms: serde_json::json!({ "min": 5000, "max": 300000 }),
            version: "0.1.0".to_owned(),
            security_evidence: serde_json::json!({
                "vocabulary": "the §12.8 acquisition-call shapes",
                "delivery": "the capability-call lane",
            }),
        },
    ]
}
