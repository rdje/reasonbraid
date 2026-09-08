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

/// Declare one region (the operator's verb — an existing declaration is
/// the idempotent no-op).
pub async fn declare(pool: &PgPool, region: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO site_regions (region_id) VALUES ($1) ON CONFLICT (region_id) DO NOTHING",
    )
    .bind(region)
    .execute(pool)
    .await?;
    Ok(())
}

/// Pair two regions (the cross-region delivery allowlist).
pub async fn pair(pool: &PgPool, from: &str, to: &str) -> Result<(), RegionRefusal> {
    let from_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(from)
            .fetch_one(pool)
            .await
            .map_err(|_| RegionRefusal::UndeclaredRegion {
                region: from.to_string(),
            })?;
    if !from_exists.unwrap_or(false) {
        return Err(RegionRefusal::UndeclaredRegion {
            region: from.to_string(),
        });
    }
    let to_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_regions WHERE region_id = $1)")
            .bind(to)
            .fetch_one(pool)
            .await
            .map_err(|_| RegionRefusal::UndeclaredRegion {
                region: to.to_string(),
            })?;
    if !to_exists.unwrap_or(false) {
        return Err(RegionRefusal::UndeclaredRegion {
            region: to.to_string(),
        });
    }
    sqlx::query(
        "INSERT INTO region_pairs (from_region, to_region) VALUES ($1, $2) \
         ON CONFLICT (from_region, to_region) DO NOTHING",
    )
    .bind(from)
    .bind(to)
    .execute(pool)
    .await
    .map_err(|_| RegionRefusal::CrossRegionRefused {
        from: from.to_string(),
        to: to.to_string(),
    })?;
    Ok(())
}

/// Remove a pair (the NEXT cross-region delivery refuses).
pub async fn unpair(pool: &PgPool, from: &str, to: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM region_pairs WHERE from_region = $1 AND to_region = $2")
        .bind(from)
        .bind(to)
        .execute(pool)
        .await?;
    Ok(())
}

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
