# `.doctrine/` — the project-declared seams

These optional files let a project adapt the **neutral** doctrine checks to its own shape
**without editing the checks**. Editing a spine check to hardcode your paths or your tool names
turns a portable standard into a fork of it — that is what these seams exist to prevent.

| file | consumed by | meaning |
|---|---|---|
| `code_paths.txt` | `TASK-ACCEPTANCE` | one extended regular expression per line: what counts as a **code change** here. Absent ⇒ the built-in Rust-workspace default (`crates/`, `src/`, `scripts/`, `*.rs`, `*.sh`, `Makefile`). |
| `evidence_tokens.txt` | `TASK-ACCEPTANCE` | one extended regular expression per line: **your** tools' output signatures, ADDED to the universal defaults. Absent ⇒ defaults only. |
| `acceptance_labels.txt` | `TASK-ACCEPTANCE` | `<NAME><TAB><extended regex>` per line: the hard-gated questions a closing leaf must answer, **in your project's spellings**. REPLACES the built-in families rather than adding to them. Absent ⇒ the portable defaults. |
| `readme_routes.txt` | `README-STABILITY` | one row per routed destination: `path\|class\|pressure control\|owner` — every destination the README, the policy, or the guard's routing hint names must end at a governed terminal (a row's path governs that path and everything under it). Absent ⇒ the guard refuses. |

Blank lines and `#` comments are ignored in all of them.

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

## When to declare acceptance labels

`TASK-ACCEPTANCE` asks the leaf a commit closes three questions — the cause, the fix, and
whether anything regressed. It ships with generic spellings, and a project that writes its
checklist differently declares its own here.

```
# .doctrine/acceptance_labels.txt
CAUSE	root.?cause|reproduce|the census
FIX	the repair|the rule|addressed
NO REGRESSION	no.?regress
```

⛔ **Derive the regexes from a census of the checklists you already write — never invent them.**
This seam exists because that was measured and got it wrong. The gate hard-gated `ROOT CAUSE`
and `ADDRESSED` while the corpus it governed wrote `REPRODUCE / ISSUE` **142** times,
`FIX / LOCKSTEP` **192** and `NO REGRESSION` **203**, against **13** and **14** for the two it
blocked on. Nobody noticed for roughly 200 commits, because a separate defect made the gate read
only one bullet in the whole file — so it never refused anything and never revealed the drift.

⭐ **An inert control does not hold a line; it hides that the line moved.** When you widen a
family, price the widening against your history before adopting it — here 20.9 % → 17.5 % — and
then price the tightening you will need afterwards. Requiring the family to be the bullet's
**lead-in** rather than a word anywhere inside it costs three points (17.5 % → 20.6 %) and is
still right: without it, a leaf that merely *discusses* a label answers it, which is exactly how
the leaf repairing this gate passed with its real checklist bullet deleted.

Expect the residual to be real rather than noise, and keep it countable: `--debt` prints the
population so it stays a number you can re-derive instead of a memory.
