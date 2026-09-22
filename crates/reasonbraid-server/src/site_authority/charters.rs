//! Registering a governance charter as a site act
//! (`SIGNOFF-REPAIR.11.4.7.2.1.2.1`).
//!
//! ⭐ **The tenant must not define the charter, and the reason is structural
//! rather than cautious.** ROADMAP §4.1 makes the charter the document that
//! constrains a tenant — its *allowed decision rules and approval thresholds*,
//! its *separation-of-duties and conflict-of-interest constraints*. §4.4 makes
//! the enrollment boundary that NAMES the charter digest a root/parent-granted
//! ceiling. A tenant that could rewrite its own allowed decision rules would
//! hold the ceiling it is bound by, which is the escalation the boundary exists
//! to prevent.
//!
//! So registration takes the shape
//! `docs/decisions/2026-09-09_site-operator-authority.md` defines for every
//! act whose subject is the site's own governance configuration — the same
//! shape `workflows::register_profile`, `policies::register_policy` and the
//! `evaluation` family already carry.
//!
//! ⚠️ The vocabulary and threshold validation runs in the HTTP layer BEFORE
//! this call and stays a typed 400, on the `SIGNOFF-REPAIR.8.2.5.2` precedent:
//! §13.3's seven families are a published constant that the book lists, so
//! naming the rule that failed is an oracle over nothing, and a caller that
//! fails validation learns nothing about authority. [`crate::charters::register`]
//! re-validates regardless — the store never holds an invalid charter, whatever
//! called it.

use super::*;

/// Authorize, register and audit as one ordered site transaction.
///
/// A denial commits its own audit record; an audit failure rolls back an
/// otherwise applied registration.
pub async fn register_charter(
    pool: &PgPool,
    subject: &GrantSubject,
    input: &crate::charters::CharterInput,
) -> Result<Receipt, Error> {
    let input = input.clone();
    let tenant_id = input.tenant_id.clone();
    let reason = input.reason.as_str().to_owned();
    authorized(
        pool,
        subject,
        Action::CharterRegister,
        json!({ "registry": "governance_charters", "tenant_id": tenant_id }),
        &reason,
        move |conn, _at| {
            let input = input.clone();
            Box::pin(async move {
                match crate::charters::register(conn, &input).await {
                    Ok(stored) => Ok(Ok(Effect::write(
                        serde_json::to_value(&stored).expect("the charter serializes"),
                        1,
                    ))),
                    // ⛔ A FIXED reason, like `workflows::register_profile`'s.
                    // The HTTP layer has already named the specific vocabulary
                    // or threshold failure as a 400, so this arm fires only on
                    // a store fault or a re-validation disagreement — and the
                    // audit record's job is to say the effect did not apply,
                    // not to re-render a message the caller already received.
                    Err(_) => Ok(Err("the governance charter was not registered")),
                }
            })
        },
    )
    .await
}
