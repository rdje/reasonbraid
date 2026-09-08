# Phase 7 SubtractionRecord (§19.8) (`PHASE-7.5.2`)

The Phase 7 exit gate's subtraction record: what the phase actively
did NOT build, each with its reason and its revisit trigger. No empty
lists — every entry is a named not-built thing.

| # | Not built | Why (the honest reason) | Revisit trigger |
| --- | --- | --- | --- |
| S-1 | The Internet exposure (the public enrollment endpoint, the mTLS serve wiring, the public listener) | the §25.1 kill/pivot: the qualification gate is incomplete (the three external gaps) — the `.2.4` contract fixed the policy, not the endpoint | the three gaps close (the review + the prompt-injection suite + the pen-test) |
| S-2 | The externally reviewed threat model | the external review is an outside-the-repo activity; the internal vocabulary (ADR-034) ships | the external reviewer engages (the pen-test's sibling) |
| S-3 | The prompt-injection action-boundary suite | the §19.4 deferral's named surface — no action-bearing exposed surface exists to test against | the exposure profile's first action-bearing surface |
| S-4 | The penetration test | the external test runs against the qualified-profile candidate; the remediation rides its findings (the `.4.3` stance) | the exposure profile's candidate freeze |
| S-5 | The RLS-bound app role (the non-superuser connection) | the dev profile connects as the postgres superuser (RLS bypasses superusers unconditionally — the `.1.3.1` record); the probe-role proof measures the layer as it will bind | the deployment profile that binds the layer (the `.1.4` serve wiring) |
| S-6 | The external secret stores (vault/KMS profiles) | the `dev_database` profile ships (the declared stance); an integration against no real store would be a sham | a deployment names an external store |
| S-7 | A confidential-qualified evaluator profile | the dev registry qualifies `general` only — the confidential dispatch is the typed refusal (the `.1.4.3` control) | a provider qualifies for the confidential class |
| S-8 | The horizontal coordinator scaling | the `.3` criteria: zero extractions until a measurement names a bottleneck (the single-writer is the deliberate ADR-002 design) | the `.4.1` harness (or a bigger one) names a seam past the target profile's bound |
| S-9 | The dependency-graph SBOM (SPDX/CycloneDX) | the artifact-level manifest (the signed digests) ships; the graph generator needs the tool census | the generator-tool census passes the supply-chain gate |
| S-10 | The archive/export evidence disposition (the evidence-preserving export) | the quarantine-preserving rule ships; the export disposition (moving the evidence, never deleting it) awaits an export surface | any export-capable surface |
| S-11 | The multi-region storage + the regional controls | no placement machinery exists — the region control has no decision point (the `.1.4.1` deferral) | any multi-region storage |
| S-12 | The multi-node churn game day + the human tabletop | the dev profile is single-node, single-operator — the scale/human exercises are impossible to run honestly (the `.4.3` gaps) | the public-enrollment profile; the first incident |
