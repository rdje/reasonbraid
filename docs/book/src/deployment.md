# Deployment — local and LAN

The Phase 1 deployment package (`.1.7.2`) is **four self-contained binaries**
and a runbook. This chapter is the product view; the operator surface is the
`deploy/README.md` runbook in the repository.

## Repository-local development and verification

Development now requires Python 3.11+ and an installed toolchain matching
`rust-toolchain.toml`. Makefile build/check/dev/book commands use
`scripts/project_env.py` to derive writable stores from the current repository.
Build output stays in `target/`; Cargo packages, temporary files, XDG data and CLI
state live under ignored `.project-data/`. A symlink or volume escape in those
store paths is refused. Moving the checkout does not require editing saved paths.

Inspect the selected paths or run a focused command:

```bash
python3 -B scripts/project_env.py --print
python3 -B scripts/project_env.py cargo test --locked --offline -p reasonbraid-core
make book
```

To initialize a local Cargo cache from a complete existing cache:

```bash
python3 -B scripts/project_env.py --seed-cargo-cache "$HOME/.cargo"
```

That explicit source is read-only. Locked archives are hash-checked before use;
credentials and global Cargo configuration are not copied. Shared source data is
retained. If the source is incomplete, use the launcher with `cargo fetch --locked`
when network access is available. Installed compiler, Python, database and mdBook
binaries remain documented read-only toolchain inputs.

Invoke direct diagnostic scripts through the launcher too. The PostgreSQL
runner enters it automatically; see the focused verification commands below.
The launcher controls default stores; explicit output paths and worker-specific
storage remain subject to the same-volume policy. It is not a filesystem sandbox.
Details and verification: `docs/decisions/2026-09-09_repository-local-command-environment.md`.

## Focused PostgreSQL verification

```bash
bash scripts/run_pg_tests.sh --list
bash scripts/run_pg_tests.sh authority command_api
bash scripts/run_pg_tests.sh site_registry_http allowlist regions
# Broad collection, without the demonstration:
RB_DEMO=0 bash scripts/run_pg_tests.sh
# Add the demonstration to a focused run:
bash scripts/run_pg_tests.sh authority --demo
```

Install PostgreSQL 16 first. The runner finds its binaries through PATH or
Homebrew; PG_BIN can select an installed directory. It ignores caller
DATABASE_URL and connection overrides, creates a unique database under
`target/pg-tests/run-*`, and verifies the server belongs to that run before
creating fixtures. Database creation rechecks that identity on its own connection,
so a changed server cannot receive the creation command. An available loopback port is selected automatically;
`--port 55432` requests a particular free port. The old PG_PORT variable is no
longer used. Suites run sequentially with one test thread to protect shared
fixtures.

Success stops the server, reaps its process group and removes the workspace.
Command output is saved to numbered logs and printed when that command exits.
Failure retains those logs, `postgres.log` and `runner.json` for diagnosis; `stopped` means
shutdown was verified, while `shutdown-unverified` requires process inspection
before cleanup. Ctrl-C also stops owned processes. Forced termination cannot
run cleanup, so inspect the receipt and server metadata before removing residue.
This is a trusted-host test service using loopback trust authentication and
synthetic data.

Database-backed tests now require the runner's live ownership receipt and exact
endpoint. Merely setting DATABASE_URL makes them refuse before migration or
cleanup. They also check the server's data directory, database and owner marker
on each new pooled connection, including replacement connections. The `pg_guard`
suite exercises these refusals. With DATABASE_URL unset, the existing offline
skips remain available; clear an inherited DATABASE_URL before an offline run.

The runner supplies a private matching synthetic passfile under its owned cluster
and explicit loopback defaults. SQLx can fall back to the home passfile after a
missing or unmatched custom file, so the former nonexistent placeholder did not
establish locality. The operator CLI uses explicit options with passfile lookup
disabled; the live CLI control verifies the runner's selected fixture credential.

```bash
bash scripts/run_pg_tests.sh pg_guard authority
# Offline checks, with no database target inherited from a development shell:
env -u DATABASE_URL make check
```

The PG CI job now uses this runner and the same suite list. Compiler installation,
packages and scratch stay under the repository; installed PostgreSQL and rustup
are read-only tool inputs. GitHub execution is verified on the next push.
Detailed contracts: `docs/decisions/2026-09-09_disposable-postgresql-runner.md` and
`docs/decisions/2026-09-09_disposable-test-pool-ownership.md`.

## Full checkpoint before pushing

Ordinary slices run focused checks and commit. Before a push, the scheduled
checkpoint also runs workspace format/strict lint/tests, the full owned PostgreSQL
collection with explicit `--demo`, Python runner/environment controls, doctrine
checks, fresh dependency checks, redacted Git-history scanning and this book build.
Build the worker binaries first and inspect test output: a missing browser or
extraction worker can cause an early return, and offline database skips do not
qualify live database behavior. Ignored Codex/Claude provider runs require separate
live-provider qualification; the schema-golden writer is deliberate regeneration.

The current source census finds 38 server suites plus MCP and CLI in the owned
runner (40 commands), and 86 Cargo test-enabled targets across 12 packages. These
are inventory counts, not passing-test counts. At the census commit, workflow
locality/Python coverage and publisher/browser fixture lifetimes still need their
tracked checkpoint repairs before broad execution. No fresh full-CI or release
claim follows from the inventory. See `docs/ci.md` for exact commands and
`docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md` for source evidence,
tool boundaries and the concrete repair sequence.

CI now has a shared setup launcher:

```bash
python3 -B scripts/ci_env.py --rust -- cargo fmt --all -- --check
```

It establishes repository stores before optional pinned compiler installation and
execs the command from the checkout root. Installed rustup is read-only; new
compiler files stay under .project-data/installed-toolchains. Installer failure,
timeout or terminal cancellation prevents dispatch and consumes child cleanup.
CI clears inherited database/provider/scanner and selected compiler overrides;
ordinary developer project_env behavior is unchanged. Explicit output paths still
need their command's storage checks. Eight focused and eighteen adjacent controls
pass, using an instrumented installer. Workflows now use the launcher; actual
remote compiler installation and full GitHub execution remain pending. The exact contract and evidence are in
`docs/tasks/artifacts/signoff_review/ci-environment.md`.

Scanner installation now has an explicit verified path:

```bash
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py cargo-deny --verify-only
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py gitleaks --verify-only
```

These commands verify pinned release bytes, bounded executable extraction and the
actual version; they do not run security gates. Omit --verify-only when the full
checkpoint reaches actual dependency/history scanning. Each invocation owns a
directory under target/ci-scanners. It retains logs, redacted reports and a
scanner.json receipt; read both scope and exit_code. A completed operation can
still have a nonzero scanner result. Successful consumed setup/execution retires
only its archive and executable; failed setup retains diagnostic files and its
last recorded child identity for inspection.

Thirteen focused controls and final affected controls pass. All eight pinned
Linux/macOS architecture archives pass identity/layout checks; actual version
execution is verified on aarch64 macOS. The native probe caught and corrected a
Gitleaks version-format mismatch that instrumented tests missed, with the original
failure retained. Workflow wiring is complete; full local/remote gates remain pending.
See `docs/tasks/artifacts/signoff_review/ci-scanners.md` for exact versions,
limits and evidence. Installation integrity does not complete release qualification.

## CI requires its runtime prerequisites

All six project command jobs enter the local launcher. Rust checks require Chrome,
build the workspace binaries and verify executable extraction/browser workers
before tests. Missing prerequisites fail instead of supplying apparent coverage.
The PostgreSQL job validates installed version-16 tools, discovers every Python
control module and runs the full owned collection with explicit `--demo`. The book
job installs pinned mdBook 0.5.4 into .project-data/cargo using a local build tree,
checks its version and builds this book. Strict shell errors and bounded jobs
preserve failures. Provider tests remain separately authorized qualification.

The supply-chain jobs run the actual pinned scanner gates. CI retains their
receipts, version/check logs and redacted Gitleaks JSON report; binaries and archives
are excluded. A real synthetic-history probe detected one deliberately unissued
value and fully redacted it from report and output. The initial alphabet example
was excluded by the scanner's example filter; its result was investigated before
correcting the fixture. No repository-wide secret-free claim follows.

Three YAML workflows, six shell command blocks, five deliberate omission controls
and fifty Python tests pass locally, including actual PostgreSQL ownership checks.
GitHub's managed checkout/artifact transport and installed OS tools are explicit
platform dependencies; project stores and artifact temporary paths derive from the
checkout. Workflow, publisher/browser and compiler-artifact prerequisites are now
complete; full Rust/security gates and actual remote outcomes remain pending. See
`docs/tasks/artifacts/signoff_review/ci-workflows.md` for commands and exact evidence.

## Historical scanner exceptions

The history scanner retains its normal rules. Two exact fingerprints in
`.gitleaksignore` identify predictable literals used only by historical local
metadata tests. Source and data-flow inspection establish that they were never
issued credentials or used for connections. Each exception names the immutable
commit, file, rule and line; it does not exclude the file or future changes.

Five native controls verify that removing either entry restores that finding,
using both gives a clean configured history result, and committing identical
content again still produces both findings. An alternate ignore path does not
replace the source-root policy in the pinned scanner, so omission checks use an
isolated copy of the exact history. Requalify these boundaries when changing the
scanner version or exceptions. The full configured history scan still runs before
push; it does not scan uncommitted files or prove that every possible secret is
absent. See `docs/tasks/artifacts/signoff_review/history-fixture-fingerprints.md`.

## Publisher verification owns its directories

The offline publisher tests create private, exclusive directories under
`target/publisher-tests`, after checking the parent components remain directories
on the repository volume. They never remove a preexisting directory to start a
test. A directory collision refuses. Repository readers and handles close before
explicit cleanup, which checks the original directory identity and confirms its
removal. An assertion failure or unfinished fixture retains its data and prints
its location for diagnosis. Inspect the owning test/process before removing it;
old `pub-N` directories may belong to historical work.

The old counter-based helper was reproduced in two isolated processes: the second
removed the first process's witness while that owner was still running. The new
helper's two-process control preserves both witnesses; finishing one owner removes
only its own directory. Fixture isolation improves verification reliability; it
does not change production publication or qualify its broader governance gates.
Exact checks and their results are tracked in
`docs/tasks/artifacts/signoff_review/publisher-fixtures.md`.

## Browser verification bounds its workers and origins

The browser integration harness gives each command a private on-volume fixture
under `target/browser-lifetime-controls`, including temporary, profile/cache and
bounded diagnostic data. Each worker starts in its own process group. Input,
stdout, stderr and exit share a deadline; each output stream has a 2 MiB ceiling
plus one byte to detect overflow. After any result the harness consumes the worker
and confirms group absence, escalating bounded shutdown if needed. Failed tests
or unconfirmed cleanup retain their directory and receipt for inspection.

A native macOS control shows that a group containing an exited, unreaped child
can transiently refuse inspection with `EPERM`. The harness waits only within a fixed bound and requires
an actual absence observation; persistent denial stays an error. Refused signal
requests never qualify cleanup. The initial eight-control harness qualification passed with strict focused lint;
the original failed fixture is retained with its unconfirmed receipt.

Local HTTP origins have explicit graceful shutdown; the serving task is consumed
before successful fixture removal. Budget admission always runs, even without a
browser. The render test reports an absent browser as unqualified; an explicitly
configured invalid browser is an error. The CI workflow requires browser presence.

The original worker left a real renderer running after it returned. The production
worker now owns Chrome before its first asynchronous launch wait, uses private
per-invocation storage and consumes process/task shutdown before returning a
successful result. The harness independently verifies browser-group absence; its
own emergency cleanup still cannot count as production-owned shutdown. Exact
baseline and controls are in
`docs/tasks/artifacts/signoff_review/browser-production-lifetimes.md`.

The earlier combined qualification passed fifteen integration controls with unchanged
production worker bytes, alongside the separately recorded five production unit
controls. Two real renders succeed across a same-volume runtime-root rename; the
original directory identity and a witness survive, while completed invocation
storage is removed. A linked parent refuses before Chrome startup and preserves
its target. Overlap is held by an explicit origin gate until both live groups and
profiles are observed, including a deliberately delayed second launch.

Origin shutdown is checked by identity. A native control demonstrates that a new
listener can reuse a closed listener's port and even its numeric file descriptor;
a later successful connection does not prove the original listener survived. The
harness consumes the original socket's close receipt as well as its serving task.
The original failed connection's peer was not captured, so its exact cause remains
unstated. All twenty-six final worker/browser groups are independently absent;
the nine earlier failed fixtures remain preserved. Exact results and remaining
boundaries are in `docs/tasks/artifacts/signoff_review/browser-combined-qualification.md`.

### Browser timing and the selected test runtime

The later full checkpoint found that two timing assumptions were too short for
real startup: the navigation deadline could fire before any request reached the
origin, and the overlap test could abandon its second launch. Navigation now has
an explicit arrival witness and a response that remains gated until the worker
returns. For example, a six-second startup delay must still reach the origin before
a thirty-second render deadline; cleanup must finish before the refusal returns.
Overlap keeps worker one's response gated, delays worker two four seconds after
its arrival, and proves both distinct profiles/groups are live before release.
Missing requests, early responses and unconfirmed cleanup still fail qualification.

That stronger deadline witness exposed a separate runtime boundary. Desktop Chrome
started crash reporting/update helpers outside its process group; native pipe
observations showed them retaining stderr after that group stopped. The worker
returned cleanup-unconfirmed rather than claiming success. The observed helpers
later exited naturally. A dedicated repository-local Chrome for Testing 153.0.8010.36
passes the delayed navigation/overlap witnesses and all sixteen integration controls
with the unchanged production worker. The archive and extracted bytes are verified;
this testing build has an ad-hoc linker signature, not verified Developer ID signing.

The result qualifies these trusted loopback fixtures. Pinned browser installation
and CI selection remain .11.4.3.1.2.5 before the full checkpoint resumes; the current
workflow still selects installed Chrome. Untrusted-content isolation, detached
process containment and aggregate resource limits remain .7.3.2. The full workspace
checkpoint has not passed, and PostgreSQL/demo has not run in this attempt. Exact
results, retained failures and scope: `docs/tasks/artifacts/signoff_review/browser-checkpoint-timing.md`.

## Public repository and publication checks

This project is public and must remain public. The director confirmed that the
earlier private-repository instruction was wrong; README, ADR-001 and their
companion guidance now reflect the correction. No visibility change is needed.
Name/package/domain and release qualification remain separate requirements.

Repository commits, branches, issues and pull requests are public. Confidential
security reports and pre-disclosure work require a separate private channel or
workspace arranged with the accountable owner; see `SECURITY.md`. Do not rely on
public Git to provide an embargo or make this repository private for that purpose.

The visibility question is resolved. The earlier checkpoint's format/dependency
passes, two redacted history-scan findings and deliberately interrupted Clippy
result retain their exact historical status. Continue the owned history-scan
repair and full checkpoint on the resulting committed source, then perform the
authorized normal push and consume triggered CI results. Details:
`docs/decisions/2026-09-09_public-repository-policy.md` and
`docs/tasks/artifacts/signoff_review/publication-precondition.md`.

## Periodic compiler artifact cleanup

Review generated artifacts about once per day. Keep diagnostics and failed-run
fixtures until their owning task has established a safe disposition. An old
`.bin` file alone is not disposable evidence: incremental compiler sessions contain
related cache components and may be in use or shared by hard links.

The checkpoint cleanup qualified the exact installed macOS compiler, preserved
every newest finalized session and excluded young, partial, working, hard-linked
and evidence-dependent data. Native controls verified both shared and exclusive
cache locks against actual compiler collection and successful later execution.
Only a frozen, identity-checked set of whole obsolete sessions was removed under
exclusive locks, after a native inactivity census.

That operation removed 645 sessions / 1,984 files / 5,116,558,334 logical bytes.
Immediately after removal, the remaining cache metadata and 3,545 inspected
source/diagnostic hashes were unchanged. Zero-byte session locks remain for the compiler to collect. Logical
bytes do not establish physical disk space recovered on a cloning filesystem.
This was a qualified maintenance operation, not a general automatic deletion
command. Recheck compiler/platform locking before repeating or broadening it.
See `docs/tasks/artifacts/signoff_review/compiler-artifact-disposition.md` for
selection, refusal conditions, native controls, exact residue and workflow evidence.

## Browser invocation storage and shutdown

Run an R3-enabled server or `reasonbraid-browse` from within the repository. The worker
finds the root from current-directory ancestors containing Cargo.toml and migrations.
It creates `.project-data/browser/run-<UUID>` exclusively, mode 0700, on the repository
volume. Profile, cache, scratch, configuration and data paths are private children;
Chrome receives explicit paths, including its log and crash-dump environment
overrides; ambient TLS/QUIC diagnostic output overrides are removed. Moving the
checkout changes those paths at runtime.
A missing root, linked storage parent, foreign volume or unsafe writable parent
refuses with `browser_storage_failed`; there is no OS/home temporary fallback.

For example, from the repository root with the installed browser configured:

```bash
python3 -B scripts/project_env.py target/debug/reasonbraid-browse <<'JSON'
{"url":"http://127.0.0.1:8080/page","steps":[{"action":"navigate","url":"http://127.0.0.1:8080/page"}],"limits":{"max_steps":4,"max_output_bytes":1048576,"time_budget_secs":5}}
JSON
```

The caller must first classify and authorize the destination; this example assumes
an owned local origin is running. Existing successful JSON keeps derived chunks,
network log, title and browser/worker versions. `click_failed`, `output_too_large`
and `time_budget_exceeded` remain named refusals. Rendering includes startup under
the requested budget, minimum one second; startup also has a twenty-second ceiling.
Every cooperative result then receives up to ten additional seconds for process
and task shutdown. Chrome is closed, its direct child reaped, its group observed
absent and the CDP/network/stderr tasks consumed. Bounded TERM/KILL requests may be
needed; a refused inspection or signal is never evidence of absence.

A render result cannot bypass failed cleanup. The worker instead emits
`browser_cleanup_unconfirmed`, preserving the original render error in the message
when present. A consumed successful invocation removes only its verified original
directory. A failed or unconfirmed invocation retains private `owner.json`,
`completion.json` and at most 64 KiB of `browser.stderr` when those files can be
written. Stderr also carries ownership/completion receipts with root-relative
workspace paths. Inspect these records before cleanup; an old numeric PID alone
must never authorize signalling a current process.

The shutdown budget covers asynchronous process/task operations, not a hard bound
on filesystem calls, stdin or response delivery. Storage assumes an operator-owned
repository. Aggregate retained-data quotas are still open. The current server-side
spawner can kill the worker at the render deadline, interrupting this shutdown;
`.7.3.1` owns that integration repair. Container enforcement for hard termination or
detached descendants remains `.7.3.2`. Broader network, sandbox, total-output and
evidence policies retain their existing qualification gaps. Linux/macOS process
support is explicit; the current real-browser evidence is native macOS evidence.

## Project binaries

```bash
make release
```

| Binary | Role |
| --- | --- |
| `rb` | the CLI — the primary surface (threads, enrollment, node ops, budget/audit inspection) |
| `rb-server` | the control plane — API + node channel + the embedded console at `/` (one listener, one binary) |
| `rb-site` | deployment-local site authority administration and audited inventory; see [site authority](site-authority.md) for required database permissions and volume checks |
| `rb-bench` | adapter benchmark runner |
| `reasonbraid-browse` | browser acquisition worker |
| `reasonbraid-extract` | resource extraction worker |
| `rb-release-manifest` | release manifest tooling |
| `rb-node` | the node worker — outbound channel, SQLite journal, the adapter supervisor |
| `rb-journal` | the node-journal inspection tool |

The control-plane binary embeds its database migrations and console assets, so a
deployed `rb-server` needs no runtime path back to the checkout for those embedded
assets. Enabling the R3 browser worker currently requires a working directory within
the checkout for private storage discovery. The `rb-site` operator tool also runs
from within the checkout to verify storage locality. The node's
journal is a local SQLite file that stays on the node's own volume.

## The two profiles

`ROADMAP.md` §6.6 names the initial profiles; Phase 1 ships two of them:

- **Developer** — loopback, one host. `make dev` boots an ephemeral on-volume
  PostgreSQL and `rb-server`; Ctrl-C cleans everything up (the `.1.7.1`
  one-command development environment).
- **Trusted LAN** — enrolled nodes on a private network. The control plane
  binds the LAN (`rb-server --host 0.0.0.0 …`); each node enrolls with a
  one-time token and connects outbound (`rb-node --server http://<cp-host>:4310 …`).

The `deploy/` runbook walks both hosts step by step: PostgreSQL provision,
server start (migrations apply on startup), enrollment, node start, and the
adapter choice (the deterministic fake for practice, the env-gated real
harness suites for live runs).

## The packaging proof

The two-host demonstration runs against the **release binaries** with one
flag — the packaging claim is verified by running the package:

```bash
bash scripts/demo_two_host.sh --database-url <url> --release
# two real hosts:
bash scripts/demo_two_host.sh --database-url <url> --release \
    --node-host <other-host> --remote-workdir '~/rb-demo'
```

The demo's ssh mode copies `rb-node` + `rb-journal` to the remote host and
keeps the journals there; the bundle's `env.txt` records the mode and the
build.

## Honest boundaries (Phase 1)

- **Trusted LAN only:** the channel credential is the dev shared secret (the
  HMAC key-proof handshake); X.509/mTLS workload identity is ADR-006/ADR-007
  (Phase 2), and the wire is plain HTTP. Public exposure waits for the G6
  Internet-qualification gate.
- **One node, one role:** the dev profile has no directory — a node id is the
  agent-role wire id it serves.
- **Flags, not config files:** everything is flags/env; process supervision
  units and container images arrive with Phase 2 operations — recorded in the
  runbook's subtraction record.

## Backup and restore (`.4.1`)

The recovery control is the RESTORE, not the dump (§17.5: a backup that has
never been restored is not a recovery control):

```bash
bash scripts/backup.sh                  # DATABASE_URL -> target/backups/<date>.dump
createdb reasonbraid_restored           # an ISOLATED target database
BACKUP_FILE=… RESTORE_DATABASE_URL=… bash scripts/restore.sh
```

The dev-profile dump is plaintext (the encryption + key story deferral is
named in the `.4.3` record), and the guard runs the restore EXERCISE on every
pass: the `backup_restore` suite seeds rows, takes a real `pg_dump`, mutates
the live database, restores into an isolated database, and asserts the
pre-mutation state came back.

## Observability, SLOs, and the runbook (`.5`)

- **Metrics:** `GET /v1/admin/metrics` exposes the seven process-wide
  counters (`authorization_denials`, `idempotency_replays`,
  `handshake_refusals`, `lease_refusals`, `dead_letters`, `results_folded`,
  `results_rejected`). The gate: the caller HOLDS `tenant_admin` in any active
  grant. Every counter is incremented at the same boundary that writes its
  record, and the live test asserts the denial delta against the denied row —
  the surface cannot drift from the ledger. Structured `log_event!` JSON lines
  carry the correlation fields under ADR-023's redaction rules.
- **SLOs:** the dev profile's hypotheses are the guard's own measurements
  (`docs/decisions/2026-09-07_phase2-slo-hypotheses.md`): acceptance, the
  demo's 34 checks, the restore exercise, and the reconcile-after-kill beats
  all target 100 % with a ZERO error budget — a red pass halts the frontier.
  Unmeasured latency families are named with their triggers, not numbers.
- **Runbook:** node lost/replaced (`docs/runbooks/node-lost-replaced.md`)
  covers detection through closure tests; its closure tests are the demo's
  SIGKILL beat, the revoke beat, the replay suites, and the restore exercise.
