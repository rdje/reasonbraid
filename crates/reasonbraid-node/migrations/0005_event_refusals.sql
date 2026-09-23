-- 0005_event_refusals.sql — SIGNOFF-REPAIR.4.4.2: the server's refusal of a work
-- result, as the node learned it from the event's receipt.
--
-- The receipt used to say only whether the event id was new, so a result the
-- server refused to fold (authority revoked, thread closed, the role no longer
-- running here) was indistinguishable from one it applied: the node kept its
-- attempt `completed`, its local budget settled, and never learned the work did
-- not land. The refusal is a fact about the EVENT (the attempt did complete;
-- the server declined its result), so it is stored on the event row, as JSON
-- `{"code": …, "message": …}`; NULL means none was reported.
ALTER TABLE outgoing_events ADD COLUMN refusal TEXT;

PRAGMA user_version = 5;
