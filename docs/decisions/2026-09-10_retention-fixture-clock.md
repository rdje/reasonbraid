---
answers:
  - How should retention tests choose expiry times?
  - Does same-content replay reset a snapshot's retention age?
  - Does repairing the retention fixture qualify production clock and scope policy?
---
# Derive expiry tests from the persisted creation instant

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.9`; REPAIR-0052.
- Evidence: docs/tasks/artifacts/signoff_review/retention-fixture-clock.md.

Observe the timestamp used by the implementation and drive the fixture relative
to it. A date that happened to lie in the future when a test was written is not
a durable expiry oracle. The unchanged original test and independent database
creation/cutoff predicates demonstrate this defect without changing production.

For the existing strict retention predicate, exact TTL equality is not expiry.
Use before/equal/after observations, exact class-specific counts and unchanged
rows on no-effect/repeated steps. Include an audit-class snapshot explicitly;
a comment that audit remains cannot substitute for creating and observing one.

Same-content replay currently updates refreshed_at but preserves created_at;
retention age remains anchored to original creation. Keep that distinction from
freshness-horizon policy, whose broader repair remains SIGNOFF-REPAIR.7.4 along
with expiry authorization/scope, caller-clock restrictions and object integrity.
This fixture-only correction neither changes nor qualifies those policies.
