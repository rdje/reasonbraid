//! The portable agent cards (`PHASE-8.1.3`, ADR-026/027): the
//! digest-pinned portable form of the §10.1 profile + the capability
//! declaration. The export is the canonical card + the `sha256:<hex>`
//! digest; the import runs the ADR-027 verification ladder — the digest
//! (the re-derivation), the compatibility (the schema version), the
//! allowlist (the EFFECTIVE federation agreement — the `.1.2` pairing),
//! and the capability rung (the LOCAL grant under the importing
//! boundary — the card's self-asserted capabilities never confer
//! authority; the local grant is the only authority that acts).

use serde::{Deserialize, Serialize};

/// The card's schema version (the compatibility rung).
pub const CARD_SCHEMA_VERSION: &str = "agent-card/1";

/// The portable card: the origin identity + the §10.1 profile + the
/// capability declaration (the profile's capabilities). The canonical
/// form is the struct's field order — the byte-identical regeneration
/// contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentCard {
    pub schema_version: String,
    pub origin_tenant_id: String,
    pub origin_role_id: String,
    pub profile: crate::profiles::AgentProfile,
    pub exported_at: String,
}

/// A card refusal — the typed rung.
#[derive(Debug)]
pub enum CardError {
    /// The presented digest does not match the card's canonical bytes.
    DigestMismatch { presented: String, computed: String },
    /// The schema version is outside the known set.
    UnsupportedSchema { got: String },
    /// No EFFECTIVE federation agreement with the origin tenant (the
    /// allowlist rung — the `.1.2` pairing).
    NoAgreement { origin_tenant: String },
    /// The storage layer failed (not a card verdict).
    Storage(sqlx::Error),
}

impl std::fmt::Display for CardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardError::DigestMismatch {
                presented,
                computed,
            } => write!(
                f,
                "the card digest does not match its bytes ({presented} != {computed})"
            ),
            CardError::UnsupportedSchema { got } => {
                write!(f, "the card schema `{got}` is outside {CARD_SCHEMA_VERSION}")
            }
            CardError::NoAgreement { origin_tenant } => write!(
                f,
                "no effective federation agreement with the origin tenant `{origin_tenant}` — the import refuses"
            ),
            CardError::Storage(e) => write!(f, "card storage failure: {e}"),
        }
    }
}

impl std::error::Error for CardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CardError::Storage(e) => Some(e),
            _ => None,
        }
    }
}

/// The canonical bytes (the byte-identical regeneration contract).
pub fn canonical_bytes(card: &AgentCard) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(card)
}

/// The ADR-011 digest over the canonical bytes.
pub fn digest_of(card: &AgentCard) -> Result<String, serde_json::Error> {
    use sha2::Digest;
    let bytes = canonical_bytes(card)?;
    Ok(format!(
        "sha256:{}",
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

/// The pure rungs (the digest + the compatibility) — the import handler
/// runs these BEFORE any storage, then the agreement + the grant rungs.
pub fn verify_pure_rungs(card: &AgentCard, presented_digest: &str) -> Result<(), CardError> {
    if card.schema_version != CARD_SCHEMA_VERSION {
        return Err(CardError::UnsupportedSchema {
            got: card.schema_version.clone(),
        });
    }
    let computed = digest_of(card).map_err(|e| {
        CardError::Storage(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        ))))
    })?;
    if computed != presented_digest {
        return Err(CardError::DigestMismatch {
            presented: presented_digest.to_string(),
            computed,
        });
    }
    Ok(())
}
