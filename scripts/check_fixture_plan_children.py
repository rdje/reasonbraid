#!/usr/bin/env python3
"""scripts/check_fixture_plan_children.py — FIXTURE-PLAN-CHILDREN doctrine.

A test fixture's cleanup plan names the tables it deletes, in order, and the
shared runtime checker refuses a plan whose declared tables omit a real foreign-key
child — dependents must precede parents, cascading ones included.

⛔ THAT CHECKER ONLY RUNS WHEN A SUITE RUNS, and that is the whole gap this gate
closes. `migrations/0062` added `evidence_citations` with a key to `tenants` and
swept only the plans naming `evidence_snapshots`; six suites could not start from
that commit until `SIGNOFF-REPAIR.11.14.1.1` — roughly twenty commits — because
nobody ran them. Then `migrations/0067` added `reference_registrations` with keys
to `resource_references` AND `tenants`, and the same author swept the plans
naming the first and not the second, in the same session in which the rule about
sweep keys was written down.

So the rule is not new and the checker is not wrong. What was missing is a
TRIGGER at commit time, and it needs no database: every foreign key this corpus
declares is inline in a `CREATE TABLE`, so the graph is readable from the
migrations.

    python3 -B scripts/check_fixture_plan_children.py            # the gate
    python3 -B scripts/check_fixture_plan_children.py --json     # the census
    python3 -B scripts/check_fixture_plan_children.py --self-test

⚠️ THIS INSTRUMENT REFUSES RATHER THAN UNDER-REPORTS, because an instrument that
silently skips what it cannot read is the failure it exists to catch:

  * a foreign key added by `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY` is not
    modelled, so its presence FAILS the gate with that reason rather than
    quietly leaving an edge out of the graph;
  * a `delete_tables(` call site whose argument shape does not parse FAILS the
    gate rather than being skipped. Two files are excluded BY PATH and by name —
    the helper's own definition, and its negative-control tests, which declare
    deliberately-incomplete plans to prove the checker refuses them. Nothing
    else is excluded, and an unreadable site is an error.

⛔ Numbers move with the tree; `--json` re-derives them. At the commit that added
this gate: 45 inline foreign keys, 0 via ALTER, 16 parent tables, 29 declared
plans, 0 refused.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

# The helper that IMPLEMENTS the plan checker, and the suite that proves it
# refuses bad plans. Both name `delete_tables(` and neither declares a real
# fixture plan; `cleanup_tests.rs` deliberately declares incomplete ones.
EXCLUDED = {
    "crates/reasonbraid-server/tests/support/cleanup.rs",
    "crates/reasonbraid-server/tests/support/cleanup_tests.rs",
}

CREATE_TABLE = re.compile(
    r"CREATE TABLE (?:IF NOT EXISTS )?(\w+)\s*\((.*?)\n\);", re.S
)
REFERENCES = re.compile(r"REFERENCES\s+(\w+)")
ALTER_FK = re.compile(
    r"ALTER TABLE\s+\w+[^;]*?ADD\s+CONSTRAINT[^;]*?FOREIGN\s+KEY", re.S | re.I
)
CALL_SITE = re.compile(r"delete_tables\(")
PLAN = re.compile(r"delete_tables\(\s*&?\w+\s*,\s*&\[(.*?)\]\s*,?\s*\)", re.S)
QUOTED = re.compile(r'"([a-z_]+)"')


def strip_sql_comments(text: str) -> str:
    """`-- …` to end of line. A `REFERENCES` inside a comment is prose."""
    return "\n".join(line.split("--", 1)[0] for line in text.splitlines())


def fk_graph(migrations: dict[str, str]) -> tuple[dict[str, set[str]], int]:
    """parent -> {children}, plus the inline edge count."""
    children: dict[str, set[str]] = {}
    edges = 0
    for _, raw in sorted(migrations.items()):
        body = strip_sql_comments(raw)
        for match in CREATE_TABLE.finditer(body):
            child, columns = match.group(1), match.group(2)
            for ref in REFERENCES.finditer(columns):
                parent = ref.group(1)
                if parent == child:
                    continue  # a self-reference orders nothing between tables
                children.setdefault(parent, set()).add(child)
                edges += 1
    return children, edges


def alter_added_keys(migrations: dict[str, str]) -> list[str]:
    return [
        name
        for name, raw in sorted(migrations.items())
        if ALTER_FK.search(strip_sql_comments(raw))
    ]


def parse_plans(sources: dict[str, str]) -> tuple[list[tuple[str, list[str]]], list[str]]:
    """Every declared plan, and every call site this parser could not read."""
    plans: list[tuple[str, list[str]]] = []
    unreadable: list[str] = []
    for path, text in sorted(sources.items()):
        if path in EXCLUDED:
            continue
        sites = len(CALL_SITE.findall(text))
        parsed = PLAN.findall(text)
        if len(parsed) != sites:
            unreadable.append(f"{path}: {sites} call site(s), {len(parsed)} readable")
        for body in parsed:
            plans.append((path, QUOTED.findall(body)))
    return plans, unreadable


def audit(migrations: dict[str, str], sources: dict[str, str]) -> dict:
    altered = alter_added_keys(migrations)
    children, edges = fk_graph(migrations)
    plans, unreadable = parse_plans(sources)
    refused = []
    for path, tables in plans:
        named = set(tables)
        missing = sorted(
            {c for t in tables for c in children.get(t, set()) if c not in named}
        )
        if missing:
            refused.append({"plan": path, "missing": missing})
    return {
        "inline_edges": edges,
        "alter_added": altered,
        "parents": len(children),
        "plans": len(plans),
        "unreadable": unreadable,
        "refused": refused,
    }


def read_tree() -> tuple[dict[str, str], dict[str, str]]:
    tracked = subprocess.run(
        ["git", "ls-files", "migrations/*.sql", "*.rs"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    migrations, sources = {}, {}
    for path in tracked:
        text = Path(path).read_text(encoding="utf-8", errors="replace")
        if path.startswith("migrations/"):
            migrations[path] = text
        elif "delete_tables(" in text:
            sources[path] = text
    return migrations, sources


def report(result: dict) -> int:
    if result["alter_added"]:
        print(
            "FIXTURE-PLAN-CHILDREN: this gate does not model foreign keys added by "
            "ALTER TABLE, and these migrations add one:\n  "
            + "\n  ".join(result["alter_added"])
            + "\n\n  Its foreign-key graph is read from inline `REFERENCES` clauses in\n"
            "  `CREATE TABLE`. Rather than leave an edge out and under-report, it stops.\n"
            "  Teach `fk_graph` the ALTER form, or move the key inline.",
            file=sys.stderr,
        )
        return 1
    if result["unreadable"]:
        print(
            "FIXTURE-PLAN-CHILDREN: a `delete_tables(` call site this gate cannot read:\n  "
            + "\n  ".join(result["unreadable"])
            + "\n\n  A site it skips is a plan it does not check, which is the exact\n"
            "  failure this gate exists to prevent — so an unreadable shape is an\n"
            "  error, not an omission. Teach `PLAN` the shape, or, if the file\n"
            "  declares no real fixture plan, add it to EXCLUDED with the reason.",
            file=sys.stderr,
        )
        return 1
    if result["refused"]:
        lines = [
            f"  {r['plan']} omits {', '.join(r['missing'])}" for r in result["refused"]
        ]
        print(
            "FIXTURE-PLAN-CHILDREN: a cleanup plan deletes a parent without deleting "
            "a child that references it:\n" + "\n".join(lines) + "\n\n"
            "  The shared runtime checker refuses such a plan BEFORE its first\n"
            "  deletion, so the suite cannot start — and it only says so when the\n"
            "  suite runs. Add each missing table to the plan, ahead of its parent.\n\n"
            "  ⭐ If you have just added a table, sweep on ITS OWN foreign keys —\n"
            "  every one of them. Six suites were dead for roughly twenty commits\n"
            "  because a sweep was keyed on the table its author had in mind.",
            file=sys.stderr,
        )
        return 1
    return 0


def self_test() -> int:
    good_migrations = {
        "migrations/0001_x.sql": (
            "CREATE TABLE tenants (\n    tenant_id TEXT PRIMARY KEY\n);\n"
            "CREATE TABLE notes (\n"
            "    note_id TEXT PRIMARY KEY,\n"
            "    tenant_id TEXT NOT NULL REFERENCES tenants (tenant_id)\n"
            ");\n"
        )
    }
    complete = {"t/a.rs": 'delete_tables(&pool, &["notes", "tenants"]).await;'}
    incomplete = {"t/a.rs": 'delete_tables(&pool, &["tenants"]).await;'}

    checks = 0

    # 1. a complete plan passes, and 2. an incomplete one is refused by name.
    ok = audit(good_migrations, complete)
    assert ok["refused"] == [], ok
    assert ok["plans"] == 1 and ok["inline_edges"] == 1, ok
    checks += 2
    bad = audit(good_migrations, incomplete)
    assert bad["refused"] == [{"plan": "t/a.rs", "missing": ["notes"]}], bad
    checks += 1

    # 3. a `REFERENCES` inside a comment is prose, not an edge.
    commented = {
        "migrations/0001_x.sql": (
            "CREATE TABLE tenants (\n    tenant_id TEXT PRIMARY KEY\n);\n"
            "-- notes REFERENCES tenants, one day\n"
        )
    }
    assert audit(commented, incomplete)["refused"] == [], "a comment is not an edge"
    checks += 1

    # 4. an ALTER-added key stops the gate instead of silently missing an edge.
    altered = dict(good_migrations)
    altered["migrations/0002_y.sql"] = (
        "ALTER TABLE notes ADD CONSTRAINT fk FOREIGN KEY (tenant_id) "
        "REFERENCES tenants (tenant_id);\n"
    )
    assert audit(altered, complete)["alter_added"] == ["migrations/0002_y.sql"]
    checks += 1

    # 5. an unreadable call site is an error, not a skip.
    odd = {"t/a.rs": "delete_tables(&pool, PLAN_CONST).await;"}
    assert audit(good_migrations, odd)["unreadable"], "an unreadable site must be named"
    checks += 1

    # 6. the two excluded files are excluded by path, not by luck.
    excluded = {p: 'delete_tables(&pool, PLAN_CONST).await;' for p in EXCLUDED}
    assert audit(good_migrations, excluded)["unreadable"] == [], EXCLUDED
    checks += 1

    # 7. a self-referencing key orders nothing and must not be demanded.
    selfref = {
        "migrations/0001_x.sql": (
            "CREATE TABLE nodes (\n"
            "    node_id TEXT PRIMARY KEY,\n"
            "    parent  TEXT REFERENCES nodes (node_id)\n"
            ");\n"
        )
    }
    assert audit(selfref, {"t/a.rs": 'delete_tables(&p, &["nodes"]).await;'})["refused"] == []
    checks += 1

    print(f"check_fixture_plan_children --self-test: {checks} controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    result = audit(*read_tree())
    if "--json" in args:
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if not (result["refused"] or result["unreadable"] or result["alter_added"]) else 1
    return report(result)


if __name__ == "__main__":
    raise SystemExit(main())
