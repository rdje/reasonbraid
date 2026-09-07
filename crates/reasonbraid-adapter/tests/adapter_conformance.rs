//! The conformance suite (`PHASE-2.6.1`): every adapter — the deterministic fake and
//! both real CLI adapters (against their stub binaries) — passes the ONE harness in
//! [`conformance`]. §19.4: "A 'works once' demo does not qualify an adapter."

mod conformance;

use conformance::{stubs, ConformanceScenario, Trigger};
use reasonbraid_adapter::fixtures::corpus;
use reasonbraid_adapter::{
    AdapterCapabilities, CancellationStrength, ClaudeCliAdapter, CodexCliAdapter, FakeAdapter,
    PolicyInjectionMode, RunRequest,
};
use serde_json::json;

fn request_with_prompt(prompt: &str) -> RunRequest {
    RunRequest {
        payload: json!({ "prompt": prompt }),
        deadline: None,
        budget_hint: None,
    }
}

/// The fake adapter's scenarios: the fixture corpus is the conformance oracle, and each
/// scenario's expected capabilities come from the FIXTURE's own declaration.
fn fake(name: &str, trigger: Trigger) -> ConformanceScenario<FakeAdapter> {
    let spec = corpus()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no fixture named {name}"));
    ConformanceScenario {
        name: format!("fake:{name}"),
        expected: spec.capabilities,
        adapter: FakeAdapter::from_spec(spec),
        trigger,
        request: request_with_prompt(name),
    }
}

/// The real CLI adapters share one verified boundary: streaming, best-effort
/// cancellation, no status lookup, no provider idempotency, no tools, no policy
/// injection.
fn cli_caps() -> AdapterCapabilities {
    AdapterCapabilities {
        streaming: true,
        cancellation: CancellationStrength::BestEffort,
        provider_idempotency: false,
        status_lookup: false,
        tool_support: false,
        policy_injection: PolicyInjectionMode::None,
    }
}

/// A Codex scenario: the stub branches on the PROMPT (the last CLI argument), so the
/// request's prompt selects the trigger; the scenario name uniquifies the stub dir.
fn codex(name: &str, prompt: &str, trigger: Trigger) -> ConformanceScenario<CodexCliAdapter> {
    // The missing-binary refusal is the ONLY deterministic pre-dispatch refusal of a
    // subprocess adapter — the adapter is constructed against a path that never exists.
    let binary = if trigger == Trigger::RefuseMissingBinary {
        format!("/nonexistent/conformance/{name}/codex").into()
    } else {
        stubs::codex_binary(name)
    };
    ConformanceScenario {
        name: format!("codex:{name}"),
        expected: cli_caps(),
        adapter: CodexCliAdapter::with_binary(binary),
        trigger,
        request: request_with_prompt(prompt),
    }
}

/// A Claude scenario (the same shape as the Codex one).
fn claude(name: &str, prompt: &str, trigger: Trigger) -> ConformanceScenario<ClaudeCliAdapter> {
    let binary = if trigger == Trigger::RefuseMissingBinary {
        format!("/nonexistent/conformance/{name}/claude").into()
    } else {
        stubs::claude_binary(name)
    };
    ConformanceScenario {
        name: format!("claude:{name}"),
        expected: cli_caps(),
        adapter: ClaudeCliAdapter::with_binary(binary),
        trigger,
        request: request_with_prompt(prompt),
    }
}

/// The fake adapter passes every §19.4 checkable item against its own corpus.
#[tokio::test]
async fn fake_adapter_passes_the_conformance_suite() {
    conformance::run(fake("fail_before_dispatch", Trigger::Refuse)).await;
    conformance::run(fake("lose_response_no_lookup", Trigger::Lose)).await;
    conformance::run(fake("hang_forever", Trigger::Hang)).await;
    conformance::run(fake("complete_streaming", Trigger::Complete)).await;
    conformance::run(fake("complete_single", Trigger::Complete)).await;
}

/// The Codex CLI adapter passes the same suite behind a stub `codex` binary.
#[tokio::test]
async fn codex_adapter_passes_the_conformance_suite() {
    conformance::run(codex("refuse", "never used", Trigger::RefuseMissingBinary)).await;
    conformance::run(codex("lose", "please lose the response", Trigger::Lose)).await;
    conformance::run(codex("hang", "please sleep forever", Trigger::Hang)).await;
    conformance::run(codex("complete", "hello", Trigger::Complete)).await;
}

/// The Claude CLI adapter passes the same suite behind a stub `claude` binary.
#[tokio::test]
async fn claude_adapter_passes_the_conformance_suite() {
    conformance::run(claude("refuse", "never used", Trigger::RefuseMissingBinary)).await;
    conformance::run(claude("lose", "please lose the response", Trigger::Lose)).await;
    conformance::run(claude("hang", "please sleep forever", Trigger::Hang)).await;
    conformance::run(claude("complete", "hello", Trigger::Complete)).await;
}
