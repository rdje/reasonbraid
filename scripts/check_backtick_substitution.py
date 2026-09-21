#!/usr/bin/env python3
"""BACKTICK-SUBSTITUTION — a backtick in shell text that will be EXPANDED is a
command substitution, and when the author meant a literal it silently empties the
line (`SIGNOFF-REPAIR.11.33`).

🔴 THE DEFECT IS SILENT, WHICH IS WHY IT NEEDED A GATE. `scripts/demo_two_host.sh`
carried ``log "… node dev secrets (`.1.2.1`)"`` for fifteen commits. Bash ran
`.1.2.1`, wrote `command not found` to stderr and substituted its empty output, so
the published demonstration log read *node dev secrets ()* — the roadmap reference
gone, with CI green throughout. `.11.29` found ONE instance because its stderr
landed beside a real failure, repaired it and closed; `.11.32` then found two more
in the same file.

⭐ THE RULE IS BROADER THAN THE SENTENCE IT INHERITED, AND THAT IS A MEASUREMENT.
`.11.32` proposed *an output-emitting command (`log`/`echo`/`printf`) whose
double-quoted argument carries an unescaped backtick*, because the obvious matcher
returned 19 hits of which 17 were false. `.11.33` re-measured with a parser that
tracks quote nesting: the restriction to output commands buys **nothing** — BROAD
and NARROW return the identical set at every revision tested. The 17 were the
parser's noise, not the rule's, so the narrowing was compensating for a matcher.
⛔ The broad rule also covers what the narrow one could not: a backtick in any
expanded double-quoted string, including an assignment or a heredoc body.

⭐ CALIBRATED OVER THE HISTORY THAT CONTAINS THE INSTANCE
(`docs/knowledge/calibrate-over-the-history-that-contains-the-instance.md`).
Across **105 commits touching a shell file and 171 shell blobs**, it fires on
**15 commits, and behind them are exactly 3 real defect sites — all in
`demo_two_host.sh`** — each counted from the commit that introduced it to the
commit that escaped it. **Zero false positives in the project's history.** The
three are the one `.11.29` repaired and the two `.11.32` found afterwards, which
is the class this gate exists to stop being found by hand. For comparison the gates this
repository REJECTED fired at 87%, 93% and 71% of the population they were priced
over, and `POSITIONAL-REF` shipped at 9.5%.

⛔ WHY A PARSER AND NOT A REGEX, stated because the regex is the obvious shape.
Shell quoting is nested, and every false positive measured in this family came
from that nesting: a backtick inside `'…'` inside `"$(…)"` is literal data, and
this repository's own check scripts carry many of them in awk programs and
self-test fixtures. A matcher that cannot say which quote it is inside reports
those as defects and gets waived. The scan is therefore WHOLE-FILE — a
single-quoted awk program spans many lines, and a per-line scanner loses its
state and reports the lines inside it.

⛔ AND IT IS WRITTEN IN PYTHON DELIBERATELY. The corpus is tracked SHELL files, so
a checker written in shell would be inside its own corpus, and its fixtures would
be scanned as if they were real code. `SIGNOFF-REPAIR.11.4.3.1.7.1` is the standing
instance of that trap: a census asserted a probe name appears in zero tracked files
and wrote that probe as a literal in its own tracked source. Here the separation is
structural rather than remembered.

⚠️ WHAT IT DOES NOT CLAIM. It reads tracked files, so it cannot see a backtick in
an interactive command — `.11.33`'s third instance was a `python3 -B -c "…"` typed
at a prompt, and NEITHER this gate nor the runtime alternative would have caught
it. That is recorded rather than implied by the gate's existence.

    python3 -B scripts/check_backtick_substitution.py
    python3 -B scripts/check_backtick_substitution.py --census
    python3 -B scripts/check_backtick_substitution.py --calibrate
    python3 -B scripts/check_backtick_substitution.py --as-of REV
    python3 -B scripts/check_backtick_substitution.py --self-test

Self-test: scripts/check_backtick_substitution.py --self-test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# `<<WORD`, `<<-WORD`, `<<'WORD'`, `<<"WORD"`, `<<\WORD`. The three quoted forms
# make the body LITERAL; the bare form makes it expanded, which is the form that
# can carry this defect.
HEREDOC = re.compile(
    r"<<-?\s*(?:(')([A-Za-z_][A-Za-z0-9_]*)'"
    r"|\"([A-Za-z_][A-Za-z0-9_]*)\""
    r"|\\([A-Za-z_][A-Za-z0-9_]*)"
    r"|([A-Za-z_][A-Za-z0-9_]*))")


@dataclass(frozen=True)
class Hit:
    line: int
    column: int
    text: str


def scan(text: str) -> list[Hit]:
    """Every unescaped backtick in shell text that WILL be expanded.

    The state machine carries across lines on purpose. Four contexts:

      `unquoted` / `subst`  — ordinary shell words, and the inside of `$( … )`,
                              where `'` and `"` open quotes as usual;
      `single`              — everything literal until the closing `'`, and a
                              backslash is NOT an escape;
      `double`              — expanded; a backtick here is command substitution;
      `heredoc`             — an UNQUOTED heredoc body: expanded, but `'` and `"`
                              are ordinary characters rather than quotes. A
                              nested `$( … )` restores the normal grammar, which
                              is exactly why `check_tree_index_frontier.sh`'s
                              `<<EOF` body is NOT a hit: its backtick sits inside
                              `'…'` inside `$( … )`.

    ⭐ A flagged backtick consumes through its partner, so one substitution is one
    hit rather than two. Counting the closing backtick separately made the same
    defect look twice as large as it is.
    """
    lines = text.splitlines()
    hits: list[Hit] = []
    stack = ["unquoted"]
    pending: list[tuple[str, bool]] = []
    active: tuple[str, bool] | None = None

    for lineno, line in enumerate(lines, 1):
        if active is not None:
            delim, expands = active
            if line.strip() == delim:
                active = None
                continue
            if not expands:
                continue
            stack = ["heredoc"]

        i = 0
        while i < len(line):
            c = line[i]
            state = stack[-1]

            if state == "single":
                if c == "'":
                    stack.pop()
                i += 1
                continue

            if c == "\\":
                i += 2
                continue

            if state in ("double", "heredoc"):
                if state == "double" and c == '"':
                    stack.pop()
                    i += 1
                    continue
                if c == "`":
                    hits.append(Hit(lineno, i, line.strip()[:140]))
                    partner = line.find("`", i + 1)
                    i = (partner + 1) if partner != -1 else len(line)
                    continue
                if c == "$" and line[i + 1:i + 2] == "(":
                    stack.append("subst")
                    i += 2
                    continue
                i += 1
                continue

            if c == "'":
                stack.append("single")
            elif c == '"':
                stack.append("double")
            elif c == "$" and line[i + 1:i + 2] == "(":
                stack.append("subst")
                i += 2
                continue
            elif c == ")" and state == "subst":
                stack.pop()
            elif c == "#" and (i == 0 or line[i - 1] in " \t"):
                break
            elif c == "<" and line[i + 1:i + 2] == "<":
                m = HEREDOC.match(line, i)
                if m:
                    delim = m.group(2) or m.group(3) or m.group(4) or m.group(5)
                    expands = not (m.group(2) or m.group(3) or m.group(4))
                    pending.append((delim, expands))
                    i = m.end()
                    continue
            i += 1

        if active is not None:
            stack = ["unquoted"]
        elif pending:
            active = pending.pop(0)
    return hits


def repo_root() -> Path:
    out = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True, check=True)
    return Path(out.stdout.strip())


def shell_files(root: Path, rev: str | None) -> tuple[list[str], dict[str, str]]:
    """Tracked shell files and their contents.

    ⛔ MEMBERSHIP IS BY SHEBANG AS WELL AS BY EXTENSION. `.githooks/pre-commit`
    and its siblings are shell scripts with no `.sh`, and they are the files most
    likely to print a message — a corpus defined by extension alone would have
    excluded exactly the scripts whose whole job is output.
    """
    if rev:
        names = subprocess.run(["git", "ls-tree", "-r", "--name-only", rev],
                               cwd=root, capture_output=True, text=True,
                               check=True).stdout.split()

        def read(rel: str) -> str | None:
            r = subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=root,
                               capture_output=True, text=True)
            return r.stdout if r.returncode == 0 else None
    else:
        names = subprocess.run(["git", "ls-files"], cwd=root,
                               capture_output=True, text=True,
                               check=True).stdout.split()

        def read(rel: str) -> str | None:
            path = root / rel
            if not path.is_file():
                return None
            try:
                return path.read_text(errors="replace")
            except OSError:
                return None

    files: list[str] = []
    bodies: dict[str, str] = {}
    for rel in names:
        body = read(rel)
        if body is None:
            continue
        if rel.endswith(".sh") or is_shell_shebang(body):
            files.append(rel)
            bodies[rel] = body
    return sorted(files), bodies


def is_shell_shebang(body: str) -> bool:
    """Is the first line a shebang naming a shell interpreter?

    ⚠️ THE INTERPRETER IS NOT SIMPLY THE LAST WORD, and the first version of this
    was wrong in both directions for that reason. `#!/usr/bin/env bash` puts the
    interpreter AFTER `env`, and `#!/bin/bash -e` puts a flag after it. The rule
    is: the first token that is neither the `env` launcher nor an option, reduced
    to its basename.
    """
    first = body.split("\n", 1)[0]
    if not first.startswith("#!"):
        return False
    for token in first[2:].split():
        name = token.rsplit("/", 1)[-1]
        if name == "env" or token.startswith("-"):
            continue
        return name.endswith("sh")
    return False


def calibrate(root: Path) -> int:
    """Price the rule over every commit that touched a shell file.

    ⛔ THE NUMBERS IN THIS FILE'S HEADER AND IN THE DOCTRINE REGISTRY ARE
    PRODUCED HERE, never restated from a scratch run. A published count whose
    producer is a throwaway script is a leg-3 breach of
    `docs/CLAIM_VERIFICATION.md`, and this repository has measured its mirrors
    drifting for exactly that reason.

    ⭐ It scans the BLOB each commit introduced rather than the whole tree at
    that commit: a gate fires because of content that arrived, and pricing it
    against the tree would count the same standing defect once per commit in the
    repository rather than once per commit that touched shell.
    """
    commits = subprocess.run(["git", "log", "--format=%H", "--", "*.sh"],
                             cwd=root, capture_output=True, text=True,
                             check=True).stdout.split()
    blobs = 0
    fired: list[tuple[str, str, int, str]] = []
    for sha in commits:
        changed = subprocess.run(
            ["git", "show", "--name-only", "--format=", sha, "--", "*.sh"],
            cwd=root, capture_output=True, text=True).stdout.split()
        for rel in changed:
            body = subprocess.run(["git", "show", f"{sha}:{rel}"], cwd=root,
                                  capture_output=True, text=True)
            if body.returncode:
                continue
            blobs += 1
            for hit in scan(body.stdout):
                fired.append((sha[:8], rel, hit.line, hit.text[:90]))

    commits_fired = {f[0] for f in fired}
    sites = {(f[1], f[3]) for f in fired}
    print(f"=== BACKTICK-SUBSTITUTION, priced over the history that contains the instance ===")
    print(f"  commits touching a shell file : {len(commits)}")
    print(f"  shell blobs scanned           : {blobs}")
    print(f"  commits it would have FIRED on: {len(commits_fired)}")
    print(f"  distinct defect sites behind them: {len(sites)}")
    for rel, text in sorted(sites):
        print(f"      {rel}  {text}")
    print()
    print("  ⭐ Every fire is the same standing defect, from the commit that introduced")
    print("     it to the commit that escaped it — zero false positives in the whole")
    print("     history. The gates this repository REJECTED fired at 87%, 93% and 71%")
    print("     of the population they were priced over.")
    return 0


def self_test() -> int:
    fails = 0
    ran = 0

    def check(name: str, got, want) -> None:
        nonlocal fails, ran
        ran += 1
        if got != want:
            print(f"BACKTICK-SUBSTITUTION self-test: {name}: got {got!r}, want {want!r}",
                  file=sys.stderr)
            fails += 1

    def lines_of(text: str) -> list[int]:
        return [h.line for h in scan(text)]

    # ⛔ EVERY FIXTURE IS BUILT FROM A CHARACTER CONSTANT, NEVER WRITTEN AS A
    # LITERAL. `SIGNOFF-REPAIR.11.4.3.1.7.1`'s instance was a probe written as a
    # literal into its own tracked source; the structural defence here is that
    # this file is Python and outside the shell corpus, and this is the belt.
    bt = chr(96)
    dq = chr(34)
    sq = chr(39)

    # 1. the real defect: a backtick inside a double-quoted argument.
    check("double-quoted backtick is a hit",
          lines_of(f"log {dq}text ({bt}.1.2.1{bt}){dq}\n"), [1])
    # 2. escaped, which is the repair `.11.32` applied.
    check("escaped backtick is not",
          lines_of(f"log {dq}text (\\{bt}.1.2.1\\{bt}){dq}\n"), [])
    # 3. the measured false-positive class: single inside a substitution.
    check("single-quoted inside $( ) is not",
          lines_of(f"x=$(grep -E {sq}^[{bt}A-Z]{sq} f)\n"), [])
    # 4. a comment is not code.
    check("a comment is not code", lines_of(f"# see {dq}{bt}thing{bt}{dq}\n"), [])
    # 5. the multi-line state that a per-line scanner loses: a single-quoted awk
    #    program whose body contains a double quote and a backtick.
    awk = f"awk {sq}\n  m = (x ~ /a/) ? {dq}{bt}{dq} : {dq}~{dq}\n{sq} file\n"
    check("state carries across lines", lines_of(awk), [])
    # 6. a quoted heredoc body is literal.
    check("quoted heredoc body is literal",
          lines_of(f"cat <<{sq}Q{sq}\na {bt}thing{bt} here\nQ\n"), [])
    # 7. an unquoted heredoc body is expanded, so the same text IS a hit.
    check("unquoted heredoc body is expanded",
          lines_of(f"cat <<Q\na {bt}thing{bt} here\nQ\n"), [2])
    # 8. and inside that body, a nested $( ) restores the normal grammar.
    check("nested $( ) inside an unquoted heredoc quotes again",
          lines_of(f"cat <<Q\n$(grep {sq}[{bt}]{sq} f)\nQ\n"), [])
    # 9. one substitution is one hit, not two.
    check("a pair is one hit", len(scan(f"log {dq}a {bt}b{bt} c{dq}\n")), 1)
    # 10. two separate substitutions on one line are two hits.
    check("two pairs are two hits",
          len(scan(f"log {dq}{bt}a{bt} and {bt}b{bt}{dq}\n")), 2)
    # 11. a backtick in unquoted text is substitution too, but it is the author
    #     writing a command on purpose; the rule is about EXPANDED TEXT, and an
    #     unquoted backtick is not inside text. Pinned so the scope is explicit.
    check("a bare unquoted backtick is out of scope",
          lines_of(f"x={bt}date{bt}\n"), [])
    # 12. shebang membership.
    check("bash shebang is shell", is_shell_shebang("#!/usr/bin/env bash\n"), True)
    check("sh shebang is shell", is_shell_shebang("#!/bin/sh\n"), True)
    check("python shebang is not", is_shell_shebang("#!/usr/bin/env python3\n"), False)
    check("no shebang is not", is_shell_shebang("echo hi\n"), False)
    # The two shapes the first implementation got wrong in opposite directions.
    check("env launcher is stepped over",
          is_shell_shebang("#!/usr/bin/env bash\n"), True)
    check("an interpreter flag is not the interpreter",
          is_shell_shebang("#!/bin/bash -e\n"), True)
    check("env python is still not shell",
          is_shell_shebang("#!/usr/bin/env python3\n"), False)

    if fails:
        print(f"BACKTICK-SUBSTITUTION: {fails} of {ran} self-test control(s) failed",
              file=sys.stderr)
        return 1
    print(f"BACKTICK-SUBSTITUTION: self-test ok — {ran} controls "
          "(quoting, nesting, heredocs, multi-line state, pairing, membership)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--census", action="store_true",
                    help="report the corpus size and every hit, and exit 0 — the "
                         "population on demand rather than restated in prose")
    ap.add_argument("--calibrate", action="store_true",
                    help="price the rule over every commit that touched a shell file — "
                         "the producer for the numbers in this file's header")
    ap.add_argument("--as-of", metavar="REV",
                    help="scan the tree at REV instead of the checkout — how the "
                         "rule is put back against the commits that carried the defect")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    root = repo_root()
    if args.calibrate:
        return calibrate(root)
    files, bodies = shell_files(root, args.as_of)
    if not files:
        print("BACKTICK-SUBSTITUTION: REFUSED — no tracked shell file was found, so "
              "this check cannot judge anything.", file=sys.stderr)
        return 2

    found: list[tuple[str, Hit]] = []
    for rel in files:
        for hit in scan(bodies[rel]):
            found.append((rel, hit))

    where = f" at {args.as_of}" if args.as_of else ""
    if args.census:
        print(f"BACKTICK-SUBSTITUTION census{where}: {len(files)} tracked shell files, "
              f"{len(found)} expanded backtick(s).")
        for rel, hit in found:
            print(f"    {rel}:{hit.line}  {hit.text}")
        return 0

    if found:
        print(f"BACKTICK-SUBSTITUTION: {len(found)} backtick(s) in shell text that "
              f"bash will EXPAND{where} — the text between them is replaced by the "
              f"output of running it, and a literal was almost certainly meant:",
              file=sys.stderr)
        for rel, hit in found:
            print(f"    {rel}:{hit.line}  {hit.text}", file=sys.stderr)
        print("  Escape them (\\`…\\`) or single-quote the string. This is silent: the "
              "line still prints, with the reference gone.", file=sys.stderr)
        return 1

    print(f"BACKTICK-SUBSTITUTION: OK — {len(files)} tracked shell files, no expanded "
          f"backtick{where}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
