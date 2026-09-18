//! The R1 public-Git acquisition (ROADMAP.md §12.5; the `PHASE-4.3.2` leaf).
//!
//! Executes the `2026-09-07_r1-git-acquisition-contract` decision: the https
//! transport dials ONLY through a classified reqwest client (the `.2.2` belt
//! re-classifies every dial, redirect hops included, and the pre-flight names
//! the refusing class BEFORE the clone starts), the ref selector rides the
//! URL fragment, the budgets trip with their names, the default-deny refusal
//! list is enforced (submodules/gitlinks and LFS pointers are REFUSED, never
//! skipped — and the LFS gate reads the pointer's own grammar at offset 0, not
//! a byte run anywhere in the head), NO checkout execution happens (the target
//! is a bare repository — the working tree is never materialized, hooks never
//! run), the repository is opened with `gix::open::Options::isolated()` so the
//! operator's system/application/user git configuration and the `GIT_*`/`SSH_*`
//! environment reach NOTHING while a caller-supplied URL is fetched, and the
//! resolved immutable commit is recorded.
//!
//! gix is the engine (the `.3.1` census: pure Rust, no C). The fetch runs in
//! a blocking worker (gix's blocking client); the async pre-flight runs on
//! the caller's runtime.

use std::fmt;
use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gix_transport::client::blocking_io::http::{self, Http, PostBodyDataKind};
use reqwest::blocking as blocking_reqwest;
use url::Url;

use crate::fetcher::{ClassifiedDns, DestinationResolver, SystemResolver};
use crate::project_storage::OwnedDirectory;
use crate::ssrf::SsrfVerdict;

/// The R1 ceilings (the `.3.1` budget vocabulary). The defaults are the
/// dev-profile's text-repository evidence path.
#[derive(Debug, Clone)]
pub struct GitLimits {
    /// The path-depth ceiling (trips DURING the tree walk, named).
    pub max_depth: u32,
    /// The file-count ceiling.
    pub max_files: u64,
    /// The object-count ceiling.
    pub max_objects: u64,
    /// The object-database byte ceiling.
    pub max_bytes: u64,
    /// The whole-acquisition time ceiling.
    pub max_time: Duration,
    /// The shallow fetch depth (the tip + this many ancestors). `None` = full.
    pub shallow_depth: Option<u32>,
}

impl Default for GitLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_files: 10_000,
            max_objects: 100_000,
            max_bytes: 256 * 1024 * 1024,
            max_time: Duration::from_secs(120),
            shallow_depth: Some(1),
        }
    }
}

/// Every failure is a named refusal — never a fabricated success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    UrlTooLong(usize),
    UrlHasControlCharacters,
    UrlUnparseable,
    SchemeNotAllowed(String),
    UserinfoForbidden,
    AmbiguousNumericHost(String),
    PortNotAllowed(u16),
    NoHost,
    RefSelectorInvalid(String),
    DestinationRefused {
        host: String,
        reason: String,
    },
    DnsLookupFailed,
    HttpStatus(u16),
    DepthCeilingExceeded {
        limit: u32,
        actual: u32,
    },
    BudgetExceeded {
        what: &'static str,
        limit: u64,
        actual: u64,
    },
    Refused {
        what: &'static str,
        detail: String,
    },
    NoHeadRef,
    ResolvedCommitMissing,
    TimedOut,
    TransferFailed(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UrlTooLong(max) => write!(f, "the URL exceeds the {max}-byte cap"),
            Self::UrlHasControlCharacters => {
                write!(f, "the URL carries control characters or backslashes")
            }
            Self::UrlUnparseable => write!(f, "the URL does not parse"),
            Self::SchemeNotAllowed(scheme) => {
                write!(
                    f,
                    "the scheme `{scheme}` is not allowed by the R1 policy (https-only)"
                )
            }
            Self::UserinfoForbidden => write!(f, "the URL carries userinfo (refused)"),
            Self::AmbiguousNumericHost(host) => write!(
                f,
                "the host `{host}` is an alternative (numeric) literal spelling (refused)"
            ),
            Self::PortNotAllowed(port) => {
                write!(f, "the port {port} is not allowed by the R1 policy")
            }
            Self::NoHost => write!(f, "the URL has no host"),
            Self::RefSelectorInvalid(selector) => {
                write!(
                    f,
                    "the ref selector `{selector}` is not a branch, tag, or full commit"
                )
            }
            Self::DestinationRefused { host, reason } => {
                write!(f, "the destination {host} was refused: {reason}")
            }
            Self::DnsLookupFailed => write!(f, "the destination name did not resolve"),
            Self::HttpStatus(status) => write!(f, "the HTTP response status {status} failed"),
            Self::DepthCeilingExceeded { limit, actual } => {
                write!(f, "the tree depth {actual} exceeds the {limit} ceiling")
            }
            Self::BudgetExceeded {
                what,
                limit,
                actual,
            } => {
                write!(f, "the {what} budget exceeded: {actual} > {limit}")
            }
            Self::Refused { what, detail } => {
                write!(
                    f,
                    "the repository's {what} is refused by the R1 policy: {detail}"
                )
            }
            Self::NoHeadRef => write!(f, "the remote advertised no HEAD"),
            Self::ResolvedCommitMissing => {
                write!(f, "the resolved commit is not present after the fetch")
            }
            Self::TimedOut => write!(f, "the acquisition exceeded the time ceiling"),
            Self::TransferFailed(detail) => write!(f, "the transfer failed: {detail}"),
        }
    }
}

impl std::error::Error for GitError {}

/// The measured acquisition — the `.3.3` receipt's input shape (minus the
/// receipt fields the receipt adds). `odb_path` holds the fetched objects for
/// the `.6` snapshot lane.
///
/// The working directory is OWNED here (`SIGNOFF-REPAIR.7.2.1`) rather than
/// left to the caller: the superseded contract said "the caller owns its
/// cleanup" and no production caller ever did, so every successful
/// acquisition leaked a bare repository into an ambient temporary directory.
/// The owner is shared rather than cloned, so a cloned acquisition keeps the
/// same directory alive and the last one out removes it exactly once.
#[derive(Debug, Clone)]
pub struct GitAcquisition {
    /// Kept so the fetched objects outlive `acquire`, and removed when the
    /// last holder drops. Never read directly — `odb_path` addresses it.
    _workspace: Arc<OwnedDirectory>,
    pub resolved_commit: String,
    pub object_count: u64,
    pub file_count: u64,
    pub max_path_depth: u32,
    pub odb_bytes: u64,
    pub odb_path: PathBuf,
    /// The included file paths (the manifest's `included` side; the
    /// excluded side is empty — the refusal list refuses, never excludes).
    pub paths: Vec<String>,
}

/// The R1 receipt (PHASE-4.3.3): the `.3.3` pack-wiring shape — the
/// resolved immutable commit, the requested URL/ref, the manifest, and the
/// `.2.3` receipt's digest/chain fields (the ADR-011 digest is over the
/// acquired odb bytes; the commit sha is the git identity).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GitReceipt {
    pub resolved_commit: String,
    pub requested_url: String,
    pub requested_ref: String,
    pub digest: String,
    pub chain: Vec<String>,
    pub object_count: u64,
    pub file_count: u64,
    pub max_path_depth: u32,
    pub odb_bytes: u64,
    pub manifest: GitManifest,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GitManifest {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
}

impl GitReceipt {
    pub fn from_acquisition(
        requested_url: &str,
        requested_ref: &str,
        acquisition: &GitAcquisition,
        acquired_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            resolved_commit: acquisition.resolved_commit.clone(),
            requested_url: requested_url.to_owned(),
            requested_ref: requested_ref.to_owned(),
            digest: git_digest(&acquisition.odb_path),
            chain: vec![requested_url.to_owned()],
            object_count: acquisition.object_count,
            file_count: acquisition.file_count,
            max_path_depth: acquisition.max_path_depth,
            odb_bytes: acquisition.odb_bytes,
            manifest: GitManifest {
                included: acquisition.paths.clone(),
                excluded: Vec::new(),
            },
            acquired_at,
        }
    }
}

/// The ADR-011 digest over the ACQUIRED odb bytes: every object file
/// (loose + packed), sorted by path, hashed in order — deterministic for
/// the same acquisition.
pub fn git_digest(odb_path: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    collect_files(odb_path, &mut files);
    files.sort();
    let mut hasher = Sha256::new();
    for file in files {
        if let Ok(bytes) = std::fs::read(&file) {
            hasher.update(&bytes);
        }
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn collect_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, out);
            } else {
                out.push(path);
            }
        }
    }
}

/// The R1 acquirer: the classified transport + the ceilings.
pub struct GitFetcher {
    limits: GitLimits,
    resolver: Arc<dyn DestinationResolver>,
    policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
}

impl GitFetcher {
    /// The production acquirer: the system resolver + the `.2.1` public-only
    /// policy + the default ceilings.
    pub fn new(limits: GitLimits) -> Self {
        Self {
            limits,
            resolver: Arc::new(SystemResolver),
            policy: Arc::new(|ip| crate::ssrf::evaluate(*ip)),
        }
    }

    /// The test seam (the `.2.2` seams carry over per the contract): the
    /// injected resolver + policy keep the wire tests offline.
    pub fn with_policy(
        limits: GitLimits,
        resolver: Arc<dyn DestinationResolver>,
        policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
    ) -> Self {
        Self {
            limits,
            resolver,
            policy,
        }
    }

    /// Acquire a public repository: harden + classify FIRST (the refusal
    /// names the class before any socket opens), then the classified clone.
    pub async fn acquire(&self, raw_url: &str) -> Result<GitAcquisition, GitError> {
        let parsed = harden_git_url(raw_url)?;
        self.classify(&parsed).await?;
        let limits = self.limits.clone();
        let resolver = Arc::clone(&self.resolver);
        let policy = Arc::clone(&self.policy);
        let url = parsed.to_string();
        let factory = {
            let resolver = Arc::clone(&resolver);
            let policy = Arc::clone(&policy);
            Box::new(move |url, version| classified_http_factory(url, version, &resolver, &policy))
                as Box<TransportFactory>
        };
        tokio::task::spawn_blocking(move || acquire_blocking(&url, &limits, factory))
            .await
            .map_err(|_| GitError::TransferFailed("the acquisition worker failed".into()))?
    }

    /// The pre-flight: resolve + classify BEFORE the clone starts (the same
    /// all-addresses-must-pass rule the `.2.2` fetcher enforces).
    async fn classify(&self, url: &Url) -> Result<(), GitError> {
        let host = url.host_str().ok_or(GitError::NoHost)?;
        let port = url.port_or_known_default().unwrap_or(443);
        if let Ok(ip) = host.parse::<IpAddr>() {
            return allow_ip(&self.policy, ip);
        }
        let mut addrs = self
            .resolver
            .resolve(host, port)
            .await
            .map_err(|_| GitError::DnsLookupFailed)?;
        if addrs.is_empty() {
            return Err(GitError::DnsLookupFailed);
        }
        addrs.truncate(16);
        for addr in addrs {
            allow_ip(&self.policy, addr.ip())?;
        }
        Ok(())
    }
}

fn allow_ip(
    policy: &Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
    ip: IpAddr,
) -> Result<(), GitError> {
    match policy(&ip) {
        SsrfVerdict::Allowed => Ok(()),
        SsrfVerdict::Refused { reason } => Err(GitError::DestinationRefused {
            host: ip.to_string(),
            reason,
        }),
    }
}

/// The hardened parse (the `.3.1` contract's URL grammar): https-only, no
/// userinfo, the standard port, the alternative-literal refusal, and the
/// fragment as the ref selector (a branch, a tag, a full ref name, or a
/// 40-hex commit — anything else is the typed refusal).
fn harden_git_url(raw: &str) -> Result<Url, GitError> {
    if raw.len() > 2048 {
        return Err(GitError::UrlTooLong(2048));
    }
    if raw
        .chars()
        .any(|c| c.is_control() || c == '\\' || c.is_whitespace())
    {
        return Err(GitError::UrlHasControlCharacters);
    }
    let url = Url::parse(raw).map_err(|_| GitError::UrlUnparseable)?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(GitError::UserinfoForbidden);
    }
    if url.scheme() != "https" {
        return Err(GitError::SchemeNotAllowed(url.scheme().to_owned()));
    }
    match url.host() {
        None => return Err(GitError::NoHost),
        Some(url::Host::Domain(domain)) if numeric_ambiguous(domain) => {
            return Err(GitError::AmbiguousNumericHost(domain.to_owned()));
        }
        Some(_) => {}
    }
    if let Some(port) = url.port() {
        if port != 443 {
            return Err(GitError::PortNotAllowed(port));
        }
    }
    if let Some(selector) = url.fragment() {
        if !valid_ref_selector(selector) {
            return Err(GitError::RefSelectorInvalid(selector.to_owned()));
        }
    }
    Ok(url)
}

fn numeric_ambiguous(domain: &str) -> bool {
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.is_empty() {
        return false;
    }
    let all_numeric = labels
        .iter()
        .all(|label| !label.is_empty() && label.chars().all(|c| c.is_ascii_digit()));
    let single_hex = labels.len() == 1
        && labels[0].len() > 2
        && labels[0].starts_with("0x")
        && labels[0][2..].chars().all(|c| c.is_ascii_hexdigit());
    all_numeric || single_hex
}

/// The ref-selector grammar: a full ref name (`refs/…`), a 40-hex commit, or
/// a branch/tag short name (the safe charset: `[0-9A-Za-z._/-]`).
fn valid_ref_selector(selector: &str) -> bool {
    if selector.is_empty() {
        return false;
    }
    if selector.starts_with("refs/") {
        return selector
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c));
    }
    if selector.len() == 40 && selector.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    !selector.contains("..")
        && !selector.starts_with('-')
        && selector
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c))
}

/// The refspec the fetch uses: the fragment's selector, or HEAD.
fn fetch_refspec(url: &Url) -> String {
    url.fragment()
        .map(str::to_owned)
        .unwrap_or_else(|| "HEAD".to_owned())
}

// ── the classified transport ─────────────────────────────────────────────

/// The R1 dial: a blocking reqwest client on which **every** dial, redirect
/// hops included, passes the destination policy — and it takes TWO mechanisms
/// to say that truthfully, which is why they are named here.
///
/// - A **hostname** destination is resolved through `ClassifiedDns`, so the
///   policy sees every address before a socket opens.
/// - An **IP-literal** destination reaches no resolver at all: hyper-util
///   0.1.20 says so in its own source — *"If the host is already an IP addr
///   (v4 or v6), skip resolving the dns and start connecting right away."*
///   The initial URL is covered by `GitFetcher::classify`'s pre-flight, which
///   parses an IP literal explicitly; a REDIRECT hop to one was covered by
///   nothing until `SIGNOFF-REPAIR.7.2.2` put the same policy in the redirect
///   decision itself.
///
/// ⛔ The hop cap moved with it. `Policy::limited(5)` enforced the cap and
/// nothing else; a custom policy owns both, so the cap is re-stated here
/// rather than inherited.
///
/// No proxy environment (§12.4: the proxy stays in the threat model).
struct ClassifiedGitHttp {
    client: blocking_reqwest::Client,
    /// The redirect policy can only answer *follow*, *stop* or *error* — it
    /// cannot return a typed refusal. The reason travels out of band here, so
    /// a refused hop surfaces as `DestinationRefused` naming the address and
    /// the class instead of as a generic transport failure.
    refusal: RefusalSlot,
}

/// Out-of-band channel from the redirect policy to the caller of `send`.
type RefusalSlot = Arc<std::sync::Mutex<Option<GitError>>>;

/// The R1 redirect cap, previously the argument to `Policy::limited`.
const MAX_REDIRECT_HOPS: usize = 5;

impl ClassifiedGitHttp {
    fn new(
        resolver: Arc<dyn DestinationResolver>,
        policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
    ) -> Result<Self, GitError> {
        let refusal: RefusalSlot = Arc::new(std::sync::Mutex::new(None));
        let hop_policy = {
            let refusal = Arc::clone(&refusal);
            let policy = Arc::clone(&policy);
            move |attempt: reqwest::redirect::Attempt| {
                if attempt.previous().len() > MAX_REDIRECT_HOPS {
                    return attempt.error(format!("more than {MAX_REDIRECT_HOPS} redirect hops"));
                }
                // Only an IP literal is classified here. A hostname hop is
                // already covered by `ClassifiedDns`, and re-resolving it in a
                // SYNCHRONOUS policy would either block the connector or
                // duplicate the belt's answer.
                let literal = attempt
                    .url()
                    .host_str()
                    .and_then(|host| host.parse::<IpAddr>().ok());
                if let Some(ip) = literal {
                    if let SsrfVerdict::Refused { reason } = policy(&ip) {
                        let refused = GitError::DestinationRefused {
                            host: ip.to_string(),
                            reason,
                        };
                        let detail = refused.to_string();
                        if let Ok(mut slot) = refusal.lock() {
                            *slot = Some(refused);
                        }
                        return attempt.error(detail);
                    }
                }
                attempt.follow()
            }
        };
        let client = blocking_reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::custom(hop_policy))
            .no_proxy()
            .dns_resolver(Arc::new(ClassifiedDns::new(resolver, policy)))
            .user_agent("reasonbraid-r1-git/0.1")
            .build()
            .map_err(|_| GitError::TransferFailed("the R1 HTTP client failed to build".into()))?;
        Ok(Self { client, refusal })
    }

    fn send(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
        body: Option<Vec<u8>>,
    ) -> Result<(Vec<u8>, blocking_reqwest::Response), GitError> {
        let mut builder = self.client.request(method, url);
        for header in headers {
            let header = header.as_ref();
            if let Some((name, value)) = header.split_once(':') {
                builder = builder.header(name.trim(), value.trim());
            }
        }
        if let Some(body) = body {
            builder = builder.body(body);
        }
        // Clear first: the slot belongs to the client, which outlives one
        // request, and a stale refusal must never be attributed to a later one.
        if let Ok(mut slot) = self.refusal.lock() {
            *slot = None;
        }
        let response = match builder.send() {
            Ok(response) => response,
            Err(e) => {
                let refused = self.refusal.lock().ok().and_then(|mut slot| slot.take());
                return Err(refused.unwrap_or_else(|| {
                    GitError::TransferFailed(format!("the request failed: {e}"))
                }));
            }
        };
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(GitError::HttpStatus(status));
        }
        let mut header_lines = Vec::new();
        for (name, value) in response.headers() {
            header_lines.extend_from_slice(name.as_str().as_bytes());
            header_lines.extend_from_slice(b": ");
            header_lines.extend_from_slice(value.as_bytes());
            header_lines.push(b'\n');
        }
        Ok((header_lines, response))
    }
}

/// The post body the R1 acquisition writes: bounded in memory, sent on drop
/// (the git upload-pack flow streams the want/have lines then reads). The
/// UNBOUNDED kind is refused — R1 never pushes.
struct BoundedPostBody {
    client: Arc<blocking_reqwest::Client>,
    url: String,
    headers: Vec<String>,
    buf: Vec<u8>,
    slot: Arc<std::sync::Mutex<Option<SlotData>>>,
    sent: bool,
}

struct SlotData {
    headers: Vec<u8>,
    response: blocking_reqwest::Response,
}

impl io::Write for BoundedPostBody {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for BoundedPostBody {
    fn drop(&mut self) {
        if self.sent {
            return;
        }
        self.sent = true;
        let mut builder = self.client.post(&self.url);
        for header in &self.headers {
            if let Some((name, value)) = header.split_once(':') {
                builder = builder.header(name.trim(), value.trim());
            }
        }
        let body = std::mem::take(&mut self.buf);
        let result = builder.body(body).send();
        let Ok(mut slot) = self.slot.lock() else {
            return;
        };
        if let Ok(response) = result {
            let mut header_lines = Vec::new();
            for (name, value) in response.headers() {
                header_lines.extend_from_slice(name.as_str().as_bytes());
                header_lines.extend_from_slice(b": ");
                header_lines.extend_from_slice(value.as_bytes());
                header_lines.push(b'\n');
            }
            *slot = Some(SlotData {
                headers: header_lines,
                response,
            });
        }
    }
}

/// A lazy reader over the POST response slot (the headers/body exist only
/// after the post body is dropped — the git protocol's own contract).
struct SlotReader {
    slot: Arc<std::sync::Mutex<Option<SlotData>>>,
    side: SlotSide,
    pos: usize,
}

enum SlotSide {
    Headers,
    Body,
}

impl io::Read for SlotReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut slot = self
            .slot
            .lock()
            .map_err(|_| io::Error::other("the post slot poisoned"))?;
        let data = slot
            .as_mut()
            .ok_or_else(|| io::Error::other("the post body was never sent"))?;
        match self.side {
            SlotSide::Headers => {
                if self.pos >= data.headers.len() {
                    return Ok(0);
                }
                let n = (data.headers.len() - self.pos).min(out.len());
                out[..n].copy_from_slice(&data.headers[self.pos..self.pos + n]);
                self.pos += n;
                Ok(n)
            }
            SlotSide::Body => data.response.read(out),
        }
    }
}

impl Http for ClassifiedGitHttp {
    type Headers = io::BufReader<SlotReader>;
    type ResponseBody = io::BufReader<SlotReader>;
    type PostBody = BoundedPostBody;

    fn get(
        &mut self,
        url: &str,
        _base_url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<http::GetResponse<Self::Headers, Self::ResponseBody>, http::Error> {
        let (header_lines, response) = self
            .send(reqwest::Method::GET, url, headers, None)
            .map_err(|e| http::Error::Detail {
                description: e.to_string(),
            })?;
        let slot = Arc::new(std::sync::Mutex::new(Some(SlotData {
            headers: header_lines,
            response,
        })));
        Ok(http::GetResponse {
            headers: io::BufReader::new(SlotReader {
                slot: Arc::clone(&slot),
                side: SlotSide::Headers,
                pos: 0,
            }),
            body: io::BufReader::new(SlotReader {
                slot,
                side: SlotSide::Body,
                pos: 0,
            }),
        })
    }

    fn post(
        &mut self,
        url: &str,
        _base_url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
        body_kind: PostBodyDataKind,
    ) -> Result<http::PostResponse<Self::Headers, Self::ResponseBody, Self::PostBody>, http::Error>
    {
        match body_kind {
            PostBodyDataKind::BoundedAndFitsIntoMemory => {}
            PostBodyDataKind::Unbounded => {
                return Err(http::Error::Detail {
                    description: "the R1 acquisition never streams an upload body".into(),
                });
            }
        }
        let slot = Arc::new(std::sync::Mutex::new(None));
        let post_body = BoundedPostBody {
            client: Arc::new(self.client.clone()),
            url: url.to_owned(),
            headers: headers.into_iter().map(|h| h.as_ref().to_owned()).collect(),
            buf: Vec::new(),
            slot: Arc::clone(&slot),
            sent: false,
        };
        Ok(http::PostResponse {
            post_body,
            headers: io::BufReader::new(SlotReader {
                slot: Arc::clone(&slot),
                side: SlotSide::Headers,
                pos: 0,
            }),
            body: io::BufReader::new(SlotReader {
                slot,
                side: SlotSide::Body,
                pos: 0,
            }),
        })
    }

    fn configure(
        &mut self,
        _config: &dyn std::any::Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(())
    }
}

// ── the acquisition ──────────────────────────────────────────────────────

/// The transport seam: the production factory builds the classified http
/// transport; the tests inject the file transport (the offline wire path).
type TransportFactory = dyn FnOnce(
        gix::url::Url,
        gix::protocol::transport::Protocol,
    ) -> Result<Box<dyn gix_transport::client::blocking_io::Transport + Send>, GitError>
    + Send;

fn classified_http_factory(
    url: gix::url::Url,
    version: gix::protocol::transport::Protocol,
    resolver: &Arc<dyn DestinationResolver>,
    policy: &Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
) -> Result<Box<dyn gix_transport::client::blocking_io::Transport + Send>, GitError> {
    let transport = gix_transport::client::blocking_io::http::Transport::new_http(
        ClassifiedGitHttp::new(Arc::clone(resolver), Arc::clone(policy))?,
        url,
        version,
        false,
    );
    Ok(Box::new(transport))
}

fn acquire_blocking(
    url: &str,
    limits: &GitLimits,
    transport_factory: Box<TransportFactory>,
) -> Result<GitAcquisition, GitError> {
    let parsed = Url::parse(url).map_err(|_| GitError::UrlUnparseable)?;
    // One exclusively created directory on the repository's own volume. The
    // superseded name — an ambient temporary directory joined with the process
    // id and a nanosecond field, then `create_dir_all` — was measured at 501
    // distinct values in 2000 calls with every collision between adjacent
    // calls, and `create_dir_all` ADOPTS an occupied path. Two concurrent
    // acquisitions share the process id by construction, so they could clone
    // into one directory and either one's cleanup would delete the other's
    // objects.
    let workspace = Arc::new(
        OwnedDirectory::create("git", "acquire")
            .map_err(|e| GitError::TransferFailed(e.to_string()))?,
    );
    // On error the workspace is removed by its own `Drop`, which first proves
    // the path still names the directory this acquisition created. On success
    // the acquisition keeps it alive for the receipt's digest.
    acquire_into(workspace, &parsed, limits, transport_factory)
}

/// Create the acquisition's bare repository.
///
/// ⚠️ Deliberately NOT `gix::init_bare`, which is
/// `ThreadSafeRepository::init(…, open::Options::default_for_level(Trust::Full))`
/// — `Permissions::all()`. That opens a repository fetched from a
/// caller-supplied URL while honouring the operator's system config
/// (`$(prefix)/etc/gitconfig`), application config (`$XDG_CONFIG_HOME/git/config`,
/// else `$HOME/.config/git/config`), user config (`~/.gitconfig`),
/// environment-sourced config (`GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_n`), `include`
/// and `includeIf` directives that reach outside the repository, the system and
/// application `gitattributes`, and every `GIT_*`/`SSH_*` environment category.
///
/// `open::Options::isolated()` is the remedy gix names for exactly this, in its
/// own words: "prevent accessing anything else than the repository
/// configuration file, prohibiting accessing the environment or spreading
/// beyond the git repository location". The repository's OWN config is still
/// loaded — gix always loads it, and it is the one this process just created.
fn init_acquisition_repository(dir: &std::path::Path) -> Result<gix::Repository, GitError> {
    gix::ThreadSafeRepository::init_opts(
        dir,
        gix::create::Kind::Bare,
        gix::create::Options::default(),
        gix::open::Options::isolated(),
    )
    .map(Into::into)
    .map_err(|e| GitError::TransferFailed(format!("the target repository failed: {e}")))
}

/// The whole-acquisition time ceiling, made real.
///
/// ⛔ `GitLimits::max_time` was declared, defaulted to 120 s, and read by
/// NOTHING (`SIGNOFF-REPAIR.7.2.7`). `GitError::TimedOut` was declared, carried
/// a Display message, and was mapped to the `timed_out` wire reason in
/// `api.rs` — and was constructed by nothing, so the whole refusal path was
/// wired end to end and unreachable. Five of `GitLimits`' six fields were
/// enforced; this was the sixth.
///
/// ⭐ THE HOOK ALREADY EXISTED. `gix`'s `receive` takes a `&AtomicBool` it
/// polls during the pack transfer, and this module handed it
/// `AtomicBool::default()` — a flag nobody could ever set. Giving it a flag a
/// watchdog raises bounds the TRANSFER itself, not merely the caller's wait,
/// which is the difference between this and a `tokio::time::timeout` around
/// the `spawn_blocking` handle: that would return on time and leave the
/// blocking worker transferring.
///
/// ⚠️ DECLARED LIMIT: `gix` polls the flag during the pack receive and write.
/// A remote that stalls in the connection or the ref advertisement is bounded
/// by the checkpoint AFTER that phase, not inside it, so the effective ceiling
/// is `max_time` plus however long one un-polled phase blocks.
struct Deadline {
    at: std::time::Instant,
    interrupt: Arc<std::sync::atomic::AtomicBool>,
    finished: Arc<std::sync::atomic::AtomicBool>,
}

impl Deadline {
    fn start(limit: Duration) -> Self {
        use std::sync::atomic::{AtomicBool, Ordering};
        let at = std::time::Instant::now() + limit;
        let interrupt = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let (watch, done) = (Arc::clone(&interrupt), Arc::clone(&finished));
        // ⭐ The watchdog exits as soon as the acquisition finishes, so a 120 s
        // ceiling does not cost a 120 s thread on every successful clone.
        std::thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                if std::time::Instant::now() >= at {
                    watch.store(true, Ordering::Relaxed);
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        Self {
            at,
            interrupt,
            finished,
        }
    }

    fn interrupt(&self) -> &std::sync::atomic::AtomicBool {
        &self.interrupt
    }

    fn expired(&self) -> bool {
        std::time::Instant::now() >= self.at
            || self
                .interrupt
                .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// A checkpoint between phases. The ceiling is on the WHOLE acquisition, so
    /// it is checked where one phase ends rather than only after the transfer.
    fn check(&self) -> Result<(), GitError> {
        if self.expired() {
            return Err(GitError::TimedOut);
        }
        Ok(())
    }
}

impl Drop for Deadline {
    fn drop(&mut self) {
        self.finished
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn acquire_into(
    workspace: Arc<OwnedDirectory>,
    url: &Url,
    limits: &GitLimits,
    transport_factory: Box<TransportFactory>,
) -> Result<GitAcquisition, GitError> {
    let deadline = Deadline::start(limits.max_time);
    let target_dir = workspace.path();
    let repo = init_acquisition_repository(target_dir)?;
    deadline.check()?;
    let remote = repo
        .remote_at(url.as_str())
        .map_err(|e| GitError::TransferFailed(format!("the remote failed: {e}")))?;
    let (gix_url, version) = remote
        .sanitized_url_and_version(gix::remote::Direction::Fetch)
        .map_err(|e| GitError::TransferFailed(format!("the remote url failed: {e}")))?;
    let transport = transport_factory(gix_url, version)?;
    let connection = remote.to_connection_with_transport(transport);

    let refspec = fetch_refspec(url);
    let refspec = gix::refspec::parse(
        gix::bstr::BStr::new(refspec.as_str()),
        gix::refspec::parse::Operation::Fetch,
    )
    .map_err(|e| GitError::RefSelectorInvalid(e.to_string()))?
    .into();
    let prepare = connection
        .prepare_fetch(
            &mut gix::progress::Discard,
            gix::remote::ref_map::Options {
                extra_refspecs: vec![refspec],
                ..Default::default()
            },
        )
        .map_err(|e| GitError::TransferFailed(format!("the fetch preparation failed: {e}")))?;
    let fetch_outcome = prepare
        .with_write_packed_refs_only(true)
        .with_shallow(match limits.shallow_depth {
            Some(depth) => gix::remote::fetch::Shallow::DepthAtRemote(
                std::num::NonZeroU32::new(depth.max(1)).expect("a positive depth"),
            ),
            None => gix::remote::fetch::Shallow::NoChange,
        })
        .receive(&mut gix::progress::Discard, deadline.interrupt())
        // ⛔ The deadline is consulted BEFORE the error is classified: an
        // interrupted transfer surfaces as an ordinary `gix` error, and
        // reporting it as `TransferFailed` would hide the ceiling that caused
        // it behind a message about the remote.
        .map_err(|e| {
            if deadline.expired() {
                GitError::TimedOut
            } else {
                GitError::TransferFailed(format!("the fetch failed: {e}"))
            }
        })?;
    deadline.check()?;

    // The resolved immutable commit: the requested ref, resolved through the
    // advertised refs (the HEAD entry, the exact ref name, or the requested
    // sha verified present).
    let requested = url.fragment().unwrap_or("HEAD");
    let head_id = fetch_outcome
        .ref_map
        .remote_refs
        .iter()
        .find_map(|r| {
            let (name, object) = match r {
                gix::protocol::handshake::Ref::Symbolic {
                    full_ref_name,
                    object,
                    ..
                }
                | gix::protocol::handshake::Ref::Direct {
                    full_ref_name,
                    object,
                } => (full_ref_name, *object),
                _ => return None,
            };
            if requested == "HEAD" {
                (name == "HEAD").then_some(object)
            } else if requested.starts_with("refs/") {
                (name == requested).then_some(object)
            } else {
                (name == &gix::bstr::BString::from(format!("refs/heads/{requested}"))
                    || name == &gix::bstr::BString::from(format!("refs/tags/{requested}")))
                    .then_some(object)
            }
        })
        .or_else(|| {
            (requested.len() == 40 && requested.chars().all(|c| c.is_ascii_hexdigit()))
                .then(|| gix::ObjectId::from_hex(requested.as_bytes()).expect("40 hex parses"))
        })
        .ok_or(GitError::NoHeadRef)?;
    repo.find_object(head_id)
        .map_err(|_| GitError::ResolvedCommitMissing)?;

    // The measurements + the refusal list (the walk trips the ceilings and
    // names the refused entries).
    let head_commit = repo
        .find_object(head_id)
        .map_err(|_| GitError::ResolvedCommitMissing)?
        .try_into_commit()
        .map_err(|e| GitError::TransferFailed(format!("the HEAD is not a commit: {e}")))?;
    let mut counts = (0u64, 0u32);
    let mut paths = Vec::new();
    walk_tree(
        &repo,
        head_commit
            .tree_id()
            .map_err(|e| GitError::TransferFailed(e.to_string()))?
            .detach(),
        0,
        "",
        &mut counts,
        &mut paths,
        limits,
    )?;
    deadline.check()?;
    let object_count = repo
        .objects
        .iter()
        .map_err(|e| GitError::TransferFailed(e.to_string()))?
        .count() as u64;
    if object_count > limits.max_objects {
        return Err(GitError::BudgetExceeded {
            what: "object count",
            limit: limits.max_objects,
            actual: object_count,
        });
    }
    let odb_path = target_dir.join("objects");
    let odb_bytes = dir_size(&odb_path);
    if odb_bytes > limits.max_bytes {
        return Err(GitError::BudgetExceeded {
            what: "object database bytes",
            limit: limits.max_bytes,
            actual: odb_bytes,
        });
    }
    Ok(GitAcquisition {
        _workspace: workspace,
        resolved_commit: head_id.to_string(),
        object_count,
        file_count: counts.0,
        max_path_depth: counts.1,
        odb_bytes,
        odb_path,
        paths,
    })
}

/// The Git LFS v1 pointer's version line, which the specification makes the
/// pointer's FIRST line — 42 bytes, no trailing newline of its own here so
/// that a pointer ending at the line boundary matches as readily as one with
/// the `oid`/`size` lines after it.
const LFS_POINTER_VERSION_LINE: &[u8] = b"version https://git-lfs.github.com/spec/v1";

/// Whether a blob IS a Git LFS pointer file, and therefore a stand-in for
/// content this acquisition never fetched.
///
/// The specification makes the `version` line the pointer's FIRST line, so
/// this is a prefix test at offset 0 rather than a search. The superseded
/// predicate looked for the ten-byte run `version ht` anywhere in the first 64
/// bytes, which refused any file that merely QUOTED the spec URL near its
/// start — a repository documenting LFS was rejected as if it carried the
/// content it describes. That defect over-refused; it was never a way past
/// the gate.
///
/// The line must also END where a line ends — at the newline, or at the end
/// of a blob carrying nothing else — so a longer URL with this one as its
/// prefix is not a pointer. The `oid` and `size` lines are deliberately NOT
/// required: a truncated or malformed pointer is still not the content, and
/// the refusal is the same either way.
fn is_lfs_pointer(data: &[u8]) -> bool {
    matches!(
        data.strip_prefix(LFS_POINTER_VERSION_LINE),
        Some([]) | Some([b'\n', ..])
    )
}

/// The recursive tree walk: counts files, trips the depth ceiling, and
/// enforces the refusal list — a gitlink (submodule) entry or an LFS
/// pointer file is the NAMED refusal, never a silent skip.
fn walk_tree(
    repo: &gix::Repository,
    tree_id: gix::ObjectId,
    depth: u32,
    prefix: &str,
    counts: &mut (u64, u32),
    paths: &mut Vec<String>,
    limits: &GitLimits,
) -> Result<(), GitError> {
    if depth > limits.max_depth {
        return Err(GitError::DepthCeilingExceeded {
            limit: limits.max_depth,
            actual: depth,
        });
    }
    let tree = repo
        .find_object(tree_id)
        .map_err(|e| GitError::TransferFailed(e.to_string()))?
        .try_into_tree()
        .map_err(|e| GitError::TransferFailed(e.to_string()))?;
    for entry in tree.iter() {
        let entry = entry.map_err(|e| GitError::TransferFailed(e.to_string()))?;
        if entry.mode().is_commit() {
            return Err(GitError::Refused {
                what: "submodule",
                detail: format!("the gitlink `{}`", entry.filename()),
            });
        }
        let entry_depth = depth + 1;
        if entry_depth > limits.max_depth {
            return Err(GitError::DepthCeilingExceeded {
                limit: limits.max_depth,
                actual: entry_depth,
            });
        }
        let name = entry.filename().to_string();
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if entry.mode().is_tree() {
            walk_tree(
                repo,
                entry.oid().to_owned(),
                entry_depth,
                &path,
                counts,
                paths,
                limits,
            )?;
            continue;
        }
        counts.0 += 1;
        counts.1 = counts.1.max(entry_depth);
        paths.push(path);
        if counts.0 > limits.max_files {
            return Err(GitError::BudgetExceeded {
                what: "file count",
                limit: limits.max_files,
                actual: counts.0,
            });
        }
        let blob = repo
            .find_object(entry.oid())
            .map_err(|e| GitError::TransferFailed(e.to_string()))?
            .try_into_blob()
            .ok();
        if let Some(blob) = blob {
            if is_lfs_pointer(&blob.data) {
                return Err(GitError::Refused {
                    what: "Git LFS",
                    detail: format!("the pointer file `{}`", entry.filename()),
                });
            }
        }
    }
    Ok(())
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- pure: the URL grammar -------------------------------------------

    #[test]
    fn harden_git_url_refuses_the_policy_violations() {
        for (raw, expected) in [
            (
                "http://example.org/repo.git",
                GitError::SchemeNotAllowed("http".into()),
            ),
            (
                "git://example.org/repo.git",
                GitError::SchemeNotAllowed("git".into()),
            ),
            (
                "file:///tmp/repo.git",
                GitError::SchemeNotAllowed("file".into()),
            ),
            (
                "https://user@example.org/repo.git",
                GitError::UserinfoForbidden,
            ),
            (
                "https://user:pass@example.org/repo.git",
                GitError::UserinfoForbidden,
            ),
            (
                "https://example.org:8443/repo.git",
                GitError::PortNotAllowed(8443),
            ),
            (
                "https://example.org/repo.git#..evil",
                GitError::RefSelectorInvalid("..evil".into()),
            ),
            (
                "https://example.org/repo.git#-lead",
                GitError::RefSelectorInvalid("-lead".into()),
            ),
            (
                "https://example.org/repo.git#sp ace",
                GitError::UrlHasControlCharacters,
            ),
            (
                "https://example.org/repo.git#ref~1",
                GitError::RefSelectorInvalid("ref~1".into()),
            ),
            (
                "https://example.org/\\@127.0.0.1/x",
                GitError::UrlHasControlCharacters,
            ),
        ] {
            assert_eq!(
                harden_git_url(raw),
                Err(expected.clone()),
                "the URL `{raw}` must refuse with the named reason"
            );
        }
        // The alternative-literal spellings: the WHATWG parser may
        // normalize some into IPv4 literals — every such form must
        // REFUSE, either by the ambiguity name or (normalized) by the
        // classifier the pre-flight runs.
        for raw in ["https://2130706433/repo.git", "https://127.1/repo.git"] {
            match harden_git_url(raw) {
                Err(GitError::AmbiguousNumericHost(_)) => {}
                Ok(url) => match url.host() {
                    Some(url::Host::Ipv4(ip)) if ip.is_loopback() => {}
                    other => {
                        panic!("`{raw}` must refuse or normalize to a loopback literal: {other:?}")
                    }
                },
                other => panic!("`{raw}` must refuse with the ambiguity name: {other:?}"),
            }
        }
        // The accepted grammar: the bare URL + every selector form.
        for raw in [
            "https://example.org/repo.git",
            "https://example.org/repo.git#main",
            "https://example.org/repo.git#feature/one",
            "https://example.org/repo.git#refs/tags/v1.0",
            "https://example.org/repo.git#0123456789abcdef0123456789abcdef01234567",
        ] {
            assert!(
                harden_git_url(raw).is_ok(),
                "the URL `{raw}` is the accepted grammar"
            );
        }
        // The selector the fetch uses: the fragment, or HEAD.
        assert_eq!(
            fetch_refspec(&harden_git_url("https://example.org/repo.git#main").unwrap()),
            "main"
        );
        assert_eq!(
            fetch_refspec(&harden_git_url("https://example.org/repo.git").unwrap()),
            "HEAD"
        );
    }

    // ---- the pre-flight: the SSRF proof for R1 ---------------------------

    #[tokio::test(flavor = "multi_thread")]
    async fn the_preflight_refuses_the_loopback_literal_before_any_socket() {
        let acquirer = GitFetcher::new(GitLimits::default());
        match acquirer.acquire("https://127.0.0.1/repo.git").await {
            Err(GitError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("loopback"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the loopback literal must refuse: {other:?}"),
        }
        match acquirer.acquire("https://10.0.0.1/repo.git").await {
            Err(GitError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("private"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the private literal must refuse: {other:?}"),
        }
    }

    // ---- the acquisition (offline, over the injected file transport) -----

    /// The test factory: the file transport reaches the local source repo
    /// (the offline wire path — no network, no real DNS).
    fn file_factory(path: std::path::PathBuf) -> Box<TransportFactory> {
        Box::new(move |_url, version| {
            let transport = gix_transport::client::blocking_io::file::connect(
                gix::path::into_bstr(path.clone()).into_owned(),
                version,
                false,
            )
            .map_err(|e| GitError::TransferFailed(e.to_string()))?;
            Ok(Box::new(transport)
                as Box<
                    dyn gix_transport::client::blocking_io::Transport + Send,
                >)
        })
    }

    /// A source repository: a blob, a nested tree, one commit.
    /// The fixture's OWN commit identity.
    ///
    /// `repo.commit` resolves its signature from git configuration, so these
    /// fixtures silently borrowed whatever identity the developer's machine
    /// happened to carry. A CI runner has none and `gix` refused with
    /// `AuthorMissing`, so the tests passed locally for the project's whole
    /// life and could not pass anywhere clean. A fixture owns its identity the
    /// same way it owns its storage.
    fn fixture_identity() -> gix::actor::SignatureRef<'static> {
        gix::actor::SignatureRef {
            name: "ReasonBraid Fixture".into(),
            email: "fixture@reasonbraid.invalid".into(),
            time: "0 +0000",
        }
    }

    /// A fixture repository, opened the way the acquisition opens its own —
    /// isolated from the host's system, application and user git configuration
    /// and from the `GIT_*`/`SSH_*` environment — so the suite's INPUTS do not
    /// change with whoever runs it. `.7.2.4` removed `Permissions::all()` from
    /// production and measured that the fixtures had kept it.
    ///
    /// ⛔ `the_git_environment_probe` deliberately does NOT use this, and must
    /// not be "fixed" to: its first open is the matched pair's other half, and
    /// it has to keep honouring the setting to prove the setting arrived.
    fn init_fixture_repo(dir: &std::path::Path) -> gix::Repository {
        gix::ThreadSafeRepository::init_opts(
            dir,
            gix::create::Kind::WithWorktree,
            gix::create::Options::default(),
            gix::open::Options::isolated(),
        )
        .map(Into::into)
        .expect("the fixture repository inits")
    }

    fn source_repo(dir: &std::path::Path) -> gix::ObjectId {
        let repo = init_fixture_repo(dir);
        let blob = repo.write_blob(b"hello world").expect("the blob writes");
        let nested = repo
            .write_object(&gix::objs::Tree {
                entries: vec![gix::objs::tree::Entry {
                    mode: gix::objs::tree::EntryKind::Blob.into(),
                    oid: blob.detach(),
                    filename: "nested.txt".into(),
                }],
            })
            .expect("the nested tree writes")
            .detach();
        let tree = repo
            .write_object(&gix::objs::Tree {
                entries: vec![
                    gix::objs::tree::Entry {
                        mode: gix::objs::tree::EntryKind::Blob.into(),
                        oid: blob.detach(),
                        filename: "a.txt".into(),
                    },
                    gix::objs::tree::Entry {
                        mode: gix::objs::tree::EntryKind::Tree.into(),
                        oid: nested,
                        filename: "dir".into(),
                    },
                ],
            })
            .expect("the tree writes")
            .detach();
        repo.commit_as(
            fixture_identity(),
            fixture_identity(),
            "HEAD",
            "first",
            tree,
            gix::commit::NO_PARENT_IDS,
        )
        .expect("the commit writes")
        .detach()
    }

    fn acquire_local(
        source_dir: &std::path::Path,
        limits: &GitLimits,
    ) -> Result<GitAcquisition, GitError> {
        // The same owned workspace production uses, so the fixtures exercise
        // the real storage boundary instead of an ambient temporary directory.
        let workspace =
            Arc::new(OwnedDirectory::create("git", "test-acquire").expect("the workspace creates"));
        let url =
            Url::parse(&format!("file://{}", source_dir.display())).expect("the file url parses");
        acquire_into(
            workspace,
            &url,
            limits,
            file_factory(source_dir.to_path_buf()),
        )
    }

    #[test]
    fn the_acquisition_resolves_the_commit_and_measures() {
        let tmp = OwnedDirectory::create("git", "test-src").expect("the fixture workspace creates");
        let tmp = tmp.path();
        let source_dir = tmp.join("source");
        std::fs::create_dir_all(&source_dir).expect("the source dir creates");
        let first = source_repo(&source_dir);
        let acquisition =
            acquire_local(&source_dir, &GitLimits::default()).expect("the acquisition succeeds");
        assert_eq!(acquisition.resolved_commit, first.to_string());
        assert_eq!(acquisition.file_count, 2, "the blob + the nested blob");
        assert_eq!(acquisition.max_path_depth, 2);
        assert_eq!(
            acquisition.object_count, 4,
            "blob + nested tree + root tree + commit"
        );
        assert!(acquisition.odb_bytes > 0);
        assert!(acquisition.odb_path.is_dir());
    }

    #[test]
    fn the_submodule_and_lfs_entries_refuse_with_their_names() {
        let tmp = OwnedDirectory::create("git", "test-ref").expect("the fixture workspace creates");
        let tmp = tmp.path();

        // The submodule: a gitlink entry in the tree.
        let sub_dir = tmp.join("sub");
        std::fs::create_dir_all(&sub_dir).expect("the sub dir creates");
        {
            let repo = init_fixture_repo(&sub_dir);
            let blob = repo.write_blob(b"x").expect("the blob writes");
            let tree = repo
                .write_object(&gix::objs::Tree {
                    entries: vec![gix::objs::tree::Entry {
                        mode: gix::objs::tree::EntryKind::Blob.into(),
                        oid: blob.detach(),
                        filename: "a.txt".into(),
                    }],
                })
                .expect("the tree writes")
                .detach();
            let sub_commit = repo
                .commit_as(
                    fixture_identity(),
                    fixture_identity(),
                    "HEAD",
                    "sub",
                    tree,
                    gix::commit::NO_PARENT_IDS,
                )
                .expect("the sub commit writes")
                .detach();
            // A second commit whose tree carries a gitlink to the first.
            let gitlink_tree = repo
                .write_object(&gix::objs::Tree {
                    entries: vec![gix::objs::tree::Entry {
                        mode: gix::objs::tree::EntryKind::Commit.into(),
                        oid: sub_commit,
                        filename: "sub".into(),
                    }],
                })
                .expect("the gitlink tree writes")
                .detach();
            repo.commit_as(
                fixture_identity(),
                fixture_identity(),
                "HEAD",
                "with-submodule",
                gitlink_tree,
                [sub_commit],
            )
            .expect("the gitlink commit writes");
        }
        match acquire_local(&sub_dir, &GitLimits::default()) {
            Err(GitError::Refused {
                what: "submodule", ..
            }) => {}
            other => panic!("the gitlink must refuse as a submodule: {other:?}"),
        }

        // The LFS pointer: a blob carrying the pointer signature.
        let lfs_dir = tmp.join("lfs");
        std::fs::create_dir_all(&lfs_dir).expect("the lfs dir creates");
        {
            let repo = init_fixture_repo(&lfs_dir);
            let pointer = repo
                .write_blob(
                    b"version https://git-lfs.github.com/spec/v1\noid sha256:aaaa\nsize 1\n",
                )
                .expect("the pointer writes");
            let tree = repo
                .write_object(&gix::objs::Tree {
                    entries: vec![gix::objs::tree::Entry {
                        mode: gix::objs::tree::EntryKind::Blob.into(),
                        oid: pointer.detach(),
                        filename: "big.bin".into(),
                    }],
                })
                .expect("the pointer tree writes")
                .detach();
            repo.commit_as(
                fixture_identity(),
                fixture_identity(),
                "HEAD",
                "lfs",
                tree,
                gix::commit::NO_PARENT_IDS,
            )
            .expect("the lfs commit writes");
        }
        match acquire_local(&lfs_dir, &GitLimits::default()) {
            Err(GitError::Refused {
                what: "Git LFS", ..
            }) => {}
            other => panic!("the LFS pointer must refuse: {other:?}"),
        }
    }

    /// A one-file repository whose single blob carries `content` — the shape
    /// the LFS gate reads, with nothing else in the tree to distract it.
    fn one_blob_repo(dir: &std::path::Path, filename: &str, content: &[u8]) {
        std::fs::create_dir_all(dir).expect("the fixture dir creates");
        let repo = init_fixture_repo(dir);
        let blob = repo.write_blob(content).expect("the blob writes");
        let tree = repo
            .write_object(&gix::objs::Tree {
                entries: vec![gix::objs::tree::Entry {
                    mode: gix::objs::tree::EntryKind::Blob.into(),
                    oid: blob.detach(),
                    filename: filename.into(),
                }],
            })
            .expect("the fixture tree writes")
            .detach();
        repo.commit_as(
            fixture_identity(),
            fixture_identity(),
            "HEAD",
            "fixture",
            tree,
            gix::commit::NO_PARENT_IDS,
        )
        .expect("the fixture commit writes");
    }

    #[test]
    fn the_pointer_version_line_alone_still_refuses() {
        // The 42-byte version line with NOTHING after it: no trailing newline,
        // no `oid`, no `size`. The spec makes the version line the pointer's
        // first line, so this is the smallest thing that is still a pointer,
        // and the gate must not need the rest of the file to say so.
        let tmp = OwnedDirectory::create("git", "test-lfs-prefix")
            .expect("the fixture workspace creates");
        let source = tmp.path().join("prefix");
        one_blob_repo(&source, "big.bin", LFS_POINTER_VERSION_LINE);
        match acquire_local(&source, &GitLimits::default()) {
            Err(GitError::Refused {
                what: "Git LFS",
                detail,
            }) => assert!(
                detail.contains("big.bin"),
                "the refusal names the path: {detail}"
            ),
            other => panic!("the bare version line must refuse: {other:?}"),
        }
    }

    #[test]
    fn a_file_quoting_the_pointer_line_is_acquired() {
        // The false refusal this leaf repairs. A repository that DOCUMENTS Git
        // LFS carries the version line as prose, not at offset 0 — the gate
        // used to search the first 64 bytes for the ten-byte run `version ht`
        // and reject the documentation as if it were the content it describes.
        let tmp =
            OwnedDirectory::create("git", "test-lfs-prose").expect("the fixture workspace creates");
        let source = tmp.path().join("prose");
        one_blob_repo(
            &source,
            "lfs.md",
            b"`version https://git-lfs.github.com/spec/v1` is the pointer's first line.\n",
        );
        let acquisition =
            acquire_local(&source, &GitLimits::default()).expect("the documentation is acquired");
        assert_eq!(acquisition.file_count, 1);
        assert_eq!(acquisition.paths, vec!["lfs.md".to_string()]);
    }

    /// The bare repository's `HEAD`, read as the file git writes it.
    fn head_line(git_dir: &std::path::Path) -> String {
        std::fs::read_to_string(git_dir.join("HEAD"))
            .expect("the HEAD file reads")
            .trim()
            .to_string()
    }

    /// The child half of `a_git_setting_in_the_environment_cannot_reach_the_acquisition`.
    ///
    /// ⚠️ `#[ignore]`d on purpose: it is meaningless without the environment
    /// its parent sets, and setting `GIT_CONFIG_*` in THIS process would mutate
    /// state every other test in the binary shares — which is why Rust makes
    /// `set_var` unsafe. The parent re-executes this binary so the setting is
    /// in place before the process starts.
    #[test]
    #[ignore = "driven by a_git_setting_in_the_environment_cannot_reach_the_acquisition, which supplies its environment"]
    fn the_git_environment_probe() {
        let tmp =
            OwnedDirectory::create("git", "test-gix-env").expect("the fixture workspace creates");

        // The superseded open, kept as the matched pair's other half: it proves
        // the setting really does reach a DEFAULT open on this host, so a green
        // result below is a refusal rather than a setting that never arrived.
        let ambient = tmp.path().join("ambient");
        std::fs::create_dir_all(&ambient).expect("the ambient dir creates");
        gix::init_bare(&ambient).expect("the default open succeeds");
        println!("default-permissions HEAD: {}", head_line(&ambient));

        // The acquisition's own open, exactly as production calls it.
        let owned = tmp.path().join("owned");
        std::fs::create_dir_all(&owned).expect("the owned dir creates");
        init_acquisition_repository(&owned).expect("the acquisition open succeeds");
        println!("acquisition HEAD: {}", head_line(&owned));

        // The suite's own inputs: the real fixture builder the acquisition
        // controls use, not a stand-in for it.
        let fixture = tmp.path().join("fixture");
        std::fs::create_dir_all(&fixture).expect("the fixture dir creates");
        source_repo(&fixture);
        println!("fixture HEAD: {}", head_line(&fixture.join(".git")));
    }

    #[test]
    fn a_git_setting_in_the_environment_cannot_reach_the_acquisition() {
        // R1 fetches a caller-supplied URL. `gix::init_bare` opens the target
        // with `Permissions::all()`, so the operator's system, application and
        // user configuration, `include`/`includeIf` directives reaching outside
        // the repository, and every `GIT_*`/`SSH_*` environment category are
        // honoured while that fetch runs. `init.defaultBranch` is the readable
        // end of that: it is set here through `GIT_CONFIG_COUNT`, the
        // environment-sourced config source, and it lands in the repository's
        // own `HEAD`.
        let exe = std::env::current_exe().expect("the test binary names itself");
        let probe = std::process::Command::new(exe)
            .args([
                "--exact",
                "git::tests::the_git_environment_probe",
                "--include-ignored",
                "--nocapture",
            ])
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "init.defaultBranch")
            .env("GIT_CONFIG_VALUE_0", "smuggled")
            .output()
            .expect("the probe process runs");
        let out = String::from_utf8_lossy(&probe.stdout).into_owned();
        let err = String::from_utf8_lossy(&probe.stderr).into_owned();
        assert!(probe.status.success(), "the probe failed:\n{out}\n{err}");
        assert!(
            out.contains("default-permissions HEAD: ref: refs/heads/smuggled"),
            "the setting must reach a DEFAULT open, or this control proves nothing:\n{out}"
        );
        assert!(
            out.contains("acquisition HEAD: ref: refs/heads/main"),
            "the setting must not reach the acquisition's own open:\n{out}"
        );
        assert!(
            out.contains("fixture HEAD: ref: refs/heads/main"),
            "the setting must not reach the suite's own fixtures either:\n{out}"
        );
    }

    #[test]
    fn a_version_line_naming_another_spec_is_acquired() {
        // Anchoring alone is not enough: a file whose FIRST line is a version
        // line for some other specification is not an LFS pointer either, so
        // the gate matches the LFS spec URL rather than the word `version`.
        let tmp =
            OwnedDirectory::create("git", "test-lfs-other").expect("the fixture workspace creates");
        let source = tmp.path().join("other");
        one_blob_repo(
            &source,
            "manifest.txt",
            b"version https://example.org/manifest/v1\nentries 4\n",
        );
        let acquisition =
            acquire_local(&source, &GitLimits::default()).expect("the manifest is acquired");
        assert_eq!(acquisition.file_count, 1);
        assert_eq!(acquisition.paths, vec!["manifest.txt".to_string()]);
    }

    #[test]
    fn the_receipt_carries_the_commit_the_digest_and_the_manifest() {
        let tmp =
            OwnedDirectory::create("git", "test-receipt").expect("the fixture workspace creates");
        let tmp = tmp.path();
        let source_dir = tmp.join("source");
        std::fs::create_dir_all(&source_dir).expect("the source dir creates");
        let first = source_repo(&source_dir);
        let acquisition =
            acquire_local(&source_dir, &GitLimits::default()).expect("the acquisition succeeds");
        let acquired_at = chrono::DateTime::parse_from_rfc3339("2026-09-07T14:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let receipt = GitReceipt::from_acquisition(
            "https://example.org/repo.git#main",
            "main",
            &acquisition,
            acquired_at,
        );
        assert_eq!(receipt.resolved_commit, first.to_string());
        assert_eq!(receipt.requested_url, "https://example.org/repo.git#main");
        assert_eq!(receipt.requested_ref, "main");
        assert!(receipt.digest.starts_with("sha256:"));
        assert_eq!(receipt.digest.len(), 7 + 64);
        assert_eq!(receipt.digest, git_digest(&acquisition.odb_path));
        assert_eq!(
            receipt.chain,
            vec!["https://example.org/repo.git#main".to_owned()]
        );
        assert_eq!(receipt.object_count, 4);
        assert_eq!(receipt.file_count, 2);
        assert_eq!(receipt.max_path_depth, 2);
        // The manifest: the included paths, the excluded side empty (the
        // refusal list refuses — never excludes).
        assert!(receipt.manifest.included.contains(&"a.txt".to_owned()));
        assert!(receipt
            .manifest
            .included
            .contains(&"dir/nested.txt".to_owned()));
        assert!(receipt.manifest.excluded.is_empty());
        assert_eq!(receipt.acquired_at, acquired_at);
    }

    /// `SIGNOFF-REPAIR.7.2.7` — the ceiling is ENFORCED, and refuses by name.
    ///
    /// 🔴 Observed RED first: against the pre-repair source this returns
    /// `Ok(..)`, because `max_time` was declared, defaulted to 120 s and read
    /// by nothing, while `GitError::TimedOut` was declared, given a Display
    /// message and mapped to the `timed_out` wire reason — and constructed by
    /// nothing. The whole refusal path existed and was unreachable.
    #[test]
    fn the_time_ceiling_refuses_by_name() {
        let tmp = OwnedDirectory::create("git", "test-src").expect("the fixture workspace creates");
        let tmp = tmp.path();
        let source_dir = tmp.join("source");
        std::fs::create_dir_all(&source_dir).expect("the source dir creates");
        source_repo(&source_dir);
        let limits = GitLimits {
            max_time: Duration::from_millis(0),
            ..GitLimits::default()
        };
        let result = acquire_local(&source_dir, &limits);
        assert!(
            matches!(result, Err(GitError::TimedOut)),
            "a zero ceiling must refuse as TimedOut, got {result:?}"
        );
        assert_eq!(
            GitError::TimedOut.to_string(),
            "the acquisition exceeded the time ceiling",
            "the refusal names the ceiling rather than the remote"
        );
    }

    /// ⭐ THE NEGATIVE HALF, and the acceptance demanded it by name: the repair
    /// is a BOUND, not a prohibition. Without this arm the ceiling is satisfied
    /// by an acquisition that refuses everything.
    ///
    /// ⚠️ LABELLED: this arm passes BOTH before and after the repair, and that
    /// is what it is for. It covers the rule's boundary, not the defect — its
    /// own red comes from a degenerate ceiling that refuses everything, not
    /// from the pre-fix code, which refused nothing
    /// (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`).
    #[test]
    fn a_transfer_inside_the_time_ceiling_still_succeeds() {
        let tmp = OwnedDirectory::create("git", "test-src").expect("the fixture workspace creates");
        let tmp = tmp.path();
        let source_dir = tmp.join("source");
        std::fs::create_dir_all(&source_dir).expect("the source dir creates");
        let first = source_repo(&source_dir);
        let acquisition = acquire_local(&source_dir, &GitLimits::default())
            .expect("a transfer inside the default 120 s ceiling still succeeds");
        assert_eq!(acquisition.resolved_commit, first.to_string());
    }

    /// ⛔ And the watchdog does not outlive the acquisition. A 120 s ceiling
    /// must not cost a 120 s thread on every successful clone, which is what a
    /// watchdog without an early exit would do.
    ///
    /// ⚠️ LABELLED, like the arm above: it passes before and after, because it
    /// covers a DIFFERENT failure — a resource leak the repair could have
    /// introduced — rather than the unenforced ceiling. Its red is a `Drop`
    /// that forgets to release the flag.
    #[test]
    fn the_watchdog_exits_when_the_acquisition_finishes() {
        use std::sync::atomic::Ordering;
        let deadline = Deadline::start(Duration::from_secs(3600));
        let finished = Arc::clone(&deadline.finished);
        assert!(!finished.load(Ordering::Relaxed), "it starts unfinished");
        drop(deadline);
        assert!(
            finished.load(Ordering::Relaxed),
            "dropping the deadline releases the watchdog"
        );
    }

    #[test]
    fn the_budget_ceilings_trip_with_their_names() {
        let tmp =
            OwnedDirectory::create("git", "test-budget").expect("the fixture workspace creates");
        let tmp = tmp.path();
        let source_dir = tmp.join("source");
        std::fs::create_dir_all(&source_dir).expect("the source dir creates");
        source_repo(&source_dir);
        let limits = GitLimits {
            max_files: 1,
            ..GitLimits::default()
        };
        match acquire_local(&source_dir, &limits) {
            Err(GitError::BudgetExceeded {
                what: "file count",
                limit: 1,
                ..
            }) => {}
            other => panic!("the file ceiling must trip: {other:?}"),
        }
        let limits = GitLimits {
            max_depth: 1,
            ..GitLimits::default()
        };
        match acquire_local(&source_dir, &limits) {
            Err(GitError::DepthCeilingExceeded { limit: 1, .. }) => {}
            other => panic!("the depth ceiling must trip: {other:?}"),
        }
        let limits = GitLimits {
            max_objects: 2,
            ..GitLimits::default()
        };
        match acquire_local(&source_dir, &limits) {
            Err(GitError::BudgetExceeded {
                what: "object count",
                limit: 2,
                ..
            }) => {}
            other => panic!("the object ceiling must trip: {other:?}"),
        }
    }

    // ---- the classified dial: every hop, including an IP literal ----------

    /// A resolver that answers only the hosts a test names, so the wire tests
    /// stay offline (the `.2.2` seam, carried over).
    struct StaticResolver(std::collections::HashMap<String, IpAddr>);

    impl DestinationResolver for StaticResolver {
        fn resolve(
            &self,
            host: &str,
            port: u16,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = io::Result<Vec<std::net::SocketAddr>>> + Send>,
        > {
            let found = self
                .0
                .get(host)
                .map(|ip| vec![std::net::SocketAddr::new(*ip, port)]);
            Box::pin(async move {
                found.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unknown test host"))
            })
        }
    }

    /// 🔴 `SIGNOFF-REPAIR.7.2.2` — a redirect hop to an IP LITERAL reached no
    /// destination control at all.
    ///
    /// `Policy::limited(5)` auto-followed up to five hops with only the DNS
    /// belt behind them, and hyper-util 0.1.20 skips the resolver when the host
    /// is already an IP address — in its own source. `GitFetcher::classify` is
    /// a real pre-flight, but it runs ONCE, on the initial URL. So an origin
    /// that REDIRECTED into a refused range was dialed with nothing consulted.
    ///
    /// ⚠️ **The deviations a local origin forces, named rather than left to be
    /// discovered.** (1) The scheme is `http`: production reaches this client
    /// only through `harden_git_url`, which is https-only, and a local TLS
    /// origin would need a trust anchor the production client deliberately does
    /// not have. (2) The policy allows `127.0.0.1`, which `ssrf::evaluate`
    /// refuses, exactly as the R0 wire tests widen it. Everything else — the
    /// redirect policy, the DNS belt, `no_proxy`, the hop cap — is the shipped
    /// construction.
    ///
    /// ⭐ **The redirect target is `0.0.0.0`, and that is the whole instrument.**
    /// It classifies as `reserved`, and the kernel routes a connection to it at
    /// the local host — measured before this test was written — so the SAME
    /// origin answers `/private` and its hit counter reports whether the hop was
    /// actually dialed. The unrepaired client scored 1; a refusal that merely
    /// failed to connect could not be told from one that classified.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_redirect_hop_to_an_ip_literal_is_classified_like_every_other_dial() {
        use axum::http::header::LOCATION;
        use axum::http::StatusCode;
        use axum::routing::get;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("the test origin binds");
        let port = listener.local_addr().expect("the origin port").port();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        let app = axum::Router::new()
            .route(
                "/start",
                get(move || async move {
                    // An IP literal, so no resolver is consulted for this hop.
                    (
                        StatusCode::FOUND,
                        [(LOCATION, format!("http://0.0.0.0:{port}/private"))],
                    )
                }),
            )
            .route(
                "/private",
                get(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    async { "the refused destination answered" }
                }),
            );
        let origin = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let resolver: Arc<dyn DestinationResolver> = Arc::new(StaticResolver(
            [("git.test".to_owned(), IpAddr::from([127, 0, 0, 1]))]
                .into_iter()
                .collect(),
        ));
        let policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync> =
            Arc::new(|ip: &IpAddr| match ip {
                IpAddr::V4(v4) if v4.octets() == [127, 0, 0, 1] => SsrfVerdict::Allowed,
                other => crate::ssrf::evaluate(*other),
            });

        let url = format!("http://git.test:{port}/start");
        let result = tokio::task::spawn_blocking(move || {
            let http = ClassifiedGitHttp::new(resolver, policy)?;
            http.send(reqwest::Method::GET, &url, Vec::<String>::new(), None)
                .map(|(_, response)| response.status().as_u16())
        })
        .await
        .expect("the blocking dial joins");

        origin.abort();

        match result {
            Err(GitError::DestinationRefused { host, reason }) => {
                assert_eq!(host, "0.0.0.0", "the refusal names the hop's address");
                assert!(
                    reason.contains("reserved"),
                    "and the class it belongs to: {reason}"
                );
            }
            other => panic!("the IP-literal hop must be refused by name: {other:?}"),
        }
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "the refused destination was never dialed — a named refusal that still \
             connected would be a report, not a control"
        );
    }

    /// The other half of the same contract: a hop the policy ALLOWS is still
    /// followed, so the repair refuses a class rather than refusing redirects.
    #[tokio::test(flavor = "multi_thread")]
    async fn an_allowed_redirect_hop_is_still_followed() {
        use axum::http::header::LOCATION;
        use axum::http::StatusCode;
        use axum::routing::get;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("the test origin binds");
        let port = listener.local_addr().expect("the origin port").port();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        let app = axum::Router::new()
            .route(
                "/start",
                get(move || async move {
                    (
                        StatusCode::FOUND,
                        [(LOCATION, format!("http://127.0.0.1:{port}/landing"))],
                    )
                }),
            )
            .route(
                "/landing",
                get(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    async { "followed" }
                }),
            );
        let origin = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let resolver: Arc<dyn DestinationResolver> = Arc::new(StaticResolver(
            [("git.test".to_owned(), IpAddr::from([127, 0, 0, 1]))]
                .into_iter()
                .collect(),
        ));
        let policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync> =
            Arc::new(|ip: &IpAddr| match ip {
                IpAddr::V4(v4) if v4.octets() == [127, 0, 0, 1] => SsrfVerdict::Allowed,
                other => crate::ssrf::evaluate(*other),
            });

        let url = format!("http://git.test:{port}/start");
        let status = tokio::task::spawn_blocking(move || {
            let http = ClassifiedGitHttp::new(resolver, policy)?;
            http.send(reqwest::Method::GET, &url, Vec::<String>::new(), None)
                .map(|(_, response)| response.status().as_u16())
        })
        .await
        .expect("the blocking dial joins");

        origin.abort();

        assert_eq!(status, Ok(200), "an allowed hop is followed");
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "and it reached the destination exactly once"
        );
    }
}
