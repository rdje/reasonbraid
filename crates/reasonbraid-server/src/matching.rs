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
                _ => {
                    return EligibilityVerdict {
                        eligible: false,
                        reasons: vec![format!(
                            "available budget does not meet the requirement of {min}"
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
        let mut expression = EligibilityExpression::default();
        expression.exclude = vec!["rol_a".to_string()];
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
}
