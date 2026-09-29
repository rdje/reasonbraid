//! The §12.8 agent-mediated acquisition vocabulary (PHASE-4.5.2, the RX
//! pack's contract from `2026-09-07_r5r3rx-contracts-opt-in.md`): the TYPED
//! response shapes an enrolled agent answers an acquisition call with —
//! never an unstructured blob. The not-inspected-original record rides the
//! same surface: the network records that other participants may NOT have
//! inspected the original.
//!
//! ⚠️ **Only the call is wired** (`SIGNOFF-REPAIR.11.59`). The RX resolver
//! publishes an [`AcquisitionCall`]; nothing receives an [`AcquisitionAnswer`],
//! so nothing checks the second-verifier rule either. The answer half is
//! deferred to `SIGNOFF-REPAIR.11.59.1`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The typed acquisition-call answer (the six §12.8 shapes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AcquisitionAnswer {
    /// An immutable snapshot the agent produced (its ADR-011 digest).
    ImmutableSnapshot { digest: String },
    /// A minimal authorized excerpt (the excerpt + the source's digest).
    MinimalExcerpt { digest: String, excerpt: String },
    /// A structured fact with provenance (the fact rides its source).
    StructuredFact { provenance: String, fact: Value },
    /// A redacted derivative (the derivative's digest + the redaction list).
    RedactedDerivative {
        digest: String,
        redactions: Vec<String>,
    },
    /// A local test/query receipt (the receipt rides its digest).
    TestReceipt { digest: String, receipt: Value },
    /// A refusal or an access limitation, named.
    Refusal { limitation: String },
}

/// The acquisition-call request the enrolled agent answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionCall {
    pub call_id: String,
    pub locator: String,
    /// The second-verifier rule: when set, the answer must be corroborated
    /// by another authorized verifier — the raw private source never
    /// leaves its host.
    #[serde(default)]
    pub requires_second_verifier: bool,
}

/// The answered call: the answer + the not-inspected-original record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionAnswerRecord {
    pub call: AcquisitionCall,
    pub answer: AcquisitionAnswer,
    /// TRUE when other participants may NOT have inspected the original —
    /// the network records the limitation, never a silent claim of
    /// inspection. ⛔ REQUIRED (`SIGNOFF-REPAIR.11.59`): under
    /// `#[serde(default)]` an omitted field read `false`, which is exactly a
    /// silent claim of inspection.
    pub original_not_inspected: bool,
    /// The second verifier's corroboration (absent until it lands).
    #[serde(default)]
    pub second_verifier: Option<SecondVerification>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecondVerification {
    pub verifier: String,
    pub answer_digest: String,
    pub at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vocabulary_roundtrips_every_shape() {
        let records = vec![
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_1".into(),
                    locator: "https://internal/report".into(),
                    requires_second_verifier: false,
                },
                answer: AcquisitionAnswer::ImmutableSnapshot {
                    digest: "sha256:aaaa".into(),
                },
                original_not_inspected: false,
                second_verifier: None,
            },
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_2".into(),
                    locator: "https://internal/report".into(),
                    requires_second_verifier: true,
                },
                answer: AcquisitionAnswer::Refusal {
                    limitation: "the raw source never leaves the host".into(),
                },
                original_not_inspected: true,
                second_verifier: None,
            },
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_3".into(),
                    locator: "https://internal/db".into(),
                    requires_second_verifier: true,
                },
                answer: AcquisitionAnswer::StructuredFact {
                    provenance: "the local query against the internal db".into(),
                    fact: serde_json::json!({ "rows": 3 }),
                },
                original_not_inspected: true,
                second_verifier: Some(SecondVerification {
                    verifier: "node-2".into(),
                    answer_digest: "sha256:bbbb".into(),
                    at: chrono::DateTime::parse_from_rfc3339("2026-09-07T15:00:00Z")
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                }),
            },
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_6".into(),
                    locator: "https://internal/policy".into(),
                    requires_second_verifier: false,
                },
                answer: AcquisitionAnswer::MinimalExcerpt {
                    digest: "sha256:cccc".into(),
                    excerpt: "the allowed paragraph".into(),
                },
                original_not_inspected: true,
                second_verifier: None,
            },
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_7".into(),
                    locator: "https://internal/x".into(),
                    requires_second_verifier: true,
                },
                answer: AcquisitionAnswer::RedactedDerivative {
                    digest: "sha256:dddd".into(),
                    redactions: vec!["the hostname".into()],
                },
                original_not_inspected: true,
                second_verifier: None,
            },
            AcquisitionAnswerRecord {
                call: AcquisitionCall {
                    call_id: "cal_8".into(),
                    locator: "https://internal/suite".into(),
                    requires_second_verifier: false,
                },
                answer: AcquisitionAnswer::TestReceipt {
                    digest: "sha256:ffff".into(),
                    receipt: serde_json::json!({ "passed": 12, "failed": 0 }),
                },
                original_not_inspected: true,
                second_verifier: None,
            },
        ];
        // `SIGNOFF-REPAIR.11.59`: this test is named for EVERY shape, and it
        // round-tripped three of the six. The match is exhaustive, so a seventh
        // shape fails to compile here until a record carries it.
        let shape = |answer: &AcquisitionAnswer| match answer {
            AcquisitionAnswer::ImmutableSnapshot { .. } => "immutable_snapshot",
            AcquisitionAnswer::MinimalExcerpt { .. } => "minimal_excerpt",
            AcquisitionAnswer::StructuredFact { .. } => "structured_fact",
            AcquisitionAnswer::RedactedDerivative { .. } => "redacted_derivative",
            AcquisitionAnswer::TestReceipt { .. } => "test_receipt",
            AcquisitionAnswer::Refusal { .. } => "refusal",
        };
        let covered: std::collections::BTreeSet<&str> =
            records.iter().map(|r| shape(&r.answer)).collect();
        assert_eq!(
            covered.len(),
            6,
            "every shape is round-tripped: {covered:?}"
        );
        for record in records {
            let json = serde_json::to_value(&record).expect("the record serializes");
            let back: AcquisitionAnswerRecord =
                serde_json::from_value(json).expect("the record deserializes");
            assert_eq!(back, record);
        }
        // The tagged wire shape: the answer's kind names itself.
        let value = serde_json::to_value(AcquisitionAnswer::MinimalExcerpt {
            digest: "sha256:cccc".into(),
            excerpt: "the allowed paragraph".into(),
        })
        .unwrap();
        assert_eq!(value["kind"], "minimal_excerpt");
        assert_eq!(value["excerpt"], "the allowed paragraph");
    }

    /// `SIGNOFF-REPAIR.11.59`: a record that does not say whether the original
    /// was inspected is refused. Under `#[serde(default)]` it read `false`, a
    /// silent claim of inspection, which is what the field exists to prevent.
    #[test]
    fn a_record_that_omits_the_not_inspected_flag_is_refused() {
        let omitted = serde_json::json!({
            "call": { "call_id": "cal_5", "locator": "https://internal/x" },
            "answer": { "kind": "immutable_snapshot", "digest": "sha256:eeee" },
        });
        let read = serde_json::from_value::<AcquisitionAnswerRecord>(omitted);
        assert!(
            read.is_err(),
            "an omitted flag is not a claim of inspection: {read:?}"
        );
    }

    #[test]
    fn the_second_verifier_rule_is_carried_not_enforced() {
        // The vocabulary carries the requirement, and a record without the
        // corroboration is still a well-formed record. Nothing enforces the
        // rule yet, because nothing receives an answer (`SIGNOFF-REPAIR.11.59`;
        // the answer half is `.11.59.1`).
        let record = AcquisitionAnswerRecord {
            call: AcquisitionCall {
                call_id: "cal_4".into(),
                locator: "https://internal/x".into(),
                requires_second_verifier: true,
            },
            answer: AcquisitionAnswer::RedactedDerivative {
                digest: "sha256:dddd".into(),
                redactions: vec!["the hostname".into()],
            },
            original_not_inspected: true,
            second_verifier: None,
        };
        assert!(record.call.requires_second_verifier);
        assert!(record.second_verifier.is_none());
    }
}
