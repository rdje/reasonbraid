//! The stage-1 eligibility evaluation (`PHASE-3.3.1`, backlog 29's policy-filter
//! half): the initiator's typed expression over the §10.3 stage-1 fields,
//! resolved against a candidate's shipped facts. PURE functions only — an
//! ineligible role is never restored by ranking (ADR-014's ordering), and the
//! checks run against the profile AS VISIBLE AT THE EXPRESSION'S SCOPE (a
//! tenant-hidden capability cannot satisfy a network-scope requirement).

use serde::{Deserialize, Serialize};

use crate::presence::PresenceState;
use crate::profiles::{filter_profile, AgentProfile, ClaimConfidence, ReaderClass};

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

/// The deterministic stage-1 evaluation. Each refusal names its field; an
/// eligible verdict carries the positive reasons too.
pub fn eligible(
    expression: &EligibilityExpression,
    candidate: &EligibilityCandidate,
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

    // 4. The capability requirements: each claim must be VISIBLE and carry at
    // least the required provenance.
    for requirement in &expression.capabilities {
        let claim = profile
            .capabilities
            .iter()
            .find(|c| c.taxonomy_id == requirement.taxonomy_id);
        let visible_claims = visible_obj.get("capabilities").and_then(|v| v.as_array());
        let claim_visible = visible_claims.is_some_and(|arr| {
            arr.iter().any(|c| {
                c.get("taxonomy_id").and_then(|t| t.as_str())
                    == Some(requirement.taxonomy_id.as_str())
            })
        });
        match (claim, claim_visible) {
            (Some(claim), true)
                if confidence_rank(claim.confidence)
                    >= confidence_rank(requirement.min_confidence) =>
            {
                reasons.push(format!(
                    "capability `{}` holds ({} ≥ {})",
                    requirement.taxonomy_id,
                    claim.confidence.rank_name(),
                    requirement.min_confidence.rank_name()
                ));
            }
            (Some(claim), true) => {
                return EligibilityVerdict {
                    eligible: false,
                    reasons: vec![format!(
                        "capability `{}` is {} but the requirement needs {}",
                        requirement.taxonomy_id,
                        claim.confidence.rank_name(),
                        requirement.min_confidence.rank_name()
                    )],
                };
            }
            (Some(_), false) => {
                return EligibilityVerdict {
                    eligible: false,
                    reasons: vec![format!(
                        "capability `{}` is not visible at the requested scope",
                        requirement.taxonomy_id
                    )],
                };
            }
            (None, _) => {
                return EligibilityVerdict {
                    eligible: false,
                    reasons: vec![format!(
                        "capability `{}` is not declared",
                        requirement.taxonomy_id
                    )],
                };
            }
        }
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
/// each candidate by the INVERSE of their heaviest attribute overlap with the
/// OTHER eligible candidates — a candidate whose provider is unique among the
/// panel scores 1.0; two sharers score lower. No dependence facts → the
/// feature scores 0 (unknown contributes nothing, never a guess).
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

            // The diversity feature (`.6.2`): the inverse of the heaviest
            // attribute overlap with the OTHER eligible candidates.
            let diversity_score = match dependence {
                None => 0.0,
                Some(facts) => {
                    let mine = facts.get(&candidate.role_id);
                    let others: Vec<&crate::dependence::MemberFacts> = candidates
                        .iter()
                        .filter(|(_, v)| v.eligible)
                        .filter(|(c, _)| c.role_id != candidate.role_id)
                        .filter_map(|(c, _)| facts.get(&c.role_id))
                        .collect();
                    let mut heaviest = 0.0f64;
                    if let Some(mine) = mine {
                        for value in [
                            &mine.provider,
                            &mine.model_family,
                            &mine.harness,
                            &mine.lineage,
                            &mine.owner,
                        ]
                        .into_iter()
                        .flatten()
                        {
                                let sharers = others
                                    .iter()
                                    .filter(|o| {
                                        o.provider.as_ref() == Some(value)
                                            || o.model_family.as_ref() == Some(value)
                                            || o.harness.as_ref() == Some(value)
                                            || o.lineage.as_ref() == Some(value)
                                            || o.owner.as_ref() == Some(value)
                                    })
                                    .count();
                                if !others.is_empty() {
                                    let fraction = sharers as f64 / others.len() as f64;
                                    if fraction > heaviest {
                                        heaviest = fraction;
                                    }
                                }
                        }
                    }
                    1.0 - heaviest
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
                    explanation: if dependence.is_none() {
                        "no dependence facts are loaded (the feature contributes nothing)".to_string()
                    } else if diversity_score >= 1.0 {
                        "the candidate varies across the panel's dependence attributes".to_string()
                    } else {
                        format!(
                            "the candidate shares a dependence attribute with {:.0}% of the other candidates",
                            (1.0 - diversity_score) * 100.0
                        )
                    },
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
    ranked.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.role_id.cmp(&b.role_id))
    });
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
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)));
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
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)));
        assert!(!verdict.eligible);
        assert!(
            verdict.reasons[0].contains("not visible"),
            "the refusal names the visibility: {:?}",
            verdict.reasons
        );
    }

    /// The presence gate refuses an offline candidate.
    #[test]
    fn an_offline_candidate_refuses_the_default_expression() {
        let mut cand = candidate("rol_a", Some(profile_with(vec![], vec![])));
        cand.presence_state = PresenceState::Offline;
        let verdict = eligible(&EligibilityExpression::default(), &cand);
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
        let verdict = eligible(&expression, &candidate("rol_a", Some(profile)));
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
        let good_verdict = eligible(expression, &good);
        let partial_verdict = eligible(expression, &partial);
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
        let verdict = eligible(&expression, &cand);
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
        let va = eligible(&expression, &a);
        let vb = eligible(&expression, &b);
        let vc = eligible(&expression, &c);
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
        let va = eligible(&expression, &a);
        let ranked = rank(&expression, &[(a, va)], &RankingPreferences::default());
        let diversity = ranked[0]
            .features
            .iter()
            .find(|f| f.feature == "diversity")
            .unwrap();
        assert_eq!(diversity.score, 0.0, "{ranked:?}");
    }

    /// The tie-break is deterministic (the role id order).
    #[test]
    fn the_ranking_is_deterministic() {
        let expression = EligibilityExpression::default();
        let a = candidate("rol_a", Some(profile_with(vec![], vec![])));
        let b = candidate("rol_b", Some(profile_with(vec![], vec![])));
        let va = eligible(&expression, &a);
        let vb = eligible(&expression, &b);
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
}
