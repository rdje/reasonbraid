//! The typed policy schema + the versioned registry (`PHASE-6.1.2`, ADR-019):
//! the policy is a versioned, digest-pinned DOCUMENT — the stable clause ids,
//! the applicability + the explicit non-applicability, the exception schema,
//! and the OWNERSHIP metadata (the owning authority is a GRANT reference — the
//! label grants nothing; an unresolvable owning authority is invalid at
//! registration). The registry pattern mirrors the workflow profiles
//! (ADR-016): the unknown version is the typed refusal, never a stored guess.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use sqlx::PgPool;

/// The lifecycle vocabulary (the closed set).
pub const LIFECYCLES: [&str; 6] = [
    "draft",
    "active",
    "suspended",
    "superseded",
    "deprecated",
    "retracted",
];

/// One normative statement: a STABLE clause id + the statement text. The id
/// is the resolution's + the provenance's anchor — it never changes between
/// versions of the same policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClauseStatement {
    pub id: String,
    pub statement: String,
}

/// The policy-version submission (`.1.2`, ADR-019): the §15.1 fields. The
/// server DERIVES the ADR-011 digest from the canonical document
/// ([`document_digest`], `SIGNOFF-REPAIR.9.1.3`); a declared `digest` is
/// optional and, when present, must be that one. The owning authority is a
/// grant id the REGISTRAR must hold (`SIGNOFF-REPAIR.9.1.2`).
///
/// ⚠️ `reason` is a WIRE field of the submission and not a column of the
/// document (`SIGNOFF-REPAIR.6.1.5.4`). Registering a policy is a site act, and
/// every site act is attributable: a caller who chooses what the whole site
/// reads as governance is exactly the kind of caller who must be able to explain
/// it afterwards. [`register`] therefore never reads it —
/// [`crate::site_authority::register_policy`] passes it to the authorization
/// record, which is where a reason belongs.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyVersionInput {
    pub reason: crate::site_authority::Reason,
    pub policy_id: String,
    pub version: String,
    #[serde(default)]
    pub digest: Option<String>,
    pub lifecycle: String,
    pub title: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub rationale: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub risk_class: String,
    pub owning_authority: String,
    pub clauses: Vec<ClauseStatement>,
    #[serde(default)]
    pub applicability: Vec<Value>,
    #[serde(default)]
    pub non_applicability: Vec<Value>,
    #[serde(default)]
    pub dependencies: Vec<Value>,
    #[serde(default)]
    pub conflicts: Vec<Value>,
    #[serde(default)]
    pub precedence_hints: Vec<Value>,
    #[serde(default)]
    pub exceptions: Vec<Value>,
    #[serde(default)]
    pub provenance: Vec<Value>,
}

/// The registered row, as stored.
///
/// ⭐ `digest_verified` is DERIVED on every read (`SIGNOFF-REPAIR.9.1.3`): does
/// the stored document still hash to the stored digest? A version registered
/// before the server derived digests carries whatever its registrar declared,
/// which nothing derived, so it reads `false` rather than being trusted.
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredPolicy {
    pub policy_id: String,
    pub version: String,
    pub digest: String,
    pub lifecycle: String,
    pub title: String,
    pub intent: String,
    pub rationale: String,
    pub domain: String,
    pub risk_class: String,
    pub owning_authority: String,
    pub clauses: Vec<ClauseStatement>,
    pub applicability: Vec<Value>,
    pub non_applicability: Vec<Value>,
    pub dependencies: Vec<Value>,
    pub conflicts: Vec<Value>,
    pub precedence_hints: Vec<Value>,
    pub exceptions: Vec<Value>,
    pub provenance: Vec<Value>,
    pub digest_verified: bool,
}

/// The typed refusal reasons — the caller maps them to an HTTP error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    MalformedDigest(String),
    DigestMismatch { declared: String, derived: String },
    MalformedVersion(String),
    UnknownLifecycle(String),
    DuplicateClause(String),
    GhostAuthority(String),
    Duplicate(String),
    EmptyClauses,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not the ADR-011 `sha256:<64 hex>` shape")
            }
            PolicyError::DigestMismatch { declared, derived } => write!(
                f,
                "the declared digest `{declared}` is not the document's digest `{derived}`: \
                 the server derives it from the canonical document"
            ),
            PolicyError::MalformedVersion(v) => {
                write!(
                    f,
                    "version `{v}` is not a semantic version (digits and dots, 1–3 parts)"
                )
            }
            PolicyError::UnknownLifecycle(l) => {
                write!(
                    f,
                    "lifecycle `{l}` is not in the vocabulary ({})",
                    LIFECYCLES.join(", ")
                )
            }
            PolicyError::DuplicateClause(id) => {
                write!(
                    f,
                    "clause id `{id}` repeats — the ids are STABLE anchors, never duplicated"
                )
            }
            PolicyError::GhostAuthority(grant) => {
                write!(f, "the owning authority `{grant}` is not an active grant — the label grants nothing")
            }
            PolicyError::Duplicate(what) => {
                write!(f, "{what} already exists — the record's identity is its content, register a new version")
            }
            PolicyError::EmptyClauses => {
                write!(f, "the policy carries at least one normative statement")
            }
        }
    }
}

fn is_sha256_hex(digest: &str) -> bool {
    digest
        .strip_prefix("sha256:")
        .map(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or(false)
}

/// The content a policy digest is taken over (`SIGNOFF-REPAIR.9.1.3`): every
/// field of the document except `lifecycle`, which is a STATUS rather than
/// content, and the wire-only `reason` and `digest`.
///
/// One borrowed view serves both the submission and a stored row, so the digest
/// registration stores and the digest a later read re-derives are one function's
/// output — the `charters::canonical` precedent.
struct DocumentContent<'a> {
    policy_id: &'a str,
    version: &'a str,
    title: &'a str,
    intent: &'a str,
    rationale: &'a str,
    domain: &'a str,
    risk_class: &'a str,
    owning_authority: &'a str,
    clauses: &'a [ClauseStatement],
    applicability: &'a [Value],
    non_applicability: &'a [Value],
    dependencies: &'a [Value],
    conflicts: &'a [Value],
    precedence_hints: &'a [Value],
    exceptions: &'a [Value],
    provenance: &'a [Value],
}

impl DocumentContent<'_> {
    fn digest(&self) -> String {
        let text = |value: &str| Value::String(value.to_string());
        let list = |values: &[Value]| Value::Array(values.to_vec());
        let mut document = serde_json::Map::new();
        document.insert("policy_id".into(), text(self.policy_id));
        document.insert("version".into(), text(self.version));
        document.insert("title".into(), text(self.title));
        document.insert("intent".into(), text(self.intent));
        document.insert("rationale".into(), text(self.rationale));
        document.insert("domain".into(), text(self.domain));
        document.insert("risk_class".into(), text(self.risk_class));
        document.insert("owning_authority".into(), text(self.owning_authority));
        document.insert(
            "clauses".into(),
            serde_json::to_value(self.clauses).expect("the clauses serialize"),
        );
        document.insert("applicability".into(), list(self.applicability));
        document.insert("non_applicability".into(), list(self.non_applicability));
        document.insert("dependencies".into(), list(self.dependencies));
        document.insert("conflicts".into(), list(self.conflicts));
        document.insert("precedence_hints".into(), list(self.precedence_hints));
        document.insert("exceptions".into(), list(self.exceptions));
        document.insert("provenance".into(), list(self.provenance));
        let mut canonical = String::new();
        write_canonical(&Value::Object(document), &mut canonical);
        format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
    }
}

/// Compact JSON with object keys sorted by byte order at EVERY depth and arrays
/// in the order given — the canonical form the book states.
///
/// ⛔ The keys are sorted HERE rather than left to `serde_json::Map`. The map
/// iterates in key order only while serde_json's `preserve_order` feature is
/// off; it is off today (`cargo tree -e features -i serde_json`), but a feature
/// any dependency enables would reach this crate through unification and would
/// silently change every digest. Sorting explicitly makes the digest a property
/// of this function alone.
///
/// ⭐ A stored row re-derives the digest registration computed: `sqlx` binds a
/// `Value` by serializing it with serde_json, `jsonb` keeps each number's text,
/// and a read parses that text back, so every leaf serializes identically on
/// both sides. The live control registers unsorted keys, nested objects, `1e2`,
/// `0.1` and an integer above 2^53 and requires the read to verify.
fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        leaf => out.push_str(&leaf.to_string()),
    }
}

/// The digest the server derives for a submission (`SIGNOFF-REPAIR.9.1.3`):
/// `sha256` over the canonical document.
pub fn document_digest(input: &PolicyVersionInput) -> String {
    DocumentContent {
        policy_id: &input.policy_id,
        version: &input.version,
        title: &input.title,
        intent: &input.intent,
        rationale: &input.rationale,
        domain: &input.domain,
        risk_class: &input.risk_class,
        owning_authority: &input.owning_authority,
        clauses: &input.clauses,
        applicability: &input.applicability,
        non_applicability: &input.non_applicability,
        dependencies: &input.dependencies,
        conflicts: &input.conflicts,
        precedence_hints: &input.precedence_hints,
        exceptions: &input.exceptions,
        provenance: &input.provenance,
    }
    .digest()
}

fn is_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    (1..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Validate the SUBMISSION — every rule that asks only about the document.
///
/// ⭐ Split out of [`register`] by `SIGNOFF-REPAIR.6.1.5.4` so the HTTP layer
/// can run it BEFORE the site-authority gate, which is the order
/// `SIGNOFF-REPAIR.7.1.2.1` established for `validate_steps` and stated its
/// reasons for: each rule here is over a PUBLISHED constant — the ADR-011
/// digest shape, the semantic-version shape, the [`LIFECYCLES`] vocabulary the
/// book enumerates, and clause ids that are stable anchors — so naming the one
/// that failed is an oracle over nothing, and every site route already bounds
/// its input on extraction.
///
/// ⛔ The two refusals that consult the DATABASE — a ghost owning authority and
/// a taken `(policy_id, version)` — are deliberately NOT here. Answering either
/// before the gate would hand a principal with no site authority an existence
/// oracle over the site's grants and over a registry it may not write.
///
/// [`register`] calls this regardless of who called it first.
pub fn validate(input: &PolicyVersionInput) -> Result<(), PolicyError> {
    // `SIGNOFF-REPAIR.9.1.3`: a declared digest is optional, and when present
    // it must be the document's own. Both rules are over the submission alone,
    // so they sit with the others before the site gate.
    if let Some(declared) = &input.digest {
        if !is_sha256_hex(declared) {
            return Err(PolicyError::MalformedDigest(declared.clone()));
        }
        let derived = document_digest(input);
        if *declared != derived {
            return Err(PolicyError::DigestMismatch {
                declared: declared.clone(),
                derived,
            });
        }
    }
    if !is_semver(&input.version) {
        return Err(PolicyError::MalformedVersion(input.version.clone()));
    }
    if !LIFECYCLES.contains(&input.lifecycle.as_str()) {
        return Err(PolicyError::UnknownLifecycle(input.lifecycle.clone()));
    }
    if input.clauses.is_empty() {
        return Err(PolicyError::EmptyClauses);
    }
    let mut seen = std::collections::BTreeSet::new();
    for clause in &input.clauses {
        if !seen.insert(clause.id.as_str()) {
            return Err(PolicyError::DuplicateClause(clause.id.clone()));
        }
    }
    Ok(())
}

/// Register one policy version (the typed, validated registry row).
///
/// ⚠️ Takes a CONNECTION rather than the pool (`SIGNOFF-REPAIR.6.1.5.4`), which
/// is what lets [`crate::site_authority::register_policy`] run the write, its
/// authorization and its audit record in ONE transaction — the same change
/// `SIGNOFF-REPAIR.7.1.2.1` made to [`crate::workflows::register`], and for the
/// same reason.
///
/// ⛔ And it returns the TWO-LEVEL result for the reason `authorized`'s own doc
/// comment gives: *a domain refusal is separate from a SQL error, and an
/// unavailable database must never masquerade as one*. That matters here in a
/// way it did not before, because this refusal is now an AUDIT RECORD: the
/// shipped code turned every failed INSERT into [`PolicyError::Duplicate`] and
/// every failed grant lookup into [`PolicyError::GhostAuthority`], so an outage
/// would have written *"that policy version is already registered"* into an
/// operator's trail — a statement about what happened that nothing established.
/// The inner `Err` is a refusal the caller made; the outer one is the database
/// failing to answer, and the act rolls back with nothing recorded.
pub async fn register(
    conn: &mut sqlx::PgConnection,
    input: &PolicyVersionInput,
    registrar: &reasonbraid_core::GrantSubject,
) -> Result<Result<RegisteredPolicy, PolicyError>, sqlx::Error> {
    if let Err(refusal) = validate(input) {
        return Ok(Err(refusal));
    }
    // The ownership = the authority binding (ADR-019): the owning authority
    // must be a LIVE grant — a label-only policy fails at registration.
    //
    // ⭐ `SIGNOFF-REPAIR.9.3.1` settled the asymmetry `.11.9.1.2.2` measured:
    // this site admitted on `status = 'active'` alone while `resolve` below,
    // in the same module, also required the grant to be unexpired — so a
    // lapsed grant registered a policy version the resolver would then refuse.
    // Both then asked `authority::grant_is_live`, which is also the first time
    // either consulted `valid_from`; `grant_held_by` below carries the same
    // liveness predicate.
    //
    // `.9.3.4.2` added COVERING `policy_version_register`.
    //
    // ⭐ `SIGNOFF-REPAIR.9.1.2` adds HELD BY THE REGISTRAR, the question both
    // earlier leaves routed here. Holding `policy_register` lets a principal
    // write the library; it never let them attach somebody else's authority to
    // what they wrote, and the publication verbs treat this grant as the
    // policy's owner. So the grant must be one the registrar holds, which is
    // the rule every other site that cites an authority already follows
    // (`authority::grant_held_by`, `.9.3.1`). A policy owned by someone else is
    // registered by that owner
    // (`docs/decisions/2026-09-25_a-policy-registrar-holds-the-authority-it-names.md`).
    if !crate::authority::grant_held_by(
        &mut *conn,
        &input.owning_authority,
        registrar,
        reasonbraid_core::GrantAction::PolicyVersionRegister,
    )
    .await?
    {
        return Ok(Err(PolicyError::GhostAuthority(
            input.owning_authority.clone(),
        )));
    }
    // ⛔ `ON CONFLICT DO NOTHING` rather than letting the unique violation
    // raise, and the reason is `SIGNOFF-REPAIR.6.1.5.4`'s alone: this INSERT now
    // runs inside the site act's transaction, and in PostgreSQL a failed
    // statement ABORTS that transaction — so the audit record `authorized`
    // writes next would itself fail with *current transaction is aborted* and
    // the caller would see a 500 instead of an audited refusal. The control
    // caught exactly that before this line existed. The constraint still
    // arbitrates: `rows_affected() == 0` is the coordinate already being taken,
    // and it is the only thing that produces zero.
    // `SIGNOFF-REPAIR.9.1.3`: the stored digest is the DERIVED one, whatever was
    // declared (`validate` above already refused a declaration that disagrees).
    let digest = document_digest(input);
    let inserted = sqlx::query(
        "INSERT INTO policy_versions \
         (policy_id, version, digest, lifecycle, title, intent, rationale, domain, risk_class, \
          owning_authority, clauses, applicability, non_applicability, dependencies, conflicts, \
          precedence_hints, exceptions, provenance) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18) \
         ON CONFLICT (policy_id, version) DO NOTHING",
    )
    .bind(&input.policy_id)
    .bind(&input.version)
    .bind(&digest)
    .bind(&input.lifecycle)
    .bind(&input.title)
    .bind(&input.intent)
    .bind(&input.rationale)
    .bind(&input.domain)
    .bind(&input.risk_class)
    .bind(&input.owning_authority)
    .bind(serde_json::to_value(&input.clauses).expect("the clauses serialize"))
    .bind(serde_json::to_value(&input.applicability).expect("the applicability serializes"))
    .bind(serde_json::to_value(&input.non_applicability).expect("the non-applicability serializes"))
    .bind(serde_json::to_value(&input.dependencies).expect("the dependencies serialize"))
    .bind(serde_json::to_value(&input.conflicts).expect("the conflicts serialize"))
    .bind(serde_json::to_value(&input.precedence_hints).expect("the precedence serializes"))
    .bind(serde_json::to_value(&input.exceptions).expect("the exceptions serialize"))
    .bind(serde_json::to_value(&input.provenance).expect("the provenance serializes"))
    .execute(&mut *conn)
    .await?;
    if inserted.rows_affected() == 0 {
        return Ok(Err(PolicyError::Duplicate(format!(
            "policy `{}` version {}",
            input.policy_id, input.version
        ))));
    }
    Ok(Ok(RegisteredPolicy {
        policy_id: input.policy_id.clone(),
        version: input.version.clone(),
        digest,
        lifecycle: input.lifecycle.clone(),
        title: input.title.clone(),
        intent: input.intent.clone(),
        rationale: input.rationale.clone(),
        domain: input.domain.clone(),
        risk_class: input.risk_class.clone(),
        owning_authority: input.owning_authority.clone(),
        clauses: input.clauses.clone(),
        applicability: input.applicability.clone(),
        non_applicability: input.non_applicability.clone(),
        dependencies: input.dependencies.clone(),
        conflicts: input.conflicts.clone(),
        precedence_hints: input.precedence_hints.clone(),
        exceptions: input.exceptions.clone(),
        provenance: input.provenance.clone(),
        digest_verified: true,
    }))
}

/// The stored policy row (a FromRow struct — the 18 columns exceed the
/// tuple impl's 16-column ceiling).
#[derive(Debug, sqlx::FromRow)]
struct PolicyRow {
    policy_id: String,
    version: String,
    digest: String,
    lifecycle: String,
    title: String,
    intent: String,
    rationale: String,
    domain: String,
    risk_class: String,
    owning_authority: String,
    clauses: Value,
    applicability: Value,
    non_applicability: Value,
    dependencies: Value,
    conflicts: Value,
    precedence_hints: Value,
    exceptions: Value,
    provenance: Value,
}

/// The registered policies, newest first.
pub async fn list(pool: &PgPool) -> Result<Vec<RegisteredPolicy>, sqlx::Error> {
    let rows: Vec<PolicyRow> = sqlx::query_as(
        "SELECT policy_id, version, digest, lifecycle, title, intent, rationale, domain, \
         risk_class, owning_authority, clauses, applicability, non_applicability, dependencies, \
         conflicts, precedence_hints, exceptions, provenance \
         FROM policy_versions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let mut policy = RegisteredPolicy {
                policy_id: row.policy_id,
                version: row.version,
                digest: row.digest,
                lifecycle: row.lifecycle,
                title: row.title,
                intent: row.intent,
                rationale: row.rationale,
                domain: row.domain,
                risk_class: row.risk_class,
                owning_authority: row.owning_authority,
                clauses: serde_json::from_value(row.clauses).expect("the clauses parse"),
                applicability: serde_json::from_value(row.applicability)
                    .expect("the applicability parses"),
                non_applicability: serde_json::from_value(row.non_applicability)
                    .expect("the non-applicability parses"),
                dependencies: serde_json::from_value(row.dependencies)
                    .expect("the dependencies parse"),
                conflicts: serde_json::from_value(row.conflicts).expect("the conflicts parse"),
                precedence_hints: serde_json::from_value(row.precedence_hints)
                    .expect("the precedence parses"),
                exceptions: serde_json::from_value(row.exceptions).expect("the exceptions parse"),
                provenance: serde_json::from_value(row.provenance).expect("the provenance parses"),
                digest_verified: false,
            };
            // `SIGNOFF-REPAIR.9.1.3`: re-derived from what is STORED, on every
            // read, so a row whose digest was declared rather than derived says
            // so instead of being trusted.
            policy.digest_verified = policy.content().digest() == policy.digest;
            policy
        })
        .collect())
}

impl RegisteredPolicy {
    /// The stored document as the digest sees it (`SIGNOFF-REPAIR.9.1.3`).
    fn content(&self) -> DocumentContent<'_> {
        DocumentContent {
            policy_id: &self.policy_id,
            version: &self.version,
            title: &self.title,
            intent: &self.intent,
            rationale: &self.rationale,
            domain: &self.domain,
            risk_class: &self.risk_class,
            owning_authority: &self.owning_authority,
            clauses: &self.clauses,
            applicability: &self.applicability,
            non_applicability: &self.non_applicability,
            dependencies: &self.dependencies,
            conflicts: &self.conflicts,
            precedence_hints: &self.precedence_hints,
            exceptions: &self.exceptions,
            provenance: &self.provenance,
        }
    }
}

// ── The layering + the precedence (`.1.3`, ADR-019) ────────────────────────────────

/// One policy reference in the resolution request (the id + the version).
///
/// ⚠️ `Serialize` since `SIGNOFF-REPAIR.9.2.1.3.2`: the same shape is what a
/// projection stores as its resolved set, and one type for one concept beats a
/// second struct that must be kept in step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRef {
    pub policy_id: String,
    pub version: String,
}

/// The resolution target: the layer + the target the clauses must apply to
/// (the §15.3 applicability filter's inputs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionTarget {
    pub layer: String,
    pub target: String,
}

/// The resolution request (`.1.3`): the collection to resolve + the target +
/// the requested waiver/exception references.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionRequest {
    pub policies: Vec<PolicyRef>,
    pub target: ResolutionTarget,
    #[serde(default)]
    pub exception_grants: Vec<String>,
}

/// One resolved clause: the winning policy/version, the clause, and the
/// explanation trail (the path the seven steps took to it).
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedClause {
    pub policy_id: String,
    pub version: String,
    pub clause_id: String,
    pub statement: String,
    pub path: Vec<String>,
}

/// The resolution result: the resolved clauses + the explanation tree. The
/// conflicts list is EMPTY on success — an unresolved binding conflict is
/// the typed refusal (the fail-closed step), never a silent pick.
#[derive(Debug, Clone, Serialize)]
pub struct Resolution {
    pub target: ResolutionTarget,
    pub resolved: Vec<ResolvedClause>,
    pub explanation: Vec<String>,
    pub conflicts: Vec<String>,
}

impl PolicyError {
    fn unknown_ref(policy_id: &str, version: &str) -> Self {
        PolicyError::Duplicate(format!(
            "policy `{policy_id}` version {version} is not registered"
        ))
    }
    fn expired_authority(policy_id: &str, grant: &str) -> Self {
        PolicyError::GhostAuthority(format!(
            "the owning authority `{grant}` of `{policy_id}` is not an active, unexpired grant"
        ))
    }
    fn missing_dependency(policy_id: &str, dependency: &str) -> Self {
        PolicyError::Duplicate(format!(
            "`{policy_id}` depends on `{dependency}`, which is not in the resolved set"
        ))
    }
    fn conflicting_pair(a: &str, b: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the precedence hints conflict: `{a}` over `{b}` AND `{b}` over `{a}`"
        ))
    }
    fn unknown_waiver(waiver: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the requested exception `{waiver}` is not allowed by any policy's exception schema"
        ))
    }
    fn binding_conflict(clause_id: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the unresolved binding conflict: clause `{clause_id}` is carried by multiple \
             applicable policies and no precedence settles it (fail-closed)"
        ))
    }
}

/// The stored resolution row (the id, the version, the lifecycle, the digest,
/// the owning authority, the clauses, the applicability, the
/// non-applicability, the dependencies, the conflicts, the precedence hints,
/// the exceptions).
#[derive(Debug, sqlx::FromRow)]
struct ResolutionRow {
    policy_id: String,
    version: String,
    lifecycle: String,
    #[allow(dead_code)]
    // the digest rides the registry rows; the resolution reads the facts it needs
    digest: String,
    owning_authority: String,
    clauses: Value,
    applicability: Value,
    non_applicability: Value,
    dependencies: Value,
    conflicts: Value,
    precedence_hints: Value,
    exceptions: Value,
}

/// The seven-step resolution (ADR-019's §15.3 pipeline, fail-closed).
pub async fn resolve(
    pool: &PgPool,
    request: &ResolutionRequest,
) -> Result<Resolution, PolicyError> {
    if request.policies.is_empty() {
        return Err(PolicyError::EmptyClauses);
    }
    let mut explanation = Vec::new();

    // Load the named policies; a ghost reference is the typed refusal.
    let mut loaded = Vec::new();
    for reference in &request.policies {
        let row: Option<ResolutionRow> = sqlx::query_as(
            "SELECT policy_id, version, lifecycle, digest, owning_authority, clauses, \
             applicability, non_applicability, dependencies, conflicts, precedence_hints, \
             exceptions \
             FROM policy_versions WHERE policy_id = $1 AND version = $2",
        )
        .bind(&reference.policy_id)
        .bind(&reference.version)
        .fetch_optional(pool)
        .await
        .map_err(|_| PolicyError::unknown_ref(&reference.policy_id, &reference.version))?;
        let row =
            row.ok_or_else(|| PolicyError::unknown_ref(&reference.policy_id, &reference.version))?;
        loaded.push(row);
    }
    let ids: Vec<&str> = loaded.iter().map(|r| r.policy_id.as_str()).collect();
    explanation.push(format!(
        "loaded {} policies: {}",
        loaded.len(),
        ids.join(", ")
    ));

    // Step 1: the issuer authority — each owning grant must be ACTIVE and
    // unexpired (the label grants nothing; an expired grant grants nothing).
    for row in &loaded {
        // ⛔ `None` — the ONE site entitled to ask about liveness alone
        // (`.9.3.4.2`). This is not a caller citing an authority: it asks of
        // every LOADED policy's owner whether that authority still stands, so
        // there is no verb being attempted and no verb to cover. Passing an
        // action here would refuse a policy whose owner is perfectly valid.
        let valid = crate::authority::grant_is_live(pool, &row.owning_authority, None)
            .await
            .map_err(|_| PolicyError::expired_authority(&row.policy_id, &row.owning_authority))?;
        if !valid {
            return Err(PolicyError::expired_authority(
                &row.policy_id,
                &row.owning_authority,
            ));
        }
    }
    explanation.push("step 1: every owning authority is an active, unexpired grant".to_string());

    // Step 2: the applicability filter — a policy applies when SOME
    // applicability selector matches the target AND NO non-applicability
    // selector matches. The empty applicability matches everything (the
    // baseline policy).
    let selector_matches = |selector: &Value| -> bool {
        let layer = selector
            .get("layer")
            .and_then(|v| v.as_str())
            .unwrap_or("*");
        let target = selector
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("*");
        (layer == "*" || layer == request.target.layer)
            && (target == "*" || target == request.target.target)
    };
    let selectors: Vec<Vec<Value>> = loaded
        .iter()
        .map(|row| {
            serde_json::from_value::<Vec<Value>>(row.applicability.clone())
                .expect("the applicability parses")
        })
        .collect();
    let non_selectors: Vec<Vec<Value>> = loaded
        .iter()
        .map(|row| {
            serde_json::from_value::<Vec<Value>>(row.non_applicability.clone())
                .expect("the non-applicability parses")
        })
        .collect();
    let applicable: Vec<bool> = loaded
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let positive = selectors[i].is_empty() || selectors[i].iter().any(selector_matches);
            let negative = non_selectors[i].iter().any(selector_matches);
            positive && !negative && row.lifecycle != "suspended" && row.lifecycle != "retracted"
        })
        .collect();
    explanation.push(format!(
        "step 2: the applicability filter left {} of {} policies applying",
        applicable.iter().filter(|a| **a).count(),
        loaded.len()
    ));

    // Step 3: the dependencies + the explicit conflicts — every dependency
    // must be IN the set; every explicit conflict must be OUT.
    for (i, row) in loaded.iter().enumerate() {
        let dependencies: Vec<Value> =
            serde_json::from_value(row.dependencies.clone()).expect("the dependencies parse");
        for dependency in &dependencies {
            let policy = dependency
                .get("policy")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !ids.contains(&policy) {
                return Err(PolicyError::missing_dependency(&row.policy_id, policy));
            }
        }
        let conflicts: Vec<Value> =
            serde_json::from_value(row.conflicts.clone()).expect("the conflicts parse");
        for conflict in &conflicts {
            let policy = conflict
                .get("policy")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if ids.contains(&policy) {
                return Err(PolicyError::binding_conflict(&format!(
                    "{} conflicts with {}",
                    row.policy_id, policy
                )));
            }
        }
        let _ = i;
    }
    explanation.push(
        "step 3: every dependency is in the set; no explicit conflict is present".to_string(),
    );

    // Step 4: the precedence hints — the `over` edges; a cycle is the
    // refusal (the charter precedence must be a DAG).
    let mut precedence: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in &loaded {
        let hints: Vec<Value> =
            serde_json::from_value(row.precedence_hints.clone()).expect("the hints parse");
        for hint in &hints {
            let over = hint
                .get("over")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if over == row.policy_id {
                continue; // a self-hint is a no-op, not a conflict
            }
            precedence
                .entry(row.policy_id.clone())
                .or_default()
                .push(over.to_string());
            if precedence
                .get(over)
                .map(|list| list.contains(&row.policy_id))
                .unwrap_or(false)
            {
                return Err(PolicyError::conflicting_pair(&row.policy_id, over));
            }
        }
    }
    // The transitive winner for a pair: A wins over B when A can reach B
    // through the `over` edges.
    let wins_over = |a: &str, b: &str| -> bool {
        let mut stack = vec![a.to_string()];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(current) = stack.pop() {
            if current == b {
                return true;
            }
            if !seen.insert(current.clone()) {
                continue;
            }
            if let Some(next) = precedence.get(&current) {
                stack.extend(next.iter().cloned());
            }
        }
        false
    };
    explanation.push(format!(
        "step 4: the precedence edges form a DAG ({} edges)",
        precedence.values().map(|v| v.len()).sum::<usize>()
    ));

    // Step 5: the exceptions/waivers — each requested exception must be
    // allowed by SOME policy's exception schema.
    for waiver in &request.exception_grants {
        let allowed = loaded.iter().any(|row| {
            let exceptions: Vec<Value> =
                serde_json::from_value(row.exceptions.clone()).expect("the exceptions parse");
            exceptions
                .iter()
                .any(|e| e.get("waiver").and_then(|v| v.as_str()) == Some(waiver.as_str()))
        });
        if !allowed {
            return Err(PolicyError::unknown_waiver(waiver));
        }
    }
    explanation
        .push("step 5: every requested exception rides a policy's exception schema".to_string());

    // Step 6: the clause-id collisions across the APPLICABLE policies — the
    // precedence settles them; the unresolved binding conflict FAILS CLOSED.
    let mut by_clause: std::collections::BTreeMap<String, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, row) in loaded.iter().enumerate() {
        if !applicable[i] {
            continue;
        }
        let clauses: Vec<ClauseStatement> =
            serde_json::from_value(row.clauses.clone()).expect("the clauses parse");
        for clause in &clauses {
            by_clause.entry(clause.id.clone()).or_default().push(i);
        }
    }
    let mut resolved = Vec::new();
    let mut conflicts = Vec::new();
    for (clause_id, carriers) in &by_clause {
        let winner = if carriers.len() == 1 {
            carriers[0]
        } else {
            // The candidate that wins over ALL the others transitively.
            let candidates: Vec<usize> = carriers
                .iter()
                .copied()
                .filter(|&candidate| {
                    carriers.iter().copied().all(|other| {
                        other == candidate
                            || wins_over(&loaded[candidate].policy_id, &loaded[other].policy_id)
                    })
                })
                .collect();
            if candidates.len() != 1 {
                conflicts.push(clause_id.clone());
                continue;
            }
            candidates[0]
        };
        let clauses: Vec<ClauseStatement> =
            serde_json::from_value(loaded[winner].clauses.clone()).expect("the clauses parse");
        let clause = clauses
            .iter()
            .find(|c| c.id == *clause_id)
            .expect("the clause is present");
        resolved.push(ResolvedClause {
            policy_id: loaded[winner].policy_id.clone(),
            version: loaded[winner].version.clone(),
            clause_id: clause.id.clone(),
            statement: clause.statement.clone(),
            path: vec![
                "authority: active grant".to_string(),
                "applicability: matched".to_string(),
                "precedence: the winner over the carriers".to_string(),
            ],
        });
    }
    if !conflicts.is_empty() {
        return Err(PolicyError::binding_conflict(&conflicts.join(", ")));
    }
    explanation.push(format!(
        "step 6: {} clauses resolved with no unresolved binding conflict",
        resolved.len()
    ));

    // Step 7: the explanation tree rides the result.
    Ok(Resolution {
        target: ResolutionTarget {
            layer: request.target.layer.clone(),
            target: request.target.target.clone(),
        },
        resolved,
        explanation,
        conflicts,
    })
}

/// The impact map (`.1.3`): the derivable coverage — the clauses × the
/// applicability selectors the policy DECLARES (never an achievement claim).
pub async fn impact(
    pool: &PgPool,
    policy_id: &str,
    version: &str,
) -> Result<Vec<Value>, PolicyError> {
    let row: Option<ResolutionRow> = sqlx::query_as(
        "SELECT policy_id, version, lifecycle, digest, owning_authority, clauses, \
         applicability, non_applicability, dependencies, conflicts, precedence_hints, \
         exceptions \
         FROM policy_versions WHERE policy_id = $1 AND version = $2",
    )
    .bind(policy_id)
    .bind(version)
    .fetch_optional(pool)
    .await
    .map_err(|_| PolicyError::unknown_ref(policy_id, version))?;
    let row = row.ok_or_else(|| PolicyError::unknown_ref(policy_id, version))?;
    let clauses: Vec<ClauseStatement> =
        serde_json::from_value(row.clauses).expect("the clauses parse");
    Ok(clauses
        .into_iter()
        .map(|clause| {
            serde_json::json!({
                "clause_id": clause.id,
                "statement": clause.statement,
                "applicability": row.applicability,
                "non_applicability": row.non_applicability,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn submission(extra: Value) -> PolicyVersionInput {
        let mut body = json!({
            "reason": "the unit control registers a policy version",
            "policy_id": "unit-policy",
            "version": "1.0.0",
            "lifecycle": "active",
            "title": "the unit policy",
            "owning_authority": "grt_unit",
            "clauses": [ { "id": "c1", "statement": "every thread declares its objective" } ],
        });
        for (key, value) in extra.as_object().expect("an object").iter() {
            body[key] = value.clone();
        }
        serde_json::from_value(body).expect("the submission parses")
    }

    /// `SIGNOFF-REPAIR.9.1.3`: keys sorted at every depth, arrays in order,
    /// compact. The expected text is written out by hand.
    #[test]
    fn the_canonical_form_sorts_keys_at_every_depth_and_keeps_array_order() {
        let value = json!({ "b": [ { "z": 1, "a": 2 }, 3 ], "a": { "d": "x", "c": null } });
        let mut out = String::new();
        write_canonical(&value, &mut out);
        assert_eq!(out, r#"{"a":{"c":null,"d":"x"},"b":[{"a":2,"z":1},3]}"#);
    }

    /// The digest covers content and nothing else: `lifecycle` is a status, and
    /// the wire-only `reason` and `digest` are not the document.
    #[test]
    fn the_digest_covers_the_content_and_not_the_status() {
        let base = document_digest(&submission(json!({})));
        assert!(base.starts_with("sha256:") && base.len() == 71, "{base}");
        let draft = document_digest(&submission(json!({ "lifecycle": "draft" })));
        assert_eq!(base, draft, "the lifecycle is a status, not content");
        let reasoned = document_digest(&submission(json!({ "reason": "another reason" })));
        assert_eq!(base, reasoned, "the reason is not content");
        let changed = document_digest(&submission(json!({
            "clauses": [ { "id": "c1", "statement": "a different statement" } ],
        })));
        assert_ne!(base, changed, "a changed clause changes the digest");
        let versioned = document_digest(&submission(json!({ "version": "1.0.1" })));
        assert_ne!(base, versioned, "the coordinate is content");
    }

    /// A declared digest is optional; when present it must be the derived one.
    #[test]
    fn a_declared_digest_must_be_the_documents_own() {
        assert_eq!(validate(&submission(json!({}))), Ok(()));
        let derived = document_digest(&submission(json!({})));
        assert_eq!(
            validate(&submission(json!({ "digest": derived.clone() }))),
            Ok(())
        );
        let forged = format!("sha256:{}", "a".repeat(64));
        assert_eq!(
            validate(&submission(json!({ "digest": forged.clone() }))),
            Err(PolicyError::DigestMismatch {
                declared: forged,
                derived
            })
        );
        assert_eq!(
            validate(&submission(json!({ "digest": "sha256:nothex" }))),
            Err(PolicyError::MalformedDigest("sha256:nothex".into()))
        );
    }
}
