-- 0094_operator_adjudication.sql — SIGNOFF-REPAIR.11.4.7.2.1.5.5 (ROADMAP §11.3's
-- fourth option): an operator ends an `outcome_unknown` attempt.
--
-- `node_ambiguous_attempts` (migration 0087) recorded what each handshake
-- answered, and the only closures were the server's own receipt (`adjudicated`)
-- or the node settling it itself (`resolved_by_node`). §11.3 names a fourth
-- ending — a human adjudicates — and it had no path: `journal.reconcile` ran
-- only on a receipt, and the operator surface offered no adjudicate action.
--
-- The verdict is RECORDED here by an admitted, audited verb
-- (`POST /v1/admin/nodes/ambiguous-attempts/adjudicate`) and APPLIED by the
-- node at its next handshake, which is when the row closes `adjudicated` with
-- the admission as its evidence. Until then the row stays open and carries the
-- verdict, so an operator can see a decision the node has not yet taken.
--
-- `operator_record` is the admission (`authorization_records.record_id`) the
-- administrative effect is keyed on — the same record every other node
-- administration cites. Not a foreign key: the effect row is, and this column
-- is the evidence string the directive carries.

ALTER TABLE node_ambiguous_attempts
    ADD COLUMN operator_verdict TEXT CHECK (operator_verdict IN ('completed', 'failed_known')),
    ADD COLUMN operator_reason  TEXT,
    ADD COLUMN operator_record  TEXT,
    ADD COLUMN adjudicated_at   TIMESTAMPTZ,
    ADD CONSTRAINT node_ambiguous_attempts_verdict_complete CHECK (
        (operator_verdict IS NULL) = (operator_record IS NULL)
        AND (operator_verdict IS NULL) = (adjudicated_at IS NULL)
        AND (operator_verdict IS NULL) = (operator_reason IS NULL)
    );

-- The fifteenth administrative operation. The vocabulary mirrors
-- `AdministrativeOperation::KINDS`, and the same control that pinned fourteen
-- now pins fifteen.
ALTER TABLE administrative_effects DROP CONSTRAINT administrative_effects_operation_check;
ALTER TABLE administrative_effects ADD CONSTRAINT administrative_effects_operation_check CHECK (
    COALESCE(
        jsonb_typeof(operation) = 'object'
        AND operation->>'kind' IN (
            'grant_revoke', 'boundary_revoke', 'breaker_arm',
            'breaker_reset', 'node_enroll_token_issue', 'node_revoke',
            'node_command_replay', 'node_command_quarantine', 'node_inbox_prune',
            'node_attempt_adjudicate',
            'profile_card_import', 'capability_claim_attest', 'federation_direction_propose',
            'federation_direction_accept', 'federation_direction_revoke'
        ),
        false
    )
);
