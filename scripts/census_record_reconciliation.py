#!/usr/bin/env python3
"""Reconcile the source-census records against the task-tree leaves they route to.

`SIGNOFF-REPAIR.11.9`'s question: a review record names one or more repair
candidate leaves; that leaf is later censused and SPLIT into children; does
every clause of the record survive the split, or does the split census the
leaf's own goal line and silently drop the reviewer's framing?

The mechanical signal is CITATION. When a leaf re-reads the records routed to
it, this project's convention is to name them — `census-2.md:56` or `R-31-32-5`
— as `SIGNOFF-REPAIR.4.2` did for all fifteen of its records. A split leaf that
cites none of its records did not re-read them, whatever else it did well.

⛔ CITATION IS A POPULATION, NOT A VERDICT. An uncited record may still be fully
handled — the leaf simply did not name it. This tool reports which records are
uncited by a split leaf so a human can classify that set; it does not claim they
are unaccounted for (`SIGNOFF-REPAIR.11.4.5.2`: classify the N before publishing
it). "Accounted for" includes DELIBERATELY DECLINED, which no search can see.

    python3 -B scripts/census_record_reconciliation.py             # the census
    python3 -B scripts/census_record_reconciliation.py --uncited    # just the review set
    python3 -B scripts/census_record_reconciliation.py --json
    python3 -B scripts/census_record_reconciliation.py --self-test
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CENSUS_DIR = ROOT / "docs" / "tasks" / "artifacts" / "signoff_review"
TREE_DIR = ROOT / "docs" / "tasks"

# ⚠️ TWO id shapes exist in these artifacts: `R-6-27-2` and `R-31-32`. The first
# version of this regex required three parts and found 70 of 131 — a 47% blind
# spot its own self-test caught only because that control asserted a population
# size rather than re-deriving one from the same regex.
_RECORD = re.compile(r"^## (R-[0-9]+(?:-[0-9]+)+)\s*$")
_CANDIDATES = re.compile(r"^- Repair candidates:\s*(.+?)\s*$")
_LEAF_REF = re.compile(r"`([A-Z0-9-]+(?:\.[0-9]+)+)`")
_HEADING = re.compile(r"^(#{2,6})\s+([A-Z0-9-]+(?:\.[0-9]+)+)\s+—")


def records() -> list[dict]:
    """Every review record: its id, file, line, and the leaves it routes to."""
    out = []
    for path in sorted(CENSUS_DIR.glob("census-*.md")):
        lines = path.read_text().splitlines()
        current = None
        for n, line in enumerate(lines, 1):
            m = _RECORD.match(line)
            if m:
                if current is not None:
                    current["end"] = n - 1
                current = {
                    "id": m.group(1),
                    "file": path.name,
                    "line": n,
                    "end": len(lines),
                    "candidates": [],
                    "body_line": None,
                }
                out.append(current)
                continue
            if current is None:
                continue
            c = _CANDIDATES.match(line)
            if c and not current["candidates"]:
                current["candidates"] = _LEAF_REF.findall(c.group(1))
            elif line.strip() and not line.startswith("- ") and current["body_line"] is None:
                current["body_line"] = n
        if current is not None:
            current["end"] = len(lines)
    return out


def leaf_sections() -> dict[str, str]:
    """Each leaf id -> the text of its own section, across every tracked tree."""
    sections: dict[str, str] = {}
    # ⛔ A leaf's section is its OWN text, ending at the next heading of ANY
    # level — not at the next heading of its own level. Letting a parent run to
    # its next sibling would fold every child's text into it, so a child's
    # citation would count for the parent and no split leaf could ever read as
    # uncited. The first version did exactly that; the `parent-excludes-child`
    # control is what caught it.
    for tree in sorted(TREE_DIR.glob("*.md")):
        lines = tree.read_text().splitlines()
        marks = [(n, m.group(2)) for n, line in enumerate(lines) if (m := _HEADING.match(line))]
        # Any heading at all bounds a section, including non-leaf ones.
        bounds = sorted({n for n, _ in marks} | {
            n for n, line in enumerate(lines) if re.match(r"^#{1,6}\s", line)
        })
        for n, leaf in marks:
            nxt = next((b for b in bounds if b > n), len(lines))
            sections.setdefault(leaf, "")
            sections[leaf] += "\n".join(lines[n:nxt]) + "\n"
    return sections


def descendants(leaf: str, known: set[str]) -> list[str]:
    """Every known leaf id strictly beneath this one."""
    return sorted(k for k in known if k.startswith(leaf + ".") and k != leaf)


# ⚠️ The tree writes RUNS of citations with the filename elided after the first:
# "`census-2.md:28`, `:49`, `:56`, `:70`". A bare `:N` inherits the nearest
# filename to its left. A regex requiring the prefix on every reference finds 3
# of this tree's citations instead of 19 — `SIGNOFF-REPAIR.4.2` names fifteen
# records and every one of the elided ones would read as UNCITED.
_CITE_ANY = re.compile(r"(?:(census-[0-9]+\.md))?:([0-9]+)")


def citations(text: str) -> set[tuple[str, int]]:
    """Every (file, line) citation in the text, resolving the elided form."""
    found: set[tuple[str, int]] = set()
    current: str | None = None
    for m in _CITE_ANY.finditer(text):
        if m.group(1):
            current = m.group(1)
        if current:
            found.add((current, int(m.group(2))))
    return found


def cites(text: str, rec: dict) -> bool:
    """Does this section name the record, by id or by a file:line INSIDE it?

    ⚠️ The line-reference convention points at whatever line the author was
    reading — in practice the `- Repair candidates:` line, not the `## R-…`
    heading. Matching only the heading and the first body line found 3 of the
    tree's citations instead of all of them, and `SIGNOFF-REPAIR.4.2`'s fifteen
    explicit ones read as UNCITED. A reference anywhere within the record's
    line range names that record; that is the rule, and it needs no guess about
    which line an author picked."""
    if rec["id"] in text:
        return True
    for fname, num in citations(text):
        if fname == rec["file"] and rec["line"] <= num <= rec["end"]:
            return True
    return False


def reconcile() -> dict:
    recs = records()
    sections = leaf_sections()
    known = set(sections)
    rows = []
    for rec in recs:
        for leaf in rec["candidates"]:
            kids = descendants(leaf, known)
            scope = sections.get(leaf, "") + "\n".join(sections.get(k, "") for k in kids)
            rows.append(
                {
                    "record": rec["id"],
                    "where": f"{rec['file']}:{rec['line']}",
                    "leaf": leaf,
                    "leaf_exists": leaf in known,
                    "split": len(kids),
                    "cited": cites(scope, rec),
                }
            )
    return {"records": recs, "rows": rows}


def run(mode: str) -> int:
    data = reconcile()
    recs, rows = data["records"], data["rows"]
    split_rows = [r for r in rows if r["split"] > 0]
    uncited_split = [r for r in split_rows if not r["cited"]]
    missing_leaf = [r for r in rows if not r["leaf_exists"]]

    if mode == "json":
        print(json.dumps({"totals": {
            "records": len(recs),
            "routings": len(rows),
            "to_split_leaves": len(split_rows),
            "uncited_by_split_leaf": len(uncited_split),
            "naming_a_leaf_that_does_not_exist": len(missing_leaf),
        }, "rows": rows}, indent=2))
        return 0

    if mode != "uncited":
        print(f"review records                       : {len(recs)}")
        print(f"record -> candidate-leaf routings     : {len(rows)}")
        print(f"  … whose leaf has been SPLIT         : {len(split_rows)}")
        print(f"  … cited by that leaf or a child     : {sum(1 for r in split_rows if r['cited'])}")
        print(f"  … UNCITED by that split leaf        : {len(uncited_split)}")
        print(f"routings naming a leaf that does not exist: {len(missing_leaf)}")
        print()

    print(f"UNCITED by a SPLIT leaf — the set to classify ({len(uncited_split)}):")
    for r in sorted(uncited_split, key=lambda r: (r["leaf"], r["where"])):
        print(f"  {r['leaf']:<34} <- {r['record']:<12} {r['where']}  (leaf has {r['split']} children)")
    if missing_leaf:
        print()
        print(f"routings to a leaf id with no heading anywhere ({len(missing_leaf)}):")
        for r in sorted(missing_leaf, key=lambda r: r["leaf"]):
            print(f"  {r['leaf']:<34} <- {r['record']:<12} {r['where']}")
    print()
    print("⛔ UNCITED is a POPULATION, not a defect count: a record can be fully handled")
    print("   by a leaf that never named it, and 'accounted for' includes deliberately")
    print("   DECLINED, which no search can see. Classify before concluding.")
    return 0


def self_test() -> int:
    failures = []

    def check(name, got, want):
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    recs = records()
    # The population is read from the real artifacts, and a known record resolves.
    check("records-found", len(recs) > 100, True)
    by_id = {r["id"]: r for r in recs}
    check("known-record-present", "R-31-32-5" in by_id, True)
    check("known-record-routes", "SIGNOFF-REPAIR.4.1" in by_id["R-31-32-5"]["candidates"], True)
    # Every record names at least one candidate — the parser's own coverage.
    check("all-records-have-candidates", [r["id"] for r in recs if not r["candidates"]], [])

    sections = leaf_sections()
    check("leaf-section-found", "SIGNOFF-REPAIR.4.2" in sections, True)
    # A parent's section must NOT swallow its children's text, or every child's
    # citation would count for the parent and nothing would ever read as uncited.
    check(
        "parent-excludes-child",
        "REPAIR-0152" in sections.get("SIGNOFF-REPAIR.4.2", ""),
        False,
    )
    check("child-has-own-text", "REPAIR-0152" in sections.get("SIGNOFF-REPAIR.4.2.1", ""), True)
    # Descendant walking is by dotted prefix, and `.4.2` must not claim `.4.20`.
    known = {"A.4.2", "A.4.2.1", "A.4.2.1.1", "A.4.20", "A.4"}
    check("descendants", descendants("A.4.2", known), ["A.4.2.1", "A.4.2.1.1"])
    # Citation matches both conventions, and nothing else.
    rec = {"id": "R-1-2-3", "file": "census-9.md", "line": 12, "end": 18, "body_line": 16}
    check("cite-by-id", cites("… see `R-1-2-3` …", rec), True)
    check("cite-by-heading-line", cites("… from `census-9.md:12` …", rec), True)
    check("cite-by-candidates-line", cites("… from `census-9.md:14` …", rec), True)
    check("cite-by-body-line", cites("… from `census-9.md:16` …", rec), True)
    check("cite-at-range-end", cites("… from `census-9.md:18` …", rec), True)
    check("no-cite-past-range", cites("… from `census-9.md:19` …", rec), False)
    check("no-cite-before-range", cites("… from `census-9.md:11` …", rec), False)
    check("no-false-cite-other-file", cites("… from `census-8.md:12` …", rec), False)
    check("no-false-cite-bare", cites("… census-9.md …", rec), False)
    check("cite-elided-filename", cites("`census-9.md:3`, `:14`", rec), True)
    check("elided-does-not-leak-backwards", cites("`:14`, `census-9.md:3`", rec), False)
    # The elided form must inherit the NEAREST filename to its left, not the
    # first one in the text. Expressed against a record in the EARLIER file, so
    # a wrong inheritance would show up as a false positive rather than being
    # masked by the right answer.
    earlier = {"id": "R-9-9", "file": "census-8.md", "line": 12, "end": 18, "body_line": 14}
    check(
        "elided-inherits-nearest-not-first",
        cites("`census-8.md:99`, `census-9.md:99`, `:14`", earlier),
        False,
    )
    check(
        "elided-inherits-nearest-positive",
        cites("`census-9.md:99`, `census-8.md:99`, `:14`", earlier),
        True,
    )
    # The real tree: `SIGNOFF-REPAIR.4.2` names fifteen records explicitly, so
    # its own routings must read as cited. This control is what proved the
    # heading-only matcher wrong.
    live = {r["id"]: r for r in recs}
    sec42 = sections.get("SIGNOFF-REPAIR.4.2", "")
    check("real-tree-citation", cites(sec42, live["R-48-49-6"]), True)

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print("census_record_reconciliation --self-test: 21 controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    if "--json" in args:
        return run("json")
    return run("uncited" if "--uncited" in args else "full")


if __name__ == "__main__":
    raise SystemExit(main())
