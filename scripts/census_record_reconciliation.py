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
    python3 -B scripts/census_record_reconciliation.py --rank       # the tranche ranking
    python3 -B scripts/census_record_reconciliation.py --classified # against the ledger
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
#
# 🔴 And the elided form MUST be anchored to the opening backtick it is always
# written with. The first version matched any `:N`, so a leaf that cited a
# census file and then cited SOURCE line numbers — `api.rs:1467`, or five
# `scripts/check_*.sh:NN` — had every one of them inherit the census filename.
# Measured when two leaves written on 2026-09-13 did exactly that: eight bogus
# references, four of them landing inside real records, which silently moved the
# headline population from 114 to 111. The run convention puts each elided
# reference in its own code span, so the backtich is the discriminator that
# costs nothing and the `source-path-is-not-an-elided-citation` control is what
# holds it.
_CITE_ANY = re.compile(r"(census-[0-9]+\.md):([0-9]+)|`:([0-9]+)")


def citations(text: str) -> set[tuple[str, int]]:
    """Every (file, line) citation in the text, resolving the elided form."""
    found: set[tuple[str, int]] = set()
    current: str | None = None
    for m in _CITE_ANY.finditer(text):
        if m.group(1):
            current = m.group(1)
            found.add((current, int(m.group(2))))
        elif current:
            found.add((current, int(m.group(3))))
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


LEDGER = ROOT / "docs" / "tasks" / "artifacts" / "signoff_review" / "RECONCILIATION.md"

# ⛔ The closed set of clause states. A ledger row outside it is a breach, not a
# new category: the whole value of the ledger is that a reader can take the six
# words at face value. `none` is the only state with no owning leaf, and it must
# spell that absence as an em dash rather than leaving the cell blank.
STATES = {"handled", "owned", "attach", "unowned", "declined", "none"}

# A ledger row: | `R-…` | <ordinal> | <state> | `LEAF` or — | evidence |
_LEDGER_ROW = re.compile(
    r"^\|\s*`(R-[0-9]+(?:-[0-9]+)+)`\s*\|\s*([0-9]+)\s*\|\s*([a-z]+)\s*\|\s*"
    r"(?:`([A-Z0-9-]+(?:\.[0-9]+)+)`|(\u2014))\s*\|"
)


def ledger() -> list[dict]:
    """Every clause row of the reconciliation ledger, as written."""
    if not LEDGER.exists():
        return []
    out = []
    for n, line in enumerate(LEDGER.read_text().splitlines(), 1):
        m = _LEDGER_ROW.match(line)
        if m:
            out.append(
                {
                    "line": n,
                    "record": m.group(1),
                    "clause": int(m.group(2)),
                    "state": m.group(3),
                    "owner": m.group(4),
                }
            )
    return out


def ledger_breaches(rows: list[dict], known_records: set[str], known_leaves: set[str]) -> list[str]:
    """Every way a ledger row can be wrong, named rather than counted."""
    out = []
    seen: set[tuple[str, int]] = set()
    for r in rows:
        where = f"{LEDGER.name}:{r['line']}"
        if r["record"] not in known_records:
            out.append(f"{where}: names {r['record']}, which no census record carries")
        if r["state"] not in STATES:
            out.append(f"{where}: state {r['state']!r} is outside the closed set")
        if r["state"] == "none":
            if r["owner"] is not None:
                out.append(f"{where}: state 'none' names an owner")
        elif r["owner"] is None:
            out.append(f"{where}: state {r['state']!r} names no owner")
        elif r["owner"] not in known_leaves:
            out.append(f"{where}: owner {r['owner']} is not a leaf of any tracked tree")
        key = (r["record"], r["clause"])
        if key in seen:
            out.append(f"{where}: {r['record']} clause {r['clause']} appears twice")
        seen.add(key)
    return out


def narrowest(rows: list[dict]) -> dict[str, tuple[str, int]]:
    """Per record: its narrowest candidate leaf and how many records name that leaf.

    ⭐ The RANKING, and it is the opposite of the obvious one. A record's own
    fan-out measures how sure the REVIEWER was; this measures how much the
    TARGET depends on the record. Measured over this corpus the two are
    anti-correlated — every fan-out-1 record names a container leaf that 22 to
    99 records also name — so ranking by fan-out puts the least reconcilable
    records first (`SIGNOFF-REPAIR.11.9.1`).
    """
    freq: dict[str, int] = {}
    for r in rows:
        freq[r["leaf"]] = freq.get(r["leaf"], 0) + 1
    out = {}
    for r in rows:
        best = out.get(r["record"])
        if best is None or freq[r["leaf"]] < best[1]:
            out[r["record"]] = (r["leaf"], freq[r["leaf"]])
    return out


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


def uncited_records(rows: list[dict]) -> list[str]:
    """Records cited by NONE of their candidate leaves — the 114 to classify.

    ⛔ Record granularity, not routing granularity. The claim this set supports
    is about a RECORD ("did anyone re-read it?"), so it is measured at the
    record's own level (`docs/CLAIM_VERIFICATION.md` leg 1).
    """
    cited: set[str] = {r["record"] for r in rows if r["cited"]}
    return sorted({r["record"] for r in rows} - cited)


def run(mode: str) -> int:
    data = reconcile()
    recs, rows = data["records"], data["rows"]
    split_rows = [r for r in rows if r["split"] > 0]
    uncited_split = [r for r in split_rows if not r["cited"]]
    missing_leaf = [r for r in rows if not r["leaf_exists"]]

    if mode == "rank":
        rank = narrowest(rows)
        pending = uncited_records(rows)
        print(f"UNCITED records ranked by their NARROWEST candidate leaf ({len(pending)}):")
        print()
        for rid in sorted(pending, key=lambda r: (rank[r][1], r)):
            leaf, n = rank[rid]
            fan = sum(1 for r in rows if r["record"] == rid)
            print(f"  {rid:<12} narrowest={leaf:<30} named by {n:>3} records   (own fan-out {fan})")
        print()
        print("⭐ Rank by THIS, not by the record's own fan-out. The two are")
        print("   anti-correlated over this corpus: a record naming one leaf named a")
        print("   CONTAINER, so it is one of dozens that leaf could never cite each.")
        return 0

    if mode == "classified":
        pending = uncited_records(rows)
        known_records = {r["id"] for r in recs}
        known_leaves = set(leaf_sections())
        led = ledger()
        breaches = ledger_breaches(led, known_records, known_leaves)
        done = {r["record"] for r in led}
        remaining = [r for r in pending if r not in done]
        counts: dict[str, int] = {}
        for r in led:
            counts[r["state"]] = counts.get(r["state"], 0) + 1
        print(f"uncited records STILL to classify     : {len(pending)}")
        print(f"  … carrying at least one ledger row  : {len(pending) - len(remaining)}")
        print(f"  … NOT yet classified                : {len(remaining)}")
        print(f"records with a ledger row             : {len(done)}")
        print(f"  … of them, no longer uncited        : {len(done - set(pending))}")
        print(f"ledger clause rows                    : {len(led)}")
        for state in sorted(STATES):
            print(f"  {state:<10}                          : {counts.get(state, 0)}")
        print()
        print("⭐ The two measures CONVERGE, and that is the mechanism working rather")
        print("   than a discrepancy: classifying a record's clauses gives each an owner,")
        print("   the owning leaf then NAMES the record, and the record leaves the")
        print("   uncited population. A row here that is still uncited is one whose")
        print("   clauses were all already handled or owned elsewhere.")
        print()
        if breaches:
            print(f"LEDGER BREACHES ({len(breaches)}):")
            for b in breaches:
                print(f"  {b}")
            print()
            return 1
        print("ledger: every row names a real record, a state in the closed set and a real owner")
        return 0

    if mode == "json":
        print(json.dumps({"totals": {
            "records": len(recs),
            "routings": len(rows),
            "to_split_leaves": len(split_rows),
            "uncited_by_split_leaf": len(uncited_split),
            "naming_a_leaf_that_does_not_exist": len(missing_leaf),
            "uncited_records": len(uncited_records(rows)),
            "ledger_clause_rows": len(ledger()),
            "records_with_a_ledger_row": len({r["record"] for r in ledger()}),
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
    # 🔴 The control the first version did not have, and the defect it missed:
    # a SOURCE path's line number after a census citation is not an elided
    # reference. Both spellings a leaf actually writes are covered.
    check(
        "source-path-is-not-an-elided-citation",
        cites("see `census-9.md:3` and `crates/x/src/api.rs:14`", rec),
        False,
    )
    check(
        "bare-source-path-is-not-an-elided-citation",
        cites("see `census-9.md:3`, then scripts/check_x.sh:14", rec),
        False,
    )
    # The real tree: `SIGNOFF-REPAIR.4.2` names fifteen records explicitly, so
    # its own routings must read as cited. This control is what proved the
    # heading-only matcher wrong.
    live = {r["id"]: r for r in recs}
    sec42 = sections.get("SIGNOFF-REPAIR.4.2", "")
    check("real-tree-citation", cites(sec42, live["R-48-49-6"]), True)

    # ── the ledger (`SIGNOFF-REPAIR.11.9.1`) ────────────────────────────────
    # Parsing is asserted against SYNTHETIC rows, and every refusal is fired at
    # least once: a control never observed RED is not known to work
    # (`docs/CLAIM_VERIFICATION.md` leg 2).
    def parse_one(line: str):
        m = _LEDGER_ROW.match(line)
        if not m:
            return None
        return {"line": 1, "record": m.group(1), "clause": int(m.group(2)),
                "state": m.group(3), "owner": m.group(4)}

    row = parse_one("| `R-1-2` | 3 | owned | `A.1.2` | because |")
    check("ledger-row-record", row and row["record"], "R-1-2")
    check("ledger-row-clause", row and row["clause"], 3)
    check("ledger-row-state", row and row["state"], "owned")
    check("ledger-row-owner", row and row["owner"], "A.1.2")
    check("ledger-row-emdash-owner",
          parse_one("| `R-1-2` | 1 | none | \u2014 | no finding |"), 
          {"line": 1, "record": "R-1-2", "clause": 1, "state": "none", "owner": None})
    # The ledger's own header, separator and vocabulary table must not parse as
    # clause rows — they are prose about the format, not entries in it.
    check("ledger-skips-header", parse_one("| Record | Clause | State | Owner | Evidence |"), None)
    check("ledger-skips-separator", parse_one("| --- | --- | --- | --- | --- |"), None)
    check("ledger-skips-vocabulary", parse_one("| `handled` | a leaf did the work | none |"), None)

    known_r, known_l = {"R-1-2"}, {"A.1.2"}
    def one_breach(line):
        r = parse_one(line)
        return ledger_breaches([r], known_r, known_l) if r else ["unparsed"]

    check("ledger-clean-row", one_breach("| `R-1-2` | 1 | owned | `A.1.2` | why |"), [])
    check("ledger-rejects-unknown-record",
          len(one_breach("| `R-9-9` | 1 | owned | `A.1.2` | why |")), 1)
    check("ledger-rejects-unknown-owner",
          len(one_breach("| `R-1-2` | 1 | owned | `A.9.9` | why |")), 1)
    # An out-of-set state never reaches the state cell's `[a-z]+`… it does, so
    # the closed-set check is what refuses it.
    check("ledger-rejects-unknown-state",
          len(one_breach("| `R-1-2` | 1 | resolved | `A.1.2` | why |")), 1)
    check("ledger-rejects-none-with-owner",
          len(one_breach("| `R-1-2` | 1 | none | `A.1.2` | why |")), 1)
    check("ledger-rejects-emdash-for-a-real-state",
          len(one_breach("| `R-1-2` | 1 | owned | \u2014 | why |")), 1)
    dup = [parse_one("| `R-1-2` | 1 | owned | `A.1.2` | a |"),
           parse_one("| `R-1-2` | 1 | handled | `A.1.2` | b |")]
    check("ledger-rejects-duplicate-clause", len(ledger_breaches(dup, known_r, known_l)), 1)

    # The ranking, on a fixture whose answer is the OPPOSITE of the fan-out
    # ranking — which is the whole point of the measurement that chose it.
    rank_rows = [
        {"record": "R-a", "leaf": "T.1"},                        # fan-out 1, container
        {"record": "R-b", "leaf": "T.1"},
        {"record": "R-c", "leaf": "T.1"},
        {"record": "R-c", "leaf": "T.2"},                        # fan-out 2, narrow leaf
    ]
    ranked = narrowest(rank_rows)
    check("narrowest-picks-the-rarest-leaf", ranked["R-c"], ("T.2", 1))
    check("narrowest-container-for-fanout-one", ranked["R-a"], ("T.1", 3))

    # Record granularity: one cited candidate makes the RECORD cited, even when
    # another candidate did not cite it. The claim is about the record.
    mixed = [{"record": "R-x", "cited": True}, {"record": "R-x", "cited": False},
             {"record": "R-y", "cited": False}]
    check("uncited-is-per-record", uncited_records(mixed), ["R-y"])

    # The REAL ledger, against the REAL trees — the one control here with no
    # loyalty to the parser's fixtures (`TOOLBOX.md`: a self-test written
    # alongside the code shares its blind spots).
    live_led = ledger()
    check("live-ledger-has-rows", len(live_led) > 0, True)
    check("live-ledger-is-clean",
          ledger_breaches(live_led, {r["id"] for r in recs}, set(sections)), [])

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print("census_record_reconciliation --self-test: 42 controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    if "--json" in args:
        return run("json")
    if "--rank" in args:
        return run("rank")
    if "--classified" in args:
        return run("classified")
    return run("uncited" if "--uncited" in args else "full")


if __name__ == "__main__":
    raise SystemExit(main())
