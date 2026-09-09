# Semantic introspection for agent-operated diagnosis

Status: proposed after the director's 2026-09-09 discussion; implementation is not
authorized by inference from a request for an opinion. Capture owner:
SIGNOFF-REPAIR.3.3.4.3.3.3.2. Assessment owner: SIGNOFF-REPAIR.6.4.
The active bootstrap recovery work continues without a task-tree pivot.

The engineering recommendation is a typed core/server semantic API, exposed
through MCP, that explains state, decisions and dependencies with verifiable
evidence. Candidate operations: inspect an entity and its version; trace an
operation from admission through execution/commit/delivery/acknowledgment;
explain an authorization decision with its actual boundary/grant/policy inputs;
check named invariants; follow causal/dependency references. Responses should
be bounded and versioned, carry observation time and consistency limitations,
identify missing/unknown evidence, and permit drill-down through source IDs.

Use existing durable facts where available. A graph of relationships alone does
not establish causation. If historical inputs or external effects were not
captured, report that diagnostic/replay limit explicitly. Distinguish measured
facts, inferred hypotheses and verified root causes. Enforce tenant/site scope,
secret redaction, output/work budgets and auditability within the service;
MCP metadata is not an authorization boundary.

Proposed progression: read-only operation tracing, authorization explanations
and invariant checks; isolated replay with qualified input capture; separately
permissioned repair planning/execution with state/version preconditions,
idempotency, outcome receipts and verification. This supports an agent's
observe/hypothesize/reproduce/repair/verify loop, incident review, regression and
upgrade diagnosis, and evidence-based behavior explanations. It cannot promise
that all defects are automatically diagnosable or repairable.

The current bootstrap investigation provides a concrete first scenario: a lost
acknowledgment is not proof of rollback. An inspection must distinguish a
committed result, a proved rollback and an unknown outcome, retaining the
request/operation identity and evidence needed to recover without duplicating a
successful effect. This is a proposed API use case; the server request-key
protocol under implementation is not itself the full introspection surface.

Protocol support checked during the discussion: the official
[MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
defines structured tool results and resource links. This supports the proposed
transport shape; it is not a claim that ReasonBraid implements the design, or a
decision to change its selected MCP protocol profile. The assessment must check
that profile and its actual implementation before adding tools.
