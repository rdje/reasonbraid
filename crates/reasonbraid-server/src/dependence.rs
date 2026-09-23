//! The dependence indicators (`PHASE-3.6.1`, §10.4): the observable conditions
//! that may correlate failures, computed as PURE overlaps over the shipped
//! lineage/ownership facts. Each indicator is a NAMED attribute + its overlap
//! groups + an explanation naming the counts — NEVER a score, never an
//! independence claim (the §10.4 rule: the UI labels these "diversity and
//! dependence indicators", never "independent probability"). The
//! similarity/timing + the calibrated correlated-error estimator are the
//! named deferrals (the labeled-evaluation domain owns them).

use serde::Serialize;

/// One panel member's dependence facts (the shipped incarnation lineage +
/// the ownership).
#[derive(Debug, Clone, PartialEq)]
pub struct MemberFacts {
    pub role_id: String,
    pub provider: Option<String>,
    pub model_family: Option<String>,
    pub harness: Option<String>,
    /// The declared distillation lineage (rides the incarnation's config
    /// when a lineage field lands — `None` today).
    pub lineage: Option<String>,
    /// The owning principal (the role template / the owner overlap).
    pub owner: Option<String>,
}

/// One overlap group: a value shared by ≥2 members.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndicatorGroup {
    pub value: String,
    pub members: Vec<String>,
}

/// One dependence indicator: the attribute, the overlap groups (a group of
/// ONE member is variation, not dependence — only ≥2 groups ride), and the
/// explanation (the counts + the named values, never a probability).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DependenceIndicator {
    pub attribute: &'static str,
    pub groups: Vec<IndicatorGroup>,
    /// The members that do not declare this attribute — unknown, never
    /// counted as variation (`SIGNOFF-REPAIR.5.1.3`).
    pub undeclared: usize,
    pub explanation: String,
}

/// One observable condition the shipped facts carry: its display name and
/// the picker that reads it from a member's facts.
pub type AttributePicker = (&'static str, fn(&MemberFacts) -> Option<&str>);

/// The §10.4 observable conditions the shipped facts carry — the common
/// provider, the model family, the harness, the declared lineage and the
/// owner. The indicators and the ranking's diversity feature read this ONE
/// list, so the two never disagree about what a dependence attribute is.
pub const ATTRIBUTES: [AttributePicker; 5] = [
    ("provider", |m: &MemberFacts| m.provider.as_deref()),
    ("model_family", |m: &MemberFacts| m.model_family.as_deref()),
    ("harness", |m: &MemberFacts| m.harness.as_deref()),
    ("lineage", |m: &MemberFacts| m.lineage.as_deref()),
    ("owner", |m: &MemberFacts| m.owner.as_deref()),
];

/// The pure computation over the [`ATTRIBUTES`]. A member that does not
/// declare an attribute is counted as `undeclared` and is never evidence of
/// variation: *varies* needs two declarers (`SIGNOFF-REPAIR.5.1.3`).
pub fn dependence_indicators(members: &[MemberFacts]) -> Vec<DependenceIndicator> {
    ATTRIBUTES
        .into_iter()
        .map(|(attribute, pick)| {
            let mut by_value: std::collections::BTreeMap<&str, Vec<&str>> =
                std::collections::BTreeMap::new();
            let mut undeclared = 0usize;
            for member in members {
                match pick(member) {
                    Some(value) => by_value
                        .entry(value)
                        .or_default()
                        .push(member.role_id.as_str()),
                    None => undeclared += 1,
                }
            }
            let declared = members.len() - undeclared;
            let groups: Vec<IndicatorGroup> = by_value
                .into_iter()
                .filter(|(_, ids)| ids.len() >= 2)
                .map(|(value, ids)| IndicatorGroup {
                    value: value.to_string(),
                    members: ids.into_iter().map(|s| s.to_string()).collect(),
                })
                .collect();
            let shared_members: usize = groups.iter().map(|g| g.members.len()).sum();
            let total = members.len();
            let mut explanation = if declared == 0 {
                format!("no panel member declares a {attribute}, so nothing is known about it")
            } else if declared == 1 {
                format!("only 1 of {total} panel members declares a {attribute}, so nothing is compared")
            } else if groups.is_empty() {
                format!(
                    "no two panel members share a {attribute} (it varies across the {declared} that declare one)"
                )
            } else {
                format!(
                    "{} of {} panel members share a {attribute} with at least one other ({} overlap group{})",
                    shared_members,
                    total,
                    groups.len(),
                    if groups.len() == 1 { "" } else { "s" }
                )
            };
            if undeclared > 0 && declared >= 2 {
                explanation.push_str(&format!("; {undeclared} of {total} do not declare one"));
            }
            DependenceIndicator {
                attribute,
                groups,
                undeclared,
                explanation,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(
        role: &str,
        provider: Option<&str>,
        model: Option<&str>,
        harness: Option<&str>,
        owner: Option<&str>,
    ) -> MemberFacts {
        MemberFacts {
            role_id: role.to_string(),
            provider: provider.map(|s| s.to_string()),
            model_family: model.map(|s| s.to_string()),
            harness: harness.map(|s| s.to_string()),
            lineage: None,
            owner: owner.map(|s| s.to_string()),
        }
    }

    /// The shared provider is the overlap; the explanation names the counts.
    #[test]
    fn a_shared_provider_forms_the_overlap_group() {
        let members = vec![
            member(
                "rol_a",
                Some("openai"),
                Some("gpt"),
                Some("codex"),
                Some("own1"),
            ),
            member(
                "rol_b",
                Some("openai"),
                Some("gpt"),
                Some("claude"),
                Some("own2"),
            ),
            member(
                "rol_c",
                Some("anthropic"),
                Some("claude"),
                Some("claude"),
                Some("own1"),
            ),
        ];
        let indicators = dependence_indicators(&members);
        let provider = indicators
            .iter()
            .find(|i| i.attribute == "provider")
            .unwrap();
        assert_eq!(provider.groups.len(), 1, "{provider:?}");
        assert_eq!(provider.groups[0].value, "openai");
        assert_eq!(provider.groups[0].members, vec!["rol_a", "rol_b"]);
        assert!(
            provider.explanation.contains("2 of 3"),
            "the explanation names the counts: {}",
            provider.explanation
        );
    }

    /// A spread attribute forms no group — the variation is the honest fact.
    #[test]
    fn a_spread_attribute_forms_no_group() {
        let members = vec![
            member("rol_a", Some("openai"), None, None, None),
            member("rol_b", Some("anthropic"), None, None, None),
        ];
        let indicators = dependence_indicators(&members);
        let provider = indicators
            .iter()
            .find(|i| i.attribute == "provider")
            .unwrap();
        assert!(provider.groups.is_empty(), "{provider:?}");
        assert!(
            provider.explanation.contains("varies"),
            "the explanation names the variation: {}",
            provider.explanation
        );
    }

    /// A group of ONE member is variation, not dependence.
    #[test]
    fn a_single_member_is_never_a_group() {
        let members = vec![member("rol_a", Some("openai"), None, None, None)];
        let indicators = dependence_indicators(&members);
        for indicator in &indicators {
            assert!(indicator.groups.is_empty(), "{indicator:?}");
        }
    }

    /// `SIGNOFF-REPAIR.5.1.3` shape (b): an attribute nobody declares is
    /// unknown, not variation, and the undeclared are counted. As found the
    /// silent attribute read *varies across the panel*, and a member without
    /// a provider was folded into the variation of those with one.
    #[test]
    fn an_undeclared_attribute_is_unknown_not_variation() {
        let members = vec![
            member("rol_a", Some("openai"), None, None, None),
            member("rol_b", Some("anthropic"), None, None, None),
            member("rol_c", None, None, None, None),
        ];
        let indicators = dependence_indicators(&members);
        let find = |name: &str| {
            let indicator = indicators.iter().find(|i| i.attribute == name).unwrap();
            serde_json::to_value(indicator).unwrap()
        };
        let harness = find("harness");
        let text = harness["explanation"].as_str().unwrap();
        assert!(
            !text.contains("varies"),
            "nobody declares a harness: {text}"
        );
        assert!(text.contains("no panel member declares"), "{text}");
        assert_eq!(harness["undeclared"], serde_json::json!(3), "{harness}");

        let provider = find("provider");
        let text = provider["explanation"].as_str().unwrap();
        assert!(text.contains("varies across the 2"), "{text}");
        assert!(text.contains("1 of 3 do not declare"), "{text}");
        assert_eq!(provider["undeclared"], serde_json::json!(1), "{provider}");
    }

    /// The owner overlap rides (the role-template dependence).
    #[test]
    fn the_shared_owner_rides_as_the_role_template_overlap() {
        let members = vec![
            member("rol_a", None, None, None, Some("own1")),
            member("rol_b", None, None, None, Some("own1")),
        ];
        let indicators = dependence_indicators(&members);
        let owner = indicators.iter().find(|i| i.attribute == "owner").unwrap();
        assert_eq!(owner.groups.len(), 1, "{owner:?}");
        assert_eq!(owner.groups[0].value, "own1");
    }

    /// The explanations never claim a probability or independence (the §10.4
    /// label rule, pre-checked at the source of the strings).
    #[test]
    fn the_explanations_never_claim_independence() {
        let members = vec![
            member(
                "rol_a",
                Some("openai"),
                Some("gpt"),
                Some("codex"),
                Some("own1"),
            ),
            member(
                "rol_b",
                Some("openai"),
                Some("gpt"),
                Some("codex"),
                Some("own1"),
            ),
        ];
        for indicator in dependence_indicators(&members) {
            let text = indicator.explanation.to_lowercase();
            assert!(
                !text.contains("independent") && !text.contains("probability"),
                "the explanation overclaims: {}",
                indicator.explanation
            );
        }
    }
}
