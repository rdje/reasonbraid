#!/usr/bin/env python3
"""Census tracked Markdown for a blank line INSIDE a table body.

A blank line TERMINATES a GFM table. Every row after it stops being a row: the
renderer emits the remaining lines as one paragraph of literal pipe-separated
text. Nothing about the source looks wrong, so this survives every review that
reads the file instead of the page.

    python3 -B scripts/census_broken_tables.py            # the census
    python3 -B scripts/census_broken_tables.py --check    # the BROKEN-TABLE gate
    python3 -B scripts/census_broken_tables.py --json
    python3 -B scripts/census_broken_tables.py --calibrate [N]
    python3 -B scripts/census_broken_tables.py --self-test

⭐ EVERY PARSING RULE BELOW IS THE RENDERER'S ANSWER, NOT THE SPECIFICATION'S.
`TABLE-ARITY-RATCHET` established mdbook as this corpus's authority after a check
shipped asserting the opposite of what the renderer did — and its own `--self-test`
asserted that same false answer. Each rule here was rendered before being written:

  · a blank line after the delimiter row  → empty `<table>`, rows become `<p>`
  · a blank line mid-body                 → the rows after it become ONE `<p>`,
                                            NOT a second table
  · delimiter cell count ≠ header's       → NOT A TABLE AT ALL (no `<table>`)
  · a table indented 1–3 spaces           → still a table
  · a table indented 4+ spaces            → a code block (`<pre>`), not a table

⛔ The middle one is why this cannot be a variant of `TABLE-ARITY-RATCHET`: that
gate compares a row's cell count against its header's, and an ORPHANED ROW HAS NO
HEADER to disagree with. The doctrine's intent was satisfied and the rendering
broke — `BOOK-LINKS`' founding shape.

⚠️ AN UNBALANCED FENCE REFUSES RATHER THAN GUESSING (`SIGNOFF-REPAIR.11.18`): if
the markers do not pair, every later line reads as fenced and the instrument
silently stops looking. That is an instrument narrowing its own scope without
saying so.
"""

from __future__ import annotations

import collections
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

FENCE = re.compile(r"^(```|~~~)")
# A delimiter row: cells of dashes with optional alignment colons.
DELIM_CELL = re.compile(r"^:?-+:?$")


def _dedent(line: str) -> tuple[str, int]:
    """Return the line without its leading spaces, and how many there were.

    ⛔ Tabs count as 4 toward the code-block threshold, which is what makes the
    4-space rule below match the renderer on a tab-indented line.
    """
    n = 0
    for ch in line:
        if ch == " ":
            n += 1
        elif ch == "\t":
            n += 4
        else:
            break
    return line.lstrip(" \t"), n


def split_cells(row: str) -> list[str]:
    """GFM cells of one row. Only a backslash-escaped `\\|` is not a separator.

    ⛔ An inline code span does NOT protect a pipe — `TABLE-ARITY-RATCHET`
    measured that against the renderer and shipped the opposite first.
    """
    cells, buf, i = [], [], 0
    while i < len(row):
        c = row[i]
        if c == "\\" and i + 1 < len(row):
            buf.append(row[i : i + 2])
            i += 2
            continue
        if c == "|":
            cells.append("".join(buf))
            buf = []
            i += 1
            continue
        buf.append(c)
        i += 1
    cells.append("".join(buf))
    # A leading and/or trailing pipe produces an empty outer cell; GFM ignores it.
    if cells and not cells[0].strip():
        cells = cells[1:]
    if cells and not cells[-1].strip():
        cells = cells[:-1]
    return cells


def is_delimiter(row: str) -> int | None:
    """Cell count if this is a delimiter row, else None."""
    if "|" not in row and "-" not in row:
        return None
    cells = split_cells(row)
    if not cells:
        return None
    if all(DELIM_CELL.match(c.strip()) for c in cells):
        return len(cells)
    return None


def scan(text: str, path: str = "<text>") -> list[dict]:
    """Every blank line inside a table body, with the rows it orphans."""
    lines = text.splitlines()
    fence: str | None = None
    fence_line = 0
    out: list[dict] = []
    in_table = False
    i = 0
    while i < len(lines):
        stripped, indent = _dedent(lines[i])
        m = FENCE.match(stripped)
        if m:
            if indent < 4:
                if fence is None:
                    fence, fence_line = m.group(1), i + 1
                elif stripped.startswith(fence):
                    fence = None
                in_table = False
                i += 1
                continue
        if fence is not None:
            i += 1
            continue

        # A code block, not a table (the renderer's answer for 4+ spaces).
        if indent >= 4:
            in_table = False
            i += 1
            continue

        if not in_table:
            n = is_delimiter(stripped)
            if n is not None and i > 0:
                header, hindent = _dedent(lines[i - 1])
                # ⛔ The renderer emits NO table when the counts disagree.
                if hindent < 4 and "|" in header and len(split_cells(header)) == n:
                    in_table = True
            i += 1
            continue

        # In a table body.
        if stripped == "":
            k = i
            while k < len(lines) and not lines[k].strip():
                k += 1
            if k < len(lines):
                nxt, nindent = _dedent(lines[k])
                if nindent < 4 and nxt.startswith("|"):
                    n = k
                    while n < len(lines):
                        cand, cindent = _dedent(lines[n])
                        if cindent >= 4 or not cand.startswith("|"):
                            break
                        n += 1
                    out.append(
                        {
                            "file": path,
                            "blank_line": i + 1,
                            "first_orphan": k + 1,
                            "orphaned_rows": n - k,
                        }
                    )
                    i = n
                    continue          # more blanks may follow in the same table
            in_table = False
            i += 1
            continue
        if not stripped.startswith("|"):
            in_table = False
        i += 1

    if fence is not None:
        raise ValueError(
            f"{path}: unbalanced code fence opened at line {fence_line} — refusing to "
            "classify rather than reading every later line as fenced"
        )
    return out


def tracked_markdown() -> list[str]:
    return subprocess.run(
        ["git", "ls-files", "*.md"], capture_output=True, text=True, cwd=ROOT
    ).stdout.split()


def census() -> dict:
    per_file: dict[str, dict] = {}
    blanks = rows = 0
    for f in tracked_markdown():
        hits = scan((ROOT / f).read_text(encoding="utf-8"), f)
        if hits:
            per_file[f] = {
                "blanks": len(hits),
                "orphaned_rows": sum(h["orphaned_rows"] for h in hits),
                "detail": hits,
            }
            blanks += len(hits)
            rows += per_file[f]["orphaned_rows"]
    return {
        "files_scanned": len(tracked_markdown()),
        "files_affected": len(per_file),
        "blank_lines_in_a_table": blanks,
        "rows_rendered_as_literal_text": rows,
        "by_file": per_file,
    }


def calibrate(depth: int) -> dict:
    """What the gate would have fired on, commit by commit.

    `SIGNOFF-REPAIR.11.6`'s standing requirement: measure a rule against its
    population before proposing it, and `POSITIONAL-REF`'s method.
    """
    shas = subprocess.run(
        ["git", "rev-list", "--reverse", "-n", str(depth + 1), "HEAD"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    ).stdout.split()

    def count_at(sha: str, f: str) -> int:
        blob = subprocess.run(
            ["git", "show", f"{sha}:{f}"], capture_output=True, text=True, cwd=ROOT
        )
        if blob.returncode != 0:
            return 0                      # the file does not exist at that commit
        try:
            return len(scan(blob.stdout, f))
        except ValueError:
            return 0                      # an unbalanced fence at that commit

    def full_state(sha: str) -> dict[str, int]:
        names = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", sha],
            capture_output=True, text=True, cwd=ROOT,
        ).stdout.split()
        st = {}
        for f in names:
            if f.endswith(".md"):
                k = count_at(sha, f)
                if k:
                    st[f] = k
        return st

    # ⛔ Incremental on purpose: a full re-scan per commit is 200 × every tracked
    # .md, which does not finish in a usable time. Only the files a commit TOUCHED
    # can change its own count, so the baseline is computed once and each step
    # re-scans that commit's changed Markdown alone.
    touching = blocked = 0
    blocked_at = []
    prev: dict[str, int] = full_state(shas[0]) if shas else {}
    for a, b in zip(shas, shas[1:]):
        changed = [
            f
            for f in subprocess.run(
                ["git", "diff", "--name-only", a, b, "--", "*.md"],
                capture_output=True, text=True, cwd=ROOT,
            ).stdout.split()
        ]
        if not changed:
            continue
        cur = dict(prev)
        gained = []
        for f in changed:
            after = count_at(b, f)
            before = prev.get(f, 0)
            if after:
                cur[f] = after
            else:
                cur.pop(f, None)
            if after > before:
                gained.append(f)
        if cur != prev:
            touching += 1
        if gained:
            blocked += 1
            blocked_at.append({"commit": b[:7], "files": sorted(gained)})
        prev = cur
    examined = max(len(shas) - 1, 0)
    return {
        "commits_examined": examined,
        "commits_changing_the_population": touching,
        "commits_blocked": blocked,
        "blocked_pct": round(100 * blocked / examined, 1) if examined else 0.0,
        "detail": blocked_at,
    }


def self_test() -> int:
    fails = 0

    def check(name, got, want):
        nonlocal fails
        if got != want:
            print(f"BROKEN-TABLE self-test: {name}: got {got!r}, want {want!r}", file=sys.stderr)
            fails += 1

    def n(text):
        return sum(h["orphaned_rows"] for h in scan(text))

    # ── arms 1–3: the defect, as the renderer produced it ────────────────────
    # 1 — a blank right after the delimiter row. mdbook: empty <table>, the row
    #     comes back as <p>| 1 | 2 |</p>.
    check("blank after the delimiter", n("| A | B |\n| --- | --- |\n\n| 1 | 2 |\n"), 1)
    # 2 — a blank mid-body. mdbook: ONE table, ONE data row, the rest one <p>.
    check(
        "blank mid-body orphans every later row",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n\n| 3 | 4 |\n| 5 | 6 |\n"),
        2,
    )
    # 3 — THE NEGATIVE, and the arm that stops this gate being "no blank line
    #     near a table": a blank ENDING a table is how every table ends.
    check(
        "a blank ending a table is not a defect",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n\nProse after the table.\n"),
        0,
    )
    # 4 — and a table at end of file.
    check("a table at EOF", n("| A | B |\n| --- | --- |\n| 1 | 2 |\n"), 0)

    # ── arms 5–6: the fence exemption (SIGNOFF-REPAIR.11.18's lesson) ────────
    check(
        "a table inside a fence is not judged",
        n("```\n| A | B |\n| --- | --- |\n\n| 1 | 2 |\n```\n"),
        0,
    )
    check(
        "an INDENTED fence still opens a fence",
        n("  ```\n| A | B |\n| --- | --- |\n\n| 1 | 2 |\n  ```\n"),
        0,
    )

    # ── arms 7–9: the renderer's answers about what IS a table ───────────────
    # 7 — mdbook renders NO <table> when the delimiter's cell count differs.
    check(
        "arity mismatch is not a table",
        n("| A | B |\n| --- |\n\n| 1 | 2 |\n"),
        0,
    )
    # 8 — mdbook renders <pre> at four spaces of indent.
    check(
        "a 4-space-indented table is a code block",
        n("    | A | B |\n    | --- | --- |\n\n    | 1 | 2 |\n"),
        0,
    )
    # 9 — and still a table at three.
    check(
        "a 3-space-indented table is a table",
        n("   | A | B |\n   | --- | --- |\n\n   | 1 | 2 |\n"),
        1,
    )

    # ── arm 10: the escaped pipe, which TABLE-ARITY-RATCHET had backwards ────
    check(
        "an escaped pipe is not a cell separator",
        n("| A | B |\n| --- | --- |\n\n| x \\| y | 2 |\n"),
        1,
    )

    # ── arm 11: refuse, do not guess ─────────────────────────────────────────
    try:
        scan("```\n| A | B |\n| --- | --- |\n\n| 1 | 2 |\n", "fixture.md")
        print("BROKEN-TABLE self-test: an unbalanced fence did not refuse", file=sys.stderr)
        fails += 1
    except ValueError:
        pass

    # ── arm 12: the real corpus is reachable ─────────────────────────────────
    real = census()
    if real["files_scanned"] < 50:
        print(
            f"BROKEN-TABLE self-test: only {real['files_scanned']} tracked .md found"
            " — expected the real corpus",
            file=sys.stderr,
        )
        fails += 1

    if fails:
        return 1
    print(
        "BROKEN-TABLE self-test: 12 arms — the defect in two positions, THREE negatives"
        " (a blank that ends a table, EOF, a fence), the renderer's three answers about"
        " what is a table, an escaped pipe, an unbalanced fence refused, and"
        f" {real['files_scanned']} tracked files reachable"
    )
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    if "--calibrate" in argv:
        i = argv.index("--calibrate")
        depth = int(argv[i + 1]) if len(argv) > i + 1 and argv[i + 1].isdigit() else 200
        c = calibrate(depth)
        if "--json" in argv:
            print(json.dumps(c, indent=2))
            return 0
        print(f"over {c['commits_examined']} commits")
        print(f"  commits changing the population : {c['commits_changing_the_population']}")
        print(f"  commits it would have BLOCKED   : {c['commits_blocked']}  ({c['blocked_pct']}%)")
        for d in c["detail"][-12:]:
            print(f"     {d['commit']}  {', '.join(d['files'])}")
        return 0

    c = census()
    if "--json" in argv:
        print(json.dumps(c, indent=2))
        return 0

    if "--check" in argv:
        if not c["by_file"]:
            print(
                f"BROKEN-TABLE: OK — no blank line inside a table body"
                f" ({c['files_scanned']} tracked .md scanned)"
            )
            return 0
        print(
            "BROKEN-TABLE: a blank line ENDS a Markdown table — every row after it"
            " renders as literal text, not as a row.",
            file=sys.stderr,
        )
        for f, d in c["by_file"].items():
            for h in d["detail"]:
                print(
                    f"    {f}:{h['blank_line']} blank line ends the table;"
                    f" {h['orphaned_rows']} row(s) from line {h['first_orphan']}"
                    " render as a paragraph",
                    file=sys.stderr,
                )
        print(
            "  Delete the blank line, or — if two distinct tables were intended —"
            " give the second one its own header and delimiter row.",
            file=sys.stderr,
        )
        return 1

    print(f"tracked .md scanned              : {c['files_scanned']}")
    print(f"files with a blank inside a table: {c['files_affected']}")
    print(f"blank lines inside a table body  : {c['blank_lines_in_a_table']}")
    print(f"ROWS rendered as literal text    : {c['rows_rendered_as_literal_text']}")
    for f, d in c["by_file"].items():
        print(f"  {f}")
        for h in d["detail"]:
            print(
                f"    line {h['blank_line']:>6} → {h['orphaned_rows']:>3} orphaned row(s)"
                f" from line {h['first_orphan']}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
