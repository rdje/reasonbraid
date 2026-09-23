-- 0101_origin_delivery_receipts.sql — SIGNOFF-REPAIR.5.3.5.3.3 (ADR-026): a
-- delivery across an ORIGIN execution binding is a cross-domain act, and it
-- leaves a receipt on BOTH sides when the origin node acknowledges it.
--
--   origin_delivery   — the IMPORTING tenant's receipt: its work item
--                       (`local_ref`, the command id) was acknowledged by the
--                       origin tenant's node (`remote_ref`, `ack:{node}:{cursor}`).
--   origin_execution  — the ORIGIN tenant's receipt: its node holds work
--                       (`local_ref`, `{node}:{cursor}`) admitted by the
--                       importing tenant's authorization record (`remote_ref`).
--
-- The receipts cross-reference and never merge (0049): each side's `remote_ref`
-- names a record in the OTHER side's domain.
ALTER TABLE cross_domain_receipts DROP CONSTRAINT cross_domain_receipts_kind_check;
ALTER TABLE cross_domain_receipts ADD CONSTRAINT cross_domain_receipts_kind_check
    CHECK (kind IN ('card_import', 'agreement', 'origin_delivery', 'origin_execution'));
