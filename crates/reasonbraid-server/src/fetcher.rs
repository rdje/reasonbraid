//! The R0 safe HTTPS fetcher (ROADMAP.md §12.3–12.4; the `PHASE-4.2.2` leaf).
//!
//! The engine the `.2.3` R0 resolver entry will consume. It enforces the `.2.1`
//! destination policy at TWO layers: a pre-flight resolve+classify that names
//! the refusing class BEFORE any socket opens (the measured SSRF proof), and a
//! classified DNS belt installed inside reqwest's resolver hook, so a dial can
//! never touch an address the policy refused even if the resolver's answer
//! changed between the pre-flight and the connection (the re-resolution/
//! rebinding check). Redirects are followed MANUALLY: every hop re-parses,
//! re-classifies, and counts against the hop cap — there is no auto-follow.
//! TLS verifies against the system roots; no proxy environment is honored;
//! no cookie state and no ambient credentials exist (the client sets exactly
//! one static user agent, nothing else).

use std::fmt;
use std::future::Future;
use std::io;
use std::io::Read as _;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, LOCATION};
use reqwest::{Client, Method, Response, StatusCode};
use url::{Host, Url};

use crate::ssrf::{self, SsrfVerdict};

/// The R0 ceilings (ROADMAP.md §12.4: byte/time ceilings, decompression-ratio
/// limits, redirect caps). The dev-profile defaults are deliberately small for
/// a text/HTML evidence path — the `.2.3` receipt records them with the bytes.
#[derive(Debug, Clone)]
pub struct FetchLimits {
    /// Hard cap on the DECODED body bytes (the ceiling is on the acquired
    /// bytes, so a lying `Content-Length` cannot buy more).
    pub max_bytes: usize,
    /// Ceiling on the WHOLE acquisition: every hop plus the body read.
    pub max_time: Duration,
    /// How many redirect hops may be followed before the named refusal.
    pub max_redirects: u8,
    /// decoded-bytes / declared-bytes may not exceed this (the zip-bomb brake).
    pub max_decompression_ratio: f64,
    /// The raw URL string length cap.
    pub max_url_length: usize,
    /// The per-connection dial ceiling.
    pub connect_timeout: Duration,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            max_bytes: 4 * 1024 * 1024,
            max_time: Duration::from_secs(30),
            max_redirects: 5,
            max_decompression_ratio: 10.0,
            max_url_length: 2048,
            connect_timeout: Duration::from_secs(10),
        }
    }
}

/// Every failure is a named refusal — the caller (`.2.3`) maps these onto the
/// typed resolution results, never a fabricated success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    UrlTooLong(usize),
    UrlHasControlCharacters,
    UrlUnparseable,
    SchemeNotAllowed(String),
    UserinfoForbidden,
    AmbiguousNumericHost(String),
    PortNotAllowed(u16),
    NoHost,
    DnsLookupFailed,
    NoAddresses,
    DestinationRefused { host: String, reason: String },
    HopLimitExceeded(u8),
    MissingRedirectLocation,
    UnexpectedStatus(u16),
    MediaTypeRefused(String),
    EmptyBody,
    ByteCeilingExceeded(usize),
    DecompressionRatioExceeded { declared: u64, decoded: usize },
    ReadFailed,
    ConnectFailed,
    TimedOut,
    ClientBuildFailed,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UrlTooLong(max) => write!(f, "the URL exceeds the {max}-byte cap"),
            Self::UrlHasControlCharacters => {
                write!(f, "the URL carries control characters or backslashes")
            }
            Self::UrlUnparseable => write!(f, "the URL does not parse"),
            Self::SchemeNotAllowed(scheme) => {
                write!(f, "the scheme `{scheme}` is not allowed by the R0 policy")
            }
            Self::UserinfoForbidden => write!(f, "the URL carries userinfo (refused)"),
            Self::AmbiguousNumericHost(host) => write!(
                f,
                "the host `{host}` is an alternative (numeric) literal spelling (refused)"
            ),
            Self::PortNotAllowed(port) => {
                write!(f, "the port {port} is not allowed by the R0 policy")
            }
            Self::NoHost => write!(f, "the URL has no host"),
            Self::DnsLookupFailed => write!(f, "the destination name did not resolve"),
            Self::NoAddresses => write!(f, "the destination name resolved to no addresses"),
            Self::DestinationRefused { host, reason } => {
                write!(f, "the destination {host} was refused: {reason}")
            }
            Self::HopLimitExceeded(limit) => {
                write!(f, "the redirect chain exceeded the {limit}-hop cap")
            }
            Self::MissingRedirectLocation => {
                write!(f, "a redirect status carried no Location header")
            }
            Self::UnexpectedStatus(status) => {
                write!(f, "the response status {status} is not a success")
            }
            Self::MediaTypeRefused(media) => {
                write!(f, "the media type `{media}` is outside R0's text/HTML")
            }
            Self::EmptyBody => write!(f, "the 2xx response body was empty"),
            Self::ByteCeilingExceeded(max) => {
                write!(f, "the body exceeded the {max}-byte ceiling")
            }
            Self::DecompressionRatioExceeded { declared, decoded } => write!(
                f,
                "the body decoded to {decoded} bytes from a declared {declared} — over the ratio limit"
            ),
            Self::ReadFailed => write!(f, "the body stream failed"),
            Self::ConnectFailed => write!(f, "the connection (dial/TLS) failed"),
            Self::TimedOut => write!(f, "the acquisition exceeded the time ceiling"),
            Self::ClientBuildFailed => write!(f, "the HTTP client failed to build"),
        }
    }
}

impl FetchError {
    /// The variant's stable name — the resolution result's `kind`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UrlTooLong(_) => "url_too_long",
            Self::UrlHasControlCharacters => "url_control_characters",
            Self::UrlUnparseable => "url_unparseable",
            Self::SchemeNotAllowed(_) => "scheme_not_allowed",
            Self::UserinfoForbidden => "userinfo_forbidden",
            Self::AmbiguousNumericHost(_) => "ambiguous_numeric_host",
            Self::PortNotAllowed(_) => "port_not_allowed",
            Self::NoHost => "no_host",
            Self::DnsLookupFailed => "dns_lookup_failed",
            Self::NoAddresses => "no_addresses",
            Self::DestinationRefused { .. } => "destination_refused",
            Self::HopLimitExceeded(_) => "hop_limit_exceeded",
            Self::MissingRedirectLocation => "missing_redirect_location",
            Self::UnexpectedStatus(_) => "unexpected_status",
            Self::MediaTypeRefused(_) => "media_type_refused",
            Self::EmptyBody => "empty_body",
            Self::ByteCeilingExceeded(_) => "byte_ceiling_exceeded",
            Self::DecompressionRatioExceeded { .. } => "decompression_ratio_exceeded",
            Self::ReadFailed => "read_failed",
            Self::ConnectFailed => "connect_failed",
            Self::TimedOut => "timed_out",
            Self::ClientBuildFailed => "client_build_failed",
        }
    }
}

impl std::error::Error for FetchError {}

/// What the response-type sniff settled on.
///
/// R0's own acquisitions are text/HTML (§12.3) and those two are the only kinds
/// it produces. `DeclaredType` is the third: a response admitted because the
/// RANKED resolver advertises its declared content type
/// (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`). The sniff makes
/// no claim about such a body beyond "its declared type is one this pack asked
/// for" — the type itself is in `content_type`, which is `Some` by construction
/// whenever this variant is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SniffedKind {
    Html,
    Text,
    DeclaredType,
}

impl SniffedKind {
    /// The media-type label a caller falls back to when the response declared
    /// none. `DeclaredType` is only produced FROM a declared type, so its label
    /// is the unclassified-bytes type and the fallback does not fire for it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Html => "text/html",
            Self::Text => "text/plain",
            Self::DeclaredType => "application/octet-stream",
        }
    }
}

impl fmt::Display for SniffedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The acquired document — the `.2.3` snapshot receipt's input shape (minus
/// the digest, which `.2.3` computes over `bytes` per ADR-011).
#[derive(Debug, PartialEq)]
pub struct FetchedDocument {
    pub final_url: Url,
    /// Every hop visited, first to last (the final URL included).
    pub chain: Vec<Url>,
    pub status: u16,
    pub content_type: Option<String>,
    pub sniffed: SniffedKind,
    pub bytes: Vec<u8>,
}

/// The ADR-011 acquisition receipt — the `.6` snapshot lane's input shape
/// (PHASE-4.2.3). The digest is `sha256:<hex>` over the ACQUIRED bytes
/// (never a server-claimed hash); the chain records the requested locator
/// verbatim plus every hop the fetcher visited.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AcquisitionReceipt {
    pub digest: String,
    pub byte_count: usize,
    pub content_type: Option<String>,
    pub sniffed: SniffedKind,
    pub final_url: String,
    pub chain: Vec<String>,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
}

impl AcquisitionReceipt {
    /// Build the receipt from an acquired document: the digest over the
    /// acquired bytes, the byte count, the type, and the hop chain (the raw
    /// requested locator first, then every hop first-to-last).
    pub fn from_document(
        original_locator: &str,
        document: &FetchedDocument,
        acquired_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        let mut chain = vec![original_locator.to_owned()];
        chain.extend(document.chain.iter().map(|url| url.to_string()));
        Self {
            digest: digest_sha256_hex(&document.bytes),
            byte_count: document.bytes.len(),
            content_type: document.content_type.clone(),
            sniffed: document.sniffed,
            final_url: document.final_url.to_string(),
            chain,
            acquired_at,
        }
    }
}

/// The ADR-011 digest over the acquired bytes: `sha256:<64 hex>`.
pub fn digest_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// The DNS seam. The production resolver is the system one; the tests inject
/// a static map so the wire tests never dial real DNS.
pub trait DestinationResolver: Send + Sync + 'static {
    fn resolve(
        &self,
        host: &str,
        port: u16,
    ) -> Pin<Box<dyn Future<Output = io::Result<Vec<SocketAddr>>> + Send>>;
}

/// The system resolver (tokio's getaddrinfo).
#[derive(Debug, Default)]
pub struct SystemResolver;

impl DestinationResolver for SystemResolver {
    fn resolve(
        &self,
        host: &str,
        port: u16,
    ) -> Pin<Box<dyn Future<Output = io::Result<Vec<SocketAddr>>> + Send>> {
        let host = host.to_owned();
        Box::pin(async move {
            tokio::net::lookup_host((host, port))
                .await
                .map(|it| it.collect())
        })
    }
}

/// The classified belt: reqwest's resolver hook re-resolves through the inner
/// resolver and keeps ONLY the addresses the destination policy allows. A
/// connection therefore can never dial a refused address, even when the DNS
/// answer changed between the pre-flight and the dial.
pub(crate) struct ClassifiedDns {
    resolver: Arc<dyn DestinationResolver>,
    policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
}

impl ClassifiedDns {
    pub(crate) fn new(
        resolver: Arc<dyn DestinationResolver>,
        policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
    ) -> Self {
        Self { resolver, policy }
    }
}

impl Resolve for ClassifiedDns {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = Arc::clone(&self.resolver);
        let policy = Arc::clone(&self.policy);
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let mut addrs = resolver
                .resolve(&host, 0)
                .await
                .map_err(|err| -> Box<dyn std::error::Error + Send + Sync> { Box::new(err) })?;
            addrs.truncate(16);
            let resolved = addrs.len();
            // The first refusal is kept so the belt can SAY what it refused.
            // Retaining alone failed closed but reported nothing: a host whose
            // every address is private used to surface as a bare connect
            // failure, indistinguishable from an origin being down
            // (`SIGNOFF-REPAIR.7.2.2`).
            let mut refusal = None;
            addrs.retain(|addr| match policy(&addr.ip()) {
                SsrfVerdict::Allowed => true,
                SsrfVerdict::Refused { reason } => {
                    refusal.get_or_insert_with(|| format!("{}: {reason}", addr.ip()));
                    false
                }
            });
            if addrs.is_empty() && resolved > 0 {
                let detail = refusal.unwrap_or_else(|| "every address was refused".to_owned());
                return Err::<Addrs, Box<dyn std::error::Error + Send + Sync>>(
                    format!("the destination policy refused every address of `{host}` ({detail})")
                        .into(),
                );
            }
            let addrs: Addrs = Box::new(addrs.into_iter());
            Ok::<Addrs, Box<dyn std::error::Error + Send + Sync>>(addrs)
        })
    }
}

/// The fetcher's configuration. `Default` is the production R0 shape:
/// https-only, port 443, the system resolver, the `.2.1` public-only policy.
/// The tests narrow or widen pieces deliberately — never accidentally.
pub struct FetcherConfig {
    pub limits: FetchLimits,
    pub schemes: Vec<&'static str>,
    pub ports: Vec<u16>,
    pub resolver: Arc<dyn DestinationResolver>,
    pub policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            limits: FetchLimits::default(),
            schemes: vec!["https"],
            ports: vec![443],
            resolver: Arc::new(SystemResolver),
            policy: Arc::new(|ip| ssrf::evaluate(*ip)),
        }
    }
}

/// The R0 fetcher.
pub struct Fetcher {
    limits: FetchLimits,
    schemes: Vec<&'static str>,
    ports: Vec<u16>,
    client: Client,
    resolver: Arc<dyn DestinationResolver>,
    policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
}

impl Fetcher {
    /// The production fetcher: the default ceilings, https-only, the system
    /// roots, the system resolver, and the `.2.1` public-only policy.
    pub fn new(limits: FetchLimits) -> Result<Self, FetchError> {
        Self::from_config(FetcherConfig {
            limits,
            ..FetcherConfig::default()
        })
    }

    pub fn from_config(config: FetcherConfig) -> Result<Self, FetchError> {
        let belt = ClassifiedDns {
            resolver: Arc::clone(&config.resolver),
            policy: Arc::clone(&config.policy),
        };
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none()) // the manual per-hop policy owns redirects
            .connect_timeout(config.limits.connect_timeout)
            .no_proxy() // the proxy stays in the threat model (§12.4): no ambient proxy env
            .dns_resolver(Arc::new(belt))
            .user_agent("reasonbraid-r0-fetcher/0.1")
            .build()
            .map_err(|_| FetchError::ClientBuildFailed)?;
        Ok(Self {
            limits: config.limits,
            schemes: config.schemes,
            ports: config.ports,
            client,
            resolver: config.resolver,
            policy: config.policy,
        })
    }

    /// GET a document. Every refusal is a typed [`FetchError`]; the time
    /// ceiling wraps the WHOLE acquisition (every hop + the body read).
    ///
    /// The accept set is R0's own: `text/html`, `application/xhtml+xml`,
    /// `text/*`, or an untyped body that is printable. Use
    /// [`Self::fetch_admitting`] to acquire for a pack that advertises more.
    pub async fn fetch(&self, raw_url: &str) -> Result<FetchedDocument, FetchError> {
        self.fetch_with(Method::GET, raw_url, None, &[]).await
    }

    /// GET a document for a RANKED capability pack, admitting the media types
    /// that pack advertises in addition to R0's own accept set
    /// (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`).
    ///
    /// This widens exactly one gate: which DECLARED content type is accepted.
    /// The destination policy, the scheme list, the byte ceiling, the
    /// decompression-ratio brake, the redirect policy and the time ceiling are
    /// untouched, and an empty `admitted` is identical to [`Self::fetch`].
    pub async fn fetch_admitting(
        &self,
        raw_url: &str,
        admitted: &[String],
    ) -> Result<FetchedDocument, FetchError> {
        self.fetch_with(Method::GET, raw_url, None, admitted).await
    }

    /// GET with a per-request Authorization header (the R5 pack's
    /// delegated session — the credential attaches for THIS acquisition
    /// only, never ambient; the `.5.1` contract's per-request rule).
    pub async fn fetch_authenticated(
        &self,
        raw_url: &str,
        header_value: &str,
    ) -> Result<FetchedDocument, FetchError> {
        self.fetch_with(
            Method::GET,
            raw_url,
            Some(("Authorization", header_value.to_owned())),
            &[],
        )
        .await
    }

    /// The R3 pack's pre-flight: the hardened parse + the destination
    /// classification BEFORE the browser worker is spawned (the worker
    /// receives an already-classified URL).
    pub async fn preflight(&self, raw_url: &str) -> Result<Url, FetchError> {
        let parsed = harden_url(
            raw_url,
            &self.schemes,
            &self.ports,
            self.limits.max_url_length,
        )?;
        self.classify(&parsed).await?;
        Ok(parsed)
    }

    /// HEAD a document (the metadata check — no body is read or sniffed from
    /// bytes; the header-only sniff falls back to `Text` when untyped).
    pub async fn fetch_head(&self, raw_url: &str) -> Result<FetchedDocument, FetchError> {
        self.fetch_with(Method::HEAD, raw_url, None, &[]).await
    }

    async fn fetch_with(
        &self,
        method: Method,
        raw_url: &str,
        extra_header: Option<(&'static str, String)>,
        admitted: &[String],
    ) -> Result<FetchedDocument, FetchError> {
        let run = async {
            let mut current = harden_url(
                raw_url,
                &self.schemes,
                &self.ports,
                self.limits.max_url_length,
            )?;
            let mut chain: Vec<Url> = Vec::new();
            let mut method = method;
            let mut hops: u8 = 0;
            loop {
                // The pre-flight: resolve + classify BEFORE any socket opens.
                self.classify(&current).await?;
                chain.push(current.clone());
                let mut request_builder = self.client.request(method.clone(), current.clone());
                if let Some((name, value)) = &extra_header {
                    request_builder = request_builder.header(*name, value);
                }
                let request = request_builder
                    .build()
                    .map_err(|_| FetchError::UrlUnparseable)?;
                let response = self
                    .client
                    .execute(request)
                    .await
                    .map_err(|_| FetchError::ConnectFailed)?;
                let status = response.status();
                if status.is_redirection() {
                    hops += 1;
                    if hops > self.limits.max_redirects {
                        return Err(FetchError::HopLimitExceeded(self.limits.max_redirects));
                    }
                    let location = response
                        .headers()
                        .get(LOCATION)
                        .and_then(|value| value.to_str().ok())
                        .ok_or(FetchError::MissingRedirectLocation)?
                        .to_owned();
                    let next = current
                        .join(&location)
                        .map_err(|_| FetchError::UrlUnparseable)?;
                    if status == StatusCode::SEE_OTHER {
                        method = Method::GET;
                    }
                    // The redirect hop re-enters the FULL validation: the new
                    // URL is re-hardened and re-classified at the top of the
                    // loop — the escape dies here, not at the socket.
                    current = harden_url(
                        next.as_str(),
                        &self.schemes,
                        &self.ports,
                        self.limits.max_url_length,
                    )?;
                    continue;
                }
                if !status.is_success() {
                    return Err(FetchError::UnexpectedStatus(status.as_u16()));
                }
                let content_type = response
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                let declared = response
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok());
                let (bytes, sniffed) = if method == Method::HEAD {
                    // No body to mislead with: the header decides, and an
                    // untyped HEAD is recorded as text.
                    match sniff_kind(content_type.as_deref(), &[], admitted) {
                        Some(kind) => (Vec::new(), kind),
                        None if content_type.is_none() => (Vec::new(), SniffedKind::Text),
                        None => {
                            return Err(FetchError::MediaTypeRefused(
                                content_type
                                    .as_deref()
                                    .unwrap_or("application/octet-stream")
                                    .to_owned(),
                            ));
                        }
                    }
                } else {
                    // The WIRE bytes are read bounded first, then decoded by
                    // hand — reqwest's auto-decode would hide the encoded
                    // size the ratio brake measures.
                    let encoding = response
                        .headers()
                        .get(CONTENT_ENCODING)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_owned);
                    let wire = read_body(response, self.limits.max_bytes).await?;
                    if wire.is_empty() {
                        return Err(FetchError::EmptyBody);
                    }
                    let bytes = decode_body(wire, encoding.as_deref(), &self.limits)?;
                    if bytes.is_empty() {
                        return Err(FetchError::EmptyBody);
                    }
                    // An identity body's ratio rides the declared length (the
                    // lying-Content-Length belt); an encoded body's ratio was
                    // measured against its own envelope inside decode_body.
                    if encoding.as_deref().is_none_or(is_identity_encoding) {
                        check_ratio(declared, bytes.len(), self.limits.max_decompression_ratio)?;
                    }
                    let sniffed = sniff_kind(content_type.as_deref(), &bytes, admitted)
                        .ok_or_else(|| {
                            FetchError::MediaTypeRefused(
                                content_type
                                    .as_deref()
                                    .unwrap_or("application/octet-stream")
                                    .to_owned(),
                            )
                        })?;
                    (bytes, sniffed)
                };
                return Ok(FetchedDocument {
                    final_url: current,
                    chain,
                    status: status.as_u16(),
                    content_type,
                    sniffed,
                    bytes,
                });
            }
        };
        tokio::time::timeout(self.limits.max_time, run)
            .await
            .map_err(|_| FetchError::TimedOut)?
    }

    /// The pre-flight: resolve the host, then demand EVERY resolved address
    /// passes the destination policy. The first refusal names its class.
    async fn classify(&self, url: &Url) -> Result<(), FetchError> {
        // An IP literal skips DNS entirely (reqwest dials literals directly,
        // so the belt cannot see them — the pre-flight is their only gate).
        // The `Host` match sees the address WITHOUT the IPv6 brackets, which
        // is exactly the form the classifier wants.
        let domain = match url.host() {
            None => return Err(FetchError::NoHost),
            Some(Host::Ipv4(ip)) => return self.allow_ip(IpAddr::V4(ip)),
            Some(Host::Ipv6(ip)) => return self.allow_ip(IpAddr::V6(ip)),
            Some(Host::Domain(domain)) => domain.to_owned(),
        };
        let port = url.port_or_known_default().unwrap_or(443);
        let mut addrs = self
            .resolver
            .resolve(&domain, port)
            .await
            .map_err(|_| FetchError::DnsLookupFailed)?;
        if addrs.is_empty() {
            return Err(FetchError::NoAddresses);
        }
        addrs.truncate(16);
        for addr in addrs {
            self.allow_ip(addr.ip())?;
        }
        Ok(())
    }

    fn allow_ip(&self, ip: IpAddr) -> Result<(), FetchError> {
        match (self.policy)(&ip) {
            SsrfVerdict::Allowed => Ok(()),
            SsrfVerdict::Refused { reason } => Err(FetchError::DestinationRefused {
                host: ip.to_string(),
                reason,
            }),
        }
    }
}

/// The hardened parse (§12.4: one library; reject the ambiguous/userinfo/
/// invalid forms). Pure — unit-tested without a socket.
fn harden_url(
    raw: &str,
    schemes: &[&'static str],
    ports: &[u16],
    max_length: usize,
) -> Result<Url, FetchError> {
    if raw.len() > max_length {
        return Err(FetchError::UrlTooLong(max_length));
    }
    if raw
        .chars()
        .any(|c| c.is_control() || c == '\\' || c.is_whitespace())
    {
        return Err(FetchError::UrlHasControlCharacters);
    }
    let url = Url::parse(raw).map_err(|_| FetchError::UrlUnparseable)?;
    // The ambiguity refusal comes FIRST — the most specific, diagnostic name
    // even for a scheme the policy would refuse anyway.
    if !url.username().is_empty() || url.password().is_some() {
        return Err(FetchError::UserinfoForbidden);
    }
    if !schemes.contains(&url.scheme()) {
        return Err(FetchError::SchemeNotAllowed(url.scheme().to_owned()));
    }
    match url.host() {
        None => return Err(FetchError::NoHost),
        Some(Host::Domain(domain)) if numeric_ambiguous(domain) => {
            return Err(FetchError::AmbiguousNumericHost(domain.to_owned()));
        }
        Some(_) => {}
    }
    if let Some(port) = url.port() {
        if !ports.contains(&port) {
            return Err(FetchError::PortNotAllowed(port));
        }
    }
    Ok(url)
}

/// The alternative-literal refusal (§12.4 "alternative literal"): a "domain"
/// that is really a number — the decimal/octal/hex IPv4 spellings some stacks
/// resolve ("2130706433", "127.1", "0177.0.0.1", "0x7f000001"). Refused before
/// DNS ever sees it, whatever the parser normalized.
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

/// The bounded body read — the byte ceiling applies to the DECODED bytes, so
/// neither a lying `Content-Length` nor a compressed bomb can exceed it.
async fn read_body(response: Response, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| FetchError::ReadFailed)?;
        if bytes.len() + chunk.len() > max_bytes {
            return Err(FetchError::ByteCeilingExceeded(max_bytes));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn is_identity_encoding(value: &str) -> bool {
    value.trim().is_empty() || value.trim().eq_ignore_ascii_case("identity")
}

/// The manual content-encoding decode (§12.4: the decompression-ratio limit).
/// The decoded size is bounded by the byte ceiling AND measured against the
/// ENVELOPE's own length — the zip bomb trips the ratio before the ceiling
/// does. An unsupported or stacked encoding is a named refusal, never a guess.
fn decode_body(
    wire: Vec<u8>,
    encoding: Option<&str>,
    limits: &FetchLimits,
) -> Result<Vec<u8>, FetchError> {
    let encoding = encoding.unwrap_or("identity").to_ascii_lowercase();
    if encoding.contains(',') {
        return Err(FetchError::MediaTypeRefused(format!(
            "content-encoding `{encoding}`"
        )));
    }
    let encoding = encoding.trim();
    if is_identity_encoding(encoding) {
        return Ok(wire);
    }
    match encoding {
        "gzip" | "x-gzip" => inflate(flate2::read::GzDecoder::new(&wire[..]), &wire, limits),
        "deflate" => inflate(flate2::read::DeflateDecoder::new(&wire[..]), &wire, limits),
        "br" => inflate(brotli::Decompressor::new(&wire[..], 4096), &wire, limits),
        other => Err(FetchError::MediaTypeRefused(format!(
            "content-encoding `{other}`"
        ))),
    }
}

fn inflate(
    reader: impl std::io::Read,
    envelope: &[u8],
    limits: &FetchLimits,
) -> Result<Vec<u8>, FetchError> {
    let mut decoded = Vec::new();
    // The take() hard-stops the decode one byte past the ceiling — the ratio
    // check below then sees the truth of the payload.
    let mut reader = reader.take(limits.max_bytes as u64 + 1);
    reader
        .read_to_end(&mut decoded)
        .map_err(|_| FetchError::ReadFailed)?;
    if decoded.len() > limits.max_bytes {
        return Err(FetchError::ByteCeilingExceeded(limits.max_bytes));
    }
    check_ratio(
        Some(envelope.len() as u64),
        decoded.len(),
        limits.max_decompression_ratio,
    )?;
    Ok(decoded)
}

/// The decompression-ratio brake, pure: decoded/declared must stay inside the
/// limit. A chunked body (no declared length) is bounded by the byte ceiling
/// alone. A declared length of zero is a protocol lie — the byte ceiling
/// still bounds it, so nothing extra is read.
fn check_ratio(declared: Option<u64>, decoded: usize, limit: f64) -> Result<(), FetchError> {
    if let Some(declared) = declared {
        if declared > 0 {
            let ratio = decoded as f64 / declared as f64;
            if ratio > limit {
                return Err(FetchError::DecompressionRatioExceeded { declared, decoded });
            }
        }
    }
    Ok(())
}

/// The response-type sniff: the header decides first; without a usable header,
/// the first bytes decide — HTML tags, then a JSON-shaped refusal, then
/// UTF-8 text; everything else is refused by the caller.
/// `admitted` is the RANKED resolver's own advertised media types, supplied by
/// the caller for this acquisition only. It widens nothing by itself: an empty
/// slice — what `fetch` passes — leaves R0's text/HTML accept set exactly as
/// shipped (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`).
fn sniff_kind(content_type: Option<&str>, head: &[u8], admitted: &[String]) -> Option<SniffedKind> {
    let primary = content_type.map(|ct| {
        ct.split(';')
            .next()
            .unwrap_or(ct)
            .trim()
            .to_ascii_lowercase()
    });
    match primary.as_deref() {
        Some("text/html") | Some("application/xhtml+xml") => return Some(SniffedKind::Html),
        Some(media) if media.starts_with("text/") => return Some(SniffedKind::Text),
        // The pack that was ranked asked for this exact type. The comparison is
        // over the already-lowercased primary type, with the advertisement
        // lowercased too, so a registry row's casing cannot decide a gate.
        Some(media)
            if admitted
                .iter()
                .any(|kind| kind.trim().eq_ignore_ascii_case(media)) =>
        {
            return Some(SniffedKind::DeclaredType);
        }
        Some(_) => return None, // any other declared type: refused by the caller
        None => {}
    }
    let rest = skip_whitespace_and_bom(head);
    if rest.is_empty() {
        return None;
    }
    if prefix_ignoring_case(rest, b"<!doctype html") || prefix_ignoring_case(rest, b"<html") {
        return Some(SniffedKind::Html);
    }
    match rest.first() {
        Some(b'{') | Some(b'[') => return None, // JSON-shaped — outside R0's text/HTML
        _ => {}
    }
    if std::str::from_utf8(rest).is_ok()
        && rest
            .iter()
            .all(|b| !b.is_ascii_control() || *b == b'\t' || *b == b'\n' || *b == b'\r')
    {
        return Some(SniffedKind::Text);
    }
    None
}

fn skip_whitespace_and_bom(mut bytes: &[u8]) -> &[u8] {
    loop {
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            bytes = &bytes[3..];
            continue;
        }
        match bytes.first() {
            Some(byte) if byte.is_ascii_whitespace() => bytes = &bytes[1..],
            _ => break,
        }
    }
    bytes
}

fn prefix_ignoring_case(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes.len() >= prefix.len() && bytes[..prefix.len()].eq_ignore_ascii_case(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::{CONTENT_TYPE as AXUM_CONTENT_TYPE, LOCATION as AXUM_LOCATION};
    use axum::http::{Response as AxumResponse, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn https_only() -> (Vec<&'static str>, Vec<u16>) {
        (vec!["https"], vec![443])
    }

    // ---- pure: the hardened URL parse -----------------------------------

    #[test]
    fn harden_url_refuses_the_ambiguous_and_invalid_forms() {
        let (schemes, ports) = https_only();
        for (raw, expected) in [
            ("http://user@example.com/", FetchError::UserinfoForbidden),
            (
                "http://user:pass@example.com/",
                FetchError::UserinfoForbidden,
            ),
            (
                "http://example.com:8080@evil.example/",
                FetchError::UserinfoForbidden,
            ),
            (
                "ftp://example.com/",
                FetchError::SchemeNotAllowed("ftp".into()),
            ),
            (
                "http://example.com/",
                FetchError::SchemeNotAllowed("http".into()),
            ),
            (
                "https://example.com:8443/",
                FetchError::PortNotAllowed(8443),
            ),
            (
                "https://example.com/\n",
                FetchError::UrlHasControlCharacters,
            ),
            (
                "https://example.com\\@127.0.0.1/",
                FetchError::UrlHasControlCharacters,
            ),
            (
                "https://example.com/ path",
                FetchError::UrlHasControlCharacters,
            ),
            ("https://%E0%A4%A/", FetchError::UrlUnparseable),
            ("https://example.com:99999/", FetchError::UrlUnparseable),
            ("not a url", FetchError::UrlHasControlCharacters),
        ] {
            assert_eq!(
                harden_url(raw, &schemes, &ports, 2048),
                Err(expected.clone()),
                "the URL `{raw}` must refuse with the named reason"
            );
        }
        // The alternative-literal spellings: the WHATWG parser may normalize
        // some of them into IPv4 literals BEFORE our check sees them — every
        // such form must still be REFUSED, either by the ambiguity check or,
        // when normalized, by the classification (the normalized literal is
        // a loopback address, so the production policy refuses it).
        for raw in [
            "https://2130706433/",
            "https://127.1/",
            "https://0177.0.0.1/",
            "https://0x7f000001/",
        ] {
            match harden_url(raw, &schemes, &ports, 2048) {
                Err(FetchError::AmbiguousNumericHost(_)) => {}
                Ok(url) => match url.host() {
                    Some(Host::Ipv4(ip)) if ip.is_loopback() => {}
                    other => {
                        panic!("`{raw}` must refuse or normalize to a loopback literal: {other:?}")
                    }
                },
                other => panic!("`{raw}` must refuse with the ambiguity name: {other:?}"),
            }
        }
        // The length cap.
        assert_eq!(
            harden_url("https://example.com/", &schemes, &ports, 10),
            Err(FetchError::UrlTooLong(10))
        );
        // Real domains with hex-looking labels are NOT numeric spellings.
        assert!(harden_url("https://0day.example/", &schemes, &ports, 2048).is_ok());
        assert!(harden_url("https://cafe.example/", &schemes, &ports, 2048).is_ok());
        // The url crate's parser may normalize some numeric spellings into
        // IPv4 literals before our check sees them — every such form must
        // still be REFUSED here (as an IP literal it dies at the
        // classification, which is the same guarantee with a different name).
        if let Ok(url) = harden_url("https://127.0.0.1/", &schemes, &ports, 2048) {
            assert!(matches!(url.host(), Some(Host::Ipv4(_))));
        }
    }

    // ---- pure: the response-type sniff ----------------------------------

    #[test]
    fn sniff_kind_decides_header_first_then_magic() {
        for (content_type, head, expected) in [
            (Some("text/html"), &b"<html>"[..], Some(SniffedKind::Html)),
            (
                Some("text/html; charset=utf-8"),
                &b"x"[..],
                Some(SniffedKind::Html),
            ),
            (
                Some("application/xhtml+xml"),
                &b"<?xml"[..],
                Some(SniffedKind::Html),
            ),
            (Some("TEXT/PLAIN"), &b"hello"[..], Some(SniffedKind::Text)),
            (Some("text/markdown"), &b"# hi"[..], Some(SniffedKind::Text)),
            (Some("application/json"), &b"{}"[..], None),
            (Some("application/pdf"), &b"%PDF"[..], None),
            (
                None,
                &b"<!doctype html><title>x"[..],
                Some(SniffedKind::Html),
            ),
            (None, &b"  \n<HTML lang=en>"[..], Some(SniffedKind::Html)),
            (None, &b"\xEF\xBB\xBF<html>"[..], Some(SniffedKind::Html)),
            (None, &b"{\"a\":1}"[..], None), // JSON-shaped, no header
            (None, &b"plain words here"[..], Some(SniffedKind::Text)),
            (None, &b"\x00\x01\x02binary"[..], None),
            (None, &b""[..], None),
        ] {
            assert_eq!(
                sniff_kind(content_type, head, &[]),
                expected,
                "content-type {content_type:?} with head {:?}",
                String::from_utf8_lossy(head)
            );
        }
    }

    // ---- the census: this leg's accept set vs the R2 advertisement -------
    //
    // `SIGNOFF-REPAIR.7.3.3.5.1`. The R2 arm of the resolution path acquires
    // through THIS fetcher, and the registry row it is ranked on advertises a
    // format set. These two controls measure the relationship between them.
    // They are a census, not a regression: they PASS on unchanged production,
    // and their job is to make the mismatch a fact rather than a reading.

    /// migration `0027_r2_extract_worker_entry.sql`'s `media_types`, verbatim.
    /// Stated here so the census enumerates the advertisement rather than
    /// whatever the author remembered of it.
    const R2_ADVERTISED_MEDIA_TYPES: [&str; 5] = [
        "application/pdf",
        "application/zip",
        "application/x-tar",
        "application/atom+xml",
        "application/rss+xml",
    ];

    /// R0's own accept set refuses every type the R2 pack advertises — and
    /// admits each of them when that pack is the one RANKED. Both halves, in one
    /// control, because the repair is exactly the difference between them
    /// (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`).
    #[test]
    fn every_r2_advertised_type_is_refused_by_r0_and_admitted_for_its_own_pack() {
        let body = br#"<?xml version="1.0" encoding="utf-8"?><feed><title>x</title></feed>"#;
        let advertised: Vec<String> = R2_ADVERTISED_MEDIA_TYPES
            .iter()
            .map(|kind| (*kind).to_owned())
            .collect();
        for media in R2_ADVERTISED_MEDIA_TYPES {
            assert_eq!(
                sniff_kind(Some(media), body, &[]),
                None,
                "`{media}` is outside R0's own accept set, whatever the body is",
            );
            assert_eq!(
                sniff_kind(Some(media), body, &advertised),
                Some(SniffedKind::DeclaredType),
                "`{media}` is admitted for the pack that advertises it",
            );
            // The parameter is the header's full value: a charset must not
            // decide a gate, and neither must casing.
            assert_eq!(
                sniff_kind(Some(&format!("{media}; charset=utf-8")), body, &advertised),
                Some(SniffedKind::DeclaredType),
                "`{media}` with parameters is the same declared type",
            );
        }
        // A type the ranked pack does NOT advertise stays refused. This is the
        // half that keeps the widening bounded: the admitted set is the row's,
        // not "anything declared".
        for outsider in ["application/json", "application/octet-stream", "image/png"] {
            assert_eq!(
                sniff_kind(Some(outsider), body, &advertised),
                None,
                "`{outsider}` is outside the advertisement and stays refused",
            );
        }
        // An empty admitted set is byte-for-byte R0's shipped behaviour.
        assert_eq!(sniff_kind(Some("application/pdf"), body, &[]), None);
        // The other direction: the COMPLETE declared-type accept set. Adding a
        // sixth arm to `sniff_kind` without revisiting this census fails here.
        for (media, expected) in [
            ("text/html", Some(SniffedKind::Html)),
            ("application/xhtml+xml", Some(SniffedKind::Html)),
            ("text/xml", Some(SniffedKind::Text)),
            ("text/plain", Some(SniffedKind::Text)),
            ("text/anything-at-all", Some(SniffedKind::Text)),
        ] {
            assert_eq!(
                sniff_kind(Some(media), body, &[]),
                expected,
                "the declared-type accept set is `text/html`, \
                 `application/xhtml+xml` and `text/*` — `{media}`",
            );
        }
    }

    /// Untyped, the verdict is a property of the BYTES, not of the format.
    ///
    /// This is the part that disciplines the finding's shape: "the five
    /// advertised formats are unacquirable" is the wrong claim, because an
    /// untyped body is judged by its content. A document whose bytes happen to
    /// be text passes whatever format it belongs to; the same document with one
    /// NUL does not. So the leg cannot honour a FORMAT-shaped advertisement at
    /// all — its rule is not about formats.
    #[test]
    fn an_untyped_body_is_judged_by_its_bytes_not_by_its_format() {
        // A PDF body that happens to be entirely printable is ACCEPTED untyped.
        let textual_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n";
        assert_eq!(
            sniff_kind(None, textual_pdf, &[]),
            Some(SniffedKind::Text),
            "an all-printable body is accepted untyped, whatever format it is",
        );
        // The same document with one NUL — which any real PDF carries in its
        // object streams and its binary-marker comment — is REFUSED.
        let mut binary_pdf = textual_pdf.to_vec();
        binary_pdf.extend_from_slice(b"stream\n\x00\x01\x02\nendstream\n");
        assert_eq!(
            sniff_kind(None, &binary_pdf, &[]),
            None,
            "one non-text byte refuses the same document",
        );

        // ZIP and tar cannot reach that branch at all, and the reason is
        // structural rather than incidental: both formats begin with
        // fixed-width binary fields. A ZIP local file header is
        // `PK\x03\x04` followed by nine little-endian integer fields, and a
        // tar header is a 512-byte block whose 100-byte name field is
        // NUL-padded. Neither can be all-printable.
        let mut zip_local_header = b"PK\x03\x04".to_vec();
        zip_local_header.extend_from_slice(&[0x14, 0x00]); // version needed
        zip_local_header.extend_from_slice(&[0x00, 0x00]); // flags
        zip_local_header.extend_from_slice(&[0x00, 0x00]); // method: stored
        zip_local_header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // time+date
        zip_local_header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // crc32
        zip_local_header.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // compressed
        zip_local_header.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // uncompressed
        zip_local_header.extend_from_slice(&[0x01, 0x00]); // name length
        zip_local_header.extend_from_slice(&[0x00, 0x00]); // extra length
        zip_local_header.extend_from_slice(b"ax");
        assert_eq!(
            sniff_kind(None, &zip_local_header, &[]),
            None,
            "a ZIP's local file header carries binary fields by construction",
        );

        let mut tar_header = vec![0u8; 512];
        tar_header[..2].copy_from_slice(b"ax"); // the NUL-padded 100-byte name
        tar_header[257..262].copy_from_slice(b"ustar");
        assert_eq!(
            sniff_kind(None, &tar_header, &[]),
            None,
            "a tar header block is NUL-padded fixed-width fields by construction",
        );
    }

    // ---- pure: the decompression-ratio brake ----------------------------

    #[test]
    fn check_ratio_refuses_the_lying_length_and_allows_the_bounded() {
        assert!(check_ratio(None, 1_000_000, 10.0).is_ok()); // chunked: byte ceiling owns it
        assert!(check_ratio(Some(1000), 5000, 10.0).is_ok()); // ratio 5
        assert_eq!(
            check_ratio(Some(10), 1000, 10.0),
            Err(FetchError::DecompressionRatioExceeded {
                declared: 10,
                decoded: 1000
            })
        );
        assert!(check_ratio(Some(0), 1000, 10.0).is_ok()); // the lie is bounded by the ceiling
    }

    // ---- pure: the ADR-011 receipt ---------------------------------------

    #[test]
    fn the_receipt_carries_the_digest_and_the_chain() {
        let document = FetchedDocument {
            final_url: Url::parse("https://example.org/final").unwrap(),
            chain: vec![
                Url::parse("https://example.org/start").unwrap(),
                Url::parse("https://example.org/final").unwrap(),
            ],
            status: 200,
            content_type: Some("text/html".to_owned()),
            sniffed: SniffedKind::Html,
            bytes: b"<!doctype html><title>hi</title>".to_vec(),
        };
        let acquired_at = chrono::DateTime::parse_from_rfc3339("2026-09-07T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let receipt =
            AcquisitionReceipt::from_document("https://example.org/start", &document, acquired_at);
        // The digest is the ADR-011 shape over the ACQUIRED bytes.
        assert_eq!(
            receipt.digest,
            digest_sha256_hex(b"<!doctype html><title>hi</title>")
        );
        assert!(receipt.digest.starts_with("sha256:"));
        assert_eq!(receipt.digest.len(), 7 + 64, "sha256: + 64 hex");
        // The chain: the raw requested locator first, then every hop.
        assert_eq!(
            receipt.chain,
            vec![
                "https://example.org/start".to_owned(),
                "https://example.org/start".to_owned(),
                "https://example.org/final".to_owned(),
            ]
        );
        assert_eq!(receipt.byte_count, 32);
        assert_eq!(receipt.content_type.as_deref(), Some("text/html"));
        assert_eq!(receipt.sniffed, SniffedKind::Html);
        assert_eq!(receipt.final_url, "https://example.org/final");
        assert_eq!(receipt.acquired_at, acquired_at);
        // The digest the receipt carries passes the §12.1 validator's shape
        // (the resources layer accepts the same scheme the receipt produces).
        let reference = crate::resources::ResourceReference {
            original_locator: "https://example.org/start".to_owned(),
            scheme: "https".to_owned(),
            media_type_hint: None,
            expected_digest: Some(receipt.digest.clone()),
            fragment_or_selector: None,
            credential_binding_ref: None,
            owning_node_or_capability: None,
            visibility_scope: "tenant".to_owned(),
            purpose: None,
            retention_class: None,
            risk_class: "standard".to_owned(),
        };
        assert!(
            reference.digest_error().is_none(),
            "the receipt digest is a valid §12.1 digest"
        );
    }

    // ---- wire: the measured refusals (all offline) ----------------------

    /// The test resolver pins every configured domain to 127.0.0.1 — the wire
    /// tests never dial real DNS.
    #[derive(Clone)]
    struct StaticResolver {
        hosts: HashMap<String, IpAddr>,
    }

    impl DestinationResolver for StaticResolver {
        fn resolve(
            &self,
            host: &str,
            port: u16,
        ) -> Pin<Box<dyn Future<Output = io::Result<Vec<SocketAddr>>> + Send>> {
            let found = self
                .hosts
                .get(host)
                .map(|ip| vec![SocketAddr::new(*ip, port)]);
            Box::pin(async move {
                found.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unknown test host"))
            })
        }
    }

    /// The allow-loopback policy: the local test origin passes, everything
    /// else keeps the `.2.1` production verdicts.
    fn allow_loopback() -> Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync> {
        Arc::new(|ip| {
            if ip.is_loopback() {
                SsrfVerdict::Allowed
            } else {
                ssrf::evaluate(*ip)
            }
        })
    }

    fn production_policy() -> Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync> {
        Arc::new(|ip| ssrf::evaluate(*ip))
    }

    /// A tiny local origin: the routes the refusals and the ceilings target.
    /// Returns the bound port + the request counter (the refusal proof reads
    /// the counter: a refused destination must leave it at zero).
    async fn spawn_origin() -> (u16, Arc<AtomicUsize>) {
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        let app = Router::new()
            .route(
                "/ok",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async {
                            (
                                [(AXUM_CONTENT_TYPE, "text/html")],
                                "<!doctype html><html><head><title>hi</title></head></html>",
                            )
                        }
                    }
                }),
            )
            .route(
                "/text",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async { ([(AXUM_CONTENT_TYPE, "text/plain")], "a plain page") }
                    }
                }),
            )
            .route(
                "/json",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async { ([(AXUM_CONTENT_TYPE, "application/json")], "{\"a\":1}") }
                    }
                }),
            )
            .route(
                "/redir-private",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async {
                            (
                                StatusCode::FOUND,
                                [(AXUM_LOCATION, "http://10.0.0.1/escape")],
                                "moved",
                            )
                        }
                    }
                }),
            )
            .route(
                "/redir-loop",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async { (StatusCode::FOUND, [(AXUM_LOCATION, "/redir-loop")], "again") }
                    }
                }),
            )
            .route(
                "/big",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async { ([(AXUM_CONTENT_TYPE, "text/plain")], "x".repeat(4096)) }
                    }
                }),
            )
            .route(
                "/gz-ok",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async {
                            // A small, legitimate gzip page: the ratio brake
                            // must let it through.
                            let mut encoder = flate2::write::GzEncoder::new(
                                Vec::new(),
                                flate2::Compression::default(),
                            );
                            use std::io::Write as _;
                            encoder
                                .write_all(b"a gzip page")
                                .expect("the gzip page compresses");
                            let compressed = encoder.finish().expect("the gzip page finishes");
                            AxumResponse::builder()
                                .header(AXUM_CONTENT_TYPE, "text/plain")
                                .header(axum::http::header::CONTENT_ENCODING, "gzip")
                                .body(Body::from(compressed))
                                .expect("the gzip page builds")
                        }
                    }
                }),
            )
            .route(
                "/bomb",
                get({
                    let counter = Arc::clone(&counter);
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                        async {
                            // A REAL compressed bomb: 1000 decoded bytes in a
                            // ~35-byte gzip envelope. The ratio brake sees
                            // decoded/declared ≈ 28 — over the 10.0 limit.
                            let mut encoder = flate2::write::GzEncoder::new(
                                Vec::new(),
                                flate2::Compression::default(),
                            );
                            use std::io::Write as _;
                            encoder
                                .write_all(&vec![b'z'; 1000])
                                .expect("the bomb compresses");
                            let compressed = encoder.finish().expect("the bomb finishes");
                            AxumResponse::builder()
                                .header(AXUM_CONTENT_TYPE, "text/plain")
                                .header(axum::http::header::CONTENT_ENCODING, "gzip")
                                .body(Body::from(compressed))
                                .expect("the bomb response builds")
                        }
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("the test origin binds");
        let port = listener.local_addr().expect("the port is known").port();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("the test origin serves");
        });
        (port, hits)
    }

    fn test_fetcher(
        port: u16,
        policy: Arc<dyn Fn(&IpAddr) -> SsrfVerdict + Send + Sync>,
        limits: FetchLimits,
    ) -> Fetcher {
        let resolver = Arc::new(StaticResolver {
            hosts: [("fetch.test".to_owned(), IpAddr::from([127, 0, 0, 1]))].into(),
        });
        Fetcher::from_config(FetcherConfig {
            limits,
            schemes: vec!["http"],
            ports: vec![port],
            resolver,
            policy,
        })
        .expect("the test fetcher builds")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_fetcher_returns_the_html_document_with_its_chain() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        let document = fetcher
            .fetch(&format!("http://fetch.test:{port}/ok"))
            .await
            .expect("the html page fetches");
        assert_eq!(document.sniffed, SniffedKind::Html);
        assert_eq!(document.sniffed.as_str(), "text/html");
        assert_eq!(document.content_type.as_deref(), Some("text/html"));
        assert_eq!(document.status, 200);
        assert!(String::from_utf8_lossy(&document.bytes).contains("<title>hi"));
        assert_eq!(document.chain.len(), 1, "one hop: the origin itself");
        assert_eq!(document.chain[0].host_str(), Some("fetch.test"));
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_fetcher_returns_text_through_the_magic_sniff() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        let document = fetcher
            .fetch(&format!("http://127.0.0.1:{port}/text"))
            .await
            .expect("the literal text page fetches");
        assert_eq!(document.sniffed, SniffedKind::Text);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    /// THE SSRF PROOF: the loopback literal is refused by the production
    /// policy BEFORE any request reaches the origin.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_fetcher_refuses_the_loopback_literal_before_any_request() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, production_policy(), FetchLimits::default());
        match fetcher.fetch(&format!("http://127.0.0.1:{port}/ok")).await {
            Err(FetchError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("loopback"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the loopback literal must refuse: {other:?}"),
        }
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "no request may reach the origin"
        );
    }

    /// The mapped-form loopback (the `.2.1` re-classification) flows through
    /// the fetcher: refused with the embedded IPv4's class.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_mapped_loopback_literal_refuses_with_its_class() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, production_policy(), FetchLimits::default());
        match fetcher
            .fetch(&format!("http://[::ffff:127.0.0.1]:{port}/ok"))
            .await
        {
            Err(FetchError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("loopback"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the mapped loopback must refuse: {other:?}"),
        }
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    /// The redirect-chain escape: the first hop is allowed, the second hop's
    /// private destination dies at the re-classification — never dialed.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_fetcher_refuses_the_private_redirect_hop() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        match fetcher
            .fetch(&format!("http://fetch.test:{port}/redir-private"))
            .await
        {
            Err(FetchError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("private"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the private hop must refuse: {other:?}"),
        }
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "only the first hop was dialed — the escape never connected"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_redirect_hop_cap_names_itself() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(
            port,
            allow_loopback(),
            FetchLimits {
                max_redirects: 3,
                ..FetchLimits::default()
            },
        );
        assert_eq!(
            fetcher
                .fetch(&format!("http://fetch.test:{port}/redir-loop"))
                .await,
            Err(FetchError::HopLimitExceeded(3))
        );
        assert_eq!(
            hits.load(Ordering::SeqCst),
            4,
            "the origin + three followed hops"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_gzip_page_decodes_within_the_ratio() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        let document = fetcher
            .fetch(&format!("http://127.0.0.1:{port}/gz-ok"))
            .await
            .expect("the small gzip page fetches");
        assert_eq!(document.sniffed, SniffedKind::Text);
        assert_eq!(document.bytes, b"a gzip page");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_byte_ceiling_and_the_ratio_brake_refuse() {
        let (port, _hits) = spawn_origin().await;
        let fetcher = test_fetcher(
            port,
            allow_loopback(),
            FetchLimits {
                max_bytes: 64,
                ..FetchLimits::default()
            },
        );
        assert_eq!(
            fetcher.fetch(&format!("http://127.0.0.1:{port}/big")).await,
            Err(FetchError::ByteCeilingExceeded(64))
        );
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        match fetcher
            .fetch(&format!("http://127.0.0.1:{port}/bomb"))
            .await
        {
            Err(FetchError::DecompressionRatioExceeded {
                declared,
                decoded: 1000,
            }) => {
                assert!(
                    declared < 100,
                    "the gzip envelope is tiny next to its payload: {declared}"
                );
            }
            other => panic!("the bomb must trip the ratio brake: {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_media_sniffing_refuses_the_json_body() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        assert_eq!(
            fetcher
                .fetch(&format!("http://127.0.0.1:{port}/json"))
                .await,
            Err(FetchError::MediaTypeRefused("application/json".into()))
        );
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "the body was fetched and sniffed"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_head_verb_returns_the_empty_document() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        let document = fetcher
            .fetch_head(&format!("http://127.0.0.1:{port}/ok"))
            .await
            .expect("the HEAD succeeds");
        assert!(document.bytes.is_empty(), "HEAD reads no body");
        assert_eq!(document.sniffed, SniffedKind::Html, "the header decides");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_userinfo_url_is_refused() {
        let (port, hits) = spawn_origin().await;
        let fetcher = test_fetcher(port, allow_loopback(), FetchLimits::default());
        assert_eq!(
            fetcher
                .fetch(&format!("http://user@fetch.test:{port}/ok"))
                .await,
            Err(FetchError::UserinfoForbidden)
        );
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_dns_failure_names_itself() {
        let (port, hits) = spawn_origin().await;
        let resolver = Arc::new(StaticResolver {
            hosts: HashMap::new(), // nothing resolves
        });
        let fetcher = Fetcher::from_config(FetcherConfig {
            limits: FetchLimits::default(),
            schemes: vec!["http"],
            ports: vec![port],
            resolver,
            policy: allow_loopback(),
        })
        .expect("the fetcher builds");
        assert_eq!(
            fetcher
                .fetch(&format!("http://unknown.test:{port}/ok"))
                .await,
            Err(FetchError::DnsLookupFailed)
        );
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    /// The production shape refuses before any network exists: https-only and
    /// public-only are the default config's own answers.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_default_fetcher_is_https_only_and_public_only() {
        let fetcher = Fetcher::new(FetchLimits::default()).expect("the fetcher builds");
        assert_eq!(
            fetcher.fetch("http://example.com/").await,
            Err(FetchError::SchemeNotAllowed("http".into()))
        );
        match fetcher.fetch("https://127.0.0.1/").await {
            Err(FetchError::DestinationRefused { reason, .. }) => {
                assert!(
                    reason.contains("loopback"),
                    "the reason names the class: {reason}"
                );
            }
            other => panic!("the loopback target must refuse: {other:?}"),
        }
    }
}
