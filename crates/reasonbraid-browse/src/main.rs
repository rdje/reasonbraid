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

#[derive(Debug, Serialize)]
pub struct BrowseError {
    pub kind: String,
    pub message: String,
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
        Err((kind, message)) => respond_error(&kind, &message),
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
    respond(&serde_json::json!({ "error": { "kind": kind, "message": message } }));
}

/// The bounded interaction: the step budget first, then the wall-clock
/// ceiling over the whole render (the trip refuses, never hangs).
async fn run(request: &BrowseRequest) -> Result<BrowseResponse, (String, String)> {
    if request.steps.len() > request.limits.max_steps {
        return Err((
            "step_budget_exceeded".to_owned(),
            format!(
                "{} steps exceeds the {}-step budget",
                request.steps.len(),
                request.limits.max_steps
            ),
        ));
    }
    if request.steps.is_empty() {
        return Err((
            "no_steps".to_owned(),
            "the request carries no steps".to_owned(),
        ));
    }
    run_browser(request).await
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn run_browser(_: &BrowseRequest) -> Result<BrowseResponse, (String, String)> {
    Err((
        "browser_platform_unsupported".to_owned(),
        "owned browser processes currently require Linux or macOS".to_owned(),
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn run_browser(request: &BrowseRequest) -> Result<BrowseResponse, (String, String)> {
    let binary = browser_binary()
        .ok_or_else(|| {
            (
                "browser_missing".to_owned(),
                "no browser binary found — set R3_BROWSER_BIN to the pinned chromium".to_owned(),
            )
        })?
        .canonicalize()
        .map_err(|e| ("browser_launch_failed".to_owned(), e.to_string()))?;
    let mut owner = lifetime::Lifetime::new()
        .map_err(|e| ("browser_storage_failed".to_owned(), e.to_string()))?;
    let budget = std::time::Duration::from_secs(request.limits.time_budget_secs.max(1));
    let result = match tokio::time::timeout(budget, run_inner(request, &binary, &mut owner)).await {
        Ok(result) => result,
        Err(_) => Err((
            "time_budget_exceeded".to_owned(),
            format!(
                "the render exceeded the {}-second budget",
                request.limits.time_budget_secs
            ),
        )),
    };
    if let Err(cleanup) = owner.finish(result.is_ok()).await {
        let detail = match &result {
            Ok(_) => cleanup,
            Err((kind, message)) => format!("{kind}: {message}; {cleanup}"),
        };
        return Err(("browser_cleanup_unconfirmed".to_owned(), detail));
    }
    result
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
