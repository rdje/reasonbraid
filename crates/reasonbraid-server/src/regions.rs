//! The regional-routing machinery (`PHASE-8.5.2`, ADR-035 §20.10): the
//! regions are DECLARED (the declaration is the fail-closed seam) and
//! the cross-region delivery rides the EXPLICIT pair allowlist. The
//! routing decision names its refusal — the undeclared region or the
//! unpaired cross-region — never a silent drop. The `.5.3`
//! store-and-forward consumes this routing for the site-level
//! delivery.

use serde::Serialize;
use sqlx::PgPool;

/// The routing refusal — each variant names the boundary it refuses.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum RegionRefusal {
    /// The region has no declaration row (the fail-closed seam).
    UndeclaredRegion { region: String },
    /// The cross-region delivery has no pair row (the explicit allowlist).
    CrossRegionRefused { from: String, to: String },
}

impl std::fmt::Display for RegionRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionRefusal::UndeclaredRegion { region } => {
                write!(
                    f,
                    "the region `{region}` is undeclared — the routing refuses"
                )
            }
            RegionRefusal::CrossRegionRefused { from, to } => {
                write!(
                    f,
                    "the cross-region delivery `{from}` → `{to}` has no declared pair — the routing refuses"
                )
            }
        }
    }
}

impl std::error::Error for RegionRefusal {}

/// The two failure FAMILIES a routing decision can have, kept apart in the type
/// (`SIGNOFF-REPAIR.11.10`). A `RegionRefusal` is a verdict ABOUT the site's
/// configuration, reached by reading it successfully. A storage failure means
/// no verdict was reached at all — it says nothing about the configuration, and
/// delivering one as the other sends an operator to inspect region declarations
/// that are perfectly correct.
///
/// `SIGNOFF-REPAIR.3.2.1` drew this same line for the `pair` verb in the
/// site-authority service and recorded the reason there: an unavailable
/// database must never masquerade as an undeclared region. The review record
/// that asked for it named `pair` AND `route`; this is the other half.
#[derive(Debug)]
pub enum RouteError {
    /// The declarations and the pair allowlist were read, and the answer is no.
    Refused(RegionRefusal),
    /// A read failed, so the decision could not be made.
    Storage(sqlx::Error),
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // The refusal's own wording is unchanged: `PHASE-8.5.2`'s acceptance
            // and the operator-facing contract both rest on these exact names.
            RouteError::Refused(refusal) => refusal.fmt(f),
            // The cause belongs in controlled diagnostics, not in a message a
            // caller could mistake for a statement about the site.
            RouteError::Storage(_) => f.write_str(
                "the routing decision could not be made — a site-configuration read failed",
            ),
        }
    }
}

impl std::error::Error for RouteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RouteError::Refused(refusal) => Some(refusal),
            RouteError::Storage(error) => Some(error),
        }
    }
}

impl From<sqlx::Error> for RouteError {
    fn from(error: sqlx::Error) -> Self {
        RouteError::Storage(error)
    }
}

/// The routing decision: both regions must be DECLARED; the
/// cross-region delivery requires the pair row; the same-region
/// delivery always routes.
///
/// Each of the three reads propagates its storage error (`?` into
/// `RouteError::Storage`) rather than converting it into a verdict about the
/// site's configuration — `SIGNOFF-REPAIR.11.10`. The refusal branches, their
/// variants and their wording are unchanged.
pub async fn route(pool: &PgPool, from: &str, to: &str) -> Result<(), RouteError> {
    let declared: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(from)
            .fetch_one(pool)
            .await?;
    if !declared.unwrap_or(false) {
        return Err(RouteError::Refused(RegionRefusal::UndeclaredRegion {
            region: from.to_string(),
        }));
    }
    let declared: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(to)
            .fetch_one(pool)
            .await?;
    if !declared.unwrap_or(false) {
        return Err(RouteError::Refused(RegionRefusal::UndeclaredRegion {
            region: to.to_string(),
        }));
    }
    if from != to {
        let paired: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM region_pairs WHERE from_region = $1 AND to_region = $2)",
        )
        .bind(from)
        .bind(to)
        .fetch_one(pool)
        .await?;
        if !paired.unwrap_or(false) {
            return Err(RouteError::Refused(RegionRefusal::CrossRegionRefused {
                from: from.to_string(),
                to: to.to_string(),
            }));
        }
    }
    Ok(())
}
