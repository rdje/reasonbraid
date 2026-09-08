//! The adapter contract (`ROADMAP.md` §11.2, `KICKOFF.md` §3 WP4): a narrow,
//! vendor-neutral boundary between the node's supervision and a harness/provider.
//!
//! The contract is the SDK surface (`.4.1`): [`SDK_VERSION`] is the version
//! token every adapter reports through [`Adapter::sdk_version`] — the first
//! axis of the compatibility matrix (`docs/decisions/2026-09-08_sdk-compatibility-matrix-schema.md`).
//! A change to the contract's types or semantics MUST bump the token; the
//! harness + the matrix re-derive the qualification against the bump.
//!
//! # What the contract is FOR
//!
//! - **Capabilities are declared, not inferred** ([`AdapterCapabilities`]): streaming,
//!   cancellation strength, provider idempotency, status lookup, tool support, and the
//!   policy-injection mode. Domain code branches on capabilities — never on provider
//!   names, and never on a vendor DTO (vendor-specific types exist only inside an
//!   adapter implementation, never in `reasonbraid-core`).
//! - **Dispatch acknowledgement is distinct from completion**: [`Adapter::invoke`]
//!   either refuses before any provider contact ([`InvokeOutcome::FailedBeforeDispatch`])
//!   or returns a dispatch acknowledgement ([`DispatchAck`]) plus an
//!   [`AttemptHandle`] whose events (`output chunks → completed | failed_known`) are the
//!   result — which may never arrive ([`Adapter::query_status`] is then the ONLY way to
//!   prove it).
//! - **Unsupported status lookup is honest**: [`StatusLookupOutcome::Unsupported`] is a
//!   fact about the adapter, not a retry recommendation. What to do with an
//!   indeterminate attempt is the caller's/policy's decision (`§11.3`, `§14.6`: retry of
//!   `outcome_unknown` requires duplicate-risk authorization).
//! - **Credentials never cross this boundary**: the contract has no credential field.
//!   An adapter resolves its own credentials just in time, out of band (`§16.5`).
//!
//! # The fake adapter
//!
//! [`crate::fake::FakeAdapter`] is the deterministic conformance oracle (`§11.6`):
//! script-driven, no sleeps, with a cancellation Notify so a hang is fully
//! deterministic. Its sanitized outcome fixtures live in `fixtures/` — the corpus a real
//! adapter must match semantically in `.4.2`.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The adapter-contract SDK version token (`.4.1`): the matrix's first
/// axis. Bump on ANY type/semantics change to the contract — the bump
/// invalidates the previous qualifications (the matrix re-derives).
pub const SDK_VERSION: &str = "1";

/// One supervised run request: the run spec is opaque to the adapter (untouched
/// content), plus the caller's deadline and a budget hint the WP5 reservation system
/// will type properly — carried, not enforced, at this boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub payload: Value,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
    #[serde(default)]
    pub budget_hint: Option<Value>,
}

/// The declared behavior of an adapter (`§11.2`'s `AdapterCapabilities`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdapterCapabilities {
    pub streaming: bool,
    pub cancellation: CancellationStrength,
    pub provider_idempotency: bool,
    pub status_lookup: bool,
    pub tool_support: bool,
    pub policy_injection: PolicyInjectionMode,
}

/// How strongly cancellation can be relied upon (`§11.2`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CancellationStrength {
    None,
    BestEffort,
    Confirmed,
}

/// How system/developer policy reaches the harness (`§11.2`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyInjectionMode {
    None,
    PromptOnly,
    Structured,
}

/// A dispatch acknowledgement: the provider accepted the call. This is NOT completion —
/// the attempt may still stream, fail, or vanish.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DispatchAck {
    /// The provider's request id when it exposes one (the proof handle a status lookup
    /// needs); absent when the provider has no such handle.
    #[serde(default)]
    pub provider_request_id: Option<String>,
}

/// Events an accepted attempt produces. Chunks are opaque untrusted content (a chunk may
/// be malformed for the CONSUMER's schema — the adapter must not parse domain meaning);
/// the terminal events carry the definitive result. [`ProviderRequestId`] surfaces the
/// provider's request handle when it only becomes known AFTER dispatch (e.g. Codex's
/// `thread.started` event) — the supervisor attaches it to the attempt as the proof
/// handle, exactly like an ack-carried id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttemptEvent {
    ProviderRequestId { request_id: String },
    OutputChunk { chunk: String },
    Completed { usage: Option<Value> },
    FailedKnown { reason: String },
}

impl AttemptEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            AttemptEvent::ProviderRequestId { .. } => "provider_request_id",
            AttemptEvent::OutputChunk { .. } => "output_chunk",
            AttemptEvent::Completed { .. } => "completed",
            AttemptEvent::FailedKnown { .. } => "failed_known",
        }
    }
}

/// The outcome of [`Adapter::invoke`].
#[derive(Debug)]
pub enum InvokeOutcome {
    /// The adapter (or provider) deterministically refused before any dispatch: the
    /// attempt failed BEFORE the boundary — safe to redeliver/retry by policy.
    FailedBeforeDispatch {
        reason: String,
        usage: Option<Value>,
    },
    /// The call was dispatched: the acknowledgement is the boundary, and the handle
    /// streams the attempt's events. If the stream ends WITHOUT a terminal event, the
    /// response was lost after dispatch — only a status lookup can prove the result.
    Accepted(DispatchAck, AttemptHandle),
}

/// The result of a provider status lookup (`§11.2`: `query_status(operation_id) ->
/// supported | unsupported | result`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatusLookupOutcome {
    /// This adapter/provider cannot look a past attempt up. The attempt's fate is
    /// indeterminate — an honest `outcome_unknown`, NEVER a retry recommendation.
    Unsupported,
    /// The provider proved the attempt's result.
    Supported(AttemptResult),
}

/// A proven attempt result (from a lookup, or from the stream's terminal event).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttemptResult {
    Completed { usage: Option<Value> },
    FailedKnown { reason: String },
}

/// The outcome of [`Adapter::cancel`] for one operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CancellationOutcome {
    /// The cancellation is confirmed: the attempt will produce no further work.
    Confirmed,
    /// The request was delivered, but the provider may or may not stop.
    BestEffort,
    /// The provider/adapter ignored the cancellation and continues.
    Ignored,
}

/// A normalized usage receipt (`§14.5`: what is actually observable; unknown dimensions
/// are recorded as unknown, never as zero).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NormalizedUsage {
    #[serde(default)]
    pub input_tokens: Option<i64>,
    #[serde(default)]
    pub output_tokens: Option<i64>,
    #[serde(default)]
    pub cost: Option<Value>,
    pub confidence: UsageConfidence,
}

/// How trustworthy a normalized usage number is (`§14.3`/`§14.5`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageConfidence {
    Exact,
    Estimated,
    Unknown,
}

/// A streaming attempt handle: `next()` yields the attempt's events in order and then
/// `None`. `None` without a preceding terminal event means the response was lost after
/// dispatch (or the attempt was cancelled) — the attempt is indeterminate.
pub struct AttemptHandle(Box<dyn AttemptStream + Send>);

/// The stream behind an [`AttemptHandle`]: implement this to provide the events of one
/// accepted attempt (or build the handle with [`AttemptHandle::new`]).
pub trait AttemptStream: Send {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>>;
}

impl AttemptHandle {
    pub fn new(inner: Box<dyn AttemptStream + Send>) -> Self {
        Self(inner)
    }

    pub async fn next(&mut self) -> Option<AttemptEvent> {
        self.0.next().await
    }
}

impl fmt::Debug for AttemptHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AttemptHandle(..)")
    }
}

/// The narrow adapter contract. Implementations are `Send + Sync`; the node supervises
/// them with OS-level limits where a sidecar is involved (`§11.7`).
///
/// `async fn` in the trait is deliberate: the returned futures are `Send` in practice
/// (every Phase 0 implementation is), and spelling out the `impl Future + Send` bounds
/// by hand would triple the contract's surface for no gain at this phase — revisit if a
/// non-`Send` adapter ever becomes necessary.
#[allow(async_fn_in_trait)]
pub trait Adapter: Send + Sync {
    /// The adapter's declared behavior — the ONLY thing callers may assume about it.
    fn capabilities(&self) -> AdapterCapabilities;

    /// The contract version this adapter implements (the SDK's token —
    /// defaults to the crate's [`SDK_VERSION`]; the harness refuses a
    /// mismatch instead of guessing the semantics).
    fn sdk_version(&self) -> &'static str {
        SDK_VERSION
    }

    /// Dispatch one run. Returns a refusal BEFORE any provider contact, or a dispatch
    /// acknowledgement plus the streaming handle. Never blocks for the full result.
    async fn invoke(&self, request: &RunRequest, operation_id: &str) -> InvokeOutcome;

    /// Request cancellation of a dispatched attempt.
    async fn cancel(&self, operation_id: &str) -> CancellationOutcome;

    /// Look up a past attempt by its operation. [`StatusLookupOutcome::Unsupported`] is
    /// a capability fact — not a retry recommendation.
    async fn query_status(&self, operation_id: &str) -> StatusLookupOutcome;

    /// Normalize a raw provider usage receipt into the shared shape. Unknown or
    /// unmetered dimensions stay `None` (`§14.1`: recorded as unknown, not zero).
    fn normalize_usage(&self, raw_receipt: &Value) -> NormalizedUsage;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SDK version token is the matrix's first axis: the pinned value
    /// + every shipped adapter reports it (the harness refuses a mismatch).
    #[test]
    fn the_sdk_version_token_pins_the_contract() {
        assert_eq!(SDK_VERSION, "1", "the token is the pinned contract version");
        let fake = crate::fake::FakeAdapter::new(
            vec![],
            crate::fake::StatusLookupSpec::Unsupported,
            AdapterCapabilities {
                streaming: false,
                cancellation: CancellationStrength::BestEffort,
                provider_idempotency: false,
                status_lookup: false,
                tool_support: false,
                policy_injection: PolicyInjectionMode::None,
            },
        );
        assert_eq!(
            fake.sdk_version(),
            SDK_VERSION,
            "the shipped adapter reports the contract's token"
        );
    }
}
