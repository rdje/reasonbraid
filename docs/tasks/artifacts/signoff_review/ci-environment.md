# Repository-local CI environment

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.1`; REPAIR-0034.
- Baseline: 238051b; workflow repair parent .11.4.3.1.3.
- Scope: scripts/ci_env.py and eight focused controls. No workflow wiring, scanner installation, product Rust, full gate or remote CI execution in this leaf.

## Baseline and corrected mechanism

The source's three raw Cargo workflow commands were executed against an
instrumented Cargo executable in a unique owned fixture. Each inherited
CARGO_HOME and TMPDIR pointing outside that fixture checkout, into another
explicitly owned on-volume probe directory. All three preserved those ambient
stores. The diagnostic returned zero after observing the violation; it did not
compile code or execute GitHub Actions. Every probe fixture was removed.
Evidence: target/ci-workflow-controls/environment-baseline.json.

The new launcher derives its root from its own script location, establishes the
17 configured project stores before any external command, sets private creation
mode and disables Python bytecode. CI clears inherited DATABASE_URL, RB_DEMO,
RB_LIVE_CODEX/RB_LIVE_CLAUDE, GITLEAKS_CONFIG/GITLEAKS_CONFIG_TOML,
RB_READONLY_TOOLCHAIN, RUSTUP_TOOLCHAIN, Rust distribution/update overrides and
RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS; compiler wrappers are cleared too. This is a
CI execution contract, not a change to the developer project_env launcher or an
arbitrary filesystem sandbox. Explicit tool paths and additional caches still
need their owning command's locality contract.

With --rust, an exact numeric rust-toolchain.toml pin is required. A complete
local compiler is reused; otherwise the installed read-only rustup executable
provisions the pin with minimal profile, rustfmt/clippy and no self-update into
.project-data/installed-toolchains. Cargo downloads and scratch already point
inside the checkout. Compiler discovery refuses linked installation paths,
linked component files, volume escapes and non-executable components. The
requested command receives explicit compiler binaries and the local RUSTUP_HOME.
No compiler download was performed for these instrumented controls; actual
installation and tool execution remain checkpoint/remote evidence.

Installation uses the existing supervised process primitive with a twenty-minute
timeout. It reaps the child/process group before returning or propagating failure.
The launcher handles terminal signals while installation is in flight. Installer
failure/timeout prevents command dispatch; successful setup changes to the root
and execs the requested command, leaving no extra launcher parent. The requested
command's own lifetime/budget remains its command/workflow owner's responsibility.

## Executed controls

All commands below entered python3 -B scripts/project_env.py. The candidate and
regression drivers used the existing run_command supervisor with bounded waits,
repository-local logs and consumed exit receipts.

| Control | Observed result |
| --- | --- |
| Relocated checkout with quoted original path | real child cwd is the moved root; all configured stores are local/same-volume; tempfile is local; ambient store never created |
| Instrumented installation and reuse | exact pin/components/no-self-update arguments; local installer stores; actual child exits/reaps; second call uses existing compiler without installer; PATH executes its instrumented cargo |
| Installer exit 7 | launcher exits 2; command marker absent; installer group absent |
| Installer timeout | two-second test timeout propagates after reaping; no dispatch |
| Terminal SIGTERM | launcher exits 130 after installer shutdown; no dispatch; groups consumed |
| Linked installation store | refuses before installer; unrelated target remains empty |
| Linked compiler component | refuses without reinstallation; unrelated tool bytes preserved |
| Floating stable pin | refuses before installer |

`python3 -B -m unittest discover -s scripts/tests -p test_ci_env.py -v` passed
8 tests in 4.012s, rc=0. Existing test_project_env.py passed 5 in 0.013s and
existing test_pg_runner.py passed 13 in 1.126s, both rc=0. Total selected controls:
26, of which 8 cover the new launcher and 18 cover its reused environment/process
mechanisms. The runner module's mock-failure messages are expected assertions,
not failed tests or live PostgreSQL launches. No live PG suite ran in this leaf.

Raw evidence is environment-candidate.log/.exit and environment-regression.log/.exit
under target/ci-workflow-controls. Directory-specific residue inspection found
zero environment-* fixture directories. An initial unfiltered glob reported
three matches; exact file-type inspection showed only baseline/candidate evidence
files, not leaked fixtures. Those evidence files remain retained.

Final wrapped Python verification returned zero: both new Python sources parse;
the developer launcher, process supervisor and all three workflows are byte-identical
to 238051b; both exit receipts are zero and no owned environment fixture directory
remains. make book, eight rendered deployment markers and git diff --check pass,
rc=0. Final summary is environment-final-verification.json in the same evidence
directory. Final staged doctrines run in the commit hook. Workflows remain unchanged until .3.3; scanner version and
integrity setup is .3.2. Full local/remote CI is still .11.4.3.1.2.
