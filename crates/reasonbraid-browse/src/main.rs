//! The R3 browser worker (PHASE-4.5.2): the stdio JSON protocol, the
//! bounded interaction (the step budget + the wall-clock ceiling), the
//! network log (every request the page makes is recorded — the disclosure),
//! and the refusal vocabulary — every refusal names its kind. The rendered
//! text is ALWAYS a Derivation (the parent digest + the derived chunks).

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod lifetime;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod storage;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

/// The worker's wire request: the already-classified URL (the server's
/// pre-flight ran BEFORE the spawn — the worker trusts the caller) + the
/// bounded interaction + the ceilings.
#[derive(Debug, Deserialize)]
pub struct BrowseRequest {
    pub url: String,
    pub steps: Vec<BrowseStep>,
    pub limits: BrowseLimits,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BrowseStep {
    Navigate { url: String },
    Click { selector: String },
    Scroll { y: i64 },
    Type { selector: String, text: String },
}

#[derive(Debug, Deserialize)]
pub struct BrowseLimits {
    pub max_steps: usize,
    pub max_output_bytes: u64,
    pub time_budget_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct BrowseResponse {
    pub parent_digest: String,
    pub chunks: Vec<DerivedChunk>,
    pub network_log: Vec<NetworkEntry>,
    pub page_title: String,
    pub browser_version: String,
    pub worker_version: String,
}

#[derive(Debug, Serialize)]
pub struct DerivedChunk {
    pub digest: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkEntry {
    pub url: String,
    pub method: String,
}

/// A refusal carries TWO independent facts, never one collapsed into the other.
///
/// `kind` names the RENDER's own outcome, because that is the fact the caller
/// must act on: a tripped budget is the caller's own limit, a `click_failed` is
/// its own selector. `cleanup_confirmed` names what the worker OBSERVED about
/// the browser process and its owned tasks, because that is the fact an
/// operator must act on. A single field cannot carry both, and the one that
/// used to be overwritten was the caller's.
///
/// `cleanup_confirmed` means: no owned browser process or task is known to
/// outlive this worker. A refusal raised before any browser was spawned
/// satisfies it by construction — nothing was started, so nothing can remain —
/// and that is the only way it is ever true without an observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowseError {
    pub kind: String,
    pub message: String,
    pub cleanup_confirmed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_error: Option<String>,
}

impl BrowseError {
    /// A refusal raised with no owned browser behind it.
    pub fn refused(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            cleanup_confirmed: true,
            cleanup_error: None,
        }
    }
}

/// Combine the render's outcome with the cleanup's outcome into one response.
///
/// The doctrine is that the worker never claims an unobserved termination. It
/// is satisfied by CARRYING the cleanup fact, not by destroying the render's —
/// so a render that already failed keeps its own kind and gains the cleanup
/// fact beside it. A render that SUCCEEDED has no failure of its own to name,
/// and must not read as complete while a browser may still be running: that
/// one becomes `browser_cleanup_unconfirmed`, as it always has.
///
/// The cleanup detail is also appended to the message. That duplication is
/// deliberate: the server-side spawner reads only `kind` and `message`, so
/// until it reads the fields the message is the only channel that carries the
/// operator's fact into its log. Both are written from one expression here and
/// cannot drift apart.
pub fn settle(
    render: Result<BrowseResponse, (String, String)>,
    cleanup: Result<(), String>,
) -> Result<BrowseResponse, BrowseError> {
    match (render, cleanup) {
        (Ok(response), Ok(())) => Ok(response),
        (Ok(_), Err(cleanup)) => Err(BrowseError {
            kind: "browser_cleanup_unconfirmed".to_owned(),
            message: cleanup.clone(),
            cleanup_confirmed: false,
            cleanup_error: Some(cleanup),
        }),
        (Err((kind, message)), Ok(())) => Err(BrowseError {
            kind,
            message,
            cleanup_confirmed: true,
            cleanup_error: None,
        }),
        (Err((kind, message)), Err(cleanup)) => Err(BrowseError {
            kind,
            message: format!("{message}; {cleanup}"),
            cleanup_confirmed: false,
            cleanup_error: Some(cleanup),
        }),
    }
}

pub const WORKER_VERSION: &str = "0.1.0";

pub fn digest_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// The browser binary: the `R3_BROWSER_BIN` override, or the platform
/// defaults (the pinned chromium's named provenance — the startup check
/// refuses to run without it).
pub fn browser_binary() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("R3_BROWSER_BIN") {
        return Some(std::path::PathBuf::from(path));
    }
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
    ];
    candidates
        .iter()
        .map(std::path::PathBuf::from)
        .find(|path| path.exists())
}

fn main() {
    let mut input = String::new();
    let request: BrowseRequest =
        match std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
            .map_err(|e| e.to_string())
            .and_then(|_| serde_json::from_str(&input).map_err(|e| e.to_string()))
        {
            Ok(request) => request,
            Err(message) => {
                respond_error("request_unreadable", &message);
                return;
            }
        };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("the worker runtime builds");
    match runtime.block_on(run(&request)) {
        Ok(response) => respond(&response),
        Err(error) => respond(&serde_json::json!({ "error": error })),
    }
}

fn respond(payload: &impl Serialize) {
    if let Ok(json) = serde_json::to_string(payload) {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "{json}");
        let _ = stdout.flush();
    }
}

fn respond_error(kind: &str, message: &str) {
    respond(&serde_json::json!({ "error": BrowseError::refused(kind, message) }));
}

/// The bounded interaction: the step budget first, then the wall-clock
/// ceiling over the whole render (the trip refuses, never hangs).
async fn run(request: &BrowseRequest) -> Result<BrowseResponse, BrowseError> {
    if request.steps.len() > request.limits.max_steps {
        return Err(BrowseError::refused(
            "step_budget_exceeded",
            format!(
                "{} steps exceeds the {}-step budget",
                request.steps.len(),
                request.limits.max_steps
            ),
        ));
    }
    if request.steps.is_empty() {
        return Err(BrowseError::refused(
            "no_steps",
            "the request carries no steps",
        ));
    }
    run_browser(request).await
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn run_browser(_: &BrowseRequest) -> Result<BrowseResponse, BrowseError> {
    Err(BrowseError::refused(
        "browser_platform_unsupported",
        "owned browser processes currently require Linux or macOS",
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn run_browser(request: &BrowseRequest) -> Result<BrowseResponse, BrowseError> {
    // These two refusals precede the owned browser: nothing is spawned, so
    // there is nothing a cleanup could leave behind.
    let binary = browser_binary()
        .ok_or_else(|| {
            BrowseError::refused(
                "browser_missing",
                "no browser binary found — set R3_BROWSER_BIN to the pinned chromium",
            )
        })?
        .canonicalize()
        .map_err(|e| BrowseError::refused("browser_launch_failed", e.to_string()))?;
    let mut owner = lifetime::Lifetime::new()
        .map_err(|e| BrowseError::refused("browser_storage_failed", e.to_string()))?;
    let budget = std::time::Duration::from_secs(request.limits.time_budget_secs.max(1));
    let render = match tokio::time::timeout(budget, run_inner(request, &binary, &mut owner)).await {
        Ok(result) => result,
        Err(_) => Err((
            "time_budget_exceeded".to_owned(),
            format!(
                "the render exceeded the {}-second budget",
                request.limits.time_budget_secs
            ),
        )),
    };
    // From here a browser was owned, so cleanup is an OBSERVATION and no
    // response may bypass its result.
    let cleanup = owner.finish(render.is_ok()).await;
    settle(render, cleanup)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn run_inner(
    request: &BrowseRequest,
    binary: &std::path::Path,
    owner: &mut lifetime::Lifetime,
) -> Result<BrowseResponse, (String, String)> {
    tokio::time::timeout(std::time::Duration::from_secs(20), owner.launch(binary))
        .await
        .map_err(|_| {
            (
                "browser_launch_failed".to_owned(),
                "browser startup exceeded 20 seconds".to_owned(),
            )
        })?
        .map_err(|e| ("browser_launch_failed".to_owned(), e))?;
    let browser = owner
        .browser
        .as_ref()
        .expect("launch established the browser");
    let version = browser
        .version()
        .await
        .map_err(|e| ("browser_version_failed".to_owned(), e.to_string()))?
        .product;
    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|e| ("page_failed".to_owned(), e.to_string()))?;

    // The network log: every request the page makes is recorded (the
    // disclosure — the `.5.1` contract).
    let log = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<NetworkEntry>::new()));
    {
        let log = std::sync::Arc::clone(&log);
        let mut events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent>()
            .await
            .map_err(|e| ("page_failed".to_owned(), e.to_string()))?;
        owner.network = Some(tokio::spawn(async move {
            while let Some(event) = events.next().await {
                log.lock().await.push(NetworkEntry {
                    url: event.request.url.clone(),
                    method: event.request.method.clone(),
                });
            }
        }));
    }

    for step in &request.steps {
        match step {
            BrowseStep::Navigate { url } => {
                page.goto(url)
                    .await
                    .map_err(|e| ("navigation_failed".to_owned(), e.to_string()))?;
            }
            BrowseStep::Click { selector } => {
                let element = page
                    .find_element(selector)
                    .await
                    .map_err(|e| ("click_failed".to_owned(), e.to_string()))?;
                element
                    .click()
                    .await
                    .map_err(|e| ("click_failed".to_owned(), e.to_string()))?;
            }
            BrowseStep::Scroll { y } => {
                page.evaluate(format!("window.scrollBy(0, {y})"))
                    .await
                    .map_err(|e| ("scroll_failed".to_owned(), e.to_string()))?;
            }
            BrowseStep::Type { selector, text } => {
                let element = page
                    .find_element(selector)
                    .await
                    .map_err(|e| ("type_failed".to_owned(), e.to_string()))?;
                element
                    .click()
                    .await
                    .map_err(|e| ("type_failed".to_owned(), e.to_string()))?;
                element
                    .type_str(text)
                    .await
                    .map_err(|e| ("type_failed".to_owned(), e.to_string()))?;
            }
        }
        // A short settle beat after each step so the page's reactions land.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    let body = page
        .find_element("body")
        .await
        .map_err(|e| ("page_failed".to_owned(), e.to_string()))?;
    let text = body
        .inner_text()
        .await
        .map_err(|e| ("text_failed".to_owned(), e.to_string()))?
        .unwrap_or_default();
    let title = page
        .get_title()
        .await
        .map_err(|e| ("title_failed".to_owned(), e.to_string()))?
        .unwrap_or_default();

    if text.len() as u64 > request.limits.max_output_bytes {
        return Err((
            "output_too_large".to_owned(),
            format!(
                "the rendered text ({}) exceeds the {}-byte ceiling",
                text.len(),
                request.limits.max_output_bytes
            ),
        ));
    }

    let chunks = if text.is_empty() {
        Vec::new()
    } else {
        vec![DerivedChunk {
            digest: digest_sha256_hex(text.as_bytes()),
            text,
        }]
    };
    let network_log = log.lock().await.clone();
    let response = BrowseResponse {
        parent_digest: digest_sha256_hex(
            chunks
                .iter()
                .flat_map(|c| c.text.as_bytes())
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        ),
        chunks,
        network_log,
        page_title: title,
        browser_version: version,
        worker_version: WORKER_VERSION.to_owned(),
    };
    drop(page);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered() -> BrowseResponse {
        BrowseResponse {
            parent_digest: digest_sha256_hex(b""),
            chunks: Vec::new(),
            network_log: Vec::new(),
            page_title: String::new(),
            browser_version: "test".to_owned(),
            worker_version: WORKER_VERSION.to_owned(),
        }
    }

    fn refusal() -> Result<BrowseResponse, (String, String)> {
        Err((
            "time_budget_exceeded".to_owned(),
            "the render exceeded the 30-second budget".to_owned(),
        ))
    }

    /// The whole contract, on every host: the render's outcome and the
    /// cleanup's outcome are independent, and neither is inferable from the
    /// other. Only the success/unconfirmed corner may be named for cleanup.
    #[test]
    fn a_response_carries_the_render_and_cleanup_facts_separately() {
        let unconfirmed = || Err("browser stderr completion unconfirmed".to_owned());

        // A clean render with a clean cleanup is the only success.
        assert!(settle(Ok(rendered()), Ok(())).is_ok());

        // A render refusal keeps its OWN kind whether or not cleanup confirmed.
        let fast = settle(refusal(), Ok(())).unwrap_err();
        assert_eq!(fast.kind, "time_budget_exceeded");
        assert!(fast.cleanup_confirmed);
        assert_eq!(fast.cleanup_error, None);
        assert_eq!(fast.message, "the render exceeded the 30-second budget");

        let slow = settle(refusal(), unconfirmed()).unwrap_err();
        assert_eq!(
            slow.kind, "time_budget_exceeded",
            "an unconfirmed cleanup must not overwrite the caller's own fact"
        );
        assert!(!slow.cleanup_confirmed);
        assert_eq!(
            slow.cleanup_error.as_deref(),
            Some(unconfirmed().unwrap_err().as_str())
        );
        // The operator's fact also reaches a consumer that reads only the message.
        assert!(slow
            .message
            .contains("the render exceeded the 30-second budget"));
        assert!(slow
            .message
            .contains("browser stderr completion unconfirmed"));

        // Only a SUCCESSFUL render has no failure of its own to name, so that
        // is the one corner the cleanup fact names — it must never read as
        // complete while a browser may still be running.
        let bypassed = settle(Ok(rendered()), unconfirmed()).unwrap_err();
        assert_eq!(bypassed.kind, "browser_cleanup_unconfirmed");
        assert!(!bypassed.cleanup_confirmed);
        assert_eq!(
            bypassed.cleanup_error.as_deref(),
            Some(unconfirmed().unwrap_err().as_str())
        );
    }

    /// Every kind the render can produce keeps its identity, not just the
    /// budget that exposed this. The list is the population `run_inner` and
    /// the render deadline can reach once a browser is owned.
    #[test]
    fn every_render_refusal_keeps_its_kind_under_an_unconfirmed_cleanup() {
        for kind in [
            "time_budget_exceeded",
            "browser_launch_failed",
            "browser_version_failed",
            "page_failed",
            "navigation_failed",
            "click_failed",
            "scroll_failed",
            "type_failed",
            "text_failed",
            "title_failed",
            "output_too_large",
        ] {
            let settled = settle(
                Err((kind.to_owned(), "detail".to_owned())),
                Err("browser task cancellation unconfirmed".to_owned()),
            )
            .unwrap_err();
            assert_eq!(settled.kind, kind);
            assert!(!settled.cleanup_confirmed);
        }
    }

    /// A refusal raised before any browser exists reports a confirmed cleanup
    /// because nothing was started — and says so with no cleanup error.
    #[test]
    fn a_refusal_before_any_browser_reports_nothing_left_behind() {
        let error = BrowseError::refused("no_steps", "the request carries no steps");
        assert!(error.cleanup_confirmed);
        assert_eq!(error.cleanup_error, None);
        let wire = serde_json::to_value(&error).unwrap();
        assert_eq!(wire["cleanup_confirmed"], true);
        assert!(
            wire.get("cleanup_error").is_none(),
            "an absent cleanup error is omitted, never serialized as null: {wire}"
        );
    }
}
