//! The final administrative effect representation: closed sets, bounded text,
//! and decoding that refuses evidence it cannot read rather than guessing it
//! (`SIGNOFF-REPAIR.3.3.4.7.1`).
use chrono::{TimeZone, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AdministrativeEffectRecord, AdministrativeOperation,
    AdministrativeOutcome, AdministrativeReason, AdministrativeRefusal, AdministrativeTargetId,
    AdministrativeTextError, AgentRoleId, AuthorizationDecisionRecord, AuthorizationEvaluation,
    AuthorizationRecordId, Decision, GrantAction, GrantSubject, ResourceTarget, TenantId,
};
use serde_json::json;

fn target(value: &str) -> AdministrativeTargetId {
    AdministrativeTargetId::new(value).expect("fixture target id is within bounds")
}

fn reason(value: &str) -> AdministrativeReason {
    AdministrativeReason::new(value).expect("fixture reason is within bounds")
}

fn role() -> AgentRoleId {
    "rol_00000000-0000-7000-8000-000000000031"
        .parse()
        .expect("fixture role id")
}

/// One value per variant, so a new variant that is not listed here fails the
/// exhaustive match below at compile time rather than going untested.
fn every_operation() -> Vec<AdministrativeOperation> {
    let remote = TenantId::new();
    let all = vec![
        AdministrativeOperation::GrantRevoke {
            grant_id: target("grt_hpr_00000000-0000-7000-8000-000000000021"),
        },
        AdministrativeOperation::BoundaryRevoke {
            boundary_id: target("bnd_ten_00000000-0000-7000-8000-000000000022"),
        },
        AdministrativeOperation::BreakerArm {},
        AdministrativeOperation::BreakerReset {},
        AdministrativeOperation::NodeEnrollTokenIssue {
            node_id: target("nod_00000000-0000-7000-8000-000000000023"),
        },
        AdministrativeOperation::NodeRevoke {
            node_id: target("rol_00000000-0000-7000-8000-000000000024"),
        },
        AdministrativeOperation::NodeCommandReplay {
            node_id: target("nod_00000000-0000-7000-8000-000000000025"),
            command_id: target("work_evt_00000000-0000-7000-8000-000000000026"),
        },
        AdministrativeOperation::NodeCommandQuarantine {
            node_id: target("nod_00000000-0000-7000-8000-000000000027"),
            command_id: target("work_evt_00000000-0000-7000-8000-000000000028"),
        },
        AdministrativeOperation::NodeInboxPrune {
            node_id: target("nod_00000000-0000-7000-8000-000000000029"),
        },
        AdministrativeOperation::ProfileCardImport {
            card_digest: target(
                "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
        },
        AdministrativeOperation::CapabilityClaimAttest {
            role_id: role(),
            taxonomy_id: target("rust.review"),
        },
        AdministrativeOperation::FederationDirectionPropose {
            remote_tenant_id: remote,
        },
        AdministrativeOperation::FederationDirectionAccept {
            remote_tenant_id: remote,
        },
        AdministrativeOperation::FederationDirectionRevoke {
            remote_tenant_id: remote,
        },
    ];
    // An exhaustive match over a borrowed value: adding a variant to the enum
    // without adding it here stops compiling, so the coverage claim below is
    // structural rather than a count someone remembered to update.
    for operation in &all {
        match operation {
            AdministrativeOperation::GrantRevoke { .. }
            | AdministrativeOperation::BoundaryRevoke { .. }
            | AdministrativeOperation::BreakerArm {}
            | AdministrativeOperation::BreakerReset {}
            | AdministrativeOperation::NodeEnrollTokenIssue { .. }
            | AdministrativeOperation::NodeRevoke { .. }
            | AdministrativeOperation::NodeCommandReplay { .. }
            | AdministrativeOperation::NodeCommandQuarantine { .. }
            | AdministrativeOperation::NodeInboxPrune { .. }
            | AdministrativeOperation::ProfileCardImport { .. }
            | AdministrativeOperation::CapabilityClaimAttest { .. }
            | AdministrativeOperation::FederationDirectionPropose { .. }
            | AdministrativeOperation::FederationDirectionAccept { .. }
            | AdministrativeOperation::FederationDirectionRevoke { .. } => {}
        }
    }
    all
}

fn record(operation: AdministrativeOperation) -> AdministrativeEffectRecord {
    AdministrativeEffectRecord {
        record_id: AuthorizationRecordId::new(),
        tenant_id: TenantId::new(),
        operation,
        submitted_reason: Some(reason("role retired")),
        outcome: AdministrativeOutcome::Applied {},
        effected_at: Utc.with_ymd_and_hms(2026, 9, 12, 10, 30, 0).unwrap(),
    }
}

#[test]
fn every_operation_round_trips_carrying_its_own_target() {
    let operations = every_operation();
    assert_eq!(operations.len(), AdministrativeOperation::KINDS.len());
    for operation in operations {
        let wire = serde_json::to_value(&operation).unwrap();
        assert_eq!(
            wire.get("kind").and_then(|kind| kind.as_str()),
            Some(operation.kind()),
            "the serialized tag and kind() must be the same vocabulary"
        );
        assert!(
            AdministrativeOperation::KINDS.contains(&operation.kind()),
            "`{}` is missing from KINDS, which the migration's CHECK mirrors",
            operation.kind()
        );
        let decoded: AdministrativeOperation = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(decoded, operation);
        assert_eq!(serde_json::to_value(&decoded).unwrap(), wire);
    }
}

#[test]
fn the_declared_kind_list_is_distinct_and_complete() {
    let mut sorted = AdministrativeOperation::KINDS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        AdministrativeOperation::KINDS.len(),
        "two variants share a stored discriminant"
    );
    let produced: Vec<&str> = every_operation()
        .iter()
        .map(AdministrativeOperation::kind)
        .collect();
    assert_eq!(produced, AdministrativeOperation::KINDS.to_vec());
    let mut outcomes = AdministrativeOutcome::KINDS.to_vec();
    outcomes.sort_unstable();
    outcomes.dedup();
    assert_eq!(outcomes.len(), AdministrativeOutcome::KINDS.len());
}

#[test]
fn an_operation_refuses_the_sequence_bare_string_and_unknown_kind_forms() {
    // The pinned internally tagged decoder also accepts a sequence; a stored
    // array is not an operation object and must not decode as one.
    for wire in [
        json!(["breaker_arm"]),
        json!("breaker_arm"),
        json!(42),
        json!(null),
        json!({}),
        json!({"kind": "grant_revoke_all"}),
    ] {
        assert!(
            serde_json::from_value::<AdministrativeOperation>(wire.clone()).is_err(),
            "{wire} decoded as an operation"
        );
    }
}

#[test]
fn an_operation_refuses_unknown_duplicate_and_missing_fields() {
    // An unknown member: a tenant-wide marker carrying a discarded target is the
    // shape that silently loses evidence.
    assert!(serde_json::from_value::<AdministrativeOperation>(
        json!({"kind": "breaker_arm", "grant_id": "grt_x"})
    )
    .is_err());
    // A missing member.
    assert!(
        serde_json::from_value::<AdministrativeOperation>(json!({"kind": "grant_revoke"})).is_err()
    );
    // A duplicate member survives as far as the decoder, which refuses it —
    // this is why `MapAccess` is passed through instead of a Value round trip.
    assert!(serde_json::from_str::<AdministrativeOperation>(
        r#"{"kind":"grant_revoke","grant_id":"grt_a","grant_id":"grt_b"}"#
    )
    .is_err());
    // A node command names both halves of its target or it names neither.
    assert!(serde_json::from_value::<AdministrativeOperation>(
        json!({"kind": "node_command_replay", "node_id": "nod_a"})
    )
    .is_err());
}

#[test]
fn an_operation_refuses_a_wrong_identifier_family() {
    // The typed halves are prefix-checked; a thread id is not a role.
    assert!(serde_json::from_value::<AdministrativeOperation>(json!({
        "kind": "capability_claim_attest",
        "role_id": "thr_00000000-0000-7000-8000-000000000031",
        "taxonomy_id": "rust.review",
    }))
    .is_err());
    assert!(serde_json::from_value::<AdministrativeOperation>(json!({
        "kind": "federation_direction_accept",
        "remote_tenant_id": "rol_00000000-0000-7000-8000-000000000032",
    }))
    .is_err());
}

#[test]
fn the_three_outcomes_are_distinct_and_only_applied_asserts_a_change() {
    let applied = AdministrativeOutcome::Applied {};
    let no_op = AdministrativeOutcome::NoOp {
        detail: reason("the grant was already revoked"),
    };
    let refused = AdministrativeOutcome::Refused {
        code: AdministrativeRefusal::InvalidTransition,
        detail: reason("the boundary is not in a revocable state"),
    };
    assert!(applied.changed_protected_state());
    assert!(!no_op.changed_protected_state());
    assert!(!refused.changed_protected_state());
    for outcome in [applied, no_op, refused] {
        let wire = serde_json::to_value(&outcome).unwrap();
        assert!(AdministrativeOutcome::KINDS.contains(&outcome.kind()));
        assert_eq!(
            wire.get("kind").and_then(|kind| kind.as_str()),
            Some(outcome.kind())
        );
        let decoded: AdministrativeOutcome = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(decoded, outcome);
    }
    assert_eq!(
        serde_json::to_value(AdministrativeOutcome::Refused {
            code: AdministrativeRefusal::InvalidTransition,
            detail: reason("already revoked"),
        })
        .unwrap(),
        json!({"kind":"refused","code":"invalid_transition","detail":"already revoked"})
    );
}

#[test]
fn a_refusal_code_outside_the_measured_set_is_a_decode_failure_not_a_guess() {
    // The four codes are the ones the fourteen administrative handlers actually
    // refuse an admitted operation with, and they are literally the strings the
    // HTTP response carries.
    for code in AdministrativeRefusal::CODES {
        let decoded: AdministrativeOutcome = serde_json::from_value(
            json!({"kind": "refused", "code": code, "detail": "the target moved"}),
        )
        .unwrap_or_else(|e| panic!("`{code}` must decode: {e}"));
        let AdministrativeOutcome::Refused { code: parsed, .. } = &decoded else {
            panic!("`{code}` decoded as {decoded:?}")
        };
        assert_eq!(parsed.as_str(), code);
        assert_eq!(serde_json::to_value(&decoded).unwrap()["code"], json!(code));
    }
    // ⚠️ `not_found` is the case that made this its own vocabulary: the §9.8
    // registry does not contain it, and it is exactly what a 404 returns.
    assert_eq!(AdministrativeRefusal::NotFound.as_str(), "not_found");
    // ⚠️ `unauthorized` was on the OUTSIDE list until `SIGNOFF-REPAIR.3.3.4.7.4`,
    // because `.7.3`'s census read every `ControlApiError::unauthorized` in the
    // fourteen handlers as the admission's own denial. That is true thirteen
    // times; the fourteenth is the card import's allowlist rung, which refuses an
    // ADMITTED tenant administrator over a missing federation agreement. A code
    // that names a real post-admission refusal cannot be on the fail-closed list
    // — the record would have had to disagree with the response to be written.
    assert_eq!(AdministrativeRefusal::Unauthorized.as_str(), "unauthorized");
    // Fail closed: this build writes these codes, so a code it cannot name means
    // the row was not written by a build this one understands. `quota_exceeded`
    // is a real code the product emits elsewhere and still not one of these.
    for outside in [
        "quota_exhausted",
        "quota_exceeded",
        "dependency_unavailable",
    ] {
        assert!(
            serde_json::from_value::<AdministrativeOutcome>(
                json!({"kind": "refused", "code": outside, "detail": "x"})
            )
            .is_err(),
            "`{outside}` decoded as an administrative refusal"
        );
    }
    // A refusal states both halves; a code with no detail is not a refusal.
    assert!(serde_json::from_value::<AdministrativeOutcome>(
        json!({"kind": "refused", "code": "invalid_transition"})
    )
    .is_err());
    // `applied` carries nothing, and an `applied` with a smuggled detail is not
    // silently downgraded to a plain success.
    assert!(serde_json::from_value::<AdministrativeOutcome>(
        json!({"kind": "applied", "detail": "x"})
    )
    .is_err());
}

#[test]
fn bounded_text_refuses_blank_overlong_and_control_characters_at_its_exact_edge() {
    assert!(matches!(
        AdministrativeTargetId::new("   "),
        Err(AdministrativeTextError::Blank { .. })
    ));
    assert!(matches!(
        AdministrativeReason::new(""),
        Err(AdministrativeTextError::Blank { .. })
    ));
    assert!(matches!(
        AdministrativeTargetId::new("grt_\nx"),
        Err(AdministrativeTextError::ControlCharacter { .. })
    ));
    assert!(matches!(
        AdministrativeReason::new("role\tretired"),
        Err(AdministrativeTextError::ControlCharacter { .. })
    ));

    // The limit is BYTES, checked at the edge in both directions.
    let at_limit = "x".repeat(AdministrativeTargetId::MAX_BYTES);
    assert!(AdministrativeTargetId::new(at_limit.clone()).is_ok());
    assert!(matches!(
        AdministrativeTargetId::new(format!("{at_limit}x")),
        Err(AdministrativeTextError::TooLong {
            limit: 256,
            bytes: 257,
            ..
        })
    ));
    let at_reason_limit = "y".repeat(AdministrativeReason::MAX_BYTES);
    assert!(AdministrativeReason::new(at_reason_limit.clone()).is_ok());
    assert!(matches!(
        AdministrativeReason::new(format!("{at_reason_limit}y")),
        Err(AdministrativeTextError::TooLong {
            limit: 1024,
            bytes: 1025,
            ..
        })
    ));
    // 128 three-byte characters are 384 bytes: a character count would accept
    // this and a byte count refuses it, which is the one the column enforces.
    let wide = "\u{4e16}".repeat(128);
    assert_eq!(wide.chars().count(), 128);
    assert_eq!(wide.len(), 384);
    assert!(AdministrativeTargetId::new(wide).is_err());

    // The same bounds apply on the way IN from storage, not only on the way out.
    assert!(serde_json::from_value::<AdministrativeReason>(json!("  ")).is_err());
    let overlong = "z".repeat(AdministrativeTargetId::MAX_BYTES + 1);
    assert!(serde_json::from_value::<AdministrativeTargetId>(json!(overlong)).is_err());
    assert!(serde_json::from_value::<AdministrativeTargetId>(json!(7)).is_err());
    // Spelling is preserved verbatim rather than normalized.
    let spelled = target("  RÉGION dev-local  ");
    assert_eq!(spelled.as_str(), "  RÉGION dev-local  ");
    assert_eq!(
        serde_json::to_value(&spelled).unwrap(),
        json!("  RÉGION dev-local  ")
    );
}

#[test]
fn a_record_round_trips_and_states_its_absent_reason_explicitly() {
    for operation in every_operation() {
        let value = record(operation);
        let wire = serde_json::to_value(&value).unwrap();
        let decoded: AdministrativeEffectRecord = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(serde_json::to_value(&decoded).unwrap(), wire);
    }

    // This record has no history, so a MISSING reason field is malformed
    // evidence rather than a silently defaulted absence.
    let mut wire = serde_json::to_value(record(AdministrativeOperation::BreakerReset {})).unwrap();
    wire.as_object_mut().unwrap().remove("submitted_reason");
    assert!(serde_json::from_value::<AdministrativeEffectRecord>(wire.clone()).is_err());

    // An explicit null is the way to say "this operation takes no reason".
    wire.as_object_mut()
        .unwrap()
        .insert("submitted_reason".into(), json!(null));
    let decoded: AdministrativeEffectRecord = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(decoded.submitted_reason, None);
    assert_eq!(serde_json::to_value(&decoded).unwrap(), wire);

    // An unknown member, a sequence, and a wrong identifier family all refuse.
    let mut unknown = wire.clone();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("applied".into(), json!(true));
    assert!(serde_json::from_value::<AdministrativeEffectRecord>(unknown).is_err());
    assert!(serde_json::from_value::<AdministrativeEffectRecord>(json!([])).is_err());
    let mut wrong_family = wire.clone();
    wrong_family.as_object_mut().unwrap().insert(
        "record_id".into(),
        json!("ten_00000000-0000-7000-8000-000000000041"),
    );
    assert!(serde_json::from_value::<AdministrativeEffectRecord>(wrong_family).is_err());
}

#[test]
fn the_effect_record_does_not_relabel_the_authorization_record() {
    // The admission's own JSON is unchanged: the effect is an additive, separate
    // record, so nothing an operator already reads gains or loses a field.
    let tenant_id = TenantId::new();
    let principal =
        GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000141".parse().unwrap());
    let admission = AuthorizationDecisionRecord {
        record_id: AuthorizationRecordId::new(),
        tenant_id,
        actor: actor_handle_for_subject(&principal),
        subject: None,
        boundary_id: Some("bnd_effect_record".into()),
        grant_id: Some("grt_effect_record".into()),
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant { tenant_id },
        decision: Decision::Allowed,
        evaluation: AuthorizationEvaluation::BoundaryChecked {},
        policy_digest: "fixture-digest".into(),
        policy_version: "fixture-v1".into(),
        decided_at: Utc.with_ymd_and_hms(2026, 9, 12, 10, 30, 0).unwrap(),
    };
    let wire = serde_json::to_value(&admission).unwrap();
    let mut keys: Vec<&str> = wire
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "action",
            "actor",
            "boundary_id",
            "decided_at",
            "decision",
            "evaluation",
            "grant_id",
            "policy_digest",
            "policy_version",
            "record_id",
            "subject",
            "target",
            "tenant_id",
        ],
        "the admission record gained or lost a field"
    );

    // An effect names its admission by that admission's own id; nothing about
    // the admission says whether an effect exists.
    let effect = AdministrativeEffectRecord {
        record_id: admission.record_id,
        tenant_id,
        operation: AdministrativeOperation::GrantRevoke {
            grant_id: target("grt_effect_record"),
        },
        submitted_reason: Some(reason("role retired")),
        outcome: AdministrativeOutcome::NoOp {
            detail: reason("the grant was already revoked"),
        },
        effected_at: admission.decided_at,
    };
    assert_eq!(effect.record_id, admission.record_id);
    assert!(!effect.outcome.changed_protected_state());
    assert!(serde_json::from_value::<AdministrativeEffectRecord>(wire).is_err());
}
