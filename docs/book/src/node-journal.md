# The node journal

Every ReasonBraid node keeps a durable local journal in SQLite
(`ROADMAP.md` §11.4, §17.1, §17.4). The journal owns **locally observed facts** —
received commands, local operations, provider attempts, emitted events, and
acknowledgements — while the PostgreSQL control plane owns desired commands and
delivery attempts. This chapter describes the Phase 0 journal: what it records,
the durability it enforces, how a crash is recovered honestly, and how operators
inspect it.

## What it records

| Record | Purpose |
| --- | --- |
| `commands` | A command received from the control plane, deduplicated by command id (transport redelivery records nothing new) |
| `operations` | The local execution unit; keyed 1:1 on the command, so a duplicated command never creates a second local operation |
| `attempts` | One supervised provider attempt and its state (`prepared` → `dispatched` → `completed`/`failed_known`/`outcome_unknown` → …) |
| `attempt_transitions` | The before/after boundary ledger: one row per state change, in the same transaction |
| `outgoing_events` | Events the node emitted, with acknowledgement state and the server cursor at acknowledgement |

## Durability

The journal runs SQLite in **WAL** mode with **`synchronous=FULL`** — the
conservative development profile (`ROADMAP.md` §11.4: WAL alone is not a
power-loss guarantee). The profile is verified on the live connection at open and
recorded in the journal's `journal_meta` table, so an operator can always see
which durability the writer actually enforces (connection pragmas are otherwise
invisible to other readers). `PRAGMA quick_check` is surfaced by the inspection
CLI for integrity checks.

## Honest crash recovery

The rule is **boundary-before-boundary**: a fact is persisted before the
corresponding boundary is crossed. The critical boundary is the provider
dispatch — `record_dispatch` commits `prepared → dispatched` *before* the
adapter is invoked.

After a crash, recovery has exactly two honest answers:

- `prepared` — the dispatch boundary was **never crossed**: the attempt is
  reported `safe_to_redeliver`.
- `dispatched` with no recorded result — the provider **may** have been
  contacted: the attempt becomes `outcome_unknown`. The node does not guess and
  does not retry silently.

The only ways out of `outcome_unknown` are:

- **Proof** — the adapter proves the result (e.g. a provider status lookup),
  landing the attempt on `completed` or `failed_known` with the evidence attached
  (`ROADMAP.md` §11.3 provider-lookup recovery).
- **Adjudication** — an authorized reconciliation records `reconciled`.

## Inspecting the journal

The read-only `rb-journal` CLI inspects a journal without opening SQLite by
hand (it opens the file with SQLite's read-only flag, so inspection can never
mutate the journal, and WAL mode lets it run beside a live node):

```text
$ rb-journal inspect node.db
journal: node.db
journal_mode: wal (recorded profile)
synchronous: FULL (recorded profile)
foreign_keys: 1
busy_timeout_ms: 5000
schema user_version: 2
quick_check: ok
commands: 3 · operations: 3
attempts: prepared=1 dispatched=0 completed=1 failed_before_dispatch=0 failed_known=0 outcome_unknown=1 reconciled=0
outgoing events: emitted=2 acked=1
```

Pending work and ambiguous attempts have their own views:

```text
$ rb-journal pending node.db
pending attempts: 2
  patt_...  prepared  op ...
  patt_...  dispatched  op ...  provider_request_id=prv_...

$ rb-journal ambiguous node.db
ambiguous attempts: 1
  patt_...  outcome_unknown  op ...
    provider_request_id=prv_...
    evidence={"reason":"crash recovery: dispatched with no recorded result"}
    prepared -> dispatched
    dispatched -> outcome_unknown
```

The boundary history is the evidence an operator needs to decide between a
status lookup (proof) and adjudication. Both list views also support `--json`
for scripting.

## Honest limits (Phase 0)

- This is a development profile: disk-full behavior, checkpoint policy,
  encryption, and retention are documented defaults, not production policy —
  Phase 2's retry policy and the WP8 decision set own those.
- The outbound channel, cursor resume, and the reconciliation handshake build
  on this journal (`PHASE-0.3.2`); the journal itself stores the ack cursor as
  reconnect evidence.
