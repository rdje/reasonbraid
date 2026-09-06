//! `rb` — the ReasonBraid Phase 0 CLI (`PHASE-0.6.1`).
//!
//! Thin clap glue over `reasonbraid-cli`'s [`run_*`] verbs. Development profile:
//! principals travel in the trusted `x-reasonbraid-principal` header; the state dir
//! defaults to `./.reasonbraid-cli` (see the library docs).

use clap::{Parser, Subcommand};
use reasonbraid_cli::{
    resolve_agent, resolve_principal, run_enroll, run_inspect_node_inbox, run_inspect_thread,
    run_inspect_threads, run_issue_node_token, run_prune_node_inbox, run_quarantine_command,
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
    /// Inspect one node's inbox: delivery + quarantine facts per row (`.1.2.3`).
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
        #[arg(long)]
        json: bool,
    },
    /// Contribute content to a thread.
    Contribute {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
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
        #[arg(long)]
        json: bool,
    },
    /// Close a thread with a stop reason (contributions and open objections stay).
    Close {
        #[arg(long)]
        thread: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        as_: Option<String>,
        #[arg(long)]
        tenant: Option<String>,
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
    /// All threads in a tenant.
    Threads {
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
                json,
            )
            .await
        }
        Command::Thread(ThreadCommand::Invite {
            thread,
            agent,
            as_,
            tenant,
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
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Contribute {
            thread,
            text,
            as_,
            tenant,
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
                    operation: "thread.contribute",
                    body: json!({ "content": text }),
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
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Close {
            thread,
            reason,
            as_,
            tenant,
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
                    operation: "thread.close",
                    body: json!({ "reason": reason }),
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
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Accept {
            thread,
            as_,
            tenant,
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
                    json_out: json,
                },
            )
            .await
        }
        Command::Thread(ThreadCommand::Decline {
            thread,
            as_,
            tenant,
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
        Command::Inspect(InspectCommand::Threads { as_, tenant, json }) => {
            let principal = acting_principal(&state, as_.as_deref())?;
            run_inspect_threads(cfg, &principal, tenant.as_deref(), json).await
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
