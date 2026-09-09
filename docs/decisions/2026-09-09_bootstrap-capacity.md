---
answers:
  - Why must bootstrap completion capacity be checked before sending HTTP?
  - How does the CLI size an unknown bootstrap outcome without inventing a durable receipt?
  - Does the completion-capacity check reserve physical disk space or eliminate publication errors?
---
# Validate the complete bootstrap snapshot before dispatch

- Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1`; REPAIR-0032.
- Status: exact-boundary runtime defect repaired; thirty-one selected controls, final three-scenario matrix and all-target CLI strict lint pass. Every result consumed and unique fixtures absent.
- Evidence: docs/tasks/artifacts/signoff_review/bootstrap-capacity.md.

A bounded pending record can fit while its later principal/completion snapshot
cannot. The real CLI baseline dispatches HTTP when completion is exactly one
byte over the 8 MiB limit, then returns a local encoding error. The adjacent
exact-limit completion succeeds. Original maps and the pending key survive, but
that does not make the completion fit. This is a missing pre-dispatch capacity
check, not a defect in the state codec's existing bound.

Under the existing writer guard, build an in-memory completion snapshot and pass
it through the actual state codec before publishing pending or sending HTTP.
Include all preserved maps, the exact request/name/actions and both the pending
request and completed receipt. The same completion installer builds the real
post-response snapshot; an independent fixture builds its expected shape and
checks both sides of the exact byte boundary. The later initial pending snapshot
still goes through normal bounded publication before dispatch, including any
larger prior completed receipt it retains.

For an unknown outcome, use an internal sizing sample with canonical typed human
and tenant IDs made from nil UUIDs. The current core Id Display contract is the
family prefix plus a hyphenated UUID: all admitted canonical source IDs have the
same escaped JSON width, regardless of UUID version or numeric value. The strict
outcome validator fixes kind, name and request key, and binds boundary/grant
strings to those source IDs. All other strings are already saved request input.
Use false, the longer serialized boolean. The codec validates this sample too;
no magic byte allowance substitutes for actual snapshot encoding. A known local
receipt uses its exact outcome instead of a sizing sample.

Sizing IDs are not issued identities. The sample and its encoded bytes remain
private memory, are discarded, and never reach publication, HTTP or user output.
Only the actual checked server outcome or previously validated historical receipt
can enter real completion publication. Source-prefix/record changes must retain
this correspondence; validation and the exact-boundary controls guard it.

A capacity refusal preserves the original snapshot and any existing pending
identity, sends no HTTP and releases local exclusion. A fitting invocation keeps
normal durable dispatch/completion behavior. This proves fit within the encoded
8 MiB limit, not reservation of physical disk capacity or immunity from later
filesystem, synchronization, process or server failures. Those failures retain
the existing recovery identity and honest uncertainty rules. Transport/reply
bounds and broader restart qualification keep their following owners.
