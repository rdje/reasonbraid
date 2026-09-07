//! `rb-server` — the ReasonBraid control plane (`PHASE-0.6.1`).
//!
//! Serves the control API (the WP6 CLI surface) and the node channel on one
//! listener, applying the repository migrations on startup. Development profile:
//! `--database-url` (or `DATABASE_URL`) points at the control plane's PostgreSQL.

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, ui_router};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let pool = sqlx::PgPool::connect(&args.database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    // The workload-identity CA (`.1.2.1`, ADR-007): loaded from `server_ca` or
    // generated on first boot — it must survive restarts so issued leaves chain.
    let ca = Arc::new(ensure_server_ca(&pool).await?);

    let app = api_router(pool.clone())
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
