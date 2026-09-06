//! `rb-journal` — the ReasonBraid node journal inspection CLI (`PHASE-0.3.1`).
//!
//! Operators inspect a node journal without opening SQLite by hand (`KICKOFF.md` WP3
//! acceptance). The handle is opened READ-ONLY (`SQLITE_OPEN_READONLY`), so this tool
//! can never mutate the journal — and WAL mode lets it run beside a live node.
//!
//! - `inspect`   — durability profile, integrity check, schema version, row counts.
//! - `pending`   — in-flight attempts (prepared/dispatched) and events awaiting ack.
//! - `ambiguous` — `outcome_unknown` attempts awaiting proof or adjudication, with
//!   their before/after boundary history.
//! - `events`    — every event the node emitted (original ids + payloads), the
//!   duplicate-transport evidence surface (`PHASE-0.6.2`).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use reasonbraid_node::{Journal, JournalError};

#[derive(Parser)]
#[command(
    name = "rb-journal",
    version = env!("CARGO_PKG_VERSION"),
    about = "ReasonBraid node journal inspection (read-only by construction)"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Journal health: durability profile, integrity check, schema version, row counts.
    Inspect {
        /// Path to the node journal file (e.g. `<node-dir>/node.db`).
        path: PathBuf,
    },
    /// In-flight work: attempts not yet terminal and outgoing events not yet acknowledged.
    Pending {
        /// Path to the node journal file.
        path: PathBuf,
        /// Emit machine-readable JSON instead of the human-readable listing.
        #[arg(long)]
        json: bool,
    },
    /// Ambiguous provider attempts (outcome_unknown) awaiting proof or adjudication.
    Ambiguous {
        /// Path to the node journal file.
        path: PathBuf,
        /// Emit machine-readable JSON instead of the human-readable listing.
        #[arg(long)]
        json: bool,
    },
    /// Every event the node emitted (acknowledged or not), with its original ids —
    /// the duplicate-transport evidence surface (`PHASE-0.6.2`).
    Events {
        /// Path to the node journal file.
        path: PathBuf,
        /// Emit machine-readable JSON instead of the human-readable listing.
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), JournalError> {
    match cli.command {
        Cmd::Inspect { path } => inspect(&path).await,
        Cmd::Pending { path, json } => pending(&path, json).await,
        Cmd::Ambiguous { path, json } => ambiguous(&path, json).await,
        Cmd::Events { path, json } => events(&path, json).await,
    }
}

async fn inspect(path: &PathBuf) -> Result<(), JournalError> {
    let journal = Journal::open_readonly(path).await?;
    let health = journal.health().await?;
    let counts = journal.counts().await?;

    println!("journal: {}", journal.path().display());
    println!("journal_mode: {} (recorded profile)", health.journal_mode);
    println!("synchronous: {} (recorded profile)", health.synchronous);
    println!("foreign_keys: {}", health.foreign_keys);
    println!("busy_timeout_ms: {}", health.busy_timeout_ms);
    println!("schema user_version: {}", health.user_version);
    println!("quick_check: {}", health.quick_check);
    println!(
        "commands: {} · operations: {}",
        counts.commands, counts.operations
    );
    println!(
        "attempts: prepared={} dispatched={} completed={} failed_before_dispatch={} \
         failed_known={} outcome_unknown={} reconciled={}",
        counts.attempts_prepared,
        counts.attempts_dispatched,
        counts.attempts_completed,
        counts.attempts_failed_before_dispatch,
        counts.attempts_failed_known,
        counts.attempts_outcome_unknown,
        counts.attempts_reconciled
    );
    println!(
        "outgoing events: emitted={} acked={}",
        counts.events_emitted, counts.events_acked
    );
    Ok(())
}

async fn pending(path: &PathBuf, json: bool) -> Result<(), JournalError> {
    let journal = Journal::open_readonly(path).await?;
    let attempts = journal.pending_attempts().await?;
    let events = journal.pending_events().await?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "attempts": attempts, "events": events })
        );
        return Ok(());
    }

    println!("pending attempts: {}", attempts.len());
    for a in &attempts {
        match &a.provider_request_id {
            Some(provider) => println!(
                "  {}  {}  op {}  provider_request_id={}",
                a.attempt_id, a.status, a.operation_id, provider
            ),
            None => println!("  {}  {}  op {}", a.attempt_id, a.status, a.operation_id),
        }
    }
    println!(
        "pending outgoing events (awaiting acknowledgement): {}",
        events.len()
    );
    for e in &events {
        println!(
            "  {}  op {}  emitted_at={}",
            e.event_id, e.operation_id, e.emitted_at
        );
    }
    Ok(())
}

async fn events(path: &PathBuf, json: bool) -> Result<(), JournalError> {
    let journal = Journal::open_readonly(path).await?;
    let emitted = journal.emitted_events().await?;

    if json {
        println!("{}", serde_json::json!({ "events": emitted }));
        return Ok(());
    }

    println!("emitted events: {}", emitted.len());
    for e in &emitted {
        println!(
            "  {}  op {}  emitted_at={}  payload={}",
            e.event_id, e.operation_id, e.emitted_at, e.payload
        );
    }
    Ok(())
}

async fn ambiguous(path: &PathBuf, json: bool) -> Result<(), JournalError> {
    let journal = Journal::open_readonly(path).await?;
    let attempts = journal.ambiguous_attempts().await?;

    if json {
        let mut enriched = Vec::new();
        for a in &attempts {
            let history = journal.attempt_history(&a.attempt_id).await?;
            enriched.push(serde_json::json!({
                "attempt": a,
                "transitions": history,
            }));
        }
        println!("{}", serde_json::json!({ "attempts": enriched }));
        return Ok(());
    }

    println!("ambiguous attempts: {}", attempts.len());
    for a in &attempts {
        println!(
            "  {}  outcome_unknown  op {}  updated_at={}",
            a.attempt_id, a.operation_id, a.updated_at
        );
        if let Some(provider) = &a.provider_request_id {
            println!("    provider_request_id={provider}");
        }
        if let Some(evidence) = &a.evidence {
            println!("    evidence={evidence}");
        }
        for t in journal.attempt_history(&a.attempt_id).await? {
            println!("    {} -> {}  (at {})", t.from_status, t.to_status, t.at);
        }
    }
    Ok(())
}
