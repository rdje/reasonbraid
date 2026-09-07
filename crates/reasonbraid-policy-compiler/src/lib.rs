//! The deterministic policy-projection compiler (`PHASE-6.3.2`, ADR-033).
//!
//! A hermetic pure function from the RESOLVED clause set to the target
//! bundles: no database, no network, no clock, no ambient state — the same
//! semantic inputs + compiler + profile + target parameters produce
//! BYTE-IDENTICAL output (the stable sort + the fixed templates). The
//! compiler renders, never re-resolves (the ADR-017 trap: a re-resolving
//! compiler would be a second judge). A clause that cannot ride a target is
//! a DECLARED unrepresentable — never a silent omission. The artifact's
//! ADR-011 digest is the §15.7 publication's verification primitive.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The target vocabulary (the initial §15.5 set — the MCP/host/checklist
/// targets are named deferrals; a target without an adapter is the typed
/// refusal).
pub const TARGETS: [&str; 2] = ["generic", "lock"];

/// One resolved clause: the winning policy/version + the clause + the path
/// (the server's `.1.3` resolution output, projected onto the compiler's
/// input shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputClause {
    pub policy_id: String,
    pub policy_version: String,
    pub clause_id: String,
    pub statement: String,
}

/// One locked policy (the `policy.lock`'s row): the version + the digest +
/// the owning authority (the resolution facts the lock records).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPolicy {
    pub policy_id: String,
    pub version: String,
    pub digest: String,
    pub owning_authority: String,
}

/// The compile request: the target + the resolved clauses + the lock rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompileRequest {
    pub target: String,
    pub clauses: Vec<InputClause>,
    #[serde(default)]
    pub lock: Vec<LockedPolicy>,
}

/// One declared unrepresentable: the clause that cannot ride the target +
/// the reason. The projection says so — it never silently drops it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unrepresentable {
    pub clause_id: String,
    pub policy_id: String,
    pub reason: String,
}

/// The compiled artifact: the rendered bytes + the ADR-011 digest + the
/// unrepresentable list (the operator decides on a non-empty list — the
/// compiler refuses to pretend).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledArtifact {
    pub target: String,
    pub digest: String,
    pub bytes: String,
    pub unrepresentable: Vec<Unrepresentable>,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    UnknownTarget(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnknownTarget(t) => {
                write!(
                    f,
                    "target `{t}` is not in the vocabulary ({})",
                    TARGETS.join(", ")
                )
            }
        }
    }
}

/// The ADR-011 digest over the rendered bytes (the projection's pin).
pub fn digest_sha256_hex(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Whether a statement can ride a line-based bundle: the printable text
/// (the `\n` escapes; the control characters below 0x20 cannot).
fn is_representable(statement: &str) -> bool {
    statement
        .bytes()
        .all(|b| b == b'\n' || b == b'\t' || (0x20..=0x7e).contains(&b) || b >= 0x80)
}

/// The stable escape: the newline + the tab become the two-char literals.
fn escape(statement: &str) -> String {
    statement
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// The generic bundle renderer: the header + the stable-ordered clauses.
fn render_generic(clauses: &[InputClause]) -> String {
    let mut out = String::from("# Policy bundle (deterministic projection)\n");
    for clause in clauses {
        out.push_str(&format!(
            "## {clause_id} [{policy} {version}]\n{statement}\n",
            clause_id = clause.clause_id,
            policy = clause.policy_id,
            version = clause.policy_version,
            statement = escape(&clause.statement),
        ));
    }
    out
}

/// The policy.lock renderer: the stable-ordered rows.
fn render_lock(lock: &[LockedPolicy]) -> String {
    let mut out = String::from("# policy.lock (deterministic projection)\n");
    for policy in lock {
        out.push_str(&format!(
            "{policy_id} {version} {digest} {authority}\n",
            policy_id = policy.policy_id,
            version = policy.version,
            digest = policy.digest,
            authority = policy.owning_authority,
        ));
    }
    out
}

/// Compile one projection (the pure function). The inputs are STABLY SORTED
/// first (the clause id, then the policy id — the order is part of the
/// determinism, never the caller's luck).
pub fn compile(request: &CompileRequest) -> Result<CompiledArtifact, CompileError> {
    if !TARGETS.contains(&request.target.as_str()) {
        return Err(CompileError::UnknownTarget(request.target.clone()));
    }
    let mut clauses = request.clauses.clone();
    clauses.sort_by(|a, b| {
        a.clause_id
            .cmp(&b.clause_id)
            .then_with(|| a.policy_id.cmp(&b.policy_id))
    });
    let mut lock = request.lock.clone();
    lock.sort_by(|a, b| {
        a.policy_id
            .cmp(&b.policy_id)
            .then_with(|| a.version.cmp(&b.version))
    });

    let mut unrepresentable = Vec::new();
    let representable: Vec<InputClause> = clauses
        .iter()
        .filter(|clause| {
            if is_representable(&clause.statement) {
                true
            } else {
                unrepresentable.push(Unrepresentable {
                    clause_id: clause.clause_id.clone(),
                    policy_id: clause.policy_id.clone(),
                    reason: "the statement carries a control character a line-based bundle cannot express"
                        .to_string(),
                });
                false
            }
        })
        .cloned()
        .collect();

    let body = match request.target.as_str() {
        "generic" => render_generic(&representable),
        "lock" => render_lock(&lock),
        _ => unreachable!("the vocabulary check holds above"),
    };
    let bytes = body.into_bytes();
    Ok(CompiledArtifact {
        target: request.target.clone(),
        digest: digest_sha256_hex(&bytes),
        bytes: String::from_utf8(bytes).expect("the renderer emits text"),
        unrepresentable,
    })
}
