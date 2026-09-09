//! Version-two local recovery data. These records preserve request provenance;
//! they are not credentials or independent proof of server authority.

use serde::{Deserialize, Deserializer, Serialize};

/// At most one unresolved request and one most recent completed outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapRecovery {
    #[serde(deserialize_with = "present_option")]
    pub pending: Option<BootstrapRequest>,
    #[serde(deserialize_with = "present_option")]
    pub completed: Option<CompletedBootstrap>,
}

/// The exact saved new-human request. Actions are retained for stable resends,
/// even though the current server deliberately ignores human action input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapRequest {
    pub request_id: String,
    pub server: String,
    pub name: String,
    #[serde(deserialize_with = "present_option")]
    pub actions: Option<Vec<String>>,
}

/// A retained request and its fully checked historical response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedBootstrap {
    pub request: BootstrapRequest,
    pub outcome: BootstrapOutcome,
}

/// The complete keyed enrollment response; no source identifier is synthesized
/// when reading a record. Validate through StateFile load/save before using it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapOutcome {
    pub bootstrap_request_id: String,
    pub kind: String,
    pub name: String,
    pub principal_id: String,
    pub tenant_id: String,
    pub boundary_id: String,
    pub grant_id: String,
    pub replayed: bool,
}

// Null is an explicit absence. Omitting a required nullable field is malformed
// durable data, not permission to guess that no request or action list existed.
fn present_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) use validation::{validate, validate_shape, validate_transition};

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod validation {
    use std::fmt::Display;
    use std::str::FromStr;

    use reasonbraid_core::{HumanPrincipalId, RequestId, TenantId};
    use serde_json::Value;

    use super::{BootstrapOutcome, BootstrapRecovery, BootstrapRequest};
    use crate::{CliError, StateFile};

    fn invalid(message: &str) -> CliError {
        CliError::state(message.to_owned())
    }

    fn canonical<T: FromStr + Display>(raw: &str) -> bool {
        raw.parse::<T>().is_ok_and(|id| id.to_string() == raw)
    }

    fn canonical_server(raw: &str) -> Result<String, CliError> {
        if raw.len() > 4096 || raw.trim() != raw {
            return Err(invalid(
                "bootstrap server identity is oversized or contains surrounding whitespace",
            ));
        }
        let url = reqwest::Url::parse(raw)
            .map_err(|_| invalid("bootstrap server identity is not an absolute HTTP(S) URL"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid("bootstrap server identity requires HTTP(S) without URL credentials, query or fragment"));
        }
        let canonical = url.as_str().trim_end_matches('/');
        if canonical.len() > 4096 {
            return Err(invalid("bootstrap server identity exceeds 4096 bytes"));
        }
        Ok(canonical.to_owned())
    }

    fn request(request: &BootstrapRequest) -> Result<(), CliError> {
        let uuid = request
            .request_id
            .strip_prefix("req_")
            .and_then(|raw| uuid::Uuid::parse_str(raw).ok());
        if !canonical::<RequestId>(&request.request_id)
            || !uuid.is_some_and(|id| {
                id.get_version_num() == 7 && id.get_variant() == uuid::Variant::RFC4122
            })
        {
            return Err(invalid(
                "bootstrap request identity must be a canonical req_ UUIDv7",
            ));
        }
        if canonical_server(&request.server)? != request.server {
            return Err(invalid("bootstrap server identity is not canonical"));
        }
        Ok(())
    }

    fn outcome(value: &BootstrapOutcome, binding: &BootstrapRequest) -> Result<(), CliError> {
        if value.bootstrap_request_id != binding.request_id
            || value.kind != "human"
            || value.name != binding.name
            || !canonical::<HumanPrincipalId>(&value.principal_id)
            || !canonical::<TenantId>(&value.tenant_id)
            || value.boundary_id != format!("bnd_{}", value.tenant_id)
            || value.grant_id != format!("grt_{}", value.principal_id)
        {
            return Err(invalid(
                "bootstrap outcome does not match its request or source identities",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate(recovery: &BootstrapRecovery) -> Result<(), CliError> {
        if recovery.pending.is_none() && recovery.completed.is_none() {
            return Err(invalid(
                "bootstrap recovery needs a pending or completed request",
            ));
        }
        if let Some(pending) = &recovery.pending {
            request(pending)?;
        }
        if let Some(completed) = &recovery.completed {
            request(&completed.request)?;
            outcome(&completed.outcome, &completed.request)?;
            if let Some(pending) = &recovery.pending {
                if pending.request_id == completed.request.request_id
                    && pending != &completed.request
                {
                    return Err(invalid(
                        "one bootstrap identity has conflicting saved request bindings",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_shape(root: &Value, version: u32) -> Result<(), CliError> {
        if version < 2 {
            return if root.get("bootstrap").is_some() {
                Err(invalid(
                    "bootstrap recovery requires local state version two",
                ))
            } else {
                Ok(())
            };
        }
        let recovery = &root["bootstrap"];
        let pending = &recovery["pending"];
        let completed = &recovery["completed"];
        if !recovery.is_object()
            || (!pending.is_null() && !pending.is_object())
            || (!completed.is_null()
                && (!completed.is_object()
                    || !completed["request"].is_object()
                    || !completed["outcome"].is_object()))
        {
            return Err(invalid(
                "bootstrap recovery and its records must be JSON objects",
            ));
        }
        Ok(())
    }

    /// Preserve unresolved intent against a stale full-snapshot writer. New
    /// snapshots may restore valid historical data into a legacy/empty store;
    /// once recovery exists, publication cannot silently discard its identity.
    pub(crate) fn validate_transition(old: &StateFile, new: &StateFile) -> Result<(), CliError> {
        let Some(previous) = &old.bootstrap else {
            return Ok(());
        };
        let next = new.bootstrap.as_ref().ok_or_else(|| {
            invalid("local publication cannot discard bootstrap recovery metadata")
        })?;
        let Some(pending) = &previous.pending else {
            return if previous.completed == next.completed {
                Ok(())
            } else {
                Err(invalid(
                    "a completed bootstrap receipt can change only through its pending request",
                ))
            };
        };
        if next
            .pending
            .as_ref()
            .is_some_and(|request| request != pending)
        {
            return Err(invalid(
                "local publication cannot replace an unresolved bootstrap request",
            ));
        }
        if previous.completed != next.completed || next.pending.is_none() {
            let completed = next
                .completed
                .as_ref()
                .filter(|done| &done.request == pending)
                .ok_or_else(|| {
                    invalid("pending bootstrap cleanup requires its matching completed receipt")
                })?;
            let principal = new.principals.get(&pending.name);
            if !principal.is_some_and(|principal| {
                principal.kind == "human"
                    && principal.id == completed.outcome.principal_id
                    && principal.tenant == completed.outcome.tenant_id
            }) {
                return Err(invalid(
                    "bootstrap completion requires its published principal mapping",
                ));
            }
        }
        Ok(())
    }
}
