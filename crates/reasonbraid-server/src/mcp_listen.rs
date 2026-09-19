//! The MCP listen-stream DURABLE state and the reconnect ritual
//! (`PHASE-8.3.4`, `SIGNOFF-REPAIR.6.2.4`, ADR-024, §9.6): the listen
//! stream is the EPHEMERAL transport state; the durable state — the
//! subscription, the last accepted ReasonBraid cursor, the delivery ids,
//! the deduplication — stays in REASONBRAID.
//!
//! §9.6 and ADR-024 specify the same FIVE-step ritual, performed by
//! ReasonBraid acting as an MCP *client* to an upstream MCP server:
//!
//! 1. reauthorize;
//! 2. recreate the listen request;
//! 3. reconcile any source-specific gap, if the upstream supports one;
//! 4. resume from the OWN cursor;
//! 5. surface an explicit possible-gap condition when the upstream
//!    offers no replay.
//!
//! [`reconnect`] performs all five against a [`ListenUpstream`], which is
//! the transport seam: the ritual is transport-neutral, exactly as the
//! node channel's is, and the profile's own words are *transport-neutral
//! request stream*.
//!
//! ⛔ **The rule every step serves: the MCP continuation is NEVER
//! advertised as stronger than the upstream can prove.** That sentence is
//! in both sources and it is what makes step 3 a separate step — *offers
//! a replay* and *covers our gap* are different facts, and
//! [`UpstreamReplay`] exists because a boolean cannot hold two facts.

use std::fmt;
use std::future::Future;

use futures_util::stream::Stream;
use sqlx::PgPool;

/// The dedup window's size (the recent delivery ids kept per subscription).
pub const DEDUP_WINDOW: usize = 64;

/// What can go wrong recording a delivery (`SIGNOFF-REPAIR.6.2.3`).
///
/// ⛔ The malformed window is a SEPARATE variant rather than a storage error,
/// because the two need different answers from an operator: a storage failure is
/// retried, and a row whose `dedup_window` is not an array of delivery ids has
/// to be looked at. Collapsing them would send the second one down the first
/// one's path.
#[derive(Debug)]
pub enum ListenError {
    /// The durable row could not be read or written.
    Storage(sqlx::Error),
    /// The stored `dedup_window` is not an array of delivery ids, so this
    /// subscription's deduplication cannot be evaluated. The message names the
    /// subscription, because that is what an operator has to go and look at.
    MalformedWindow(String),
}

impl std::fmt::Display for ListenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Connection details belong in controlled diagnostics, not in a
            // message a caller may surface — the rule `site_authority::Error`
            // and `PolicyError` already follow.
            Self::Storage(_) => f.write_str("the listen state could not be read or written"),
            Self::MalformedWindow(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for ListenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(e) => Some(e),
            Self::MalformedWindow(_) => None,
        }
    }
}

impl From<sqlx::Error> for ListenError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

/// Record one accepted delivery: the dedup check (the delivery id seen
/// → the replay SKIP, the cursor unchanged) and the cursor advance. The
/// caller's transaction commits the state WITH the delivery's effects.
pub async fn record_delivery_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    subscription_id: &str,
    delivery_id: &str,
    cursor: i64,
) -> Result<bool, ListenError>
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
    // ⛔ FAIL CLOSED (`SIGNOFF-REPAIR.6.2.3`). This was
    // `serde_json::from_value(window).unwrap_or_default()`, which turned any
    // value that is not an array of delivery ids into an EMPTY window — so the
    // subscription silently stopped deduplicating and a delivery it had already
    // recorded replayed and was ACCEPTED. ⛔ And it was self-erasing: the UPDATE
    // below then overwrote the malformed value with a fresh one-element array,
    // destroying the only evidence that anything had been wrong.
    //
    // ⭐ Refusing costs one subscription's deliveries until an operator looks at
    // it; continuing costs every delivery on that subscription being applicable
    // twice, invisibly. This module exists to prevent the second, so it may not
    // trade it for the first. It is the same fail-closed position `quota.rs`
    // takes for an unconfigured scope, and for the same reason.
    //
    // ⚠️ The refusal returns BEFORE the UPDATE, deliberately: a refusal that
    // still rewrote the row would destroy the same evidence the old code did.
    let seen: Vec<String> = serde_json::from_value(window).map_err(|_| {
        ListenError::MalformedWindow(format!(
            "the dedup window stored for subscription `{subscription_id}` is not an array of delivery ids, so deduplication cannot be evaluated"
        ))
    })?;
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
    // cursor 5 left the stored cursor at 5 — and `reconnect` reads that value as
    // the OWN cursor, so one out-of-order delivery rewound the resume point and
    // every delivery above it was re-offered on the next reconnect.
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

/// What the upstream can PROVE about replay — the input to step 3 of the
/// ritual (`SIGNOFF-REPAIR.6.2.4`).
///
/// ⛔ This replaced a `bool`, and the replacement IS the repair. The old
/// `resume_plan(own_cursor, upstream_replay: bool)` set
/// `possible_gap = !upstream_replay`, so an upstream that answered *yes, I
/// replay* closed the gap **whatever it could actually replay from**. An
/// upstream whose earliest replayable point sits ABOVE our own cursor offers
/// a replay and still cannot produce the deliveries in between — and
/// reporting no gap there is exactly *advertising the continuation as
/// stronger than the upstream can prove*, which §9.6 and ADR-024 forbid in
/// the same sentence.
///
/// ⭐ The old two-valued function was DELETED rather than kept as a wrapper.
/// A wrapper would have had to read `true` as *replays everything*, which is
/// the over-claim itself — keeping it would have left the defect reachable
/// behind a shorter name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpstreamReplay {
    /// The upstream offers no replay mechanism at all.
    None,
    /// The upstream will replay from this cursor onward, and no earlier.
    From(i64),
}

/// The reconnect's resume plan: the resume is ALWAYS from the OWN cursor, and
/// the possible-gap flag names the honest condition — the deliveries between
/// what we hold and what the upstream can still produce may be lost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlan {
    /// The cursor the delivery resumes from — the OWN last accepted cursor,
    /// or `None` when this subscription has accepted nothing yet (the durable
    /// row is written by the FIRST delivery, so a reconnect can legitimately
    /// precede it).
    pub resume_from: Option<i64>,
    /// What the upstream proved it can replay.
    pub upstream_replay: UpstreamReplay,
    /// The possible-gap condition to surface to the operator.
    pub possible_gap: bool,
}

/// Steps 3–5: reconcile what the upstream can replay against what we hold,
/// resume from the OWN cursor, and decide the possible-gap condition.
///
/// The gap is claimed CLOSED in exactly one case — the upstream can replay
/// from a point at or below the cursor we have already accepted, so nothing
/// between the two can be missing. Every other case surfaces the gap:
///
/// - no replay at all — the profile's own literal condition;
/// - a replay floor ABOVE our cursor — the upstream offers a replay that does
///   not reach back far enough, and the deliveries in between are unprovable;
/// - no own cursor — we have accepted nothing, so there is no cursor to
///   compare the upstream's floor against and no continuity to claim. ⛔ Not
///   an edge case to tidy away: it is the same rule applied to our own
///   ignorance, and it errs towards surfacing a gap rather than hiding one.
pub fn reconcile(own_cursor: Option<i64>, upstream_replay: UpstreamReplay) -> ResumePlan {
    let possible_gap = match (own_cursor, &upstream_replay) {
        (Some(own), UpstreamReplay::From(earliest)) => *earliest > own,
        (None, UpstreamReplay::From(_)) | (_, UpstreamReplay::None) => true,
    };
    ResumePlan {
        resume_from: own_cursor,
        upstream_replay,
        possible_gap,
    }
}

/// The upstream's authorization for ONE recreated listen request (step 1).
///
/// ⛔ **It grants NOTHING inside ReasonBraid.** ADR-024: *the remote MCP
/// metadata NEVER grants authority; the OAuth/authorization maps to the
/// tenant identity + the scoped grants*. This value admits US to the
/// upstream; what we may then do with what it sends is decided by
/// ReasonBraid's own grants, on the ordinary path.
///
/// ⛔ [`fmt::Debug`] is written by hand and REDACTS the credential. ADR-024's
/// *the tokens are never copied into the thread content* is about content,
/// and a derived `Debug` in a log line or a `.expect()` message is the other
/// way the same secret escapes — the same position `ListenError::Storage`
/// already takes for connection details.
#[derive(Clone)]
pub struct UpstreamAuthorization(String);

impl UpstreamAuthorization {
    /// Wrap a credential the upstream issued.
    #[must_use]
    pub fn new(credential: String) -> Self {
        Self(credential)
    }

    /// The credential, for the transport that presents it upstream.
    #[must_use]
    pub fn credential(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for UpstreamAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UpstreamAuthorization(<redacted>)")
    }
}

/// What can go wrong talking to the upstream during the ritual.
///
/// The two variants are the two steps that touch the network, kept apart
/// because they need different answers: a reauthorization the upstream
/// refuses is a credential or enrolment problem, and a listen request that
/// will not open is a transport or subscription problem.
#[derive(Debug)]
pub enum UpstreamError {
    /// Step 1 — the upstream refused or could not complete the reauthorization.
    Reauthorization(String),
    /// Step 2 — the listen request could not be recreated.
    Listen(String),
}

impl fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reauthorization(what) => {
                write!(f, "the upstream reauthorization did not complete: {what}")
            }
            Self::Listen(what) => write!(f, "the listen request could not be recreated: {what}"),
        }
    }
}

impl std::error::Error for UpstreamError {}

/// What can go wrong performing the whole ritual.
#[derive(Debug)]
pub enum ReconnectError {
    /// The OWN cursor could not be read, so step 4 has no input. ⛔ There is
    /// no fallback: resuming from a cursor we could not read would re-deliver
    /// or skip silently, which is the failure the durable state exists to
    /// prevent.
    State(ListenError),
    /// An upstream step did not complete.
    Upstream(UpstreamError),
}

impl fmt::Display for ReconnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(e) => write!(f, "{e}"),
            Self::Upstream(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ReconnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(e) => Some(e),
            Self::Upstream(e) => Some(e),
        }
    }
}

/// One delivery as the upstream offers it on the resumed stream.
///
/// The `cursor` is the upstream's ordering value, which is what
/// [`record_delivery_in_tx`] stores as the OWN cursor once ReasonBraid has
/// accepted the delivery — the two are the same scale by construction, because
/// accepting is what makes an upstream cursor ours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    /// The upstream's id for this delivery — the dedup key.
    pub delivery_id: String,
    /// The upstream's cursor for this delivery.
    pub cursor: i64,
}

/// What a recreated listen request yields: what the upstream proved it can
/// replay, and the stream it will deliver on.
///
/// ⛔ The stream is part of the RESULT rather than something the caller opens
/// afterwards, because a reconnect that does not hand back a live stream has
/// not reconnected to anything — it has only asked a question.
pub struct UpstreamListen<S> {
    /// What the upstream can replay, for step 3.
    pub replay: UpstreamReplay,
    /// The resumed delivery stream.
    pub deliveries: S,
}

/// The ritual's outcome: the plan the delivery resumes on, and the stream it
/// resumes on.
pub struct Reconnected<S> {
    /// The resume point and the possible-gap condition.
    pub plan: ResumePlan,
    /// The resumed delivery stream.
    pub deliveries: S,
}

/// The upstream listen source, as the reconnect ritual needs it.
///
/// ⭐ The seam is here, and not in a transport type, because §9.6 calls
/// `subscriptions/listen` a *transport-neutral request stream* and the node
/// channel already demonstrates the shape: the semantics — the cursor, the
/// replay, the reconciliation — are written once and a transport is supplied
/// under them.
///
/// The futures are spelled with `impl Future<Output = …> + Send` rather than
/// `async fn`, so a caller can hold the returned future across a `.await` in a
/// spawned task.
pub trait ListenUpstream {
    /// The resumed delivery stream this upstream produces.
    type Deliveries: Stream<Item = Result<Delivery, UpstreamError>> + Send;

    /// Step 1 — reauthorize, and return the authorization the recreated listen
    /// request will carry.
    fn reauthorize(
        &self,
    ) -> impl Future<Output = Result<UpstreamAuthorization, UpstreamError>> + Send;

    /// Steps 2 and 3 — recreate the listen request from our OWN cursor, and
    /// report what the upstream can replay along with the stream it resumes on.
    ///
    /// `resume_from` is what ReasonBraid holds, not what the upstream last
    /// sent; `None` means this subscription has accepted nothing yet.
    fn recreate_listen(
        &self,
        authorization: &UpstreamAuthorization,
        subscription_id: &str,
        resume_from: Option<i64>,
    ) -> impl Future<Output = Result<UpstreamListen<Self::Deliveries>, UpstreamError>> + Send;
}

/// Perform the reconnect ritual and return the plan the delivery resumes on.
///
/// The order is the profile's order, and it is load-bearing rather than
/// stylistic: the reauthorization comes FIRST because the recreated listen
/// request carries it, and the own cursor is read from the DURABLE state
/// rather than from anything the dead stream held — that is the whole point
/// of the state being durable.
pub async fn reconnect<U: ListenUpstream + Sync>(
    pool: &PgPool,
    upstream: &U,
    tenant_id: &str,
    subscription_id: &str,
) -> Result<Reconnected<U::Deliveries>, ReconnectError> {
    let own_cursor = listen_state(pool, tenant_id, subscription_id)
        .await
        .map_err(|e| ReconnectError::State(ListenError::Storage(e)))?
        .map(|(cursor, _last_delivery)| cursor);
    let authorization = upstream
        .reauthorize()
        .await
        .map_err(ReconnectError::Upstream)?;
    let listen = upstream
        .recreate_listen(&authorization, subscription_id, own_cursor)
        .await
        .map_err(ReconnectError::Upstream)?;
    Ok(Reconnected {
        plan: reconcile(own_cursor, listen.replay),
        deliveries: listen.deliveries,
    })
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

    /// The resume point is ALWAYS the OWN cursor, whatever the upstream says
    /// about replay — step 4 does not negotiate.
    #[test]
    fn the_resume_point_is_always_the_own_cursor() {
        for replay in [
            UpstreamReplay::None,
            UpstreamReplay::From(0),
            UpstreamReplay::From(41),
            UpstreamReplay::From(42),
            UpstreamReplay::From(99),
        ] {
            assert_eq!(
                reconcile(Some(42), replay.clone()).resume_from,
                Some(42),
                "the resume point moved for {replay:?}"
            );
        }
    }

    /// The gap is CLOSED in exactly one case: the upstream can replay from at
    /// or below the cursor we already hold.
    #[test]
    fn the_gap_closes_only_when_the_replay_reaches_our_cursor() {
        assert!(
            !reconcile(Some(42), UpstreamReplay::From(42)).possible_gap,
            "a replay floor AT our cursor leaves nothing unproven"
        );
        assert!(
            !reconcile(Some(42), UpstreamReplay::From(7)).possible_gap,
            "a replay floor BELOW our cursor covers more than we need"
        );
    }

    /// The literal condition both sources name: no replay → the gap surfaces.
    #[test]
    fn no_replay_surfaces_the_gap() {
        assert!(reconcile(Some(42), UpstreamReplay::None).possible_gap);
        assert!(reconcile(None, UpstreamReplay::None).possible_gap);
    }

    /// ⛔ The arm the old boolean could not express, and the reason it was
    /// replaced: the upstream OFFERS a replay and still cannot reach back to
    /// our cursor, so the deliveries in between are unprovable. Under
    /// `resume_plan(42, true)` this reported no gap.
    #[test]
    fn a_replay_that_starts_above_our_cursor_still_surfaces_the_gap() {
        let plan = reconcile(Some(42), UpstreamReplay::From(43));
        assert_eq!(plan.resume_from, Some(42));
        assert!(
            plan.possible_gap,
            "a replay floor one above our cursor leaves delivery 43 unprovable"
        );
    }

    /// No own cursor is not a closed gap: we have accepted nothing, so there is
    /// no cursor to compare the upstream's floor against.
    #[test]
    fn an_unstarted_subscription_claims_no_continuity() {
        let plan = reconcile(None, UpstreamReplay::From(0));
        assert_eq!(plan.resume_from, None);
        assert!(plan.possible_gap);
    }

    /// The credential never reaches a log through the derived formatter.
    #[test]
    fn the_upstream_authorization_redacts_its_credential() {
        let auth = UpstreamAuthorization::new("bearer-secret-value".to_string());
        assert_eq!(auth.credential(), "bearer-secret-value");
        let rendered = format!("{auth:?}");
        assert!(
            !rendered.contains("bearer-secret-value"),
            "the credential reached Debug: {rendered}"
        );
        assert_eq!(rendered, "UpstreamAuthorization(<redacted>)");
    }
}
