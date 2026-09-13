//! Evaluation provenance is distinct from action and preserves missing history.
use chrono::{TimeZone, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AuthorizationDecisionRecord, AuthorizationEvaluation,
    AuthorizationRecordId, BoundaryStatus, CachedDecisionKind, Decision, GrantAction, GrantSubject,
    ResourceTarget, TargetSelector, TenantAdminInspection, TenantId,
};
use serde_json::json;

fn record() -> AuthorizationDecisionRecord {
    let tenant_id = TenantId::new();
    let principal =
        GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000141".parse().unwrap());
    AuthorizationDecisionRecord {
        record_id: AuthorizationRecordId::new(),
        tenant_id,
        actor: actor_handle_for_subject(&principal),
        subject: None,
        boundary_id: Some("bnd_evaluation_record".into()),
        grant_id: Some("grt_evaluation_record".into()),
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant { tenant_id },
        decision: Decision::Allowed,
        evaluation: AuthorizationEvaluation::BoundaryChecked {},
        policy_digest: "fixture-digest".into(),
        policy_version: "fixture-v1".into(),
        decided_at: Utc.with_ymd_and_hms(2026, 9, 9, 0, 0, 0).unwrap(),
    }
}

#[test]
fn historical_json_keeps_unknown_evaluation_instead_of_inferred_authority() {
    let mut wire = serde_json::to_value(record()).unwrap();
    wire.as_object_mut().unwrap().remove("evaluation");
    let decoded: AuthorizationDecisionRecord = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(
        decoded.evaluation,
        AuthorizationEvaluation::LegacyUnspecified {}
    );
    let mut round_trip = serde_json::to_value(decoded).unwrap();
    assert_eq!(
        round_trip.as_object_mut().unwrap().remove("evaluation"),
        Some(json!({"kind":"legacy_unspecified"}))
    );
    assert_eq!(round_trip, wire, "no other historical field is rewritten");
    assert_ne!(
        AuthorizationEvaluation::default(),
        AuthorizationEvaluation::BoundaryChecked {}
    );
}

#[test]
fn inspection_provenance_round_trips_the_actual_subject_status_scope_and_purpose() {
    for principal in [
        GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000141".parse().unwrap()),
        GrantSubject::Role("rol_00000000-0000-7000-8000-000000000142".parse().unwrap()),
    ] {
        for inspection in [
            TenantAdminInspection::NodesPresence {},
            TenantAdminInspection::Grants {},
            TenantAdminInspection::Boundaries {},
            TenantAdminInspection::Incarnations {},
            TenantAdminInspection::Runs {},
            TenantAdminInspection::Breakers {},
            TenantAdminInspection::Usage {},
            TenantAdminInspection::AuthorizationRecord {
                record_id: AuthorizationRecordId::new(),
            },
        ] {
            let mut record = record();
            record.actor = actor_handle_for_subject(&principal);
            record.evaluation = AuthorizationEvaluation::TenantAdminInspection {
                principal: principal.clone(),
                inspection,
                boundary_status: Some(BoundaryStatus::Revoked),
                grant_selector: Some(TargetSelector::TenantWide),
            };
            let wire = serde_json::to_value(&record).unwrap();
            assert_eq!(wire["evaluation"]["kind"], "tenant_admin_inspection");
            assert_eq!(wire["evaluation"]["boundary_status"], "revoked");
            assert_eq!(
                wire["evaluation"]["principal"],
                serde_json::to_value(&principal).unwrap()
            );
            assert_eq!(
                serde_json::from_value::<AuthorizationDecisionRecord>(wire).unwrap(),
                record
            );
        }
    }
}

#[test]
fn evaluation_metadata_rejects_unknown_or_ambiguous_shapes() {
    let principal = json!({"kind":"human","id":"hpr_00000000-0000-7000-8000-000000000141"});
    for invalid in [
        json!(null),
        json!("boundary_checked"),
        json!([]),
        json!(["boundary_checked"]),
        json!({}),
        json!({"kind":"unknown"}),
        json!({"kind":"boundary_checked","extra":true}),
        json!({"kind":"legacy_unspecified","principal":principal}),
        json!({"kind":"tenant_admin_inspection","principal":principal}),
        json!({"kind":"tenant_admin_inspection","principal":principal,"inspection":{"kind":"site_registry"}}),
        json!({"kind":"tenant_admin_inspection","principal":principal,"inspection":["grants"]}),
        json!({"kind":"tenant_admin_inspection","principal":principal,"inspection":{"kind":"grants","extra":true}}),
        json!({"kind":"tenant_admin_inspection","principal":principal,"inspection":{"kind":"authorization_record","record_id":"not-a-record"}}),
        json!({"kind":"tenant_admin_inspection","principal":principal,"inspection":{"kind":"grants"},"boundary_status":"unknown"}),
    ] {
        assert!(
            serde_json::from_value::<AuthorizationEvaluation>(invalid.clone()).is_err(),
            "accepted {invalid}"
        );
    }
    assert!(serde_json::from_str::<AuthorizationEvaluation>(
        r#"{"kind":"boundary_checked","kind":"legacy_unspecified"}"#
    )
    .is_err());
    assert!(serde_json::from_str::<TenantAdminInspection>(
        r#"{"kind":"authorization_record","record_id":"authz_00000000-0000-7000-8000-000000000141","record_id":"authz_00000000-0000-7000-8000-000000000142"}"#
    ).is_err());
}

#[test]
fn selector_scope_cannot_discard_named_threads_or_accept_sequence_alternatives() {
    let thread = "thr_00000000-0000-7000-8000-000000000141";
    for wire in [
        json!({"kind":"tenant_wide"}),
        json!({"kind":"threads","threads":[thread]}),
    ] {
        let selector: TargetSelector = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(selector).unwrap(), wire);
    }
    let mut failures = Vec::new();
    for invalid in [
        json!({"kind":"tenant_wide","threads":[thread]}),
        json!({"kind":"tenant_wide","extra":true}),
        json!(["tenant_wide"]),
        json!(["threads", [thread]]),
    ] {
        if serde_json::from_value::<TargetSelector>(invalid.clone()).is_ok() {
            failures.push(format!("direct selector accepted {invalid}"));
        }
        let evidence = json!({
            "kind":"tenant_admin_inspection",
            "principal":{"kind":"human","id":"hpr_00000000-0000-7000-8000-000000000141"},
            "inspection":{"kind":"grants"},
            "boundary_status":"revoked",
            "grant_selector":invalid,
        });
        if serde_json::from_value::<AuthorizationEvaluation>(evidence).is_ok() {
            failures.push(format!("enclosing scope evidence accepted {invalid}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `SIGNOFF-REPAIR.3.4.4` — the remaining tagged authority codecs. `Decision` and
/// `CachedDecisionKind` each carry an internally tagged UNIT variant, the same
/// shape `.3.3.3.2.2.1` repaired for the selector and evaluation types: the
/// pinned Serde decoder discards a unit variant's extra members and accepts the
/// sequence form, neither of which `deny_unknown_fields` switches off.
///
/// The direction that matters is stated in the fixtures: for both types the
/// discarded member is the DENIAL's reason, so the lenient decoder turned a deny
/// into an allow while throwing away the evidence of what was refused.
#[test]
fn the_decision_codecs_refuse_discarded_members_and_sequence_alternatives() {
    for wire in [
        json!({"decision":"allowed"}),
        json!({"decision":"denied","reason":"the grant is revoked"}),
    ] {
        let decision: Decision = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(decision).unwrap(), wire);
    }

    let mut failures = Vec::new();
    for invalid in [
        json!({"decision":"allowed","reason":"the grant is revoked"}),
        json!({"decision":"allowed","extra":true}),
        json!(["allowed"]),
        json!(["denied", {"reason":"x"}]),
    ] {
        if let Ok(decoded) = serde_json::from_value::<Decision>(invalid.clone()) {
            failures.push(format!("Decision accepted {invalid} as {decoded:?}"));
        }
        // The enclosing audit record must not launder it either.
        let mut wire = serde_json::to_value(record()).unwrap();
        wire.as_object_mut()
            .unwrap()
            .insert("decision".into(), invalid.clone());
        if serde_json::from_value::<AuthorizationDecisionRecord>(wire).is_ok() {
            failures.push(format!("the enclosing record accepted {invalid}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The node's cached admission decision, same shape and same direction: a
/// discarded `reason` on `{"decision":"allow"}` is a deny read as an allow.
#[test]
fn the_cached_decision_kind_refuses_discarded_members_and_sequence_alternatives() {
    for wire in [
        json!({"decision":"allow"}),
        json!({"decision":"deny","reason":"boundary frozen"}),
    ] {
        let kind: CachedDecisionKind = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(kind).unwrap(), wire);
    }

    let mut failures = Vec::new();
    for invalid in [
        json!({"decision":"allow","reason":"boundary frozen"}),
        json!({"decision":"allow","extra":true}),
        json!(["allow"]),
        json!(["deny", {"reason":"x"}]),
    ] {
        if let Ok(decoded) = serde_json::from_value::<CachedDecisionKind>(invalid.clone()) {
            failures.push(format!(
                "CachedDecisionKind accepted {invalid} as {decoded:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
