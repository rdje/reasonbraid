//! The universal resource reference (`PHASE-4.1.2`, backlog 31): the §12.1
//! contract, typed at the boundary. The original locator is IMMUTABLE — no
//! verb updates it — and the ADR-011 `sha256:<hex>` scheme validates
//! `expected_digest`. Accepting a reference is NOT a promise the core can
//! resolve it (the explicit-failure doctrine: the resolution is the `.1.3`
//! registry's).
//!
//! **The identity of a reference is the `(original_locator, expected_digest)`
//! PAIR** (`SIGNOFF-REPAIR.11.14.3.2`,
//! `docs/decisions/2026-09-16_a-citation-registers-the-reference-it-names.md`)
//! — the key `migrations/0023_resource_references.sql:21` already declares, and
//! the key a contribution's `EvidenceRef { uri, digest }` already names. The
//! same pair is the REPLAY; a second digest for the same locator is a second
//! reference, because §12.6's live page changes and §12.1 forbids erasing that
//! security-relevant distinction. ⛔ The older `locator_digest_conflict`
//! refusal is retired: it was a §9.8 cross-tenant existence leak (it revealed a
//! pin the caller was never shown) and a cross-tenant denial (the first
//! principal to pin a locator made it uncitable by everyone else, in any form).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The durable `resource_references` row shape (the query's tuple type).
type ResourceRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
);

/// The typed §12.1 reference (deny-unknown-fields; the digest is the ADR-011
/// scheme).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResourceReference {
    pub original_locator: String,
    pub scheme: String,
    #[serde(default)]
    pub media_type_hint: Option<String>,
    /// `sha256:<64-hex>` — anything else is a typed refusal.
    #[serde(default)]
    pub expected_digest: Option<String>,
    #[serde(default)]
    pub fragment_or_selector: Option<String>,
    /// Opaque; never a secret.
    #[serde(default)]
    pub credential_binding_ref: Option<String>,
    #[serde(default)]
    pub owning_node_or_capability: Option<String>,
    #[serde(default)]
    pub visibility_scope: String,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub retention_class: Option<String>,
    #[serde(default)]
    pub risk_class: String,
}

impl ResourceReference {
    /// The ADR-011 digest validation: `sha256:<64 lowercase hex>` or absent.
    pub fn digest_error(&self) -> Option<&'static str> {
        match &self.expected_digest {
            None => None,
            Some(digest) => match digest.strip_prefix("sha256:") {
                None => Some("the expected digest must be `sha256:<64 hex>` (ADR-011)"),
                Some(rest) if rest.len() == 64 && rest.chars().all(|c| c.is_ascii_hexdigit()) => {
                    None
                }
                Some(_) => Some("the expected digest must be `sha256:<64 hex>` (ADR-011)"),
            },
        }
    }
}

/// The URI's scheme, per RFC 3986 §3.1: `ALPHA *( ALPHA / DIGIT / "+" / "-" /
/// "." )` terminated by `:`. Returned VERBATIM — §12.1 keeps canonicalization
/// separate and scheme-specific, so nothing is lower-cased here.
///
/// A contribution cites evidence as a bare URI (`SIGNOFF-REPAIR.11.14.3.2`),
/// and the §12.1 reference it registers needs a scheme. Deriving it from the
/// locator is what stops the two from disagreeing. ⚠️ `POST /v1/resources`
/// still takes `scheme` as an unvalidated caller field — owned by
/// `SIGNOFF-REPAIR.11.14.3.5`, not by this function.
pub fn scheme_of(uri: &str) -> Option<&str> {
    let (scheme, _) = uri.split_once(':')?;
    let mut characters = scheme.chars();
    if !characters.next()?.is_ascii_alphabetic() {
        return None;
    }
    if !characters.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')) {
        return None;
    }
    Some(scheme)
}

/// The submission outcome: a fresh reference, or a replay (the same locator +
/// the same digest — the pair the unique index is built on).
#[derive(Debug, Clone, Serialize)]
pub struct SubmitOutcome {
    pub resource_id: String,
    pub replayed: bool,
}

/// Who registered a reference — the tenant the DETAIL READ is bound to, plus
/// the actor handle recorded for the audit trail (`SIGNOFF-REPAIR.11.14.3.4`).
///
/// ⭐ A registration is a SET rather than a column, for the reason
/// `migrations/0067` records: `UNIQUE (original_locator, expected_digest)` makes
/// one row serve every tenant that names the pair, so the tenant belongs to the
/// disclosure DECISION rather than to the row. This is [`crate::snapshots::Citer`]'s
/// shape, applied to the other content-addressed table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registrant {
    /// The registering tenant — the detail read compares against this.
    pub tenant_id: String,
    /// The actor handle that registered it. An audit breadcrumb; the
    /// disclosure decision is the TENANT's.
    pub principal: String,
}

/// The reference store's refusals — every one names its reason.
// Only `Debug` derives: the storage variant carries the original SQLx error so
// it survives to `std::error::Error::source`, and that error is neither `Clone`
// nor `Eq`. This matches the `AssessmentError` contract from `.11.14.3.1`.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReferenceError {
    /// The digest is outside the ADR-011 scheme.
    InvalidDigest(&'static str),
    /// The store itself failed. A database fault does not prove anything about
    /// the caller's input and must never be reported as though it did
    /// (`SIGNOFF-REPAIR.7.4.2`).
    Storage(sqlx::Error),
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(_) => write!(f, "the reference store is unavailable"),
            Self::InvalidDigest(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for ReferenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(cause) => Some(cause),
            Self::InvalidDigest(_) => None,
        }
    }
}

/// Submit the reference (the caller is the enrolled principal; the handler
/// passes the actor handle).
///
/// The replay is keyed on the `(original_locator, expected_digest)` pair, which
/// is the table's own unique key — `NULLS NOT DISTINCT` since
/// `migrations/0065`, so an unpinned citation replays too rather than inserting
/// a second unpinned row. A concurrent insert of the same pair loses the
/// insert's `ON CONFLICT`, which IS the replay observed from the other side:
/// the row the winner wrote is re-read and returned rather than reported as a
/// fault.
///
/// Generic over the executor for the same reason [`crate::claims::submit`] is:
/// a contribution's citation registers its reference inside the thread's own
/// transaction, so the contribution event and the reference row commit together
/// or not at all (`SIGNOFF-REPAIR.11.14.3.2`).
pub async fn submit<'e, E>(
    mut executor: E,
    reference: &ResourceReference,
    submitted_by: &str,
    registrant: &Registrant,
) -> Result<SubmitOutcome, ReferenceError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    if let Some(reason) = reference.digest_error() {
        return Err(ReferenceError::InvalidDigest(reason));
    }
    // `IS NOT DISTINCT FROM` rather than `=`, because the unpinned reference's
    // digest is NULL and `NULL = NULL` is unknown. The `::text` cast is not
    // decoration: a bare parameter under this operator gives the planner nothing
    // to infer the type from.
    const FIND_PAIR: &str = "SELECT resource_id FROM resource_references \
         WHERE original_locator = $1 AND expected_digest IS NOT DISTINCT FROM $2::text \
         LIMIT 1";
    let existing: Option<String> = sqlx::query_scalar(FIND_PAIR)
        .bind(&reference.original_locator)
        .bind(&reference.expected_digest)
        .fetch_optional(&mut *executor)
        .await
        .map_err(ReferenceError::Storage)?;
    if let Some(resource_id) = existing {
        record_registration(&mut *executor, &resource_id, registrant).await?;
        return Ok(SubmitOutcome {
            resource_id,
            replayed: true,
        });
    }

    // ⛔ `ON CONFLICT DO NOTHING` rather than a caught unique violation: this
    // runs inside the caller's transaction (a contribution registers its
    // citations there), and a raised violation would abort that transaction,
    // so the re-read after it could never run.
    let inserted: Option<String> = sqlx::query_scalar(
        "INSERT INTO resource_references \
         (resource_id, original_locator, scheme, media_type_hint, expected_digest, \
          fragment_or_selector, credential_binding_ref, owning_node_or_capability, \
          visibility_scope, purpose, retention_class, risk_class, submitted_by) \
         VALUES ('res_' || gen_random_uuid()::text, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
         ON CONFLICT (original_locator, expected_digest) DO NOTHING \
         RETURNING resource_id",
    )
    .bind(&reference.original_locator)
    .bind(&reference.scheme)
    .bind(&reference.media_type_hint)
    .bind(&reference.expected_digest)
    .bind(&reference.fragment_or_selector)
    .bind(&reference.credential_binding_ref)
    .bind(&reference.owning_node_or_capability)
    .bind(&reference.visibility_scope)
    .bind(&reference.purpose)
    .bind(&reference.retention_class)
    .bind(&reference.risk_class)
    .bind(submitted_by)
    .fetch_optional(&mut *executor)
    .await
    .map_err(ReferenceError::Storage)?;
    if let Some(resource_id) = inserted {
        record_registration(&mut *executor, &resource_id, registrant).await?;
        return Ok(SubmitOutcome {
            resource_id,
            replayed: false,
        });
    }
    // The pre-check and the insert are not atomic, so two callers can both pass
    // it. The index settles the race, and losing it means the other caller
    // wrote exactly the row this one wanted: the replay.
    let resource_id: String = sqlx::query_scalar(FIND_PAIR)
        .bind(&reference.original_locator)
        .bind(&reference.expected_digest)
        .fetch_one(&mut *executor)
        .await
        .map_err(ReferenceError::Storage)?;
    record_registration(&mut *executor, &resource_id, registrant).await?;
    Ok(SubmitOutcome {
        resource_id,
        replayed: true,
    })
}

/// Record one registration — idempotent, so a re-registration by the same
/// tenant keeps the original time and actor.
///
/// ⛔ Written on the REPLAY as well as on the insert. Without that, the first
/// tenant to name a pair would own the row's read for ever and every other
/// tenant citing the same URL would be refused its own reference — the
/// cross-tenant denial `SIGNOFF-REPAIR.11.14.3.2` retired `locator_digest_conflict`
/// to remove, reintroduced one layer down.
async fn record_registration<'e, E>(
    mut executor: E,
    resource_id: &str,
    registrant: &Registrant,
) -> Result<(), ReferenceError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO reference_registrations (resource_id, tenant_id, registered_by) \
         VALUES ($1, $2, $3) ON CONFLICT (resource_id, tenant_id) DO NOTHING",
    )
    .bind(resource_id)
    .bind(&registrant.tenant_id)
    .bind(&registrant.principal)
    .execute(&mut *executor)
    .await
    .map_err(ReferenceError::Storage)?;
    Ok(())
}

/// Read one reference for a tenant that REGISTERED it — the detail read's gate
/// (`SIGNOFF-REPAIR.11.14.3.4`).
///
/// ⛔ There is deliberately no unbound read in this module. A caller cannot ask
/// for a reference without naming the tenant the answer is for, which is the
/// property `crate::snapshots` already holds for the row this one points at.
pub async fn get_for_tenant(
    pool: &PgPool,
    resource_id: &str,
    tenant_id: &str,
) -> Result<
    Option<(
        String,
        ResourceReference,
        String,
        chrono::DateTime<chrono::Utc>,
    )>,
    sqlx::Error,
> {
    let registered: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM reference_registrations \
         WHERE resource_id = $1 AND tenant_id = $2)",
    )
    .bind(resource_id)
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;
    if !registered {
        return Ok(None);
    }
    get(pool, resource_id).await
}

/// Read one reference, UNBOUND. Private to this module since
/// `SIGNOFF-REPAIR.11.14.3.4`: every caller outside it goes through
/// [`get_for_tenant`].
async fn get(
    pool: &PgPool,
    resource_id: &str,
) -> Result<
    Option<(
        String,
        ResourceReference,
        String,
        chrono::DateTime<chrono::Utc>,
    )>,
    sqlx::Error,
> {
    let row: Option<ResourceRow> = sqlx::query_as(
        "SELECT resource_id, original_locator, scheme, media_type_hint, expected_digest, \
                fragment_or_selector, credential_binding_ref, owning_node_or_capability, \
                visibility_scope, purpose, retention_class, risk_class, submitted_by, created_at \
         FROM resource_references WHERE resource_id = $1",
    )
    .bind(resource_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(
            resource_id,
            original_locator,
            scheme,
            media_type_hint,
            expected_digest,
            fragment_or_selector,
            credential_binding_ref,
            owning_node_or_capability,
            visibility_scope,
            purpose,
            retention_class,
            risk_class,
            submitted_by,
            created_at,
        )| {
            (
                resource_id,
                ResourceReference {
                    original_locator,
                    scheme,
                    media_type_hint,
                    expected_digest,
                    fragment_or_selector,
                    credential_binding_ref,
                    owning_node_or_capability,
                    visibility_scope,
                    purpose,
                    retention_class,
                    risk_class,
                },
                submitted_by,
                created_at,
            )
        },
    ))
}

/// The `submitted_by` handle for a principal id, in the SAME shape
/// `api::submit_resource` writes ([`reasonbraid_core::actor_handle_for_subject`])
/// — so the column stays ONE namespace whichever writer filled it
/// (`SIGNOFF-REPAIR.11.14.3.2`).
///
/// ⚠️ The handle is a one-way `Uuid::new_v5` that joins to no identity table,
/// and the locator replay leaves it naming the FIRST citer whatever happens
/// afterwards — both measured by
/// `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`.
/// It is an audit breadcrumb, never an authorization input. An id that parses
/// as neither principal shape is recorded verbatim rather than dropped; the
/// HTTP layer has already refused one before a command reaches the domain.
pub fn actor_handle(principal: &str) -> String {
    if let Ok(human) = principal.parse::<reasonbraid_core::HumanPrincipalId>() {
        return reasonbraid_core::actor_handle_for_subject(&reasonbraid_core::GrantSubject::Human(
            human,
        ))
        .to_string();
    }
    if let Ok(role) = principal.parse::<reasonbraid_core::AgentRoleId>() {
        return reasonbraid_core::actor_handle_for_subject(&reasonbraid_core::GrantSubject::Role(
            role,
        ))
        .to_string();
    }
    principal.to_owned()
}
