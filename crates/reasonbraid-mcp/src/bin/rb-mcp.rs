//! `rb-mcp` — the MCP server over stdio (`SIGNOFF-REPAIR.6.8`, ADR-024, §9.6).
//!
//! 🔴 **Until this binary existed, six implemented, authorized, live-tested MCP
//! tools were reachable by no client.** `cargo metadata` reported 0 workspace
//! packages depending on `reasonbraid-mcp`, whose only targets were a `lib` and
//! a build script: the tool router was constructed nowhere outside
//! `#[cfg(test)]`. The transport had been deferred to `PHASE-8.3.4`, which
//! closed without it — `docs/knowledge/a-deferral-dies-with-the-leaf-it-names.md`.
//!
//! ⭐ **stdio, and the profile was priced rather than preferred**
//! (`docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md`):
//! `transport-io` adds **no** package to the graph, while the Streamable-HTTP
//! server transport adds three — one of them a second `base64` major, which
//! `deny.toml` forbids outright. It is also inside the LAN bar by construction:
//! this process opens no socket and accepts no inbound connection. The client
//! spawns it and owns its lifetime.
//!
//! ⛔ **The pool is LAZY, and that is a property of the transport rather than an
//! optimisation.** A client spawns this process and expects a handshake
//! immediately; refusing to start because a database is unreachable would turn
//! every tool into an unreachable one again, for a session that might only list
//! them. `connect_lazy` defers the connection to the first tool that needs it,
//! where the failure is reported as that tool's typed error — to the caller who
//! asked, instead of to a stderr nobody reads.
//!
//! ⚠️ **The principal rides the tool's arguments, not the transport**, which is
//! the dev profile's trust shape and is the same shape the HTTP header carries.
//! A stdio server has no header to carry one, and inventing an ambient identity
//! here would be a new authority this leaf has no mandate to create.

use rmcp::ServiceExt;

const DATABASE_URL: &str = "DATABASE_URL";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ⛔ Every diagnostic goes to stderr. stdout IS the transport — a stray
    // `println!` there is a malformed JSON-RPC frame, and the client's decoder
    // is what would report it.
    let url = std::env::var(DATABASE_URL).map_err(|_| {
        format!("{DATABASE_URL} is required: rb-mcp serves the inspection tools over that database")
    })?;
    let pool = sqlx::PgPool::connect_lazy(&url)
        .map_err(|error| format!("the {DATABASE_URL} value does not parse: {error}"))?;

    eprintln!("rb-mcp: serving the MCP tools over stdio (no socket is opened)");
    let service = reasonbraid_mcp::McpTools { pool }
        .serve(rmcp::transport::stdio())
        .await?;
    // The client owns the lifetime: this returns when the peer closes the
    // stream or the session ends, which for stdio is the client exiting.
    let reason = service.waiting().await?;
    eprintln!("rb-mcp: the session ended ({reason:?})");
    Ok(())
}
