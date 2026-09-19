//! Registering a policy version as a site act (`SIGNOFF-REPAIR.6.1.5.4`).
//!
//! `POST /v1/policies` admitted any enrolled principal into a FIRST-COME
//! identifier namespace. `policy_versions` is keyed `PRIMARY KEY (policy_id,
//! version)` with no tenant column, so the first caller to name a coordinate
//! owns it and every later caller is refused as a duplicate — and what the
//! other tenants then read under that id is the first caller's text.
//!
//! ⚠️ Stated at its real width, because the leaf must not inflate it. Unlike
//! [`crate::workflows::resolve`], [`crate::policy::resolve`] names an EXPLICIT
//! `(policy_id, version)` pair, so a foreign registration does not silently
//! re-shape a deliberation the way a foreign workflow profile did. What it does
//! is take a coordinate the rightful author then cannot use, and put text under
//! a governance id that every enrolled principal reads through
//! `GET /v1/policies`, the impact map and the MCP policy bundle. It was
//! reproduced at runtime in all four of those legs before this module existed.
//!
//! The library is site-wide BY DESIGN and its ownership model is a GRANT, not a
//! tenant (`docs/decisions/2026-09-19_the-policy-library-is-shared-the-lifecycle-is-its-tenants.md`).
//! So the repair is never a tenant column; it is the authority the write always
//! needed, in the shape `docs/decisions/2026-09-09_site-operator-authority.md`
//! defines for every site-wide act. That record also required this registry and
//! the workflow registry to be decided consistently, which is why this file is
//! deliberately [`super::workflows`] with one subject changed.
//!
//! ⛔ The READS stay on enrolment. A policy only its author can read is not
//! governance, `SIGNOFF-REPAIR.6.1.5.3`'s control asserts it, and the defect was
//! only ever the write.

use super::*;

/// Authorize, register and audit as one ordered site transaction.
///
/// A denial commits its own audit record; an audit failure rolls back an
/// otherwise applied registration.
///
/// ⚠️ The document validation runs in the HTTP layer, BEFORE this call, and
/// stays a typed 400 rather than becoming a site refusal — the same order, for
/// the same two reasons, that [`super::workflows::register_profile`] states.
/// [`crate::policy::validate`] asks only about the SUBMISSION: the ADR-011
/// digest shape, the semantic version, the published [`crate::policy::LIFECYCLES`]
/// vocabulary, a nonempty clause list and clause ids that do not repeat. Each is
/// a rule over a published constant, so naming the one that failed is an oracle
/// over nothing, and a caller that passes it still meets the gate.
///
/// ⛔ The two STATEFUL refusals deliberately do NOT move out with it, and the
/// difference is the point: whether `owning_authority` is a live grant and
/// whether `(policy_id, version)` is already taken are questions about the
/// DATABASE, and answering either one before the gate would hand a principal
/// with no site authority an existence oracle over the site's grants and over a
/// registry it may not write. They stay inside, as DOMAIN refusals — audited
/// `denied` WITH the grant and boundary attached, because the caller did hold
/// the authority — and surface as 403 with an audit id rather than the 400 they
/// used to be. That status change is a wire change and is named in
/// `docs/book/src/site-authority.md`.
///
/// [`crate::policy::register`] re-validates regardless: the registry never
/// stores an invalid document, whatever called it.
pub async fn register_policy(
    pool: &PgPool,
    subject: &GrantSubject,
    input: &crate::policy::PolicyVersionInput,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let input = input.clone();
    authorized(
        pool,
        subject,
        Action::PolicyRegister,
        json!({
            "registry": "policy_versions",
            "policy_id": input.policy_id,
            "version": input.version,
        }),
        reason.as_str(),
        move |conn, _at| {
            let input = input.clone();
            Box::pin(async move {
                use crate::policy::PolicyError;
                // ⛔ The `?` is the OUTER result: a database that could not
                // answer is not a refusal and must not be audited as one — the
                // act rolls back with nothing recorded, because nothing was
                // decided. Only the inner `Err` is a refusal.
                Ok(match crate::policy::register(conn, &input).await? {
                    Ok(registered) => Ok(Effect::write(
                        serde_json::to_value(&registered).expect("the policy serializes"),
                        1,
                    )),
                    // ⛔ The two domain refusals are told apart in the audit
                    // record. "the version exists" and "the named authority is
                    // not live" are different operator errors, and one opaque
                    // reason would leave the trail unable to say which happened.
                    Err(PolicyError::GhostAuthority(_)) => {
                        Err("the named owning authority is not an active, unexpired grant")
                    }
                    Err(PolicyError::Duplicate(_)) => {
                        Err("that policy version is already registered")
                    }
                    // The five submission rules cannot reach here — the HTTP
                    // layer ran them before the gate and `register` re-ran them
                    // before the write — but a reason is recorded rather than
                    // assumed away.
                    Err(_) => Err("the submitted document is not a valid policy version"),
                })
            })
        },
    )
    .await
}
