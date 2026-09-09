# Publication precondition audit

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.1`; REPAIR-0042. Inspected committed source:
`f0265e26bbbb7dcd44fa61155768e5cba2689913`. Raw receipts/logs:
`target/checkpoint-ci/full-f0265e2`. This is a completed blocker audit, not a
completed full CI checkpoint or public-release clearance.

## Independently confirmed policy conflict

README.md:4 says: “Keep this repository **private** until that ADR’s clearance gate
passes.” docs/adr/001-uncleared-working-name.md:36 retains that requirement until
named clearance or rename. No clearance or public-push authorization is established.

The project-local GitHub tool configuration initially returns exit 4, no configured
authentication. Existing Git transport still succeeds. For the required privacy/CI
API reads, explicitly use the OS-account shared `.config/gh` tool configuration as
a read-only input, deriving its absolute path at runtime. Its file metadata is
unchanged before/after the API command; no credential content is printed/copied.
All writable caches and output remain repository-local. This exception does not
authorize changing shared credentials or adopting shared project caches.

`gh repo view --json nameWithOwner,isPrivate,defaultBranchRef` returns rc=0:
rdje/reasonbraid, isPrivate=false, default branch main. An independent unauthenticated
HTTPS request to the [official repository API](https://api.github.com/repos/rdje/reasonbraid)
returns rc=0 with the same full_name, private=false and visibility=public. Curl
explicitly disables its default config and sends no authentication header. The web
cache lookup failed; that failed lookup is not visibility evidence.

`git ls-remote origin refs/heads/main` returns
`b932c054023ea127520e74cfaf95b9bdf1ea47fe`, matching the local tracking ref. The
inspected source is 310 commits ahead. These observations establish the current
public state, not that the repository was previously private, who changed it, or
when. API creation/update/push timestamps do not identify a visibility-change event.
Earlier private-state statements were not authenticated observations and cannot
be reused as current evidence.

No push or visibility change was made. Director decision `.11.4.3.1.2.3` must
resolve whether to restore private visibility under the existing policy or supply
an explicit revised publication instruction. The concrete proposed action is to
set rdje/reasonbraid visibility to private and re-read it through authenticated and
unauthenticated APIs before any push. Changing visibility cannot undo prior public
access and is not a claim of name clearance.

## Gate results and safe stop

| Gate on f0265e2 | Actual result |
| --- | --- |
| Format | pass, rc=0, 0.510s; group consumed |
| cargo-deny 0.20.2 | pass, rc=0, 4.879s; advisories, bans, licenses and sources all report OK; scanner group reaped |
| Gitleaks 8.30.1 full Git history | fail, rc=1, 5.871s; exactly two redacted generic-api-key matches; scanner group reaped |
| Strict all-target/all-feature Clippy | deliberately interrupted at the confirmed policy blocker after 341.153s; supervisor rc=130, group cleanup consumed; neither pass nor lint failure |
| Worker build, workspace tests, Python collection, PostgreSQL/demo | not started by this checkpoint; no new coverage claim |
| Book/doctrines | focused documentation validation for this audit, recorded separately from the uncompleted full checkpoint |
| Push/remote workflows | not performed |

The two history findings point to commit
`82155f53146585dc047734447e7288ed5c14e187`,
`crates/reasonbraid-server/tests/pg_guard.rs`, lines 68 and 110. Both secret fields
are REDACTED. Initial source inspection places the literals in local preflight and
symlink fixture tests, but full provenance/classification remains `.11.4.3.1.2.2`.
No credential leak or false-positive conclusion is asserted. No exclusion or
history rewrite is added. The flagged commit is not an ancestor of the confirmed
remote main; this does not inventory other refs, copies or all historical access.

Scanner evidence is retained in `target/ci-scanners/cargo-deny-zqdflyxw` and
`target/ci-scanners/gitleaks-9u8ke3yi`. Exact archive/version identities were verified
before each gate. The selected check invocation has no offline/disabled-advisory
option; receipt/log evidence does not establish any stronger release guarantee.

After verifying the sole live gate and its parent command, send SIGINT only to
the owned checkpoint driver. Its established supervisor terminates/reaps the
Clippy group and returns KeyboardInterrupt/130. Consume that result and both scanner
results. The native handoff census returns rc=0, no project-owned job remaining.
`checkpoint-stop.json` records why this was an intentional stop; preserve the
partial Clippy log and exception receipt. No downstream gate started.

## Resume

Resolve `.11.4.3.1.2.3` with the director. Then qualify/fix the exact history findings
under `.2.2` and execute the remaining full checkpoint `.2` on the resulting named
committed source, using a fresh source-specific receipt directory. Never overwrite
these receipts or count the interrupted lint as passed. Complete local gates before
normal authorized push, confirm remote advancement and consume triggered CI.
After that, return to bounded CLI transport `.3.3.4.3.3.3.3.2.3.2`.
