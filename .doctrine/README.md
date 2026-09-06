# `.doctrine/` — the project-declared seams

These optional files let a project adapt the **neutral** doctrine checks to its own shape
**without editing the checks**. Editing a spine check to hardcode your paths or your tool names
turns a portable standard into a fork of it — that is what these seams exist to prevent.

| file | consumed by | meaning |
|---|---|---|
| `code_paths.txt` | `TASK-ACCEPTANCE` | one extended regular expression per line: what counts as a **code change** here. Absent ⇒ the built-in Rust-workspace default (`crates/`, `src/`, `scripts/`, `*.rs`, `*.sh`, `Makefile`). |
| `evidence_tokens.txt` | `TASK-ACCEPTANCE` | one extended regular expression per line: **your** tools' output signatures, ADDED to the universal defaults. Absent ⇒ defaults only. |
| `readme_routes.txt` | `README-STABILITY` | one row per routed destination: `path\|class\|pressure control\|owner` — every destination the README, the policy, or the guard's routing hint names must end at a governed terminal (a row's path governs that path and everything under it). Absent ⇒ the guard refuses. |

Blank lines and `#` comments are ignored in all three.

## When to declare evidence tokens

`TASK-ACCEPTANCE` requires each hard-gated checklist box to contain output from a tool that was
actually run. It ships with signatures that are universal to any Rust project (`error[E1234]`,
`could not compile`, `clippy::…`, `test result: ok`, panics, profilers) and to any project's
build-flow forensics (`git log -S`, `shellcheck`, `bash -n`, `make -n`, `ENOSPC`…).

If your project has its own instruments — a coverage reporter, a conformance gate, a custom
linter — declare their output signatures here so an author can cite them:

```
# .doctrine/evidence_tokens.txt
WIDGET-COVERAGE:
MYGATE: (pass|fail)
```

⚠️ **A signature family that does not match your real corpus is a gate that teaches authors to
waive it.** Before adopting a token, check it against the evidence your team actually pastes; a
family that backs almost nothing is worse than no family, because the honest response to it is a
waiver — which is itself a bug report about the gate (see `WAIVER-ROUTING`).
