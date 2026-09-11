# Verifying a networked agent platform — assessment and proposal

Capture owner: `SIGNOFF-REPAIR.11.5`; raised by the director on 2026-09-11:
"testing in this project will be super important … A network interacting agent
over a LAN or internet. it will not be easy. your take", followed by "I guess
MCP which is already supported by the project … will help tremendously, right?"

This is an assessment and a proposal. Nothing here is implemented, and it
changes no accepted execution baseline. It exists so the reasoning survives the
session that produced it.

## The evidence this rests on

One day of repairs, `REPAIR-0060` through `REPAIR-0068`, produced six defects.
What found each one is the finding:

| Defect | Instrument that found it | A return-value assertion? |
| --- | --- | --- |
| R2 input collision (`.7.3.3.1`) | source read, then a purpose-built overlap probe | no |
| Worker abandoned alive (`.7.3.3.2.2`) | asking the OS whether the pid is still in the process table | no |
| Recreated-schema ACL (`.11.4.3.1.2.14`) | comparing `pg_namespace.nspacl` against a pristine database | no |
| Guard fixture clock collision (`.11.4.3.1.2.17`) | running the suite WITHOUT `--test-threads=1` | no |
| Evidence identifier collisions (`.7.4.1`) | minting 1,000 identifiers and counting distinct ones | no |
| R2 success path uncovered (`.7.3.3.4`) | a census of which pipeline stages any live test executes | no |

Not one was a wrong return value. Every one was identity, lifetime, ordering,
concurrency, or environment — and every one was an ABSENCE or a DUPLICATE, which
is precisely what an assertion over a function's output cannot see.

That is not a coincidence about this particular day. It is what the system is:
ROADMAP §6.4's delivery semantics, §8.6's per-thread ordering, §11.3's provider
ambiguity, §14.6's budget invariants, §12.6's evidence provenance and §16's
authority boundaries are all properties of sequences, processes and identities
over time. A suite built on "call it, assert the output" stays green through
exactly the failures that matter here.

## Four proposals, ranked

### 1. A control must have failed on the defect it was written for

The highest-signal practice of the whole day, and nearly free. Every control
added in `REPAIR-0060`–`0068` was run against the unrepaired code and observed to
fail before being accepted: the abandoned-worker control named a live pid, the
schema-usage control reproduced SQLSTATE 42501, the identifier controls were
measured against 918 and 269 collisions.

Two of today's defects had passed for the project's life precisely because
nothing ever asked whether their control COULD fail — `site_authority` under
`--test-threads=1`, and the R2 success path behind an SSRF refusal. A green
suite proved nothing about either.

Proposed as a doctrine: a leaf that adds a control to repair a defect records the
failing run of that control against the unrepaired source. This verifies a
decision was recorded, not that it was right — the same honest limit
`TASK-ACCEPTANCE` already carries.

### 2. Coverage of paths, not lines

Line coverage would call the R2 pipeline well covered while no live test executes
its success path at all. The useful instrument is a registry of named pipeline
stages — acquire, extract, snapshot, derive, cite, publish, deploy — each bound to
the live test that actually executes it, and a gate when a stage has none.

`.7.3.3.4` is the first instance and already has an owner. The registry
generalises it so the second instance is found by a check rather than by someone
noticing.

### 3. Deterministic simulation is the missing keystone

`reasonbraid-simulator` is in ROADMAP §7.2 and §19.2 and does not exist. For a
LAN or Internet agent network, partition, reorder, duplication and
crash-at-every-boundary coverage cannot be obtained by running real nodes and
hoping to hit the interleaving. It needs a logical clock, an injectable
transport, and invariants asserted over the EVENT LOG rather than over call
returns:

- every committed event has a unique monotonically increasing thread sequence;
- a redelivered command produces exactly one domain effect;
- a dispatched attempt with no recorded result is `outcome_unknown` and is never
  silently retried;
- a fenced lease can neither write nor renew;
- no reservation, no dispatch.

Seeded and replayable, so a failure is a counterexample that can be re-run rather
than a story about a flake. This is where the node-channel protocol
(`docs/book/src/node-channel.md`) gets real coverage; today it has exactly the
kill-point tests its authors thought to write.

### 4. An adversarial concurrency lane

`scripts/run_pg_tests.py:356` appends `--test-threads=1`. That is a legitimate
fixture-protection choice AND a blindfold: it hid a real defect until `make
check` ran the same tests in parallel. The answer is not to remove it — it is a
second lane that deliberately runs concurrently, against fixtures that own their
state well enough to allow it. Today's repairs to the extraction inputs, the
guard fixtures and the schema ACL are exactly the prerequisite work.

## On MCP

The honest answer to "will MCP help tremendously" is: it helps a specific thing a
lot, it is orthogonal to the coverage problem, and today it is a liability rather
than an asset.

**It would have caught none of the six defects above.** MCP is a boundary, not an
instrument. The tools that found them were `ps`, `pg_namespace.nspacl`, a
process-table probe and a distinctness counter. No protocol supplies those,
because the defects were absences, and absences are not return values.

It is also not currently qualified on its own wire. `docs/book/src/qualification-review.md`
records that MCP reads do not uniformly enforce target authority and that listen
dedup needs correction (`.6.1`–`.6.3` open), and `docs/book/src/deployment.md`
states that the MCP producer test "is an internal durable-state test, not
MCP-wire or agent qualification". MCP is untested surface that needs testing.

Where it genuinely earns its place:

- **Real agents, early and cheaply.** ROADMAP §11.6's active-client path lets an
  already-running agent call ReasonBraid tools without ReasonBraid waking a
  harness. Real heterogeneous clients find product- and protocol-level defects
  that no synthetic test will.
- **An external yardstick.** A versioned protocol this project did not write,
  with its own conformance fixtures, is dimensionally different verification —
  what `docs/CLAIM_VERIFICATION.md` asks for and what a self-written suite
  structurally cannot provide.
- **Observability to agents.** The director's `.6.4` semantic-introspection
  proposal would make the system diagnosable by an agent. That is a diagnosis
  lever, which is real, and still not coverage.

One trap to state explicitly: MCP-driven tests bind correctness to an external
SDK and a live transport. ROADMAP §25's risk register carries a stop/reframe
trigger for "core correctness depends on an external protocol's unstable optional
behavior". MCP therefore belongs in a CONFORMANCE lane and never in the
correctness lane — the opposite pole from the deterministic simulator, which is
where correctness has to live.

Short form: MCP widens who can reach the system; it does not deepen what can be
seen inside it. This project's defects are in the seams, and seams need
instruments.

## The constraint any of this must respect

The full checkpoint already takes roughly two hours, with about 2,982 seconds of
`02-check` unaccounted for (`.11.4.3.1.2.15`). Additional lanes must arrive with
a tiering story — focused checks per commit, adversarial and simulation lanes on
a schedule, the full checkpoint before a push — or they will be skipped, which is
worse than not having them. A gate people route around is a gate that lies.

## Status

Proposal only. `SIGNOFF-REPAIR.11.5` owns assessing it against the roadmap's
existing §19 test strategy and deciding which parts become tracked work. No
pivot: the current frontier remains the checkpoint and its repairs.
