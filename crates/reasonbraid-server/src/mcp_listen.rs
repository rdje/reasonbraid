//! The MCP listen-stream DURABLE state (`PHASE-8.3.4`, ADR-024, §9.6):
//! the listen stream is the EPHEMERAL transport state; the durable state
//! — the subscription, the last accepted ReasonBraid cursor, the delivery
//! ids, the deduplication — stays in REASONBRAID. The reconnect resumes
//! from the OWN cursor and surfaces the possible-gap when the upstream
//! offers no replay (never stronger than the upstream can prove).

use sqlx::PgPool;

/// The dedup window's size (the recent delivery ids kept per subscription).
pub const DEDUP_WINDOW: usize = 64;

/// Record one accepted delivery: the dedup check (the delivery id seen
/// → the replay SKIP, the cursor unchanged) and the cursor advance. The
/// caller's transaction commits the state WITH the delivery's effects.
pub async fn record_delivery_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    subscription_id: &str,
    delivery_id: &str,
    cursor: i64,
) -> Result<bool, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let row: Option<(i64, serde_json::Value)> = sqlx::query_as(
        "SELECT last_cursor, dedup_window FROM mcp_listen_state \
         WHERE tenant_id = $1 AND subscription_id = $2 FOR UPDATE",
    )
    .bind(tenant_id)
    .bind(subscription_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((_last, window)) = row else {
        // The first delivery registers the state (the cursor starts HERE).
        sqlx::query(
            "INSERT INTO mcp_listen_state \
             (tenant_id, subscription_id, last_cursor, last_delivery, dedup_window, updated_at) \
             VALUES ($1, $2, $3, $4, $5, now())",
        )
        .bind(tenant_id)
        .bind(subscription_id)
        .bind(cursor)
        .bind(delivery_id)
        .bind(serde_json::json!([delivery_id]))
        .execute(&mut *tx)
        .await?;
        return Ok(true);
    };
    let seen: Vec<String> = serde_json::from_value(window).unwrap_or_default();
    if seen.iter().any(|d| d == delivery_id) {
        return Ok(false); // the replay skip — the cursor unchanged
    }
    // ⛔ THE MOST RECENT `DEDUP_WINDOW` ids, and the direction is the whole
    // repair (`SIGNOFF-REPAIR.6.2.1`). This was `push` + `truncate`, which
    // appends to the END and keeps the FRONT — so once the window was full every
    // new id was written at index `DEDUP_WINDOW` and discarded on the same line,
    // and the window froze on the first 64 ids it ever saw. Measured over 70
    // deliveries: it held `d-000 … d-063` and replaying `d-069` was ACCEPTED,
    // which is a double delivery, because this function's `true` is what tells
    // the caller to commit the delivery's effects.
    //
    // ⭐ The array stays CHRONOLOGICAL — oldest first, newest last — so an
    // existing row keeps its meaning and the drain simply removes from the old
    // end. It also heals a row that is already over-long, whatever wrote it.
    let mut next = seen;
    next.push(delivery_id.to_string());
    if next.len() > DEDUP_WINDOW {
        next.drain(..next.len() - DEDUP_WINDOW);
    }
    // ⛔ THE CURSOR IS A HIGH-WATER MARK (`SIGNOFF-REPAIR.6.2.2`). This was
    // `SET last_cursor = $1`, unconditional, so recording cursor 100 and then
    // cursor 5 left the stored cursor at 5 — and `resume_plan` reads that value
    // as the OWN cursor, so one out-of-order delivery rewound the resume point
    // and every delivery above it was re-offered on the next reconnect.
    //
    // ⭐ A CLAMP, not a refusal, and the two differ observably. A late delivery
    // is a real delivery — the dedup check above has already said it is new — so
    // refusing it would DROP it, which is worse than the cursor problem it would
    // fix. The delivery is applied and enters the window; only the mark is
    // protected.
    //
    // ⭐ `last_delivery` moves with the cursor and not with the write, so the
    // pair stays ONE fact — *the delivery that set the mark* — rather than two
    // independent latest-writes that can disagree about which delivery they
    // describe. `listen_state` returns them together as the reconnect's input.
    sqlx::query(
        "UPDATE mcp_listen_state \
         SET last_cursor = GREATEST(last_cursor, $1), \
             last_delivery = CASE WHEN $1 > last_cursor THEN $2 ELSE last_delivery END, \
             dedup_window = $3, updated_at = now() \
         WHERE tenant_id = $4 AND subscription_id = $5",
    )
    .bind(cursor)
    .bind(delivery_id)
    .bind(serde_json::to_value(&next).expect("the window serializes"))
    .bind(tenant_id)
    .bind(subscription_id)
    .execute(&mut *tx)
    .await?;
    Ok(true)
}

/// The reconnect's resume plan: the resume is ALWAYS from the OWN
/// cursor; the possible-gap flag names the honest condition when the
/// upstream offers no replay (the deliveries between the own cursor and
/// the upstream's state may be lost — the continuation is never
/// advertised as stronger than the upstream can prove).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlan {
    /// The cursor the delivery resumes from (the OWN last cursor).
    pub resume_from: i64,
    /// Whether the upstream offers a replay (false → the possible-gap).
    pub upstream_replay: bool,
    /// The possible-gap condition to surface to the operator.
    pub possible_gap: bool,
}

/// Compute the resume plan for a reconnected listen stream.
pub fn resume_plan(own_cursor: i64, upstream_replay: bool) -> ResumePlan {
    ResumePlan {
        resume_from: own_cursor,
        upstream_replay,
        possible_gap: !upstream_replay,
    }
}

/// Read the durable state for a subscription (the reconnect's input).
pub async fn listen_state(
    pool: &PgPool,
    tenant_id: &str,
    subscription_id: &str,
) -> Result<Option<(i64, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT last_cursor, last_delivery FROM mcp_listen_state \
         WHERE tenant_id = $1 AND subscription_id = $2",
    )
    .bind(tenant_id)
    .bind(subscription_id)
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The resume plan is always the OWN cursor; the gap flag rides the
    /// upstream's replay capability.
    #[test]
    fn the_resume_plan_is_the_own_cursor_and_the_gap_is_honest() {
        let with_replay = resume_plan(42, true);
        assert_eq!(with_replay.resume_from, 42);
        assert!(!with_replay.possible_gap, "the replay closes the gap");
        let without_replay = resume_plan(42, false);
        assert_eq!(without_replay.resume_from, 42);
        assert!(
            without_replay.possible_gap,
            "no replay → the possible-gap surfaces"
        );
    }
}
