#!/usr/bin/env python3
"""scripts/census_open_leaves.py — the open-leaf count the corrective exit bar is
(`SIGNOFF-REPAIR.12.2`).

`docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md` ends the
corrective tree when every leaf in a blocking class is closed. That makes the
exit a COUNT, and nothing derived it: a count reading only `- Status:` lines
reported 50 open leaves where 65 were open, because a leaf may record its state
in any of three forms and `TASK-STATUS` requires none of them.

    python3 -B scripts/census_open_leaves.py              # the census
    python3 -B scripts/census_open_leaves.py --check      # the gate
    python3 -B scripts/census_open_leaves.py --json
    python3 -B scripts/census_open_leaves.py --self-test

A leaf is a heading `## … ###### <TREE>.<n>… — <title>`; its section runs to the
next heading of any level OUTSIDE a fenced code block. ⚠️ A shell comment inside
a fence (`# over the last 200 commits …`) is not a heading: the first version of
this census ended three leaves' sections at one, lost the `done` Status line
written after the fence, and reported all three open. Its state is read from the FIRST of:

    - Status: `<state>` …          the state named
    - Opened and closed …           closed, inline
    - Opened: `<state>` …           the state named

and `done` is the only closed state. Its class comes from its `- ⚖️ Bar` line:
**blocking** with its class numbers, **deferred** (with or without "with a
trigger"), or **structural**.

⛔ The gate refuses two things, and only two: an OPEN leaf with no bar line (the
exit bar cannot count what nobody classified), and a leaf whose state cannot be
read (it would be counted as whatever a reader guessed). It does not judge
whether a class is right; that is the bar's decision, made in the leaf.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TREE = ROOT / "docs" / "tasks" / "SIGNOFF-REPAIR.md"

HEADING = re.compile(r"^(#{2,6}) ([A-Z][A-Z0-9-]*(?:\.\d+)+) — ")
ANY_HEADING = re.compile(r"^#{1,6} ")
STATUS = re.compile(r"^- Status: `([^`]+)`")
OPENED = re.compile(r"^- Opened: `([^`]+)`")
OPENED_AND_CLOSED = re.compile(r"^- Opened and closed\b")
BAR = re.compile(r"^- ⚖️ Bar\b")
BLOCKING = re.compile(r"\*\*blocking\*\*,?\s*class(?:es)?\s+([0-9](?:\s*(?:,|and|/)\s*[0-9])*)")
CLOSED_STATES = {"done"}


def leaves(text: str) -> list[dict]:
    """Every leaf with its line, state, openness and bar."""
    found: list[dict] = []
    current: dict | None = None
    fenced = False
    for number, line in enumerate(text.split("\n"), 1):
        if line.lstrip().startswith("```"):
            fenced = not fenced
        if fenced or line.lstrip().startswith("```"):
            if current is not None:
                current["body"].append(line)
            continue
        match = HEADING.match(line)
        if match:
            current = {"id": match.group(2), "line": number, "body": []}
            found.append(current)
            continue
        if ANY_HEADING.match(line):
            current = None
            continue
        if current is not None:
            current["body"].append(line)
    for leaf in found:
        state, form = None, None
        for line in leaf["body"]:
            if (m := STATUS.match(line)):
                state, form = m.group(1), "status"
                break
        if state is None:
            for line in leaf["body"]:
                if OPENED_AND_CLOSED.match(line):
                    state, form = "done", "opened-and-closed"
                    break
                if (m := OPENED.match(line)):
                    state, form = m.group(1), "opened"
                    break
        leaf["state"], leaf["form"] = state, form
        leaf["open"] = state is not None and state not in CLOSED_STATES
        bar = next((line for line in leaf["body"] if BAR.match(line)), None)
        leaf["bar"] = classify(bar)
        del leaf["body"]
    return found


def classify(bar: str | None) -> dict | None:
    if bar is None:
        return None
    if (m := BLOCKING.search(bar)):
        return {"kind": "blocking", "classes": sorted({int(c) for c in re.findall(r"[0-9]", m.group(1))})}
    if "**deferred" in bar:
        return {"kind": "deferred"}
    if "**structural**" in bar:
        return {"kind": "structural"}
    return {"kind": "unrecognised"}


def census(text: str) -> dict:
    all_leaves = leaves(text)
    open_leaves = [leaf for leaf in all_leaves if leaf["open"]]
    unreadable = [leaf for leaf in all_leaves if leaf["state"] is None]
    unbarred = [leaf for leaf in open_leaves if leaf["bar"] is None]
    by_kind: dict[str, list[dict]] = {}
    for leaf in open_leaves:
        kind = leaf["bar"]["kind"] if leaf["bar"] else "no bar"
        by_kind.setdefault(kind, []).append(leaf)
    by_class: dict[int, list[str]] = {}
    for leaf in by_kind.get("blocking", []):
        by_class.setdefault(min(leaf["bar"]["classes"]), []).append(leaf["id"])
    return {
        "leaves": len(all_leaves),
        "open": len(open_leaves),
        "status_only_open": sum(1 for leaf in open_leaves if leaf["form"] == "status"),
        "by_kind": {kind: [leaf["id"] for leaf in items] for kind, items in sorted(by_kind.items())},
        "blocking_by_lowest_class": {str(k): v for k, v in sorted(by_class.items())},
        "unbarred_open": [f"{leaf['id']} (line {leaf['line']}, {leaf['state']})" for leaf in unbarred],
        "unreadable": [f"{leaf['id']} (line {leaf['line']})" for leaf in unreadable],
    }


def breaches(result: dict) -> list[tuple[str, str]]:
    """What the gate refuses, as (why, leaf) pairs; the self-test holds it too."""
    failures = [("an open leaf with no bar line", item) for item in result["unbarred_open"]]
    failures += [("a leaf whose state cannot be read", item) for item in result["unreadable"]]
    return failures


def report(result: dict) -> None:
    print(f"open leaves: {result['open']} of {result['leaves']} "
          f"({result['status_only_open']} of them visible to a Status-only count)")
    for kind, ids in result["by_kind"].items():
        print(f"  {kind}: {len(ids)}")
    for klass, ids in result["blocking_by_lowest_class"].items():
        print(f"  blocking, class {klass}: {len(ids)} — {', '.join(ids)}")


def self_test() -> int:
    tree = "\n".join([
        "## T.1 — a status leaf", "- Status: `pending`", "- ⚖️ Bar (`X`): **blocking**, class 2 and 3, why.",
        "### T.1.1 — an opened leaf", "- Opened: `pending` by T.1.", "- ⚖️ Bar (`X`): **deferred**, trigger: t.",
        "### T.1.2 — opened and closed", "- Opened and closed by T.1; REPAIR-1.",
        "### T.1.3 — opened done", "- Opened: `done`; REPAIR-2.",
        "### T.1.4 — Status wins over Opened", "- Opened: `pending`.", "- Status: `done` — REPAIR-3.",
        "### T.1.5 — open and unclassified", "- Status: `active`.",
        "### T.1.6 — no state at all", "- Owns: something.",
        "### T.1.7 — structural", "- Status: `active`.", "- ⚖️ Bar (`X`): **structural**: closes with its children.",
        "#### Commit acceptance — not a leaf", "- Status: `pending` is not T.1.7's second line",
        "### T.1.8 — class four", "- Status: `blocked`.", "- ⚖️ Bar (`X`): **blocking**, class 4, a gate.",
        "### T.1.9 — a fence holds a shell comment", "- Opened: `pending` by T.1.", "```bash",
        "# over the last 200 commits, a comment and not a heading", "```", "- Status: `done`.",
    ])
    result = census(tree)
    assert result["leaves"] == 10, result
    assert result["open"] == 5, result  # T.1, T.1.1, T.1.5, T.1.7, T.1.8
    assert result["status_only_open"] == 4, result  # T.1.1 is visible only through Opened
    assert result["by_kind"] == {"blocking": ["T.1", "T.1.8"], "deferred": ["T.1.1"],
                                 "no bar": ["T.1.5"], "structural": ["T.1.7"]}, result["by_kind"]
    assert result["blocking_by_lowest_class"] == {"2": ["T.1"], "4": ["T.1.8"]}, result
    assert result["unbarred_open"] == ["T.1.5 (line 14, active)"], result
    assert result["unreadable"] == ["T.1.6 (line 16)"], result
    assert breaches(result) == [("an open leaf with no bar line", "T.1.5 (line 14, active)"),
                                ("a leaf whose state cannot be read", "T.1.6 (line 16)")], breaches(result)
    assert classify("- ⚖️ Bar: **blocking**, class 4 and 3, a lying test.") == {"kind": "blocking", "classes": [3, 4]}
    assert classify("- ⚖️ Bar: **deferred with a trigger**, below the bar.") == {"kind": "deferred"}
    print("census_open_leaves: self-test OK (10 leaves, 3 state forms, a non-leaf heading, a fenced comment, 4 bar kinds)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    tree = DEFAULT_TREE
    if "--tree" in argv:
        tree = Path(argv[argv.index("--tree") + 1])
    result = census(tree.read_text(encoding="utf-8"))
    if "--json" in argv:
        print(json.dumps(result, indent=2))
        return 0
    if "--check" in argv:
        failures = breaches(result)
        for why, item in failures:
            print(f"OPEN-LEAF-CENSUS: {why} — {item}", file=sys.stderr)
        if failures:
            print(f"OPEN-LEAF-CENSUS: {len(failures)} breach(es). Give each open leaf its ⚖️ Bar "
                  "line and each leaf a readable state (docs/decisions/2026-09-25_the-corrective-"
                  "tree-ends-at-a-bug-bar.md).", file=sys.stderr)
            return 1
        print(f"OPEN-LEAF-CENSUS: OK — {result['open']} open of {result['leaves']}, every open leaf classified")
        return 0
    report(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
