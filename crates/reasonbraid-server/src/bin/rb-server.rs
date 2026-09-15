//! `rb-server` — the ReasonBraid control plane (`PHASE-0.6.1`).
//!
//! Serves the control API (the WP6 CLI surface) and the node channel on one
//! listener, applying the repository migrations on startup. Development profile:
//! `--database-url` (or `DATABASE_URL`) points at the control plane's PostgreSQL.

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use reasonbraid_server::{
    api_router_with_publication_root, ca::ensure_server_ca_with_store, node_router, publisher,
    r5r3rx_enabled, secret_store, sync_gated_entries, ui_router,
};

#[derive(Debug, Parser)]
#[command(
    name = "rb-server",
    version,
    about = "ReasonBraid control plane (Phase 0)"
)]
struct Args {
    /// Bind host (loopback by default — the Phase 0 dev profile).
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Bind port.
    #[arg(long, default_value_t = 4310)]
    port: u16,

    /// PostgreSQL URL for the control plane store (defaults to $DATABASE_URL).
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// The DECLARED secret-store profile (`.1.4.2`): the configuration
    /// choice the key reads route through. An undeclared name refuses the
    /// boot — never a silent fallback to the dev rows.
    #[arg(long, default_value = secret_store::PROFILE_DEV_DATABASE)]
    secret_store_profile: String,

    /// The publication repository ROOT (`.9.2.1.1`): the one directory
    /// `POST /v1/policy-publications/{id}/publish` may write inside. Leaving
    /// it unset CLOSES that verb — the caller's `repo_path` is a location
    /// within this root, never a path the server will open on its word.
    #[arg(long)]
    publication_repo_root: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // The declared publication root (`.9.2.1.1`): validated BEFORE anything
    // mutates, so a typo'd path refuses the boot instead of refusing it after
    // the schema has already moved. ⛔ This leaf adds this one argument and its
    // check; the secret-store ordering below and the `--host` gate are
    // `.11.12`'s to decide, and are deliberately left exactly where they are.
    let publication_repo_root = match args.publication_repo_root.as_deref() {
        Some(declared) => Some(publisher::validate_root(declared).map_err(|e| format!("{e}"))?),
        None => None,
    };

    let pool = sqlx::PgPool::connect(&args.database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    // The declared secret store (`.1.4.2`): resolved ONCE at boot — the
    // undeclared profile is the typed refusal, never a silent fallback.
    let store = secret_store::SecretStore::resolve(&args.secret_store_profile)
        .map_err(|e| format!("{e}"))?;

    // The workload-identity CA (`.1.2.1`, ADR-007): loaded from `server_ca` or
    // generated on first boot — it must survive restarts so issued leaves chain.
    // The material reads THROUGH the resolved store (the registry is the seam).
    let ca = Arc::new(ensure_server_ca_with_store(&pool, &store).await?);

    // The `.5.3` OPT-IN gate's startup sync: the R3/R5/RX registry rows
    // exist ONLY while the gate is open (the resolve never returns a
    // disabled pack — the disabled pack has no row).
    sync_gated_entries(&pool, r5r3rx_enabled()).await?;

    let app = api_router_with_publication_root(pool.clone(), publication_repo_root)
        .merge(node_router(pool, ca))
        .merge(ui_router());
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("rb-server listening on http://{addr} (Phase 0 dev profile)");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    eprintln!("rb-server shutting down");
}
