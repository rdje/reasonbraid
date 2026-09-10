---
answers:
  - How should a partial fixture plan gain missing dependency tables?
  - Why preserve the deployment CA while repairing node-related fixture cleanup?
  - What does the twenty-of-twenty-five fixture-plan census cover?
---
# Add only the explicit dependency closure of the fixture's existing scope

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.3`; REPAIR-0053.
- Evidence: docs/tasks/artifacts/signoff_review/partial-fixture-cleanup.md.

Derive incoming-FK dependencies from the observed schema, confirm them in the
actual unchanged producer/consumer runs, and write the required tables explicitly
at the caller. Preserve the relative order of its original tables and every
feature assertion. The shared checker validates the declared plan; it never
expands deletion scope itself.

Deleting a tenant/role requires its dependent node/incarnation/run/recruitment
rows and their transitive children. That does not justify copying every table
from a broader fixture. These six callers never requested resource, deployment-CA
or unrelated audit deletion. Keep those outside the closure, and verify real CA
row preservation alongside removal of actual predecessor residue.

The source population is the original 25 literal-array DELETE-loop fixtures.
After fourteen earlier migrations and these six, twenty use checked plans; five
remain under .2.7.4. Explicitly bound that census: other direct SQL shapes or
relationships without FKs are not proven covered merely because this population
is reconciled. A clean isolated database is likewise not a predecessor-residue
control, and repeated producer tests are not distinct additional coverage.
