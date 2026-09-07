//! The universal resource reference (`PHASE-4.1.2`, backlog 31): the §12.1
//! contract, typed at the boundary. The original locator is IMMUTABLE — the
//! update-refusal is the absent verb plus the same-locator/different-digest
//! conflict (the ADR-011 `sha256:<hex>` scheme validates `expected_digest`).
//! Accepting a reference is NOT a promise the core can resolve it (the
//! explicit-failure doctrine: the resolution is the `.1.3` registry's).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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

/// The submission outcome: a fresh reference, a replay (the same locator +
/// digest), or a conflict (the same locator with a DIFFERENT digest — the
/// immutability of the locator's binding).
#[derive(Debug, Clone, Serialize)]
pub struct SubmitOutcome {
    pub resource_id: String,
    pub replayed: bool,
}

/// Submit the reference (the caller is the enrolled principal; the handler
/// passes the actor handle).
pub async fn submit(
    pool: &PgPool,
    reference: &ResourceReference,
    submitted_by: &str,
) -> Result<SubmitOutcome, sqlx::Error> {
    // The replay/conflict decision BEFORE the insert (the unique index backs
    // it): the same locator + digest → the replay; the same locator with a
    // different digest → the typed conflict (surfaced by the caller).
    let existing: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT resource_id, expected_digest FROM resource_references \
         WHERE original_locator = $1 ORDER BY created_at LIMIT 1",
    )
    .bind(&reference.original_locator)
    .fetch_optional(pool)
    .await?;
    if let Some((resource_id, stored_digest)) = existing {
        if stored_digest == reference.expected_digest {
            return Ok(SubmitOutcome {
                resource_id,
                replayed: true,
            });
        }
        // The locator's digest is immutable — the caller maps this to the
        // typed conflict.
        return Err(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "locator_digest_conflict",
        ))));
    }

    let (resource_id,): (String,) = sqlx::query_as(
        "INSERT INTO resource_references \
         (resource_id, original_locator, scheme, media_type_hint, expected_digest, \
          fragment_or_selector, credential_binding_ref, owning_node_or_capability, \
          visibility_scope, purpose, retention_class, risk_class, submitted_by) \
         VALUES ('res_' || gen_random_uuid()::text, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
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
    .fetch_one(pool)
    .await?;
    Ok(SubmitOutcome {
        resource_id,
        replayed: false,
    })
}

/// Read one reference (the inspection).
pub async fn get(
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
    let row: Option<(
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
    )> = sqlx::query_as(
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
