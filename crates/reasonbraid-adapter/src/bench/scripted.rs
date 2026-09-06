//! The WP7 scripted agent (`PHASE-0.7`): a deterministic, case-aware adapter that
//! plays the corpus's scripted outputs through the REAL [`Adapter`] contract.
//!
//! It is NOT the WP4 conformance fake — it exists to prove the benchmark harness
//! itself: the same `invoke → stream → completed(usage)` path a real provider
//! takes, with outputs and usage derived deterministically from the request, so
//! the graders, workflow routing, and cost accounting are testable without a
//! single model call. The payload carries `{role, case_id, prompt}`; the answer
//! comes from the corpus's [`ScriptedAnswers`].

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::bench::corpus::{Case, ScriptedAnswers};
use crate::contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptStream, CancellationOutcome,
    CancellationStrength, DispatchAck, InvokeOutcome, NormalizedUsage, PolicyInjectionMode,
    RunRequest, StatusLookupOutcome, UsageConfidence,
};

/// The scripted role a request names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    AgentA,
    AgentB,
    Critique,
    Revision,
    Moderator,
}

impl Role {
    pub fn wire(&self) -> &'static str {
        match self {
            Role::AgentA => "agent_a",
            Role::AgentB => "agent_b",
            Role::Critique => "critique",
            Role::Revision => "revision",
            Role::Moderator => "moderator",
        }
    }

    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "agent_a" => Some(Role::AgentA),
            "agent_b" => Some(Role::AgentB),
            "critique" => Some(Role::Critique),
            "revision" => Some(Role::Revision),
            "moderator" => Some(Role::Moderator),
            _ => None,
        }
    }
}

/// The deterministic, case-aware benchmark agent.
pub struct ScriptedAgent {
    /// case id → scripted answers.
    cases: HashMap<String, ScriptedAnswers>,
}

impl ScriptedAgent {
    pub fn new(cases: &[Case]) -> Self {
        Self {
            cases: cases
                .iter()
                .map(|case| (case.id.clone(), case.scripted.clone()))
                .collect(),
        }
    }

    /// The scripted raw text one request names, with the confidence in the
    /// requested format — exactly what a complying model is asked to emit.
    fn text_for(&self, payload: &Value) -> String {
        let case_id = payload
            .get("case_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let role = payload
            .get("role")
            .and_then(|v| v.as_str())
            .and_then(Role::parse)
            .unwrap_or(Role::AgentA);
        let scripted = &self
            .cases
            .get(case_id)
            .unwrap_or_else(|| panic!("unknown case id `{case_id}` (scripted mode)"));
        let (text, confidence) = match role {
            Role::AgentA => (&scripted.agent_a.text, scripted.agent_a.confidence),
            Role::AgentB => (&scripted.agent_b.text, scripted.agent_b.confidence),
            Role::Critique => (&scripted.critique.text, scripted.critique.confidence),
            Role::Revision => (&scripted.revision.text, scripted.revision.confidence),
            Role::Moderator => {
                let unresolved = if scripted.synthesis.unresolved.is_empty() {
                    "none".to_string()
                } else {
                    scripted.synthesis.unresolved.join("; ")
                };
                return format!(
                    "{}\nUNRESOLVED: {unresolved}\nCONFIDENCE: {}",
                    scripted.synthesis.text, scripted.synthesis.confidence
                );
            }
        };
        if text.to_lowercase().contains("confidence") {
            text.clone()
        } else {
            format!("{text}\nCONFIDENCE: {confidence}")
        }
    }

    /// Deterministic usage: characters/4 as a token stand-in (the convention the
    /// corpus self-test asserts against), never zero.
    fn usage_for(payload: &Value, text: &str) -> Value {
        let prompt_len = payload
            .get("prompt")
            .and_then(|v| v.as_str())
            .map_or(0usize, str::len);
        json!({
            "input_tokens": (prompt_len / 4).max(1) as i64,
            "output_tokens": (text.len() / 4).max(1) as i64,
            "estimated": true,
        })
    }
}

/// A fixed event sequence (one chunk, then the terminal completion) — the same
/// shape a real provider streams.
struct ScriptedStream {
    events: Vec<AttemptEvent>,
    pos: usize,
}

impl AttemptStream for ScriptedStream {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        let event = self.events.get(self.pos).cloned();
        if event.is_some() {
            self.pos += 1;
        }
        Box::pin(async move { event })
    }
}

impl Adapter for ScriptedAgent {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: true,
            cancellation: CancellationStrength::None,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: PolicyInjectionMode::None,
        }
    }

    async fn invoke(&self, request: &RunRequest, _operation_id: &str) -> InvokeOutcome {
        let text = self.text_for(&request.payload);
        let usage = Self::usage_for(&request.payload, &text);
        let events = vec![
            AttemptEvent::OutputChunk { chunk: text },
            AttemptEvent::Completed { usage: Some(usage) },
        ];
        InvokeOutcome::Accepted(
            DispatchAck {
                provider_request_id: None,
            },
            AttemptHandle::new(Box::new(ScriptedStream { events, pos: 0 })),
        )
    }

    async fn cancel(&self, _operation_id: &str) -> CancellationOutcome {
        CancellationOutcome::BestEffort
    }

    async fn query_status(&self, _operation_id: &str) -> StatusLookupOutcome {
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, raw: &Value) -> NormalizedUsage {
        NormalizedUsage {
            input_tokens: raw.get("input_tokens").and_then(|v| v.as_i64()),
            output_tokens: raw.get("output_tokens").and_then(|v| v.as_i64()),
            cost: raw.get("cost").cloned(),
            confidence: if raw.get("estimated") == Some(&Value::Bool(true)) {
                UsageConfidence::Estimated
            } else {
                UsageConfidence::Unknown
            },
        }
    }
}
