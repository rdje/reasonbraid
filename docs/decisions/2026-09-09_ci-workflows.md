---
answers:
  - Which checks and prerequisites must the CI workflows execute explicitly?
  - Does verified workflow wiring prove a successful remote CI run?
  - What evidence qualifies retaining Gitleaks reports in CI?
---
# Require the complete checkpoint commands at the CI boundary

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.3`; REPAIR-0036.
- Evidence: docs/tasks/artifacts/signoff_review/ci-workflows.md.

Route every project command through the local CI environment before installation
or execution. Require the pinned compiler where needed; build/check worker binaries
and require Chrome before Rust runtime tests. Discover every Python control module,
validate actual PostgreSQL 16 tools and request the full owned demo explicitly.
Install mdBook 0.5.4 into local stores and check its version. Run the pinned scanner
drivers in actual gate mode. Keep permissions read-only, job lifetimes bounded and
history complete where doctrine/secret checks need it. Installed runner OS tools
and GitHub-managed checkout/artifact transport are explicit platform dependencies;
project-owned stores/output/temp remain checkout-derived.

Retain only scanner receipts, version/check logs and the redacted Gitleaks JSON
report. Before wiring upload, a real hash-verified Gitleaks process must detect a
synthetic finding and show that its value is absent from both complete output and
report. The initial alphabet example hit the built-in stopword filter; investigate
that zero-finding result and retain it. Correct the fixture, without changing
scanner policy: the second probe returns one redacted finding and rc=1, as expected.
This evidence qualifies the exercised report path, not every possible metadata
content or the actual repository's secret posture.

YAML/shell parsing, explicit coverage/refusal controls and fifty passing Python
controls establish local wiring evidence. They do not establish remote runtime,
real compiler installation or full security gates. Publisher/browser lifetimes and
safe artifact disposition remain prerequisites to the full local checkpoint;
GitHub outcomes must be consumed after the authorized push. Product qualification
categories and external release gates remain unchanged.
