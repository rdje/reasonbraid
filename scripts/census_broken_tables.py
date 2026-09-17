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
  · a body line with NO leading pipe      → STILL A ROW (`3 | 4` renders as two
                                            cells; bare prose renders as one,
                                            padded to the header's width)
  · a list item, heading, blockquote or
    HTML block with no blank line before → ENDS the table
  · plain prose, or indented continuation → does NOT end it; both are absorbed

🔴 THAT LAST ONE CORRECTED THIS INSTRUMENT AFTER IT SHIPPED (`SIGNOFF-REPAIR.11.19.1`).
The first cut ended a table at the first pipe-less line, which produced BOTH
directions of error: a FALSE NEGATIVE — a blank line later in the same table was
missed — and, worse, a FALSE POSITIVE, because two adjacent tables separated by a
blank line were reported as three orphaned rows. That is ordinary Markdown, and
the gate would have blocked the next author who wrote it. A table body now ends
only at a blank line or a fence, and a pipe row after a blank is a NEW TABLE
rather than an orphan when it brings its own header and delimiter pair.

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

# Block constructs that END a table body even with NO blank line before them.
# ⛔ Asked of the renderer, one construct at a time (`SIGNOFF-REPAIR.11.19.1`),
# because the answer is not uniform and guessing it wrong goes BOTH ways: plain
# prose and indented continuation text are ABSORBED into the table as rows,
# while a list item, a heading, a blockquote and an HTML block end it.
ENDS_TABLE = re.compile(r"^(?:[-*+] |\d+[.)] |#{1,6} |> |<!--|<[a-zA-Z])")


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


def _starts_table(lines: list[str], j: int) -> bool:
    """True when a header + delimiter pair begins at line j (0-based).

    The same test in both places it is needed: opening a table, and deciding
    whether the pipe row after a blank line is a NEW table rather than an
    orphaned row.
    """
    if j + 1 >= len(lines):
        return False
    header, hindent = _dedent(lines[j])
    delim, dindent = _dedent(lines[j + 1])
    if hindent >= 4 or dindent >= 4 or "|" not in header:
        return False
    n = is_delimiter(delim)
    return n is not None and len(split_cells(header)) == n


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
            # ⛔ The renderer emits NO table when the header's and delimiter's
            # cell counts disagree, so the pair is tested together.
            if i > 0 and _starts_table(lines, i - 1):
                in_table = True
            i += 1
            continue

        # In a table body. ⛔ A table body continues across ANY non-blank line,
        # pipes or not — the renderer turns a bare prose line after a row into a
        # row. Only a blank line (or a fence) ends it. An earlier cut ended the
        # table at the first pipe-less line and so could MISS a blank line later
        # in the same table.
        if stripped != "":
            if ENDS_TABLE.match(stripped):
                in_table = False
            i += 1
            continue

        k = i
        while k < len(lines) and not lines[k].strip():
            k += 1
        in_table = False
        if k < len(lines):
            nxt, nindent = _dedent(lines[k])
            # ⛔ TWO ADJACENT TABLES separated by a blank line are ordinary
            # Markdown, not a defect. The second one declares itself with its
            # own header + delimiter pair, which is the same test that STARTS a
            # table — so ask it here rather than flagging every pipe row that
            # follows a blank.
            if nindent < 4 and nxt.startswith("|") and not _starts_table(lines, k):
                n = k
                while n < len(lines):
                    cand, cindent = _dedent(lines[n])
                    if cindent >= 4 or not cand.strip() or not cand.startswith("|"):
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
                # The orphaned rows are NOT a table, so resume scanning after them.
                i = n
                continue
        i = k if k > i else i + 1

    if fence is not None:
        raise ValueError(
            f"{path}: unbalanced code fence opened at line {fence_line} — refusing to "
            "classify rather than reading every later line as fenced"
        )
    return out


def absorbed(text: str, path: str = "<text>") -> list[dict]:
    """Every line a table SWALLOWS: prose abutting a table with no blank line.

    The MIRROR of `scan`. That one finds a blank line where none belongs, which
    SPLITS a table; this finds no blank line where one belongs, so the following
    block is absorbed and every line of it becomes a row padded to the header's
    width (`SIGNOFF-REPAIR.11.19.2`).

    ⛔ Which constructs end a table without a blank line is NOT uniform and was
    established against the renderer, not read off a specification — see
    `ENDS_TABLE`. Getting it wrong here over-counts: a bullet list abutting a
    table is fine, and reading it as absorbed inflated this population from 11
    to 15 on the first pass.
    """
    lines = text.splitlines()
    fence: str | None = None
    out: list[dict] = []
    in_table = False
    for i, raw in enumerate(lines):
        stripped, indent = _dedent(raw)
        m = FENCE.match(stripped)
        if m and indent < 4:
            fence = None if (fence and stripped.startswith(fence)) else (fence or m.group(1))
            in_table = False
            continue
        if fence is not None:
            continue
        if indent >= 4:
            in_table = False
            continue
        if not in_table:
            if i > 0 and _starts_table(lines, i - 1):
                in_table = True
            continue
        if stripped == "" or ENDS_TABLE.match(stripped):
            in_table = False
            continue
        if not stripped.startswith("|"):
            out.append({"file": path, "line": i + 1, "text": stripped[:90]})
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
    swallowed: dict[str, list[dict]] = {}
    for f in tracked_markdown():
        hits = absorbed((ROOT / f).read_text(encoding="utf-8"), f)
        if hits:
            swallowed[f] = hits
    return {
        "files_scanned": len(tracked_markdown()),
        "files_affected": len(per_file),
        "blank_lines_in_a_table": blanks,
        "rows_rendered_as_literal_text": rows,
        "by_file": per_file,
        "absorbed_lines": sum(len(v) for v in swallowed.values()),
        "absorbed_by_file": {k: len(v) for k, v in swallowed.items()},
        "absorbed_detail": swallowed,
    }


def calibrate(depth: int, absorbed_too: bool = False) -> dict:
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
            n = len(scan(blob.stdout, f))
            if absorbed_too:
                n += len(absorbed(blob.stdout, f))
            return n
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

    # ── arms 11–13: what a table BODY is, corrected by the renderer ─────────
    # 11 — TWO ADJACENT TABLES separated by a blank line are ordinary Markdown.
    #      The first cut of this gate flagged them as 3 orphaned rows, which
    #      would have blocked the next author who wrote two tables in a row
    #      (`SIGNOFF-REPAIR.11.19.1`). This is the arm that refutes it.
    check(
        "two adjacent tables separated by a blank are NOT a defect",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n\n| C | D |\n| --- | --- |\n| 5 | 6 |\n"),
        0,
    )
    # 12 — a body row written WITHOUT a leading pipe is still a row: mdbook
    #      renders `3 | 4` after a row as `<td>3</td><td>4</td>`. The first cut
    #      ended the table there and MISSED the blank line below it.
    check(
        "a pipe-less body row does not end the table",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n3 | 4\n\n| 5 | 6 |\n"),
        1,
    )
    # 13 — and so is a line with no pipe at all: mdbook renders `some prose`
    #      after a row as a one-cell row padded to the header's width.
    check(
        "a pipe-less PROSE line does not end the table either",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\nsome prose\n\n| 5 | 6 |\n"),
        1,
    )

    # ── arm 14: refuse, do not guess ─────────────────────────────────────────
    try:
        scan("```\n| A | B |\n| --- | --- |\n\n| 1 | 2 |\n", "fixture.md")
        print("BROKEN-TABLE self-test: an unbalanced fence did not refuse", file=sys.stderr)
        fails += 1
    except ValueError:
        pass

    # ── arms 15–17: what ENDS a table body, one construct at a time ─────────
    # ⛔ The answer is not uniform, and each of these was rendered separately.
    # A list item ends a table; the prose line in arm 13 does not.
    check(
        "a list item ends the table",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n- a list item\n\n| 5 | 6 |\n"),
        0,
    )
    check(
        "a heading ends the table",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n## A heading\n\n| 5 | 6 |\n"),
        0,
    )
    check(
        "a blockquote ends the table",
        n("| A | B |\n| --- | --- |\n| 1 | 2 |\n> quoted\n\n| 5 | 6 |\n"),
        0,
    )

    # ── arms 18–22: the MIRROR — a block a table SWALLOWS ───────────────────
    def a(text):
        return len(absorbed(text))

    # 18 — prose abutting a table is absorbed, one row per LINE.
    check(
        "prose abutting a table is swallowed",
        a("| A | B |\n| --- | --- |\n| 1 | 2 |\nTree complete. It is closed\nand wrapped.\n"),
        2,
    )
    # 19 — THE NEGATIVE: a blank line is exactly what makes it prose again.
    check(
        "a blank line before the block is the fix",
        a("| A | B |\n| --- | --- |\n| 1 | 2 |\n\nTree complete.\n"),
        0,
    )
    # 20 — a list item abutting is NOT absorbed. ⛔ Reading it as absorbed
    #      inflated this population from 11 to 15 on the first pass.
    check(
        "a list abutting a table is not swallowed",
        a("| A | B |\n| --- | --- |\n| 1 | 2 |\n- an item\n- another\n"),
        0,
    )
    # 21 — nor a heading.
    check(
        "a heading abutting a table is not swallowed",
        a("| A | B |\n| --- | --- |\n| 1 | 2 |\n## Changelog\n"),
        0,
    )
    # 22 — and a rowless table swallows the block that follows it, which is the
    #      corpus instance: a header and delimiter with no rows under them.
    check(
        "a ROWLESS table still swallows what abuts it",
        a("| A | B |\n| --- | --- |\nTree complete.\n"),
        1,
    )

    # ── arm 23: the real corpus is reachable ─────────────────────────────────
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
        "BROKEN-TABLE self-test: 23 arms — BOTH directions of the table boundary"
        " (a blank line where none belongs, and none where one belongs), TEN negatives,"
        " the renderer's answers about what a table, a body row and a terminator are,"
        " an escaped pipe, an unbalanced fence refused, and"
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
        if not c["by_file"] and not c["absorbed_detail"]:
            print(
                f"BROKEN-TABLE: OK — {c['files_scanned']} tracked .md scanned, no blank"
                " line inside a table body and no block absorbed by one"
            )
            return 0
        if c["by_file"]:
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
        if c["absorbed_detail"]:
            print(
                "BROKEN-TABLE: a block ABUTS a table with no blank line, so the table"
                " SWALLOWS it — each line renders as a row padded to the header's width.",
                file=sys.stderr,
            )
            for f, hits in c["absorbed_detail"].items():
                print(
                    f"    {f}:{hits[0]['line']} and {len(hits) - 1} further line(s):"
                    f" {hits[0]['text']}",
                    file=sys.stderr,
                )
            print(
                "  Put a blank line between the table and the block — or, if the table"
                " has no rows at all, remove its header and delimiter pair.",
                file=sys.stderr,
            )
        return 1

    print(f"tracked .md scanned              : {c['files_scanned']}")
    print(f"files with a blank inside a table: {c['files_affected']}")
    print(f"blank lines inside a table body  : {c['blank_lines_in_a_table']}")
    print(f"ROWS rendered as literal text    : {c['rows_rendered_as_literal_text']}")
    print(f"lines a table SWALLOWS (no blank) : {c['absorbed_lines']}")
    for f, n in c["absorbed_by_file"].items():
        print(f"  {f}: {n} absorbed line(s)")
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
