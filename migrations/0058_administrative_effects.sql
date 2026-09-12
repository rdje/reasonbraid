-- SIGNOFF-REPAIR.3.3.4.7.2: what an admitted administrative request finally DID.
--
-- An authorization record says a caller was allowed to ASK. This says what the
-- local mutation then did, and the two are separate rows on purpose: collapsing
-- them would let `allowed` be read as `applied`.
--
-- Additive in the strict sense. No existing table, column, constraint or row is
-- changed except for one added UNIQUE index that makes the composite foreign key
-- below expressible, and NOTHING is backfilled: a request that predates this
-- table simply has no effect row, which means the outcome was never recorded and
-- never that an operation succeeded or failed.

-- The composite parent key. `record_id` is already the primary key, so this adds
-- no constraint on existing data; it exists so an effect can be bound to BOTH its
-- admission and that admission's tenant in one reference.
CREATE UNIQUE INDEX authorization_records_record_tenant_key
    ON authorization_records (record_id, tenant_id);

CREATE TABLE administrative_effects (
    -- The admission's own id IS the effect's identity: at most one final effect
    -- per admitted request, and an effect with no admission is unrepresentable.
    record_id TEXT COLLATE "C" PRIMARY KEY,
    tenant_id TEXT COLLATE "C" NOT NULL,
    -- The operation and its tenant-bound target, together. The vocabulary
    -- mirrors `AdministrativeOperation::KINDS`; a control asserts the two agree,
    -- so a variant added without its constraint cannot pass silently.
    operation JSONB NOT NULL CHECK (
        COALESCE(
            jsonb_typeof(operation) = 'object'
            AND operation->>'kind' IN (
                'grant_revoke', 'boundary_revoke', 'breaker_arm',
                'breaker_reset', 'node_enroll_token_issue', 'node_revoke',
                'node_command_replay', 'node_command_quarantine', 'node_inbox_prune',
                'profile_card_import', 'capability_claim_attest', 'federation_direction_propose',
                'federation_direction_accept', 'federation_direction_revoke'
            ),
            false
        )
    ),
    -- What the caller wrote, when the operation takes a reason at all. NULL is
    -- the explicit "this operation takes none"; the bounds match
    -- `AdministrativeReason`, which is also the site-authority reason contract.
    submitted_reason TEXT CHECK (
        submitted_reason IS NULL OR octet_length(submitted_reason) BETWEEN 1 AND 1024
    ),
    -- Only `applied` asserts a protected change. `no_op` and `refused` both
    -- assert that protected state and the revocation epoch are unchanged.
    outcome JSONB NOT NULL CHECK (
        COALESCE(
            jsonb_typeof(outcome) = 'object'
            AND outcome->>'kind' IN ('applied', 'no_op', 'refused'),
            false
        )
    ),
    effected_at TIMESTAMPTZ NOT NULL,
    -- An effect cannot cite another tenant's admission. This is structural
    -- rather than a decoder check, because a decoder runs only on the way out.
    FOREIGN KEY (record_id, tenant_id)
        REFERENCES authorization_records (record_id, tenant_id)
);

COMMENT ON TABLE administrative_effects IS
    'The final local outcome of one admitted administrative request. Absence means the outcome was never recorded, never that an operation succeeded or failed.';
