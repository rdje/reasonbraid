//! The R2 extraction pipeline (PHASE-4.4.3): the server-side spawner +
//! the Derivation receipt. The spawner writes the acquired bytes to a temp
//! file, sends ONE request line to the worker process, reads ONE response
//! line, and KILLS the worker when the time budget trips — the killing
//! budget is the quarantine's enforcement, the fresh process its boundary.

use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};

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
            Self::TimedOut => write!(
                f,
                "the extraction exceeded the time budget (the worker was killed)"
            ),
            Self::WorkerRefused { kind, message } => {
                write!(f, "the worker refused: {kind}: {message}")
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

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
pub fn run_extraction(
    input_path: &std::path::Path,
    media_type: &str,
    limits: WorkerLimits,
    time_budget: Duration,
) -> Result<WorkerResponse, ExtractionError> {
    let binary = worker_path();
    if !binary.exists() {
        return Err(ExtractionError::WorkerMissing(binary.display().to_string()));
    }
    let request = serde_json::json!({
        "input_path": input_path.display().to_string(),
        "media_type": media_type,
        "limits": limits,
    });
    let mut child = Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| ExtractionError::SpawnFailed(e.to_string()))?;
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().expect("the worker stdin piped");
        writeln!(stdin, "{request}").map_err(|e| ExtractionError::RequestFailed(e.to_string()))?;
    }
    let mut output = String::new();
    let deadline = std::time::Instant::now() + time_budget;
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
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExtractionError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child
        .stdout
        .take()
        .expect("the worker stdout piped")
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
        let response = run_extraction(
            &input_path,
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        )
        .expect("the extraction runs");
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

        // The worker's refusal surfaces verbatim with its kind.
        std::fs::write(&input_path, b"not a feed").expect("the input rewrites");
        match run_extraction(
            &input_path,
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        ) {
            Err(ExtractionError::WorkerRefused { kind, message }) => {
                assert_eq!(kind, "feed_unreadable");
                assert!(message.contains("feed"), "{message}");
            }
            other => panic!("the refusal must surface: {other:?}"),
        }
        std::fs::remove_file(&input_path).ok();
    }
}
