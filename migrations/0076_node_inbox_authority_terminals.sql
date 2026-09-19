-- 0076_node_inbox_authority_terminals.sql — SIGNOFF-REPAIR.11.24.1.1.2:
-- §10.6's `expired` and `revoked` terminals gain the producer they never had,
-- and it is a DERIVATION over shipped facts rather than a new column or an
-- invented ceiling.
--
-- §10.6, in full:
--
--     queued → offered → transport_received → acknowledged → consumed
--                       ↘ expired / revoked / dead_lettered
--
-- ⛔ THE DEFECT THIS CLOSES. A queued command for a node that never comes back
-- was held FOREVER. The only DELETE against `node_inbox` is the operator prune
-- (`authority/node_admin.rs`), whose predicate is `acknowledged_at IS NOT NULL`
-- — so an UNDELIVERED row is unreachable by every removal path the server has,
-- and `PHASE-3.2.2` deferred *the offline-delivery expiry + max age* to a leaf
-- that closed without them (`git grep -ci "max_age|offline_expiry|
-- delivery_expiry" -- crates` → 0 files).
--
-- ⭐ WHAT `expired` AND `revoked` ACTUALLY ARE. §10.6 lists them as a PAIR
-- branching off the ladder, and the pair is the two ways an undelivered
-- command's AUTHORITY can end: by the passage of TIME, and by an ACT. That
-- reading is not this migration's invention — it is the negation of the one
-- predicate this repository already uses to ask whether authority stands,
-- `authority::grant_is_live` (`SIGNOFF-REPAIR.9.3.1`, the leaf that collapsed
-- five different spellings of the question into one):
--
--     status = 'active' AND valid_from <= now() AND expires_at > now()
--
-- Split into its reasons: `status <> 'active'` is the ACT (revocation), and
-- `expires_at <= now()` is the TIME. ⛔ A SIXTH spelling is not introduced
-- here; `valid_from` is omitted only because a grant that already admitted a
-- command had begun by construction, so that leg can never newly become true.
--
-- ⭐ THE LINK IS TOTAL, and that is proved by construction rather than
-- sampled. `node_inbox.authz_ref` is the admitting `authorization_records.
-- record_id` (`migrations/0013`), and that record's `grant_id` is non-null for
-- every ALLOWED decision: `authority/selection.rs` returns `Decision::Allowed`
-- ONLY from the branch that carries `grant: Some(grant)`, while the
-- grant-less `AuthoritySelection::absent()` is `Denied` by construction. A
-- work item is enqueued only on `AuthorizationOutcome::Allowed`, so every
-- dispatched row reaches a grant.
--
-- ⚠️ A row with a NULL `authz_ref` has no authority to end and stays `queued`.
-- Those are plain channel traffic (`NodeChannelState::enqueue`, which carries
-- no admission decision) and pre-0013 rows; a state is derived from a fact,
-- and absent the fact no claim is made. Measured: `enqueue` has no production
-- caller — `git grep -n "\.enqueue(" -- crates/reasonbraid-server/src` → 0.
--
-- ⛔ WHAT WAS REFUSED, and the reason it matters more than what was taken.
-- The tenant's `revocation_epoch` (`migrations/0013`) is on the row and moves
-- on every revocation, so `revoked ⇐ row.revocation_epoch <> tenants.
-- revocation_epoch` was the cheap derivation available. It is WRONG: that
-- comparison is `CachedDecision::is_invalidated`, whose verdict is
-- `CacheVerdict::Stale` — *RE-ASK the authority store* — not a denial. Any
-- revocation anywhere in the tenant moves the epoch, including one that never
-- touched this command's grant. Deriving a TERMINAL from it would publish
-- `revoked` for a command whose authority is intact, which is precisely the
-- defect `migrations/0075` repaired one migration ago: a §10.6 word used for
-- a fact that is not the one §10.6 names.
--
-- ⛔ AND NODE SUSPENSION WAS REFUSED TOO, for a second reason worth keeping.
-- `node_presence` (0017) derives `suspended` from the node's certificates, and
-- 0017 deliberately makes it REVERSIBLE — the replacement ritual issues a
-- fresh active certificate and the node stops being suspended. A terminal
-- derived from a reversible predicate is a terminal a row can LEAVE, and every
-- state this view already derives is monotone (`acknowledged_at` is set under
-- an `IS NULL` guard, `quarantined_at` once, `node_events` is append-only).
-- A grant, by contrast, is named by id: revoking it is permanent for THIS row,
-- because re-issuing authority mints a new `grant_id` the row does not point at.
--
-- ⭐ THE MAX AGE IS DERIVED, NEVER CHOSEN — which `SIGNOFF-REPAIR.11.6`
-- requires and `.11.24.1.1.2` restated. This migration picks no number. The
-- ceiling on how long an undelivered command may wait is the admitting grant's
-- own `expires_at`, a bound the issuing tenant already set, and it is exactly
-- the shape §9.2 asks for: *retention depends on the operation's retry horizon
-- and consequences*. A command cannot outlive the authority that admitted it.
--
-- THE PRECEDENCE, ARGUED BEFORE IT WAS WRITTEN (`.11.24.1.2`'s rule: deciding
-- a precedence chain by implementation order is how it acquires a wrong answer
-- nobody can see). The first draft of this migration put both terminals BELOW
-- the delivery states, on the reading that only an undelivered row can expire.
-- ⛔ THAT DRAFT WAS REFUTED BY THE CURSOR ACK, and the refutation is the reason
-- the order below is what it is:
--
--   `acknowledge` marks EVERY row up to the acked cursor
--   (`UPDATE node_inbox SET acknowledged_at = $3 WHERE cursor <= $2`), including
--   rows the tail withheld. A withheld row at cursor 3 with a delivered row at
--   cursor 4 therefore gains an `acknowledged_at` the moment the node acks 4 —
--   so a terminal placed below `transport_received` is a terminal the row LEAVES
--   without anything having been delivered.
--
--   1. `dead_lettered` stays FIRST. §16.11's preservation rule already outranks
--      everything here — a quarantined row is not prunable and not replayable by
--      age — and its disposition must survive whatever its authority did.
--   2. `consumed` stays SECOND. A `work_result` event is the strongest fact any
--      row carries: the agent acted. Authority ending afterwards does not unmake
--      it, and reporting a completed command as `revoked` would hide the work.
--   3. `revoked`, then `expired` — ABOVE `transport_received`, which is the
--      correction. A command whose grant was revoked while the node held it is
--      §10.6's `revoked`: the node's own dispatch boundary will refuse to run it
--      (the epoch bump invalidates the cached allow and the re-ask is denied), so
--      the work will not happen and the row's honest terminal is the withdrawal.
--      `revoked` precedes `expired` because a grant can be both: the ACT is the
--      operative fact and the more informative answer to an operator asking why
--      a command was never delivered. Time passing is what happens to a grant
--      nobody touched.
--
-- ⚠️ ONE ORDERING CONSEQUENCE, STATED RATHER THAN DISCOVERED LATER. Because
-- `consumed` outranks both terminals, a command the node had already dispatched
-- when its authority was withdrawn reads `revoked` until its result lands and
-- `consumed` afterwards. That is a refinement by later evidence, the same shape
-- `consumed` already has over `transport_received`, and it is the honest answer
-- in both directions: before the result there is no evidence the work happened,
-- and after it there is.
--
-- ⛔ THE FILTER IS PART OF THE REPAIR, not a separate nicety. `replay` (the one
-- function the handshake AND the poll both read the tail through) gains the
-- same predicate. Without it a row could reach `revoked` and then be delivered
-- and acknowledged, moving to `transport_received` — a terminal the row LEFT,
-- which would make this view a liar. The rows are WITHHELD, never dropped, and
-- stay inspectable at `GET /v1/nodes/inbox` and the MCP `list_inbox`: that is
-- `PHASE-3.2.2`'s own offline-KNOWN-versus-unknown distinction one level down —
-- a command that expired while its node was away is not the same as one that
-- never existed, and an operator must be able to tell them apart.
--
-- ⚠️ WIRE-VISIBLE, and stated rather than slipped in: `delivery_state` gains
-- two values a client may not have seen. Pre-1.0, development profile, and the
-- alternative is a field that reports `queued` forever for work that will never
-- move.

CREATE OR REPLACE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.operation_id = i.command_id
                            AND e.payload->>'kind' = 'work_result') THEN 'consumed'
         WHEN EXISTS (SELECT 1 FROM authorization_records r
                        JOIN authority_grants g ON g.grant_id = r.grant_id
                       WHERE r.record_id = i.authz_ref
                         AND g.status <> 'active') THEN 'revoked'
         WHEN EXISTS (SELECT 1 FROM authorization_records r
                        JOIN authority_grants g ON g.grant_id = r.grant_id
                       WHERE r.record_id = i.authz_ref
                         AND g.expires_at <= now()) THEN 'expired'
         WHEN i.acknowledged_at IS NOT NULL THEN 'transport_received'
         ELSE 'queued'
       END AS delivery_state
FROM node_inbox i;
