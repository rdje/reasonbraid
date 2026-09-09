---
answers:
  - How is private-repository publication authorization verified?
  - How should a checkpoint stop when remote visibility contradicts policy?
  - How are historical secret-scan findings kept distinct from credential conclusions?
---
# Verify actual remote visibility before publication

Director correction, 2026-09-09: the repository is public and must remain public.
The old README private instruction was wrong. The visibility blocker described
below is resolved by docs/decisions/2026-09-09_public-repository-policy.md; no
setting change is needed or authorized. The historical audit and gate results
remain evidence of what was inspected and stopped, not current publication policy.

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.1`; REPAIR-0042.
- Evidence: docs/tasks/artifacts/signoff_review/publication-precondition.md.

A local instruction to keep a repository private does not prove its remote
visibility. Before disclosing unpublished commits, bind the target to its exact
remote identity and inspect current visibility. Here both authenticated GitHub
metadata and an unauthenticated official API response report public visibility,
contradicting README/ADR-001. Neither a successful Git transport nor an earlier
memory statement establishes private access.

Stop publication and obtain the director's decision on the concrete visibility
conflict. Do not silently change repository visibility, treat the public state as
permission to publish, or infer who changed it from generic API timestamps. Fix
current-state documentation while preserving the original policy and historical
provenance. A later private setting does not undo past public access.

If a real policy blocker stops a checkpoint, supervise and consume the active gate
before handoff. Record interruption separately from pass or test failure; later
gates remain unexecuted. Commit the completed audit and exact resume pointers.
Use a new receipt directory for the resumed source, preserving the failed/partial
results and requiring the full gate before push.

A redacted history match is a scanner finding, not automatically an issued leaked
credential or a proved false positive. Give each finding concrete repair ownership,
trace fixture/provenance/use before a narrow exclusion, and preserve detection
controls. Do not bypass entire rules or rewrite history as an unreviewed remedy.
