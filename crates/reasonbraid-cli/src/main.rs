//! `rb` — the ReasonBraid Phase 0 CLI (`PHASE-0.6.1`).
//!
//! Thin clap glue over `reasonbraid-cli`'s [`run_*`] verbs. Development profile:
//! principals travel in the trusted `x-reasonbraid-principal` header; the state dir
//! defaults to `./.reasonbraid-cli` (see the library docs).

use clap::{Parser, Subcommand};
use reasonbraid_cli::{
    resolve_agent, resolve_principal, run_boundary_revoke, run_breaker_arm, run_breaker_reset,
    run_enroll, run_grant_revoke, run_inspect_boundaries, run_inspect_breakers, run_inspect_budget,
    run_inspect_grants, run_inspect_incarnations, run_inspect_node_inbox, run_inspect_runs,
    run_inspect_thread, run_inspect_threads, run_inspect_usage, run_issue_node_token,
    run_prune_node_inbox, run_quarantine_command, run_replay_command, run_revoke_node,
    run_thread_create, run_thread_verb, BudgetArgs, Config, CreateProfileArgs, PrincipalRef,
    StateFile, ThreadVerbArgs,
};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(name = "rb", version, about = "ReasonBraid Phase 0 CLI (dev profile)")]
struct Cli {
    /// Control-plane base URL (default: $REASONBRAID_SERVER or http://127.0.0.1:4310).
    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Enroll a development principal. A human without --tenant bootstraps a NEW
    /// tenant (boundary + admin grant); a role enrolls into an existing tenant.
    Enroll {
        /// `human` or `role`.
        kind: String,
        /// The name this CLI remembers the principal by.
        name: String,
        /// Existing tenant (required for roles; omitted = bootstrap a new tenant).
        #[arg(long)]
        tenant: Option<String>,
        /// Role actions to grant (comma-separated wire names, e.g. thread_contribute).
        #[arg(long, value_delimiter = ',')]
        actions: Option<Vec<String>>,
        /// Print the raw JSON response.
        #[arg(long)]
        json: bool,
    },
    /// Thread verbs.
    #[command(subcommand)]
    Thread(ThreadCommand),
    /// Inspect state through the API (no database surgery).
    #[command(subcommand)]
    Inspect(InspectCommand),
    /// Node administration (`.1.2.1`).
    #[command(subcommand)]
    Node(NodeCommand),
    /// Grant administration (`.1.3.2`).
    #[command(subcommand)]
    Grant(GrantCommand),
    /// Enrollment-boundary administration (`.1.3.2`).
    #[command(subcommand)]
    Boundary(BoundaryCommand),
    /// Spend circuit breaker administration (`.3.2`).
    #[command(subcommand)]
    Breaker(BreakerCommand),
}

#[derive(Debug, Subcommand)]
enum BreakerCommand {
    /// Arm the tenant's breaker: declare the spend threshold (BudgetDimensions
    /// JSON) — re-arming clears any trip.
    Arm {
        #[arg(long)]
        threshold: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Reset the tenant's TRIPPED breaker (the latch re-opens).
    Reset {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum GrantCommand {
    /// Revoke a grant — the subject loses its authority at the next decision.
    Revoke {
        #[arg(long)]
        grant: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum BoundaryCommand {
    /// Revoke the enrollment boundary — the tenant's ceiling is gone, so every
    /// grant under it is refused at the next decision (the nuclear option).
    Revoke {
        #[arg(long)]
        boundary: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum NodeCommand {
    /// Issue a one-time node enrollment token (tenant_admin authority).
    IssueToken {
        /// The node the token is bound to (a raw nod_… id).
        #[arg(long)]
        node: String,
        /// The host claim the token is bound to (e.g. the host name).
        #[arg(long)]
        host_claim: String,
        /// Token lifetime in seconds (default 3600).
        #[arg(long)]
        ttl_seconds: Option<i64>,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Quarantine one inbox command — it is never re-delivered (`.1.2.3`).
    Quarantine {
        /// The node whose inbox holds the command.
        #[arg(long)]
        node: String,
        /// The inbox command id to quarantine.
        #[arg(long)]
        command: String,
        /// WHY it is quarantined (required — a quarantine without a reason is a silent skip).
        #[arg(long)]
        reason: String,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Replay one DEAD-LETTERED inbox command (`.2.4`): the quarantine clears
    /// and the command re-enters the delivery tail with a fresh admission
    /// decision.
    Replay {
        /// The node whose inbox holds the dead-lettered command.
        #[arg(long)]
        node: String,
        /// The dead-lettered command id to replay.
        #[arg(long)]
        command: String,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Revoke a node.s workload certificates (`.1.3.1`): the next handshake
    /// is refused and presence reads suspended.
    Revoke {
        /// The node whose active certificates are revoked.
        #[arg(long)]
        node: String,
        /// WHY it is revoked (required).
        #[arg(long)]
        reason: String,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Inspect one node.s inbox: delivery + quarantine facts per row (`.1.2.3`).
    Inbox {
        /// The node whose inbox to list.
        #[arg(long)]
        node: String,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Prune DELIVERED inbox rows older than a retention window — an explicit,
    /// measured operator action with a before/after report (`.1.2.3`).
    Prune {
        /// The node whose delivered history to prune.
        #[arg(long)]
        node: String,
        /// The retention window in seconds (rows acknowledged at least this long ago).
        #[arg(long, default_value_t = 604800)]
        min_age_seconds: i64,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ThreadCommand {
    /// Create a thread (thread_create — targets the tenant).
    Create {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        budget_calls: Option<u64>,
        #[arg(long)]
        budget_input_tokens: Option<u64>,
        #[arg(long)]
        budget_output_tokens: Option<u64>,
        #[arg(long)]
        budget_wall_clock: Option<u64>,
        /// Thread classification: `general` | `confidential` (default `general`).
        #[arg(long)]
        classification: Option<String>,
        /// Workflow profile: `single-agent` | `blind-independent` | `critique-revise`
        /// | `moderator` (default `single-agent` — the ADR-002 routing default).
        #[arg(long)]
        workflow_profile: Option<String>,
        /// Allow join requests (default off — explicit participants first, §20.3).
        #[arg(long)]
        allow_join_requests: bool,
        /// The acting principal (a state-file name or a raw hpr_…/rol_… id).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Invite an agent role to a thread.
    Invite {
        #[arg(long)]
        thread: String,
        /// The invited agent role (a state-file name or a raw rol_… id).
        #[arg(long)]
        agent: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Contribute content to a thread.
    Contribute {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        text: String,
        /// Contribution kind: `position` (default) | `claim` | `assumption` |
        /// `evidence-reference` | `question` | `summary`.
        #[arg(long, default_value = "position")]
        kind: String,
        /// An evidence URI to attach (repeatable — references only, no acquisition).
        #[arg(long)]
        evidence_uri: Vec<String>,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Challenge a contribution (target = the contribution's event id).
    Challenge {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Revise after a challenge (target = the challenge's event id).
    Revise {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Close a thread with a stop reason (contributions and open objections stay).
    Close {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        reason: String,
        /// Close outcome (`.2.4.1`, §13.4): the twelve terminals — `accepted_unanimously`,
        /// `accepted_with_recorded_objections`, `accepted_by_rule` (the `decided` alias's
        /// canonical name, the default), `advisory_answer_only`, `deadlocked` (the
        /// `inconclusive` alias's canonical name), `no_quorum`, `insufficient_evidence`,
        /// `budget_exhausted`, `expired`, `cancelled`, `human_decision_required`,
        /// `unsafe_to_continue`. A decision terminal refuses `--unresolved`.
        #[arg(long)]
        outcome: Option<String>,
        /// An item that prevented a decision (repeatable; refused on a decided close).
        #[arg(long)]
        unresolved: Vec<String>,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Accept this role's PENDING invitation (`.1.3.1`): the actor is the
    /// invited role — accepting is the transaction that dispatches the work.
    Accept {
        #[arg(long)]
        thread: String,
        /// The acting principal (the invited role).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Join a thread whose join requests are open (`.1.3.2`): the actor is the
    /// joining role — no invitation needed (the self-request path).
    Join {
        #[arg(long)]
        thread: String,
        /// The acting principal (the joining role).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Advance the thread to its next round (`.1.5.2`): the round is SERVER-assigned;
    /// humans carry the `thread_advance_round` grant, roles stay deny-by-default.
    AdvanceRound {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Decline this role's PENDING invitation (`.1.3.1`).
    Decline {
        #[arg(long)]
        thread: String,
        /// The acting principal (the invited role).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Remove one participant (tenant_admin; `.1.3.1`) — the role is `revoked`.
    RemoveParticipant {
        #[arg(long)]
        thread: String,
        /// The participant to remove (a rol_…/hpr_… wire id).
        #[arg(long)]
        participant: String,
        /// The acting principal (a tenant admin).
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Cancel a thread — the abandonment terminal, with a reason (distinct from close).
    Cancel {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        /// The delegated subject (`.1.4.2`: `hpr_…` | `rol_…` — the grant holder).
        #[arg(long)]
        on_behalf_of: Option<String>,
        /// Why (audit context).
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum InspectCommand {
    /// One thread: state, participants, counters, event timeline, and audit records.
    Thread {
        thread: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// One thread's budget ledger: the ceiling + every reservation row — held vs
    /// settled usage, denials with their reasons (`.1.6.1`; read-only, inspect-gated).
    Budget {
        thread: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// All threads in a tenant.
    Threads {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's grants with their statuses (`.1.3.2`; tenant_admin).
    Grants {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's enrollment boundaries with their statuses (`.1.3.2`; tenant_admin).
    Boundaries {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's incarnations with their §8.1 facts (`.1.6.1`; tenant_admin).
    Incarnations {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's usage reconciliation (`.3.3`; tenant_admin).
    Usage {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's spend circuit breaker state (`.3.2`; tenant_admin).
    Breakers {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// The tenant's runs with their attempt→incarnation links (`.1.6.2`; tenant_admin).
    Runs {
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

/// Resolve the acting principal: `--as` value, or the most recently used one.
fn acting_principal(
    state: &StateFile,
    as_: Option<&str>,
) -> Result<PrincipalRef, reasonbraid_cli::CliError> {
    match as_ {
        Some(name) => resolve_principal(state, name),
        None => Err(reasonbraid_cli::CliError::Usage(
            "--as <name-or-id> is required (dev profile: the CLI acts as an enrolled principal)"
                .to_string(),
        )),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut cfg = Config::from_env();
    if let Some(server) = &cli.server {
        cfg.server_base = server.trim_end_matches('/').to_string();
    }

    let outcome = run(cli, &cfg).await;
    match outcome {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

async fn run(cli: Cli, cfg: &Config) -> Result<String, reasonbraid_cli::CliError> {
    let state = StateFile::load(&cfg.state_dir)?;
    match cli.command {
        Command::Enroll {
            kind,
            name,
            tenant,
            actions,
            json,
        } => run_enroll(cfg, &kind, &name, tenant.as_deref(), actions, json).await,
        Command::Thread(ThreadCommand::Create {
            subject,
            objective,
            budget_calls,
            budget_input_tokens,
            budget_output_tokens,
            budget_wall_clock,
            classification,
            workflow_profile,
            allow_join_requests,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let budget = BudgetArgs {
                calls: budget_calls,
                input_tokens: budget_input_tokens,
                output_tokens: budget_output_tokens,
                wall_clock_seconds: budget_wall_clock,
            };
            // The create verb's tenant comes from the acting principal (enroll
            // stored it); --tenant overrides.
            let mut principal_for_create = principal;
            if let Some(t) = &tenant {
                principal_for_create.tenant = Some(t.clone());
            }
            let profile = CreateProfileArgs {
                classification,
                workflow_profile,
                allow_join_requests,
            };
            run_thread_create(
                cfg,
                &principal_for_create,
                &subject,
                &objective,
                &budget,
                &profile,
                on_behalf_of.as_deref(),
                purpose.as_deref(),
                json,
            )
            .await
        }
        Command::Thread(ThreadCommand::Invite {
            thread,
            agent,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let role = resolve_agent(&state, &agent)?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.invite",
                    body: json!({ "agent_role": role }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Contribute {
            thread,
            text,
            kind,
            evidence_uri,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            // The human kebab spelling (`evidence-reference`) normalizes to the wire
            // snake_case (`evidence_reference`) — the `.1.1.3` profile precedent.
            let kind = kind.replace('-', "_");
            let evidence_refs: Vec<serde_json::Value> = evidence_uri
                .iter()
                .map(|uri| json!({ "uri": uri }))
                .collect();
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.contribute",
                    body: json!({
                        "content": text,
                        "kind": kind,
                        "evidence_refs": evidence_refs,
                    }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Challenge {
            thread,
            target,
            text,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.challenge",
                    body: json!({ "target_event_id": target, "content": text }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Revise {
            thread,
            target,
            text,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.revise",
                    body: json!({ "target_event_id": target, "content": text }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Close {
            thread,
            reason,
            outcome,
            unresolved,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let mut body = json!({ "reason": reason, "unresolved": unresolved });
            if let Some(outcome) = outcome {
                body["outcome"] = json!(outcome.replace('-', "_"));
            }
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.close",
                    body,
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Cancel {
            thread,
            reason,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.cancel",
                    body: json!({ "reason": reason }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Accept {
            thread,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.accept_invitation",
                    body: json!({}),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Join {
            thread,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.join",
                    body: json!({}),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::AdvanceRound {
            thread,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.advance_round",
                    body: json!({}),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Decline {
            thread,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.decline_invitation",
                    body: json!({}),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::RemoveParticipant {
            thread,
            participant,
            as_,
            tenant,
            on_behalf_of,
            purpose,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_thread_verb(
                cfg,
                &state,
                &principal,
                &ThreadVerbArgs {
                    thread_id: thread,
                    tenant,
                    operation: "thread.remove_participant",
                    body: json!({ "participant": participant }),
                    on_behalf_of,
                    purpose,
                    json_out: json,
                },
            )
            .await
        }
        Command::Inspect(InspectCommand::Thread {
            thread,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_thread(cfg, &state, &principal, &thread, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Budget {
            thread,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_budget(cfg, &state, &principal, &thread, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Threads { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_threads(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Grants { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_grants(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Boundaries { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_boundaries(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Incarnations { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_incarnations(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Runs { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_runs(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Breakers { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_breakers(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Inspect(InspectCommand::Usage { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_usage(cfg, &principal, tenant.as_deref(), json).await
        }
        Command::Grant(GrantCommand::Revoke {
            grant,
            reason,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_grant_revoke(cfg, &principal, &tenant, &grant, &reason, json).await
        }
        Command::Boundary(BoundaryCommand::Revoke {
            boundary,
            reason,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_boundary_revoke(cfg, &principal, &tenant, &boundary, &reason, json).await
        }
        Command::Breaker(BreakerCommand::Arm {
            threshold,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_breaker_arm(cfg, &principal, &tenant, &threshold, json).await
        }
        Command::Breaker(BreakerCommand::Reset { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_breaker_reset(cfg, &principal, &tenant, json).await
        }
        Command::Node(NodeCommand::IssueToken {
            node,
            host_claim,
            ttl_seconds,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_issue_node_token(
                cfg,
                &principal,
                &tenant,
                &node,
                &host_claim,
                ttl_seconds,
                json,
            )
            .await
        }
        Command::Node(NodeCommand::Revoke {
            node,
            reason,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_revoke_node(cfg, &principal, &tenant, &node, &reason, json).await
        }
        Command::Node(NodeCommand::Quarantine {
            node,
            command,
            reason,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_quarantine_command(cfg, &principal, &tenant, &node, &command, &reason, json).await
        }
        Command::Node(NodeCommand::Replay {
            node,
            command,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_replay_command(cfg, &principal, &tenant, &node, &command, json).await
        }
        Command::Node(NodeCommand::Inbox {
            node,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_inspect_node_inbox(cfg, &principal, &tenant, &node, json).await
        }
        Command::Node(NodeCommand::Prune {
            node,
            min_age_seconds,
            as_,
            tenant,
            json,
        }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            let tenant = tenant.or(principal.tenant.clone()).ok_or_else(|| {
                reasonbraid_cli::CliError::usage(
                    "cannot determine the tenant — pass --tenant".to_string(),
                )
            })?;
            run_prune_node_inbox(cfg, &principal, &tenant, &node, min_age_seconds, json).await
        }
    }
}
