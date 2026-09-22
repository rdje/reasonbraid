//! `rb-reconciler` — the §15.8 publication reconciler (`SIGNOFF-REPAIR.9.3.5.1.2`;
//! ROADMAP §7.2's `reasonbraid-reconciler`, named `rb-*` like its siblings).
//!
//! One pass over every publication that recorded a Git operation: observe its
//! refs, apply what §15.8 lets a machine apply, and REPORT the rest. It never
//! migrates the schema — that is `rb-server`'s act — and it never adjudicates:
//! a conflict, a quarantine, a freeze or an out-of-band ref is printed and left
//! for a human.
//!
//! Exit status: `0` when every publication is consistent or was recovered; `3`
//! when at least one requires a human; `1` when the pass itself failed.

use std::process::ExitCode;

use clap::Parser;
use reasonbraid_server::publisher;
use reasonbraid_server::reconciliation::{self, Outcome};

#[derive(Debug, Parser)]
#[command(
    name = "rb-reconciler",
    version,
    about = "Reconcile publications against their Git repositories (ROADMAP §15.8)"
)]
struct Args {
    /// PostgreSQL URL for the control plane store (defaults to $DATABASE_URL).
    #[arg(long, env = "DATABASE_URL", hide_env_values = true)]
    database_url: String,

    /// The publication repository ROOT — the same directory `rb-server` was
    /// given. A recorded repository is resolved inside it and nowhere else.
    #[arg(long)]
    publication_repo_root: std::path::PathBuf,

    /// Reconcile only this publication instead of every candidate.
    #[arg(long)]
    publication: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Args::parse();
    let root = match publisher::validate_root(&args.publication_repo_root) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("rb-reconciler: {e}");
            return ExitCode::from(1);
        }
    };
    let pool = match sqlx::PgPool::connect(&args.database_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("rb-reconciler: could not connect to the control plane database");
            return ExitCode::from(1);
        }
    };
    let ids = match args.publication {
        Some(id) => vec![id],
        None => match reconciliation::candidates(&pool).await {
            Ok(ids) => ids,
            Err(e) => {
                eprintln!("rb-reconciler: {e}");
                return ExitCode::from(1);
            }
        },
    };
    let mut needs_human = false;
    let mut failed = false;
    for id in ids {
        match reconciliation::reconcile_publication(&pool, Some(&root), &id).await {
            Ok(outcome) => {
                needs_human |= matches!(outcome, Outcome::RequiresHuman { .. });
                println!("{id}: {outcome}");
            }
            Err(e) => {
                failed = true;
                println!("{id}: error: {e}");
            }
        }
    }
    pool.close().await;
    if failed {
        ExitCode::from(1)
    } else if needs_human {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}
