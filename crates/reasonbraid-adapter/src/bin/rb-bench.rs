//! `rb-bench` — the WP7 deliberation/routing benchmark (`PHASE-0.7`).
//!
//! Runs the versioned corpus through the four workflows and writes the report
//! (JSON + markdown) under `--out`. Default: the deterministic scripted agent
//! (the self-test harness — no tokens). `--agent codex` requires
//! `RB_LIVE_CODEX=1` and dispatches REAL calls through the Codex CLI adapter —
//! spend is bounded by `--max-calls`.

use std::path::PathBuf;

use clap::Parser;
use reasonbraid_adapter::bench::{
    corpus::{Corpus, Prompts},
    report::{build, write},
    scripted::ScriptedAgent,
    workflows::{run_workflow, Workflow},
    BenchError,
};
use reasonbraid_adapter::CodexCliAdapter;

#[derive(Debug, Parser)]
#[command(
    name = "rb-bench",
    version,
    about = "ReasonBraid WP7 deliberation benchmark"
)]
struct Args {
    /// The corpus directory (contains corpus.json + prompts.json). Defaults to
    /// the crate's `bench/v1/` when run from a checkout.
    #[arg(long)]
    corpus: Option<PathBuf>,

    /// The agent: `scripted` (deterministic, default) or `codex` (real calls).
    #[arg(long, default_value = "scripted")]
    agent: String,

    /// Workflows to run (comma-separated: single, blind, critique_revise,
    /// moderator). Default: all four.
    #[arg(long, default_value = "all")]
    workflow: String,

    /// Case ids to run (comma-separated). Default: every case.
    #[arg(long)]
    case: Option<String>,

    /// Report output directory (repo-root-relative default under target/).
    #[arg(long)]
    out: Option<PathBuf>,

    /// Hard cap on agent calls (a real-run safety bound; the scripted corpus is
    /// 8 cases × 9 calls = 72).
    #[arg(long, default_value_t = 72)]
    max_calls: u64,
}

fn default_corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bench/v1")
}

fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/bench")
        .join(chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string())
}

fn git_rev() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let corpus_dir = args.corpus.unwrap_or_else(default_corpus_dir);
    let corpus = Corpus::load(&corpus_dir)?;
    let prompts = Prompts::load(&corpus_dir)?;
    let corpus_digest = Corpus::digest(&corpus_dir)?;
    let prompts_digest = Prompts::digest(&corpus_dir)?;

    let workflows: Vec<Workflow> = if args.workflow == "all" {
        Workflow::ALL.to_vec()
    } else {
        args.workflow
            .split(',')
            .map(|w| Workflow::parse(w.trim()).ok_or_else(|| format!("unknown workflow `{w}`")))
            .collect::<Result<Vec<_>, _>>()?
    };
    let selected: Vec<&reasonbraid_adapter::bench::Case> = match &args.case {
        None => corpus.cases.iter().collect(),
        Some(list) => {
            let ids: Vec<&str> = list.split(',').map(str::trim).collect();
            corpus
                .cases
                .iter()
                .filter(|c| ids.contains(&c.id.as_str()))
                .collect()
        }
    };
    if selected.is_empty() {
        return Err("no cases selected".into());
    }
    let call_budget: u64 = selected.len() as u64 * workflows.iter().map(|w| w.calls()).sum::<u64>();
    if call_budget > args.max_calls {
        return Err(format!(
            "the selection needs {call_budget} calls, over --max-calls {} — \
             narrow --case/--workflow or raise the cap deliberately",
            args.max_calls
        )
        .into());
    }

    let run_id = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let mut results = Vec::new();
    match args.agent.as_str() {
        "scripted" => {
            let agent = ScriptedAgent::new(&corpus.cases);
            run_selected(&agent, &prompts, &selected, &workflows, &mut results).await?;
        }
        "codex" => {
            if std::env::var("RB_LIVE_CODEX") != Ok("1".to_string()) {
                return Err(
                    "the codex agent spends REAL tokens — set RB_LIVE_CODEX=1 deliberately \
                     (the scripted agent is the default and needs no network)"
                        .into(),
                );
            }
            let agent = CodexCliAdapter::new();
            run_selected(&agent, &prompts, &selected, &workflows, &mut results).await?;
        }
        other => return Err(format!("unknown agent `{other}` (scripted | codex)").into()),
    }

    let out_dir = args.out.unwrap_or_else(default_out_dir);
    let report = build(
        &run_id,
        &git_rev(),
        &args.agent,
        &corpus,
        &corpus_digest,
        &prompts_digest,
        &results,
    );
    write(&report, &out_dir)?;
    eprintln!("rb-bench: report written to {}", out_dir.display());
    Ok(())
}

async fn run_selected<A: reasonbraid_adapter::Adapter + Sync>(
    agent: &A,
    prompts: &Prompts,
    selected: &[&reasonbraid_adapter::bench::Case],
    workflows: &[Workflow],
    results: &mut Vec<(String, Vec<reasonbraid_adapter::bench::WorkflowResult>)>,
) -> Result<(), BenchError> {
    for case in selected {
        let mut case_results = Vec::new();
        for workflow in workflows {
            eprintln!("rb-bench: case {} · {}", case.id, workflow.wire());
            case_results.push(run_workflow(agent, prompts, case, *workflow).await?);
        }
        results.push((case.id.clone(), case_results));
    }
    Ok(())
}
