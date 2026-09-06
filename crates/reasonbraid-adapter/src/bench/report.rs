//! The WP7 reporter (`PHASE-0.7`): the versioned, machine-readable report and its
//! human-readable rendering.
//!
//! # What the report refuses to do
//!
//! - **No independence score.** Agreement between independent answers is a
//!   descriptive count with an explicit warning that agreement ≠ correctness —
//!   never a number that ranks agents.
//! - **No average-only summaries.** Every aggregate carries the case count, the
//!   minimum, and the maximum beside the mean (§13.7: "results include cases and
//!   uncertainty, not only an average").
//! - **No invented calibration.** Brier scores are computed for factual cases
//!   only (a rubric case has no binary outcome); everything else is reported as
//!   not-applicable, never guessed.
//! - **Unmeasured stays unmeasured.** Human-review minutes are recorded as
//!   `not_measured` with the reason — unknown is not zero (§14.1).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::{json, Value};

use crate::bench::corpus::{CLASS_FACTUAL, CLASS_INSUFFICIENT_EVIDENCE};
use crate::bench::grader::brier;
use crate::bench::workflows::WorkflowResult;

/// One workflow's result row for one case.
#[derive(Debug, Clone, Serialize)]
pub struct CaseRow {
    pub case_id: String,
    pub class: String,
    pub workflow: String,
    pub score: f64,
    /// `Some` when the answer declared a parseable confidence; `None` records a
    /// structure failure instead of guessing one.
    pub confidence: Option<f64>,
    /// Brier vs the binary outcome — factual cases only, else `None`.
    pub brier: Option<f64>,
    pub calls: u64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub wall_ms: u128,
    pub structure_valid: bool,
    pub citations_valid: usize,
    pub citations_invalid: usize,
    /// The insufficient-evidence honesty trap: the answer asserted a concrete
    /// claim the case cannot support. `None` on other classes (the trap does
    /// not apply — recorded as not-applicable, never as "no claim").
    pub asserted_claim: Option<bool>,
}

/// A spread-bearing aggregate (n, min, max, mean) for one metric.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Spread {
    pub n: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

impl Spread {
    pub fn of(values: &[f64]) -> Self {
        let n = values.len();
        if n == 0 {
            return Spread {
                n: 0,
                min: f64::NAN,
                max: f64::NAN,
                mean: f64::NAN,
            };
        }
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().sum::<f64>() / n as f64;
        Spread { n, min, max, mean }
    }
}

/// The full run report (schema v1).
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub schema: u32,
    pub run_id: String,
    pub git_rev: String,
    pub agent: String,
    pub corpus_version: u32,
    pub corpus_digest: String,
    pub prompts_digest: String,
    pub rows: Vec<CaseRow>,
    /// class → workflow → metric → spread.
    pub aggregates: BTreeMap<String, BTreeMap<String, BTreeMap<String, Spread>>>,
    /// Descriptive observations, never scores.
    pub agreement: Agreement,
    /// Explicitly unmeasured metrics and why.
    pub not_measured: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Agreement {
    /// Cases where the two independent answers were textually equal.
    pub equal_count: usize,
    pub compared_count: usize,
    pub note: &'static str,
}

/// Build the report from per-case workflow results.
pub fn build(
    run_id: &str,
    git_rev: &str,
    agent: &str,
    corpus: &crate::bench::corpus::Corpus,
    corpus_digest: &str,
    prompts_digest: &str,
    results: &[(String, Vec<WorkflowResult>)],
) -> Report {
    let mut rows = Vec::new();
    for (case_id, workflow_results) in results {
        let case = corpus
            .cases
            .iter()
            .find(|c| c.id == *case_id)
            .expect("every result row belongs to a corpus case");
        for result in workflow_results {
            let outcome = if result.graded.score >= 1.0 { 1.0 } else { 0.0 };
            rows.push(CaseRow {
                case_id: case_id.clone(),
                class: case.class.clone(),
                workflow: result.workflow.to_string(),
                score: result.graded.score,
                confidence: result.graded.confidence,
                brier: (case.class == CLASS_FACTUAL)
                    .then(|| brier(result.graded.confidence, outcome))
                    .flatten(),
                calls: result.calls,
                input_tokens: result.input_tokens,
                output_tokens: result.output_tokens,
                wall_ms: result.wall_ms,
                structure_valid: result.structure_valid,
                citations_valid: result.graded.citations_valid,
                citations_invalid: result.graded.citations_invalid,
                asserted_claim: (case.class == CLASS_INSUFFICIENT_EVIDENCE)
                    .then_some(result.trap.asserted_claim),
            });
        }
    }

    // class → workflow → metric → values (reduced to spreads afterwards).
    let mut values: BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<f64>>>> =
        BTreeMap::new();
    for row in &rows {
        let by_workflow = values
            .entry(row.class.clone())
            .or_default()
            .entry(row.workflow.clone())
            .or_default();
        by_workflow
            .entry("score".to_string())
            .or_default()
            .push(row.score);
        if let Some(confidence) = row.confidence {
            by_workflow
                .entry("confidence".to_string())
                .or_default()
                .push(confidence);
        }
        if let Some(b) = row.brier {
            by_workflow
                .entry("brier_factual".to_string())
                .or_default()
                .push(b);
        }
        by_workflow
            .entry("calls".to_string())
            .or_default()
            .push(row.calls as f64);
        by_workflow
            .entry("input_tokens".to_string())
            .or_default()
            .push(row.input_tokens as f64);
        by_workflow
            .entry("output_tokens".to_string())
            .or_default()
            .push(row.output_tokens as f64);
        by_workflow
            .entry("wall_ms".to_string())
            .or_default()
            .push(row.wall_ms as f64);
    }
    let mut aggregates: BTreeMap<String, BTreeMap<String, BTreeMap<String, Spread>>> =
        BTreeMap::new();
    for (class, by_workflow) in values {
        for (workflow, metrics) in by_workflow {
            for (name, collected) in metrics {
                aggregates
                    .entry(class.clone())
                    .or_default()
                    .entry(workflow.clone())
                    .or_default()
                    .insert(name, Spread::of(&collected));
            }
        }
    }

    // Agreement (descriptive only).
    let mut equal_count = 0;
    let mut compared_count = 0;
    for (_case_id, workflow_results) in results {
        if let Some(blind) = workflow_results.iter().find(|r| r.workflow == "blind") {
            compared_count += 1;
            if blind.details.get("agreement").and_then(|v| v.as_bool()) == Some(true) {
                equal_count += 1;
            }
        }
    }

    Report {
        schema: 1,
        run_id: run_id.to_string(),
        git_rev: git_rev.to_string(),
        agent: agent.to_string(),
        corpus_version: corpus.version,
        corpus_digest: corpus_digest.to_string(),
        prompts_digest: prompts_digest.to_string(),
        rows,
        aggregates,
        agreement: Agreement {
            equal_count,
            compared_count,
            note: "agreement between independent answers is a descriptive count, \
                   NOT a score: agreement is not correctness, and no independence \
                   score is computed by this harness",
        },
        not_measured: vec![
            json!({
                "metric": "human_review_minutes",
                "reason": "requires a human review study — owned by WP8, not machine-measurable here"
            }),
            json!({
                "metric": "reversal_rate",
                "reason": "requires longitudinal re-runs; Phase 1's enduring harness"
            }),
        ],
    }
}

/// Write the report (JSON + a human-readable markdown) under `out_dir`.
pub fn write(report: &Report, out_dir: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(
        out_dir.join("report.json"),
        serde_json::to_string_pretty(report).expect("report serializes") + "\n",
    )?;
    std::fs::write(out_dir.join("report.md"), markdown(report))?;
    Ok(())
}

fn spread_row(name: &str, spread: &Spread) -> String {
    if spread.n == 0 {
        return format!("| {name} | — | — | — | — |");
    }
    format!(
        "| {name} | {} | {:.3} | {:.3} | {:.3} |",
        spread.n, spread.min, spread.mean, spread.max
    )
}

/// The human-readable rendering (the director's review surface).
pub fn markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# WP7 benchmark report — run `{}`\n\n",
        report.run_id
    ));
    out.push_str(&format!(
        "- agent: `{}` · corpus v{} (`{}`)\n- prompts digest: `{}`\n- git: `{}`\n\n",
        report.agent,
        report.corpus_version,
        &report.corpus_digest[..16.min(report.corpus_digest.len())],
        &report.prompts_digest[..16.min(report.prompts_digest.len())],
        report.git_rev
    ));
    out.push_str("## Per-case rows\n\n");
    out.push_str(
        "| case | class | workflow | score | confidence | brier(factual) | calls | tok in | tok out | wall ms | struct | cite ok/bad | asserted claim |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for row in &report.rows {
        out.push_str(&format!(
            "| {} | {} | {} | {:.3} | {} | {} | {} | {} | {} | {} | {} | {}/{} | {} |\n",
            row.case_id,
            row.class,
            row.workflow,
            row.score,
            row.confidence
                .map_or("∅".to_string(), |c| format!("{c:.2}")),
            row.brier.map_or("—".to_string(), |b| format!("{b:.3}")),
            row.calls,
            row.input_tokens,
            row.output_tokens,
            row.wall_ms,
            row.structure_valid,
            row.citations_valid,
            row.citations_invalid,
            row.asserted_claim
                .map_or("—".to_string(), |claimed| claimed.to_string()),
        ));
    }
    out.push_str("\n## Aggregates (spread-bearing — min/mean/max, never a bare average)\n\n");
    for (class, by_workflow) in &report.aggregates {
        for (workflow, metrics) in by_workflow {
            out.push_str(&format!("### {class} · {workflow}\n\n"));
            out.push_str("| metric | n | min | mean | max |\n|---|---|---|---|---|\n");
            for (name, spread) in metrics {
                out.push_str(&spread_row(name, spread));
                out.push('\n');
            }
            out.push('\n');
        }
    }
    out.push_str(&format!(
        "## Agreement (descriptive, not a score)\n\n\
         Independent answers were textually equal in {} of {} compared cases.\n\n\
         > {}\n\n",
        report.agreement.equal_count, report.agreement.compared_count, report.agreement.note
    ));
    out.push_str("## Not measured\n\n");
    for item in &report.not_measured {
        out.push_str(&format!(
            "- **{}**: {}\n",
            item["metric"].as_str().unwrap_or("?"),
            item["reason"].as_str().unwrap_or("?")
        ));
    }
    out
}
