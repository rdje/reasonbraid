//! Registering a resolver as a site act (`SIGNOFF-REPAIR.7.1.3.1`).
//!
//! `POST /v1/resolvers` admitted any tenant ADMINISTRATOR and wrote a row into
//! `resolver_capabilities`, which carries no tenant column and which every
//! tenant's resolution ranks. Before `.7.1.3` a fast-advertised row the server
//! cannot execute silenced every tenant's acquisition; since then it is skipped
//! and named, so a tenant's row could no longer do anything except appear in
//! every other tenant's answer. Registering a resolver describes what this
//! SERVER can acquire through, which is site configuration: it takes the shape
//! `docs/decisions/2026-09-09_site-operator-authority.md` defines for every
//! site-wide act, as `workflow_register` did for the workflow registry.

use super::*;

/// Authorize, register and audit as one ordered site transaction.
///
/// The insert is INSERT-ONLY (`ON CONFLICT DO NOTHING`) inside the act, so two
/// concurrent registrations of one new id cannot replace each other, which a
/// pre-check followed by the upsert would allow: an id that already exists is a
/// DOMAIN refusal, audited `denied` with the grant attached, because the caller
/// did hold the authority. Replacing an advertisement stays the product's own
/// boot-time verb (`resolvers::register`), never this one (`.7.3.6.2`).
pub async fn register_resolver(
    pool: &PgPool,
    subject: &GrantSubject,
    advertise: &crate::resolvers::ResolverAdvertise,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let advertise = advertise.clone();
    let resolver_id = advertise.resolver_id.clone();
    authorized(
        pool,
        subject,
        Action::ResolverRegister,
        json!({ "registry": "resolver_capabilities", "resolver_id": resolver_id }),
        reason.as_str(),
        move |conn, _at| {
            let advertise = advertise.clone();
            Box::pin(async move {
                match crate::resolvers::insert_new(conn, &advertise).await {
                    Ok(true) => Ok(Ok(Effect::write(
                        json!({ "resolver_id": advertise.resolver_id, "registered": true }),
                        1,
                    ))),
                    Ok(false) => Ok(Err(
                        "the resolver is already registered; this verb registers a new \
                         resolver and does not replace an existing advertise",
                    )),
                    Err(_) => Ok(Err("the resolver was not registered")),
                }
            })
        },
    )
    .await
}
