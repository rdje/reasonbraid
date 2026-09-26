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

The script builds the project binaries, boots `rb-server`, and drives two `rb-node`
workers (one per agent role) through the real channel. It prints `PASS`/`FAIL`
per acceptance point and exits nonzero on any failure. Evidence lands under
`target/demo/<run-id>/evidence/`: the step `timeline.txt`, the thread
inspections, both node-journal dumps, the ambiguous-attempt record, the
duplicate-delivery receipt, the presence probes, the audit/event/budget
fetches (`audit-a.json`, `events-a.json`, `audit-b.json`, `budget-b.json`),
`channel-auth.txt` (the run's
node ids + dev secrets), and a `summary.md` mapping every acceptance point to
its evidence.

## The scenario

1. A human enrolls and creates a thread; two agent roles enroll.
2. One-time enrollment tokens are issued for both nodes and the nodes **enroll
   with their dev secrets** (`.1.2.1`).
3. `thread.invite` records a **pending invitation** (`.1.3.1` explicit
   participants) — no work yet; the role's **accept** (`rb thread accept`) is
   the transaction that dispatches the **work item with a reservation** into
   its inbox.
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
10. The human contributes a position carrying an **evidence reference**
    (`.1.5.1`) and closes thread A: closure preserves the contribution AND the
    unresolved challenge.
11. **The audit reconstruction** (`.1.8.1`): the supported read surfaces
    rebuild the story without database surgery — A's audit records (invite →
    accept → contribute → close, each with its 64-hex policy digest; the
    create's authority is tenant-scoped), the ordered event timeline (the
    evidence reference rides the human contribution's event), B's audit (the
    close authority), and B's budget ledger (the denied reservation row with
    the engine's reason).
12. **The inspection console** (`.1.6.3`): the same binary that serves the API
    serves the embedded page at `/`; the beat asserts the shell, that the
    served `app.js` names each documented read surface and no write verb, and
    that the page's live same-origin fetch (the dev-profile header, the exact
    endpoints) returns the demo's thread and its budget ledger. Naming a
    surface is a presence check; the stronger checks run the page itself (see
    [How the console is checked](web-ui.md#how-the-console-is-checked)).
    ⚠️ From `SIGNOFF-REPAIR.4.4.2.2` until `.4.4.2.2.1` this beat failed: it
    still looked for the inbox's old `?node=` parameter after the page had
    been corrected to `?node_id=`.

## Two real hosts

Nodes run locally by default. To place the nodes on a second host:

```bash
bash scripts/demo_two_host.sh --database-url ... \
    --node-host other-host --remote-workdir '~/rb-demo'
```

The node binaries are copied to the remote scratch directory and the journals
live there (node-local state is device-local by design). The bundle's `env.txt`
records which mode ran. `psql` must be reachable where the script runs (it is
the fencing-token evidence path). The remote login shell must be bash-compatible,
and a leading `~/` in `--remote-workdir` is the remote home.

⚠️ **Until 2026-09-26 this mode could not have passed** (`SIGNOFF-REPAIR.11.3.5`),
and nothing ran it. Every remote path was wrapped in single quotes, so the
documented `'~/rb-demo'` was never expanded: the directories were made under a
literal `~` while the binaries were copied to the real home. The node was
started from its own subdirectory as `./rb-node`, one level below where the
binary was copied. Its scripted output went to the remote shell unquoted. The
directory was made inside the backgrounded job, so the pid file was written
before the directory existed. And the certificate check looked at the
orchestrator's own disk. All five are corrected, and the mode now passes every
acceptance check, measured with stand-in `ssh` and `scp` that run the remote
side on the same machine in a separate home, using a working directory holding
`~/`, a space and a quote. It has not been run between two real hosts.

**What the run proves about itself.** The demonstration waits for ITS OWN server,
not for any answer on the port: its process must be alive and must have printed
the line `rb-server` writes only after it has bound the port. If another program
already holds the port, the run stops at *server listens* instead of carrying on
against it. The database URL reaches no command line: `rb-server` reads it from
its environment and `psql` from libpq's own variables. The bundle's `env.txt`
records the URL without user information or query.

## Honest boundaries (Phase 1)

- The dev profile has no directory: **a node id is the agent role wire id it
  serves** (one node, one role).
- Since `.1.2.2` the channel identity is the **workload certificate**: the
  enrollment issues a 10-minute leaf (CN = the node id), the node stores it
  beside its journal, and every handshake signs with its key. The transport is
  still plain HTTP/1 (no mTLS claim — ADR-006/007); `channel-auth.txt` records
  the run's dev secrets, which remain the `.6.1` dev stance for enrollment,
  NOT production identity.
- The demo uses the **deterministic fake adapter** (no tokens). The real-harness
  leg is the env-gated `RB_LIVE_CODEX=1` Codex suite (`PHASE-0.4.2`).
- Two live processes for one node id would fence each other (each handshake
  rotates the lease token) — the demo kills before it restarts, which is the
  discipline the fencing contract enforces.
