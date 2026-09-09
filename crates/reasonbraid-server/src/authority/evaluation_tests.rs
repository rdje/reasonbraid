//! Pure controls against the evaluator used by the real transactional authorizer.
use super::*;
use chrono::{Duration, TimeZone};
use reasonbraid_core::{actor_handle_for_subject, TenantId, ThreadId};

fn fixture(action: GrantAction) -> (EnrollmentAuthorityBoundary, AuthorityGrant, CommandAuthz) {
    let tenant_id = TenantId::new();
    let principal =
        GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000001".parse().unwrap());
    let start = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    let boundary = EnrollmentAuthorityBoundary {
        boundary_id: "bnd_evaluation".into(),
        tenant_id,
        parent_or_root_authority: "fixture-root".into(),
        target_owner: "fixture-owner".into(),
        permitted_actions: vec![action],
        permitted_domains: vec!["deliberation".into()],
        risk_ceiling: RiskClass::Medium,
        spend_ceiling: Some(serde_json::json!({"amount":100})),
        delegable: false,
        max_delegation_depth: 1,
        valid_from: start,
        expires_at: start + Duration::days(30),
        charter_digest: "fixture-charter".into(),
        policy_version: "fixture-policy".into(),
        status: BoundaryStatus::Active,
    };
    let grant = AuthorityGrant {
        grant_id: "grt_evaluation".into(),
        boundary_id: boundary.boundary_id.clone(),
        tenant_id,
        issuer: "hpr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        subject: principal.clone(),
        actions: vec![action],
        selector: TargetSelector::TenantWide,
        risk_ceiling: RiskClass::Low,
        spend_limits: Some(serde_json::json!({"amount":50})),
        delegable: false,
        valid_from: start + Duration::days(1),
        expires_at: start + Duration::days(29),
        status: GrantStatus::Active,
    };
    let context = CommandAuthz {
        actor: actor_handle_for_subject(&principal),
        principal,
        delegate_subject: None,
        delegation_scope: None,
        action,
        target: ResourceTarget::Tenant { tenant_id },
    };
    (boundary, grant, context)
}

fn at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 9, 0, 0, 0).unwrap()
}

fn denied(b: &EnrollmentAuthorityBoundary, g: &AuthorityGrant, a: &CommandAuthz) {
    assert!(matches!(
        evaluate(Some(b), Some(g), a, at()),
        Decision::Denied { .. }
    ));
}

#[test]
fn unrelated_boundary_tenant_or_subject_cannot_supply_authority() {
    let (b, g, mut a) = fixture(GrantAction::TenantAdmin);
    assert_eq!(evaluate(Some(&b), Some(&g), &a, at()), Decision::Allowed);
    let mut other = b.clone();
    other.boundary_id = "bnd_unrelated".into();
    denied(&other, &g, &a);
    other = b.clone();
    other.tenant_id = TenantId::new();
    denied(&other, &g, &a);
    a.principal = GrantSubject::Role("rol_00000000-0000-7000-8000-000000000001".parse().unwrap());
    denied(&b, &g, &a);
    // Subject evaluation uses the delegated authority source. Caller permission
    // is evaluated separately; this result alone is not a delegation allowance.
    a.delegate_subject = Some(g.subject.clone());
    assert_eq!(evaluate(Some(&b), Some(&g), &a, at()), Decision::Allowed);
    a.delegate_subject = Some(a.principal.clone());
    denied(&b, &g, &a);
}

#[test]
fn tenant_targets_require_tenant_wide_selectors() {
    for action in [
        GrantAction::ThreadCreate,
        GrantAction::ThreadCreateAuto,
        GrantAction::TenantAdmin,
        GrantAction::ThreadInspect,
    ] {
        let (b, mut g, a) = fixture(action);
        assert_eq!(evaluate(Some(&b), Some(&g), &a, at()), Decision::Allowed);
        for threads in [vec![], vec![ThreadId::new()]] {
            g.selector = TargetSelector::Threads { threads };
            denied(&b, &g, &a);
        }
    }
}

#[test]
fn actions_only_accept_their_supported_target_kinds() {
    for (action, tenant_allowed, thread_allowed) in [
        (GrantAction::ThreadCreate, true, false),
        (GrantAction::ThreadCreateAuto, true, false),
        (GrantAction::TenantAdmin, true, false),
        (GrantAction::ThreadInspect, true, true),
        (GrantAction::ThreadInvite, false, true),
        (GrantAction::ThreadContribute, false, true),
        (GrantAction::ThreadClose, false, true),
        (GrantAction::ThreadCancel, false, true),
        (GrantAction::ThreadInvitationRespond, false, true),
        (GrantAction::ThreadAdvanceRound, false, true),
    ] {
        let (b, g, mut a) = fixture(action);
        assert_eq!(
            evaluate(Some(&b), Some(&g), &a, at()) == Decision::Allowed,
            tenant_allowed,
            "{action} on tenant"
        );
        a.target = ResourceTarget::Thread {
            tenant_id: g.tenant_id,
            thread_id: ThreadId::new(),
        };
        assert_eq!(
            evaluate(Some(&b), Some(&g), &a, at()) == Decision::Allowed,
            thread_allowed,
            "{action} on thread"
        );
    }
}

#[test]
fn thread_selectors_cover_only_named_threads_in_the_same_tenant() {
    let (b, mut g, mut a) = fixture(GrantAction::ThreadInspect);
    let selected = ThreadId::new();
    g.selector = TargetSelector::Threads {
        threads: vec![selected],
    };
    a.target = ResourceTarget::Thread {
        tenant_id: g.tenant_id,
        thread_id: selected,
    };
    assert_eq!(evaluate(Some(&b), Some(&g), &a, at()), Decision::Allowed);
    a.target = ResourceTarget::Thread {
        tenant_id: g.tenant_id,
        thread_id: ThreadId::new(),
    };
    denied(&b, &g, &a);
    a.target = ResourceTarget::Thread {
        tenant_id: TenantId::new(),
        thread_id: selected,
    };
    denied(&b, &g, &a);
    g.selector = TargetSelector::TenantWide;
    denied(&b, &g, &a);
    a.target = ResourceTarget::Tenant {
        tenant_id: TenantId::new(),
    };
    denied(&b, &g, &a);
}

#[test]
fn evaluation_rechecks_each_ceiling_and_live_authority() {
    let (b, g, a) = fixture(GrantAction::TenantAdmin);
    assert_eq!(evaluate(Some(&b), Some(&g), &a, at()), Decision::Allowed);
    for change in 0..8 {
        let mut invalid = g.clone();
        match change {
            0 => invalid.actions.push(GrantAction::ThreadCreate),
            1 => invalid.risk_ceiling = RiskClass::High,
            2 => invalid.spend_limits = Some(serde_json::json!({"amount":101})),
            3 => invalid.delegable = true,
            4 => invalid.valid_from = b.valid_from - Duration::seconds(1),
            5 => invalid.expires_at = b.expires_at + Duration::seconds(1),
            6 => invalid.status = GrantStatus::Revoked,
            7 => invalid.actions.clear(),
            _ => unreachable!(),
        }
        denied(&b, &invalid, &a);
    }
    for status in [BoundaryStatus::Suspended, BoundaryStatus::Revoked] {
        let mut inactive = b.clone();
        inactive.status = status;
        denied(&inactive, &g, &a);
    }
    assert!(matches!(
        evaluate(None, Some(&g), &a, at()),
        Decision::Denied { .. }
    ));
    assert!(matches!(
        evaluate(Some(&b), None, &a, at()),
        Decision::Denied { .. }
    ));
}

#[test]
fn evaluation_excludes_exact_expiration_and_allows_the_start() {
    let (b, g, a) = fixture(GrantAction::TenantAdmin);
    let tick = Duration::nanoseconds(1);
    for (at, allowed) in [
        (g.valid_from - tick, false),
        (g.valid_from, true),
        (g.expires_at - tick, true),
        (g.expires_at, false),
    ] {
        assert_eq!(
            evaluate(Some(&b), Some(&g), &a, at) == Decision::Allowed,
            allowed,
            "at {at}"
        );
    }
    let mut equal = g;
    equal.valid_from = b.valid_from;
    equal.expires_at = b.expires_at;
    assert_eq!(
        evaluate(Some(&b), Some(&equal), &a, b.valid_from),
        Decision::Allowed
    );
    assert!(matches!(
        evaluate(Some(&b), Some(&equal), &a, b.expires_at),
        Decision::Denied { .. }
    ));
}

#[test]
fn frozen_inspection_excepted_status_does_not_enable_commands() {
    for subject in [
        GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000001".parse().unwrap()),
        GrantSubject::Role("rol_00000000-0000-7000-8000-000000000002".parse().unwrap()),
    ] {
        for status in [
            BoundaryStatus::Active,
            BoundaryStatus::Suspended,
            BoundaryStatus::Revoked,
        ] {
            let (mut b, mut g, mut a) = fixture(GrantAction::TenantAdmin);
            b.status = status;
            g.subject = subject.clone();
            a.principal = subject.clone();
            a.actor = actor_handle_for_subject(&subject);
            assert_eq!(
                evaluate_tenant_admin_read(Some(&b), Some(&g), &a, at()),
                Decision::Allowed
            );
            assert_eq!(
                b.status, status,
                "projection cannot change source attribution"
            );
            assert_eq!(
                evaluate(Some(&b), Some(&g), &a, at()) == Decision::Allowed,
                status == BoundaryStatus::Active
            );
        }
    }
}

#[test]
fn frozen_inspection_retains_binding_scope_and_every_grant_ceiling() {
    for change in 0..13 {
        let (mut b, mut g, mut a) = fixture(GrantAction::TenantAdmin);
        b.status = BoundaryStatus::Revoked;
        match change {
            0 => g.boundary_id = "bnd_other".into(),
            1 => b.tenant_id = TenantId::new(),
            2 => {
                a.principal =
                    GrantSubject::Role("rol_00000000-0000-7000-8000-000000000002".parse().unwrap())
            }
            3 => {
                a.target = ResourceTarget::Tenant {
                    tenant_id: TenantId::new(),
                }
            }
            4 => g.selector = TargetSelector::Threads { threads: vec![] },
            5 => g.actions.clear(),
            6 => g.actions.push(GrantAction::ThreadCreate),
            7 => g.risk_ceiling = RiskClass::High,
            8 => g.spend_limits = Some(serde_json::json!({"amount":101})),
            9 => g.delegable = true,
            10 => g.valid_from = b.valid_from - Duration::seconds(1),
            11 => g.expires_at = b.expires_at + Duration::seconds(1),
            12 => g.status = GrantStatus::Revoked,
            _ => unreachable!(),
        }
        assert!(
            matches!(
                evaluate_tenant_admin_read(Some(&b), Some(&g), &a, at()),
                Decision::Denied { .. }
            ),
            "change {change}"
        );
    }
}

#[test]
fn frozen_inspection_windows_are_nonempty_and_half_open() {
    let (mut b, g, a) = fixture(GrantAction::TenantAdmin);
    b.status = BoundaryStatus::Suspended;
    let tick = Duration::nanoseconds(1);
    for (time, allowed) in [
        (g.valid_from - tick, false),
        (g.valid_from, true),
        (g.expires_at - tick, true),
        (g.expires_at, false),
    ] {
        assert_eq!(
            evaluate_tenant_admin_read(Some(&b), Some(&g), &a, time) == Decision::Allowed,
            allowed,
            "{time}"
        );
    }
    let mut coextensive = g.clone();
    coextensive.valid_from = b.valid_from;
    coextensive.expires_at = b.expires_at;
    assert_eq!(
        evaluate_tenant_admin_read(Some(&b), Some(&coextensive), &a, b.valid_from),
        Decision::Allowed
    );
    assert!(matches!(
        evaluate_tenant_admin_read(Some(&b), Some(&coextensive), &a, b.expires_at),
        Decision::Denied { .. }
    ));
    for change in 0..4 {
        let mut invalid_b = b.clone();
        let mut invalid_g = g.clone();
        match change {
            0 => invalid_g.valid_from = invalid_g.expires_at,
            1 => invalid_b.valid_from = invalid_b.expires_at,
            2 => invalid_b.valid_from = at() + tick,
            3 => invalid_b.expires_at = at(),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                evaluate_tenant_admin_read(Some(&invalid_b), Some(&invalid_g), &a, at()),
                Decision::Denied { .. }
            ),
            "change {change}"
        );
    }
    assert!(matches!(
        evaluate_tenant_admin_read(None, Some(&g), &a, at()),
        Decision::Denied { .. }
    ));
    assert!(matches!(
        evaluate_tenant_admin_read(Some(&b), None, &a, at()),
        Decision::Denied { .. }
    ));
}

#[test]
fn frozen_inspection_refuses_other_actions_targets_and_delegation() {
    for action in [
        GrantAction::ThreadCreate,
        GrantAction::ThreadCreateAuto,
        GrantAction::ThreadInspect,
        GrantAction::ThreadInvite,
        GrantAction::ThreadContribute,
        GrantAction::ThreadClose,
        GrantAction::ThreadCancel,
        GrantAction::ThreadInvitationRespond,
        GrantAction::ThreadAdvanceRound,
    ] {
        let (b, g, a) = fixture(action);
        assert!(
            matches!(
                evaluate_tenant_admin_read(Some(&b), Some(&g), &a, at()),
                Decision::Denied { .. }
            ),
            "{action}"
        );
    }
    for change in 0..3 {
        let (b, g, mut a) = fixture(GrantAction::TenantAdmin);
        match change {
            0 => {
                a.target = ResourceTarget::Thread {
                    tenant_id: g.tenant_id,
                    thread_id: ThreadId::new(),
                }
            }
            1 => a.delegate_subject = Some(g.subject.clone()),
            2 => a.delegation_scope = Some(TargetSelector::TenantWide),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                evaluate_tenant_admin_read(Some(&b), Some(&g), &a, at()),
                Decision::Denied { .. }
            ),
            "change {change}"
        );
    }
}
