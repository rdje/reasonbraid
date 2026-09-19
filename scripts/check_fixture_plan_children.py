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
declares is readable from the migrations.

⛔ THAT SENTENCE ONCE READ "every foreign key this corpus declares is INLINE in a
`CREATE TABLE`", AND `migrations/0072` FALSIFIED IT (`SIGNOFF-REPAIR.7.1.2.2.1`).
It wrote `ALTER TABLE routing_resolutions ADD COLUMN tenant_id TEXT REFERENCES
tenants (tenant_id)` — a column-level reference on an ALTER, matched neither by
the `CREATE TABLE` walk nor by the `ADD CONSTRAINT … FOREIGN KEY` refusal. The
edge was invisible to BOTH, so this gate reported `0 refused` while twenty-two
plans named `tenants` without its two new children and twenty suites could not
start. `ADD COLUMN … REFERENCES` is now modelled; `ADD CONSTRAINT` still refuses.
⭐ The claim had been calibrated on the corpus that existed, with no arm that
fails when the corpus grows a shape —
`docs/knowledge/calibrate-over-the-history-that-contains-the-instance.md`.

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
plans, 0 refused. At `SIGNOFF-REPAIR.7.1.2.2.1`, which taught it the ALTER
column: 45 inline, **3 added-column**, 0 via ADD CONSTRAINT, 16 parents, 29
plans, 0 refused after a 22-plan sweep. ⚠️ One of those three edges dates from
`migrations/0060` — `node_enrollment_tokens.issued_under` →
`authorization_records` — so the blind spot predated the migration that exposed
it by twelve, and every plan reaching that parent happened to order it correctly.
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
# ⛔ THE PRODUCTION THAT DEFEATED THIS GATE (`SIGNOFF-REPAIR.7.1.2.2.1`).
# `migrations/0072` writes `ALTER TABLE routing_resolutions ADD COLUMN tenant_id
# TEXT REFERENCES tenants (tenant_id)` — a COLUMN-level reference on an ALTER,
# which `CREATE TABLE` parsing cannot see and `ALTER_FK` does not match either,
# because that pattern wants `ADD CONSTRAINT … FOREIGN KEY`. So the edge was
# neither modelled nor refused, and twenty-one suites could not start while this
# gate reported `0 refused`.
ALTER_STATEMENT = re.compile(r"ALTER TABLE\s+(?:ONLY\s+)?(\w+)([^;]*);", re.S | re.I)
ADD_COLUMN_REF = re.compile(
    r"ADD COLUMN\s+\w+[^,;]*?REFERENCES\s+(\w+)", re.S | re.I
)
CALL_SITE = re.compile(r"delete_tables\(")
PLAN = re.compile(r"delete_tables\(\s*&?\w+\s*,\s*&\[(.*?)\]\s*,?\s*\)", re.S)
QUOTED = re.compile(r'"([a-z_]+)"')


def strip_sql_comments(text: str) -> str:
    """`-- …` to end of line. A `REFERENCES` inside a comment is prose."""
    return "\n".join(line.split("--", 1)[0] for line in text.splitlines())


def fk_graph(migrations: dict[str, str]) -> tuple[dict[str, set[str]], int, int]:
    """parent -> {children}, plus the inline and the added-column edge counts.

    ⭐ The two counts stay APART on purpose. They are the gate's own evidence
    that it can see both shapes, and a single total would hide the added-column
    edges going back to zero the way they silently were.
    """
    children: dict[str, set[str]] = {}
    inline = 0
    added = 0
    for _, raw in sorted(migrations.items()):
        body = strip_sql_comments(raw)
        for match in CREATE_TABLE.finditer(body):
            child, columns = match.group(1), match.group(2)
            for ref in REFERENCES.finditer(columns):
                parent = ref.group(1)
                if parent == child:
                    continue  # a self-reference orders nothing between tables
                children.setdefault(parent, set()).add(child)
                inline += 1
        for statement in ALTER_STATEMENT.finditer(body):
            child, rest = statement.group(1), statement.group(2)
            for ref in ADD_COLUMN_REF.finditer(rest):
                parent = ref.group(1)
                if parent == child:
                    continue  # a self-reference orders nothing between tables
                children.setdefault(parent, set()).add(child)
                added += 1
    return children, inline, added


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
    children, inline_edges, added_column_edges = fk_graph(migrations)
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
        "inline_edges": inline_edges,
        "added_column_edges": added_column_edges,
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
    assert ok["added_column_edges"] == 0, ok
    checks += 3
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

    # 5. ⭐ THE ARM THAT WOULD HAVE CAUGHT `migrations/0072`, and it fails
    #    against the parser this gate shipped with: a column-level `REFERENCES`
    #    on an `ALTER TABLE` is a real edge, not an unmodelled one. It must be
    #    MODELLED (so the incomplete plan is refused) and must NOT appear in
    #    `alter_added` (which is the refusal for `ADD CONSTRAINT … FOREIGN KEY`,
    #    a shape still outside the model).
    added_column = dict(good_migrations)
    added_column["migrations/0002_z.sql"] = (
        "ALTER TABLE notes ADD COLUMN owner TEXT REFERENCES tenants (tenant_id);\n"
    )
    grown = audit(added_column, incomplete)
    assert grown["alter_added"] == [], grown
    assert grown["added_column_edges"] == 1, grown
    assert grown["refused"] == [{"plan": "t/a.rs", "missing": ["notes"]}], grown
    checks += 3

    # 6. the same edge into a parent the plan does NOT already reach — the exact
    #    `0072` shape, where `routing_recommendations` gained its FIRST key to
    #    `tenants` and no plan had ever needed to name it.
    fresh = {
        "migrations/0001_x.sql": "CREATE TABLE tenants (\n    tenant_id TEXT PRIMARY KEY\n);\n"
        "CREATE TABLE journal (\n    row_id TEXT PRIMARY KEY\n);\n",
        "migrations/0002_y.sql": "ALTER TABLE journal ADD COLUMN tenant_id TEXT "
        "REFERENCES tenants (tenant_id);\n",
    }
    late = audit(fresh, {"t/a.rs": 'delete_tables(&p, &["tenants"]).await;'})
    assert late["refused"] == [{"plan": "t/a.rs", "missing": ["journal"]}], late
    assert audit(fresh, {"t/a.rs": 'delete_tables(&p, &["journal", "tenants"]).await;'})[
        "refused"
    ] == [], "the swept plan passes"
    checks += 2

    # 7. an unreadable call site is an error, not a skip.
    odd = {"t/a.rs": "delete_tables(&pool, PLAN_CONST).await;"}
    assert audit(good_migrations, odd)["unreadable"], "an unreadable site must be named"
    checks += 1

    # 8. the two excluded files are excluded by path, not by luck.
    excluded = {p: 'delete_tables(&pool, PLAN_CONST).await;' for p in EXCLUDED}
    assert audit(good_migrations, excluded)["unreadable"] == [], EXCLUDED
    checks += 1

    # 9. a self-referencing key orders nothing and must not be demanded.
    selfref = {
        "migrations/0001_x.sql": (
            "CREATE TABLE nodes (\n"
            "    node_id TEXT PRIMARY KEY,\n"
            "    parent  TEXT REFERENCES nodes (node_id)\n"
            ");\n"
        )
    }
    assert audit(selfref, {"t/a.rs": 'delete_tables(&p, &["nodes"]).await;'})["refused"] == []
    selfref_altered = {
        "migrations/0001_x.sql": "CREATE TABLE nodes (\n    node_id TEXT PRIMARY KEY\n);\n",
        "migrations/0002_y.sql": "ALTER TABLE nodes ADD COLUMN parent TEXT "
        "REFERENCES nodes (node_id);\n",
    }
    late_selfref = audit(selfref_altered, {"t/a.rs": 'delete_tables(&p, &["nodes"]).await;'})
    assert late_selfref["refused"] == [], late_selfref
    assert late_selfref["added_column_edges"] == 0, late_selfref
    checks += 2

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
