# Security policy

This repository is public and must remain public, as the director clarified on
2026-09-09 (docs/decisions/2026-09-09_public-repository-policy.md). The software
remains unreleased. Public source availability does not establish production
qualification or provide a confidential disclosure channel.

## Reporting a vulnerability

- Report privately to the **accountable owner**, Richard DJE; role authority is
  recorded in `docs/decisions/2026-09-06_accountable-owners.md`. Use an established
  private contact channel. Do not put sensitive details in public issues, commits,
  pull requests or logs. If a channel is not established, arrange one with the
  owner before sharing confidential details; this policy does not assert a
  dedicated reporting endpoint exists.
- A useful report names the affected surface, reproduction steps and impact.
  Triage verifies the evidence and assigns a task-tree owner; confidentiality
  applies while the owner coordinates the response.

## Vetting and coordinated disclosure

- Reproduce, scope and own the defect. Track the repair and its verification;
  keep public tracking limited to information suitable for public disclosure.
- Public repository branches and commits cannot provide an embargo. Confidential
  investigation and pre-disclosure fixes require a separately arranged private
  channel or workspace approved by the accountable owner. Do not change this
  repository's visibility to create one.
- The accountable owner coordinates when details and fixes become public.
  Publish the authorized fix, regression evidence and disclosure note together;
  do not promise that already published information can be made confidential.

## Supported versions

- The unreleased `0.1.0` development line is the only line; no released versions
  or historical support commitments exist.
- The planned supported-version window is the latest release plus the previous
  minor, beginning with the first public release. Repository visibility alone
  does not start that window or establish a release date.

## Limits

- No bounty program or CVE pipeline is established.
- A dedicated reporting channel, coordinated-disclosure operation and the
  supported-version process still require release qualification under
  ROADMAP §16.10 and the existing Phase-7/G9 owners.
