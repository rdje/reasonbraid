//! The actual public subject type, including enclosing authority payloads.
use chrono::{TimeZone, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AuthorityGrant, AuthorizationDecisionRecord, AuthorizationRecordId,
    Decision, DelegationConstraints, GrantAction, GrantStatus, GrantSubject, RiskClass,
    TargetSelector, TenantId,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

const HUMAN: &str = "hpr_00000000-0000-7000-8000-000000000001";
const ROLE: &str = "rol_00000000-0000-7000-8000-000000000001";
fn subjects() -> [(GrantSubject, Value); 2] {
    [
        (
            GrantSubject::Human(HUMAN.parse().unwrap()),
            json!({"kind":"human","id":HUMAN}),
        ),
        (
            GrantSubject::Role(ROLE.parse().unwrap()),
            json!({"kind":"role","id":ROLE}),
        ),
    ]
}
fn round_trip<T>(value: &T) -> Value
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let bytes = serde_json::to_vec(value).expect("the actual public type serializes");
    let decoded: T = serde_json::from_slice(&bytes).expect("the actual public type deserializes");
    assert_eq!(&decoded, value);
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn subjects_have_canonical_kind_id_json_and_round_trip() {
    for (subject, expected) in subjects() {
        assert_eq!(round_trip(&subject), expected);
        assert_eq!(
            serde_json::json!({"subject": &subject})["subject"],
            expected
        );
        // JSON member order is not authority: either order must decode identically.
        let reversed = format!(
            r#"{{"id":"{}","kind":"{}"}}"#,
            subject.id_string(),
            expected["kind"].as_str().unwrap()
        );
        assert_eq!(
            serde_json::from_str::<GrantSubject>(&reversed).unwrap(),
            subject
        );
    }
}

#[test]
fn enclosing_grants_audits_and_delegation_constraints_round_trip_real_subjects() {
    let tenant = TenantId::new();
    let at = Utc.with_ymd_and_hms(2026, 9, 9, 0, 0, 0).unwrap();
    for (subject, expected) in subjects() {
        let grant = AuthorityGrant {
            grant_id: "grt_subject_fixture".into(),
            boundary_id: "bnd_subject_fixture".into(),
            tenant_id: tenant,
            issuer: HUMAN.parse().unwrap(),
            subject: subject.clone(),
            actions: vec![GrantAction::ThreadInspect],
            selector: TargetSelector::TenantWide,
            risk_ceiling: RiskClass::Low,
            spend_limits: None,
            delegable: false,
            valid_from: at,
            expires_at: at + chrono::Duration::hours(1),
            status: GrantStatus::Active,
        };
        assert_eq!(round_trip(&grant)["subject"], expected);
        let audit = AuthorizationDecisionRecord {
            record_id: AuthorizationRecordId::new(),
            tenant_id: tenant,
            actor: actor_handle_for_subject(&subject),
            subject: Some(subject.clone()),
            boundary_id: Some(grant.boundary_id),
            grant_id: Some(grant.grant_id),
            action: GrantAction::ThreadInspect,
            target: reasonbraid_core::ResourceTarget::Tenant { tenant_id: tenant },
            decision: Decision::Allowed,
            evaluation: reasonbraid_core::AuthorizationEvaluation::BoundaryChecked {},
            policy_digest: "fixture-digest".into(),
            policy_version: "fixture-v1".into(),
            decided_at: at,
        };
        assert_eq!(round_trip(&audit)["subject"], expected);
        let delegation = DelegationConstraints {
            on_behalf_of: subject,
            purpose: Some("bounded delegated read".into()),
            scope: TargetSelector::TenantWide,
        };
        assert_eq!(round_trip(&delegation)["on_behalf_of"], expected);
    }
}

#[test]
fn malformed_ambiguous_and_mismatched_subjects_are_rejected() {
    for input in [
        json!({"kind":"human","id":ROLE}),
        json!({"kind":"role","id":HUMAN}),
        json!({"kind":"tenant","id":HUMAN}),
        json!({"kind":"Human","id":HUMAN}),
        json!({"kind":"human"}),
        json!({"id":HUMAN}),
        json!({"kind":"human","id":null}),
        json!({"kind":"human","id":17}),
        json!({"kind":"human","id":"hpr_not-a-uuid"}),
        json!({"kind":"human","id":HUMAN,"authority":"operator"}),
        json!({"human":HUMAN}),
        json!(HUMAN),
        json!(["human", HUMAN]),
        json!(null),
    ] {
        assert!(
            serde_json::from_value::<GrantSubject>(input.clone()).is_err(),
            "accepted: {input}"
        );
    }
    for input in [
        format!(r#"{{"kind":"human","kind":"role","id":"{HUMAN}"}}"#),
        format!(r#"{{"kind":"human","id":"{HUMAN}","id":"{ROLE}"}}"#),
        format!(r#"{{"id":"{HUMAN}","kind":"human","id":"{HUMAN}"}}"#),
        format!(r#"{{"extra":true,"id":"{HUMAN}","kind":"human"}}"#),
    ] {
        assert!(
            serde_json::from_str::<GrantSubject>(&input).is_err(),
            "accepted: {input}"
        );
    }
}
