//! The compiler's determinism proofs (`.3.2`, ADR-033): the byte-identical
//! guarantee, the stable ordering, the escaping, the declared
//! unrepresentable, and the lock render.

use reasonbraid_policy_compiler::{compile, CompileRequest, InputClause, LockedPolicy};

fn clause(policy_id: &str, clause_id: &str, statement: &str) -> InputClause {
    InputClause {
        policy_id: policy_id.to_string(),
        policy_version: "1.0.0".to_string(),
        clause_id: clause_id.to_string(),
        statement: statement.to_string(),
    }
}

#[test]
fn the_same_inputs_compile_to_byte_identical_output() {
    let request = CompileRequest {
        target: "generic".to_string(),
        clauses: vec![
            clause("p-a", "c2", "the second clause"),
            clause("p-a", "c1", "the first clause"),
        ],
        lock: vec![],
    };
    let first = compile(&request).expect("compiles");
    let second = compile(&request).expect("compiles");
    assert_eq!(first.bytes, second.bytes, "the bytes repeat");
    assert_eq!(first.digest, second.digest, "the digest repeats");
}

#[test]
fn the_shuffled_inputs_render_in_the_stable_order() {
    let forward = compile(&CompileRequest {
        target: "generic".to_string(),
        clauses: vec![
            clause("p-a", "c1", "one"),
            clause("p-a", "c2", "two"),
            clause("p-b", "c1", "other"),
        ],
        lock: vec![],
    })
    .expect("compiles");
    let shuffled = compile(&CompileRequest {
        target: "generic".to_string(),
        clauses: vec![
            clause("p-b", "c1", "other"),
            clause("p-a", "c2", "two"),
            clause("p-a", "c1", "one"),
        ],
        lock: vec![],
    })
    .expect("compiles");
    assert_eq!(
        forward.bytes, shuffled.bytes,
        "the stable sort pins the order"
    );
    let lines: Vec<&str> = forward.bytes.lines().collect();
    assert_eq!(lines[1], "## c1 [p-a 1.0.0]");
    assert_eq!(lines[2], "one");
    assert_eq!(lines[3], "## c1 [p-b 1.0.0]");
}

#[test]
fn the_newlines_escape_and_the_control_characters_declare_themselves() {
    let artifact = compile(&CompileRequest {
        target: "generic".to_string(),
        clauses: vec![
            clause("p-a", "c1", "the first line\nthe second line"),
            clause("p-a", "c2", "a control \u{0001} character"),
        ],
        lock: vec![],
    })
    .expect("compiles");
    assert!(
        artifact.bytes.contains("the first line\\nthe second line"),
        "the newline escapes: {}",
        artifact.bytes
    );
    assert!(
        !artifact.bytes.contains("the first line\nthe second"),
        "no raw newline survives inside the statement"
    );
    assert!(
        !artifact.bytes.contains("a control"),
        "the control-char clause is NOT rendered"
    );
    assert_eq!(artifact.unrepresentable.len(), 1);
    assert_eq!(artifact.unrepresentable[0].clause_id, "c2");
    assert!(
        artifact.unrepresentable[0]
            .reason
            .contains("control character"),
        "the reason names the shape: {:?}",
        artifact.unrepresentable
    );
}

#[test]
fn the_lock_renders_the_stable_rows() {
    let artifact = compile(&CompileRequest {
        target: "lock".to_string(),
        clauses: vec![],
        lock: vec![
            LockedPolicy {
                policy_id: "p-b".to_string(),
                version: "1.0.0".to_string(),
                digest: "sha256:bb".to_string(),
                owning_authority: "grt-b".to_string(),
            },
            LockedPolicy {
                policy_id: "p-a".to_string(),
                version: "1.1.0".to_string(),
                digest: "sha256:aa".to_string(),
                owning_authority: "grt-a".to_string(),
            },
        ],
    })
    .expect("compiles");
    let lines: Vec<&str> = artifact.bytes.lines().collect();
    assert_eq!(lines[1], "p-a 1.1.0 sha256:aa grt-a");
    assert_eq!(lines[2], "p-b 1.0.0 sha256:bb grt-b");
}

#[test]
fn an_unknown_target_refuses() {
    let error = compile(&CompileRequest {
        target: "mcp".to_string(),
        clauses: vec![],
        lock: vec![],
    })
    .expect_err("the unknown target refuses");
    assert!(error.to_string().contains("vocabulary"), "{error}");
}

#[test]
fn the_codex_and_claude_projections_render_their_harness_shapes() {
    let clauses = vec![
        clause("p-a", "c1", "the objective clause"),
        clause("p-a", "c2", "the authority clause"),
    ];
    let codex = compile(&CompileRequest {
        target: "codex".to_string(),
        clauses: clauses.clone(),
        lock: vec![],
    })
    .expect("compiles");
    assert!(
        codex
            .bytes
            .contains("- `c1` [p-a 1.0.0]: the objective clause"),
        "the codex backticks the ids: {}",
        codex.bytes
    );
    let claude = compile(&CompileRequest {
        target: "claude".to_string(),
        clauses: clauses.clone(),
        lock: vec![],
    })
    .expect("compiles");
    assert!(
        claude
            .bytes
            .contains("- c1 [p-a 1.0.0]: the objective clause"),
        "the claude bundle uses the plain ids: {}",
        claude.bytes
    );
    // The byte-identical guarantee rides every target.
    let again = compile(&CompileRequest {
        target: "codex".to_string(),
        clauses,
        lock: vec![],
    })
    .expect("compiles");
    assert_eq!(codex.bytes, again.bytes, "the codex bytes repeat");
}

#[test]
fn the_oversized_statement_declares_itself_not_truncates() {
    let long = "x".repeat(9000);
    let artifact = compile(&CompileRequest {
        target: "codex".to_string(),
        clauses: vec![clause("p-a", "c1", &long)],
        lock: vec![],
    })
    .expect("compiles");
    assert!(
        !artifact.bytes.contains("xxx"),
        "the oversized statement is NOT rendered"
    );
    assert_eq!(artifact.unrepresentable.len(), 1);
    assert!(
        artifact.unrepresentable[0].reason.contains("exceeds"),
        "the reason names the limit: {:?}",
        artifact.unrepresentable
    );
}

#[test]
fn the_backticks_escape_in_the_codex_bundle() {
    let artifact = compile(&CompileRequest {
        target: "codex".to_string(),
        clauses: vec![clause("p-a", "c1", "a `backtick` in the statement")],
        lock: vec![],
    })
    .expect("compiles");
    assert!(
        artifact.bytes.contains("a \\`backtick\\` in the statement"),
        "the backticks escape: {}",
        artifact.bytes
    );
}
