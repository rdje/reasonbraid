#!/usr/bin/env python3
"""Census ROADMAP §18.5's nine operator surfaces against the routes that expose them.

§18.5 says the admin UI/CLI *must expose* nine things. `SIGNOFF-REPAIR.4.6`
graded them by hand (4 covered, 3 partial, 2 absent), and `.4.6.1` built the
missing ones. A hand-derived verdict decays silently: a route renamed or removed
leaves the grade standing. This instrument keeps the verdict attached to the
facts (`SIGNOFF-REPAIR.4.6.1.6`).

    python3 -B scripts/census_operator_surfaces.py            # the census
    python3 -B scripts/census_operator_surfaces.py --check    # the gate: exit 1 if a mapped route is gone
    python3 -B scripts/census_operator_surfaces.py --json
    python3 -B scripts/census_operator_surfaces.py --self-test

⭐ THE MAPPING IS A JUDGEMENT; THE ROUTES ARE DERIVED. Which route answers which
bullet is written once, in `.doctrine/operator_surfaces.tsv`, with the leaf that
made it true. Whether each
route EXISTS is read from the server's routers every run, through
`census_route_documentation.product_routes` — imported, not copied, so there is
one parser for "a route this server registers", and a route inside a
`#[cfg(test)]` block does not count.

⛔ A BLOCKED PART IS NAMED, NEVER COUNTED AS EXPOSED. Bullet 8's *checkpoint
age* has no checkpoint to measure until ADR-022's chain is built; the census
reports the bullet as `partial` with that blocker, and the day a checkpoint
route appears it must be added here rather than assumed.

⚠️ `exposed` means the routes exist. It is not a claim that each route returns
every field §18.5 names; that is what the owning leaves' live controls assert.
"""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVER_SRC = "crates/reasonbraid-server/src"

MAPPING = ROOT / ".doctrine/operator_surfaces.tsv"


def load_surfaces(path: Path | None = None) -> list[dict]:
    """§18.5, bullet by bullet: the routes that expose it, the leaf that made
    the mapping true, and — for a part not yet buildable — what blocks it.

    ⭐ Read from `.doctrine/operator_surfaces.tsv`, which
    `crates/reasonbraid-server/tests/operator_surfaces.rs` reads too
    (`SIGNOFF-REPAIR.4.6.1.7`): this census proves each route is REGISTERED,
    that test proves the running app SERVES it, and neither keeps a copy.
    """
    rows = []
    for line in (path or MAPPING).read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        bullet, owner, routes, blocked, surface = line.split("\t")
        row = {"bullet": int(bullet), "surface": surface, "routes": routes.split(),
               "owner": owner}
        if blocked != "-":
            row["blocked"] = blocked
        rows.append(row)
    return rows


SURFACES: list[dict] = load_surfaces()


def _route_census():
    path = ROOT / "scripts/census_route_documentation.py"
    spec = importlib.util.spec_from_file_location("census_route_documentation", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def registered_routes() -> set[str]:
    """Every route a tracked server source registers outside `#[cfg(test)]`."""
    census = _route_census()
    files = subprocess.run(["git", "ls-files", "--", f"{SERVER_SRC}/*.rs"], cwd=ROOT,
                           capture_output=True, text=True, check=True).stdout.split()
    routes: set[str] = set()
    for name in files:
        routes.update(census.product_routes((ROOT / name).read_text(encoding="utf-8")))
    return routes


def adjudicate(routes: set[str], surfaces: list[dict] | None = None) -> list[dict]:
    rows = []
    for surface in surfaces if surfaces is not None else SURFACES:
        missing = [r for r in surface["routes"] if r not in routes]
        verdict = "missing" if missing else ("partial" if surface.get("blocked") else "exposed")
        rows.append({**surface, "verdict": verdict, "missing_routes": missing})
    return rows


def summary(rows: list[dict]) -> str:
    counts = {v: sum(r["verdict"] == v for r in rows) for v in ("exposed", "partial", "missing")}
    return (f"OPERATOR-SURFACES: {len(rows)} §18.5 surfaces — {counts['exposed']} exposed · "
            f"{counts['partial']} partial (blocked part named) · {counts['missing']} missing")


def report(rows: list[dict]) -> None:
    print(summary(rows))
    for row in rows:
        print(f"  {row['bullet']}. [{row['verdict']:<7}] {row['surface']}  ({row['owner']})")
        for route in row["routes"]:
            mark = "MISSING " if route in row["missing_routes"] else ""
            print(f"       {mark}{route}")
        if row.get("blocked"):
            print(f"       ⏸ blocked: {row['blocked']}")


def check(rows: list[dict]) -> int:
    missing = [r for r in rows if r["verdict"] == "missing"]
    if not missing:
        print(summary(rows))
        return 0
    print("OPERATOR-SURFACES: a route §18.5's census maps is no longer registered:", file=sys.stderr)
    for row in missing:
        print(f"  bullet {row['bullet']} ({row['surface']}): {', '.join(row['missing_routes'])}"
              f" — owner {row['owner']}", file=sys.stderr)
    print("""
  The surface was exposed by that route. Either it was renamed — update the
  mapping in scripts/census_operator_surfaces.py to the new path — or it was
  removed, and §18.5 has lost a surface that its owning leaf must restore.
  ⛔ Do not delete the bullet's route from the mapping to make this green.""",
          file=sys.stderr)
    return 1


def self_test() -> int:
    arms: list[tuple[str, bool]] = []
    live = registered_routes()
    rows = adjudicate(live)
    arms.append(("the live routers expose every mapped route", check(rows) == 0))
    arms.append(("the live verdict is 8 exposed and bullet 8 partial",
                 [r["verdict"] for r in rows].count("exposed") == 8
                 and rows[7]["verdict"] == "partial"))
    # ⛔ a mapped route that vanishes turns the gate red — the property the leaf asked for
    gone = adjudicate(live - {"/v1/admin/backups"})
    arms.append(("a removed route is reported missing",
                 gone[8]["verdict"] == "missing"
                 and gone[8]["missing_routes"] == ["/v1/admin/backups"]))
    arms.append(("…and the gate refuses it", check(gone) == 1))
    # ⛔ a blocked part never promotes a bullet to `exposed`
    arms.append(("a blocked bullet stays partial with all its routes present",
                 adjudicate(live, [SURFACES[7]])[0]["verdict"] == "partial"))
    # ⛔ a route registered only inside #[cfg(test)] is not a product surface
    census = _route_census()
    probe = ('fn r() -> Router { Router::new() }\n#[cfg(test)]\nmod tests {\n'
             '    fn t() { let _ = Router::new().route("/v1/admin/backups", get(x)); }\n}\n')
    arms.append(("a route inside #[cfg(test)] does not count",
                 census.product_routes(probe) == []))
    # …and the SAME route outside it does, or the arm above passes for an
    # unrelated reason (a parser that finds nothing also returns []).
    outside = probe.replace("#[cfg(test)]\n", "")
    arms.append(("the same route outside #[cfg(test)] counts",
                 census.product_routes(outside) == ["/v1/admin/backups"]))
    # ⛔ the mapping file is the ONE list: nine bullets, numbered 1-9, every
    # route written as a path, and nothing silently dropped by the parser.
    arms.append(("the shared mapping holds the nine §18.5 bullets in order",
                 [r["bullet"] for r in SURFACES] == list(range(1, 10))
                 and all(r["routes"] and all(x.startswith("/v1/") for x in r["routes"])
                         for r in SURFACES)))
    failed = [name for name, ok in arms if not ok]
    for name, ok in arms:
        print(f"census_operator_surfaces: arm {'ok' if ok else 'FAILED'} — {name}")
    print(f"census_operator_surfaces --self-test: {len(arms) - len(failed)}/{len(arms)} controls pass")
    return 1 if failed else 0


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    rows = adjudicate(registered_routes())
    if mode == "--check":
        return check(rows)
    if mode == "--json":
        print(json.dumps(rows, indent=2))
        return 0
    report(rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
