//! The WP7 benchmark self-test (`PHASE-0.7`): the scripted agent plays the whole
//! corpus through every workflow, and the harness's OWN expected scores (carried
//! in the corpus) must be reproduced exactly — the measurement pipeline is
//! proven before any real tokens are spent.
//!
//! Also asserted here: the report carries NO independence score (agreement is
//! descriptive), aggregates are spread-bearing (never a bare average), Brier is
//! computed for factual cases only, and the insufficient-evidence honesty trap
//! flags asserted numbers.

use std::path::PathBuf;

use reasonbraid_adapter::bench::{
    corpus::{Corpus, Prompts},
    report::build,
    scripted::ScriptedAgent,
    workflows::{run_workflow, Workflow},
};

fn bench_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bench/v1")
}

/// Run every workflow on every case with the scripted agent and assert the
/// computed scores equal the corpus's expected ones (the harness's oracle).
#[tokio::test]
async fn scripted_run_reproduces_the_corpus_oracle_exactly() {
    let dir = bench_dir();
    let corpus = Corpus::load(&dir).expect("corpus loads");
    let prompts = Prompts::load(&dir).expect("prompts load");
    let agent = ScriptedAgent::new(&corpus.cases);

    for case in &corpus.cases {
        for workflow in Workflow::ALL {
            let result = run_workflow(&agent, &prompts, case, workflow)
                .await
                .unwrap_or_else(|e| panic!("case {} · {}: {e}", case.id, workflow.wire()));
            let expected = match workflow {
                Workflow::Single => case.scripted.expected.single,
                Workflow::BlindIndependent => case.scripted.expected.blind,
                Workflow::CritiqueRevise => case.scripted.expected.critique_revise,
                Workflow::ModeratorSynthesis => case.scripted.expected.moderator,
            };
            assert!(
                (result.graded.score - expected).abs() < 1e-9,
                "case {} · {}: computed {} != expected {} (answer: {:?})",
                case.id,
                workflow.wire(),
                result.graded.score,
                expected,
                result.final_answer
            );
            assert_eq!(
                result.calls,
                workflow.calls(),
                "case {} · {}: call accounting",
                case.id,
                workflow.wire()
            );
            assert!(
                result.structure_valid,
                "case {} · {}: the scripted output must satisfy the structure check",
                case.id,
                workflow.wire()
            );
            assert!(
                result.input_tokens > 0 && result.output_tokens > 0,
                "case {} · {}: token accounting is never zero",
                case.id,
                workflow.wire()
            );
            if workflow == Workflow::ModeratorSynthesis {
                let count = result
                    .details
                    .get("unresolved_count")
                    .and_then(|v| v.as_u64())
                    .expect("unresolved_count present");
                assert_eq!(
                    count,
                    case.scripted.synthesis.unresolved.len() as u64,
                    "case {}: the unresolved register is parsed faithfully",
                    case.id
                );
            }
        }
    }
}

/// The insufficient-evidence honesty trap: the scripted blind partner asserts a
/// concrete number the sources cannot support, and the trap flags it (recorded
/// in the blind details even when the refuser wins the adjudication).
#[tokio::test]
async fn the_honesty_trap_flags_the_asserting_agent() {
    let dir = bench_dir();
    let corpus = Corpus::load(&dir).expect("corpus loads");
    let prompts = Prompts::load(&dir).expect("prompts load");
    let agent = ScriptedAgent::new(&corpus.cases);
    let case = corpus
        .cases
        .iter()
        .find(|c| c.id == "insuff-001")
        .expect("the case exists");

    let result = run_workflow(&agent, &prompts, case, Workflow::BlindIndependent)
        .await
        .expect("runs");
    assert_eq!(
        result.details["asserted_claim_b"].as_bool(),
        Some(true),
        "the asserting agent is flagged: {}",
        result.details
    );
    assert_eq!(
        result.details["asserted_claim_a"].as_bool(),
        Some(false),
        "the refuser is not flagged"
    );
}

/// The report hygiene rules: no independence score, spread-bearing aggregates,
/// factual-only Brier.
#[tokio::test]
async fn the_report_refuses_independence_scores_and_bare_averages() {
    let dir = bench_dir();
    let corpus = Corpus::load(&dir).expect("corpus loads");
    let prompts = Prompts::load(&dir).expect("prompts load");
    let agent = ScriptedAgent::new(&corpus.cases);

    let mut results = Vec::new();
    for case in &corpus.cases {
        let mut case_results = Vec::new();
        for workflow in Workflow::ALL {
            case_results.push(
                run_workflow(&agent, &prompts, case, workflow)
                    .await
                    .expect("runs"),
            );
        }
        results.push((case.id.clone(), case_results));
    }

    let report = build(
        "self-test",
        "test",
        "scripted",
        &corpus,
        "corpus-digest",
        "prompts-digest",
        &results,
    );
    let json = serde_json::to_string(&report).expect("serializes");
    // The typed shape IS the guarantee: no metric key or case-row field derives
    // any score from agreement/similarity between answers.
    for (class, by_workflow) in &report.aggregates {
        for (workflow, metrics) in by_workflow {
            for metric in metrics.keys() {
                assert!(
                    !metric.contains("independen") && !metric.contains("agreement"),
                    "{class} · {workflow}: forbidden metric `{metric}`"
                );
            }
        }
    }
    // The agreement section is a descriptive observation with the warning.
    assert!(
        json.contains("\"agreement\""),
        "the descriptive section exists"
    );
    assert!(
        json.contains("agreement is not correctness"),
        "the report states agreement ≠ correctness"
    );

    for (class, by_workflow) in &report.aggregates {
        for (workflow, metrics) in by_workflow {
            for (metric, spread) in metrics {
                assert!(
                    spread.n > 0,
                    "{class} · {workflow} · {metric}: empty aggregate"
                );
                assert!(
                    spread.min <= spread.max,
                    "{class} · {workflow} · {metric}: min/max inverted"
                );
                assert!(
                    (spread.min..=spread.max).contains(&spread.mean)
                        || (spread.max..=spread.min).contains(&spread.mean),
                    "{class} · {workflow} · {metric}: mean outside [min, max]"
                );
            }
        }
    }

    for row in &report.rows {
        if row.class == "factual" {
            assert!(
                row.brier.is_some(),
                "{} · {}: factual rows carry a Brier score",
                row.case_id,
                row.workflow
            );
        } else {
            assert!(
                row.brier.is_none(),
                "{} · {}: no invented calibration outside factual cases",
                row.case_id,
                row.workflow
            );
        }
    }

    assert!(
        report.agreement.compared_count == 8,
        "agreement compared over every case"
    );
    assert_eq!(report.not_measured.len(), 2);
}

/// The factual Brier direction: a confident wrong answer scores worse than a
/// confident right one (the calibration signal the report exposes per case).
#[test]
fn brier_penalizes_confident_mistakes() {
    use reasonbraid_adapter::bench::grader::brier;
    let right = brier(Some(0.95), 1.0).expect("confidence");
    let wrong = brier(Some(0.95), 0.0).expect("confidence");
    let humble_wrong = brier(Some(0.2), 0.0).expect("confidence");
    assert!(right < 0.01);
    assert!(wrong > 0.8);
    assert!(
        humble_wrong < wrong,
        "low confidence mitigates the Brier penalty"
    );
}

/// The prompt-wiring regression: the first REAL run showed the revision leg
/// re-rendering the CRITIQUE instruction (no confidence lines, one broken
/// answer). The five workflow templates must exist, and critique/revision must
/// be distinct instructions — the scripted oracle cannot see this class (it
/// answers by role), so the corpus itself carries the check.
#[test]
fn the_critique_and_revision_prompts_are_distinct() {
    let dir = bench_dir();
    let prompts = Prompts::load(&dir).expect("prompts load");
    for workflow in ["single", "blind", "critique", "revision", "moderator"] {
        assert!(
            prompts.workflows.contains_key(workflow),
            "missing workflow template `{workflow}`"
        );
    }
    for class in [
        "factual",
        "code_review",
        "ambiguous_policy",
        "insufficient_evidence",
    ] {
        let critique = prompts
            .render("critique", class, &[])
            .unwrap_or_else(|e| panic!("{class}: {e}"));
        let revision = prompts
            .render("revision", class, &[])
            .unwrap_or_else(|e| panic!("{class}: {e}"));
        assert!(
            critique.to_lowercase().contains("critique"),
            "{class}: the critique template names its role"
        );
        assert!(
            revision.to_lowercase().contains("revising")
                || revision.to_lowercase().contains("revised"),
            "{class}: the revision template names its role"
        );
        assert_ne!(critique, revision, "{class}: distinct instructions");
    }
}
