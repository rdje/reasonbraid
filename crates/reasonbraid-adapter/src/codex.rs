//! The first REAL adapter (`PHASE-0.4.2`): the Codex-family CLI, supervised as a
//! subprocess on the narrowest supported machine interface — `codex exec --json`
//! (`ROADMAP.md` §11.6: "prefer current supported machine interfaces… revalidate
//! against official documentation and changelog").
//!
//! # The machine interface (verified 2026-09-06, codex-cli 0.153.4)
//!
//! `codex exec --json --skip-git-repo-check --ephemeral --sandbox read-only -- <prompt>`
//! prints a JSONL event stream on stdout:
//!
//! ```text
//! {"type":"thread.started","thread_id":"…"}                      → ProviderRequestId
//! {"type":"item.completed","item":{"type":"agent_message","text":"…"}} → OutputChunk
//! {"type":"turn.completed","usage":{"input_tokens":…,"output_tokens":…}}
//!                                                                → Completed{usage}
//! ```
//!
//! # What is honestly unsupported
//!
//! - **Status lookup**: `codex exec` can RESUME a thread, but it offers no first-class
//!   query of a past attempt's outcome. [`CodexCliAdapter::query_status`] returns
//!   [`StatusLookupOutcome::Unsupported`] — a lost response therefore stays
//!   `outcome_unknown` (the WP4 acceptance's honest leg, exercised by a REAL adapter).
//! - **Provider idempotency keys**: `exec` exposes none → `provider_idempotency: false`.
//! - **Cancellation**: killing the subprocess is `BestEffort` — the provider may or
//!   may not stop; `Confirmed` would need provider proof we do not have.
//! - **Cost**: the event stream reports token usage, never money → normalized `cost`
//!   stays `None` (unknown, not zero — `§14.1`).
//!
//! # Content safety
//!
//! The run payload's `prompt` travels as the USER prompt argument — never as system
//! config or policy. Everything in it is untrusted content (`§16.6`); the harness's
//! own instructions live in the user's Codex configuration, not in ours. The adapter
//! holds no credentials: Codex uses its ambient user login (`§16.5`).

use std::ffi::OsString;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::Mutex;

use crate::contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptStream, CancellationOutcome,
    CancellationStrength, DispatchAck, InvokeOutcome, NormalizedUsage, PolicyInjectionMode,
    RunRequest, StatusLookupOutcome, UsageConfidence,
};

/// The `codex exec` flags this adapter always passes (machine interface, no session
/// persistence, no repo requirements, read-only sandbox), ended by `--`.
///
/// ⛔ The `--` is the SECURITY boundary of the prompt (`SIGNOFF-REPAIR.10.1.1`).
/// The prompt is untrusted participant content (`§16.6`) and follows these
/// flags as an argument. Without `--`, a prompt beginning with `-` is read by
/// the CLI's option parser, AFTER `--sandbox read-only`, so it could countermand
/// the sandbox the action boundary rests on. `claude.rs` already ended its flags
/// this way, and `ACTION-BOUNDARY` now pins the separator for both.
const EXEC_ARGS: &[&str] = &[
    "exec",
    "--json",
    "--skip-git-repo-check",
    "--ephemeral",
    "--sandbox",
    "read-only",
    "--",
];

/// The first real harness adapter: supervises `codex exec --json` as a child process.
pub struct CodexCliAdapter {
    binary: OsString,
    capabilities: AdapterCapabilities,
    /// One child per operation, so `cancel` can reach it while the handle streams.
    ///
    /// ⚠️ Shared with each handle, which removes its own entry when dropped
    /// (`SIGNOFF-REPAIR.10.1.3`): the map used to keep every child for the
    /// adapter's lifetime, so `kill_on_drop` never fired.
    children: crate::subprocess::Children,
}

impl Default for CodexCliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexCliAdapter {
    /// The adapter for the `codex` binary on PATH.
    pub fn new() -> Self {
        Self::with_binary("codex")
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
            children: crate::subprocess::Children::default(),
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

impl Adapter for CodexCliAdapter {
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
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(operation_id.to_string(), Arc::clone(&child));

        let (stderr_buffer, stderr_drain) = crate::subprocess::drain_stderr(stderr);

        InvokeOutcome::Accepted(
            DispatchAck {
                // Codex reveals its thread id in the stream's first event, not in the
                // ack — the handle surfaces it as `ProviderRequestId`.
                provider_request_id: None,
            },
            crate::contract::AttemptHandle::new(Box::new(CodexHandle {
                lines: BufReader::new(stdout),
                child,
                children: Arc::clone(&self.children),
                operation_id: operation_id.to_string(),
                stderr: stderr_buffer,
                stderr_drain,
                finished: false,
            })),
        )
    }

    async fn cancel(&self, operation_id: &str) -> CancellationOutcome {
        let child = {
            let children = self
                .children
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        // `codex exec` can RESUME a thread but has no first-class status query for a
        // past attempt. Honest Unsupported — the caller's only safe exit from
        // ambiguity is proof it cannot obtain, or adjudication.
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, raw_receipt: &Value) -> NormalizedUsage {
        let number = |key: &str| raw_receipt.get(key).and_then(|v| v.as_i64());
        let input_tokens = number("input_tokens");
        let output_tokens = number("output_tokens");
        let reasoning = number("reasoning_output_tokens").unwrap_or(0);
        let confidence = if input_tokens.is_some() || output_tokens.is_some() {
            // A `turn.completed` usage block IS the provider's authoritative receipt.
            UsageConfidence::Exact
        } else {
            UsageConfidence::Unknown
        };
        NormalizedUsage {
            input_tokens,
            // Reasoning tokens are output tokens (billable as such); fold them in.
            output_tokens: output_tokens.map(|o| o + reasoning),
            // The CLI stream reports tokens, never money: cost is unknown, not zero.
            cost: None,
            confidence,
        }
    }
}

/// The stream over one Codex child: JSONL events mapped to the contract, then the
/// exit status as the terminal verdict.
struct CodexHandle {
    lines: BufReader<ChildStdout>,
    child: Arc<Mutex<Child>>,
    /// The adapter's map and this attempt's key in it, so dropping the handle
    /// releases the child (`SIGNOFF-REPAIR.10.1.3`).
    children: crate::subprocess::Children,
    operation_id: String,
    /// The LAST bytes of the child's stderr (`crate::subprocess::drain_stderr`).
    stderr: Arc<Mutex<Vec<u8>>>,
    stderr_drain: tokio::task::JoinHandle<()>,
    finished: bool,
}

impl Drop for CodexHandle {
    fn drop(&mut self) {
        crate::subprocess::release(&self.children, &self.operation_id, &self.child);
    }
}

impl AttemptStream for CodexHandle {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        Box::pin(self.next_event())
    }
}

impl CodexHandle {
    async fn next_event(&mut self) -> Option<AttemptEvent> {
        if self.finished {
            return None;
        }
        loop {
            // `SIGNOFF-REPAIR.10.1.2`: at most `MAX_LINE_BYTES` of a line is ever
            // held; `read_line` held the whole line before anything was checked.
            match crate::subprocess::read_line_bounded(
                &mut self.lines,
                crate::subprocess::MAX_LINE_BYTES,
            )
            .await
            {
                Ok(crate::subprocess::Line::Eof) => {
                    // EOF: the child is done — the exit status is the terminal verdict.
                    // AWAIT the stderr drain first (bounded): the task may not have
                    // consumed the pipe's tail yet, and racing it leaves the reason
                    // EMPTY (the PHASE-1-MAINT-2 defect). The bound guards against a
                    // grandchild that inherited stderr keeping the pipe open.
                    // `SIGNOFF-REPAIR.10.1.3`: the drain, then the child, both
                    // bounded, and the child reaped; the wait used to be unbounded.
                    let status =
                        crate::subprocess::settle(&self.child, &mut self.stderr_drain).await;
                    // `SIGNOFF-REPAIR.10.1.2`: cut on a character boundary; the
                    // byte slice this replaced panicked inside a multi-byte one.
                    let stderr_tail = crate::subprocess::stderr_tail(&self.stderr.lock().await);
                    self.finished = true;
                    return match status {
                        Some(s) if s.success() => None, // no turn.completed seen: lost response
                        Some(s) => Some(AttemptEvent::FailedKnown {
                            reason: format!(
                                "codex exited with {s}; stderr tail: {}",
                                stderr_tail.trim()
                            ),
                        }),
                        None => Some(AttemptEvent::FailedKnown {
                            reason: "codex child vanished without an exit status".to_string(),
                        }),
                    };
                }
                // A line that is not UTF-8 cannot be a JSONL event, and is
                // skipped as a human or status line is.
                Ok(crate::subprocess::Line::NotUtf8) => continue,
                Ok(crate::subprocess::Line::TooLong) => {
                    // ⛔ A definitive failure naming the bound, as the supervisor
                    // treats output over ITS bound: never a silently shortened
                    // result, and never a line held whole to find out.
                    let _ = self.child.lock().await.start_kill();
                    let _ = crate::subprocess::settle(&self.child, &mut self.stderr_drain).await;
                    self.finished = true;
                    return Some(AttemptEvent::FailedKnown {
                        reason: format!(
                            "a codex event line exceeded the {}-byte bound; the child was stopped",
                            crate::subprocess::MAX_LINE_BYTES
                        ),
                    });
                }
                Ok(crate::subprocess::Line::Text(line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let Ok(event) = serde_json::from_str::<Value>(trimmed) else {
                        continue; // human/status lines are not events; never parse them
                    };
                    match event.get("type").and_then(|t| t.as_str()) {
                        Some("thread.started") => {
                            if let Some(id) = event.get("thread_id").and_then(|v| v.as_str()) {
                                return Some(AttemptEvent::ProviderRequestId {
                                    request_id: id.to_string(),
                                });
                            }
                        }
                        Some("item.completed") => {
                            if let Some(text) = event.pointer("/item/text").and_then(|t| t.as_str())
                            {
                                return Some(AttemptEvent::OutputChunk {
                                    chunk: text.to_string(),
                                });
                            }
                        }
                        Some("turn.completed") => {
                            let usage = event.get("usage").cloned();
                            // `SIGNOFF-REPAIR.10.1.3`: the completion ENDS the
                            // attempt, so the child is settled before it is
                            // reported; this arm used to return at once, leaving
                            // the child unreaped and the drain running.
                            let _ = crate::subprocess::settle(&self.child, &mut self.stderr_drain)
                                .await;
                            self.finished = true;
                            return Some(AttemptEvent::Completed { usage });
                        }
                        _ => continue,
                    }
                }
                Err(_) => return None,
            }
        }
    }
}
