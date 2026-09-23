//! The stage-1 eligibility evaluation (`PHASE-3.3.1`, backlog 29's policy-filter
//! half): the initiator's typed expression over the §10.3 stage-1 fields,
//! resolved against a candidate's shipped facts. PURE functions only — an
//! ineligible role is never restored by ranking (ADR-014's ordering), and the
//! checks run against the profile AS VISIBLE AT THE EXPRESSION'S SCOPE (a
//! tenant-hidden capability cannot satisfy a network-scope requirement).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::presence::PresenceState;
use crate::profiles::{
    filter_profile, AgentProfile, CapabilityClaim, ClaimConfidence, ReaderClass,
};

/// The initiator's eligibility expression (§10.3 stage 1). Unknown fields are
/// typed rejections; every field defaults to the least restrictive shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct EligibilityExpression {
    /// The reader scope the evaluation runs at: every check reads the
    /// candidate's profile filtered at this scope (a hidden field satisfies
    /// nothing).
    pub scope: ReaderClass,
    /// Capability requirements: each with the minimum provenance.
    pub capabilities: Vec<CapabilityRequirement>,
    /// Topic/policy interests the candidate must declare.
    pub interests: Vec<String>,
    /// Confidentiality classes the candidate must be compatible with.
    pub confidentiality: Vec<String>,
    /// The minimum DECLARED concurrency (the availability gate).
    pub min_concurrency: Option<i64>,
    /// The minimum available budget (the hard-budget gate).
    pub min_budget: Option<f64>,
    /// The allowed presence states (default: `available` only).
    pub presence_states: Vec<String>,
    /// Explicit exclusions (role ids) — a named recusal refuses regardless.
    pub exclude: Vec<String>,
    /// The project/domain tags the stage-2 affinity feature matches against.
    pub domains: Vec<String>,
    /// The preferred cost/latency class (the stage-2 latency feature).
    pub preferred_latency: Option<String>,
}

impl Default for EligibilityExpression {
    fn default() -> Self {
        Self {
            scope: ReaderClass::Network,
            capabilities: Vec::new(),
            interests: Vec::new(),
            confidentiality: Vec::new(),
            min_concurrency: None,
            min_budget: None,
            presence_states: vec!["available".to_string()],
            exclude: Vec::new(),
            domains: Vec::new(),
            preferred_latency: None,
        }
    }
}

/// One capability requirement: the taxonomy id + the minimum provenance the
/// claim must carry (the §10.1 rule: a high self-declared score is never
/// equivalent to verified competence).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    pub taxonomy_id: String,
    pub min_confidence: ClaimConfidence,
}

/// The candidate's shipped facts (the `.3.3` surface loads them; the
/// evaluator stays pure).
#[derive(Debug, Clone, PartialEq)]
pub struct EligibilityCandidate {
    pub role_id: String,
    pub profile: Option<AgentProfile>,
    pub presence_state: PresenceState,
    pub concurrency: Option<i64>,
    pub available_budget: Option<f64>,
}

/// The verdict: the decision + the named reasons (every stage-1 field that
/// decided, either way — the explanation substrate the `.3.2` features ride).
#[derive(Debug, Clone, PartialEq)]
pub struct EligibilityVerdict {
    pub eligible: bool,
    pub reasons: Vec<String>,
}

fn confidence_rank(c: ClaimConfidence) -> u8 {
    match c {
        ClaimConfidence::SelfAsserted => 0,
        ClaimConfidence::OwnerAttested => 1,
        ClaimConfidence::Benchmarked => 2,
        ClaimConfidence::Certified => 3,
    }
}

/// A claim is LIVE at `at` while it carries no expiry or its expiry is still
/// ahead — the half-open rule the grants use (`expires_at > now()` is live), so
/// the repository has one meaning of *expired* (`SIGNOFF-REPAIR.5.1.2`).
fn claim_live(claim: &CapabilityClaim, at: DateTime<Utc>) -> bool {
    claim.expires_at.is_none_or(|expires_at| expires_at > at)
}

/// The deterministic stage-1 evaluation at the instant `at`. Each refusal names
/// its field; an eligible verdict carries the positive reasons too.
///
/// `at` is the caller's evaluation instant, not a clock read here: the function
/// stays pure, and one request judges every candidate at the same instant.
pub fn eligible(
    expression: &EligibilityExpression,
    candidate: &EligibilityCandidate,
    at: DateTime<Utc>,
) -> EligibilityVerdict {
    let mut reasons: Vec<String> = Vec::new();

    // 1. The explicit exclusion: a named recusal refuses regardless.
    if expression.exclude.contains(&candidate.role_id) {
        return EligibilityVerdict {
            eligible: false,
            reasons: vec!["explicitly excluded by the expression".to_string()],
        };
    }

    // 2. The presence gate (the enrollment/suspension/availability fact).
    if !expression
        .presence_states
        .contains(&candidate.presence_state.as_str().to_string())
    {
        return EligibilityVerdict {
            eligible: false,
            reasons: vec![format!(
                "presence state `{}` is not among the allowed states ({})",
                candidate.presence_state.as_str(),
                expression.presence_states.join(", ")
            )],
        };
    }

    // 3. The visibility-scoped profile: every further check reads ONLY the
    // fields visible at the expression's scope.
    let Some(profile) = &candidate.profile else {
        return EligibilityVerdict {
            eligible: false,
            reasons: vec!["no profile is declared".to_string()],
        };
    };
    let visible = filter_profile(profile, expression.scope);
    let visible_obj = visible.as_object().expect("the filter returns an object");
    if visible_obj.is_empty() {
        return EligibilityVerdict {
            eligible: false,
            reasons: vec!["the profile exposes nothing at the requested scope".to_string()],
        };
    }

    // 4. The capability requirements: each claim must be DECLARED, VISIBLE at
    // the expression's scope, LIVE at `at`, and carry at least the required
    // provenance — checked in that order, so a refusal never names the expiry
    // of a claim the reader could not see. An expired claim is a declaration
    // the profile still shows, never a qualification (§10.1: capabilities carry
    // their expiry); until `SIGNOFF-REPAIR.5.1.2` nothing read `expires_at`,
    // and a lapsed attestation satisfied a requirement for ever.
    for requirement in &expression.capabilities {
        let id = requirement.taxonomy_id.as_str();
        let declared: Vec<&CapabilityClaim> = profile
            .capabilities
            .iter()
            .filter(|c| c.taxonomy_id == id)
            .collect();
        if declared.is_empty() {
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!("capability `{id}` is not declared")],
            };
        }
        let claim_visible = visible_obj
            .get("capabilities")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| {
                arr.iter()
                    .any(|c| c.get("taxonomy_id").and_then(|t| t.as_str()) == Some(id))
            });
        if !claim_visible {
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!(
                    "capability `{id}` is not visible at the requested scope"
                )],
            };
        }
        // The strongest LIVE claim decides; a profile that declares one id twice
        // is judged on its best current evidence, never on list order.
        let Some(claim) = declared
            .iter()
            .filter(|c| claim_live(c, at))
            .max_by_key(|c| confidence_rank(c.confidence))
        else {
            let expired_at = declared
                .iter()
                .filter_map(|c| c.expires_at)
                .max()
                .expect("a declared claim that is not live carries an expiry");
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!(
                    "capability `{id}` expired at {} (evaluated at {})",
                    expired_at.to_rfc3339(),
                    at.to_rfc3339()
                )],
            };
        };
        if confidence_rank(claim.confidence) < confidence_rank(requirement.min_confidence) {
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!(
                    "capability `{id}` is {} but the requirement needs {}",
                    claim.confidence.rank_name(),
                    requirement.min_confidence.rank_name()
                )],
            };
        }
        reasons.push(format!(
            "capability `{id}` holds ({} ≥ {})",
            claim.confidence.rank_name(),
            requirement.min_confidence.rank_name()
        ));
    }

    // 5. The interest/topic restrictions.
    for interest in &expression.interests {
        let visible_interests = visible_obj
            .get("interests")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let declared_and_visible = visible_interests
            .iter()
            .any(|i| i.as_str() == Some(interest.as_str()));
        if declared_and_visible {
            reasons.push(format!("interest `{interest}` matches"));
        } else {
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!(
                    "interest `{interest}` is not declared (or not visible at the requested scope)"
                )],
            };
        }
    }

    // 6. The confidentiality compatibility.
    for class in &expression.confidentiality {
        let visible_classes = visible_obj
            .get("confidentiality_classes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if visible_classes
            .iter()
            .any(|c| c.as_str() == Some(class.as_str()))
        {
            reasons.push(format!("confidentiality class `{class}` is compatible"));
        } else {
            return EligibilityVerdict {
                eligible: false,
                reasons: vec![format!(
                    "confidentiality class `{class}` is not declared (or not visible at the requested scope)"
                )],
            };
        }
    }

    // 7. The declared-concurrency gate.
    if let Some(min) = expression.min_concurrency {
        if min > 0 {
            match candidate.concurrency {
                Some(declared) if declared >= min => {
                    reasons.push(format!("declared concurrency {declared} meets {min}"));
                }
                _ => {
                    return EligibilityVerdict {
                        eligible: false,
                        reasons: vec![format!(
                            "declared concurrency does not meet the requirement of {min}"
                        )],
                    };
                }
            }
        }
    }

    // 8. The hard-budget gate.
    if let Some(min) = expression.min_budget {
        if min > 0.0 {
            match candidate.available_budget {
                Some(available) if available >= min => {
                    reasons.push(format!("available budget {available} meets {min}"));
                }
                Some(available) => {
                    return EligibilityVerdict {
                        eligible: false,
                        reasons: vec![format!(
                            "available budget {available} does not meet the requirement of {min}"
                        )],
                    };
                }
                None => {
                    return EligibilityVerdict {
                        eligible: false,
                        reasons: vec![format!(
                            "available budget is UNKNOWN (the requirement of {min} cannot be proven)"
                        )],
                    };
                }
            }
        }
    }

    EligibilityVerdict {
        eligible: true,
        reasons,
    }
}

/// The stage-2 feature weights (the initiator's preferences). Zero-weight
/// features contribute nothing; the ranking is the weighted sum.
///
/// ⛔ Each weight is a finite number in `[0, 1]` ([`Self::validate`],
/// `SIGNOFF-REPAIR.5.1.4`). The ranking is ordinal, so scaling every weight by
/// one factor keeps the order and every ratio stays expressible inside the
/// interval; outside it a negative weight inverts a feature, and weights near
/// `f64::MAX` overflow the total to `inf`, where two candidates compare equal.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct RankingPreferences {
    pub capability: f64,
    pub interest: f64,
    pub affinity: f64,
    pub latency: f64,
    pub balance: f64,
    /// The diversity weight (`.6.2`): the selection seeks the VARIATION among
    /// the dependence attributes — a spread panel scores higher.
    pub diversity: f64,
}

impl Default for RankingPreferences {
    fn default() -> Self {
        Self {
            capability: 1.0,
            interest: 1.0,
            affinity: 1.0,
            latency: 1.0,
            balance: 1.0,
            diversity: 1.0,
        }
    }
}

impl RankingPreferences {
    /// The first weight outside `[0, 1]`, named as the request spells it; a
    /// NaN or an infinity is outside too. The weights are checked in
    /// declaration order, so the refusal is deterministic.
    pub fn validate(&self) -> Result<(), String> {
        let weights = [
            ("capability", self.capability),
            ("interest", self.interest),
            ("affinity", self.affinity),
            ("latency", self.latency),
            ("balance", self.balance),
            ("diversity", self.diversity),
        ];
        match weights
            .into_iter()
            .find(|(_, weight)| !(0.0..=1.0).contains(weight))
        {
            Some((name, _)) => Err(format!(
                "the ranking weight `{name}` must be a finite number from 0 to 1"
            )),
            None => Ok(()),
        }
    }
}

/// The diversity score and its explanation (`.6.2`, `SIGNOFF-REPAIR.5.1.3`):
/// the mean over the five dependence attributes of what each one shows.
///
/// An attribute counts only when it is KNOWN on both sides: the candidate
/// declares it and at least one other candidate declares the same attribute.
/// It then scores `1 −` the share of those declarers holding the candidate's
/// value — compared attribute to attribute, so a provider named `x` never
/// matches a harness named `x`. Every other attribute scores 0: unknown
/// contributes nothing. Dividing by all five, not by the known ones, is what
/// keeps silence from paying — a fact left undeclared scores what a fact
/// shared with everyone scores, never more, so declaring less cannot rank
/// higher. ⛔ Until this repair an absent fact read as variation: a candidate
/// with none scored the maximum, `1.0`.
fn diversity(
    mine: Option<&crate::dependence::MemberFacts>,
    others: &[&crate::dependence::MemberFacts],
) -> (f64, String) {
    let attributes = crate::dependence::ATTRIBUTES;
    let mut sum = 0.0f64;
    let mut compared: Vec<String> = Vec::new();
    if let Some(mine) = mine {
        for (attribute, pick) in attributes {
            let Some(value) = pick(mine) else { continue };
            let declarers: Vec<&str> = others.iter().filter_map(|o| pick(o)).collect();
            if declarers.is_empty() {
                continue;
            }
            let sharers = declarers.iter().filter(|v| **v == value).count();
            sum += 1.0 - sharers as f64 / declarers.len() as f64;
            compared.push(if sharers == 0 {
                format!(
                    "its {attribute} is shared by none of the {}",
                    declarers.len()
                )
            } else {
                format!(
                    "its {attribute} is shared by {sharers} of the {} ({:.0}%)",
                    declarers.len(),
                    sharers as f64 / declarers.len() as f64 * 100.0
                )
            });
        }
    }
    let total = attributes.len();
    if compared.is_empty() {
        return (
            0.0,
            format!(
                "none of the {total} dependence attributes is known for both this candidate and \
                 another (unknown contributes nothing)"
            ),
        );
    }
    let mut explanation = format!(
        "compared on {} of the {total} dependence attributes, each against the other \
         candidates declaring it: {}",
        compared.len(),
        compared.join("; ")
    );
    if compared.len() < total {
        explanation.push_str(&format!(
            "; the other {} contribute nothing",
            total - compared.len()
        ));
    }
    (sum / total as f64, explanation)
}

/// The ranking order: the higher total first, ties by role id. `total_cmp`
/// keeps it a total order whatever the totals are; the match surface merges
/// its per-scope groups by this same key.
pub fn by_rank(a: &RankedCandidate, b: &RankedCandidate) -> std::cmp::Ordering {
    b.total
        .total_cmp(&a.total)
        .then_with(|| a.role_id.cmp(&b.role_id))
}

/// One feature score with its visibility-safe explanation: the explanation
/// names ONLY the initiator's own inputs and the matched facts that are
/// VISIBLE at the expression's scope — never a hidden profile field.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FeatureScore {
    pub feature: &'static str,
    pub score: f64,
    pub contribution: f64,
    pub explanation: String,
}

/// The ranked candidate: the stage-1 reasons + the feature scores + the total.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RankedCandidate {
    pub role_id: String,
    pub stage1_reasons: Vec<String>,
    pub features: Vec<FeatureScore>,
    pub total: f64,
}

/// The deterministic stage-2 ranking: a pure function of the ELIGIBLE set (the
/// ranking never restores an ineligible role — the stage-1 verdict does). The
/// scores read the profile filtered at the expression's scope only. Ties break
/// by role id (determinism).
pub fn rank(
    expression: &EligibilityExpression,
    candidates: &[(EligibilityCandidate, EligibilityVerdict)],
    preferences: &RankingPreferences,
) -> Vec<RankedCandidate> {
    rank_with_dependence(expression, candidates, preferences, None)
}

/// The rank with the dependence facts (`.6.2`): the diversity feature scores
/// each candidate by how little its KNOWN dependence facts overlap the OTHER
/// eligible candidates' ([`diversity`]). No dependence facts → the feature
/// scores 0 (unknown contributes nothing, never a guess).
pub fn rank_with_dependence(
    expression: &EligibilityExpression,
    candidates: &[(EligibilityCandidate, EligibilityVerdict)],
    preferences: &RankingPreferences,
    dependence: Option<&std::collections::HashMap<String, crate::dependence::MemberFacts>>,
) -> Vec<RankedCandidate> {
    let mut ranked: Vec<RankedCandidate> = candidates
        .iter()
        .filter(|(_, verdict)| verdict.eligible)
        .map(|(candidate, verdict)| {
            let visible = candidate
                .profile
                .as_ref()
                .map(|p| crate::profiles::filter_profile(p, expression.scope))
                .unwrap_or_else(|| serde_json::json!({}));
            let visible_obj = visible.as_object().expect("the filter returns an object");

            let visible_caps: Vec<String> = visible_obj
                .get("capabilities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.get("taxonomy_id").and_then(|t| t.as_str()).map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let required: Vec<&str> = expression
                .capabilities
                .iter()
                .map(|r| r.taxonomy_id.as_str())
                .collect();
            let satisfied = required.iter().filter(|id| visible_caps.iter().any(|c| c == *id)).count();
            let capability_score = if required.is_empty() {
                0.0
            } else {
                satisfied as f64 / required.len() as f64
            };

            let visible_interests: Vec<&str> = visible_obj
                .get("interests")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|i| i.as_str()).collect())
                .unwrap_or_default();
            let matched_interests = expression
                .interests
                .iter()
                .filter(|i| visible_interests.contains(&i.as_str()))
                .count();
            let interest_score = if expression.interests.is_empty() {
                0.0
            } else {
                matched_interests as f64 / expression.interests.len() as f64
            };

            let visible_scopes: Vec<&str> = visible_obj
                .get("scopes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|i| i.as_str()).collect())
                .unwrap_or_default();
            let matched_domains = expression
                .domains
                .iter()
                .filter(|d| visible_scopes.contains(&d.as_str()))
                .count();
            let affinity_score = if expression.domains.is_empty() {
                0.0
            } else {
                matched_domains as f64 / expression.domains.len() as f64
            };

            let latency_score = match &expression.preferred_latency {
                Some(preferred)
                    if candidate
                        .profile
                        .as_ref()
                        .and_then(|p| p.cost_latency_class.as_ref())
                        == Some(preferred) =>
                {
                    1.0
                }
                Some(_) | None => 0.0,
            };

            let balance_score = match candidate.presence_state {
                crate::presence::PresenceState::Available => 1.0,
                crate::presence::PresenceState::Draining => 0.3,
                _ => 0.0,
            };

            // The diversity feature (`.6.2`), over KNOWN facts only
            // (`SIGNOFF-REPAIR.5.1.3`): see [`diversity`].
            let (diversity_score, diversity_explanation) = match dependence {
                None => (
                    0.0,
                    "no dependence facts are loaded (the feature contributes nothing)".to_string(),
                ),
                Some(facts) => {
                    let others: Vec<&crate::dependence::MemberFacts> = candidates
                        .iter()
                        .filter(|(_, v)| v.eligible)
                        .filter(|(c, _)| c.role_id != candidate.role_id)
                        .filter_map(|(c, _)| facts.get(&c.role_id))
                        .collect();
                    diversity(facts.get(&candidate.role_id), &others)
                }
            };

            let features = vec![
                FeatureScore {
                    feature: "capability_match",
                    score: capability_score,
                    contribution: preferences.capability * capability_score,
                    explanation: format!(
                        "{} of the {} required capabilities are declared (and visible at this scope)",
                        satisfied,
                        required.len()
                    ),
                },
                FeatureScore {
                    feature: "interest_match",
                    score: interest_score,
                    contribution: preferences.interest * interest_score,
                    explanation: format!(
                        "{} of the {} required interests are declared",
                        matched_interests,
                        expression.interests.len()
                    ),
                },
                FeatureScore {
                    feature: "domain_affinity",
                    score: affinity_score,
                    contribution: preferences.affinity * affinity_score,
                    explanation: format!(
                        "{} of the {} project/domain tags match the declared scopes",
                        matched_domains,
                        expression.domains.len()
                    ),
                },
                FeatureScore {
                    feature: "latency_class",
                    score: latency_score,
                    contribution: preferences.latency * latency_score,
                    explanation: match &expression.preferred_latency {
                        Some(p) => format!("the declared cost/latency class matches `{p}`"),
                        None => "no latency preference was expressed".to_string(),
                    },
                },
                FeatureScore {
                    feature: "workload_balance",
                    score: balance_score,
                    contribution: preferences.balance * balance_score,
                    explanation: format!(
                        "the presence state `{}` admits work",
                        candidate.presence_state.as_str()
                    ),
                },
                FeatureScore {
                    feature: "diversity",
                    score: diversity_score,
                    contribution: preferences.diversity * diversity_score,
                    explanation: diversity_explanation,
                },
            ];
            let total: f64 = features.iter().map(|f| f.contribution).sum();
            RankedCandidate {
                role_id: candidate.role_id.clone(),
                stage1_reasons: verdict.reasons.clone(),
                features,
                total,
            }
        })
        .collect();
    ranked.sort_by(by_rank);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{Availability, VisibilityClass};

    fn profile_with(
        caps: Vec<crate::profiles::CapabilityClaim>,
        interests: Vec<&str>,
    ) -> AgentProfile {
        AgentProfile {
            display_label: "candidate".to_string(),
            purpose: "probe".to_string(),
            conversation_modes: vec![],
            capabilities: caps,
            interests: interests.into_iter().map(|s| s.to_string()).collect(),
            languages: vec![],
            structured_output_formats: vec![],
            scopes: vec![],
            confidentiality_classes: vec!["internal".to_string()],
            availability: Some(Availability {
                operating_hours: None,
                concurrency: Some(2),
                wake_policy: None,
            }),
            resolver_tool_capabilities: vec![],
            cost_latency_class: None,
            resource_ceilings: None,
            visibility: crate::profiles::VisibilityPolicy::default(),
            grants_by_reference: vec![],
            incarnation_id: None,
        }
    }

    fn candidate(role: &str, profile: Option<AgentProfile>) -> EligibilityCandidate {
        EligibilityCandidate {
            role_id: role.to_string(),
            profile,
            presence_state: PresenceState::Available,
            concurrency: Some(2),
            available_budget: Some(100.0),
        }
    }

    /// The fixed evaluation instant every unit control judges at: the verdicts
    /// stay a pure function of their inputs, with no clock in the test.
    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-23T12:00:00Z")
            .expect("a valid instant")
            .with_timezone(&Utc)
    }

    fn requirement(id: &str, min: ClaimConfidence) -> CapabilityRequirement {
        CapabilityRequirement {
            taxonomy_id: id.to_string(),
            min_confidence: min,
        }
    }

    /// The provenance gate: a self-asserted claim fails a benchmarked
    /// requirement (the §10.1 rule in the evaluator).
    #[test]
    fn a_self_asserted_claim_fails_a_benchmarked_requirement() {
        let profile = profile_with(
            vec![crate::profiles::CapabilityClaim {
                taxonomy_id: "code_review".to_string(),
                confidence: ClaimConfidence::SelfAsserted,
                evidence_ref: None,
                expires_at: None,
            }],
            vec![],
        );
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::Benchmarked)],
            ..Default::default()
        };
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)), at());
        assert!(!verdict.eligible, "the provenance gate refuses");
        assert!(
            verdict.reasons[0].contains("benchmarked"),
            "the refusal names the requirement: {:?}",
            verdict.reasons
        );
    }

    /// The visibility-scoped checks: a tenant-hidden capability cannot satisfy
    /// a network-scope requirement — the check reads the filtered profile.
    #[test]
    fn a_tenant_hidden_capability_cannot_satisfy_a_network_requirement() {
        let mut profile = profile_with(
            vec![crate::profiles::CapabilityClaim {
                taxonomy_id: "code_review".to_string(),
                confidence: ClaimConfidence::OwnerAttested,
                evidence_ref: None,
                expires_at: None,
            }],
            vec![],
        );
        profile.visibility.capabilities = VisibilityClass::Tenant;
        let expression = EligibilityExpression {
            scope: ReaderClass::Network,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            ..Default::default()
        };
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)), at());
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("not visible"),
            "the refusal names the visibility: {:?}",
            verdict.reasons
        );
    }

    fn claim(confidence: ClaimConfidence, expires_at: Option<DateTime<Utc>>) -> CapabilityClaim {
        CapabilityClaim {
            taxonomy_id: "code_review".to_string(),
            confidence,
            evidence_ref: None,
            expires_at,
        }
    }

    /// The expiry gate (`SIGNOFF-REPAIR.5.1.2`, §10.1): a claim is live while
    /// its expiry is still AHEAD of the evaluation instant — the grants'
    /// half-open rule, so a claim expiring exactly at `at` is already expired.
    #[test]
    fn an_expired_claim_does_not_satisfy_a_requirement() {
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            ..Default::default()
        };
        let judged = |expires_at: Option<DateTime<Utc>>| {
            let profile = profile_with(
                vec![claim(ClaimConfidence::OwnerAttested, expires_at)],
                vec![],
            );
            eligible(&expression, &candidate("rol_a", Some(profile)), at())
        };

        let lapsed = judged(Some(at() - chrono::Duration::days(1)));
        assert!(!lapsed.eligible, "a lapsed claim refuses");
        assert_eq!(
            lapsed.reasons,
            vec!["capability `code_review` expired at 2026-09-22T12:00:00+00:00 (evaluated at 2026-09-23T12:00:00+00:00)".to_string()],
            "the refusal names the expiry and the instant it was judged at"
        );
        assert!(
            !judged(Some(at())).eligible,
            "expiring AT the instant is expired"
        );
        assert!(
            judged(Some(at() + chrono::Duration::seconds(1))).eligible,
            "a claim still ahead of its expiry holds"
        );
        assert!(judged(None).eligible, "a claim without an expiry holds");
    }

    /// The strongest LIVE claim decides: a lapsed attestation beside a live one
    /// is not a refusal, and a live self-assertion beside a lapsed attestation
    /// does not inherit the attestation's provenance.
    #[test]
    fn the_strongest_live_claim_decides() {
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            ..Default::default()
        };
        let lapsed = Some(at() - chrono::Duration::hours(1));
        let renewed = profile_with(
            vec![
                claim(ClaimConfidence::OwnerAttested, lapsed),
                claim(ClaimConfidence::OwnerAttested, None),
            ],
            vec![],
        );
        assert!(eligible(&expression, &candidate("rol_a", Some(renewed)), at()).eligible);

        let downgraded = profile_with(
            vec![
                claim(ClaimConfidence::OwnerAttested, lapsed),
                claim(ClaimConfidence::SelfAsserted, None),
            ],
            vec![],
        );
        let verdict = eligible(&expression, &candidate("rol_a", Some(downgraded)), at());
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0]
                .contains("is self_asserted but the requirement needs owner_attested"),
            "{:?}",
            verdict.reasons
        );
    }

    /// Visibility is checked BEFORE expiry, so a refusal never discloses when a
    /// claim the reader cannot see lapsed.
    #[test]
    fn a_hidden_lapsed_claim_is_refused_as_hidden_not_as_expired() {
        let mut profile = profile_with(
            vec![claim(
                ClaimConfidence::OwnerAttested,
                Some(at() - chrono::Duration::days(1)),
            )],
            vec![],
        );
        profile.visibility.capabilities = VisibilityClass::Tenant;
        let expression = EligibilityExpression {
            scope: ReaderClass::Network,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            ..Default::default()
        };
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)), at());
        assert_eq!(
            verdict.reasons,
            vec!["capability `code_review` is not visible at the requested scope".to_string()]
        );
    }

    /// The presence gate refuses an offline candidate.
    #[test]
    fn an_offline_candidate_refuses_the_default_expression() {
        let mut cand = candidate("rol_a", Some(profile_with(vec![], vec![])));
        cand.presence_state = PresenceState::Offline;
        let verdict = eligible(&EligibilityExpression::default(), &cand, at());
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("offline"),
            "{:?}",
            verdict.reasons
        );
    }

    /// The explicit exclusion refuses regardless of every other fact.
    #[test]
    fn an_excluded_role_refuses_regardless() {
        let expression = EligibilityExpression {
            exclude: vec!["rol_a".to_string()],
            ..EligibilityExpression::default()
        };
        let verdict = eligible(
            &expression,
            &candidate("rol_a", Some(profile_with(vec![], vec![]))),
            at(),
        );
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("excluded"),
            "{:?}",
            verdict.reasons
        );
    }

    /// The concurrency + budget gates refuse shortfalls.
    #[test]
    fn the_concurrency_and_budget_gates_refuse_shortfalls() {
        let mut expression = EligibilityExpression {
            min_concurrency: Some(4),
            ..Default::default()
        };
        let verdict = eligible(
            &expression,
            &candidate("rol_a", Some(profile_with(vec![], vec![]))),
            at(),
        );
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("concurrency"),
            "{:?}",
            verdict.reasons
        );

        expression.min_concurrency = None;
        expression.min_budget = Some(200.0);
        let verdict = eligible(
            &expression,
            &candidate("rol_a", Some(profile_with(vec![], vec![]))),
            at(),
        );
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("budget"),
            "{:?}",
            verdict.reasons
        );
    }

    /// The happy path: every gate passes and the verdict carries the reasons.
    #[test]
    fn an_eligible_candidate_carries_the_positive_reasons() {
        let mut profile = profile_with(
            vec![crate::profiles::CapabilityClaim {
                taxonomy_id: "code_review".to_string(),
                confidence: ClaimConfidence::OwnerAttested,
                evidence_ref: None,
                expires_at: None,
            }],
            vec!["parser trivia"],
        );
        profile.visibility.confidentiality_classes = VisibilityClass::Tenant;
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            interests: vec!["parser trivia".to_string()],
            confidentiality: vec!["internal".to_string()],
            ..Default::default()
        };
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)), at());
        assert!(verdict.eligible, "{:?}", verdict.reasons);
        assert!(
            verdict.reasons.iter().any(|r| r.contains("code_review")),
            "the capability reason rides: {:?}",
            verdict.reasons
        );
        assert!(
            verdict.reasons.iter().any(|r| r.contains("parser trivia")),
            "the interest reason rides: {:?}",
            verdict.reasons
        );
    }

    fn ranked_pair(
        expression: &EligibilityExpression,
        preferences: &RankingPreferences,
    ) -> Vec<RankedCandidate> {
        // Both candidates satisfy the STAGE-1 gates (the same capability +
        // interest); the good one ALSO declares the domain scope the
        // expression's affinity feature matches.
        let mut good_profile = profile_with(
            vec![crate::profiles::CapabilityClaim {
                taxonomy_id: "code_review".to_string(),
                confidence: ClaimConfidence::OwnerAttested,
                evidence_ref: None,
                expires_at: None,
            }],
            vec!["parser trivia"],
        );
        good_profile.scopes = vec!["repo:example/parser".to_string()];
        let good = candidate("rol_good", Some(good_profile));
        let partial = candidate(
            "rol_partial",
            Some(profile_with(
                vec![crate::profiles::CapabilityClaim {
                    taxonomy_id: "code_review".to_string(),
                    confidence: ClaimConfidence::OwnerAttested,
                    evidence_ref: None,
                    expires_at: None,
                }],
                vec!["parser trivia"],
            )),
        );
        let good_verdict = eligible(expression, &good, at());
        let partial_verdict = eligible(expression, &partial, at());
        rank(
            expression,
            &[(good, good_verdict), (partial, partial_verdict)],
            preferences,
        )
    }

    /// The ranking scores the exact match and orders by the weighted total.
    #[test]
    fn the_ranking_orders_by_the_weighted_total() {
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            interests: vec!["parser trivia".to_string()],
            domains: vec!["repo:example/parser".to_string()],
            ..Default::default()
        };
        let ranked = ranked_pair(&expression, &RankingPreferences::default());
        assert_eq!(ranked.len(), 2, "{ranked:?}");
        assert_eq!(
            ranked[0].role_id, "rol_good",
            "the full match ranks first: {ranked:?}"
        );
        assert!(ranked[0].total > ranked[1].total, "{ranked:?}");
        // The explanations are visibility-safe: they name the counts + the
        // matched visible facts, never a raw profile field.
        for feature in &ranked[0].features {
            assert!(
                !feature.explanation.contains("internal")
                    && !feature.explanation.contains("example"),
                "the explanation leaks a hidden field: {feature:?}"
            );
        }
    }

    /// A zero-weight feature contributes nothing; the other weights dominate.
    #[test]
    fn a_zero_weight_contributes_nothing() {
        let expression = EligibilityExpression {
            scope: ReaderClass::Tenant,
            capabilities: vec![requirement("code_review", ClaimConfidence::OwnerAttested)],
            ..Default::default()
        };
        let ranked = ranked_pair(
            &expression,
            &RankingPreferences {
                capability: 0.0,
                ..Default::default()
            },
        );
        for feature in &ranked[0].features {
            if feature.feature == "capability_match" {
                assert_eq!(feature.contribution, 0.0, "{ranked:?}");
            }
        }
    }

    /// The ranking never restores an ineligible role.
    #[test]
    fn the_ranking_never_restores_an_ineligible_role() {
        let expression = EligibilityExpression {
            exclude: vec!["rol_a".to_string()],
            ..EligibilityExpression::default()
        };
        let cand = candidate("rol_a", Some(profile_with(vec![], vec![])));
        let verdict = eligible(&expression, &cand, at());
        assert!(!verdict.eligible);
        let ranked = rank(
            &expression,
            &[(cand, verdict)],
            &RankingPreferences::default(),
        );
        assert!(
            ranked.is_empty(),
            "the ineligible role never appears: {ranked:?}"
        );
    }

    /// The diversity feature: a candidate whose provider is unique among the
    /// panel scores higher than the sharers — the selection seeks the
    /// variation (`.6.2`).
    #[test]
    fn the_diversity_feature_rewards_the_attribute_variation() {
        let expression = EligibilityExpression::default();
        let a = candidate("rol_a", Some(profile_with(vec![], vec![])));
        let b = candidate("rol_b", Some(profile_with(vec![], vec![])));
        let c = candidate("rol_c", Some(profile_with(vec![], vec![])));
        let va = eligible(&expression, &a, at());
        let vb = eligible(&expression, &b, at());
        let vc = eligible(&expression, &c, at());
        let facts: std::collections::HashMap<String, crate::dependence::MemberFacts> = [
            (
                "rol_a".to_string(),
                crate::dependence::MemberFacts {
                    role_id: "rol_a".to_string(),
                    provider: Some("openai".to_string()),
                    model_family: None,
                    harness: None,
                    lineage: None,
                    owner: None,
                },
            ),
            (
                "rol_b".to_string(),
                crate::dependence::MemberFacts {
                    role_id: "rol_b".to_string(),
                    provider: Some("openai".to_string()),
                    model_family: None,
                    harness: None,
                    lineage: None,
                    owner: None,
                },
            ),
            (
                "rol_c".to_string(),
                crate::dependence::MemberFacts {
                    role_id: "rol_c".to_string(),
                    provider: Some("anthropic".to_string()),
                    model_family: None,
                    harness: None,
                    lineage: None,
                    owner: None,
                },
            ),
        ]
        .into_iter()
        .collect();
        let ranked = rank_with_dependence(
            &expression,
            &[(a, va), (b, vb), (c, vc)],
            &RankingPreferences::default(),
            Some(&facts),
        );
        let diversity_of = |role: &str| {
            ranked
                .iter()
                .find(|r| r.role_id == role)
                .unwrap()
                .features
                .iter()
                .find(|f| f.feature == "diversity")
                .unwrap()
                .score
        };
        assert!(diversity_of("rol_c") > diversity_of("rol_a"), "{ranked:?}");
        assert_eq!(diversity_of("rol_a"), diversity_of("rol_b"), "{ranked:?}");
        // The explanation names the overlap fraction, never a probability.
        let a_expl = &ranked
            .iter()
            .find(|r| r.role_id == "rol_a")
            .unwrap()
            .features
            .iter()
            .find(|f| f.feature == "diversity")
            .unwrap()
            .explanation;
        assert!(
            a_expl.contains("50%"),
            "the explanation names the overlap fraction: {a_expl}"
        );
    }

    /// Without the dependence facts the feature contributes nothing (unknown
    /// contributes nothing, never a guess).
    #[test]
    fn the_diversity_feature_scores_zero_without_the_facts() {
        let expression = EligibilityExpression::default();
        let a = candidate("rol_a", Some(profile_with(vec![], vec![])));
        let va = eligible(&expression, &a, at());
        let ranked = rank(&expression, &[(a, va)], &RankingPreferences::default());
        let diversity = ranked[0]
            .features
            .iter()
            .find(|f| f.feature == "diversity")
            .unwrap();
        assert_eq!(diversity.score, 0.0, "{ranked:?}");
    }

    /// One candidate's dependence facts, for the `.5.1.3` controls.
    fn facts_of(
        role: &str,
        provider: Option<&str>,
        harness: Option<&str>,
        owner: Option<&str>,
    ) -> crate::dependence::MemberFacts {
        crate::dependence::MemberFacts {
            role_id: role.to_string(),
            provider: provider.map(str::to_string),
            model_family: None,
            harness: harness.map(str::to_string),
            lineage: None,
            owner: owner.map(str::to_string),
        }
    }

    /// Rank `roles` (every one eligible) with `facts`; each role's diversity
    /// feature, in role order.
    fn diversity_by_role(
        roles: &[&str],
        facts: Vec<crate::dependence::MemberFacts>,
    ) -> Vec<(String, f64, String)> {
        let expression = EligibilityExpression::default();
        let candidates: Vec<(EligibilityCandidate, EligibilityVerdict)> = roles
            .iter()
            .map(|role| {
                let c = candidate(role, Some(profile_with(vec![], vec![])));
                let v = eligible(&expression, &c, at());
                (c, v)
            })
            .collect();
        let facts: std::collections::HashMap<String, crate::dependence::MemberFacts> =
            facts.into_iter().map(|f| (f.role_id.clone(), f)).collect();
        let mut out: Vec<(String, f64, String)> = rank_with_dependence(
            &expression,
            &candidates,
            &RankingPreferences::default(),
            Some(&facts),
        )
        .into_iter()
        .map(|r| {
            let d = r
                .features
                .iter()
                .find(|f| f.feature == "diversity")
                .unwrap();
            (r.role_id.clone(), d.score, d.explanation.clone())
        })
        .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// `SIGNOFF-REPAIR.5.1.3` shape (a): a candidate with no known fact — no
    /// entry at all, or an entry declaring nothing — scores 0, and says the
    /// unknown contributes nothing. As found both scored the maximum, `1.0`.
    #[test]
    fn a_candidate_with_no_known_fact_scores_zero_diversity() {
        let scored = diversity_by_role(
            &["rol_a", "rol_b", "rol_c", "rol_d"],
            vec![
                facts_of("rol_a", Some("openai"), None, None),
                facts_of("rol_b", Some("anthropic"), None, None),
                facts_of("rol_d", None, None, None),
            ],
        );
        for (role, score, explanation) in &scored[2..] {
            assert_eq!(*score, 0.0, "{role} knows nothing: {scored:?}");
            assert!(
                explanation.contains("unknown contributes nothing"),
                "{role}: {explanation}"
            );
        }
        assert!(
            scored[0].1 > 0.0,
            "a declared, unshared provider scores: {scored:?}"
        );
    }

    /// `SIGNOFF-REPAIR.5.1.3` shape (a): declaring less never ranks higher. A
    /// candidate that also declares a provider nobody shares scores ABOVE one
    /// that leaves it undeclared; as found the two tied at `1.0`.
    #[test]
    fn declaring_an_unshared_fact_ranks_above_leaving_it_undeclared() {
        let scored = diversity_by_role(
            &["rol_a", "rol_c", "rol_d"],
            vec![
                facts_of("rol_a", Some("openai"), None, Some("own1")),
                facts_of("rol_c", None, None, Some("own3")),
                facts_of("rol_d", Some("anthropic"), None, Some("own4")),
            ],
        );
        let (c, d) = (&scored[1], &scored[2]);
        assert!(
            d.1 > c.1,
            "the declarer ranks above the silent one: {scored:?}"
        );
        assert!(c.2.contains("compared on 1 of the 5"), "{}", c.2);
        assert!(d.2.contains("compared on 2 of the 5"), "{}", d.2);
    }

    /// `SIGNOFF-REPAIR.5.1.3` shape (c): the sharer test compares a value
    /// with the SAME attribute. A provider named `x` beside a harness named
    /// `x` is no overlap; as found it was a full one and scored `0`.
    #[test]
    fn the_sharer_test_compares_the_same_attribute_only() {
        let scored = diversity_by_role(
            &["rol_a", "rol_b"],
            vec![
                facts_of("rol_a", Some("x"), None, Some("own1")),
                facts_of("rol_b", None, Some("x"), Some("own2")),
            ],
        );
        for (role, score, explanation) in &scored {
            assert!(*score > 0.0, "{role} shares nothing: {scored:?}");
            assert!(
                !explanation.contains("provider") && !explanation.contains("harness"),
                "only the owner is known on both sides: {explanation}"
            );
        }
    }

    /// The tie-break is deterministic (the role id order).
    #[test]
    fn the_ranking_is_deterministic() {
        let expression = EligibilityExpression::default();
        let a = candidate("rol_a", Some(profile_with(vec![], vec![])));
        let b = candidate("rol_b", Some(profile_with(vec![], vec![])));
        let va = eligible(&expression, &a, at());
        let vb = eligible(&expression, &b, at());
        let ranked = rank(
            &expression,
            &[(a, va), (b, vb)],
            &RankingPreferences::default(),
        );
        assert_eq!(
            ranked[0].role_id, "rol_a",
            "the tie breaks by role id: {ranked:?}"
        );
        assert_eq!(ranked[1].role_id, "rol_b");
    }

    /// `SIGNOFF-REPAIR.5.1.4`: both ends of `[0, 1]` are admitted; a weight
    /// below, above, NaN or infinite is refused by name, the first in
    /// declaration order when several stray.
    #[test]
    fn a_weight_outside_the_unit_interval_is_refused_by_name() {
        let ends = [0.0, 1.0].map(|w| RankingPreferences {
            capability: w,
            interest: w,
            affinity: w,
            latency: w,
            balance: w,
            diversity: w,
        });
        for preferences in ends {
            assert_eq!(preferences.validate(), Ok(()), "{preferences:?}");
        }
        for stray in [
            -1.0,
            -f64::MIN_POSITIVE,
            1.0 + f64::EPSILON,
            f64::MAX,
            f64::NAN,
            f64::INFINITY,
        ] {
            let preferences = RankingPreferences {
                diversity: stray,
                ..RankingPreferences::default()
            };
            let refusal = preferences
                .validate()
                .expect_err("a stray weight is refused");
            assert!(refusal.contains("`diversity`"), "{stray}: {refusal}");
        }
        let two = RankingPreferences {
            latency: -1.0,
            diversity: 2.0,
            ..RankingPreferences::default()
        };
        assert!(two.validate().unwrap_err().contains("`latency`"));
    }
}
