# Architecture decision records

ADRs live here, one file per decision. Copy `TEMPLATE.md`. Status, context,
options, evidence, choice, consequences, and a revisit trigger are required
(`ROADMAP.md` §23). No ADR is accepted merely because the roadmap names a
candidate technology.

| ADR | Title | Status | Leaf |
| --- | --- | --- | --- |
| [001](001-uncleared-working-name.md) | Uncleared working name | accepted (internal only) | `PHASE-0.0.1` |
| [002](002-phase1-scope.md) | Phase 1 scope: GO on the LAN vertical slice, single-agent-default routing | accepted (signed by the accountable owner 2026-09-06) | `PHASE-0.8.2` |
| [004](004-postgresql-aggregate-outbox.md) | PostgreSQL aggregate/event/outbox pattern: locked head, claim-first, one-transaction writes, in-server library | accepted (evidence-gated) | `PHASE-1.1.1` |
| [006](006-node-transport-reconnect.md) | Node transport and reconnect protocol: the authenticated outbound channel, accepted with evidence (promotes the WP3 + `.1.2.2` channel decisions) | accepted (evidence-gated) | `PHASE-2.1.1` |
| [007](007-workload-identity-issuance.md) | Workload identity issuance: a project-local CA with short-lived certificates (the spike measured rcgen + rustls over a real TLS 1.3 handshake; step-ca/SPIRE evaluated, not installed) | accepted (evidence-gated) | `PHASE-2.1.1` |
| [008](008-authorization-engine-and-cached-decisions.md) | Authorization engine and cached-decision semantics: the shipped in-tx evaluator stays (accepted with evidence); the node caches ONLY the admission decisions riding its delivery — 60s freshness, a revocation epoch bump invalidates, the §16.4 fail table governs the unreachable store | accepted (evidence-gated) | `PHASE-2.1.5.1` |
| [009](009-delegated-authority-representation.md) | Delegated authority representation: chain-in-envelope for the dev profile (the plumbing was pre-shaped; the widening invariant is a pure, tested function; the wire form beats a token blob) | accepted (evidence-gated) | `PHASE-2.1.4.1` |
