//! The R3 browser pipeline (PHASE-4.5.3): the server-side spawner + the
//! render receipt. The spawner sends ONE request line to the browser
//! worker (the URL is ALREADY classified — the caller's pre-flight ran
//! before the spawn), reads ONE response line, and KILLS the worker when
//! the time budget trips — the killing budget inside the deployment's
//! container boundary is the quarantine's enforcement.
//!
//! ⛔ The pre-flight classifies ONE url. Everything the PAGE then asks for is
//! bounded by the worker's own enforcement of the two advertised deny-policies
//! (`SIGNOFF-REPAIR.7.3.5`), and `BrowserReceipt::refused_requests` carries
//! what it refused — a denial nobody can see is indistinguishable from a page
//! that never asked.

use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The worker's wire response (the success shape).
#[derive(Debug, Deserialize)]
pub struct BrowseWorkerResponse {
    /// The page's OWN bytes — the serialized document the chunks derive from,
    /// and the artefact the `EvidenceSnapshot` stores
    /// (`SIGNOFF-REPAIR.11.24.1.3.1`).
    ///
    /// ⛔ REQUIRED, with no `#[serde(default)]`, and that is the opposite
    /// choice from `refused_requests` below — deliberately. An older worker
    /// that performed no refusals genuinely has none to report, so a default
    /// is the truth. An older worker that sends no document has not rendered
    /// nothing; it has rendered something this server cannot record, and
    /// defaulting to an empty string would store an empty snapshot and call it
    /// the page. The decode failure is the honest outcome, and the two
    /// binaries ship together.
    pub document: String,
    /// The digest of `document` — the PARENT of every chunk below.
    pub parent_digest: String,
    pub chunks: Vec<BrowseChunk>,
    pub network_log: Vec<NetworkEntry>,
    /// Every request the worker's two advertised deny-policies refused
    /// (`SIGNOFF-REPAIR.7.3.5`). `#[serde(default)]` so a worker built before
    /// the field existed still parses — an older worker sends no refusals
    /// because it performed none.
    #[serde(default)]
    pub refused_requests: Vec<RefusedRequest>,
    pub page_title: String,
    pub browser_version: String,
    pub worker_version: String,
}

/// One request the R3 worker refused, named on the receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefusedRequest {
    pub url: String,
    /// `subresource` or `redirect` — the advertised policy that refused it.
    pub policy: String,
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowseChunk {
    pub digest: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkEntry {
    pub url: String,
    pub method: String,
}

/// The render receipt (the `.5.1` contract's Derivation + disclosure): the
/// rendered chunks (each with its own ADR-011 digest), the network log
/// (every request the page made — the disclosure), and the browser
/// version.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BrowserReceipt {
    pub parent_digest: String,
    pub chunks: Vec<BrowseChunk>,
    pub network_log: Vec<NetworkEntry>,
    /// What the pack's advertised deny-policies actually refused on this
    /// render. ⛔ A denial nobody can see is indistinguishable from a page that
    /// never asked.
    pub refused_requests: Vec<RefusedRequest>,
    pub page_title: String,
    pub browser_version: String,
    pub requested_url: String,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseError {
    WorkerMissing(String),
    SpawnFailed(String),
    RequestFailed(String),
    TimedOut,
    WorkerRefused { kind: String, message: String },
}

impl fmt::Display for BrowseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerMissing(path) => write!(f, "the browser worker `{path}` is absent"),
            Self::SpawnFailed(detail) => write!(f, "the browser worker failed to spawn: {detail}"),
            Self::RequestFailed(detail) => write!(f, "the browser request failed: {detail}"),
            Self::TimedOut => {
                write!(
                    f,
                    "the render exceeded the time budget (the worker was killed)"
                )
            }
            Self::WorkerRefused { kind, message } => {
                write!(f, "the browser worker refused: {kind}: {message}")
            }
        }
    }
}

impl std::error::Error for BrowseError {}

/// The browser worker's binary: the `R3_BROWSER_BIN` override, or the
/// server-binary-adjacent default (the same derivation the R2 spawner
/// uses).
pub fn browse_worker_path() -> PathBuf {
    if let Ok(path) = std::env::var("R3_WORKER_BIN") {
        return PathBuf::from(path);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|dir| {
            if dir.ends_with("deps") {
                dir.parent().map(|p| p.to_path_buf())
            } else {
                Some(dir)
            }
        })
        .map(|dir| dir.join("reasonbraid-browse"))
        .unwrap_or_else(|| PathBuf::from("reasonbraid-browse"))
}

/// Run one render: the already-classified URL + the steps + the budgets
/// in, the Derivation response (or the named refusal) out. The time
/// budget KILLS the worker on the trip.
pub fn run_browse(url: &str, time_budget: Duration) -> Result<BrowseWorkerResponse, BrowseError> {
    let binary = browse_worker_path();
    if !binary.exists() {
        return Err(BrowseError::WorkerMissing(binary.display().to_string()));
    }
    let request = serde_json::json!({
        "url": url,
        "steps": [{ "action": "navigate", "url": url }],
        "limits": {
            "max_steps": 4,
            "max_output_bytes": 4 * 1024 * 1024,
            "time_budget_secs": time_budget.as_secs().max(1)
        }
    });
    let mut child = Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| BrowseError::SpawnFailed(e.to_string()))?;
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().expect("the worker stdin piped");
        writeln!(stdin, "{request}").map_err(|e| BrowseError::RequestFailed(e.to_string()))?;
    }
    let mut output = String::new();
    let deadline = std::time::Instant::now() + time_budget;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| BrowseError::RequestFailed(e.to_string()))?
        {
            if !status.success() {
                return Err(BrowseError::RequestFailed(format!(
                    "the worker exited with {status}"
                )));
            }
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(BrowseError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child
        .stdout
        .take()
        .expect("the worker stdout piped")
        .read_to_string(&mut output)
        .map_err(|e| BrowseError::RequestFailed(e.to_string()))?;
    if let Ok(envelope) = serde_json::from_str::<serde_json::Value>(&output) {
        if let Some(error) = envelope.get("error") {
            return Err(BrowseError::WorkerRefused {
                kind: error["kind"].as_str().unwrap_or("unknown").to_owned(),
                message: error["message"].as_str().unwrap_or("unknown").to_owned(),
            });
        }
    }
    serde_json::from_str::<BrowseWorkerResponse>(&output)
        .map_err(|e| BrowseError::RequestFailed(format!("the response failed to parse: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SIGNOFF-REPAIR.11.24.1.3.1.1` — the worker now sends the page's own
    /// bytes beside their digest, and this server decodes that response.
    ///
    /// ⛔ The compatibility is DRIVEN rather than assumed. `BrowseWorkerResponse`
    /// carries no `deny_unknown_fields`, so a new field is ignored — but
    /// "ignored" is a property of an attribute that is absent, and an absent
    /// attribute is exactly the kind of thing a later edit adds without
    /// noticing what it breaks. This control fails the moment it is added.
    ///
    /// ⭐ It also pins what the server currently does with the document:
    /// nothing. `parent_digest` is the digest of the page's bytes rather than
    /// of the chunk derived from them, which is the whole of this leaf's
    /// user-visible effect; storing the bytes is `.11.24.1.3.1`'s.
    #[test]
    fn a_worker_response_carrying_the_document_decodes_and_keeps_its_parent() {
        let document = "<html><body><p>rendered</p></body></html>";
        let payload = serde_json::json!({
            "document": document,
            "parent_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000001",
            "chunks": [{
                "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000002",
                "text": "rendered",
            }],
            "network_log": [],
            "refused_requests": [],
            "page_title": "t",
            "browser_version": "v",
            "worker_version": "w",
        });
        let decoded: BrowseWorkerResponse =
            serde_json::from_value(payload).expect("the server decodes the worker's response");
        assert_eq!(
            decoded.parent_digest,
            "sha256:0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(decoded.chunks.len(), 1);
        assert_ne!(
            decoded.parent_digest, decoded.chunks[0].digest,
            "the parent and the derivation are two different artefacts on the wire"
        );
    }
}
