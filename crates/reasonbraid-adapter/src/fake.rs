//! The deterministic fake adapter (`ROADMAP.md` §11.6): "deterministic scripts for
//! every state, delay, crash, malformed output, refusal, and ambiguity case. This is
//! the conformance oracle, not a throwaway mock."
//!
//! A [`FakeAdapter`] plays a [`ScriptStep`] script per operation. There are NO sleeps
//! anywhere: [`ScriptStep::HangForever`] waits on a per-operation cancellation
//! [`tokio::sync::Notify`], so a hang is resolved exactly by
//! [`FakeAdapter::cancel`] — fully deterministic, no timing races. The same script
//! produces the same event sequence every time.
//!
//! The sanitized outcome corpus (`fixtures/*.json`, [`crate::fixtures::corpus`]) is the
//! conformance set: every adapter outcome has a fixture, no fixture carries a
//! credential, and a real adapter (`.4.2`) must match the same semantics behind the
//! same [`crate::contract::Adapter`] contract.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, Notify};

use crate::contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptResult, AttemptStream,
    CancellationOutcome, DispatchAck, InvokeOutcome, NormalizedUsage, RunRequest,
    StatusLookupOutcome, UsageConfidence,
};

/// One step of a fake attempt's script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "step", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScriptStep {
    /// Stream one output chunk (opaque untrusted content).
    EmitChunk { chunk: String },
    /// Stream a chunk that is garbage for any reasonable output schema — it must pass
    /// through VERBATIM as untrusted content, never be parsed into domain meaning.
    MalformedOutput { chunk: String },
    /// Terminal: the attempt completed, optionally with a raw usage receipt.
    Complete {
        #[serde(default)]
        usage: Option<Value>,
    },
    /// Terminal: a definitive, PROVEN failure (never a guess).
    FailKnown { reason: String },
    /// The adapter refuses BEFORE any dispatch: `invoke` returns
    /// [`InvokeOutcome::FailedBeforeDispatch`] and no attempt handle exists.
    FailBeforeDispatch { reason: String },
    /// The attempt hangs (no further events) until it is cancelled — a cancellation
    /// here is `Confirmed` and ends the stream with no terminal event.
    HangForever,
    /// A cancellation arriving HERE is reported `Ignored` and the script advances
    /// (the provider goes on working).
    IgnoreCancellation,
    /// The dispatch is acknowledged, then the response is LOST: the stream ends with
    /// no terminal event. Only a status lookup can prove the result.
    LoseResponse,
}

impl ScriptStep {
    /// The wire tag of this step (used by the corpus-coverage tests).
    pub fn tag(&self) -> &'static str {
        match self {
            ScriptStep::EmitChunk { .. } => "emit_chunk",
            ScriptStep::MalformedOutput { .. } => "malformed_output",
            ScriptStep::Complete { .. } => "complete",
            ScriptStep::FailKnown { .. } => "fail_known",
            ScriptStep::FailBeforeDispatch { .. } => "fail_before_dispatch",
            ScriptStep::HangForever => "hang_forever",
            ScriptStep::IgnoreCancellation => "ignore_cancellation",
            ScriptStep::LoseResponse => "lose_response",
        }
    }
}

/// What a configured status lookup proves (or that it proves nothing).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StatusLookupSpec {
    Unsupported,
    Completed {
        #[serde(default)]
        usage: Option<Value>,
    },
    FailedKnown {
        reason: String,
    },
}

/// A sanitized, deterministic outcome fixture — the corpus entry that names one
/// adapter outcome and the script + lookup + capabilities that produce it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FixtureSpec {
    pub name: String,
    /// The outcome class this fixture covers (used by the coverage tests).
    pub outcome: String,
    pub capabilities: AdapterCapabilities,
    pub status_lookup: StatusLookupSpec,
    pub script: Vec<ScriptStep>,
    /// The journal terminal state a full supervisor run must reach; absent for the
    /// hang/ignore fixtures that need explicit cancel driving.
    #[serde(default)]
    pub expected_terminal: Option<String>,
}

/// Per-operation state the fake keeps for cancellation, event logging, and lookups.
struct FakeOpState {
    state: Mutex<OpState>,
    cancel_notify: Notify,
}

struct OpState {
    pos: usize,
    cancelled: bool,
    event_log: Vec<AttemptEvent>,
}

impl FakeOpState {
    async fn next_event(&self, script: &[ScriptStep]) -> Option<AttemptEvent> {
        enum Action {
            Emit(AttemptEvent),
            End,
            Advance,
            Wait,
        }
        loop {
            let action = {
                let mut st = self.state.lock().await;
                match script.get(st.pos) {
                    None => Action::End,
                    Some(ScriptStep::HangForever) if st.cancelled => Action::End,
                    Some(ScriptStep::HangForever) => Action::Wait,
                    Some(ScriptStep::EmitChunk { chunk })
                    | Some(ScriptStep::MalformedOutput { chunk }) => {
                        let event = AttemptEvent::OutputChunk {
                            chunk: chunk.clone(),
                        };
                        st.pos += 1;
                        st.event_log.push(event.clone());
                        Action::Emit(event)
                    }
                    Some(ScriptStep::Complete { usage }) => {
                        let event = AttemptEvent::Completed {
                            usage: usage.clone(),
                        };
                        st.pos += 1;
                        st.event_log.push(event.clone());
                        Action::Emit(event)
                    }
                    Some(ScriptStep::FailKnown { reason }) => {
                        let event = AttemptEvent::FailedKnown {
                            reason: reason.clone(),
                        };
                        st.pos += 1;
                        st.event_log.push(event.clone());
                        Action::Emit(event)
                    }
                    Some(ScriptStep::FailBeforeDispatch { .. })
                    | Some(ScriptStep::IgnoreCancellation) => {
                        // Never reached after acceptance (FailBeforeDispatch is handled
                        // in invoke; IgnoreCancellation is transparent to the stream) —
                        // skip defensively rather than stall.
                        st.pos += 1;
                        Action::Advance
                    }
                    Some(ScriptStep::LoseResponse) => {
                        st.pos += 1;
                        Action::End
                    }
                }
            };
            match action {
                Action::Emit(event) => return Some(event),
                Action::End => return None,
                Action::Advance => continue,
                Action::Wait => self.cancel_notify.notified().await,
            }
        }
    }
}

struct FakeHandle {
    shared: Arc<FakeOpState>,
    script: Vec<ScriptStep>,
}

impl AttemptStream for FakeHandle {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        let shared = Arc::clone(&self.shared);
        let script = self.script.clone();
        Box::pin(async move { shared.next_event(&script).await })
    }
}

/// The deterministic fake adapter: one script per operation, a configured status
/// lookup, and declared capabilities — the conformance oracle (`§11.6`).
pub struct FakeAdapter {
    script: Vec<ScriptStep>,
    lookup: StatusLookupSpec,
    capabilities: AdapterCapabilities,
    ops: Mutex<HashMap<String, Arc<FakeOpState>>>,
}

impl FakeAdapter {
    pub fn new(
        script: Vec<ScriptStep>,
        lookup: StatusLookupSpec,
        capabilities: AdapterCapabilities,
    ) -> Self {
        Self {
            script,
            lookup,
            capabilities,
            ops: Mutex::new(HashMap::new()),
        }
    }

    /// Build the fake from a corpus fixture.
    pub fn from_spec(spec: FixtureSpec) -> Self {
        Self::new(spec.script, spec.status_lookup, spec.capabilities)
    }

    /// Whether this fake's status lookup can prove results (realism: a provider that
    /// supports lookup exposes a provider request id in its dispatch acknowledgement).
    fn lookup_supported(&self) -> bool {
        !matches!(self.lookup, StatusLookupSpec::Unsupported)
    }

    /// The event log one operation produced, for determinism assertions.
    pub async fn event_log(&self, operation_id: &str) -> Vec<AttemptEvent> {
        let guard = self.ops.lock().await;
        match guard.get(operation_id) {
            Some(shared) => shared.state.lock().await.event_log.clone(),
            None => Vec::new(),
        }
    }
}

impl Adapter for FakeAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        self.capabilities
    }

    async fn invoke(&self, _request: &RunRequest, operation_id: &str) -> InvokeOutcome {
        match self.script.first() {
            Some(ScriptStep::FailBeforeDispatch { reason }) => {
                InvokeOutcome::FailedBeforeDispatch {
                    reason: reason.clone(),
                    usage: None,
                }
            }
            _ => {
                let shared = Arc::new(FakeOpState {
                    state: Mutex::new(OpState {
                        pos: 0,
                        cancelled: false,
                        event_log: Vec::new(),
                    }),
                    cancel_notify: Notify::new(),
                });
                self.ops
                    .lock()
                    .await
                    .insert(operation_id.to_string(), Arc::clone(&shared));
                let ack = DispatchAck {
                    provider_request_id: self
                        .lookup_supported()
                        .then(|| format!("prv_{operation_id}")),
                };
                InvokeOutcome::Accepted(
                    ack,
                    AttemptHandle::new(Box::new(FakeHandle {
                        shared,
                        script: self.script.clone(),
                    })),
                )
            }
        }
    }

    async fn cancel(&self, operation_id: &str) -> CancellationOutcome {
        let shared = {
            let guard = self.ops.lock().await;
            match guard.get(operation_id) {
                Some(shared) => Arc::clone(shared),
                None => return CancellationOutcome::BestEffort,
            }
        };
        let mut st = shared.state.lock().await;
        match self.script.get(st.pos) {
            Some(ScriptStep::HangForever) => {
                st.cancelled = true;
                // notify_one (not notify_waiters): a permit is stored if the stream
                // has not registered its `notified()` yet — the cancellation must
                // never be lost to a waiter-registration race.
                shared.cancel_notify.notify_one();
                CancellationOutcome::Confirmed
            }
            Some(ScriptStep::IgnoreCancellation) => {
                st.pos += 1;
                CancellationOutcome::Ignored
            }
            _ => CancellationOutcome::BestEffort,
        }
    }

    async fn query_status(&self, operation_id: &str) -> StatusLookupOutcome {
        let known = self.ops.lock().await.contains_key(operation_id);
        if !known {
            return StatusLookupOutcome::Unsupported;
        }
        match &self.lookup {
            StatusLookupSpec::Unsupported => StatusLookupOutcome::Unsupported,
            StatusLookupSpec::Completed { usage } => {
                StatusLookupOutcome::Supported(AttemptResult::Completed {
                    usage: usage.clone(),
                })
            }
            StatusLookupSpec::FailedKnown { reason } => {
                StatusLookupOutcome::Supported(AttemptResult::FailedKnown {
                    reason: reason.clone(),
                })
            }
        }
    }

    fn normalize_usage(&self, raw_receipt: &Value) -> NormalizedUsage {
        let number = |key: &str| raw_receipt.get(key).and_then(|v| v.as_i64());
        let input_tokens = number("input_tokens");
        let output_tokens = number("output_tokens");
        let cost = raw_receipt.get("cost").cloned();
        let confidence = if raw_receipt.get("exact") == Some(&Value::Bool(true)) {
            UsageConfidence::Exact
        } else if input_tokens.is_some() || output_tokens.is_some() || cost.is_some() {
            UsageConfidence::Estimated
        } else {
            UsageConfidence::Unknown
        };
        NormalizedUsage {
            input_tokens,
            output_tokens,
            cost,
            confidence,
        }
    }
}
