---
answers:
  - Must the ReasonBraid repository remain public or private?
  - Does public repository visibility establish name or release clearance?
  - How does security disclosure work with a public repository?
---
# The repository is public and must remain public

- Authority: director correction on 2026-09-09.
- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.3`; REPAIR-0043.
- Supersedes: the private-repository requirement in README, ADR-001 and their
  companion records; the visibility blocker from REPAIR-0042.

The director states: “README.md is wrong about the visibility. The project is
public and will, should remain public.” Treat the earlier private instruction
as incorrect documentation. Keep `rdje/reasonbraid` public. Do not request privacy
restoration or change the remote setting to satisfy the superseded wording.
The previous audit's authenticated and unauthenticated GitHub API observations
already agree with this instruction; no historical visibility-change actor or
cause was established or is needed to apply the correction.

Public source visibility is explicitly authorized. Name/package/domain clearance,
license selection, release qualification and deployment security retain their
own gates; they do not require a private source repository. Continue normal
already-authorized pushes after the full local checkpoint and consume remote CI.
The two historical scanner findings remain owned and must be resolved before
claiming the history gate passes.

A public repository cannot provide a confidential embargo through its branches,
commits, issues or pull requests. Report sensitive details privately to the
accountable owner using an established private channel. Arrange a separate
private investigation/workspace before confidential work; public tracking should
contain only information suitable for disclosure. This directive does not create
a reporting endpoint, promise a bounty/CVE service, or authorize making this
repository private. SECURITY.md describes the current process and its limits.

The original visibility audit and interrupted checkpoint remain historical
evidence in docs/tasks/artifacts/signoff_review/publication-precondition.md;
their private-restoration proposal is explicitly superseded. README, ADR,
Kickoff/Phase-0 routes, security/risk/owner records, live docs and the book carry
the corrected policy. Validation and commit receipt are in the owning task leaf.
