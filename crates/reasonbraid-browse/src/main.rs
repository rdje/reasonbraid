//! The R3 browser worker (PHASE-4.5.2): the stdio JSON protocol, the
//! bounded interaction (the step budget + the wall-clock ceiling), the
//! network log (every request the page makes is recorded — the disclosure),
//! and the refusal vocabulary — every refusal names its kind. The rendered
//! text is ALWAYS a Derivation (the parent digest + the derived chunks).
//!
//! The registry advertises this pack with `subresource_policy: "deny"` and
//! `redirect_policy: "deny"`, and since `SIGNOFF-REPAIR.7.3.5` the worker
//! ENFORCES both through the CDP `Fetch` domain: a document request for a URL
//! the caller asked to navigate to continues, and every other request — a
//! subresource, or a document the caller never named, which is what a redirect
//! is — fails with `BlockedByClient` and is NAMED in `refused_requests`.

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
    /// The page's OWN bytes — the serialized document the chunks derive from.
    ///
    /// ⛔ It is carried, not just digested (`SIGNOFF-REPAIR.11.24.1.3.1.1`).
    /// §12.6's `EvidenceSnapshot` is *immutable* and addressable by its
    /// *raw-byte digest*; a digest with no bytes behind it is a claim about an
    /// artefact nobody kept.
    pub document: String,
    /// The digest of `document` — the PARENT of every chunk below.
    pub parent_digest: String,
    pub chunks: Vec<DerivedChunk>,
    pub network_log: Vec<NetworkEntry>,
    /// Every request the two advertised deny-policies refused, in order.
    pub refused_requests: Vec<RefusedRequest>,
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

/// One request the page issued and the worker REFUSED, named on the receipt
/// rather than silently absent. ⛔ A denial nobody can see is indistinguishable
/// from a page that never asked.
#[derive(Debug, Clone, Serialize)]
pub struct RefusedRequest {
    pub url: String,
    /// `subresource` or `redirect` — the advertised policy that refused it.
    pub policy: &'static str,
    pub resource_type: String,
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

/// Which advertised deny-policy refuses this request, if either.
///
/// This is the whole of the R3 pack's request policy, as a pure function of
/// what the browser reports and what the caller asked for. `Some("subresource")`
/// and `Some("redirect")` are refusals; `None` continues.
///
/// ⭐ THE TWO ADVERTISED LINES, WITH NOTHING ADDED. The resolver registry
/// publishes this pack with `subresource_policy: "deny"` and
/// `redirect_policy: "deny"` (`reasonbraid-server/src/resolvers.rs::gated_advertises`).
/// A non-document request IS a subresource; a document request for a URL nobody
/// asked to navigate to IS a redirect. Anything beyond those two would enforce
/// something the pack does not advertise, which is the defect
/// `SIGNOFF-REPAIR.7.3.5` repaired in the other direction.
///
/// ⛔ IT IS A FUNCTION SO THAT A COMMIT CAN GUARD IT (`SIGNOFF-REPAIR.7.3.6.3`).
/// Inline in the interception task, the decision could only be exercised by
/// driving a real Chrome, so the enforcement half of the claim was gated at
/// PUSH time while the advertisement half was gated at every commit. Nothing
/// here touches the network, the browser or the filesystem.
///
/// ⚠️ IT DOES NOT READ THE REGISTRY, and that is deliberate. Plumbing the
/// advertised word down to the worker was the rejected alternative: it would
/// make a value an operator can write decide whether this pack isolates
/// anything, and `resolver_capabilities` has no tenant column
/// (`SIGNOFF-REPAIR.11.9.1.1.1`). The pair is GATED instead — a moved
/// advertisement is refused by `scripts/census_advertised_policies.py`, a moved
/// behaviour by the controls below.
fn refusing_policy(
    is_document: bool,
    url: &str,
    requested: &std::collections::HashSet<String>,
) -> Option<&'static str> {
    if !is_document {
        Some("subresource")
    } else if requested.contains(url) {
        None
    } else {
        Some("redirect")
    }
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

    // ⛔ THE POLICY DECISION ITSELF LIVES IN `refusing_policy`, a pure function,
    // and that placement is the repair rather than a tidiness preference
    // (`SIGNOFF-REPAIR.7.3.6.3`). Inline in this async closure, the only way to
    // exercise the decision was to drive a real Chrome — so it was guarded by
    // `browser_roundtrip`, which needs a browser and therefore runs in CI at
    // PUSH time. The advertisement it implements is guarded by a doctrine gate
    // on every COMMIT. Two halves of one claim, gated hundreds of commits
    // apart. As a pure function the decision is covered by `cargo test -p
    // reasonbraid-browse --bins`, which needs nothing.
    //
    // ⛔ THE ADVERTISED DENY-POLICIES, ENFORCED. The resolver registry tells
    // every caller that this pack runs with `subresource_policy: "deny"` and
    // `redirect_policy: "deny"` (`resolvers.rs::gated_advertises`), and until
    // `SIGNOFF-REPAIR.7.3.5` the worker enforced neither: it subscribed to
    // `EventRequestWillBeSent` purely to LOG, so the page's own requests
    // reached whatever they named. A control drove a page whose `<img>` named
    // `0.0.0.0` and the origin recorded the dial.
    //
    // The gate is the CDP `Fetch` domain, which pauses every request BEFORE it
    // leaves the browser: a DOCUMENT request for a URL this worker was asked to
    // navigate to continues, and everything else fails. That is exactly the two
    // advertised lines — a document request for a URL nobody asked for IS the
    // redirect, and every non-document request IS a subresource.
    //
    // ⚠️ The consequence is stated rather than discovered: a page that assembles
    // its text from an external stylesheet or script renders less text here than
    // in a desktop browser. That is what "deny" means, and it is the posture
    // this pack advertises for untrusted content.
    let refusals = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<RefusedRequest>::new()));
    {
        use chromiumoxide::cdp::browser_protocol::fetch::{
            ContinueRequestParams, EnableParams, EventRequestPaused, FailRequestParams,
            RequestPattern,
        };
        use chromiumoxide::cdp::browser_protocol::network::{ErrorReason, ResourceType};

        // Every request the caller explicitly asked for, normalised the way the
        // browser will present it back.
        let requested: std::collections::HashSet<String> = std::iter::once(request.url.clone())
            .chain(request.steps.iter().filter_map(|step| match step {
                BrowseStep::Navigate { url } => Some(url.clone()),
                _ => None,
            }))
            .collect();

        page.execute(
            EnableParams::builder()
                .pattern(RequestPattern::builder().url_pattern("*").build())
                .build(),
        )
        .await
        .map_err(|e| ("page_failed".to_owned(), e.to_string()))?;

        let mut paused = page
            .event_listener::<EventRequestPaused>()
            .await
            .map_err(|e| ("page_failed".to_owned(), e.to_string()))?;
        let refusals_for_task = std::sync::Arc::clone(&refusals);
        let page_for_task = page.clone();
        owner.intercept = Some(tokio::spawn(async move {
            while let Some(event) = paused.next().await {
                let url = event.request.url.clone();
                let is_document = matches!(event.resource_type, ResourceType::Document);
                let policy = refusing_policy(is_document, &url, &requested);
                let outcome = match policy {
                    None => page_for_task
                        .execute(ContinueRequestParams::new(event.request_id.clone()))
                        .await
                        .map(|_| ()),
                    Some(policy) => {
                        refusals_for_task.lock().await.push(RefusedRequest {
                            url,
                            policy,
                            resource_type: format!("{:?}", event.resource_type),
                        });
                        page_for_task
                            .execute(FailRequestParams::new(
                                event.request_id.clone(),
                                ErrorReason::BlockedByClient,
                            ))
                            .await
                            .map(|_| ())
                    }
                };
                // A request the browser has already torn down cannot be
                // answered; the refusal is still recorded, which is the fact
                // the receipt needs.
                let _ = outcome;
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

    // The page's OWN bytes: the serialized document, read BEFORE the response
    // is shaped, because it is the artefact everything else here derives from.
    let document = page
        .content()
        .await
        .map_err(|e| ("document_failed".to_owned(), e.to_string()))?;

    if (document.len() + text.len()) as u64 > request.limits.max_output_bytes {
        return Err((
            "output_too_large".to_owned(),
            format!(
                "the render's output ({} document + {} text) exceeds the {}-byte ceiling",
                document.len(),
                text.len(),
                request.limits.max_output_bytes
            ),
        ));
    }

    let network_log = log.lock().await.clone();
    let refused_requests = refusals.lock().await.clone();
    let response = render_response(
        document,
        text,
        network_log,
        refused_requests,
        title,
        version,
    );
    drop(page);
    Ok(response)
}

/// Shape one render's response from the page's OWN bytes and the text derived
/// from it.
///
/// 🔴 **THE DEFECT THIS REPLACES** (`SIGNOFF-REPAIR.11.24.1.3.1.1`).
/// `parent_digest` used to be `digest(concat of the chunk texts)`. The worker
/// produces exactly one chunk — `body.inner_text()` — so the concatenation was
/// that chunk's own text and **`parent_digest` equalled `chunks[0].digest`
/// byte for byte**. The receipt advertised a parent/derivation pair in which
/// the parent WAS the derivation, under a §12.6 clause written to keep them
/// apart: *a quote, summary, OCR result, model-generated caption, or repository
/// analysis is not the original source.* An empty page was worse still — the
/// parent digested the empty string, so a render that returned no text claimed
/// a parent that was nothing at all.
///
/// ⭐ **IT IS A FUNCTION SO THAT A CONTROL CAN GUARD IT**, which is
/// `SIGNOFF-REPAIR.7.3.6.3`'s move applied a second time in this same file.
/// Inline in the render, the parent/derivation distinction could only be
/// exercised by driving a real Chrome, and the two lines that collapsed it
/// would have needed a browser to catch.
pub fn render_response(
    document: String,
    text: String,
    network_log: Vec<NetworkEntry>,
    refused_requests: Vec<RefusedRequest>,
    page_title: String,
    browser_version: String,
) -> BrowseResponse {
    // ⛔ The parent is the DOCUMENT, whatever the text turned out to be. A page
    // that rendered to no text still has bytes, and they are still its
    // provenance.
    let parent_digest = digest_sha256_hex(document.as_bytes());
    let chunks = if text.is_empty() {
        Vec::new()
    } else {
        vec![DerivedChunk {
            digest: digest_sha256_hex(text.as_bytes()),
            text,
        }]
    };
    BrowseResponse {
        document,
        parent_digest,
        chunks,
        network_log,
        refused_requests,
        page_title,
        browser_version,
        worker_version: WORKER_VERSION.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered() -> BrowseResponse {
        BrowseResponse {
            document: String::new(),
            parent_digest: digest_sha256_hex(b""),
            chunks: Vec::new(),
            network_log: Vec::new(),
            refused_requests: Vec::new(),
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

    /// `SIGNOFF-REPAIR.11.24.1.3.1.1` — the parent is the DOCUMENT, and the
    /// chunk derived from it is something else.
    ///
    /// 🔴 **THE DEFECT, stated as the identity it produced.** `parent_digest`
    /// was `digest(concat of the chunk texts)`, and this worker produces
    /// exactly one chunk, so the concatenation WAS that chunk's text and the
    /// two digests were equal byte for byte. §12.6's edge exists to say *this
    /// derivation is not the original source*; an edge whose parent and child
    /// are the same bytes says nothing.
    ///
    /// ⭐ The arm that matters is the INEQUALITY, not the equality: asserting
    /// only that the parent digests the document would pass against a worker
    /// that also made the chunk out of the document.
    #[test]
    fn the_parent_is_the_document_and_the_chunk_is_not() {
        let document = "<html><body><p>rendered</p></body></html>".to_owned();
        let text = "rendered".to_owned();
        let response = render_response(
            document.clone(),
            text.clone(),
            Vec::new(),
            Vec::new(),
            "t".to_owned(),
            "v".to_owned(),
        );

        assert_eq!(
            response.parent_digest,
            digest_sha256_hex(document.as_bytes()),
            "the parent is the page's own bytes"
        );
        assert_eq!(response.document, document, "and those bytes are carried");
        assert_eq!(response.chunks.len(), 1);
        assert_eq!(
            response.chunks[0].digest,
            digest_sha256_hex(text.as_bytes()),
            "the chunk is the text derived from the page"
        );
        assert_ne!(
            response.parent_digest, response.chunks[0].digest,
            "a derivation whose digest equals its parent's is not a derivation — this \
             is the equality the superseded construction produced on every render"
        );
    }

    /// A page that rendered to NO text still has bytes, and they are still its
    /// provenance. The superseded construction digested the empty string here,
    /// so the receipt named a parent that was nothing at all.
    #[test]
    fn a_page_that_renders_no_text_still_has_a_parent() {
        let document = "<html><body></body></html>".to_owned();
        let response = render_response(
            document.clone(),
            String::new(),
            Vec::new(),
            Vec::new(),
            String::new(),
            "v".to_owned(),
        );
        assert!(response.chunks.is_empty(), "no text, no derivation");
        assert_eq!(
            response.parent_digest,
            digest_sha256_hex(document.as_bytes())
        );
        assert_ne!(
            response.parent_digest,
            digest_sha256_hex(b""),
            "an empty-rendering page is not an empty page"
        );
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

#[cfg(test)]
mod request_policy_tests {
    use super::refusing_policy;
    use std::collections::HashSet;

    fn asked(urls: &[&str]) -> HashSet<String> {
        urls.iter().map(|u| (*u).to_owned()).collect()
    }

    /// `SIGNOFF-REPAIR.7.3.6.3`: the R3 pack's request policy, guarded without
    /// a browser.
    ///
    /// The end-to-end proof stays `browser_roundtrip`'s
    /// `a_subresource_the_pack_advertises_as_denied_is_not_dialed`, which drives
    /// a real Chrome and watches an origin's own counter — that is the control
    /// that showed the defect. What it cannot do is run on every commit: it
    /// needs a browser, so it runs in CI at push. These arms run wherever
    /// `cargo test` does, so the decision is covered between pushes too.
    #[test]
    fn the_request_policy_enforces_exactly_the_two_advertised_lines() {
        let requested = asked(&["https://example.org/page"]);

        // A DOCUMENT request for a URL the caller asked to navigate to is the
        // navigation itself, and continues. Denying it would be a blackout
        // rather than a policy — the almost-fix `.7.3.5` neutralized into.
        assert_eq!(
            refusing_policy(true, "https://example.org/page", &requested),
            None,
            "the requested navigation continues",
        );

        // A DOCUMENT request for a URL nobody asked for IS the redirect.
        assert_eq!(
            refusing_policy(true, "https://elsewhere.test/moved", &requested),
            Some("redirect"),
            "a document request for an unasked URL is the redirect policy",
        );

        // Every NON-document request is a subresource, including one whose URL
        // the caller did ask for — the resource type decides, not the URL. An
        // implementation that checked the URL first would let a page fetch the
        // navigated document as an image and call it asked-for.
        assert_eq!(
            refusing_policy(false, "https://example.org/logo.png", &requested),
            Some("subresource"),
        );
        assert_eq!(
            refusing_policy(false, "https://example.org/page", &requested),
            Some("subresource"),
            "the resource type decides a subresource, never the URL",
        );
    }

    /// ⛔ THE REFUSAL IS A CLASSIFICATION, NOT A PROHIBITION, and this arm is
    /// the one that fails against the almost-fix. `.7.3.5` neutralized its
    /// repair into "deny means deny, refuse everything" and the real browser
    /// answered `navigation_failed` / `net::ERR_BLOCKED_BY_CLIENT`. A policy
    /// that refuses every request passes every other arm here and fails this
    /// one.
    #[test]
    fn a_policy_that_refused_everything_would_fail_this() {
        let requested = asked(&["https://example.org/a", "https://example.org/b"]);
        for url in ["https://example.org/a", "https://example.org/b"] {
            assert_eq!(
                refusing_policy(true, url, &requested),
                None,
                "every URL the caller asked to navigate to still loads",
            );
        }
    }

    /// Multi-step navigation: `BrowseStep::Navigate` adds to the asked-for set,
    /// so a later step's document is not mistaken for a redirect.
    #[test]
    fn a_later_navigation_step_is_not_a_redirect() {
        let requested = asked(&["https://example.org/one", "https://example.org/two"]);
        assert_eq!(
            refusing_policy(true, "https://example.org/two", &requested),
            None,
        );
        assert_eq!(
            refusing_policy(true, "https://example.org/three", &requested),
            Some("redirect"),
            "…and a URL no step named is still the redirect policy",
        );
    }

    /// ⚠️ The comparison is EXACT, stated rather than left to be discovered: a
    /// trailing slash, a fragment or a different case makes a URL a different
    /// URL, so a server that normalises the navigation target yields a document
    /// request the policy calls a redirect. That is the fail-closed direction —
    /// recorded here because the arm documents the edge rather than hiding it.
    #[test]
    fn the_asked_for_comparison_is_exact_and_fails_closed() {
        let requested = asked(&["https://example.org/page"]);
        assert_eq!(
            refusing_policy(true, "https://example.org/page/", &requested),
            Some("redirect"),
            "a trailing slash is a different URL, and the policy refuses rather than guesses",
        );
    }
}
