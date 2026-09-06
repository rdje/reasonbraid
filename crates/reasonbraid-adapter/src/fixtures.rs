//! The sanitized deterministic outcome corpus (`PHASE-0.4.1`): one fixture per adapter
//! outcome, consumable by the fake adapter (via [`crate::fake::FixtureSpec`]) and by
//! the node's supervisor tests. Every fixture is JSON, carries NO credential (enforced
//! by [`tests::corpus_is_credential_free`] mechanically), and every script step and
//! outcome class is covered ([`tests::corpus_covers_every_step_and_outcome_class`]).
//!
//! A real adapter (`.4.2`) must reproduce these semantics behind the same
//! [`crate::contract::Adapter`] contract — the corpus is the conformance oracle.

use crate::fake::FixtureSpec;

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
}
