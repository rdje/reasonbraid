//! The WP7 benchmark corpus (`PHASE-0.7`): the versioned case set the workflow
//! comparison runs against.
//!
//! # Why the corpus carries scripted answers and expected scores
//!
//! The deterministic `scripted` agent mode exists to prove the HARNESS — the
//! graders, the workflow routing, the cost accounting — before any real tokens
//! are spent. Every case therefore carries scripted agent outputs AND the score
//! each workflow MUST produce for them; the self-test (`tests/bench_harness.rs`)
//! asserts computed == expected over the whole corpus, so a wrong measurement
//! pipeline fails loudly instead of publishing wrong numbers.
//!
//! # Versioning
//!
//! The corpus and prompt files live under `bench/v1/` and are hashed (SHA-256)
//! into every report: a result is always traceable to the exact inputs that
//! produced it (§13.7: "hypotheses, prompts, models, datasets, scoring rules …
//! are versioned before evaluation").

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The four case classes KICKOFF WP7 names.
pub const CLASS_FACTUAL: &str = "factual";
pub const CLASS_CODE_REVIEW: &str = "code_review";
pub const CLASS_AMBIGUOUS_POLICY: &str = "ambiguous_policy";
pub const CLASS_INSUFFICIENT_EVIDENCE: &str = "insufficient_evidence";

/// A case's ground truth: factual cases carry accepted answer keys; rubric cases
/// carry a deterministic presence checklist (never an LLM judge — an LLM grader
/// would share the measured models' correlated errors, §13.7).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GroundTruth {
    /// Accepted answer keys; the normalized answer must CONTAIN one of them.
    Factual {
        /// Lowercased answer keys (the grader normalizes both sides).
        accepted: Vec<String>,
    },
    /// A deterministic checklist: the score is the fraction of checks present.
    Rubric { checks: Vec<RubricCheck> },
}

/// One rubric check: any of the needles present in the normalized answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RubricCheck {
    pub id: String,
    pub label: String,
    /// Any-of needles (normalized contains-match).
    pub needles: Vec<String>,
}

/// One cited source the case offers (code-review / policy / insufficient-evidence
/// cases; factual cases cite nothing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSource {
    pub id: String,
    pub title: String,
    /// The (short) content the agents may cite from.
    pub excerpt: String,
}

/// The scripted outputs one case's deterministic agents produce, per role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptedAnswers {
    /// The first agent's answer (single / blind-A / critique-revise original).
    pub agent_a: ScriptedTurn,
    /// The second agent's answer (blind-B).
    pub agent_b: ScriptedTurn,
    /// The critique of A's answer (the critique/revise second call).
    pub critique: ScriptedTurn,
    /// The revision agent A produces after the critique.
    pub revision: ScriptedTurn,
    /// The moderator's synthesis (with its unresolved register).
    pub synthesis: ScriptedSynthesis,
    /// The score each workflow MUST produce for these scripted outputs — the
    /// self-test's oracle (the harness's expected values, not a quality claim).
    pub expected: ExpectedScores,
}

/// One scripted turn: the raw text and the confidence line it carries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptedTurn {
    pub text: String,
    /// The confidence in [0, 1] the scripted answer declares.
    pub confidence: f64,
}

/// A scripted synthesis: text + confidence + the unresolved register.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptedSynthesis {
    pub text: String,
    pub confidence: f64,
    pub unresolved: Vec<String>,
}

/// The expected workflow scores for the scripted outputs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedScores {
    pub single: f64,
    pub blind: f64,
    pub critique_revise: f64,
    pub moderator: f64,
}

/// One benchmark case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub class: String,
    pub statement: String,
    pub ground_truth: GroundTruth,
    /// The source list (empty for factual cases).
    #[serde(default)]
    pub sources: Vec<CaseSource>,
    pub scripted: ScriptedAnswers,
}

/// The versioned corpus document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    pub version: u32,
    pub cases: Vec<Case>,
}

impl Corpus {
    /// Load and validate the corpus from `bench/v1/corpus.json`.
    pub fn load(dir: &Path) -> Result<Self, CorpusError> {
        let raw = std::fs::read_to_string(dir.join("corpus.json"))
            .map_err(|e| CorpusError::Io(e.to_string()))?;
        let corpus: Corpus = serde_json::from_str(&raw)
            .map_err(|e| CorpusError::Malformed(format!("corpus.json: {e}")))?;
        corpus.validate()?;
        Ok(corpus)
    }

    /// Validate structural invariants before any run (fail early, not mid-run).
    pub fn validate(&self) -> Result<(), CorpusError> {
        if self.cases.is_empty() {
            return Err(CorpusError::Malformed(
                "the corpus has no cases".to_string(),
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for case in &self.cases {
            if !seen.insert(case.id.clone()) {
                return Err(CorpusError::Malformed(format!(
                    "duplicate case id `{}`",
                    case.id
                )));
            }
            if !matches!(
                case.class.as_str(),
                CLASS_FACTUAL
                    | CLASS_CODE_REVIEW
                    | CLASS_AMBIGUOUS_POLICY
                    | CLASS_INSUFFICIENT_EVIDENCE
            ) {
                return Err(CorpusError::Malformed(format!(
                    "case `{}` has unknown class `{}`",
                    case.id, case.class
                )));
            }
            for turn in [
                &case.scripted.agent_a,
                &case.scripted.agent_b,
                &case.scripted.critique,
                &case.scripted.revision,
            ] {
                if !(0.0..=1.0).contains(&turn.confidence) {
                    return Err(CorpusError::Malformed(format!(
                        "case `{}` has a confidence outside [0, 1]",
                        case.id
                    )));
                }
            }
        }
        Ok(())
    }

    /// The SHA-256 digest of the corpus file — the versioning anchor in reports.
    pub fn digest(dir: &Path) -> Result<String, CorpusError> {
        let bytes =
            std::fs::read(dir.join("corpus.json")).map_err(|e| CorpusError::Io(e.to_string()))?;
        Ok(hex(Sha256::digest(&bytes)))
    }
}

/// The versioned prompt templates (`bench/v1/prompts.json`), one per workflow
/// with class-specific instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prompts {
    pub version: u32,
    /// workflow name → class → template (placeholders: `{statement}`,
    /// `{sources}`, `{answer}`, `{critique}`, `{answer_a}`, `{answer_b}`).
    pub workflows: BTreeMap<String, BTreeMap<String, String>>,
}

impl Prompts {
    pub fn load(dir: &Path) -> Result<Self, CorpusError> {
        let raw = std::fs::read_to_string(dir.join("prompts.json"))
            .map_err(|e| CorpusError::Io(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| CorpusError::Malformed(format!("prompts.json: {e}")))
    }

    pub fn digest(dir: &Path) -> Result<String, CorpusError> {
        let bytes =
            std::fs::read(dir.join("prompts.json")).map_err(|e| CorpusError::Io(e.to_string()))?;
        Ok(hex(Sha256::digest(&bytes)))
    }

    pub fn render(
        &self,
        workflow: &str,
        class: &str,
        vars: &[(&str, &str)],
    ) -> Result<String, CorpusError> {
        let template = self
            .workflows
            .get(workflow)
            .and_then(|by_class| by_class.get(class))
            .ok_or_else(|| {
                CorpusError::Malformed(format!("no `{workflow}` template for class `{class}`"))
            })?;
        let mut out = template.clone();
        for (key, value) in vars {
            out = out.replace(&format!("{{{key}}}"), value);
        }
        Ok(out)
    }
}

fn hex(digest: sha2::digest::Output<Sha256>) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// A typed corpus error.
#[derive(Debug)]
pub enum CorpusError {
    Io(String),
    Malformed(String),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorpusError::Io(detail) => write!(f, "corpus IO error: {detail}"),
            CorpusError::Malformed(detail) => write!(f, "corpus is malformed: {detail}"),
        }
    }
}

impl std::error::Error for CorpusError {}
