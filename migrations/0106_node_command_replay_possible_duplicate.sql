-- 0106_node_command_replay_possible_duplicate.sql — SIGNOFF-REPAIR.4.4.7.2.2:
-- the sixteenth administrative operation.
--
-- A replay that also AUTHORIZES the possible duplicate of an `outcome_unknown`
-- attempt (ROADMAP §11.3's third policy, §14.6's only sanctioned retry of an
-- unknown outcome). It is its own kind rather than a flag on
-- `node_command_replay`, so the audit can never read a duplicate-risk decision
-- as a plain replay. The vocabulary mirrors `AdministrativeOperation::KINDS`,
-- and the control that pinned fifteen now pins sixteen (`0094`'s precedent).
ALTER TABLE administrative_effects DROP CONSTRAINT administrative_effects_operation_check;
ALTER TABLE administrative_effects ADD CONSTRAINT administrative_effects_operation_check CHECK (
    COALESCE(
        jsonb_typeof(operation) = 'object'
        AND operation->>'kind' IN (
            'grant_revoke', 'boundary_revoke', 'breaker_arm',
            'breaker_reset', 'node_enroll_token_issue', 'node_revoke',
            'node_command_replay', 'node_command_replay_possible_duplicate',
            'node_command_quarantine', 'node_inbox_prune',
            'node_attempt_adjudicate',
            'profile_card_import', 'capability_claim_attest', 'federation_direction_propose',
            'federation_direction_accept', 'federation_direction_revoke'
        ),
        false
    )
);
