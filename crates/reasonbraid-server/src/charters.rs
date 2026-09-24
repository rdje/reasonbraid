//! The governance charter's decision rules (`SIGNOFF-REPAIR.11.4.7.2.1.2.1`;
//! ROADMAP §4.1 + §13.3).
//!
//! §4.1: *Each tenant has a versioned `GovernanceCharter` defining … allowed
//! decision rules and approval thresholds.* Before this module the charter had
//! a NAME and no CONTENT — `enrollment_boundaries.charter_digest` was a TEXT
//! label nothing resolved (`dev-charter-digest`, `fixture-charter`) — and
//! §13.3's seven rule families existed only as prose.
//!
//! ⭐ **The store is CONTENT-ADDRESSED, and that is what makes it versioned.**
//! A row is keyed by the digest of its own canonical content, so changing the
//! allowed set writes a DIFFERENT row and the old one stays readable forever.
//! `reasonbraid_core::authority`'s decision digest already folds
//! `charter_digest` into every authorization record, so a decision taken under
//! Monday's charter stays interpretable after Tuesday's change — with no
//! history rewritten and no `superseded_at` bookkeeping to get wrong.
//!
//! ⛔ **The digest is the SERVER's product, never the caller's.** This is the
//! `publications::stage` rule (`SIGNOFF-REPAIR.9.2.1.3.1`) applied to a second
//! content-addressed store: a request that asserts a digest is refused unless
//! it equals the one registration derives.
//!
//! ⚠️ **Thresholds parameterize families; they are not families.** §13.3's
//! second family is *simple **or** supermajority of a defined electorate* — one
//! family whose bar is a number — which is exactly why §4.1 names *allowed
//! decision rules **and** approval thresholds* in one breath. Splitting it into
//! two wire names would be this module choosing a vocabulary the roadmap did
//! not.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// ROADMAP §13.3's seven supported rule families, one variant each.
///
/// ⛔ Seven, not nine. *Simple or supermajority* is ONE family parameterized by
/// a threshold, and *role-weighted or chambered* is one family whose weighting
/// §13.3 says the charter defines. Inventing extra wire names would be a
/// vocabulary this module chose rather than one the roadmap states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRule {
    /// *owner decides after consultation*
    OwnerDecides,
    /// *simple or supermajority of a defined electorate* — the charter's
    /// threshold says which, and this is the only family that requires one.
    MajorityOfElectorate,
    /// *unanimity of all non-recused electorate members, with explicit
    /// abstention semantics*
    Unanimity,
    /// *consensus with no unresolved blocking objection*
    Consensus,
    /// *role-weighted or chambered approval defined by a governance charter*
    RoleWeighted,
    /// *human committee approval*
    HumanCommittee,
    /// *advisory synthesis with no binding decision*
    AdvisorySynthesis,
}

impl DecisionRule {
    pub const ALL: [DecisionRule; 7] = [
        DecisionRule::OwnerDecides,
        DecisionRule::MajorityOfElectorate,
        DecisionRule::Unanimity,
        DecisionRule::Consensus,
        DecisionRule::RoleWeighted,
        DecisionRule::HumanCommittee,
        DecisionRule::AdvisorySynthesis,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OwnerDecides => "owner_decides",
            Self::MajorityOfElectorate => "majority_of_electorate",
            Self::Unanimity => "unanimity",
            Self::Consensus => "consensus",
            Self::RoleWeighted => "role_weighted",
            Self::HumanCommittee => "human_committee",
            Self::AdvisorySynthesis => "advisory_synthesis",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == value)
    }

    /// Whether this family's bar is a number the charter must supply.
    ///
    /// ⛔ Only [`Self::MajorityOfElectorate`]. A threshold on `unanimity` is
    /// not a stricter rule, it is a contradiction — unanimity IS 1.0 — and one
    /// on `advisory_synthesis` would parameterize a family §13.3 defines as
    /// having *no binding decision*.
    pub const fn requires_threshold(self) -> bool {
        matches!(self, Self::MajorityOfElectorate)
    }
}

/// What a caller submits to register a charter version.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharterInput {
    pub tenant_id: String,
    pub allowed_decision_rules: Vec<String>,
    /// rule wire-name → the fraction of the electorate that carries a decision.
    #[serde(default)]
    pub approval_thresholds: BTreeMap<String, f64>,
    /// ⛔ NOT an input in the sense of being trusted: supplying it asserts what
    /// registration will derive, and a disagreement is refused.
    #[serde(default)]
    pub charter_digest: Option<String>,
    /// The site act's audit reason.
    pub reason: crate::site_authority::Reason,
}

/// The stored charter.
#[derive(Debug, Clone, Serialize)]
pub struct StoredCharter {
    pub charter_digest: String,
    pub tenant_id: String,
    pub allowed_decision_rules: Vec<DecisionRule>,
    pub approval_thresholds: BTreeMap<String, f64>,
}

/// Every way a charter registration or lookup is refused.
#[derive(Debug, Clone, PartialEq)]
pub enum CharterError {
    /// A rule name that is not one of §13.3's seven.
    UnknownRule(String),
    /// The same rule twice in the allowed set.
    DuplicateRule(String),
    /// An empty allowed set: a charter that permits no decision rule cannot
    /// govern a decision, and storing one would make the refusal below
    /// unfalsifiable.
    NoRules,
    /// A family that needs a threshold and was given none.
    ThresholdMissing(DecisionRule),
    /// A threshold on a family that takes none.
    ThresholdNotApplicable(DecisionRule),
    /// A threshold for a rule this charter does not allow.
    ThresholdForUnallowedRule(String),
    /// A threshold outside `(0.5, 1.0]`.
    ThresholdOutOfRange(String, f64),
    /// The caller asserted a digest that is not the one registration derives.
    DigestMismatch { asserted: String, derived: String },
    /// No charter is stored under this digest.
    Unknown(String),
    /// The tenant has no active enrollment boundary, so no charter to resolve.
    NoBoundary(String),
    /// The tenant's boundary names a charter digest nothing resolves.
    ///
    /// ⛔ This FAILS CLOSED and says so: every boundary shipped before
    /// `0084` carries a label digest, so the honest answer for one of those is
    /// *this tenant's charter is not readable*, never *the rule is allowed*.
    BoundaryCharterUnknown { tenant_id: String, digest: String },
    /// The rule is a real §13.3 family and this tenant's charter does not
    /// allow it.
    ///
    /// ⛔ It names the RULE and never the tenant's set — a refusal that
    /// enumerated the charter would answer a question the caller did not ask.
    NotAllowed(String),
    /// The store failed: the server's fault, never the caller's.
    Storage(String),
    /// The caller's input holds a character the store cannot represent
    /// (U+0000): the caller's, and permanent (`SIGNOFF-REPAIR.4.4.10.1.2`).
    /// Classified where the `sqlx::Error` still carries its SQLSTATE, because
    /// `Storage` keeps only the text.
    UnrepresentableInput,
}

impl std::fmt::Display for CharterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRule(r) => write!(
                f,
                "`{r}` is not one of ROADMAP §13.3's seven decision rules ({})",
                DecisionRule::ALL
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::DuplicateRule(r) => write!(f, "the decision rule `{r}` is named twice"),
            Self::NoRules => write!(
                f,
                "a charter must allow at least one decision rule — one that allows none \
                 cannot govern a decision"
            ),
            Self::ThresholdMissing(r) => write!(
                f,
                "`{}` is the one §13.3 family whose bar is a number (*simple or \
                 supermajority*), so this charter must state its approval threshold",
                r.as_str()
            ),
            Self::ThresholdNotApplicable(r) => write!(
                f,
                "`{}` takes no approval threshold — its bar is defined by the family itself",
                r.as_str()
            ),
            Self::ThresholdForUnallowedRule(r) => write!(
                f,
                "the approval threshold names `{r}`, which this charter does not allow"
            ),
            Self::ThresholdOutOfRange(r, v) => write!(
                f,
                "the approval threshold for `{r}` is {v}, and a majority of an electorate \
                 lies in (0.5, 1.0]"
            ),
            Self::DigestMismatch { asserted, derived } => write!(
                f,
                "the asserted charter digest `{asserted}` is not this charter's — its \
                 digest is `{derived}` (the server canonicalizes and hashes the charter; \
                 a request never supplies one)"
            ),
            Self::Unknown(d) => write!(f, "no charter is registered under digest `{d}`"),
            Self::NoBoundary(t) => write!(
                f,
                "tenant `{t}` has no active enrollment boundary, so it has no charter"
            ),
            Self::BoundaryCharterUnknown { tenant_id, digest } => write!(
                f,
                "tenant `{tenant_id}`'s boundary names charter digest `{digest}`, which \
                 no registered charter matches — the question cannot be answered, and a \
                 rule is never allowed by default"
            ),
            Self::NotAllowed(r) => write!(f, "this tenant's charter does not allow `{r}`"),
            Self::Storage(e) => write!(f, "the charter store failed: {e}"),
            Self::UnrepresentableInput => {
                write!(f, "the input holds a character the store cannot represent")
            }
        }
    }
}

/// The canonical charter document (the bytes the digest is taken over), as ONE
/// definition.
///
/// ⛔ Both the digest registration stores and the content a later reader
/// compares are this function's output. The rules are sorted and the thresholds
/// ride a `BTreeMap`, so two submissions that differ only in ORDER are the same
/// charter and land on the same row.
pub fn canonical(
    tenant_id: &str,
    rules: &[DecisionRule],
    thresholds: &BTreeMap<String, f64>,
) -> String {
    let mut sorted = rules.to_vec();
    sorted.sort();
    serde_json::to_string(&json!({
        "tenant_id": tenant_id,
        "allowed_decision_rules": sorted.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        "approval_thresholds": thresholds,
    }))
    .expect("the charter serializes")
}

/// `sha256:<hex>` over the canonical charter (the ADR-011 shape).
pub fn digest(canonical: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Validate a submission against §13.3's vocabulary and §4.1's threshold rule.
///
/// Runs BEFORE any store access and before the site gate, on the
/// `SIGNOFF-REPAIR.8.2.5.2` precedent: a caller that fails validation learns
/// nothing about authority, and one that passes it still meets the gate.
pub fn validate(
    input: &CharterInput,
) -> Result<(Vec<DecisionRule>, BTreeMap<String, f64>), CharterError> {
    let mut rules: Vec<DecisionRule> = Vec::new();
    for name in &input.allowed_decision_rules {
        let rule =
            DecisionRule::parse(name).ok_or_else(|| CharterError::UnknownRule(name.clone()))?;
        if rules.contains(&rule) {
            return Err(CharterError::DuplicateRule(name.clone()));
        }
        rules.push(rule);
    }
    if rules.is_empty() {
        return Err(CharterError::NoRules);
    }
    for (name, value) in &input.approval_thresholds {
        let rule =
            DecisionRule::parse(name).ok_or_else(|| CharterError::UnknownRule(name.clone()))?;
        if !rules.contains(&rule) {
            return Err(CharterError::ThresholdForUnallowedRule(name.clone()));
        }
        if !rule.requires_threshold() {
            return Err(CharterError::ThresholdNotApplicable(rule));
        }
        if !(*value > 0.5 && *value <= 1.0) {
            return Err(CharterError::ThresholdOutOfRange(name.clone(), *value));
        }
    }
    for rule in &rules {
        if rule.requires_threshold() && !input.approval_thresholds.contains_key(rule.as_str()) {
            return Err(CharterError::ThresholdMissing(*rule));
        }
    }
    Ok((rules, input.approval_thresholds.clone()))
}

/// Register a charter version. Idempotent over byte-identical content: the
/// row's identity IS its content, so a second write asserts nothing new.
pub async fn register(
    conn: impl sqlx::PgExecutor<'_>,
    input: &CharterInput,
) -> Result<StoredCharter, CharterError> {
    let (rules, thresholds) = validate(input)?;
    let canonical_doc = canonical(&input.tenant_id, &rules, &thresholds);
    let derived = digest(&canonical_doc);
    if let Some(asserted) = &input.charter_digest {
        if asserted != &derived {
            return Err(CharterError::DigestMismatch {
                asserted: asserted.clone(),
                derived,
            });
        }
    }
    let mut sorted = rules.clone();
    sorted.sort();
    let rule_names: Vec<&str> = sorted.iter().map(|r| r.as_str()).collect();
    sqlx::query(
        "INSERT INTO governance_charters \
           (charter_digest, tenant_id, allowed_decision_rules, approval_thresholds) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (charter_digest) DO NOTHING",
    )
    .bind(&derived)
    .bind(&input.tenant_id)
    .bind(serde_json::to_value(&rule_names).expect("the rules serialize"))
    .bind(serde_json::to_value(&thresholds).expect("the thresholds serialize"))
    .execute(conn)
    .await
    .map_err(storage)?;
    Ok(StoredCharter {
        charter_digest: derived,
        tenant_id: input.tenant_id.clone(),
        allowed_decision_rules: sorted,
        approval_thresholds: thresholds,
    })
}

fn row_to_charter(
    charter_digest: String,
    tenant_id: String,
    rules: Value,
    thresholds: Value,
) -> StoredCharter {
    let allowed_decision_rules: Vec<DecisionRule> = serde_json::from_value::<Vec<String>>(rules)
        .expect("the rules parse")
        .into_iter()
        .filter_map(|r| DecisionRule::parse(&r))
        .collect();
    StoredCharter {
        charter_digest,
        tenant_id,
        allowed_decision_rules,
        approval_thresholds: serde_json::from_value(thresholds).expect("the thresholds parse"),
    }
}

/// Read one charter by its digest.
pub async fn load(pool: &PgPool, charter_digest: &str) -> Result<StoredCharter, CharterError> {
    let mut conn = acquire(pool).await?;
    load_on(&mut conn, charter_digest).await
}

/// A store error, classified while its SQLSTATE is still readable
/// (`SIGNOFF-REPAIR.4.4.10.1.2`).
fn storage(e: sqlx::Error) -> CharterError {
    if crate::api::unrepresentable_input(&e) {
        CharterError::UnrepresentableInput
    } else {
        CharterError::Storage(e.to_string())
    }
}

async fn acquire(
    pool: &PgPool,
) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, CharterError> {
    pool.acquire().await.map_err(storage)
}

async fn load_on(
    conn: &mut sqlx::PgConnection,
    charter_digest: &str,
) -> Result<StoredCharter, CharterError> {
    let row: Option<(String, String, Value, Value)> = sqlx::query_as(
        "SELECT charter_digest, tenant_id, allowed_decision_rules, approval_thresholds \
         FROM governance_charters WHERE charter_digest = $1",
    )
    .bind(charter_digest)
    .fetch_optional(&mut *conn)
    .await
    .map_err(storage)?;
    let (d, t, r, th) = row.ok_or_else(|| CharterError::Unknown(charter_digest.to_owned()))?;
    Ok(row_to_charter(d, t, r, th))
}

/// Resolve the charter a tenant is governed by, through its ACTIVE enrollment
/// boundary.
///
/// ⛔ The boundary is the resolution path rather than `tenant_id` directly, and
/// that is the whole point: §4.4 makes the boundary the root/parent-granted
/// ceiling, so the charter a tenant is bound by is the one its ISSUER named —
/// not the newest one anybody registered for that tenant id.
pub async fn for_tenant(pool: &PgPool, tenant_id: &str) -> Result<StoredCharter, CharterError> {
    let mut conn = acquire(pool).await?;
    for_tenant_on(&mut conn, tenant_id).await
}

async fn for_tenant_on(
    conn: &mut sqlx::PgConnection,
    tenant_id: &str,
) -> Result<StoredCharter, CharterError> {
    let named: Option<(String,)> = sqlx::query_as(
        "SELECT charter_digest FROM enrollment_boundaries \
         WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(storage)?;
    let digest = named
        .map(|(d,)| d)
        .ok_or_else(|| CharterError::NoBoundary(tenant_id.to_owned()))?;
    match load_on(conn, &digest).await {
        Ok(charter) => Ok(charter),
        Err(CharterError::Unknown(_)) => Err(CharterError::BoundaryCharterUnknown {
            tenant_id: tenant_id.to_owned(),
            digest,
        }),
        Err(other) => Err(other),
    }
}

/// The §4.1 read path: *may this tenant decide under this rule?*
///
/// Returns the family's threshold when it has one, so a caller never has to
/// ask twice.
pub async fn allows(
    pool: &PgPool,
    tenant_id: &str,
    rule: &str,
) -> Result<(DecisionRule, Option<f64>), CharterError> {
    let parsed =
        DecisionRule::parse(rule).ok_or_else(|| CharterError::UnknownRule(rule.to_owned()))?;
    let mut conn = acquire(pool).await?;
    let granted = allows_on(&mut conn, tenant_id, parsed).await?;
    Ok((parsed, granted.threshold))
}

/// What a tenant's charter says about one rule it allows: the family's
/// threshold, and the digest of the charter that said so.
#[derive(Debug, Clone, PartialEq)]
pub struct Allowed {
    pub threshold: Option<f64>,
    /// The content address of the charter the answer came from. A thread
    /// records it, so what it was created under stays readable after the
    /// tenant's charter changes (decision 1 of the charter record).
    pub charter_digest: String,
}

/// [`allows`] on a caller's connection — the thread-create path asks INSIDE
/// its command transaction, after authorization, so a caller who may not
/// create in a tenant learns nothing about that tenant's charter.
pub async fn allows_on(
    conn: &mut sqlx::PgConnection,
    tenant_id: &str,
    rule: DecisionRule,
) -> Result<Allowed, CharterError> {
    let charter = for_tenant_on(conn, tenant_id).await?;
    if !charter.allowed_decision_rules.contains(&rule) {
        return Err(CharterError::NotAllowed(rule.as_str().to_owned()));
    }
    Ok(Allowed {
        threshold: charter.approval_thresholds.get(rule.as_str()).copied(),
        charter_digest: charter.charter_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(rules: &[&str], thresholds: &[(&str, f64)]) -> CharterInput {
        CharterInput {
            tenant_id: "t-1".into(),
            allowed_decision_rules: rules.iter().map(|r| (*r).to_owned()).collect(),
            approval_thresholds: thresholds
                .iter()
                .map(|(k, v)| ((*k).to_owned(), *v))
                .collect(),
            charter_digest: None,
            reason: crate::site_authority::Reason::new("a test charter").unwrap(),
        }
    }

    #[test]
    fn the_seven_families_round_trip_through_their_wire_names() {
        assert_eq!(DecisionRule::ALL.len(), 7);
        for rule in DecisionRule::ALL {
            assert_eq!(DecisionRule::parse(rule.as_str()), Some(rule));
        }
        assert_eq!(DecisionRule::parse("supermajority"), None);
    }

    #[test]
    fn only_the_majority_family_carries_a_threshold() {
        let carrying: Vec<&str> = DecisionRule::ALL
            .into_iter()
            .filter(|r| r.requires_threshold())
            .map(|r| r.as_str())
            .collect();
        assert_eq!(carrying, vec!["majority_of_electorate"]);
    }

    #[test]
    fn the_digest_is_order_independent() {
        let a = input(&["consensus", "unanimity"], &[]);
        let b = input(&["unanimity", "consensus"], &[]);
        let (ra, ta) = validate(&a).unwrap();
        let (rb, tb) = validate(&b).unwrap();
        assert_eq!(
            digest(&canonical("t-1", &ra, &ta)),
            digest(&canonical("t-1", &rb, &tb)),
            "two submissions differing only in order are one charter"
        );
    }

    #[test]
    fn a_different_tenant_is_a_different_charter() {
        let (rules, thresholds) = validate(&input(&["consensus"], &[])).unwrap();
        assert_ne!(
            digest(&canonical("t-1", &rules, &thresholds)),
            digest(&canonical("t-2", &rules, &thresholds)),
            "a charter is a tenant's document, not a shared template"
        );
    }

    #[test]
    fn a_changed_set_is_a_different_digest_and_leaves_the_old_one_addressable() {
        let (r1, t1) = validate(&input(&["consensus"], &[])).unwrap();
        let (r2, t2) = validate(&input(&["consensus", "unanimity"], &[])).unwrap();
        let before = digest(&canonical("t-1", &r1, &t1));
        let after = digest(&canonical("t-1", &r2, &t2));
        assert_ne!(before, after);
        assert_eq!(
            before,
            digest(&canonical("t-1", &r1, &t1)),
            "the earlier charter still hashes to what a past decision recorded"
        );
    }

    #[test]
    fn an_unknown_rule_names_itself() {
        let err = validate(&input(&["benevolent_dictator"], &[])).unwrap_err();
        assert_eq!(err, CharterError::UnknownRule("benevolent_dictator".into()));
        assert!(err.to_string().contains("benevolent_dictator"));
    }

    #[test]
    fn the_majority_family_without_a_threshold_is_refused() {
        assert_eq!(
            validate(&input(&["majority_of_electorate"], &[])).unwrap_err(),
            CharterError::ThresholdMissing(DecisionRule::MajorityOfElectorate)
        );
    }

    #[test]
    fn a_threshold_on_a_family_that_takes_none_is_refused() {
        assert_eq!(
            validate(&input(&["unanimity"], &[("unanimity", 0.9)])).unwrap_err(),
            CharterError::ThresholdNotApplicable(DecisionRule::Unanimity),
            "unanimity is 1.0 by definition; a threshold on it is a contradiction"
        );
    }

    #[test]
    fn a_threshold_for_an_unallowed_rule_is_refused() {
        assert_eq!(
            validate(&input(&["consensus"], &[("majority_of_electorate", 0.67)])).unwrap_err(),
            CharterError::ThresholdForUnallowedRule("majority_of_electorate".into())
        );
    }

    #[test]
    fn a_threshold_at_or_below_a_half_is_refused() {
        for bar in [0.5, 0.4, 0.0, 1.01] {
            assert_eq!(
                validate(&input(
                    &["majority_of_electorate"],
                    &[("majority_of_electorate", bar)]
                ))
                .unwrap_err(),
                CharterError::ThresholdOutOfRange("majority_of_electorate".into(), bar),
                "a majority of an electorate lies in (0.5, 1.0]"
            );
        }
        assert!(validate(&input(
            &["majority_of_electorate"],
            &[("majority_of_electorate", 1.0)]
        ))
        .is_ok());
    }

    #[test]
    fn an_empty_allowed_set_is_refused() {
        assert_eq!(
            validate(&input(&[], &[])).unwrap_err(),
            CharterError::NoRules
        );
    }

    #[test]
    fn a_repeated_rule_is_refused() {
        assert_eq!(
            validate(&input(&["consensus", "consensus"], &[])).unwrap_err(),
            CharterError::DuplicateRule("consensus".into())
        );
    }

    #[test]
    fn a_refusal_names_the_rule_and_never_the_charters_set() {
        let rendered = CharterError::NotAllowed("unanimity".into()).to_string();
        assert!(rendered.contains("unanimity"));
        for other in DecisionRule::ALL {
            if other != DecisionRule::Unanimity {
                assert!(
                    !rendered.contains(other.as_str()),
                    "a refusal that enumerated the charter would answer a question the \
                     caller did not ask — `{}` leaked",
                    other.as_str()
                );
            }
        }
    }
}
