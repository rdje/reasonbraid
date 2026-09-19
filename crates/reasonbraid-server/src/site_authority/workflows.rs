//! Registering a workflow profile as a site act (`SIGNOFF-REPAIR.7.1.2.1`).
//!
//! `POST /v1/workflow-profiles` admitted any enrolled principal and appended
//! `MAX(version) + 1` under ANY `profile_id`, the eight §13.1 built-ins
//! included, while [`crate::workflows::resolve`] selects `ORDER BY version DESC
//! LIMIT 1` site-wide with no `built_in` filter and no tenant predicate — and
//! `create_thread` / `create_thread_auto` resolve `DEFAULT_PROFILE_ID =
//! "quick_advice"` through it. So one enrolled principal chose the steps every
//! other tenant's next bare thread executed.
//!
//! ⚠️ Not a capability escape: [`crate::workflows::validate_steps`] admits only
//! the thirteen step kinds and requires a terminal last, so the vocabulary is
//! closed. It is a control-plane OVERRIDE — the attacker chooses the
//! deliberation SHAPE, and can drop `blind_solicit` or `critique` from every
//! tenant's default.
//!
//! `workflow_profiles` is keyed `(profile_id, version)` with no tenant column,
//! and its rows are the arms `routing_rules` names. Registering one is site-wide
//! configuration, which the module already said in its own words — the verb's
//! doc comment reads "the operator's verb" — and never enforced. It now takes
//! the shape `docs/decisions/2026-09-09_site-operator-authority.md` defines for
//! every site-wide act.
//!
//! ⛔ This does NOT decide that a workflow profile could not instead belong to a
//! tenant. That is the question `SIGNOFF-REPAIR.6.1.5` owns for the policy
//! registry, and a second registry must not answer it unilaterally.

use super::*;

/// Authorize, register and audit as one ordered site transaction.
///
/// A denial commits its own audit record; an audit failure rolls back an
/// otherwise applied registration.
///
/// ⚠️ The composition validation runs in the HTTP layer, BEFORE this call, and
/// stays a typed 400 rather than becoming a site refusal. Two reasons, both
/// stated so a later reader does not "fix" the order: the ADR-016 vocabulary is
/// a published constant (`STEP_KINDS`, and the book lists all thirteen), so
/// naming the rule that failed is an oracle over nothing; and every other site
/// route already parses and bounds its input before authorizing, because
/// `site_request` runs on extraction. A caller that fails validation learns
/// nothing about authority, and one that passes it still meets the gate.
///
/// [`crate::workflows::register`] re-validates regardless — the registry never
/// stores an invalid profile, whatever called it — and a failure there is a
/// DOMAIN refusal: audited `denied` WITH the grant and boundary attached,
/// because the caller did hold the authority.
pub async fn register_profile(
    pool: &PgPool,
    subject: &GrantSubject,
    profile_id: &str,
    steps: &[String],
    reason: &Reason,
) -> Result<Receipt, Error> {
    let profile_id = profile_id.to_owned();
    let steps = steps.to_vec();
    authorized(
        pool,
        subject,
        Action::WorkflowRegister,
        json!({ "registry": "workflow_profiles", "profile_id": profile_id }),
        reason.as_str(),
        move |conn, _at| {
            let profile_id = profile_id.clone();
            let steps = steps.clone();
            Box::pin(async move {
                match crate::workflows::register(conn, &profile_id, &steps).await {
                    Ok(resolved) => Ok(Ok(Effect::write(
                        serde_json::to_value(&resolved).expect("the profile serializes"),
                        1,
                    ))),
                    Err(_) => Ok(Err("the workflow profile was not registered")),
                }
            })
        },
    )
    .await
}
