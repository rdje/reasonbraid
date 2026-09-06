# The two-host demonstration

The WP6 proof: a durable conversation between a human and agent nodes survives
crashes, duplicate delivery, and budget exhaustion — with every acceptance
point asserted by a reproducible script, not by hand.

## Run it

```bash
make demo                          # ephemeral PostgreSQL + all suites + the demo
# or, against any PostgreSQL:
bash scripts/demo_two_host.sh --database-url postgres://postgres@127.0.0.1:55432/reasonbraid_test?sslmode=disable
```

The script builds the four binaries, boots `rb-server`, and drives two `rb-node`
workers (one per agent role) through the real channel. It prints `PASS`/`FAIL`
per acceptance point and exits nonzero on any failure. Evidence lands under
`target/demo/<run-id>/evidence/`: the step `timeline.txt`, the thread
inspections, both node-journal dumps, the ambiguous-attempt record, the
duplicate-delivery receipt, and a `summary.md` mapping every acceptance point
to its evidence.

## The scenario

1. A human enrolls and creates a thread; two agent roles enroll.
2. `thread.invite` dispatches a **work item with a reservation** into the
   invited role's inbox — in the same transaction as the invite event.
3. The node polls, executes the (deterministic fake) adapter, and emits a
   `work_result`; the server folds it into the thread as a contribution through
   the same claim → authorize → validate → apply flow a CLI command rides.
4. **Duplicate transport**: the same event is re-POSTed verbatim → the server
   answers `accepted:false` and the thread keeps exactly one contribution.
5. **Server crash**: `rb-server` is SIGKILLed and restarted on the same store —
   every accepted command survives.
6. The human challenges the contribution; revise work reaches the node's inbox.
7. **Node crash after dispatch**: the revise attempt hangs after its dispatch
   boundary record; the node is SIGKILLed; on restart the attempt recovers
   `outcome_unknown` — bounded, visible (`rb-journal ambiguous`), and NEVER
   silently retried. The challenge stays unresolved.
8. **Budget exhaustion**: a second thread budgets exactly one call; the second
   dispatch is denied at the server (a recorded denial row) and the node's
   budget gate refuses it `failed_before_dispatch` before any provider contact.
9. The human closes thread A: closure preserves the contribution AND the
   unresolved challenge; the audit view reconstructs the whole story.

## Two real hosts

Nodes run locally by default. To place the nodes on a second host:

```bash
bash scripts/demo_two_host.sh --database-url ... \
    --node-host other-host --remote-workdir '~/rb-demo'
```

The node binaries are copied to the remote scratch directory and the journals
live there (node-local state is device-local by design). The bundle's `env.txt`
records which mode ran.

## Honest boundaries (Phase 0)

- The dev profile has no directory: **a node id is the agent role wire id it
  serves** (one node, one role). Workload-identity signatures and the
  authenticated streaming channel are WP7's, not this demo's.
- The demo uses the **deterministic fake adapter** (no tokens). The real-harness
  leg is the env-gated `RB_LIVE_CODEX=1` Codex suite (`PHASE-0.4.2`).
