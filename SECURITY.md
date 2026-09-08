# Security policy

The disclosure + the supported-version policy (`PHASE-7.2.2`, ROADMAP
§16.10's "publish a vulnerability disclosure and supported-version policy
before public beta" line). Honest limits are stated, never hidden: the
repository is PRIVATE (the working-name gate, ADR-001) and the software is
unreleased — the public channels below exist as the vocabulary that binds
the moment the public beta opens.

## Reporting a vulnerability

- Report directly to the **accountable owner** (the release + security
  gates' owner — `docs/decisions/2026-09-06_accountable-owners.md`). The
  repo has no public channel yet; do not open an issue.
- A useful report names: the affected surface, the steps that reproduce
  it, and the impact. The project's claim-verification discipline applies
  to reports too: every report is verified by RE-DERIVATION (the repro
  must run) before any fix is built — never trusted on its wording.
- The report is treated as confidential until the resolution.

## Vetting and embargo

- **Triage** = reproduce → scope → own. The fix lands with its task-tree
  leaf, its regression test, and its disclosure note — the audit trail
  the commit workflow already demands (no silent fixes).
- **Embargo**: the resolution rides the private repo until the
  coordinated disclosure. A public disclosure happens with the release
  notes of the release that carries the fix — once public releases exist.
  Until then the embargo is "the fix ships with its evidence; nothing is
  announced because nothing is public".

## Supported versions

- The **dev line** (unreleased, `0.1.0`) is the only line. No released
  versions exist, so no historical line is supported — nothing is.
- The supported-version window (**the latest release + the previous
  minor**) begins at the FIRST public release, together with the
  disclosure channel and the CVE pipeline.

## Honest limits

- No bounty program (the repo is private and unfunded).
- No CVE pipeline yet — the public-beta trigger opens it.
- The policy above is the process record; the enforcement is the
  audit-trail discipline the hooks already run.
