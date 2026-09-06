//! The WP7 workflow runner (`PHASE-0.7`): the four workflows KICKOFF WP7
//! compares, orchestrated deterministically over the adapter contract.
//!
//! - [`Workflow::Single`] — one well-prompted agent (1 call).
//! - [`Workflow::BlindIndependent`] — two agents answer WITHOUT seeing each
//!   other; a DETERMINISTIC adjudicator (the grader, never an LLM) presents the
//!   higher-scoring answer (2 calls).
//! - [`Workflow::CritiqueRevise`] — answer → critique → revision (3 calls);
//!   revision quality = the deterministic score delta.
//! - [`Workflow::ModeratorSynthesis`] — two independent answers → a synthesis
//!   call that MUST emit a structured `UNRESOLVED` register and a confidence
//!   (3 calls).
//!
//! Cost accounting (calls, tokens, wall time) is per-call and per-workflow;
//! agreement between the independent agents is recorded as a descriptive
//! observation — NEVER as a score (agreement is not correctness, and no
//! "independence score" exists anywhere in this harness).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};

use crate::bench::corpus::{Case, Prompts};
use crate::bench::grader::{grade, trap, Graded, Trap};
use crate::bench::scripted::Role;
use crate::contract::{Adapter, AttemptEvent, InvokeOutcome, NormalizedUsage, RunRequest};

static OP_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A typed benchmark error.
#[derive(Debug)]
pub enum BenchError {
    Corpus(crate::bench::corpus::CorpusError),
    /// The adapter refused before dispatch (a provider-level fact, recorded).
    AgentRefused(String),
    /// The attempt failed or the stream ended without a terminal event.
    AgentFailed(String),
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchError::Corpus(e) => write!(f, "{e}"),
            BenchError::AgentRefused(detail) => write!(f, "agent refused the call: {detail}"),
            BenchError::AgentFailed(detail) => write!(f, "agent call failed: {detail}"),
        }
    }
}

impl std::error::Error for BenchError {}

impl From<crate::bench::corpus::CorpusError> for BenchError {
    fn from(e: crate::bench::corpus::CorpusError) -> Self {
        BenchError::Corpus(e)
    }
}

/// The workflows the benchmark compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workflow {
    Single,
    BlindIndependent,
    CritiqueRevise,
    ModeratorSynthesis,
}

impl Workflow {
    pub const ALL: [Workflow; 4] = [
        Workflow::Single,
        Workflow::BlindIndependent,
        Workflow::CritiqueRevise,
        Workflow::ModeratorSynthesis,
    ];

    pub fn wire(&self) -> &'static str {
        match self {
            Workflow::Single => "single",
            Workflow::BlindIndependent => "blind",
            Workflow::CritiqueRevise => "critique_revise",
            Workflow::ModeratorSynthesis => "moderator",
        }
    }

    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "single" => Some(Workflow::Single),
            "blind" => Some(Workflow::BlindIndependent),
            "critique_revise" => Some(Workflow::CritiqueRevise),
            "moderator" => Some(Workflow::ModeratorSynthesis),
            _ => None,
        }
    }

    /// The call count this workflow performs (the cost-accounting invariant the
    /// self-test asserts).
    pub fn calls(&self) -> u64 {
        match self {
            Workflow::Single => 1,
            Workflow::BlindIndependent => 2,
            Workflow::CritiqueRevise => 3,
            Workflow::ModeratorSynthesis => 3,
        }
    }
}

/// One completed agent call, with its accounting.
#[derive(Debug, Clone, Serialize)]
pub struct AgentCall {
    pub role: String,
    pub text: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub wall_ms: u128,
    pub usage: Option<NormalizedUsage>,
}

/// The result of one workflow on one case.
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowResult {
    pub workflow: &'static str,
    /// The answer the workflow presents as its output (the scored one).
    pub final_answer: String,
    pub calls: u64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub wall_ms: u128,
    pub graded: Graded,
    pub trap: Trap,
    /// The structured-output check (moderator: unresolved register + confidence
    /// both parse; other workflows: the confidence parses).
    pub structure_valid: bool,
    /// Workflow-specific observations (agreement, revision delta, unresolved).
    pub details: Value,
    /// The intermediate calls (critique, both blind answers, …) for the record.
    pub calls_log: Vec<AgentCall>,
}

/// Drive one adapter call to a terminal event (the bench's own light caller —
/// no journal: the harness records, the node journal is the platform's job).
async fn call(
    agent: &impl Adapter,
    role: Role,
    case_id: &str,
    prompt: &str,
) -> Result<AgentCall, BenchError> {
    let operation_id = format!("benchop_{}", OP_COUNTER.fetch_add(1, Ordering::Relaxed));
    let payload = json!({ "role": role.wire(), "case_id": case_id, "prompt": prompt });
    let request = RunRequest {
        payload,
        deadline: None,
        budget_hint: None,
    };
    let start = Instant::now();
    match agent.invoke(&request, &operation_id).await {
        InvokeOutcome::FailedBeforeDispatch { reason, .. } => Err(BenchError::AgentRefused(reason)),
        InvokeOutcome::Accepted(_ack, mut handle) => {
            let mut chunks = Vec::new();
            let mut usage_raw = None;
            let mut terminal = false;
            while let Some(event) = handle.next().await {
                match event {
                    AttemptEvent::OutputChunk { chunk } => chunks.push(chunk),
                    AttemptEvent::Completed { usage } => {
                        usage_raw = usage;
                        terminal = true;
                        break;
                    }
                    AttemptEvent::FailedKnown { reason } => {
                        return Err(BenchError::AgentFailed(format!("known failure: {reason}")))
                    }
                    AttemptEvent::ProviderRequestId { .. } => {}
                }
            }
            if !terminal {
                return Err(BenchError::AgentFailed(
                    "the stream ended without a terminal event".to_string(),
                ));
            }
            let text = chunks.join("");
            let usage = usage_raw.as_ref().map(|u| agent.normalize_usage(u));
            Ok(AgentCall {
                role: role.wire().to_string(),
                text,
                input_tokens: usage.as_ref().and_then(|u| u.input_tokens).unwrap_or(0),
                output_tokens: usage.as_ref().and_then(|u| u.output_tokens).unwrap_or(0),
                wall_ms: start.elapsed().as_millis(),
                usage,
            })
        }
    }
}

fn source_block(case: &Case) -> String {
    if case.sources.is_empty() {
        return String::new();
    }
    let mut out = String::from("Sources you may cite (reference them as CITE[n]):\n");
    for (index, source) in case.sources.iter().enumerate() {
        out.push_str(&format!(
            "[{}] {} — {}\n",
            index + 1,
            source.title,
            source.excerpt
        ));
    }
    out
}

/// The moderator's structured output: the unresolved register (empty = none)
/// and whether the structure parsed.
fn parse_synthesis(text: &str) -> (Vec<String>, bool) {
    let re = Regex::new(r"(?i)unresolved\s*:\s*([^\n]*)").expect("static regex");
    let Some(captures) = re.captures(text) else {
        return (Vec::new(), false);
    };
    let register = captures
        .get(1)
        .expect("capture group")
        .as_str()
        .trim()
        .to_string();
    let entries: Vec<String> = if register.eq_ignore_ascii_case("none") || register.is_empty() {
        Vec::new()
    } else {
        register
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    };
    (entries, true)
}

/// Run one workflow on one case.
pub async fn run_workflow(
    agent: &impl Adapter,
    prompts: &Prompts,
    case: &Case,
    workflow: Workflow,
) -> Result<WorkflowResult, BenchError> {
    let sources = source_block(case);
    match workflow {
        Workflow::Single => {
            let prompt = prompts.render(
                workflow.wire(),
                &case.class,
                &[("statement", &case.statement), ("sources", &sources)],
            )?;
            let answer = call(agent, Role::AgentA, &case.id, &prompt).await?;
            let graded = grade(case, &answer.text);
            let trap = trap(case, &answer.text);
            let structure_valid = graded.confidence.is_some();
            Ok(WorkflowResult {
                workflow: workflow.wire(),
                final_answer: answer.text.clone(),
                calls: 1,
                input_tokens: answer.input_tokens,
                output_tokens: answer.output_tokens,
                wall_ms: answer.wall_ms,
                graded,
                trap,
                structure_valid,
                details: json!({}),
                calls_log: vec![answer],
            })
        }
        Workflow::BlindIndependent => {
            let prompt = prompts.render(
                workflow.wire(),
                &case.class,
                &[("statement", &case.statement), ("sources", &sources)],
            )?;
            let answer_a = call(agent, Role::AgentA, &case.id, &prompt).await?;
            let answer_b = call(agent, Role::AgentB, &case.id, &prompt).await?;
            let graded_a = grade(case, &answer_a.text);
            let graded_b = grade(case, &answer_b.text);
            // Deterministic adjudication: the higher deterministic score wins; a
            // tie presents A (recorded). NEVER a semantic-similarity decision.
            let (winner, winner_grade, tie) = if graded_b.score > graded_a.score {
                (&answer_b, &graded_b, false)
            } else if graded_a.score > graded_b.score {
                (&answer_a, &graded_a, false)
            } else {
                (&answer_a, &graded_a, true)
            };
            let agreement = answer_a
                .text
                .trim()
                .eq_ignore_ascii_case(answer_b.text.trim());
            let trap_a = trap(case, &answer_a.text);
            let trap_b = trap(case, &answer_b.text);
            Ok(WorkflowResult {
                workflow: workflow.wire(),
                final_answer: winner.text.clone(),
                calls: 2,
                input_tokens: answer_a.input_tokens + answer_b.input_tokens,
                output_tokens: answer_a.output_tokens + answer_b.output_tokens,
                wall_ms: answer_a.wall_ms + answer_b.wall_ms,
                graded: winner_grade.clone(),
                trap: trap(case, &winner.text),
                structure_valid: winner_grade.confidence.is_some(),
                details: json!({
                    "adjudication": "deterministic_grade",
                    "tie": tie,
                    "agreement": agreement,
                    "score_a": graded_a.score,
                    "score_b": graded_b.score,
                    "confidence_a": graded_a.confidence,
                    "confidence_b": graded_b.confidence,
                    "asserted_claim_a": trap_a.asserted_claim,
                    "asserted_claim_b": trap_b.asserted_claim,
                }),
                calls_log: vec![answer_a, answer_b],
            })
        }
        Workflow::CritiqueRevise => {
            let answer_prompt = prompts.render(
                "single",
                &case.class,
                &[("statement", &case.statement), ("sources", &sources)],
            )?;
            let original = call(agent, Role::AgentA, &case.id, &answer_prompt).await?;
            let original_grade = grade(case, &original.text);

            let critique_prompt = prompts.render(
                "critique",
                &case.class,
                &[
                    ("statement", &case.statement),
                    ("sources", &sources),
                    ("answer", &original.text),
                ],
            )?;
            let critique = call(agent, Role::Critique, &case.id, &critique_prompt).await?;

            // A DISTINCT revision template (the first real run caught the
            // conflation: the revision leg re-rendered the critique instruction,
            // every revision returned no confidence and one broke the answer).
            let revision_prompt = prompts.render(
                "revision",
                &case.class,
                &[
                    ("statement", &case.statement),
                    ("sources", &sources),
                    ("answer", &original.text),
                    ("critique", &critique.text),
                ],
            )?;
            let revision = call(agent, Role::Revision, &case.id, &revision_prompt).await?;
            let revision_grade = grade(case, &revision.text);
            Ok(WorkflowResult {
                workflow: workflow.wire(),
                final_answer: revision.text.clone(),
                calls: 3,
                input_tokens: original.input_tokens + critique.input_tokens + revision.input_tokens,
                output_tokens: original.output_tokens
                    + critique.output_tokens
                    + revision.output_tokens,
                wall_ms: original.wall_ms + critique.wall_ms + revision.wall_ms,
                graded: revision_grade.clone(),
                trap: trap(case, &revision.text),
                structure_valid: revision_grade.confidence.is_some(),
                details: json!({
                    "score_original": original_grade.score,
                    "score_revision": revision_grade.score,
                    "revision_delta": revision_grade.score - original_grade.score,
                }),
                calls_log: vec![original, critique, revision],
            })
        }
        Workflow::ModeratorSynthesis => {
            let answer_prompt = prompts.render(
                "single",
                &case.class,
                &[("statement", &case.statement), ("sources", &sources)],
            )?;
            let answer_a = call(agent, Role::AgentA, &case.id, &answer_prompt).await?;
            let answer_b = call(agent, Role::AgentB, &case.id, &answer_prompt).await?;
            let synthesis_prompt = prompts.render(
                workflow.wire(),
                &case.class,
                &[
                    ("statement", &case.statement),
                    ("sources", &sources),
                    ("answer_a", &answer_a.text),
                    ("answer_b", &answer_b.text),
                ],
            )?;
            let synthesis = call(agent, Role::Moderator, &case.id, &synthesis_prompt).await?;
            let graded = grade(case, &synthesis.text);
            let (unresolved, unresolved_parsed) = parse_synthesis(&synthesis.text);
            let structure_valid = unresolved_parsed && graded.confidence.is_some();
            Ok(WorkflowResult {
                workflow: workflow.wire(),
                final_answer: synthesis.text.clone(),
                calls: 3,
                input_tokens: answer_a.input_tokens
                    + answer_b.input_tokens
                    + synthesis.input_tokens,
                output_tokens: answer_a.output_tokens
                    + answer_b.output_tokens
                    + synthesis.output_tokens,
                wall_ms: answer_a.wall_ms + answer_b.wall_ms + synthesis.wall_ms,
                graded,
                trap: trap(case, &synthesis.text),
                structure_valid,
                details: json!({
                    "unresolved_register": unresolved,
                    "unresolved_count": unresolved.len(),
                }),
                calls_log: vec![answer_a, answer_b, synthesis],
            })
        }
    }
}
