//! The §12.8 agent-mediated acquisition vocabulary (PHASE-4.5.2, the RX
//! pack's contract from `2026-09-07_r5r3rx-contracts-opt-in.md`): the TYPED
//! response shapes an enrolled agent answers an acquisition call with —
//! never an unstructured blob. The not-inspected-original record rides the
//! same surface: the network records that other participants may NOT have
//! inspected the original.

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
    /// inspection.
    #[serde(default)]
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
        ];
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

    #[test]
    fn the_second_verifier_rule_is_carried_not_enforced() {
        // The vocabulary carries the requirement; the enforcement is the
        // caller's policy (the .5.3 wiring records it) — a record without
        // the corroboration is still a well-formed record.
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
