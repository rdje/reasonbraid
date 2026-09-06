//! reasonbraid-cli — the Phase 0 command surface (`PHASE-0.6.1`).
//!
//! The `rb` binary drives the WP6 flow — enroll, create thread, invite, contribute,
//! challenge, revise, close, inspect — against the control API over HTTP JSON. This
//! library is the testable body of that binary: configuration, the local state file
//! (principal name → id, thread → tenant), the HTTP client, and one [`run_*`] function
//! per verb returning printable output. `main.rs` is thin clap glue.
//!
//! # Dev-profile identity
//!
//! The CLI presents the principal in the `x-reasonbraid-principal` header (trusted
//! dev credentials — no certificate issuer in Phase 0). Names are resolved against
//! the local state dir; raw `hpr_…`/`rol_…` ids are accepted directly.
//!
//! # State dir locality (§13)
//!
//! The state dir defaults to `<cwd>/.reasonbraid-cli` (repo-local when run from the
//! repo) and honors `REASONBRAID_CLI_STATE` — never `/tmp` or a home cache. The
//! server URL defaults to `http://127.0.0.1:4310` (`REASONBRAID_SERVER`).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use reasonbraid_core::{
    AgentRoleId, ClientContext, CommandEnvelope, HumanPrincipalId, RequestId, PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// The control API's default dev-profile address (`rb-server`).
pub const DEFAULT_SERVER: &str = "http://127.0.0.1:4310";
/// The dev-profile principal header (mirrors the server's `PRINCIPAL_HEADER`).
pub const PRINCIPAL_HEADER: &str = "x-reasonbraid-principal";

// ── Configuration ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub server_base: String,
    pub state_dir: PathBuf,
}

impl Config {
    /// Environment + defaults: `REASONBRAID_SERVER`, `REASONBRAID_CLI_STATE` (falls
    /// back to `<cwd>/.reasonbraid-cli` — same-volume, repo-local by default).
    pub fn from_env() -> Self {
        let server_base = std::env::var("REASONBRAID_SERVER")
            .unwrap_or_else(|_| DEFAULT_SERVER.to_string())
            .trim_end_matches('/')
            .to_string();
        let state_dir = match std::env::var("REASONBRAID_CLI_STATE") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => PathBuf::from(".reasonbraid-cli"),
        };
        Self {
            server_base,
            state_dir,
        }
    }
}

// ── State file ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPrincipal {
    pub kind: String,
    pub id: String,
    pub tenant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredThread {
    pub tenant_id: String,
    pub subject: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    pub principals: BTreeMap<String, StoredPrincipal>,
    pub threads: BTreeMap<String, StoredThread>,
}

impl StateFile {
    fn path(dir: &Path) -> PathBuf {
        dir.join("state.json")
    }

    pub fn load(dir: &Path) -> Result<Self, CliError> {
        let path = Self::path(dir);
        if !path.exists() {
            return Ok(StateFile::default());
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| CliError::state(format!("read {}: {e}", path.display())))?;
        serde_json::from_str(&raw)
            .map_err(|e| CliError::state(format!("parse {}: {e}", path.display())))
    }

    pub fn save(&self, dir: &Path) -> Result<(), CliError> {
        std::fs::create_dir_all(dir)
            .map_err(|e| CliError::state(format!("create {}: {e}", dir.display())))?;
        let raw = serde_json::to_string_pretty(self).expect("state serializes");
        std::fs::write(Self::path(dir), raw)
            .map_err(|e| CliError::state(format!("write state file: {e}")))
    }
}

/// A resolved actor for `--as`: the wire id, its kind, and the tenant this CLI knows
/// for it (from the state file — raw ids may have none).
pub struct PrincipalRef {
    pub id: String,
    pub kind: &'static str,
    pub tenant: Option<String>,
}

/// Resolve `--as` (a state-file name or a raw `hpr_…`/`rol_…` id).
pub fn resolve_principal(state: &StateFile, name_or_id: &str) -> Result<PrincipalRef, CliError> {
    if let Ok(h) = name_or_id.parse::<HumanPrincipalId>() {
        return Ok(PrincipalRef {
            tenant: state
                .principals
                .values()
                .find(|p| p.id == h.to_string())
                .map(|p| p.tenant.clone()),
            id: h.to_string(),
            kind: "human",
        });
    }
    if let Ok(r) = name_or_id.parse::<AgentRoleId>() {
        return Ok(PrincipalRef {
            tenant: state
                .principals
                .values()
                .find(|p| p.id == r.to_string())
                .map(|p| p.tenant.clone()),
            id: r.to_string(),
            kind: "role",
        });
    }
    match state.principals.get(name_or_id) {
        Some(p) => Ok(PrincipalRef {
            kind: if p.kind == "human" { "human" } else { "role" },
            id: p.id.clone(),
            tenant: Some(p.tenant.clone()),
        }),
        None => Err(CliError::usage(format!(
            "principal `{name_or_id}` is not enrolled in this CLI's state dir — run `rb enroll` first"
        ))),
    }
}

/// Resolve the tenant scope for a thread verb: explicit `--tenant`, then the stored
/// thread record, then the acting principal's stored tenant.
fn resolve_tenant(
    state: &StateFile,
    thread_id: &str,
    principal: &PrincipalRef,
    explicit: Option<&str>,
) -> Result<String, CliError> {
    if let Some(raw) = explicit {
        return raw
            .parse::<reasonbraid_core::TenantId>()
            .map(|t| t.to_string())
            .map_err(|_| CliError::usage(format!("tenant `{raw}` is malformed")));
    }
    if let Some(stored) = state.threads.get(thread_id) {
        return Ok(stored.tenant_id.clone());
    }
    if let Some(tenant) = &principal.tenant {
        return Ok(tenant.clone());
    }
    Err(CliError::usage(
        "cannot determine the tenant — pass --tenant or create the thread with this CLI"
            .to_string(),
    ))
}

// ── Errors ────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum CliError {
    Http(reqwest::Error),
    Server {
        status: u16,
        code: String,
        message: String,
    },
    Malformed(String),
    State(String),
    Usage(String),
}

impl CliError {
    fn state(detail: String) -> Self {
        CliError::State(detail)
    }
    fn usage(detail: String) -> Self {
        CliError::Usage(detail)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Http(e) => write!(f, "transport error: {e}"),
            CliError::Server {
                status,
                code,
                message,
            } => write!(f, "server error ({status} {code}): {message}"),
            CliError::Malformed(detail) => write!(f, "malformed server response: {detail}"),
            CliError::State(detail) => write!(f, "state error: {detail}"),
            CliError::Usage(detail) => write!(f, "{detail}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        CliError::Http(e)
    }
}

// ── HTTP client ───────────────────────────────────────────────────────────────────

pub struct ApiClient {
    base: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// A 2xx JSON answer, or a typed server error (the `{code, message}` shape).
    async fn parse(&self, response: reqwest::Response) -> Result<Value, CliError> {
        let status = response.status().as_u16();
        let body = response.bytes().await?;
        if !(200..300).contains(&status) {
            #[derive(Deserialize)]
            struct ErrorBody {
                code: String,
                message: String,
            }
            match serde_json::from_slice::<ErrorBody>(&body) {
                Ok(e) => {
                    return Err(CliError::Server {
                        status,
                        code: e.code,
                        message: e.message,
                    })
                }
                Err(_) => {
                    return Err(CliError::Server {
                        status,
                        code: "unknown".to_string(),
                        message: String::from_utf8_lossy(&body).into_owned(),
                    })
                }
            }
        }
        serde_json::from_slice(&body).map_err(|e| CliError::Malformed(e.to_string()))
    }

    pub async fn enroll(&self, body: Value) -> Result<Value, CliError> {
        let response = self
            .http
            .post(format!("{}/v1/enrollments", self.base))
            .json(&body)
            .send()
            .await?;
        self.parse(response).await
    }

    fn envelope(operation: &str, body: Value) -> CommandEnvelope {
        CommandEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            operation: operation.to_string(),
            request_id: RequestId::new(),
            // A fresh opaque key per invocation (§9.2): every CLI call is a NEW
            // command; replays are only ever transport-level.
            idempotency_key: Uuid::now_v7().to_string(),
            expected_aggregate_version: None,
            body,
            client_context: ClientContext::default(),
        }
    }

    async fn post_command(
        &self,
        path: &str,
        principal: &str,
        envelope: &CommandEnvelope,
    ) -> Result<Value, CliError> {
        let response = self
            .http
            .post(format!("{}{path}", self.base))
            .header(PRINCIPAL_HEADER, principal)
            .json(envelope)
            .send()
            .await?;
        self.parse(response).await
    }

    pub async fn create_thread(&self, principal: &str, body: Value) -> Result<Value, CliError> {
        let env = Self::envelope("thread.create", body);
        self.post_command("/v1/threads", principal, &env).await
    }

    pub async fn thread_command(
        &self,
        thread_id: &str,
        principal: &str,
        operation: &str,
        body: Value,
    ) -> Result<Value, CliError> {
        let env = Self::envelope(operation, body);
        self.post_command(
            &format!("/v1/threads/{thread_id}/commands"),
            principal,
            &env,
        )
        .await
    }

    pub async fn get(&self, path: &str, principal: &str) -> Result<Value, CliError> {
        let response = self
            .http
            .get(format!("{}{path}", self.base))
            .header(PRINCIPAL_HEADER, principal)
            .send()
            .await?;
        self.parse(response).await
    }
}

// ── Output ────────────────────────────────────────────────────────────────────────

fn or_json(value: &Value, json_out: bool) -> Result<String, CliError> {
    if json_out {
        serde_json::to_string_pretty(value).map_err(|e| CliError::Malformed(e.to_string()))
    } else {
        Ok(value.to_string())
    }
}

/// The human-readable summary of a command result.
fn command_summary(value: &Value) -> String {
    format!(
        "event {} ({}) — thread {} now {}",
        value["event_id"].as_str().unwrap_or("?"),
        value["event_type"].as_str().unwrap_or("?"),
        value["thread_id"].as_str().unwrap_or("?"),
        value["thread_state"].as_str().unwrap_or("?")
    )
}

// ── Verbs ─────────────────────────────────────────────────────────────────────────

pub async fn run_enroll(
    cfg: &Config,
    kind: &str,
    name: &str,
    tenant: Option<&str>,
    actions: Option<Vec<String>>,
    json_out: bool,
) -> Result<String, CliError> {
    let client = ApiClient::new(&cfg.server_base);
    let mut body = json!({ "kind": kind, "name": name });
    if let Some(t) = tenant {
        body["tenant_id"] = json!(t);
    }
    if let Some(a) = actions {
        body["actions"] = json!(a);
    }
    let response = client.enroll(body).await?;

    // Record the principal (and the tenant it belongs to) locally.
    let mut state = StateFile::load(&cfg.state_dir)?;
    if state.version == 0 {
        state.version = 1;
    }
    state.principals.insert(
        name.to_string(),
        StoredPrincipal {
            kind: kind.to_string(),
            id: response["principal_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            tenant: response["tenant_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        },
    );
    state.save(&cfg.state_dir)?;

    if json_out {
        return or_json(&response, true);
    }
    let replayed = response["replayed"].as_bool().unwrap_or(false);
    Ok(format!(
        "{} {kind} `{name}` as {} in tenant {}{}",
        if replayed {
            "already enrolled"
        } else {
            "enrolled"
        },
        response["principal_id"].as_str().unwrap_or("?"),
        response["tenant_id"].as_str().unwrap_or("?"),
        response["boundary_id"]
            .as_str()
            .map(|b| format!("\nboundary: {b}"))
            .unwrap_or_default(),
    ))
}

#[derive(Debug, Clone, Default)]
pub struct BudgetArgs {
    pub calls: Option<u64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub wall_clock_seconds: Option<u64>,
}

pub async fn run_thread_create(
    cfg: &Config,
    principal: &PrincipalRef,
    subject: &str,
    objective: &str,
    budget: &BudgetArgs,
    json_out: bool,
) -> Result<String, CliError> {
    let client = ApiClient::new(&cfg.server_base);
    let tenant = principal.tenant.as_deref().ok_or_else(|| {
        CliError::usage("the acting principal has no tenant — pass --tenant".to_string())
    })?;
    let mut body = json!({ "tenant_id": tenant, "subject": subject, "objective": objective });
    if budget.calls.is_some()
        || budget.input_tokens.is_some()
        || budget.output_tokens.is_some()
        || budget.wall_clock_seconds.is_some()
    {
        body["budget"] = json!({
            "calls": budget.calls,
            "input_tokens": budget.input_tokens,
            "output_tokens": budget.output_tokens,
            "wall_clock_seconds": budget.wall_clock_seconds,
        });
    }
    let response = client.create_thread(&principal.id, body).await?;

    // Remember the thread → tenant mapping for later verbs.
    let mut state = StateFile::load(&cfg.state_dir)?;
    if state.version == 0 {
        state.version = 1;
    }
    let thread_id = response["thread_id"].as_str().unwrap_or_default();
    state.threads.insert(
        thread_id.to_string(),
        StoredThread {
            tenant_id: tenant.to_string(),
            subject: subject.to_string(),
        },
    );
    state.save(&cfg.state_dir)?;

    if json_out {
        return or_json(&response, true);
    }
    Ok(format!(
        "created thread {thread_id} (state: open)\n{}",
        command_summary(&response)
    ))
}

/// The shared body of the five thread verbs (invite/contribute/challenge/revise/close).
/// The shared arguments of the five thread verbs (invite/contribute/challenge/
/// revise/close) — grouped so the runner keeps a small signature.
pub struct ThreadVerbArgs {
    pub thread_id: String,
    pub tenant: Option<String>,
    pub operation: &'static str,
    pub body: Value,
    pub json_out: bool,
}

pub async fn run_thread_verb(
    cfg: &Config,
    state: &StateFile,
    principal: &PrincipalRef,
    args: &ThreadVerbArgs,
) -> Result<String, CliError> {
    let client = ApiClient::new(&cfg.server_base);
    let tenant = resolve_tenant(state, &args.thread_id, principal, args.tenant.as_deref())?;
    let mut full = args.body.clone();
    let obj = full.as_object_mut().expect("verb body is an object");
    obj.insert("tenant_id".to_string(), json!(tenant));
    let response = client
        .thread_command(&args.thread_id, &principal.id, args.operation, full)
        .await?;
    if args.json_out {
        return or_json(&response, true);
    }
    Ok(command_summary(&response))
}

pub async fn run_inspect_thread(
    cfg: &Config,
    state: &StateFile,
    principal: &PrincipalRef,
    thread_id: &str,
    tenant_explicit: Option<&str>,
    json_out: bool,
) -> Result<String, CliError> {
    let client = ApiClient::new(&cfg.server_base);
    let tenant = resolve_tenant(state, thread_id, principal, tenant_explicit)?;
    let tenant_q = format!("tenant_id={tenant}");
    let thread = client
        .get(
            &format!("/v1/threads/{thread_id}?{tenant_q}"),
            &principal.id,
        )
        .await?;
    let events = client
        .get(
            &format!("/v1/threads/{thread_id}/events?{tenant_q}"),
            &principal.id,
        )
        .await?;
    let audit = client
        .get(
            &format!("/v1/threads/{thread_id}/audit?{tenant_q}"),
            &principal.id,
        )
        .await?;
    if json_out {
        return or_json(
            &json!({ "thread": thread, "events": events, "audit": audit }),
            true,
        );
    }

    let st = &thread["state"];
    let mut out = String::new();
    out.push_str(&format!(
        "thread: {} (tenant {})\nsubject: {}\nobjective: {}\nstate: {}{}\n",
        thread_id,
        tenant,
        st["subject"].as_str().unwrap_or("?"),
        st["objective"].as_str().unwrap_or("?"),
        st["state"].as_str().unwrap_or("?"),
        st["close_reason"]
            .as_str()
            .map(|r| format!(" — reason: {r}"))
            .unwrap_or_default(),
    ));
    out.push_str("participants:\n");
    if let Some(participants) = st["participants"].as_object() {
        for (id, state) in participants {
            out.push_str(&format!("  {id}: {}\n", state.as_str().unwrap_or("?")));
        }
    }
    out.push_str(&format!(
        "counters: contributions={} revisions={} open_challenges={}\n",
        st["contributions"].as_u64().unwrap_or(0),
        st["revisions"].as_u64().unwrap_or(0),
        st["open_challenges"].as_u64().unwrap_or(0),
    ));
    out.push_str(&format!(
        "ceiling: {}\n",
        st["ceiling_id"].as_str().unwrap_or("?")
    ));
    out.push_str("events:\n");
    for event in events["events"].as_array().cloned().unwrap_or_default() {
        out.push_str(&format!(
            "  {} #{} {} {}\n",
            event["event_id"].as_str().unwrap_or("?"),
            event["aggregate_version"].as_i64().unwrap_or(0),
            event["event_type"].as_str().unwrap_or("?"),
            event["body"]["content"]
                .as_str()
                .or(event["body"]["reason"].as_str())
                .or(event["body"]["subject"].as_str())
                .unwrap_or("")
        ));
    }
    out.push_str("audit:\n");
    for record in audit["records"].as_array().cloned().unwrap_or_default() {
        out.push_str(&format!(
            "  {} {} {} as {} — digest {}\n",
            record["record_id"].as_str().unwrap_or("?"),
            record["decision"].as_str().unwrap_or("?"),
            record["action"].as_str().unwrap_or("?"),
            record["actor"].as_str().unwrap_or("?"),
            record["policy_digest"].as_str().unwrap_or("?")
        ));
    }
    Ok(out)
}

pub async fn run_inspect_threads(
    cfg: &Config,
    principal: &PrincipalRef,
    tenant_explicit: Option<&str>,
    json_out: bool,
) -> Result<String, CliError> {
    let client = ApiClient::new(&cfg.server_base);
    let tenant = tenant_explicit
        .or(principal.tenant.as_deref())
        .ok_or_else(|| {
            CliError::usage("cannot determine the tenant — pass --tenant".to_string())
        })?;
    let listed = client
        .get(&format!("/v1/threads?tenant_id={tenant}"), &principal.id)
        .await?;
    if json_out {
        return or_json(&listed, true);
    }
    let mut out = format!("threads in tenant {tenant}:\n");
    for thread in listed["threads"].as_array().cloned().unwrap_or_default() {
        out.push_str(&format!(
            "  {} [{}] {}\n",
            thread["thread_id"].as_str().unwrap_or("?"),
            thread["state"].as_str().unwrap_or("?"),
            thread["subject"].as_str().unwrap_or("?")
        ));
    }
    Ok(out)
}

/// Resolve `--agent` for invite: a state-file name or a raw `rol_…` id.
pub fn resolve_agent(state: &StateFile, name_or_id: &str) -> Result<String, CliError> {
    if let Ok(role) = name_or_id.parse::<AgentRoleId>() {
        return Ok(role.to_string());
    }
    match state.principals.get(name_or_id) {
        Some(p) if p.kind == "role" => Ok(p.id.clone()),
        Some(_) => Err(CliError::usage(format!(
            "principal `{name_or_id}` is a human, not an agent role"
        ))),
        None => Err(CliError::usage(format!(
            "agent role `{name_or_id}` is not enrolled in this CLI's state dir"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names and raw ids resolve to the same principal; unknown names are a usage
    /// error (never a silent guess).
    #[test]
    fn principal_resolution_accepts_names_and_raw_ids() {
        let mut state = StateFile::default();
        state.principals.insert(
            "alice".to_string(),
            StoredPrincipal {
                kind: "human".to_string(),
                id: "hpr_00000000-0000-7000-8000-000000000001".to_string(),
                tenant: "ten_00000000-0000-7000-8000-000000000001".to_string(),
            },
        );

        let by_name = resolve_principal(&state, "alice").unwrap();
        assert_eq!(by_name.kind, "human");
        assert!(by_name.tenant.is_some());

        let by_id = resolve_principal(&state, "hpr_00000000-0000-7000-8000-000000000001").unwrap();
        assert_eq!(by_id.id, by_name.id);
        assert_eq!(by_id.tenant, by_name.tenant);

        let wrong_kind =
            resolve_principal(&state, "rol_00000000-0000-7000-8000-000000000001").unwrap();
        assert_eq!(wrong_kind.kind, "role");
        assert!(
            wrong_kind.tenant.is_none(),
            "raw ids unknown to the state file carry no tenant"
        );

        assert!(matches!(
            resolve_principal(&state, "nobody"),
            Err(CliError::Usage(_))
        ));
    }

    /// Tenant resolution order: explicit flag → stored thread → principal's tenant.
    #[test]
    fn tenant_resolution_order_is_explicit_then_stored_then_principal() {
        let mut state = StateFile::default();
        state.threads.insert(
            "thr_00000000-0000-7000-8000-000000000001".to_string(),
            StoredThread {
                tenant_id: "ten_00000000-0000-7000-8000-000000000002".to_string(),
                subject: "s".to_string(),
            },
        );
        let principal = PrincipalRef {
            id: "hpr_00000000-0000-7000-8000-000000000001".to_string(),
            kind: "human",
            tenant: Some("ten_00000000-0000-7000-8000-000000000003".to_string()),
        };
        let thread = "thr_00000000-0000-7000-8000-000000000001";

        assert_eq!(
            resolve_tenant(&state, thread, &principal, None).unwrap(),
            "ten_00000000-0000-7000-8000-000000000002",
            "the stored thread record wins over the principal's tenant"
        );
        assert_eq!(
            resolve_tenant(
                &state,
                thread,
                &principal,
                Some("ten_00000000-0000-7000-8000-000000000004")
            )
            .unwrap(),
            "ten_00000000-0000-7000-8000-000000000004",
            "the explicit flag always wins"
        );
        assert!(resolve_tenant(&state, thread, &principal, Some("junk")).is_err());
    }

    /// The state file round-trips through its wire form (disk persistence itself is
    /// exercised by the end-to-end suite against a repo-local state dir).
    #[test]
    fn state_file_round_trips() {
        let mut state = StateFile::default();
        assert_eq!(state.version, 0);
        state.version = 1;
        state.principals.insert(
            "alice".to_string(),
            StoredPrincipal {
                kind: "human".to_string(),
                id: "hpr_00000000-0000-7000-8000-000000000001".to_string(),
                tenant: "ten_00000000-0000-7000-8000-000000000001".to_string(),
            },
        );
        let wire = serde_json::to_string(&state).unwrap();
        let loaded: StateFile = serde_json::from_str(&wire).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.principals.len(), 1);
        assert_eq!(
            loaded.principals["alice"].id,
            "hpr_00000000-0000-7000-8000-000000000001"
        );
    }
}
