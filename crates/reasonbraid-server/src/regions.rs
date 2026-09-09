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

/// The routing decision: both regions must be DECLARED; the
/// cross-region delivery requires the pair row; the same-region
/// delivery always routes.
pub async fn route(pool: &PgPool, from: &str, to: &str) -> Result<(), RegionRefusal> {
    let declared: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(from)
            .fetch_one(pool)
            .await
            .map_err(|_| RegionRefusal::UndeclaredRegion {
                region: from.to_string(),
            })?;
    if !declared.unwrap_or(false) {
        return Err(RegionRefusal::UndeclaredRegion {
            region: from.to_string(),
        });
    }
    let declared: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(to)
            .fetch_one(pool)
            .await
            .map_err(|_| RegionRefusal::UndeclaredRegion {
                region: to.to_string(),
            })?;
    if !declared.unwrap_or(false) {
        return Err(RegionRefusal::UndeclaredRegion {
            region: to.to_string(),
        });
    }
    if from != to {
        let paired: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM region_pairs WHERE from_region = $1 AND to_region = $2)",
        )
        .bind(from)
        .bind(to)
        .fetch_one(pool)
        .await
        .map_err(|_| RegionRefusal::CrossRegionRefused {
            from: from.to_string(),
            to: to.to_string(),
        })?;
        if !paired.unwrap_or(false) {
            return Err(RegionRefusal::CrossRegionRefused {
                from: from.to_string(),
                to: to.to_string(),
            });
        }
    }
    Ok(())
}
