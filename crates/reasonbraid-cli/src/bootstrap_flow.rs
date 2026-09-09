//! One guarded bootstrap intent, from durable dispatch identity to recoverable
//! output. A retained completion is historical data, not live server authority.

use reasonbraid_core::{HumanPrincipalId, RequestId, TenantId};

use crate::bootstrap_state::{canonical_server, validate_outcome};
use crate::state_store::Writer;
use crate::{
    ApiClient, BootstrapOutcome, BootstrapRecovery, BootstrapRequest, CliError, CompletedBootstrap,
    Config, StateFile, StoredPrincipal,
};

pub(crate) fn decode_outcome(
    bytes: &[u8],
    request: &BootstrapRequest,
) -> Result<BootstrapOutcome, CliError> {
    let invalid = || CliError::Malformed("expected a complete matching bootstrap outcome".into());
    // Derived struct deserialization also accepts sequences. Require an object,
    // then decode the original bytes so duplicate fields cannot be normalized away.
    if bytes.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
        return Err(invalid());
    }
    let outcome = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    validate_outcome(&outcome, request).map_err(|_| invalid())?;
    Ok(outcome)
}

fn install_completion(
    state: &mut StateFile,
    request: &BootstrapRequest,
    outcome: &BootstrapOutcome,
) {
    state.version = 2;
    state.principals.insert(
        request.name.clone(),
        StoredPrincipal {
            kind: "human".into(),
            id: outcome.principal_id.clone(),
            tenant: outcome.tenant_id.clone(),
        },
    );
    state.bootstrap = Some(BootstrapRecovery {
        pending: Some(request.clone()),
        completed: Some(CompletedBootstrap {
            request: request.clone(),
            outcome: outcome.clone(),
        }),
    });
}

fn check_completion_capacity(
    state: &StateFile,
    request: &BootstrapRequest,
    known: Option<&BootstrapOutcome>,
) -> Result<(), CliError> {
    // This in-memory sizing sample is never published or returned. Canonical
    // typed IDs format as a family prefix plus a fixed-width hyphenated UUID;
    // validation rejects alternative representations. Source strings derive
    // from those IDs, and every other string is already fixed by the request.
    // `false` is the longer JSON boolean. A known receipt uses its exact bytes.
    let outcome = known.cloned().unwrap_or_else(|| {
        let principal_id = HumanPrincipalId::from_uuid(uuid::Uuid::nil()).to_string();
        let tenant_id = TenantId::from_uuid(uuid::Uuid::nil()).to_string();
        BootstrapOutcome {
            bootstrap_request_id: request.request_id.clone(),
            kind: "human".into(),
            name: request.name.clone(),
            boundary_id: format!("bnd_{tenant_id}"),
            grant_id: format!("grt_{principal_id}"),
            principal_id,
            tenant_id,
            replayed: false,
        }
    });
    let mut snapshot = state.clone();
    install_completion(&mut snapshot, request, &outcome);
    // Use the real codec, including all preserved maps and JSON escaping. No
    // magic byte allowance or serialized placeholder reaches durable state.
    crate::state_store::check_snapshot(&snapshot).map_err(|error| match error {
        CliError::State(detail) => CliError::state(format!(
            "bootstrap completion preflight refused before HTTP: {detail}"
        )),
        error => error,
    })
}

pub(crate) async fn run(
    cfg: &Config,
    name: &str,
    actions: Option<Vec<String>>,
    json_out: bool,
    resume: bool,
) -> Result<String, CliError> {
    let server = canonical_server(&cfg.server_base)?;
    let mut writer = Writer::open_recovery(&cfg.state_dir)?;
    let previous = writer.state().bootstrap.as_ref();
    let pending = previous.and_then(|recovery| recovery.pending.as_ref());
    let completed = previous.and_then(|recovery| recovery.completed.as_ref());
    let request = if let Some(pending) = pending {
        if pending.server != server || pending.name != name {
            return Err(CliError::state(
                "bootstrap recovery is pending for a different server or name".into(),
            ));
        }
        pending.clone()
    } else if resume {
        completed
            .filter(|done| done.request.server == server && done.request.name == name)
            .ok_or_else(|| CliError::usage("no matching bootstrap receipt to resume".into()))?
            .request
            .clone()
    } else {
        BootstrapRequest {
            request_id: RequestId::new().to_string(),
            server,
            name: name.to_owned(),
            actions,
        }
    };
    let cached = completed
        .filter(|done| done.request == request)
        .map(|done| done.outcome.clone());
    let previous_completion = completed.cloned();

    // Pending alone may fit when completion does not. Refuse before publishing
    // intent or dispatching HTTP; retain any earlier snapshot and pending key.
    check_completion_capacity(writer.state(), &request, cached.as_ref())?;

    // Even a previously visible pending snapshot is synchronized again: the
    // preceding process may have reported an unconfirmed publication phase.
    let state = writer.state_mut();
    state.version = 2;
    state.bootstrap = Some(BootstrapRecovery {
        pending: Some(request.clone()),
        completed: previous_completion,
    });
    writer.persist()?;

    let local_receipt = cached.is_some();
    let outcome = match cached {
        Some(outcome) => outcome,
        None => {
            ApiClient::new(&request.server)
                .enroll_bootstrap(&request)
                .await?
        }
    };
    install_completion(writer.state_mut(), &request, &outcome);
    writer.persist()?;
    // Completion is already durable before removing unresolved intent. Keep its
    // receipt for explicit recovery after this process returns or loses stdout.
    writer
        .state_mut()
        .bootstrap
        .as_mut()
        .expect("installed recovery")
        .pending = None;
    writer.persist()?;

    let source = if local_receipt {
        "local_receipt"
    } else {
        "server"
    };
    if json_out {
        let mut value = serde_json::to_value(&outcome)
            .map_err(|_| CliError::Malformed("bootstrap output could not be encoded".into()))?;
        value["recovery_source"] = source.into();
        return crate::or_json(&value, true);
    }
    let verb = if local_receipt {
        "recovered historical enrollment of"
    } else if outcome.replayed {
        "already enrolled"
    } else {
        "enrolled"
    };
    Ok(format!(
        "{verb} human `{}` as {} in tenant {}\nboundary: {}\nbootstrap request: {}\nrecovery source: {source}",
        outcome.name, outcome.principal_id, outcome.tenant_id,
        outcome.boundary_id, outcome.bootstrap_request_id,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn bootstrap_reply_requires_original_strict_object_and_bound_source_fields() {
        let request = BootstrapRequest {
            request_id: "req_00000000-0000-7000-8000-000000000001".into(),
            server: "http://127.0.0.1:4310".into(),
            name: "alice".into(),
            actions: None,
        };
        let good = json!({
            "bootstrap_request_id":request.request_id, "kind":"human", "name":"alice",
            "principal_id":"hpr_00000000-0000-4000-8000-000000000002",
            "tenant_id":"ten_00000000-0000-4000-8000-000000000003",
            "boundary_id":"bnd_ten_00000000-0000-4000-8000-000000000003",
            "grant_id":"grt_hpr_00000000-0000-4000-8000-000000000002", "replayed":false
        });
        let raw = serde_json::to_string(&good).unwrap();
        let mut invalid = vec![
            "[]".into(),
            "null".into(),
            "{}".into(),
            format!("{raw} {raw}"),
        ];
        for (field, value) in good.as_object().unwrap() {
            let mut missing = good.clone();
            missing.as_object_mut().unwrap().remove(field);
            invalid.push(missing.to_string());
            let duplicate = format!("{{{}:{value},{}", json!(field), &raw[1..]);
            invalid.push(duplicate);
            let mut wrong_type = good.clone();
            wrong_type[field] = Value::Null;
            invalid.push(wrong_type.to_string());
        }
        for field in [
            "bootstrap_request_id",
            "kind",
            "name",
            "principal_id",
            "tenant_id",
            "boundary_id",
            "grant_id",
        ] {
            let mut mismatch = good.clone();
            mismatch[field] = json!("SECRET-invalid-value");
            invalid.push(mismatch.to_string());
        }
        let mut extra = good.clone();
        extra["unknown"] = json!("SECRET-invalid-value");
        invalid.push(extra.to_string());
        // A sequence in the exact declaration order would otherwise be accepted
        // by Serde's derived struct visitor, despite being outside the wire contract.
        invalid.push(
            json!([
                good["bootstrap_request_id"],
                good["kind"],
                good["name"],
                good["principal_id"],
                good["tenant_id"],
                good["boundary_id"],
                good["grant_id"],
                good["replayed"]
            ])
            .to_string(),
        );
        for (index, raw) in invalid.iter().enumerate() {
            let error = decode_outcome(raw.as_bytes(), &request)
                .unwrap_err()
                .to_string();
            assert!(!error.contains("SECRET"), "case {index}");
        }
        for replayed in [false, true] {
            let mut valid = good.clone();
            valid["replayed"] = json!(replayed);
            let raw = format!(" \n{}\t", valid);
            let outcome = decode_outcome(raw.as_bytes(), &request).unwrap();
            assert_eq!(outcome.replayed, replayed);
            assert_eq!(outcome.principal_id, good["principal_id"]);
        }
        eprintln!("strict bootstrap reply: {} malformed controls and two historical-source positive controls", invalid.len());
    }
}
