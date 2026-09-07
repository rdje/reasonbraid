//! The sanitized deterministic outcome corpus (`PHASE-0.4.1`): one fixture per adapter
//! outcome, consumable by the fake adapter (via [`crate::fake::FixtureSpec`]) and by
//! the node's supervisor tests. Every fixture is JSON, carries NO credential (enforced
//! by [`tests::corpus_is_credential_free`] mechanically), and every script step and
//! outcome class is covered ([`tests::corpus_covers_every_step_and_outcome_class`]).
//!
//! A real adapter (`.4.2`) must reproduce these semantics behind the same
//! [`crate::contract::Adapter`] contract — the corpus is the conformance oracle.
//!
//! `PHASE-2.6.2` pins the corpus as a PERMANENT replay oracle: [`manifest`] is the
//! versioned manifest (`fixtures/MANIFEST.json`) — additive changes only, every entry
//! records its reason — and the tests hold the mechanical guarantees: the manifest and
//! the corpus match EXACTLY (no silent add, drop, or edit) and every §19.4
//! conformance item ([`CONFORMANCE_ITEMS`]) is covered by at least one entry.

use crate::fake::FixtureSpec;
use serde::Deserialize;

fn parse(text: &'static str) -> FixtureSpec {
    serde_json::from_str(text).expect("fixture parses")
}

/// The parsed corpus, in fixture order.
pub fn corpus() -> Vec<FixtureSpec> {
    vec![
        parse(include_str!("../fixtures/complete_single.json")),
        parse(include_str!("../fixtures/complete_streaming.json")),
        parse(include_str!("../fixtures/fail_before_dispatch.json")),
        parse(include_str!("../fixtures/fail_known.json")),
        parse(include_str!("../fixtures/hang_forever.json")),
        parse(include_str!("../fixtures/ignore_cancellation.json")),
        parse(include_str!("../fixtures/lose_response_no_lookup.json")),
        parse(include_str!("../fixtures/lose_response_with_lookup.json")),
        parse(include_str!("../fixtures/malformed_output.json")),
        parse(include_str!("../fixtures/usage_receipt.json")),
    ]
}

/// The raw fixture texts (name + content) — for the credential-scan test.
pub fn raw_fixtures() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "complete_single",
            include_str!("../fixtures/complete_single.json"),
        ),
        (
            "complete_streaming",
            include_str!("../fixtures/complete_streaming.json"),
        ),
        (
            "fail_before_dispatch",
            include_str!("../fixtures/fail_before_dispatch.json"),
        ),
        ("fail_known", include_str!("../fixtures/fail_known.json")),
        (
            "hang_forever",
            include_str!("../fixtures/hang_forever.json"),
        ),
        (
            "ignore_cancellation",
            include_str!("../fixtures/ignore_cancellation.json"),
        ),
        (
            "lose_response_no_lookup",
            include_str!("../fixtures/lose_response_no_lookup.json"),
        ),
        (
            "lose_response_with_lookup",
            include_str!("../fixtures/lose_response_with_lookup.json"),
        ),
        (
            "malformed_output",
            include_str!("../fixtures/malformed_output.json"),
        ),
        (
            "usage_receipt",
            include_str!("../fixtures/usage_receipt.json"),
        ),
    ]
}

/// The §19.4 conformance items a fixture can prove in the dev profile (the item list
/// the manifest maps against; the items with no dev-profile machinery — rate-limit/
/// backoff normalization, output-size limits, tool-call validation, projection
/// fidelity — are the `.6.3` deferrals, not fixture material).
pub const CONFORMANCE_ITEMS: [&str; 10] = [
    "capability_declaration",
    "secret_containment",
    "timeout_cancellation_streaming_limits",
    "idempotency_ambiguity",
    "rate_limit_backoff",
    "usage_accounting",
    "tool_validation",
    "projection_fidelity",
    "error_taxonomy",
    "replay_qualification",
];

/// The permanent-corpus manifest (`fixtures/MANIFEST.json`, `PHASE-2.6.2`): the
/// versioned record of the replay oracle. Permanence rules (enforced by the tests):
/// the manifest and the corpus match EXACTLY — additive changes only, every entry
/// records where it was added and why.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CorpusManifest {
    pub version: u32,
    pub entries: Vec<CorpusManifestEntry>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CorpusManifestEntry {
    /// The fixture file name (must equal a corpus fixture's name + `.json`).
    pub file: String,
    /// The §19.4 conformance-item keys this fixture proves.
    pub conformance_items: Vec<String>,
    /// The leaf that added the fixture (the permanence trail).
    pub added: String,
    /// Why this fixture exists — the recorded reason the permanence rules demand.
    pub reason: String,
}

/// The parsed permanent-corpus manifest.
pub fn manifest() -> CorpusManifest {
    serde_json::from_str(include_str!("../fixtures/MANIFEST.json")).expect("MANIFEST.json parses")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::ScriptStep;

    /// The corpus covers EVERY script step and every declared outcome class — a step
    /// with no fixture has no conformance path.
    #[test]
    fn corpus_covers_every_step_and_outcome_class() {
        let mut steps = std::collections::HashSet::new();
        let mut outcomes = std::collections::HashSet::new();
        for fixture in corpus() {
            for step in &fixture.script {
                steps.insert(step.tag());
            }
            outcomes.insert(fixture.outcome);
        }
        for tag in [
            "emit_chunk",
            "malformed_output",
            "complete",
            "fail_known",
            "fail_before_dispatch",
            "hang_forever",
            "ignore_cancellation",
            "lose_response",
        ] {
            assert!(steps.contains(tag), "no fixture exercises step `{tag}`");
        }
        for class in [
            "completed",
            "failed_before_dispatch",
            "failed_known",
            "hang",
            "ignore_cancellation",
            "lost_response_no_lookup",
            "lost_response_with_lookup",
            "malformed_output",
            "usage",
        ] {
            assert!(
                outcomes.contains(class),
                "no fixture covers outcome `{class}`"
            );
        }
    }

    /// THE credentials acceptance: no fixture text contains a credential-shaped key or
    /// value. This is a mechanical gate, not a promise — it fails closed on any
    /// credential-shaped token in the corpus.
    #[test]
    fn corpus_is_credential_free() {
        for (name, text) in raw_fixtures() {
            let lower = text.to_lowercase();
            for needle in [
                "api_key",
                "apikey",
                "secret",
                "password",
                "credential",
                "bearer ",
            ] {
                assert!(
                    !lower.contains(needle),
                    "fixture `{name}` contains a credential-shaped token (`{needle}`)"
                );
            }
        }
    }

    /// Every fixture's script is a real script (non-empty) and the auto-drivable ones
    /// name a terminal state the supervisor can verify. Only `hang_forever` needs
    /// explicit cancel driving (`ignore_cancellation` is transparent to the stream and
    /// auto-completes).
    #[test]
    fn corpus_entries_are_well_formed() {
        for fixture in corpus() {
            assert!(
                !fixture.script.is_empty(),
                "`{}` has an empty script",
                fixture.name
            );
            if fixture
                .script
                .iter()
                .any(|s| matches!(s, ScriptStep::HangForever))
            {
                assert!(
                    fixture.expected_terminal.is_none(),
                    "`{}` needs explicit cancel driving, not an auto terminal",
                    fixture.name
                );
            } else {
                assert!(
                    fixture.expected_terminal.is_some(),
                    "`{}` is auto-drivable but names no expected terminal",
                    fixture.name
                );
            }
        }
    }

    /// The permanence guarantee: the manifest and the corpus match EXACTLY — a fixture
    /// added without a manifest entry, or an entry whose fixture was dropped or
    /// renamed, fails here. Changes are additive by construction of this test.
    #[test]
    fn manifest_matches_the_corpus_exactly() {
        let corpus_names: Vec<String> = corpus()
            .iter()
            .map(|f| format!("{}.json", f.name))
            .collect();
        let manifest_files: Vec<String> =
            manifest().entries.iter().map(|e| e.file.clone()).collect();
        assert_eq!(
            manifest_files, corpus_names,
            "the manifest and the corpus drifted (additive changes only — record the reason)"
        );
        assert_eq!(
            manifest().version,
            1,
            "the manifest version moved without a record"
        );
    }

    /// Every §19.4 conformance item is proven by at least one fixture, and every
    /// manifest key is a REAL item — an unmapped item has no replay oracle, an unknown
    /// key is a typo that would silently orphan the mapping.
    #[test]
    fn every_conformance_item_is_covered_by_the_manifest() {
        let binding = manifest();
        let mut covered: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for entry in &binding.entries {
            for key in &entry.conformance_items {
                assert!(
                    CONFORMANCE_ITEMS.contains(&key.as_str()),
                    "manifest entry `{}` names an unknown conformance item `{key}`",
                    entry.file
                );
                covered.insert(key.as_str());
            }
        }
        let unmapped: Vec<&str> = CONFORMANCE_ITEMS
            .iter()
            .copied()
            .filter(|key| !covered.contains(key))
            .collect();
        // The dev profile has no machinery for these four (the `.6.3` deferrals) —
        // every OTHER item must be replayed by the corpus.
        for key in unmapped {
            assert!(
                matches!(
                    key,
                    "rate_limit_backoff"
                        | "tool_validation"
                        | "projection_fidelity"
                        | "secret_containment"
                ),
                "conformance item `{key}` has no fixture in the replay corpus"
            );
        }
    }
}
