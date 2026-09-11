# TOOLBOX.md — the tools-first diagnostic doctrine

⛔ **TOOLS-FIRST.** For ANY unknown — a failure, a crash, a hang, a surprising result, a
"why isn't this working" — reach for a diagnostic tool FIRST. Never eyeball the code and
guess a root cause.

## The rule

- A code change cannot land without **tool-backed WHY + WHERE** and a **measured
  before→after** recorded in its task-tree leaf (see the acceptance checklist in
  `DOCTRINE_ENFORCEMENT.md`).
- If no existing tool shows WHY+WHERE, **build one** — a probe, a tracer, a counter, a
  minimal reproduction harness. The diagnostic tool is a first-class deliverable, kept in
  the repo, not a throwaway.
- **ANTI-SPIN TRIPWIRE:** if you have analyzed for ~2 turns without producing NEW tool
  output that pinpoints WHY+WHERE, STOP — run a tool, build one, or escalate. Never loop
  on analysis.

## The 3-step UNKNOWN protocol (adapt the specific tools to your domain)

1. **WIDEN** — dump the full picture: enumerate all cases/states, the broadest inventory,
   so the failing one is visible in context.
2. **NARROW** — probe the specific failing case for its exact verdict + position/state.
3. **PINPOINT** — a scoped trace that names the exact function/rule/line that fails and why.

The point is to convert "it's broken somewhere" into "line X of function Y rejects input Z
because predicate P is false" before writing a single line of fix.

## This project's toolbox

<!-- Fill this in as your project grows. List each diagnostic tool, what question it
answers (WHY / WHERE / how-much), and how to invoke it (binary, flag, env var). The next
agent should be able to reach for the right tool without reading the source. -->

| Tool | Answers | How to invoke |
| --- | --- | --- |
| `scripts/project_env.py` | selected repository-local stores; verified copying of locked Cargo cache data; literal-argument command execution | `python3 -B scripts/project_env.py --print` or append a command |
| `scripts/ci_browser.py` | exact verified testing browser; local installation, version and consumed command receipts | `python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py --verify-only` or `-- cargo test -p reasonbraid-browse --test browser_roundtrip --locked` |
| `rb-journal` (`crates/reasonbraid-node`) | journal health (durability profile, `quick_check`, counts), pending attempts/events, ambiguous attempts with boundary history — without opening SQLite by hand | `rb-journal inspect\|pending\|ambiguous <node.db> [--json]` |
| `scripts/run_pg_tests.sh` | named suites with test-side ownership checks in a supervised PostgreSQL 16 cluster; `--list` shows names; no names runs the broad collection | `bash scripts/run_pg_tests.sh authority command_api` |
| Site authority controls | explicit operator gate, tenant-only denial, actual-parent liveness, audit rollback and ordered revocation | `RB_DEMO=0 bash scripts/run_pg_tests.sh site_authority` |
| Protected operator CLI | explicit deployment-local endpoint and operator role; issue/list/disable site authority and inspect audit | `python3 -B scripts/project_env.py target/debug/rb-site --help` |
| `scripts/check_visibility_policy.sh` | every tracked-Markdown sentence that states a private visibility for this repository, and whether it is a reviewed exception; `--self-test` proves the matcher fires and does not over-match | `bash scripts/check_visibility_policy.sh [--self-test]` |
| `scripts/check_book_frontier.sh` | whether a book page names a frontier leaf the task tree no longer has; `--self-test` proves row-1 extraction and the claim matcher | `bash scripts/check_book_frontier.sh [--self-test]` |
| `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` | the WP3 journal kill-point sweep + CLI integration tests (file-based SQLite, no service) | `python3 -B scripts/project_env.py cargo test -p reasonbraid-node` |
