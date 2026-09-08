//! The resolver SDK surface (`PHASE-8.4.1`, ADR-027, ROADMAP §12.2): the
//! DECLARATIVE advertise contract a third-party resolver publishes — the
//! typed shape the registry consumes. The server's capability registry
//! (`reasonbraid-server/src/resolvers.rs`) is the consumer; this crate is
//! the SDK home third parties depend on.
//!
//! The acquisition-execution trait (the `.4.4` load side) is the named
//! follow-on: today the built-in packs execute through the server's own
//! receipt path, and a third-party resolver contributes its ADVERTISE
//! only — the execution surface opens with the signed-load machinery.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The ADR-018 sandbox ladder (the advertise's allowed values).
pub const SANDBOX_LEVELS: [&str; 4] = ["none", "process", "constrained_process", "vm_container"];

/// The ADR-018 egress classes.
pub const EGRESS_CLASSES: [&str; 4] = ["none", "loopback", "listed", "any"];

/// The typed §12.2 advertise — the resolver's declared capability row.
/// A third-party resolver publishes this shape; the registry's
/// filter-then-rank consumes it (the scheme + the ADR-018 isolation
/// filters first, then the latency-midpoint rank).
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

#[cfg(test)]
mod tests {
    use super::*;

    fn advertise(sandbox: &str, egress: &str) -> ResolverAdvertise {
        ResolverAdvertise {
            resolver_id: "test-resolver".into(),
            schemes: vec!["https".into()],
            locator_patterns: vec![],
            media_types: vec![],
            max_bytes: 1024,
            abilities: vec![],
            authentication_classes: vec!["none".into()],
            egress_class: egress.into(),
            sandbox_level: sandbox.into(),
            redirect_policy: "deny".into(),
            archive_policy: "deny".into(),
            subresource_policy: "deny".into(),
            javascript_policy: "deny".into(),
            snapshot_formats: vec![],
            derivation_formats: vec![],
            latency_range_ms: serde_json::json!({ "min": 100, "max": 500 }),
            version: "0.1.0".into(),
            security_evidence: serde_json::Value::Null,
        }
    }

    /// The SDK's advertise shape validates the ADR-018 vocabulary — the
    /// same fail-closed gate the registry applies.
    #[test]
    fn the_sdk_advertise_validates_the_isolation_vocabulary() {
        assert_eq!(advertise("vm_container", "listed").isolation_error(), None);
        assert_eq!(
            advertise("jail", "listed").isolation_error(),
            Some("the sandbox level is outside the ADR-018 ladder")
        );
        assert_eq!(
            advertise("vm_container", "everywhere").isolation_error(),
            Some("the egress class is outside the ADR-018 vocabulary")
        );
    }
}
