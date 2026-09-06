# The two-host demonstration

The WP6 proof: a durable conversation between a human and agent nodes survives
crashes, duplicate delivery, and budget exhaustion — with every acceptance
point asserted by a reproducible script, not by hand. Since `.1.2.2` the whole
scenario rides the **authenticated channel**: enrolled nodes, key-proof
handshakes, leases, and observable presence.

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
duplicate-delivery receipt, the presence probes, `channel-auth.txt` (the run's
node ids + dev secrets), and a `summary.md` mapping every acceptance point to
its evidence.

## The scenario

1. A human enrolls and creates a thread; two agent roles enroll.
2. One-time enrollment tokens are issued for both nodes and the nodes **enroll
   with their dev secrets** (`.1.2.1`).
3. `thread.invite` dispatches a **work item with a reservation** into the
   invited role's inbox — in the same transaction as the invite event.
4. The node polls, executes the (deterministic fake) adapter, and emits a
   `work_result`; the server folds it into the thread as a contribution through
   the same claim → authorize → validate → apply flow a CLI command rides.
   The node's presence is observably **online** through the channel API.
5. **Duplicate transport**: the same event is re-POSTed verbatim (with the
   node's live fencing token) → the server answers `accepted:false` and the
   thread keeps exactly one contribution.
6. **Server crash**: `rb-server` is SIGKILLed and restarted on the same store —
   every accepted command survives, and the durable lease + presence survive
   with them (the node re-handshakes, rotating its fencing token).
7. The human challenges the contribution; revise work reaches the node's inbox.
8. **Node crash after dispatch**: the revise attempt hangs after its dispatch
   boundary record; the node is SIGKILLed; on restart the attempt recovers
   `outcome_unknown` — bounded, visible (`rb-journal ambiguous`), and NEVER
   silently retried. The challenge stays unresolved.
9. **Budget exhaustion**: a second thread budgets exactly one call; the second
   dispatch is denied at the server (a recorded denial row) and the node's
   budget gate refuses it `failed_before_dispatch` before any provider contact.
10. The human closes thread A: closure preserves the contribution AND the
    unresolved challenge; the audit view reconstructs the whole story.

## Two real hosts

Nodes run locally by default. To place the nodes on a second host:

```bash
bash scripts/demo_two_host.sh --database-url ... \
    --node-host other-host --remote-workdir '~/rb-demo'
```

The node binaries are copied to the remote scratch directory and the journals
live there (node-local state is device-local by design). The bundle's `env.txt`
records which mode ran. `psql` must be reachable where the script runs (it is
the fencing-token evidence path).

## Honest boundaries (Phase 1)

- The dev profile has no directory: **a node id is the agent role wire id it
  serves** (one node, one role).
- The channel credential is the dev shared secret (server as trust store) with
  the HMAC key-proof handshake; X.509/mTLS workload identity is ADR-006/
  ADR-007 (Phase 2). `channel-auth.txt` records the run's dev secrets — that is
  the `.6.1` dev stance, NOT production identity.
- The demo uses the **deterministic fake adapter** (no tokens). The real-harness
  leg is the env-gated `RB_LIVE_CODEX=1` Codex suite (`PHASE-0.4.2`).
- Two live processes for one node id would fence each other (each handshake
  rotates the lease token) — the demo kills before it restarts, which is the
  discipline the fencing contract enforces.
