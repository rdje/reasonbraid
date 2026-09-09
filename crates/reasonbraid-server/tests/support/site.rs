//! Test fixtures issue site authority only through the deployment-controlled service.
use chrono::{Duration, Utc};
use reasonbraid_core::GrantSubject;
use reasonbraid_server::site_authority::{self as site, Action, Reason, Scope};
use sqlx::PgPool;

pub const ALL: &[Action] = &[
    Action::RegistryInspect,
    Action::AdapterAllow,
    Action::AdapterRevoke,
    Action::RegionDeclare,
    Action::RegionPair,
    Action::RegionUnpair,
];

pub async fn provision(pool: &PgPool, principal: &str, actions: &[Action]) -> (String, String) {
    let subject = if principal.starts_with("hpr_") {
        GrantSubject::Human(principal.parse().unwrap())
    } else {
        GrantSubject::Role(principal.parse().unwrap())
    };
    let scope = Scope {
        actions: actions.to_vec(),
        valid_from: Utc::now() - Duration::hours(2),
        expires_at: Utc::now() + Duration::hours(4),
    };
    let reason = Reason::new("explicit operator-issued HTTP test authority").unwrap();
    let boundary = site::issue_boundary(pool, &scope, &reason).await.unwrap();
    let boundary_id = boundary.result["boundary_id"].as_str().unwrap().to_owned();
    let grant = site::issue_grant(pool, &boundary_id, &subject, &scope, &reason)
        .await
        .unwrap();
    (
        boundary_id,
        grant.result["grant_id"].as_str().unwrap().to_owned(),
    )
}
