//! The R5 credential broker surface (PHASE-4.5.2, the
//! `2026-09-07_r5r3rx-contracts-opt-in.md` contract): the LOCAL broker that
//! resolves the opaque `credential_binding_ref` into a per-request
//! credential AT THE REQUEST BOUNDARY. The credential never enters a
//! reference, never logs (the `Debug` impl redacts it), never persists
//! outside the broker's own store; every authenticated acquisition records
//! the explicit disclosure — a credential is a disclosure, not a
//! permission.
//!
//! The dev-profile store is an in-memory registry (the test seam); the
//! deployment integration is the OS keychain (named, out of the dev
//! profile's scope).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The resolved credential: its disclosure class + the transport value.
/// The value is the ONLY field that must never log.
#[derive(Clone, PartialEq)]
pub struct Credential {
    /// The disclosure class (e.g. `github-token-read`) — logged, never
    /// secret.
    pub class: String,
    /// The transport value (the `Authorization` content) — REDACTED in
    /// every display path.
    value: String,
}

impl Credential {
    pub fn new(class: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            class: class.into(),
            value: value.into(),
        }
    }

    /// The per-request attach: the header the fetcher adds for THAT
    /// acquisition only (never ambient).
    pub fn authorization_header(&self) -> (&'static str, String) {
        ("Authorization", self.value.clone())
    }
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("class", &self.class)
            .field("value", &"<redacted>")
            .finish()
    }
}

/// The explicit-disclosure record (the `.5.3` receipt's input): WHAT was
/// disclosed, WHERE, WHEN — never the value.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DisclosureRecord {
    pub credential_class: String,
    pub host: String,
    pub at: chrono::DateTime<chrono::Utc>,
}

/// The broker's failure — every refusal names its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrokerError {
    UnknownBinding(String),
    Unavailable(String),
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownBinding(reference) => {
                write!(
                    f,
                    "the credential binding `{reference}` is unknown to the broker"
                )
            }
            Self::Unavailable(detail) => write!(f, "the credential store is unavailable: {detail}"),
        }
    }
}

impl std::error::Error for BrokerError {}

/// The broker: resolves bindings locally, discloses explicitly.
#[derive(Debug)]
pub struct Broker {
    /// The dev-profile store: the registered binding → credential map
    /// (the deployment's keychain replaces it).
    store: Arc<Mutex<HashMap<String, Credential>>>,
}

impl Default for Broker {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Broker {
    /// Register a binding (the operator's verb — the credential enters the
    /// store once, never a reference).
    pub fn register(&self, binding: impl Into<String>, credential: Credential) {
        let mut store = self.store.lock().expect("the broker store locks");
        store.insert(binding.into(), credential);
    }

    /// Resolve the binding at the request boundary. The returned
    /// credential's value is the caller's to attach per-request — the
    /// broker never logs it.
    pub fn resolve(&self, binding: &str) -> Result<Credential, BrokerError> {
        let store = self.store.lock().expect("the broker store locks");
        store
            .get(binding)
            .cloned()
            .ok_or_else(|| BrokerError::UnknownBinding(binding.to_owned()))
    }

    /// The explicit disclosure the acquisition records with its receipt.
    pub fn disclose(
        &self,
        binding: &str,
        host: &str,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<DisclosureRecord, BrokerError> {
        let credential = self.resolve(binding)?;
        Ok(DisclosureRecord {
            credential_class: credential.class,
            host: host.to_owned(),
            at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_broker_resolves_locally_and_never_logs_the_value() {
        let broker = Broker::default();
        broker.register(
            "cred_github_read",
            Credential::new("github-token-read", "ghp_SECRET"),
        );
        let credential = broker
            .resolve("cred_github_read")
            .expect("the binding resolves");
        assert_eq!(credential.class, "github-token-read");
        assert_eq!(
            credential.authorization_header(),
            ("Authorization", "ghp_SECRET".to_owned())
        );
        // The redacted display: the value never appears in any Debug path.
        let debug = format!("{credential:?}");
        assert!(!debug.contains("ghp_SECRET"), "{debug}");
        assert!(debug.contains("<redacted>"), "{debug}");
    }

    #[test]
    fn the_broker_names_the_unknown_binding_and_the_disclosure() {
        let broker = Broker::default();
        assert_eq!(
            broker.resolve("cred_nope"),
            Err(BrokerError::UnknownBinding("cred_nope".into()))
        );
        broker.register("cred_a", Credential::new("token-read", "S3CR3T-XYZ"));
        let at = chrono::DateTime::parse_from_rfc3339("2026-09-07T15:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let disclosure = broker
            .disclose("cred_a", "github.example", at)
            .expect("the disclosure builds");
        assert_eq!(disclosure.credential_class, "token-read");
        assert_eq!(disclosure.host, "github.example");
        assert_eq!(disclosure.at, at);
        // The disclosure JSON never carries the value.
        let json = serde_json::to_string(&disclosure).unwrap();
        assert!(!json.contains("S3CR3T-XYZ"), "{json}");
    }
}
