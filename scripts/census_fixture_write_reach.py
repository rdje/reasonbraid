#!/usr/bin/env python3
"""scripts/census_fixture_write_reach.py — FIXTURE-WRITE-REACH doctrine.

A PostgreSQL suite declares a cleanup plan: the tables it deletes, in order,
before its first test. `scripts/check_fixture_plan_children.py` already refuses a
plan that deletes a parent without its foreign-key children. This gate answers
the OTHER question, which no foreign key can express:

    does the plan name every table the suite's own HTTP calls write?

⛔ IT IS A POLLUTION GATE, NOT A CORRECTNESS ONE, AND THE DIFFERENCE MATTERS.
A row a suite leaves behind never hurts that suite; it hurts whichever LATER
suite in the same cluster asserts over the same table. So the failure is
order-dependent, lands somewhere else, and looks like a flake.

That is exactly how it was found (`SIGNOFF-REPAIR.7.1.2.2.2`):

    RB_DEMO=0 bash scripts/run_pg_tests.sh routing evaluation   # evaluation 2/1
    RB_DEMO=0 bash scripts/run_pg_tests.sh evaluation routing   # both pass

`tests/routing.rs` has posted to `/v1/evaluations/trials` since REPAIR-0274 — its
shadow-recommendation fixtures cite a trial as their `evidence_ref` — and its
plan never named `evaluation_trials`. `tests/evaluation.rs` then asserts the
CONTENT of that table and never empties it either. Both plans were wrong, for two
different reasons, and nothing could say so.

⚠️ THE QUESTION THIS GATE ANSWERS IS A PROXY, AND THE LEAF SAYS SO RATHER THAN
IMPLYING COVERAGE IT DOES NOT HAVE. The defect above is *an assertion over a
table the suite never emptied*, and which tables an assertion quantifies over is
not mechanically derivable — it would need to know what `assert_eq!` over a
`Vec<Value>` from `GET /v1/…` is really quantifying over. What IS derivable is
which tables a suite's own calls WRITE, and that is a superset of the tables it
can pollute for others. Purge-what-you-write is the mechanizable half; it covers
the observed defect from the `routing.rs` side.

⚠️ AND IT IS AN OVER-APPROXIMATION IN ONE MORE WAY, STATED HERE RATHER THAN
DISCOVERED: it matches PATH literals, not HTTP verbs, so a suite that only reads
`/v1/evaluations/trials` is charged with the reach of the POST sharing that path.
Purging a table the suite merely read costs one DELETE; missing one costs another
suite a flake. A self-test arm pins that choice so narrowing it stays a decision.

⛔ AND THE SHARPEST BOUND OF ALL, MEASURED RATHER THAN REASONED. This gate
catches the `evaluation_trials` defect only because the ASSERTING suite also
POSTs to `/v1/evaluations/trials`. Falsified both ways round: removing
`evaluation.rs`'s purge brings the original `2/1` failure straight back, while
removing `routing.rs`'s leaves every suite GREEN — the assert-side purge is what
fixes the bug, and the write-side purge is hygiene that stops the next one. A
suite that asserts over a table it NEVER writes would still be invisible here.
That case has no instance in this tree today; when one appears it needs a
different instrument, not a wider regex.

The route-to-table reach is not re-derived here. It is taken from
`census_shared_registry_writes.py`, which already walks every handler, so the two
instruments cannot disagree about what a route touches.

    python3 -B scripts/census_fixture_write_reach.py            # the gate
    python3 -B scripts/census_fixture_write_reach.py --json     # the census
    python3 -B scripts/census_fixture_write_reach.py --self-test

⛔ Numbers move with the tree; `--json` re-derives them. At the commit that added
this gate: 29 declared plans, 5 refused, 2 tables exempt.
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# ⛔ REFERENCE DATA, NOT FIXTURE DATA — a migration creates these rows and the
# product reads them, so a purge would delete what the next test needs rather
# than what the last test left. Each entry carries its reason and its evidence,
# and an exemption without one is not an exemption.
#
# ⚠️ "A migration INSERTs into it" is NOT the rule and must not be made the rule:
# `usage_quotas` is seeded by `migrations/0068` and is purged safely by eight
# plans, because the application re-creates a tenant's quota rows on enrolment.
# The rule is whether the PRODUCT depends on rows no test creates.
EXEMPT = {
    "workflow_profiles": (
        "migrations/0032 seeds the eight built-in profiles and thread creation "
        "resolves `quick_advice` through them; purging it breaks every later "
        "bare thread. Recorded as deliberate at SIGNOFF-REPAIR.7.1.2.1."
    ),
    "resolver_capabilities": (
        "migrations/0025-0027 seed the R0, R1 and R2 resolver entries; the "
        "acquisition path resolves against them and no test creates them."
    ),
}

# The helper that IMPLEMENTS plan checking and the suite that proves it refuses
# bad plans: neither declares a real fixture plan.
EXCLUDED = {
    "crates/reasonbraid-server/tests/support/cleanup.rs",
    "crates/reasonbraid-server/tests/support/cleanup_tests.rs",
}

PLAN = re.compile(r"delete_tables\(\s*&?\w+\s*,\s*&\[(.*?)\]\s*,?\s*\)", re.S)
QUOTED = re.compile(r'"([a-z0-9_]+)"')
CALL_SITE = re.compile(r"delete_tables\(")


def _load(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def route_pattern(route: str) -> re.Pattern[str] | None:
    """The literal a test writes for this route, as a regex.

    `POST /v1/policy-publications/{publication_id}/effective` becomes a pattern
    matching `/v1/policy-publications/<anything>/effective`, because a fixture
    interpolates the id. ⛔ GET routes are excluded: this census is about writes.
    """
    verb, path = route.split(" ", 1)
    if verb == "GET":
        return None
    return re.compile(
        "".join(
            "[^\"/ ]+" if seg.startswith("{") else re.escape(seg)
            for seg in re.split(r"(\{[^}]*\})", path)
            if seg
        )
    )


def write_reach(rows: list[dict]) -> dict[str, set[str]]:
    """route -> every table the walk says it writes, of any tenancy."""
    reach = {}
    for row in rows:
        tables = (
            set(row["site_global"]) | set(row["tenant_scoped"]) | set(row["unknown"])
        )
        reach[str(row["route"])] = tables
    return reach


def audit(reach: dict[str, set[str]], sources: dict[str, str]) -> dict:
    patterns = {r: p for r in reach if (p := route_pattern(r)) is not None}
    plans, unreadable, refused = 0, [], []
    for path, text in sorted(sources.items()):
        if path in EXCLUDED:
            continue
        sites = len(CALL_SITE.findall(text))
        parsed = PLAN.findall(text)
        if len(parsed) != sites:
            unreadable.append(f"{path}: {sites} call site(s), {len(parsed)} readable")
            continue
        if not parsed:
            continue
        plans += 1
        planned: set[str] = set()
        for body in parsed:
            planned |= set(QUOTED.findall(body))
        written: set[str] = set()
        for route, pattern in patterns.items():
            if pattern.search(text):
                written |= reach[route]
        missing = sorted(t for t in written - planned if t not in EXEMPT)
        if missing:
            refused.append({"plan": path, "missing": missing})
    return {
        "plans": plans,
        "exempt": sorted(EXEMPT),
        "unreadable": unreadable,
        "refused": refused,
    }


def read_tree() -> tuple[dict[str, set[str]], dict[str, str]]:
    import subprocess

    census = _load("shared_registry_writes", "scripts/census_shared_registry_writes.py")
    reach = write_reach(census.collect())
    tracked = subprocess.run(
        ["git", "ls-files", "*.rs"],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    ).stdout.split()
    sources = {}
    for path in tracked:
        text = (ROOT / path).read_text(encoding="utf-8", errors="replace")
        if "delete_tables(" in text:
            sources[path] = text
    return reach, sources


def report(result: dict) -> int:
    if result["unreadable"]:
        print(
            "FIXTURE-WRITE-REACH: a `delete_tables(` call site this gate cannot read:\n  "
            + "\n  ".join(result["unreadable"])
            + "\n\n  A site it skips is a plan it does not check. Teach `PLAN` the shape,\n"
            "  or add the file to EXCLUDED with the reason.",
            file=sys.stderr,
        )
        return 1
    if result["refused"]:
        lines = [
            f"  {r['plan']} omits {', '.join(r['missing'])}" for r in result["refused"]
        ]
        print(
            "FIXTURE-WRITE-REACH: a suite writes a table its cleanup plan never "
            "deletes:\n" + "\n".join(lines) + "\n\n"
            "  Those rows outlive the suite. They never hurt it — they hurt whichever\n"
            "  LATER suite in the same cluster asserts over the same table, which is\n"
            "  why the failure is order-dependent and lands somewhere else.\n\n"
            "  ⭐ `routing.rs` wrote `evaluation_trials` and never purged it, so\n"
            "  `routing evaluation` failed `evaluation` 2/1 while the reverse order\n"
            "  passed both (SIGNOFF-REPAIR.7.1.2.2.2).\n\n"
            "  Add each table to the plan, ahead of any parent it references. If it is\n"
            "  reference data a migration seeds and the product reads, add it to EXEMPT\n"
            "  WITH ITS REASON — an exemption without one is not an exemption.",
            file=sys.stderr,
        )
        return 1
    return 0


def self_test() -> int:
    checks = 0
    reach = {
        "POST /v1/evaluations/trials": {"evaluation_trials"},
        "POST /v1/threads": {"aggregate_state", "event_log"},
        "GET /v1/evaluations/trials": {"evaluation_trials"},
        "POST /v1/workflow-profiles": {"workflow_profiles"},
    }

    # 1. a plan naming everything its calls write passes.
    complete = {
        "t/a.rs": 'post("/v1/evaluations/trials"); '
        'delete_tables(&p, &["evaluation_trials"]).await;'
    }
    assert audit(reach, complete)["refused"] == [], audit(reach, complete)
    checks += 1

    # 2. ⭐ THE ARM THAT WOULD HAVE CAUGHT `routing.rs`: the suite posts to a
    #    route that writes a table its plan never names.
    leaky = {
        "t/a.rs": 'post("/v1/evaluations/trials"); '
        'delete_tables(&p, &["aggregate_state"]).await;'
    }
    assert audit(reach, leaky)["refused"] == [
        {"plan": "t/a.rs", "missing": ["evaluation_trials"]}
    ], audit(reach, leaky)
    checks += 1

    # 3. ⚠️ THE INSTRUMENT'S BOUND, PINNED RATHER THAN GLOSSED. It matches PATH
    #    literals, not HTTP verbs, so a suite that only GETs `/v1/evaluations/
    #    trials` is still charged with the reach of the POST that shares that
    #    path. That is a deliberate OVER-approximation: purging a table the
    #    suite only read costs one DELETE, while missing one costs another
    #    suite a flake. ⛔ This arm asserts the over-approximation so that
    #    narrowing it later is a visible decision rather than a silent drift.
    reader = {
        "t/a.rs": 'get("/v1/evaluations/trials"); '
        'delete_tables(&p, &["aggregate_state"]).await;'
    }
    assert audit(reach, reader)["refused"] == [
        {"plan": "t/a.rs", "missing": ["evaluation_trials"]}
    ], audit(reach, reader)
    #    A path NO route in the corpus serves charges nothing, which is what
    #    keeps the over-approximation bounded by the route table.
    unrelated = {
        "t/a.rs": 'get("/v1/nothing-here"); delete_tables(&p, &["aggregate_state"]).await;'
    }
    assert audit(reach, unrelated)["refused"] == [], audit(reach, unrelated)
    checks += 2

    # 4. an interpolated path parameter still matches its route template.
    templated = {
        "POST /v1/policy-publications/{publication_id}/effective": {"policy_publications"}
    }
    interpolated = {
        "t/a.rs": 'post(&format!("/v1/policy-publications/{id}/effective")); '
        'delete_tables(&p, &["aggregate_state"]).await;'
    }
    assert audit(templated, interpolated)["refused"] == [
        {"plan": "t/a.rs", "missing": ["policy_publications"]}
    ], audit(templated, interpolated)
    checks += 1

    # 5. an EXEMPT table is not demanded — reference data a migration seeds.
    seeded = {
        "t/a.rs": 'post("/v1/workflow-profiles"); '
        'delete_tables(&p, &["aggregate_state"]).await;'
    }
    assert audit(reach, seeded)["refused"] == [], audit(reach, seeded)
    checks += 1

    # 6. ⛔ every exemption carries a reason, or it is a silent hole.
    assert all(isinstance(v, str) and len(v) > 40 for v in EXEMPT.values()), EXEMPT
    checks += 1

    # 7. a file with no plan is not a plan, and an unreadable site is an error.
    assert audit(reach, {"t/a.rs": "no plan here"})["plans"] == 0
    odd = {"t/a.rs": "delete_tables(&pool, PLAN_CONST).await;"}
    assert audit(reach, odd)["unreadable"], "an unreadable site must be named"
    checks += 2

    # 8. the two excluded files are excluded by path, not by luck.
    excluded = {p: "delete_tables(&pool, PLAN_CONST).await;" for p in EXCLUDED}
    assert audit(reach, excluded)["unreadable"] == [], EXCLUDED
    checks += 1

    # 9. the live corpus parses and the reach comes from the other census.
    try:
        live_reach, live_sources = read_tree()
        live_ok = len(live_reach) > 20 and len(live_sources) > 10
    except Exception as exc:  # noqa: BLE001
        live_ok = False
        print(f"census_fixture_write_reach: live-corpus arm raised {exc!r}")
    assert live_ok, "the live corpus must parse"
    checks += 1

    print(f"census_fixture_write_reach --self-test: {checks} controls pass")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    result = audit(*read_tree())
    if "--json" in argv:
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if not (result["refused"] or result["unreadable"]) else 1
    return report(result)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
