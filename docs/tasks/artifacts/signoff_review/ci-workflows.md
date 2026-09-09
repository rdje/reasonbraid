# Complete CI workflow routing

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.3`; REPAIR-0036.
- Baseline: f07e214; three workflow files change. Product Rust and launcher/scanner implementations do not change.
- Evidence directory: target/ci-workflow-controls/wiring. Local source/command controls are complete; full local checkpoint and actual GitHub execution remain .11.4.3.1.2 after publisher/browser/cleanup prerequisites.

## Commands and ownership

All six project command jobs enter scripts/ci_env.py before their command or
installer. Rust jobs request the exact local compiler with --rust. All jobs use
Ubuntu 24.04, explicit time limits, contents: read and checkout without persisted
credentials. Doctrine and secret-history jobs fetch full history. Rust compilation
uses two build jobs and disables incremental compilation in CI only.

| Job | Required command scope |
| --- | --- |
| rust/check | strict shell; required installed Chrome; format; all-target/all-feature warning-denying lint; locked workspace binary build; executable extraction/browser workers; locked workspace tests with visible output |
| rust/pg-tests | runtime pg_config bindir and existing PostgreSQL-16 executable validation; all Python control modules; entire owned runner collection with explicit --demo |
| rust/book | pinned mdBook 0.5.4 locked installation into .project-data/cargo using target/ci-mdbook-build; exact version check; book build |
| doctrines/enforce | existing full doctrine script through local environment |
| supply-chain/cargo-deny | verified pinned scanner driver's actual dependency gate, not --verify-only |
| supply-chain/secret-scan | verified pinned Gitleaks driver's actual redacted history gate, not --verify-only |

Required worker/browser presence prevents those absence paths being mistaken for
coverage. Actual test logs still need inspection; this does not qualify browser
lifetime or ignored provider tests. pg-tests creates its own cluster rather than
starting the image's shared PostgreSQL service. Python discovers every test_*.py
module, including actual cluster/process controls; there are currently six modules
and fifty tests. No provider authorization or release-gate advancement is added.

The official [Ubuntu 24.04 runner inventory](https://raw.githubusercontent.com/actions/runner-images/main/images/ubuntu/Ubuntu2404-Readme.md)
observed image 20260831.293.1 with Python 3.12.3, PostgreSQL 16.15, Chrome
152.0.7977.64, rustup 1.29.0, curl and jq available. Image inventory is a
prerequisite observation, not execution evidence; workflows check actual binary
inputs at runtime. Installed runner tools and operating-system dependencies are
necessary read-only inputs. Checkout and artifact transport use GitHub's managed
runner infrastructure; its lifecycle metadata is a required platform boundary.
Project command installations, caches, temporary data and reports derive from the
current checkout. No workflow writes GitHub environment files or installs global
tools. The launcher is not a general filesystem sandbox.

Scanner evidence upload uses a narrow allowlist: scanner.json, version.log and
check.log for both jobs, plus gitleaks.json for the secret job. Upload runs after
success or failure with repository-derived TMPDIR/TMP/TEMP. No downloaded archive
or executable is selected. Missing evidence warns while the failed command remains
failed. A retained receipt's completed state does not imply a passing exit_code.

## Real redaction and calibrated synthetic input

Before adding report upload, execute the retained verified Gitleaks binary from
target/ci-scanners/gitleaks-5issw4mh read-only; binary SHA-256 is
ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f.
Create an exclusive synthetic Git repository with its own configuration/hooks under
target/ci-workflow-controls/wiring/redaction-uonk3osa. No actual credentials or
project history are used.

The initial deliberately unissued alphabet-based sample returned rc=0 with zero
findings. Stop and inspect the result rather than treating it as a redaction pass.
The pinned [Gitleaks configuration](https://raw.githubusercontent.com/gitleaks/gitleaks/v8.30.1/config/gitleaks.toml)
contains the lowercase alphabet in its global example stopwords; the first sample
contains that sequence case-insensitively. Keep the zero-finding report and log.
A second commit uses a deliberately non-working value derived from a public
synthetic seed's SHA-256, avoiding that example filter without changing scanner
policy. This is fixture calibration, not a demonstrated project scanner defect.

The actual detect command uses --redact=100, --no-color, JSON report output and a
thirty-second bound. It returns rc=1, exactly one github-pat finding across the
two-commit fixture. The synthetic token is absent from both entire JSON report and
console output; every Secret field equals REDACTED. The process result is consumed.
Copy and verify the initial/final reports into redaction-initial-report.json and
redaction-report.json, then remove only the successful exclusive fixture; verify
absence. redaction-verification.json, diagnostic/final logs and both reports retain
the observation. This proves this real fixture's redaction, not a complete project
secret scan or freedom from secrets in all diagnostic metadata.

## Verification

- Installed Ruby 4.0.6/Psych parses all three actual YAML files; bash -n accepts all six extracted command blocks. actionlint and Python yaml are absent, so no result from either is claimed.
- Wrapped check_routes.py validates trigger/permission/job bounds, local launcher entry, compiler selection, strict Rust sequence, required browser/workers, all Python discovery, full demo, pinned book, actual scanner gates, full history and artifact allowlists. Five in-memory omissions (ambient command, missing Python, optional demo, missing worker build, scanner setup only) are all refused. routing-verification.json retains exact source hashes; rc=0.
- `python3 -B scripts/project_env.py python3 -B scripts/ci_env.py -- python3 -B -m unittest discover -s scripts/tests -p 'test_*.py' -v` passes all 50 tests in 14.542s, rc=0, including four actual PostgreSQL controls. python-controls.log/.exit retain the output. This exercises the real no-compiler CI command boundary; instrumented installer controls are not a real CI compiler download.
- make book and final nine rendered markers, seven unchanged adjacent sources and empty owned fixture-directory checks pass, rc=0. The first flag marker exposed smart punctuation replacing --demo; a code span preserves the literal option in the final HTML. final-verification.json retains this correction and exact residue census. Full Rust/PG collection, fresh dependency gate, project history scan and actual GitHub run are intentionally separate checkpoint evidence, not counted as passed here.

Commit acceptance and continuation: docs/tasks/SIGNOFF-REPAIR.md. Qualification
categories remain unchanged. Complete the publisher/browser/cleanup prerequisites,
then run the full checkpoint and consume its results before the authorized push.
