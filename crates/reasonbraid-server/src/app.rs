//! The control plane's HTTP surface, composed in ONE place
//! (`SIGNOFF-REPAIR.4.6.1.7`).
//!
//! `rb-server` serves the union of five routers. Before this module that union
//! was written inline in the binary's `main`, and nothing outside `main` could
//! build it — so the `OPERATOR-SURFACES` gate, which reads route strings from
//! the source, could only prove a surface was REGISTERED. A router left out of
//! the merge would have kept the gate green while the surface was gone.
//!
//! `main` now builds its app through [`control_plane_app`], and
//! `tests/operator_surfaces.rs` builds the SAME app through the same function
//! and calls every route `.doctrine/operator_surfaces.tsv` maps, which proves
//! each is SERVED.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

/// Everything the composed surface needs, as `rb-server` resolves it at boot.
pub struct ControlPlane {
    pub pool: PgPool,
    pub ca: Arc<crate::ca::ServerCa>,
    /// The declared publication root (`--publication-repo-root`); `None`
    /// closes the publish verb.
    pub publication_repo_root: Option<PathBuf>,
    /// The dependency monitor `GET /v1/health` reads.
    pub health: Arc<crate::health::HealthMonitor>,
    /// The declared backup directory (`--backup-dir`); `None` makes
    /// `GET /v1/admin/backups` say so rather than report nothing.
    pub backup_dir: Option<PathBuf>,
}

/// The whole HTTP surface `rb-server` serves: the control API, the node
/// channel, the console, the health read and the backup status.
pub fn control_plane_app(plane: ControlPlane) -> Router {
    crate::api_router_with_publication_root(plane.pool.clone(), plane.publication_repo_root)
        .merge(crate::node_router(plane.pool.clone(), plane.ca))
        .merge(crate::ui_router())
        .merge(crate::health::health_router(plane.health))
        .merge(crate::backup_router(plane.pool, plane.backup_dir))
}
