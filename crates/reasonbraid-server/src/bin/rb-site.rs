//! Deployment-local site authority tooling. No HTTP enrollment or schema mutation.

use std::io::{self, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use reasonbraid_core::GrantSubject;
use reasonbraid_server::site_authority::{
    self as site, Action, Collection, Disable, Reason, Scope,
};
use serde_json::{json, Value};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

#[derive(Parser)]
#[command(
    name = "rb-site",
    version,
    about = "Deployment-local site authority administration"
)]
struct Cli {
    /// Explicit loopback PostgreSQL URL; credentials are never printed.
    #[arg(long, env = "RB_SITE_DATABASE_URL", hide_env_values = true)]
    database_url: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Issue, inspect or disable site boundaries.
    Boundary {
        #[command(subcommand)]
        command: BoundaryCommand,
    },
    /// Issue, inspect or disable grants bound to an exact site boundary.
    Grant {
        #[command(subcommand)]
        command: GrantCommand,
    },
    /// Inspect durable site decisions and outcomes.
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
}

#[derive(Subcommand)]
enum BoundaryCommand {
    Issue(IssueBoundary),
    List(List),
    Suspend(DisableRecord),
    Revoke(DisableRecord),
}

#[derive(Subcommand)]
enum GrantCommand {
    Issue(IssueGrant),
    List(List),
    Suspend(DisableRecord),
    Revoke(DisableRecord),
}

#[derive(Subcommand)]
enum AuditCommand {
    List(List),
}

fn reason(value: &str) -> Result<Reason, String> {
    Reason::new(value).map_err(|error| error.to_string())
}

fn subject(value: &str) -> Result<GrantSubject, &'static str> {
    let invalid = "subject must be human:hpr_<UUID> or role:rol_<UUID>";
    let (kind, id) = value.split_once(':').ok_or(invalid)?;
    match kind {
        "human" => id.parse().map(GrantSubject::Human).map_err(|_| invalid),
        "role" => id.parse().map(GrantSubject::Role).map_err(|_| invalid),
        _ => Err(invalid),
    }
}

fn timestamp(value: &str) -> Result<DateTime<Utc>, &'static str> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| "timestamp must be RFC3339 with an explicit UTC offset")
}

#[derive(Args)]
struct Window {
    /// Repeat for each permitted site action.
    #[arg(long = "action", value_enum, required = true)]
    actions: Vec<Action>,
    /// Inclusive RFC3339 start, including an explicit UTC offset.
    #[arg(long, value_parser = timestamp)]
    valid_from: DateTime<Utc>,
    /// Exclusive RFC3339 expiry, including an explicit UTC offset.
    #[arg(long, value_parser = timestamp)]
    expires_at: DateTime<Utc>,
}

impl Window {
    fn scope(&self) -> Scope {
        Scope {
            actions: self.actions.clone(),
            valid_from: self.valid_from,
            expires_at: self.expires_at,
        }
    }
}

#[derive(Args)]
struct IssueBoundary {
    #[command(flatten)]
    window: Window,
    #[arg(long, value_parser = reason)]
    reason: Reason,
}

#[derive(Args)]
struct IssueGrant {
    #[arg(long)]
    boundary: String,
    #[arg(long, value_parser = subject)]
    subject: GrantSubject,
    #[command(flatten)]
    window: Window,
    #[arg(long, value_parser = reason)]
    reason: Reason,
}

#[derive(Args)]
struct DisableRecord {
    /// The exact site boundary or grant identifier.
    id: String,
    #[arg(long, value_parser = reason)]
    reason: Reason,
}

#[derive(Args)]
struct List {
    /// Return at most this many records, newest ID first.
    #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u16).range(1..=100))]
    limit: u16,
    /// Continue strictly before a prior receipt's next_before identifier.
    #[arg(long)]
    before: Option<String>,
    #[arg(long, value_parser = reason)]
    reason: Reason,
}

impl Command {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result<site::Receipt, site::Error> {
        match self {
            Self::Boundary { command } => match command {
                BoundaryCommand::Issue(args) => {
                    site::issue_boundary(pool, &args.window.scope(), &args.reason).await
                }
                BoundaryCommand::List(args) => list(pool, Collection::Boundaries, args).await,
                BoundaryCommand::Suspend(args) => {
                    site::disable_boundary(pool, &args.id, Disable::Suspend, &args.reason).await
                }
                BoundaryCommand::Revoke(args) => {
                    site::disable_boundary(pool, &args.id, Disable::Revoke, &args.reason).await
                }
            },
            Self::Grant { command } => match command {
                GrantCommand::Issue(args) => {
                    site::issue_grant(
                        pool,
                        &args.boundary,
                        &args.subject,
                        &args.window.scope(),
                        &args.reason,
                    )
                    .await
                }
                GrantCommand::List(args) => list(pool, Collection::Grants, args).await,
                GrantCommand::Suspend(args) => {
                    site::disable_grant(pool, &args.id, Disable::Suspend, &args.reason).await
                }
                GrantCommand::Revoke(args) => {
                    site::disable_grant(pool, &args.id, Disable::Revoke, &args.reason).await
                }
            },
            Self::Audit {
                command: AuditCommand::List(args),
            } => list(pool, Collection::Audit, args).await,
        }
    }
}

async fn list(
    pool: &sqlx::PgPool,
    collection: Collection,
    args: &List,
) -> Result<site::Receipt, site::Error> {
    site::inspect(
        pool,
        collection,
        args.limit,
        args.before.as_deref(),
        &args.reason,
    )
    .await
}

/// Validate without echoing the input on failure. Host/database/login/port are
/// explicit, and URL options cannot redirect the connection or select files.
struct Target {
    address: IpAddr,
    port: u16,
    username: String,
    password: String,
    database: String,
}

fn target(value: &str) -> Result<Target, &'static str> {
    let invalid = "an explicit loopback PostgreSQL URL with login, port and database is required; only sslmode=disable is supported";
    if value.len() > 8192 {
        return Err(invalid);
    }
    let parsed = url::Url::parse(value).map_err(|_| invalid)?;
    let host = parsed
        .host_str()
        .ok_or(invalid)?
        .trim_start_matches('[')
        .trim_end_matches(']');
    let address: IpAddr = host.parse().map_err(|_| invalid)?;
    if !matches!(parsed.scheme(), "postgres" | "postgresql")
        || !address.is_loopback()
        || parsed.port().is_none_or(|port| port == 0)
        || parsed.username().is_empty()
        || parsed.path().trim_start_matches('/').is_empty()
        || parsed.path().trim_start_matches('/').contains('/')
        || parsed.fragment().is_some()
    {
        return Err(invalid);
    }
    let options: Vec<_> = parsed.query_pairs().collect();
    if options.len() > 1
        || options
            .iter()
            .any(|(key, value)| key != "sslmode" || value != "disable")
    {
        return Err(invalid);
    }
    let decode = |value: &str| {
        percent_encoding::percent_decode_str(value)
            .decode_utf8()
            .map(|value| value.into_owned())
            .map_err(|_| invalid)
    };
    Ok(Target {
        address,
        port: parsed.port().ok_or(invalid)?,
        username: decode(parsed.username())?,
        password: decode(parsed.password().unwrap_or(""))?,
        database: decode(parsed.path().trim_start_matches('/'))?,
    })
}

fn output(value: &Value, code: u8) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if serde_json::to_writer(&mut stdout, value).is_err() || writeln!(stdout).is_err() {
        eprintln!("rb-site: output failed; inspect audit history before retrying issuance");
        return ExitCode::from(1);
    }
    ExitCode::from(code)
}

fn error(code: &str, message: &str, exit: u8) -> ExitCode {
    output(&json!({"error": {"code": code, "message": message}}), exit)
}

fn repository_root() -> Result<PathBuf, &'static str> {
    let current = std::env::current_dir().map_err(|_| "cannot locate repository")?;
    current
        .ancestors()
        .find(|path| path.join("Cargo.toml").is_file() && path.join("migrations").is_dir())
        .map(PathBuf::from)
        .ok_or("run rb-site from within the repository")
}

#[cfg(unix)]
async fn verify_storage(
    pool: &sqlx::PgPool,
    root: &std::path::Path,
    database: &str,
) -> Result<(), &'static str> {
    use std::os::unix::fs::MetadataExt;
    let invalid = "cannot verify repository-volume database storage; the operator needs data_directory inspection permission";
    let (directory, actual): (String, String) =
        sqlx::query_as("SELECT current_setting('data_directory'), current_database()")
            .fetch_one(pool)
            .await
            .map_err(|_| invalid)?;
    let data = std::fs::metadata(directory).map_err(|_| invalid)?;
    let repository = std::fs::metadata(root).map_err(|_| invalid)?;
    if actual != database || !data.is_dir() || data.dev() != repository.dev() {
        return Err("selected database storage must be on the repository filesystem volume");
    }
    Ok(())
}

#[cfg(not(unix))]
async fn verify_storage(
    _: &sqlx::PgPool,
    _: &std::path::Path,
    _: &str,
) -> Result<(), &'static str> {
    Err("operator storage verification currently requires a Unix deployment")
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let target = match target(&cli.database_url) {
        Ok(target) => target,
        Err(message) => return error("invalid_target", message, 2),
    };
    let root = match repository_root() {
        Ok(root) => root,
        Err(message) => return error("invalid_workspace", message, 2),
    };
    // This executable has not created a runtime or any threads. Remove libpq
    // defaults before constructing SQLx options; no shared environment is changed.
    for (key, _) in std::env::vars_os() {
        if key.to_str().is_some_and(|key| key.starts_with("PG")) {
            std::env::remove_var(key);
        }
    }
    // Avoid even the default-host socket-directory census in SQLx's constructor.
    std::env::set_var("PGHOST", target.address.to_string());
    std::env::set_var("PGUSER", &target.username);
    let options = PgConnectOptions::new_without_pgpass()
        .host(&target.address.to_string())
        .port(target.port)
        .username(&target.username)
        .password(&target.password)
        .database(&target.database)
        .ssl_mode(PgSslMode::Disable)
        .application_name("rb-site");
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => return error("runtime_unavailable", "could not start operator runtime", 1),
    };
    runtime.block_on(async {
        let pool = match PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
        {
            Ok(pool) => pool,
            Err(_) => {
                return error(
                    "database_unavailable",
                    "could not connect to the selected operator database",
                    1,
                )
            }
        };
        let locality = tokio::time::timeout(
            Duration::from_secs(5), verify_storage(&pool, &root, &target.database),
        ).await;
        if let Err(message) = locality.unwrap_or(Err("database storage verification timed out")) {
            pool.close().await;
            return error("storage_unverified", message, 1);
        }
        let result =
            tokio::time::timeout(Duration::from_secs(30), cli.command.execute(&pool)).await;
        pool.close().await;
        match result {
            Ok(Ok(receipt)) => match serde_json::to_value(receipt) {
                Ok(value) => output(&value, 0),
                Err(_) => error(
                    "output_failed",
                    "inspect audit history before retrying issuance",
                    1,
                ),
            },
            Ok(Err(site::Error::InvalidInput(message))) => error("invalid_input", message, 2),
            Ok(Err(site::Error::OperatorRequired)) => error(
                "operator_required",
                "explicit database operator authority required",
                3,
            ),
            Ok(Err(site::Error::Refused { reason, audit_id })) => {
                output(&json!({"error": {"code": reason, "audit_id": audit_id}}), 3)
            }
            Ok(Err(site::Error::Sql(_))) => error(
                "database_operation_failed",
                "site authority database operation failed; inspect audit history before retrying issuance",
                1,
            ),
            Err(_) => error(
                "operation_timeout",
                "operation timed out; inspect audit history before retrying issuance",
                1,
            ),
        }
    })
}
