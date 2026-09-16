---
answers:
  - Does `rb-server` refuse a non-loopback bind, and if not, where is that limit published?
  - Why does `--host 0.0.0.0` bind every interface with no gate?
  - What does the startup line tell an operator about how far a boot reaches?
  - Must every configuration refusal happen before the migrations run?
---
# `rb-server`'s bind address is a documented profile choice, reported at boot and not gated

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.12`
- **Date:** 2026-09-16

## Context

`SIGNOFF-REPAIR.11.12` was opened with two findings that share one shape — the
boot acts before it checks — and its own text says they are **not** the same
defect and must be decided separately.

The first is an ordering: `sqlx::migrate!(…)` ran on the line *before*
`SecretStore::resolve`, whose own comment states the contract — *"an undeclared
profile refuses the boot — never a silent fallback"*. A typo'd profile against
the wrong database left that database migrated and no service running.

The second is the bind: `host` is `#[arg(long, default_value = "127.0.0.1")]`
with no predicate, so `--host 0.0.0.0` binds every interface, and the startup
line printed `rb-server listening on http://{addr} (Phase 0 dev profile)` either
way.

## The ordering: fail-closed, and it is not a decision

There is nothing to weigh. `SecretStore::resolve` is a pure match on a string
and the address parse is a pure parse; neither touches the database. Both now
run before `PgPool::connect`, so a refusal cannot arrive after a mutation.

The control does not need PostgreSQL: it points the server at a port nothing
serves, so a boot that reaches the connection reports `PoolTimedOut` and a boot
that refuses first reports the configuration it refused. One knob — which
argument is wrong — decides which message appears.

## The bind: reported, not refused, and here is why

⛔ **A refusal would break a shipped, documented capability.**
`docs/book/src/deployment.md` names two supported profiles, and the second is
*Trusted LAN* — *"The control plane binds the LAN (`rb-server --host 0.0.0.0 …`);
each node enrolls with a one-time token and connects outbound"*. The
`deploy/` runbook and `scripts/demo_two_host.sh` both walk that path. Gating the
argument on "the dev profile" would refuse the demonstration this repository
ships.

⛔ **And a gate here would be the wrong instrument for the real limit.** What
bounds Internet exposure is G6/G7 and blockers **B1–B3** — an externally
reviewed threat model, a penetration test, and a prompt-injection action-boundary
suite. None of them is a property of an argument, and a bind check would read
like a control over an exposure it cannot govern. The limit is published in
`docs/book/src/blockers.md` and in the Phase-7 gate record, which states the
position exactly: *NOT MET for the Internet exposure — Met as the hardening-
machinery exit for the LAN profile*.

⭐ **What WAS wrong is that the startup line could not tell the two apart.** An
operator reading a log saw `(Phase 0 dev profile)` whether the server was
reachable from one loopback or from every host on the network. The boot now
names its reach — `reachable from: loopback | one named interface | every
interface` — computed by `reasonbraid_server::bind_exposure`, a pure function
with its own unit control over six addresses including both IPv6 forms.

## The rejected alternatives

1. **Refuse a non-loopback bind under the dev profile.** Rejected: it breaks the
   trusted-LAN profile the book documents and the two-host demonstration script.
2. **Add an `--i-understand-this-is-exposed` flag.** Rejected: a flag that every
   LAN deployment must pass is a flag every LAN deployment learns to pass, and it
   would add a second place where the exposure position is stated — the drift
   `INDEX-FRONTIER` exists to stop. The book and the gate record are the one
   place.
3. **Leave the startup line alone and record the decision only here.** Rejected:
   the acceptance asks where the limit is published, and a log line is where an
   operator actually is at the moment the exposure is taken.

## What this decision does NOT say

It does not say a non-loopback bind is safe. It says the bind address is a
deployment profile choice whose limits are published, that the boot now reports
which choice was made, and that the gate on Internet exposure is G6/G7 — which
remains **NOT MET**, with B1–B3 open and outside this repository's reach.
