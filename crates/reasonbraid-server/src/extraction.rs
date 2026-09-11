//! The R2 extraction pipeline (PHASE-4.4.3): the server-side spawner +
//! the Derivation receipt. The spawner writes the acquired bytes to a temp
//! file, sends ONE request line to the worker process, reads ONE response
//! line, and KILLS the worker when the time budget trips — the killing
//! budget is the quarantine's enforcement, the fresh process its boundary.
//!
//! Every return also carries EXPLICIT direct-child completion evidence
//! (`SIGNOFF-REPAIR.7.3.3.2.2`): whether a child was ever started, whether
//! this process observed its exit and reaped it, or whether a bounded stop
//! left that unresolved. No path abandons a live child, and no path claims a
//! termination it did not observe. Input lifetime and cleanup decisions that
//! depend on the reader being finished belong to `run_extraction_reporting`;
//! `SIGNOFF-REPAIR.7.3.3.3` owns migrating the R2 API caller onto it.

use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// How long a worker whose exchange ended abnormally may finish on its own
/// (its stdin is already closed) before the stop escalates to a signal.
const STOP_GRACE: Duration = Duration::from_millis(500);

/// The whole stop/reap bound, measured from the end of the exchange. It caps
/// the spawner's own cleanup so a stuck worker cannot hold the caller open.
const STOP_BUDGET: Duration = Duration::from_secs(5);

/// The poll step shared by the exchange wait and the stop wait.
const POLL_STEP: Duration = Duration::from_millis(10);

/// The derived chunk (the worker's wire shape, mirrored).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DerivedChunk {
    pub digest: String,
    pub text: String,
}

/// The Derivation receipt (the `.4.1` contract): the parent digest, the
/// derived chunks (each with its own ADR-011 digest), and the extractor
/// version — the extraction is ALWAYS a Derivation, never the original.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ExtractionReceipt {
    pub parent_digest: String,
    pub chunks: Vec<DerivedChunk>,
    pub excluded: Vec<String>,
    pub extractor_version: String,
    pub requested_url: String,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
}

/// The worker's wire response (the success shape).
#[derive(Debug, Deserialize)]
pub struct WorkerResponse {
    pub parent_digest: String,
    pub chunks: Vec<DerivedChunk>,
    pub excluded: Vec<String>,
    pub extractor_version: String,
}

/// The worker's refusal (the `{error: {kind, message}}` envelope).
#[derive(Debug, Deserialize)]
pub struct WorkerErrorEnvelope {
    pub error: WorkerErrorBody,
}

#[derive(Debug, Deserialize)]
pub struct WorkerErrorBody {
    pub kind: String,
    pub message: String,
}

/// The extraction failure — the worker's named refusal surfaces verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractionError {
    WorkerMissing(String),
    SpawnFailed(String),
    RequestFailed(String),
    TimedOut,
    WorkerRefused { kind: String, message: String },
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerMissing(path) => write!(f, "the extraction worker `{path}` is absent"),
            Self::SpawnFailed(detail) => {
                write!(f, "the extraction worker failed to spawn: {detail}")
            }
            Self::RequestFailed(detail) => write!(f, "the extraction request failed: {detail}"),
            // The budget trip is the fact; whether the stop was observed is
            // the separate `WorkerCompletion` evidence, never asserted here.
            Self::TimedOut => write!(f, "the extraction exceeded the time budget"),
            Self::WorkerRefused { kind, message } => {
                write!(f, "the worker refused: {kind}: {message}")
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

/// What this process knows about the direct worker child it created.
///
/// This is evidence about ONE process — the direct child. It says nothing
/// about descendants that child may have started; bounded pipes, descendant
/// containment and aggregate retained storage remain `SIGNOFF-REPAIR.7.3.4`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerCompletion {
    /// No direct child was created: the worker binary was absent, or the
    /// spawn itself failed. There is nothing to stop, reap or wait for.
    NeverStarted,
    /// This process observed the direct child's exit AND reaped it. The
    /// status is the platform's own rendering of that exit.
    Consumed { success: bool, status: String },
    /// The bounded stop did not observe the direct child's exit. The child
    /// may still be running. `detail` records what was attempted and what
    /// failed; a caller must not read this as a termination.
    Unconfirmed { pid: u32, detail: String },
}

impl WorkerCompletion {
    /// True when this process knows the direct child is finished and reaped.
    /// A caller that must not delete a worker's input before the reader is
    /// done checks THIS, never the request's success alone.
    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed { .. })
    }
}

impl fmt::Display for WorkerCompletion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeverStarted => write!(f, "no worker process was started"),
            Self::Consumed { status, .. } => {
                write!(f, "the worker was consumed ({status})")
            }
            Self::Unconfirmed { pid, detail } => {
                write!(
                    f,
                    "the worker (pid {pid}) completion is unconfirmed: {detail}"
                )
            }
        }
    }
}

/// One extraction attempt: the request's own outcome AND the completion
/// evidence for the direct child that served it. The two are independent
/// facts — a named refusal is still a finished worker, and a tripped budget
/// says nothing by itself about whether the stop was observed.
#[derive(Debug)]
pub struct ExtractionRun {
    pub result: Result<WorkerResponse, ExtractionError>,
    pub completion: WorkerCompletion,
}

/// The worker binary's path: the `R2_WORKER_BIN` override, or the default —
/// next to the server's own binary (the deployment ships them together; in
/// the dev target dir the worker sits beside the server bin's PARENT dir).
pub fn worker_path() -> PathBuf {
    if let Ok(path) = std::env::var("R2_WORKER_BIN") {
        return PathBuf::from(path);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|dir| {
            // target/debug/deps/<bin> -> target/debug — the worker lives
            // beside the server binary.
            if dir.ends_with("deps") {
                dir.parent().map(|p| p.to_path_buf())
            } else {
                Some(dir)
            }
        })
        .map(|dir| dir.join("reasonbraid-extract"))
        .unwrap_or_else(|| PathBuf::from("reasonbraid-extract"))
}

/// Run one extraction: the temp file + the media type + the ceilings in,
/// the Derivation response (or the named refusal) out. The time budget
/// KILLS the worker on the trip.
///
/// This entrypoint keeps the established call shape and error classification.
/// It does NOT carry the direct-child completion evidence: a caller whose
/// input lifetime or cleanup depends on the worker having finished must use
/// [`run_extraction_reporting`] (`SIGNOFF-REPAIR.7.3.3.3` migrates the R2 API
/// caller). An `Ok` from either entrypoint is only produced after the exchange
/// observed the child's exit, so it always carries a consumed completion.
pub fn run_extraction(
    input_path: &std::path::Path,
    media_type: &str,
    limits: WorkerLimits,
    time_budget: Duration,
) -> Result<WorkerResponse, ExtractionError> {
    run_extraction_reporting(input_path, media_type, limits, time_budget).result
}

/// Run one extraction and report the direct child's completion beside the
/// request's outcome. Every return path passes through ONE bounded stop/reap,
/// so no early failure leaves a live worker behind with nobody waiting for it.
pub fn run_extraction_reporting(
    input_path: &std::path::Path,
    media_type: &str,
    limits: WorkerLimits,
    time_budget: Duration,
) -> ExtractionRun {
    let binary = worker_path();
    if !binary.exists() {
        return ExtractionRun {
            result: Err(ExtractionError::WorkerMissing(binary.display().to_string())),
            completion: WorkerCompletion::NeverStarted,
        };
    }
    let request = serde_json::json!({
        "input_path": input_path.display().to_string(),
        "media_type": media_type,
        "limits": limits,
    });
    let mut child = match Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return ExtractionRun {
                result: Err(ExtractionError::SpawnFailed(error.to_string())),
                completion: WorkerCompletion::NeverStarted,
            }
        }
    };
    let result = exchange(&mut child, &request, time_budget);
    // The budget already tripped on a timeout: stop at once rather than
    // granting a worker that outran its ceiling more time to finish.
    let grace = if matches!(result, Err(ExtractionError::TimedOut)) {
        Duration::ZERO
    } else {
        STOP_GRACE
    };
    let completion = stop_and_reap(&mut child, grace, STOP_BUDGET);
    ExtractionRun { result, completion }
}

/// The request/response exchange with one already-spawned worker. Every
/// failure returns; stopping and reaping the child is the caller's single
/// bounded step, so no branch here can abandon it.
fn exchange(
    child: &mut std::process::Child,
    request: &serde_json::Value,
    time_budget: Duration,
) -> Result<WorkerResponse, ExtractionError> {
    {
        use std::io::Write;
        // The pipes were requested at spawn; a missing handle is a refusal,
        // never a panic inside the control plane.
        let mut stdin = child.stdin.take().ok_or_else(|| {
            ExtractionError::RequestFailed("the worker stdin is unavailable".to_owned())
        })?;
        writeln!(stdin, "{request}").map_err(|e| ExtractionError::RequestFailed(e.to_string()))?;
    }
    let deadline = Instant::now() + time_budget;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| ExtractionError::RequestFailed(e.to_string()))?
        {
            if !status.success() {
                return Err(ExtractionError::RequestFailed(format!(
                    "the worker exited with {status}"
                )));
            }
            break;
        }
        if Instant::now() >= deadline {
            return Err(ExtractionError::TimedOut);
        }
        std::thread::sleep(POLL_STEP);
    }
    let mut output = String::new();
    child
        .stdout
        .take()
        .ok_or_else(|| {
            ExtractionError::RequestFailed("the worker stdout is unavailable".to_owned())
        })?
        .read_to_string(&mut output)
        .map_err(|e| ExtractionError::RequestFailed(e.to_string()))?;
    if let Ok(envelope) = serde_json::from_str::<WorkerErrorEnvelope>(&output) {
        return Err(ExtractionError::WorkerRefused {
            kind: envelope.error.kind,
            message: envelope.error.message,
        });
    }
    serde_json::from_str::<WorkerResponse>(&output)
        .map_err(|e| ExtractionError::RequestFailed(format!("the response failed to parse: {e}")))
}

/// The one process this spawner owns, reduced to what the stop needs. The
/// seam exists so the classification can be exercised against injected
/// outcomes that no cooperative worker reproduces natively.
trait DirectChild {
    fn pid(&self) -> u32;
    /// `Some((success, rendered status))` once the child has exited and been
    /// reaped; `None` while it is still running.
    fn poll(&mut self) -> std::io::Result<Option<(bool, String)>>;
    /// Ask the operating system to terminate the child.
    fn request_stop(&mut self) -> std::io::Result<()>;
}

impl DirectChild for std::process::Child {
    fn pid(&self) -> u32 {
        self.id()
    }

    fn poll(&mut self) -> std::io::Result<Option<(bool, String)>> {
        Ok(self
            .try_wait()?
            .map(|status| (status.success(), status.to_string())))
    }

    fn request_stop(&mut self) -> std::io::Result<()> {
        self.kill()
    }
}

/// Poll until the child is observed finished or the deadline passes. A zero
/// budget still polls exactly once — a child that already exited is reaped
/// without waiting.
fn wait_until<C: DirectChild>(
    child: &mut C,
    deadline: Instant,
) -> std::io::Result<Option<WorkerCompletion>> {
    loop {
        if let Some((success, status)) = child.poll()? {
            return Ok(Some(WorkerCompletion::Consumed { success, status }));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(POLL_STEP);
    }
}

/// Stop and reap the direct child within `budget`, and say honestly which of
/// those two this process actually observed.
///
/// Phase one gives a worker whose stdin is already closed `grace` to finish on
/// its own; phase two asks the operating system to terminate it and waits out
/// the remaining budget. A stop request that fails is recorded and the wait
/// continues — the child may exit anyway — but it never turns into a claimed
/// termination.
fn stop_and_reap<C: DirectChild>(
    child: &mut C,
    grace: Duration,
    budget: Duration,
) -> WorkerCompletion {
    let started = Instant::now();
    let pid = child.pid();
    match wait_until(child, started + grace) {
        Ok(Some(completion)) => return completion,
        Ok(None) => (),
        Err(error) => {
            return WorkerCompletion::Unconfirmed {
                pid,
                detail: format!("the worker status is unreadable: {error}"),
            }
        }
    }
    let stop_failure = child.request_stop().err().map(|error| error.to_string());
    match wait_until(child, started + budget) {
        Ok(Some(completion)) => completion,
        Ok(None) => WorkerCompletion::Unconfirmed {
            pid,
            detail: match stop_failure {
                Some(failure) => format!(
                    "the stop request failed ({failure}) and the worker did not exit within {budget:?}"
                ),
                None => format!("the worker did not exit within {budget:?} of the stop request"),
            },
        },
        Err(error) => WorkerCompletion::Unconfirmed {
            pid,
            detail: format!("the worker status is unreadable after the stop request: {error}"),
        },
    }
}

/// The worker's ceilings (mirrored from the worker crate's wire shape).
#[derive(Debug, Clone, Serialize)]
pub struct WorkerLimits {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_chunks: usize,
    pub max_entry_bytes: u64,
    pub max_decompression_ratio: f64,
}

impl Default for WorkerLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024,
            max_output_bytes: 4 * 1024 * 1024,
            max_chunks: 256,
            max_entry_bytes: 4 * 1024 * 1024,
            max_decompression_ratio: 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A child that never exits. This is SYNTHETIC injection: no cooperative
    /// worker reproduces an unkillable process on demand, so the honest
    /// classification of that case is proved here rather than claimed.
    struct StubChild {
        exits_after_polls: Option<usize>,
        polls: usize,
        stops: usize,
        poll_error: Option<std::io::ErrorKind>,
        stop_error: Option<std::io::ErrorKind>,
    }

    impl StubChild {
        fn never_exits() -> Self {
            Self {
                exits_after_polls: None,
                polls: 0,
                stops: 0,
                poll_error: None,
                stop_error: None,
            }
        }
    }

    impl DirectChild for StubChild {
        fn pid(&self) -> u32 {
            4242
        }

        fn poll(&mut self) -> std::io::Result<Option<(bool, String)>> {
            self.polls += 1;
            if let Some(kind) = self.poll_error {
                return Err(std::io::Error::from(kind));
            }
            match self.exits_after_polls {
                Some(threshold) if self.polls >= threshold => {
                    Ok(Some((true, "exit status: 0".to_owned())))
                }
                _ => Ok(None),
            }
        }

        fn request_stop(&mut self) -> std::io::Result<()> {
            self.stops += 1;
            match self.stop_error {
                Some(kind) => Err(std::io::Error::from(kind)),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn a_child_that_exits_in_the_grace_window_is_never_signalled() {
        let mut child = StubChild {
            exits_after_polls: Some(2),
            ..StubChild::never_exits()
        };
        let completion = stop_and_reap(
            &mut child,
            Duration::from_millis(200),
            Duration::from_millis(400),
        );
        assert_eq!(
            completion,
            WorkerCompletion::Consumed {
                success: true,
                status: "exit status: 0".to_owned()
            }
        );
        assert!(completion.is_consumed());
        assert_eq!(
            child.stops, 0,
            "a worker that finished must not be signalled"
        );
    }

    #[test]
    fn a_child_that_never_exits_is_reported_unconfirmed_not_terminated() {
        let mut child = StubChild::never_exits();
        let completion = stop_and_reap(&mut child, Duration::ZERO, Duration::from_millis(30));
        match &completion {
            WorkerCompletion::Unconfirmed { pid, detail } => {
                assert_eq!(*pid, 4242);
                assert!(detail.contains("did not exit"), "{detail}");
            }
            other => panic!("an unstoppable worker must stay unconfirmed: {other:?}"),
        }
        assert!(!completion.is_consumed());
        assert_eq!(child.stops, 1, "the stop is attempted exactly once");
    }

    #[test]
    fn a_failed_stop_request_is_recorded_in_the_unconfirmed_evidence() {
        let mut child = StubChild {
            stop_error: Some(std::io::ErrorKind::PermissionDenied),
            ..StubChild::never_exits()
        };
        let completion = stop_and_reap(&mut child, Duration::ZERO, Duration::from_millis(30));
        match completion {
            WorkerCompletion::Unconfirmed { detail, .. } => {
                assert!(detail.contains("the stop request failed"), "{detail}");
            }
            other => panic!("a refused stop must stay unconfirmed: {other:?}"),
        }
    }

    #[test]
    fn an_unreadable_status_never_becomes_a_claimed_termination() {
        let mut child = StubChild {
            poll_error: Some(std::io::ErrorKind::Other),
            ..StubChild::never_exits()
        };
        let completion = stop_and_reap(
            &mut child,
            Duration::from_millis(10),
            Duration::from_millis(30),
        );
        match completion {
            WorkerCompletion::Unconfirmed { detail, .. } => {
                assert!(detail.contains("unreadable"), "{detail}");
            }
            other => panic!("an unreadable status must stay unconfirmed: {other:?}"),
        }
        assert_eq!(
            child.stops, 0,
            "an unreadable status stops before signalling"
        );
    }

    /// A child that already exited is reaped by the first poll even with no
    /// grace at all — the timeout path pays no extra wait.
    #[test]
    fn a_zero_grace_stop_still_reaps_an_already_finished_child() {
        let mut child = StubChild {
            exits_after_polls: Some(1),
            ..StubChild::never_exits()
        };
        let completion = stop_and_reap(&mut child, Duration::ZERO, Duration::from_millis(30));
        assert!(completion.is_consumed());
        assert_eq!(child.polls, 1);
        assert_eq!(child.stops, 0);
    }

    /// The spawner roundtrip against the REAL worker binary (the workspace
    /// builds it beside the server's own — the skip keeps the -p-only run
    /// green like the profiles' DATABASE_URL skip).
    #[test]
    fn the_spawner_extracts_through_the_worker_and_surfaces_the_refusal() {
        let binary = worker_path();
        if !binary.exists() {
            println!(
                "SKIP: the extraction worker is absent at {binary:?} — run `cargo test --all`"
            );
            return;
        }
        let input_path = std::env::temp_dir().join(format!(
            "r2-server-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        let feed = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Spawner Feed</title>
  <entry><title>One</title><summary>the first</summary></entry>
</feed>"#;
        std::fs::write(&input_path, feed).expect("the input writes");
        let run = run_extraction_reporting(
            &input_path,
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        );
        assert!(
            run.completion.is_consumed(),
            "the real worker must be consumed: {}",
            run.completion
        );
        let response = run.result.expect("the extraction runs");
        assert!(response.parent_digest.starts_with("sha256:"));
        assert_eq!(response.extractor_version, "0.1.0");
        let joined: String = response
            .chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("Spawner Feed"), "{joined}");
        assert!(joined.contains("One"), "{joined}");

        // The worker's refusal surfaces verbatim with its kind, and the
        // refusing worker is still a finished, reaped one.
        std::fs::write(&input_path, b"not a feed").expect("the input rewrites");
        let run = run_extraction_reporting(
            &input_path,
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        );
        assert!(
            run.completion.is_consumed(),
            "a refusing worker is still consumed: {}",
            run.completion
        );
        match run.result {
            Err(ExtractionError::WorkerRefused { kind, message }) => {
                assert_eq!(kind, "feed_unreadable");
                assert!(message.contains("feed"), "{message}");
            }
            other => panic!("the refusal must surface: {other:?}"),
        }
        std::fs::remove_file(&input_path).ok();
    }
}
