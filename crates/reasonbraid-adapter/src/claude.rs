//! The second REAL adapter (`PHASE-1.4.1`): the Claude-family CLI, supervised as a
//! subprocess on the narrowest supported machine interface — `claude -p
//! --output-format stream-json` (`ROADMAP.md` §11.6: "an official non-interactive
//! CLI JSON path"; the `.4.2` Codex mirror).
//!
//! # The machine interface (verified 2026-09-06 against claude 2.1.263, live)
//!
//! ```text
//! claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>
//! ```
//!
//! prints a JSONL event stream on stdout (probe evidence on-volume in
//! `target/claude-probes/probe_verbose.jsonl`):
//!
//! ```text
//! {"type":"system","subtype":"init","session_id":"…","model":"…"}   → ProviderRequestId
//! {"type":"assistant","message":{"content":[{"type":"text","text":"…"},…]}}
//!                                                                   → OutputChunk per text block
//! {"type":"result","subtype":"success","is_error":false,"usage":{…},
//!  "total_cost_usd":0.030367,"duration_ms":…,"num_turns":…}         → Completed{usage,cost}
//! ```
//!
//! Facts pinned by three bounded live probes (two dispatches, one pre-dispatch
//! refusal):
//!
//! - `--verbose` is REQUIRED with `--output-format stream-json` — the CLI refuses
//!   the combination before any dispatch (`Error: When using --print,
//!   --output-format=stream-json requires --verbose`), so the flag is not optional.
//! - The prompt argument must follow a `--` separator: `--tools` is variadic and
//!   otherwise swallows the prompt.
//! - `--restricted` removes the code-running tools and WebFetch; `--tools ''`
//!   disables ALL tools — the boundary is content-only.
//! - `session_id` arrives FIRST (in `system/init`), so the provider handle is
//!   available before the first chunk (Codex reveals its id in the stream too).
//! - `total_cost_usd` is MONEY: unlike Codex (tokens only), the Claude receipt
//!   carries an authoritative dollar figure — normalized `cost` is `Some`, not
//!   `None`. Anthropic's `input_tokens` already includes cache reads and
//!   `output_tokens` already includes thinking tokens, so no folding (unlike
//!   Codex's reasoning-token fold).
//!
//! # What is honestly unsupported
//!
//! - **Status lookup**: `--resume <session-id>` CONTINUES a session; it is not a
//!   query of a past attempt's outcome. [`ClaudeCliAdapter::query_status`] returns
//!   [`StatusLookupOutcome::Unsupported`] — a lost response stays `outcome_unknown`.
//! - **Provider idempotency keys**: the CLI exposes none → `provider_idempotency: false`.
//! - **Cancellation**: killing the subprocess is `BestEffort` — the provider may or
//!   may not stop; `Confirmed` would need provider proof this boundary cannot obtain.
//!
//! # Content safety
//!
//! The run payload's `prompt` travels as the USER prompt argument (after `--`) —
//! never as system config or policy. Everything in it is untrusted content
//! (`§16.6`); the harness's own instructions live in the user's Claude
//! configuration, not in ours. The adapter holds no credentials: Claude uses its
//! ambient user login (`§16.5`).

use std::collections::VecDeque;
use std::ffi::OsString;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::Mutex;

use crate::contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptStream, CancellationOutcome,
    CancellationStrength, DispatchAck, InvokeOutcome, NormalizedUsage, PolicyInjectionMode,
    RunRequest, StatusLookupOutcome, UsageConfidence,
};

/// The `claude -p` flags this adapter always passes (machine interface, no session
/// persistence, no tools, restricted mode; `--verbose` is REQUIRED by the CLI with
/// `stream-json`). The prompt follows the `--` separator — `--tools` is variadic and
/// would otherwise swallow it.
const EXEC_ARGS: &[&str] = &[
    "-p",
    "--output-format",
    "stream-json",
    "--restricted",
    "--tools",
    "",
    "--verbose",
    "--",
];

/// The second real harness adapter: supervises `claude -p --output-format
/// stream-json` as a child process.
pub struct ClaudeCliAdapter {
    binary: OsString,
    capabilities: AdapterCapabilities,
    /// One child per operation, so `cancel` can reach it while the handle streams.
    children: Mutex<std::collections::HashMap<String, Arc<Mutex<Child>>>>,
}

impl Default for ClaudeCliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCliAdapter {
    /// The adapter for the `claude` binary on PATH.
    pub fn new() -> Self {
        Self::with_binary("claude")
    }

    /// The same adapter pointed at a specific binary (tests use a stub script here —
    /// the supervision mechanics are identical, only the provider is fake).
    pub fn with_binary(binary: impl Into<OsString>) -> Self {
        Self {
            binary: binary.into(),
            capabilities: AdapterCapabilities {
                streaming: true,
                cancellation: CancellationStrength::BestEffort,
                provider_idempotency: false,
                status_lookup: false,
                tool_support: false,
                policy_injection: PolicyInjectionMode::None,
            },
            children: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// The run payload → user prompt: a `"prompt"` string when present, otherwise the
    /// payload's JSON text. Always USER content, never instructions (`§16.6`).
    fn prompt_for(payload: &Value) -> String {
        match payload.get("prompt") {
            Some(Value::String(s)) => s.clone(),
            _ => payload.to_string(),
        }
    }
}

impl Adapter for ClaudeCliAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        self.capabilities
    }

    async fn invoke(&self, request: &RunRequest, operation_id: &str) -> InvokeOutcome {
        let prompt = Self::prompt_for(&request.payload);
        let mut child = match Command::new(&self.binary)
            .args(EXEC_ARGS)
            .arg(&prompt)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                // The binary is missing or unspawnable: a deterministic refusal
                // BEFORE any dispatch.
                return InvokeOutcome::FailedBeforeDispatch {
                    reason: format!("failed to spawn `{}`: {e}", self.binary.to_string_lossy()),
                    usage: None,
                };
            }
        };

        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let child = Arc::new(Mutex::new(child));
        self.children
            .lock()
            .await
            .insert(operation_id.to_string(), Arc::clone(&child));

        let (stderr_buffer, stderr_drain) = drain_stderr(stderr);

        InvokeOutcome::Accepted(
            DispatchAck {
                // Claude reveals its session id in the stream's first event (system/init),
                // not in the ack — the handle surfaces it as `ProviderRequestId`.
                provider_request_id: None,
            },
            crate::contract::AttemptHandle::new(Box::new(ClaudeHandle {
                lines: BufReader::new(stdout),
                child,
                stderr: stderr_buffer,
                stderr_drain,
                pending_chunks: VecDeque::new(),
                finished: false,
            })),
        )
    }

    async fn cancel(&self, operation_id: &str) -> CancellationOutcome {
        let child = {
            let children = self.children.lock().await;
            match children.get(operation_id) {
                Some(child) => Arc::clone(child),
                None => return CancellationOutcome::BestEffort,
            }
        };
        // Killing the subprocess is best-effort: the provider may or may not stop.
        // `Confirmed` would need provider proof this boundary cannot obtain.
        let _ = child.lock().await.start_kill();
        CancellationOutcome::BestEffort
    }

    async fn query_status(&self, _operation_id: &str) -> StatusLookupOutcome {
        // `claude --resume <session-id>` CONTINUES a session; it is not a status query
        // for a past attempt. Honest Unsupported — the caller's only safe exit from
        // ambiguity is proof it cannot obtain, or adjudication.
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, raw_receipt: &Value) -> NormalizedUsage {
        // The receipt is the FULL `result` event: the token counts live under `usage`,
        // the money under `total_cost_usd` — neither alone is the receipt.
        let usage = raw_receipt.get("usage");
        let number = |key: &str| usage.and_then(|u| u.get(key)).and_then(|v| v.as_i64());
        let input_tokens = number("input_tokens");
        let output_tokens = number("output_tokens");
        // Anthropic's input_tokens already includes cache reads and output_tokens
        // already includes thinking tokens — billed as such; no folding (unlike the
        // Codex receipt, where reasoning tokens must be added explicitly).
        let confidence = if usage.is_some() {
            // A `result` event's usage block IS the provider's authoritative receipt.
            UsageConfidence::Exact
        } else {
            UsageConfidence::Unknown
        };
        NormalizedUsage {
            input_tokens,
            output_tokens,
            // Claude reports money: the observed dollar figure is the cost —
            // unlike Codex (tokens only), it is known, not None.
            cost: raw_receipt.get("total_cost_usd").cloned(),
            confidence,
        }
    }
}

/// Drain the child's stderr into a bounded buffer so a chatty child can never
/// deadlock on a full pipe; the tail feeds `FailedKnown` reasons. The returned
/// handle MUST be awaited before the buffer is snapshotted — the drain task may
/// not have consumed the pipe's tail yet when stdout hits EOF (`PHASE-1-MAINT-2`:
/// the race, reproduced on the Codex adapter and fixed in both mirrors).
fn drain_stderr(
    stderr: tokio::process::ChildStderr,
) -> (Arc<Mutex<String>>, tokio::task::JoinHandle<()>) {
    let buffer = Arc::new(Mutex::new(String::new()));
    let out = Arc::clone(&buffer);
    let handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let mut buf = out.lock().await;
            if buf.len() < 8192 {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
    });
    (buffer, handle)
}

/// The stream over one Claude child: JSONL events mapped to the contract, then the
/// exit status as the terminal verdict.
struct ClaudeHandle {
    lines: BufReader<ChildStdout>,
    child: Arc<Mutex<Child>>,
    stderr: Arc<Mutex<String>>,
    stderr_drain: tokio::task::JoinHandle<()>,
    /// Text blocks of an `assistant` message not yet streamed (one chunk per block;
    /// thinking blocks never enter here — the reply is the text).
    pending_chunks: VecDeque<String>,
    finished: bool,
}

impl AttemptStream for ClaudeHandle {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        Box::pin(self.next_event())
    }
}

impl ClaudeHandle {
    async fn next_event(&mut self) -> Option<AttemptEvent> {
        if self.finished {
            return None;
        }
        if let Some(chunk) = self.pending_chunks.pop_front() {
            return Some(AttemptEvent::OutputChunk { chunk });
        }
        loop {
            let mut line = String::new();
            match self.lines.read_line(&mut line).await {
                Ok(0) => {
                    // EOF: the child is done — the exit status is the terminal verdict.
                    // AWAIT the stderr drain first (bounded): the task may not have
                    // consumed the pipe's tail yet, and racing it leaves the reason
                    // EMPTY (the PHASE-1-MAINT-2 defect, fixed in both mirrors). The
                    // bound guards against a grandchild that inherited stderr keeping
                    // the pipe open.
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        &mut self.stderr_drain,
                    )
                    .await;
                    let stderr_tail = {
                        let buf = self.stderr.lock().await;
                        if buf.len() > 1024 {
                            buf[buf.len() - 1024..].to_string()
                        } else {
                            buf.clone()
                        }
                    };
                    let status = {
                        let mut child = self.child.lock().await;
                        child.wait().await.ok()
                    };
                    self.finished = true;
                    return match status {
                        Some(s) if s.success() => None, // no result seen: lost response
                        Some(s) => Some(AttemptEvent::FailedKnown {
                            reason: format!(
                                "claude exited with {s}; stderr tail: {}",
                                stderr_tail.trim()
                            ),
                        }),
                        None => Some(AttemptEvent::FailedKnown {
                            reason: "claude child vanished without an exit status".to_string(),
                        }),
                    };
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let Ok(event) = serde_json::from_str::<Value>(trimmed) else {
                        continue; // human/status lines are not events; never parse them
                    };
                    match event.get("type").and_then(|t| t.as_str()) {
                        Some("system") => {
                            // system/init arrives first and carries the session id —
                            // the proof handle an operator would adjudicate with.
                            if let Some(id) = event.get("session_id").and_then(|v| v.as_str()) {
                                return Some(AttemptEvent::ProviderRequestId {
                                    request_id: id.to_string(),
                                });
                            }
                        }
                        Some("assistant") => {
                            // One chunk per TEXT block; thinking blocks are the model's
                            // internal reasoning, not the reply — skipped, never parsed.
                            if let Some(content) =
                                event.pointer("/message/content").and_then(|c| c.as_array())
                            {
                                let mut first: Option<String> = None;
                                for block in content {
                                    if block.get("type").and_then(|t| t.as_str()) != Some("text") {
                                        continue;
                                    }
                                    let Some(text) = block.get("text").and_then(|t| t.as_str())
                                    else {
                                        continue;
                                    };
                                    let text = text.to_string();
                                    if first.is_none() {
                                        first = Some(text);
                                    } else {
                                        self.pending_chunks.push_back(text);
                                    }
                                }
                                if let Some(chunk) = first {
                                    return Some(AttemptEvent::OutputChunk { chunk });
                                }
                            }
                        }
                        Some("result") => {
                            self.finished = true;
                            if event.get("is_error").and_then(|v| v.as_bool()) == Some(true) {
                                // The provider TOLD us it failed: a definitive failure,
                                // with the provider's own message as the reason.
                                let reason = event
                                    .get("result")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("claude reported an error result");
                                return Some(AttemptEvent::FailedKnown {
                                    reason: reason.to_string(),
                                });
                            }
                            // The full result event is the receipt: usage under `usage`,
                            // money under `total_cost_usd`.
                            return Some(AttemptEvent::Completed { usage: Some(event) });
                        }
                        _ => continue,
                    }
                }
                Err(_) => return None,
            }
        }
    }
}
