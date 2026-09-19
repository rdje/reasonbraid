#!/usr/bin/env python3
"""Census every SITE-GLOBAL table against WHO READS IT and whether its tenant is
recoverable at all.

`SIGNOFF-REPAIR.7.1.2`, adjudicating the population `SIGNOFF-REPAIR.7.1.1`
published. That leaf measured the WRITE side — 42 routes reaching a table with no
tenant dimension, 33 of them admitted on enrolment alone. It deliberately judged
none of them. This one supplies the two facts an adjudication needs and that a
reading cannot be trusted to supply twice:

    python3 -B scripts/census_registry_read_reach.py            # the census
    python3 -B scripts/census_registry_read_reach.py --check    # the gate
    python3 -B scripts/census_registry_read_reach.py --json
    python3 -B scripts/census_registry_read_reach.py --self-test

⭐ FACT 1 — THE TENANT MAY BE RECOVERABLE WITHOUT A COLUMN, and assuming
otherwise is how `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
(DOC-0029) came to read as a rule about every column-less table. It is not: its
argument is about CONTENT-ADDRESSED rows, which cannot carry an owner because
several tenants share one row by construction. A table whose primary key is a
FOREIGN KEY into a tenant-dimensioned table is a different animal entirely — one
join recovers the tenant. `agent_profiles.role_id REFERENCES agent_roles
(role_id)` is exactly that, and so are `profile_versions`, `quota_events` and
`recruitment_responses`. This census walks the REFERENCES graph transitively and
reports the path, so "site-global" is never again read as "ownerless".

⭐ FACT 2 — WHO READS IT DECIDES WHAT IT IS. A row nobody else's decision consults
is shared evidence, and DOC-0029's remedy (bind the READ) fits it. A row another
tenant's resolution, routing or policy decision consults is a shared CONTROL
surface, whose problem is the WRITE — and binding its read would not touch the
defect. The census cannot make that call, so it does not: it reports every
product-code reader with its enclosing function and whether the statement carries
a tenant predicate, and the adjudication happens in a decision record.

⛔ WHAT THIS DOES NOT DO. It does not classify. A verdict of *shared evidence* or
*shared control surface* is a judgement about what a decision MEANS, and a script
that guessed it would be `.11.14`'s defect one layer down — a population scoped by
a name rather than derived. The verdicts live in
`docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`
and the baseline pins them beside the mechanical facts, so a drift in either is
refused.

⚠️ THE READER SCAN IS LEXICAL, over SQL string literals in product code only.
A read assembled from fragments at runtime is not seen. No such site exists today
— `--self-test` asserts the corpus shape it depends on — and the limitation is
recorded here rather than left for a reader to discover.
"""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]

BASELINE = ROOT / ".doctrine" / "registry_read_reach_baseline.tsv"

# ⛔ THE LIVE DOCUMENTS THAT RESTATE THIS CENSUS'S FIGURE (`SIGNOFF-REPAIR.13.4`'s
# corpus rule, as a mechanism rather than a habit). Its sibling
# `census_shared_registry_writes.py` has carried this list since `.7.1.1.1`; this
# census did not, and that difference is not academic:
#
#   `.7.1.2` published `40 site-global tables`, which was true. `migrations/0072`
#   then bound both routing journals and took it to 38, and no leaf restated it.
#   `SIGNOFF-REPAIR.6.1.5.2` read `40` out of MEMORY.md, carried it forward, and
#   published `40 → 29` — a movement of eleven where only nine tables had left.
#   The write figure in the very same commit was correct, because THAT census
#   refuses with its corpus named and forces a re-derivation.
#
# ⚠️ The refusal below names these files so that a reader who refreshes the
# baseline cannot leave the prose behind. A number appearing in N places has N
# chances to be stale; prefer the one derived source.
RESTATING_DOCUMENTS = (
    "CHANGELOG.md",
    "LIVE_STATUS.md",
    "MEMORY.md",
    "docs/TASK_TREE.md",
    "docs/tasks/SIGNOFF-REPAIR.md",
)

# A Rust string literal, `\`-continuation included — sqlx statements in this
# repository are written as one literal broken over lines with a trailing `\`,
# so DOTALL is required and a line-anchored pattern silently sees a third of them.
LITERAL = re.compile(r'"(?:[^"\\]|\\.)*"', re.DOTALL)

FN = re.compile(r"\bfn\s+([a-z_][a-z_0-9]*)")

# The tenant predicate, as the binding census spells it: `tenant_id` is not the
# only spelling, and a read bound through a parent's column is still bound.
TENANT_PREDICATE = re.compile(r"\btenant(?:_id)?\b", re.IGNORECASE)


def _binding_module():
    """Import the sibling census, whose schema primitives this one reuses.

    ⭐ One answer to *does this table carry a tenant dimension*, not two that can
    drift — the same coupling, and for the same stated reason, as
    `census_shared_registry_writes.py`.
    """
    path = ROOT / "scripts" / "census_get_route_binding.py"
    spec = importlib.util.spec_from_file_location("census_get_route_binding", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def _writes_module():
    path = ROOT / "scripts" / "census_shared_registry_writes.py"
    spec = importlib.util.spec_from_file_location("census_shared_registry_writes", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def migration_text() -> str:
    return "\n".join(
        p.read_text(encoding="utf-8") for p in sorted((ROOT / "migrations").glob("*.sql"))
    )


def reference_graph() -> dict[str, list[tuple[str, str]]]:
    """table → the `(column, parent_table)` edges its schema declares.

    Both spellings are read: the inline `col … REFERENCES parent (col)` and the
    later `ALTER TABLE child ADD CONSTRAINT … FOREIGN KEY (col) REFERENCES parent`.
    """
    binding = _binding_module()
    text = migration_text()
    graph: dict[str, list[tuple[str, str]]] = {}

    for m in re.finditer(
        r"CREATE TABLE(?:\s+IF NOT EXISTS)?\s+(?:[a-z_][a-z_0-9]*\.)?([a-z_][a-z_0-9]*)\s*\(",
        text,
        re.IGNORECASE,
    ):
        child = m.group(1)
        body = text[m.end() : binding.balanced(text, m.end() - 1)]
        edges = graph.setdefault(child, [])
        for line in body.splitlines():
            ref = re.search(
                r"^\s*([a-z_][a-z_0-9]*)\b.*?\bREFERENCES\s+(?:[a-z_][a-z_0-9]*\.)?([a-z_][a-z_0-9]*)",
                line,
                re.IGNORECASE,
            )
            if ref and (ref.group(1), ref.group(2)) not in edges:
                edges.append((ref.group(1), ref.group(2)))

    for m in re.finditer(
        r"ALTER TABLE\s+(?:[a-z_][a-z_0-9]*\.)?([a-z_][a-z_0-9]*)\b[^;]*?"
        r"FOREIGN KEY\s*\(\s*([a-z_][a-z_0-9]*)\s*\)\s*REFERENCES\s+"
        r"(?:[a-z_][a-z_0-9]*\.)?([a-z_][a-z_0-9]*)",
        text,
        re.IGNORECASE | re.DOTALL,
    ):
        edges = graph.setdefault(m.group(1), [])
        if (m.group(2), m.group(3)) not in edges:
            edges.append((m.group(2), m.group(3)))

    return graph


def derivation(table: str, graph: dict[str, list[tuple[str, str]]],
               dimensioned: dict[str, bool]) -> list[str] | None:
    """The shortest REFERENCES path from `table` to a tenant-dimensioned table.

    ⛔ Breadth-first, and a self-edge or a cycle terminates rather than recurses —
    `agent_roles` references `tenants`, which references nothing, but a schema
    that grew a cycle would otherwise hang the gate.
    """
    if dimensioned.get(table):
        return [table]
    seen = {table}
    queue: list[list[str]] = [[table]]
    while queue:
        path = queue.pop(0)
        for _column, parent in graph.get(path[-1], []):
            if parent in seen:
                continue
            seen.add(parent)
            extended = path + [parent]
            if dimensioned.get(parent):
                return extended
            queue.append(extended)
    return None


def product_sources() -> list[Path]:
    """Every product `.rs` file — the crates' `src/`, never their `tests/`.

    ⚠️ A test reads these tables constantly and none of those reads is a
    disclosure surface. Counting them would put `quota_events` at twelve readers
    when the product has one.
    """
    binding = _binding_module()
    # ⛔ `crates/*/src/**/*.rs` as a git pathspec matches only the NESTED modules —
    # 29 of 120 files here, silently dropping every top-level `src/*.rs`, which is
    # where almost all of these statements live. Measured, not assumed:
    # `git ls-files 'crates/*/src/**/*.rs' | wc -l` → 29 against 120. The whole
    # tree is listed and filtered in Python instead.
    return [
        ROOT / p
        for p in binding.tracked("crates")
        if p.endswith(".rs") and "/src/" in p and "/tests/" not in p
    ]


def readers(tables: set[str]) -> dict[str, list[dict[str, object]]]:
    """table → every product-code statement that reads it.

    A reader is a SQL literal naming the table after `FROM` or `JOIN`. The
    enclosing function is the nearest preceding `fn`, and `tenant_predicate`
    reports whether the SAME statement mentions a tenant — the question an
    adjudication asks first.
    """
    binding = _binding_module()
    found: dict[str, list[dict[str, object]]] = {t: [] for t in tables}
    pattern = {t: re.compile(rf"\b(?:FROM|JOIN)\s+{t}\b", re.IGNORECASE) for t in tables}

    for path in product_sources():
        text = path.read_text(encoding="utf-8")
        spans = binding.outside_tests(text)
        starts = [(m.start(), m.group(1)) for m in FN.finditer(text)]
        rel = str(path.relative_to(ROOT))
        for literal in LITERAL.finditer(text):
            lo = literal.start()
            if not any(a <= lo < b for a, b in spans):
                continue
            body = literal.group(0)
            for table in tables:
                if not pattern[table].search(body):
                    continue
                enclosing = None
                for pos, name in starts:
                    if pos < lo:
                        enclosing = name
                    else:
                        break
                found[table].append(
                    {
                        "file": rel,
                        "line": text.count("\n", 0, lo) + 1,
                        "fn": enclosing,
                        "tenant_predicate": bool(TENANT_PREDICATE.search(body)),
                    }
                )
    for rows in found.values():
        rows.sort(key=lambda r: (r["file"], r["line"]))
    return found


def collect() -> list[dict[str, object]]:
    """One row per site-global table reached by a mutating route."""
    binding = _binding_module()
    writes = _writes_module()
    dimensioned = binding.tenant_dimensioned_tables()
    graph = reference_graph()

    tables: dict[str, set[str]] = {}
    for row in writes.collect():
        for table in row["site_global"]:
            tables.setdefault(table, set()).add(str(row["admission"]))

    read_map = readers(set(tables))
    rows: list[dict[str, object]] = []
    for table in sorted(tables):
        path = derivation(table, graph, dimensioned)
        rows.append(
            {
                "table": table,
                "admissions": sorted(tables[table]),
                "tenant_derivation": path,
                "readers": read_map[table],
            }
        )
    return rows


def report(rows: list[dict[str, object]]) -> None:
    derivable = [r for r in rows if r["tenant_derivation"]]
    ownerless = [r for r in rows if not r["tenant_derivation"]]

    print(f"site-global tables reached by a mutating route: {len(rows)}")
    print(
        f"  tenant RECOVERABLE by a foreign key: {len(derivable)}"
        f" · no derivation at all: {len(ownerless)}"
    )
    print()
    print("== THE TENANT IS RECOVERABLE — a column-less table is not an ownerless one ==")
    print()
    for row in derivable:
        print(f"-- {row['table']}")
        print(f"     derivation: {' → '.join(row['tenant_derivation'])}")
        _print_readers(row)
    print()
    print("== NO DERIVATION — nothing in the schema names an owner ==")
    print()
    for row in ownerless:
        print(f"-- {row['table']}   (admitted by: {', '.join(row['admissions'])})")
        _print_readers(row)


def _print_readers(row: dict[str, object]) -> None:
    reads = row["readers"]
    assert isinstance(reads, list)
    if not reads:
        print("     readers: none in product code — write-only")
        return
    for read in reads:
        bound = "tenant-predicated" if read["tenant_predicate"] else "NO tenant predicate"
        print(f"     read by {read['fn']}()  {read['file']}:{read['line']}  [{bound}]")


def baseline_rows(rows: list[dict[str, object]]) -> list[str]:
    """The pinned SET, as `table<TAB>derivation<TAB>readers`.

    ⭐ THE SET, NOT THE COUNT — `.7.1.1.1`'s rule, applied to the read side. A
    reader added and a reader removed holds every number while the surface moves.
    """
    out = []
    for row in rows:
        path = row["tenant_derivation"]
        assert isinstance(path, list) or path is None
        derivation = "→".join(path) if path else "-"
        reads = row["readers"]
        assert isinstance(reads, list)
        names = ",".join(
            f"{r['fn']}{'' if r['tenant_predicate'] else '!'}" for r in reads
        ) or "-"
        out.append(f"{row['table']}\t{derivation}\t{names}")
    return out


def load_baseline(path: Path | None = None) -> list[str] | None:
    target = path or BASELINE
    if not target.exists():
        return None
    return [
        line
        for line in target.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]


def check(rows: list[dict[str, object]], path: Path | None = None) -> int:
    current = baseline_rows(rows)
    pinned = load_baseline(path)
    if pinned is None:
        print(f"registry-read-reach: no baseline at {(path or BASELINE).relative_to(ROOT)}")
        return 1
    added = [r for r in current if r not in pinned]
    removed = [r for r in pinned if r not in current]
    if not added and not removed:
        print(f"registry-read-reach: OK — {len(current)} site-global tables, unchanged")
        return 0
    print("registry-read-reach: DRIFT — the adjudicated read surface moved")
    for row in removed:
        print(f"  - {row}")
    for row in added:
        print(f"  + {row}")
    print()
    print("⛔ Re-adjudicate before refreshing. The verdicts are in")
    print("   docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md;")
    print("   a new reader on a shared CONTROL surface is a design change, not a refresh.")
    print()
    print(f"⛔ This census's count is {len(current)} today and {len(pinned)} in the")
    print("   baseline. It is RESTATED in these live documents, and a correction is")
    print("   not complete until they have been censused (`SIGNOFF-REPAIR.13.4`):")
    for document in RESTATING_DOCUMENTS:
        print(f"      {document}")
    print()
    print("   ⚠️ Carrying the old figure forward is how `40 → 29` was published for a")
    print("   movement in which nine tables left a population of 38. Subtract, and if")
    print("   the arithmetic does not close, the starting figure is the stale one.")
    return 1


def self_test() -> int:
    """Falsify the three assumptions this census would be worthless without."""
    failures = []

    # 0. ⛔ THE RESTATING CORPUS IS NAMED AND TRACKED. Without this list the
    #    refusal cannot tell a reader where the number is repeated, which is how
    #    `40 → 29` was published for a movement of nine out of 38
    #    (`SIGNOFF-REPAIR.6.1.5.3.1`). An untracked path here is a refusal that
    #    names a file nobody can open.
    missing_documents = [d for d in RESTATING_DOCUMENTS if not (ROOT / d).exists()]
    if missing_documents or not RESTATING_DOCUMENTS:
        failures.append(
            f"the restating corpus is empty or untracked: {missing_documents or 'empty'}"
        )

    binding = _binding_module()
    dimensioned = binding.tenant_dimensioned_tables()
    graph = reference_graph()

    # 1. The REFERENCES walk finds the two-hop path the adjudication turns on.
    #    `quota_events → usage_quotas → tenants` is three names, and a walk that
    #    stopped at one hop would call the table ownerless.
    path = derivation("quota_events", graph, dimensioned)
    if path != ["quota_events", "usage_quotas"]:
        failures.append(f"quota_events derivation is {path}, expected via usage_quotas")

    # 2. A genuinely ownerless table stays ownerless — the NEGATIVE arm, without
    #    which a walk that returned a path for everything would pass test 1.
    if derivation("workflow_profiles", graph, dimensioned) is not None:
        failures.append("workflow_profiles reports a tenant derivation it does not have")

    # 3. The literal scan sees a `\`-continued statement. Every sqlx statement in
    #    this repository is written that way, so a line-anchored pattern would
    #    report zero readers and the census would be silently empty.
    probe = readers({"workflow_profiles"})["workflow_profiles"]
    if not any(r["fn"] == "resolve" for r in probe):
        failures.append("the literal scan missed workflows::resolve — the continuation bug")

    # 4. Test code is excluded: `quota_events` is read once in product code and
    #    nine times in tests.
    quota = readers({"quota_events"})["quota_events"]
    if len(quota) != 1:
        failures.append(f"quota_events has {len(quota)} product readers, expected 1")

    for failure in failures:
        print(f"  FAIL {failure}")
    print(f"self-test: {'FAILED' if failures else 'ok'} ({len(failures)} failure(s))")
    return 1 if failures else 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    rows = collect()
    if "--json" in argv:
        print(json.dumps(rows, indent=2, sort_keys=True))
        return 0
    if "--check" in argv:
        return check(rows)
    if "--baseline" in argv:
        print("\n".join(baseline_rows(rows)))
        return 0
    report(rows)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
