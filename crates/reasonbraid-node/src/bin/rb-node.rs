//! `rb-node` — the ReasonBraid node worker (`PHASE-0.6.2`).
//!
//! The demo/development worker: reconcile against the control plane, poll the
//! inbox tail, execute thread work items behind the configured fake adapter, and
//! emit `work_result` events. The journal is the durable half (WAL +
//! `synchronous=FULL`); killing this process and restarting it on the same journal
//! recovers dispatched-but-unresolved attempts as `outcome_unknown` — bounded,
//! visible, never silently retried.

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use reasonbraid_adapter::{
    AdapterCapabilities, CancellationStrength, FakeAdapter, PolicyInjectionMode, ScriptStep,
    StatusLookupSpec,
};
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{LocalBudget, Node, NodeChannel, Worker, WorkerError};

#[derive(Debug, Parser)]
#[command(name = "rb-node", version, about = "ReasonBraid node worker (Phase 0)")]
struct Args {
    /// Path to the node's SQLite journal file.
    #[arg(long)]
    journal: PathBuf,

    /// Control-plane base URL.
    #[arg(long, default_value = "http://127.0.0.1:4310")]
    server: String,

    /// The node id — in the dev profile, the agent role wire id this node serves.
    #[arg(long)]
    node_id: String,

    /// The fake adapter's script (a JSON array of ScriptStep, e.g.
    /// `[{"step":"emit_chunk","chunk":"..."},{"step":"complete"}]`).
    #[arg(long, value_name = "JSON")]
    fake_script: String,

    /// The fake adapter's status lookup spec (JSON; default: unsupported).
    #[arg(long, default_value = r#"{"kind":"unsupported"}"#, value_name = "JSON")]
    status_lookup: String,

    /// The node's LOCAL budget ceiling (JSON BudgetDimensions). The server-side
    /// ceiling and reservation are the durable counterpart; this bounds the
    /// node's own headroom.
    #[arg(
        long,
        default_value = r#"{"calls":1000,"input_tokens":1000000,"output_tokens":1000000,"wall_clock_seconds":3600}"#,
        value_name = "JSON"
    )]
    local_ceiling: String,

    /// Poll interval in milliseconds between channel tails.
    #[arg(long, default_value_t = 500)]
    poll_ms: u64,

    /// Lease heartbeat cadence in milliseconds (the `.1.2.2` lease TTL is 60 s;
    /// this default renews it 4× per TTL).
    #[arg(long, default_value_t = 15_000)]
    heartbeat_ms: u64,

    /// A one-time enrollment token issued by the control plane
    /// (`rb node issue-token`); when present, the node enrolls BEFORE reconciling.
    #[arg(long)]
    enroll_token: Option<String>,

    /// The token's nonce (printed by `rb node issue-token`).
    #[arg(long)]
    enroll_nonce: Option<String>,

    /// The host claim the token was bound to (default: `dev-host`).
    #[arg(long, default_value = "dev-host")]
    host_claim: String,

    /// The node's dev signing secret (any non-empty string; the server stores it —
    /// the dev trust-store stance). Required: with `--enroll-token` it is the key
    /// being enrolled; afterwards it is the key every handshake's proof rides.
    #[arg(long)]
    node_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let script: Vec<ScriptStep> = serde_json::from_str(&args.fake_script)
        .map_err(|e| format!("--fake-script is not a ScriptStep array: {e}"))?;
    let lookup: StatusLookupSpec = serde_json::from_str(&args.status_lookup)
        .map_err(|e| format!("--status-lookup is not a StatusLookupSpec: {e}"))?;
    let ceiling: BudgetDimensions = serde_json::from_str(&args.local_ceiling)
        .map_err(|e| format!("--local-ceiling is not BudgetDimensions JSON: {e}"))?;

    // Capabilities follow the declared lookup: a provider that can prove results
    // exposes a request id (the fake does exactly this).
    let capabilities = AdapterCapabilities {
        streaming: true,
        cancellation: CancellationStrength::BestEffort,
        provider_idempotency: false,
        status_lookup: !matches!(lookup, StatusLookupSpec::Unsupported),
        tool_support: false,
        policy_injection: PolicyInjectionMode::None,
    };
    let adapter = FakeAdapter::new(script, lookup, capabilities);

    // Enrollment (`.1.2.1`): when a token is provided, consume it BEFORE any
    // channel traffic. The token is the credential; the secret becomes the node's
    // dev signing key (the `.1.2.2` handshake's HMAC proof rides it).
    if let Some(token) = &args.enroll_token {
        let nonce = args.enroll_nonce.as_deref().unwrap_or_else(|| {
            eprintln!("rb-node: --enroll-token requires --enroll-nonce");
            std::process::exit(1);
        });
        let channel =
            NodeChannel::new(&args.server, args.node_id.clone(), args.node_secret.clone());
        let enroll_resp = channel
            .enroll(
                token,
                &args.node_id,
                &args.host_claim,
                nonce,
                &args.node_secret,
            )
            .await
            .map_err(|e| format!("rb-node: enrollment failed: {e}"))?;
        // Persist the workload certificate beside the journal (`.1.2.1`,
        // ADR-007): the `.1.2.2` handshake signs its proof with this key.
        save_workload_identity(&args.journal, &enroll_resp)
            .map_err(|e| format!("rb-node: could not store the workload certificate: {e}"))?;
        eprintln!(
            "rb-node: {} enrolled on host claim `{}` (cert fingerprint {})",
            args.node_id,
            args.host_claim,
            enroll_resp
                .get("cert_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
        );
    }

    let node = Node::open(
        &args.journal,
        &args.server,
        args.node_id.clone(),
        args.node_secret.clone(),
    )
    .await?;
    node.reconcile().await?;
    eprintln!("rb-node: {} reconciled with {}", args.node_id, args.server);

    // The lease heartbeat (`.1.2.2`): keep the server-side lease alive while the
    // process runs. A refused heartbeat (fenced by a newer handshake, or expired)
    // only logs: the worker's poll fails on the same condition and the main loop's
    // reconcile re-handshakes — the shared channel picks the fresh token up.
    let heartbeat_channel = node.channel().clone();
    let heartbeat_ms = args.heartbeat_ms;
    tokio::spawn(async move {
        loop {
            if let Err(e) = heartbeat_channel.heartbeat().await {
                eprintln!("rb-node: heartbeat refused ({e}) — awaiting re-handshake");
            }
            tokio::time::sleep(Duration::from_millis(heartbeat_ms)).await;
        }
    });

    let worker = Worker::new(
        node.clone(),
        adapter,
        LocalBudget::new(ceiling),
        Duration::from_millis(args.poll_ms),
    );

    loop {
        match worker.run().await {
            Ok(()) => unreachable!("the work loop runs until an error"),
            Err(WorkerError::Channel(e)) => {
                // The channel died: reconcile (re-emitting anything pending with
                // its ORIGINAL id) and resume — nothing accepted is lost, nothing
                // duplicated.
                eprintln!("rb-node: channel lost ({e}) — reconciling");
                match node.reconcile().await {
                    Ok(()) => {
                        eprintln!("rb-node: reconciled");
                    }
                    Err(e) => {
                        eprintln!("rb-node: reconcile failed ({e}) — retrying in 1s");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("rb-node: fatal worker error: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Decode a hex string (the server ships DER as hex — the codebase hand-rolls
/// hex rather than taking a dependency).
fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

/// Persist the workload certificate beside the journal (`PHASE-2.1.2.1`,
/// ADR-007): `cert.der` + `key.der` in the journal's directory. The files are
/// node-local state (the journal's own volume — §13); the `.1.2.2` handshake
/// loads them to sign its proof.
fn save_workload_identity(
    journal_path: &std::path::Path,
    enroll: &serde_json::Value,
) -> Result<(), String> {
    let dir = journal_path
        .parent()
        .ok_or_else(|| "the journal path has no parent directory".to_string())?;
    let cert = enroll
        .get("cert_der")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "the enroll response lacks cert_der".to_string())?;
    let key = enroll
        .get("key_der")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "the enroll response lacks key_der".to_string())?;
    std::fs::write(dir.join("cert.der"), hex_decode(cert)?).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("key.der"), hex_decode(key)?).map_err(|e| e.to_string())?;
    Ok(())
}
