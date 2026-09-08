//! The adapter-load allowlist ladder (`PHASE-8.4.4`, ADR-027 §16.10): the
//! ORDERED, FAIL-CLOSED verification a downloaded adapter goes through
//! before any trust:
//!
//! 1. **the allowlist membership** — the adapter's id has a ledger row;
//! 2. **the digest** — the qualification record's self-digest re-derives
//!    (`sha256:<hex>`, the ADR-011 scheme);
//! 3. **the signature** — the release identity's Ed25519 over the
//!    record's canonical bytes (the ring provider — the workspace rule);
//! 4. **the API compatibility** — the adapter's reported contract
//!    version equals the SDK token;
//! 5. **the capability manifest** — the adapter's declared capabilities
//!    stay WITHIN the deployment's allowed ceilings (a streaming
//!    declaration under a non-streaming deployment refuses).
//!
//! A refusal names its RUNG (the cheapest first, the most
//! deployment-specific last) — never a partial trust. The shipped dev
//! adapters satisfy the ladder BY CONSTRUCTION (the compiled-in
//! implementations + the seeded allowlist rows).

use serde::{Deserialize, Serialize};

use crate::{AdapterCapabilities, CancellationStrength, CertificationReport, SDK_VERSION};

/// The deployment's allowed capability CEILINGS (the rung-5 input the
/// operator declares — the adapter's declared capabilities must stay
/// within them).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedCapabilities {
    /// The maximum cancellation strength a loaded adapter may declare.
    pub cancellation: CancellationStrength,
    /// Whether streaming adapters may load.
    pub streaming: bool,
    /// Whether status-lookup adapters may load.
    pub status_lookup: bool,
    /// Whether tool-capable adapters may load.
    pub tool_support: bool,
    /// The maximum policy-injection mode a loaded adapter may declare.
    pub policy_injection: crate::PolicyInjectionMode,
}

impl AllowedCapabilities {
    /// The dev profile's ceilings (the shipped adapters' shapes).
    pub fn dev() -> Self {
        Self {
            cancellation: CancellationStrength::Confirmed,
            streaming: true,
            status_lookup: true,
            tool_support: true,
            policy_injection: crate::PolicyInjectionMode::Structured,
        }
    }
}

/// The ladder's typed refusal — each variant names its RUNG (the
/// fail-closed contract: a refusal at a rung, never a partial trust).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RungRefusal {
    /// Rung 1: the adapter's id has NO allowlist row.
    NotAllowlisted { adapter_id: String },
    /// Rung 2: the record's self-digest fails to re-derive.
    DigestMismatch { adapter_id: String },
    /// Rung 3: the signature over the record's canonical bytes fails.
    SignatureInvalid { adapter_id: String },
    /// Rung 4: the adapter's reported contract version drifts from the
    /// SDK token.
    ApiIncompatible {
        adapter_id: String,
        reported: String,
    },
    /// Rung 5: the declared capabilities exceed the deployment's
    /// allowed ceilings.
    CapabilityExceeded { adapter_id: String, reason: String },
}

impl RungRefusal {
    /// The rung's number (1-5) — the refusal's position in the ladder.
    pub fn rung(&self) -> u8 {
        match self {
            RungRefusal::NotAllowlisted { .. } => 1,
            RungRefusal::DigestMismatch { .. } => 2,
            RungRefusal::SignatureInvalid { .. } => 3,
            RungRefusal::ApiIncompatible { .. } => 4,
            RungRefusal::CapabilityExceeded { .. } => 5,
        }
    }
}

impl std::fmt::Display for RungRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RungRefusal::NotAllowlisted { adapter_id } => {
                write!(f, "rung 1 (allowlist): `{adapter_id}` has no allowlist row")
            }
            RungRefusal::DigestMismatch { adapter_id } => {
                write!(
                    f,
                    "rung 2 (digest): the record for `{adapter_id}` fails its self-digest"
                )
            }
            RungRefusal::SignatureInvalid { adapter_id } => {
                write!(
                    f,
                    "rung 3 (signature): the record for `{adapter_id}` does not verify"
                )
            }
            RungRefusal::ApiIncompatible {
                adapter_id,
                reported,
            } => {
                write!(
                    f,
                    "rung 4 (api): `{adapter_id}` reports contract {reported} (the SDK token is {SDK_VERSION})"
                )
            }
            RungRefusal::CapabilityExceeded { adapter_id, reason } => {
                write!(f, "rung 5 (capabilities): `{adapter_id}` — {reason}")
            }
        }
    }
}

impl std::error::Error for RungRefusal {}

/// The declared capabilities stay within the allowed ceilings (rung 5).
pub fn capabilities_within(declared: &AdapterCapabilities, allowed: &AllowedCapabilities) -> bool {
    let mode_rank = |m: crate::PolicyInjectionMode| match m {
        crate::PolicyInjectionMode::None => 0,
        crate::PolicyInjectionMode::PromptOnly => 1,
        crate::PolicyInjectionMode::Structured => 2,
    };
    let strength_rank = |s: CancellationStrength| match s {
        CancellationStrength::None => 0,
        CancellationStrength::BestEffort => 1,
        CancellationStrength::Confirmed => 2,
    };
    strength_rank(declared.cancellation) <= strength_rank(allowed.cancellation)
        && (!declared.streaming || allowed.streaming)
        && (!declared.status_lookup || allowed.status_lookup)
        && (!declared.tool_support || allowed.tool_support)
        && mode_rank(declared.policy_injection) <= mode_rank(allowed.policy_injection)
}

/// Run the five-rung ladder for one candidate adapter. The inputs: the
/// adapter's id, its qualification record (the digest-pinned form), the
/// signature bytes (the `.sig` the release tool's `certify sign`
/// writes), the release identity's PUBLIC key bytes (the raw PKCS8
/// DER), the allowlist membership, the deployment's allowed capability
/// ceilings, and the adapter's declared capabilities.
pub fn verify_ladder(
    adapter_id: &str,
    record: &CertificationReport,
    signature: &[u8],
    public_key_der: &[u8],
    allowlisted: bool,
    allowed: &AllowedCapabilities,
    declared: &AdapterCapabilities,
) -> Result<(), RungRefusal> {
    // Rungs 1 + 2 first (the cheapest, the most stable).
    if !allowlisted {
        return Err(RungRefusal::NotAllowlisted {
            adapter_id: adapter_id.to_string(),
        });
    }
    if !record.digest_verifies() {
        return Err(RungRefusal::DigestMismatch {
            adapter_id: adapter_id.to_string(),
        });
    }
    // Rung 3: the signature over the canonical bytes.
    let canonical = serde_json::to_vec(record).map_err(|_| RungRefusal::SignatureInvalid {
        adapter_id: adapter_id.to_string(),
    })?;
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key_der)
        .verify(&canonical, signature)
        .map_err(|_| RungRefusal::SignatureInvalid {
            adapter_id: adapter_id.to_string(),
        })?;
    // Rung 4: the API compatibility (the SDK token).
    if record.sdk_version != SDK_VERSION {
        return Err(RungRefusal::ApiIncompatible {
            adapter_id: adapter_id.to_string(),
            reported: record.sdk_version.clone(),
        });
    }
    // Rung 5: the capability manifest within the deployment's ceilings.
    if !capabilities_within(declared, allowed) {
        return Err(RungRefusal::CapabilityExceeded {
            adapter_id: adapter_id.to_string(),
            reason: format!("the declared {declared:?} exceeds the allowed {allowed:?}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoxStatus, CheckResult, SixBoxRecord, Verdict};

    fn record(sdk_version: &str) -> CertificationReport {
        let mut report = CertificationReport {
            scenario: "vendor:test".to_string(),
            sdk_version: sdk_version.to_string(),
            checks: vec![CheckResult::Passed {
                check: "dispatch_boundary".to_string(),
            }],
            verdict: Verdict::Passed,
            six_box: SixBoxRecord {
                conformance: BoxStatus::Qualified {
                    evidence: "passed".to_string(),
                },
                stub_mechanics: BoxStatus::Qualified {
                    evidence: "the corpus".to_string(),
                },
                live_dispatch: BoxStatus::Untested {
                    reason: "no CI run".to_string(),
                },
                credential_containment: BoxStatus::Qualified {
                    evidence: "no credential field".to_string(),
                },
                manifest_entry: BoxStatus::Untested {
                    reason: "make release".to_string(),
                },
                dependency_ledger: BoxStatus::Qualified {
                    evidence: "deny + lock".to_string(),
                },
            },
            digest: String::new(),
        };
        report.digest = report.digest_hex();
        report
    }

    fn caps() -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: false,
            cancellation: CancellationStrength::BestEffort,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: crate::PolicyInjectionMode::None,
        }
    }

    fn key_and_signature(report: &CertificationReport) -> (Vec<u8>, Vec<u8>) {
        use ring::signature::KeyPair;
        let rng = ring::rand::SystemRandom::new();
        let doc = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).expect("the key");
        let key = ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(doc.as_ref())
            .expect("the key parses");
        let canonical = serde_json::to_vec(report).expect("the record serializes");
        let signature = key.sign(&canonical);
        (
            key.public_key().as_ref().to_vec(),
            signature.as_ref().to_vec(),
        )
    }

    /// The full ladder passes with the REAL identity: the signed record
    /// over the canonical bytes, the self-digest, the token, the ceilings.
    #[test]
    fn the_ladder_passes_the_signed_allowlisted_adapter() {
        let report = record(SDK_VERSION);
        let (public, signature) = key_and_signature(&report);
        verify_ladder(
            "vendor:test",
            &report,
            &signature,
            &public,
            true,
            &AllowedCapabilities::dev(),
            &caps(),
        )
        .expect("the ladder passes");
    }

    /// Each rung refuses with its OWN typed name — the refusal names the
    /// rung, never a generic failure.
    #[test]
    fn each_rung_refuses_with_its_own_name() {
        let report = record(SDK_VERSION);
        let (public, signature) = key_and_signature(&report);

        // Rung 1: the allowlist.
        let refusal = verify_ladder(
            "vendor:test",
            &report,
            &signature,
            &public,
            false,
            &AllowedCapabilities::dev(),
            &caps(),
        )
        .expect_err("the unallowlisted refuses");
        assert_eq!(refusal.rung(), 1);

        // Rung 2: the digest (a tampered digest fails BEFORE the signature).
        let mut tampered = report.clone();
        tampered.digest = "sha256:deadbeef".to_string();
        let refusal = verify_ladder(
            "vendor:test",
            &tampered,
            &signature,
            &public,
            true,
            &AllowedCapabilities::dev(),
            &caps(),
        )
        .expect_err("the tampered digest refuses");
        assert_eq!(refusal.rung(), 2);

        // Rung 3: the signature (the wrong signer's bytes).
        let (_, other_signature) = key_and_signature(&report);
        let refusal = verify_ladder(
            "vendor:test",
            &report,
            &other_signature,
            &public,
            true,
            &AllowedCapabilities::dev(),
            &caps(),
        )
        .expect_err("the wrong signature refuses");
        assert_eq!(refusal.rung(), 3);

        // Rung 4: the API compatibility (the record's OWN digest stays
        // valid — only the version drifts).
        let wrong = record("2");
        let (wrong_public, wrong_signature) = key_and_signature(&wrong);
        let refusal = verify_ladder(
            "vendor:test",
            &wrong,
            &wrong_signature,
            &wrong_public,
            true,
            &AllowedCapabilities::dev(),
            &caps(),
        )
        .expect_err("the version drift refuses");
        assert_eq!(refusal.rung(), 4);

        // Rung 5: the capability ceilings (a streaming declaration under
        // a non-streaming deployment).
        let mut streaming = caps();
        streaming.streaming = true;
        let allowed = AllowedCapabilities {
            streaming: false,
            ..AllowedCapabilities::dev()
        };
        let refusal = verify_ladder(
            "vendor:test",
            &report,
            &signature,
            &public,
            true,
            &allowed,
            &streaming,
        )
        .expect_err("the exceeded capability refuses");
        assert_eq!(refusal.rung(), 5);
        assert!(format!("{refusal}").contains("rung 5"));
    }
}
