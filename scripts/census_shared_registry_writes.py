#!/usr/bin/env python3
"""Census every mutating route against the TENANT DIMENSION of what it writes.

`SIGNOFF-REPAIR.7.1.1`, answering `SIGNOFF-REPAIR.7.1`'s attached clause 2 —
two reviewers' records reduced to one finding: *census the whole shared-registry
write scope before designing the bind*.

    python3 -B scripts/census_shared_registry_writes.py            # the census
    python3 -B scripts/census_shared_registry_writes.py --check    # the gate
    python3 -B scripts/census_shared_registry_writes.py --json
    python3 -B scripts/census_shared_registry_writes.py --self-test

⛔ THE CLAUSE EXISTS BECAUSE THE BIND WAS DESIGNED ONCE WITHOUT IT.
`SIGNOFF-REPAIR.3.2` pinned a closed set of six site actions and closed `done`
while `POST /v1/resolvers` still admitted on `authorize_tenant_admin` and
`POST /v1/workflow-profiles` on enrolment alone. A rigorous closed set is still
the wrong closed set if its scope was never measured.

⭐ IT REUSES RATHER THAN RE-DERIVES, and the coupling is deliberate.
`scripts/census_get_route_binding.py` already owns the two primitives this needs
— `tenant_dimensioned_tables()`, which reads the migrations and knows that
`tenant_id` is not the only spelling of a tenant predicate, and the transitive
handler walk. Importing them means one answer to *does this table carry a tenant
dimension*, not two that can drift. ⚠️ The cost is stated: a change there changes
the answer here, which is the point — and it is why this file imports rather
than copies.

⛔ WHAT THIS MEASURES IS THE SCHEMA, NOT THE PREDICATE. A route that writes a
table with no tenant column is writing SITE-GLOBAL state: there is no tenant to
bind, so the question is whether its ADMISSION matches that reach. A route that
writes a tenant-dimensioned table may still omit the predicate, and that is a
different defect with a different instrument (`census_get_route_binding.py`, on
the read side). Conflating them is how 24 site-global reads were nearly reported
as leaks.

⚠️ THE WALK STOPS AT THE FILE BOUNDARY, inherited verbatim from the instrument
it imports. A handler that delegates its write to another module reports the
delegate BY NAME and is classified `undetermined` rather than guessed at —
`--check` refuses those, so the population is never silently under-counted.
"""

from __future__ import annotations

from functools import lru_cache
import importlib.util
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]


def _binding_module():
    """Import the sibling census, whose primitives this one is defined in terms of."""
    path = ROOT / "scripts" / "census_get_route_binding.py"
    spec = importlib.util.spec_from_file_location("census_get_route_binding", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


RB = _binding_module()

#: A write reaches the database through one of these.
#:
#: 🔴 `UPDATE` APPEARS IN SQL IN THREE NON-WRITE POSITIONS, and a regex that
#: ignores them does not report a few odd names — it reports them as TABLES:
#:
#:   * `ON CONFLICT (…) DO UPDATE SET …`  → a table called `set`
#:   * `… FOR UPDATE OF n`                → a table called `of`
#:   * `… FOR UPDATE` at a literal's end  → whatever word came next
#:
#: Measured over this repository before the fix: five names — `set`, `of`,
#: `the`, `insert`, `invite` — across fourteen statements, every one of them an
#: upsert clause or a row lock. ⛔ They looked like prose leaking in from
#: comments, and the first two repairs chased that wrong cause. The tell was
#: that `the` and `invite` are English and `set` and `of` are SQL: one
#: explanation had to cover both, and only the grammar did.
WRITE_VERB = re.compile(
    r"(?<!DO )(?<!FOR )\b(INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+"
    r"(?:ONLY\s+)?(?:public\.)?([a-z_][a-z_0-9]*)",
    re.IGNORECASE,
)

#: A route whose method list contains one of these mutates by declaration. The
#: census does not rely on this alone — it reports the tables — but a mutating
#: route that writes nothing reachable is exactly the `undetermined` class.
MUTATING_METHOD = re.compile(r"\b(post|put|patch|delete)\s*\(", re.IGNORECASE)


def registered_routes() -> list[tuple[str, str, str, str]]:
    """`(file, path, methods, handler)` for every product route registration."""
    out: list[tuple[str, str, str, str]] = []
    for path in RB.tracked("crates/*/src/*.rs") + RB.tracked("crates/*/src/**/*.rs"):
        text = RB.read(ROOT / path)
        if ".route(" not in text:
            continue
        for route, methods, _offset in RB.route_registrations(text):
            for method, handler in re.findall(
                r"\b(get|post|put|patch|delete)\s*\(\s*([a-z_][a-z_0-9]*)", methods
            ):
                out.append((path, route, method.lower(), handler))
    return out


#: How many module hops the walk follows before giving up and SAYING so.
#: ⚠️ A bound rather than a full call graph, and the residual is reported as
#: `unseen-write` rather than as "writes nothing" — see `MAX_DEPTH_NOTE`.
MAX_DEPTH = 6

MAX_DEPTH_NOTE = (
    "the walk is bounded at %d module hops and reports what it could not reach" % MAX_DEPTH
)

SRC = ROOT / "crates" / "reasonbraid-server" / "src"

#: `use crate::site_authority::{self as site, …};` and `use crate::x as y;` —
#: the alias forms the delegate walk must resolve or it drops the call
#: SILENTLY. 🔴 That is exactly what happened: `site_registry_response` calls
#: `site::execute`, no `src/site.rs` exists, and the walk reported "writes
#: nothing" for every site-registry route. ⚠️ The braced `{self as site}` form
#: is the one this repository actually uses, and a regex written for the plain
#: form matched none of them — which is why both are here and why the self-test
#: asserts the braced one.
USE_ALIAS = re.compile(
    r"use\s+crate::([a-z_][a-z_0-9]*)\s*(?:::\s*\{\s*self\s+as\s+([a-z_][a-z_0-9]*)"
    r"|as\s+([a-z_][a-z_0-9]*))",
)


@lru_cache(maxsize=None)
def _aliases(text: str) -> dict[str, str]:
    """`alias -> module` for this file's `use crate::<module>` alias forms."""
    out: dict[str, str] = {}
    for module, braced, plain in USE_ALIAS.findall(text):
        alias = braced or plain
        if alias:
            out[alias] = module
    return out


@lru_cache(maxsize=None)
def _module_text(module: str) -> str | None:
    for candidate in (SRC / f"{module}.rs", SRC / module / "mod.rs"):
        if candidate.exists():
            return RB.read(candidate)
    return None


def sql_literals(body: str) -> list[str]:
    """Each string literal in a body, SEPARATELY — where SQL lives, and only there.

    ⛔ SQL IS READ OUT OF STRING LITERALS AND NOWHERE ELSE, and the first two
    versions of this census did not do that. Scanning raw bodies matched the
    words `UPDATE`, `DELETE FROM` and `INSERT INTO` in COMMENTS and reported
    tables named `of`, `rather`, `sees`, `set`, `the`, `insert` and `invite`.
    Prose about a write is not a write, and this codebase explains its SQL in
    the line above it.

    ⚠️ ONE PASS, TRACKING STATE, rather than strip-comments-then-extract. A URL
    inside a SQL literal contains `//`, so stripping comments first corrupts the
    statement; extracting literals first keeps the commented-out SQL. Only a
    scanner that knows which it is inside gets both right.
    """
    out: list[str] = []
    i, n = 0, len(body)
    while i < n:
        ch = body[i]
        if ch == '"':
            j = i + 1
            buf: list[str] = []
            while j < n:
                if body[j] == "\\":
                    j += 2
                    continue
                if body[j] == '"':
                    break
                buf.append(body[j])
                j += 1
            out.append("".join(buf))
            i = j + 1
        elif body.startswith("//", i):
            i = body.find("\n", i)
            if i == -1:
                break
        elif body.startswith("/*", i):
            end = body.find("*/", i + 2)
            i = n if end == -1 else end + 2
        else:
            i += 1
    return out


def sql_of(body: str) -> str:
    """The literals joined — kept for callers that want one blob to eyeball.

    ⛔ NOT what the census scans. Joining lets a pattern match ACROSS two
    literals: `… FOR UPDATE` ending one and `INSERT INTO event_log` beginning
    the next produced a table called `insert`. The census scans each literal on
    its own, which makes that class impossible rather than filtered.
    """
    return "\n".join(sql_literals(body))


def tables_written(text: str, handler: str) -> tuple[set[str], set[str], bool, set[str]]:
    """`(tables written, delegates followed, hit the depth bound, delegates unresolved)`.

    🔴 THE WALK IS TRANSITIVE ACROSS MODULE BOUNDARIES, and that is not a
    refinement — a one-level walk made this census blind to **27 of 72** routes,
    reporting them as writing nothing at all. `POST /v1/admin/adapters` is the
    clearest case: `allow_adapter` calls the local `site_registry_response`,
    which dispatches a `RegistryCommand` into `site_authority`, where the write
    lives. A write census that stops at the file boundary is not a census of
    writes; it is a census of writes somebody happened to inline.

    ⛔ THE FAILURE MODE IT REPLACES WAS SILENT AND POINTED THE WRONG WAY.
    "No reachable write" reads as *nothing to adjudicate*; it meant *I could not
    see it*. A scoping defect must err towards weakening the finding, so the
    residual is now `unseen-write` and the gate refuses it
    (`docs/knowledge/a-scoping-defect-errs-in-one-direction.md`).
    """
    tables: set[str] = set()
    followed: set[str] = set()
    unresolved: set[str] = set()
    truncated = False
    frontier: list[tuple[str, str, int]] = [("", handler, 0)]
    seen: set[tuple[str, str]] = set()
    while frontier:
        module, name, depth = frontier.pop()
        if (module, name) in seen:
            continue
        seen.add((module, name))
        body_text = text if module == "" else _module_text(module)
        if body_text is None:
            continue
        fn_text = RB.function_body(body_text, name)
        if fn_text is None:
            continue
        body = fn_text[len(RB.signature(fn_text)) :]
        for literal in sql_literals(body):
            for _verb, table in WRITE_VERB.findall(literal):
                tables.add(table.lower())
        if depth >= MAX_DEPTH:
            truncated = True
            continue
        locals_ = RB.local_functions(body_text)
        alias = _aliases(body_text)
        for callee in RB.LOCAL_CALL.findall(body):
            if callee in locals_:
                frontier.append((module, callee, depth))
        for delegate in RB.DELEGATE.findall(body):
            if "::" not in delegate:
                continue
            parts = delegate.split("::")
            next_module, fn = alias.get(parts[-2], parts[-2]), parts[-1]
            if _module_text(next_module) is None:
                # ⛔ NAMED, never dropped. A delegate whose module this
                # instrument cannot open is the reason a verdict is unsafe, and
                # the caller decides what to do with that — the walk does not
                # decide for it by staying quiet.
                unresolved.add(delegate)
                continue
            followed.add(delegate)
            frontier.append((next_module, fn, depth + 1))
    return tables, followed, truncated, unresolved


@lru_cache(maxsize=1)
def admission_by_route() -> dict[str, str]:
    """`route -> admission class`, from the THIRD instrument rather than a fourth.

    ⭐ `scripts/census_admission_paths.py` already answers *how is this caller
    admitted* over its own transitive closure, and `SIGNOFF-REPAIR.11.9`'s rule
    is that an owner which already exists is not duplicated. The whole point of
    clause 2 is the JOIN — reach against admission — and neither half is new.

    ⛔ ITS CORPUS IS ONE FILE, and applying it outside that file produces a
    lie. `census_admission_paths.SOURCE` is `api.rs` alone, while the binary
    merges three routers — so running its classifier over `node_channel.rs`
    reported `POST /v1/nodes/events` and `POST /v1/nodes/rotate` as admitted by
    NOTHING. They are not: both call `verify_fencing(node_id, fencing_token,
    lease_epoch)`, a token admission absent from that census's `GATES`
    vocabulary because that vocabulary was derived from the file it reads.

    ⚠️ So routes outside its corpus are labelled `not-censused`, never `none`.
    Publishing "writes site-global state with no admission" about a route whose
    admission was never examined would be the worst error this census could
    make. Widening that instrument is its own work with its own calibration and
    is NOT done here.
    """
    path = ROOT / "scripts" / "census_admission_paths.py"
    spec = importlib.util.spec_from_file_location("census_admission_paths", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    covered = module.SOURCE
    out: dict[str, str] = {}
    text = RB.read(ROOT / covered)
    for row in module.classify(text):
        out[row["route"]] = row["gate"]
    return out


@lru_cache(maxsize=1)
def module_write_map() -> dict[str, set[str]]:
    """`module -> every table any function in it writes` — the SECOND ARM.

    ⭐ A DIFFERENT QUESTION FROM THE WALK'S, DELIBERATELY. The walk follows
    calls and under-reports: it cannot see trait dispatch, a method on a struct,
    or a submodule under `src/<module>/<file>.rs`, so **26 of 72** mutating
    routes reached no write at all — including `POST /v1/admin/regions`, which
    plainly declares one.

    This arm asks the corpus instead of the call graph: which tables does each
    module write, anywhere in it? Crossed with the modules a route REACHES —
    which the walk does report reliably — it gives a reach that can only be too
    WIDE, never too narrow. `docs/CLAIM_VERIFICATION.md` leg 2: prefer an oracle
    you did not build, and when both arms are yours, make them answer different
    questions so they cannot share a blind spot.

    ⚠️ It is an OVER-approximation and every row says so. A module that writes
    five tables lends all five to any route that touches it, and only the walk
    can say which one this route's statement named.
    """
    out: dict[str, set[str]] = {}
    for path in sorted(SRC.rglob("*.rs")):
        module = path.stem if path.name != "mod.rs" else path.parent.name
        tables = out.setdefault(module, set())
        for literal in sql_literals(RB.read(path)):
            for _verb, table in WRITE_VERB.findall(literal):
                tables.add(table.lower())
    return out


def modules_reached(text: str, handler: str) -> set[str]:
    """Every module the handler's call graph touches, however shallowly."""
    reached: set[str] = set()
    frontier: list[tuple[str, str, int]] = [("", handler, 0)]
    seen: set[tuple[str, str]] = set()
    while frontier:
        module, name, depth = frontier.pop()
        if (module, name) in seen or depth > MAX_DEPTH:
            continue
        seen.add((module, name))
        body_text = text if module == "" else _module_text(module)
        if body_text is None:
            continue
        fn_text = RB.function_body(body_text, name)
        if fn_text is None:
            continue
        body = fn_text[len(RB.signature(fn_text)) :]
        locals_ = RB.local_functions(body_text)
        alias = _aliases(body_text)
        for callee in RB.LOCAL_CALL.findall(body):
            if callee in locals_:
                frontier.append((module, callee, depth))
        for delegate in RB.DELEGATE.findall(body):
            if "::" not in delegate:
                continue
            parts = delegate.split("::")
            target = alias.get(parts[-2], parts[-2])
            if _module_text(target) is not None:
                reached.add(target)
                frontier.append((target, parts[-1], depth + 1))
    return reached


def verdict_for(site: list[str], tenant: list[str], unknown: list[str]) -> str:
    """The reach verdict for one route's tables.

    ⛔ A FREE FUNCTION so the self-test drives the REAL classifier. It began as
    a copy inside the self-test, which is a control sharing a parent with the
    thing it checks — `docs/CLAIM_VERIFICATION.md` §2: their agreement carries
    no information.
    """
    if not (site or tenant or unknown):
        # NOT "writes nothing". Neither arm reached a write, and for a mutating
        # verb that is an absence of evidence rather than evidence of absence.
        return "unseen-write"
    if unknown:
        return "undetermined"
    if site and tenant:
        return "mixed"
    return "site-global" if site else "tenant-scoped"


@lru_cache(maxsize=1)
def _collect_cached() -> tuple:
    return tuple(json.dumps(r, sort_keys=True) for r in _collect())


def collect() -> list[dict[str, object]]:
    """The census. ⭐ Memoized: every arm of the self-test wants it, and
    each call walks the whole source tree — three calls cost more than the
    doctrine enforcer it runs inside (`SIGNOFF-REPAIR.11.5`: a gate people
    route around is a gate that lies)."""
    return [json.loads(r) for r in _collect_cached()]


def _collect() -> list[dict[str, object]]:
    dimensioned = RB.tenant_dimensioned_tables()
    writes_by_module = module_write_map()
    admission = admission_by_route()
    rows: list[dict[str, object]] = []
    cache: dict[str, str] = {}
    for path, route, method, handler in registered_routes():
        if method == "get":
            continue
        text = cache.setdefault(path, RB.read(ROOT / path))
        tables, followed, truncated, unresolved = tables_written(text, handler)
        arm = "walk"
        if not tables:
            # The walk saw nothing. Fall back to the sound over-approximation
            # and SAY which arm answered, so no reader mistakes a widened reach
            # for a statement this route actually issues.
            reached = modules_reached(text, handler)
            own_module = Path(path).stem
            for module in reached | {own_module}:
                tables |= writes_by_module.get(module, set())
            arm = "module-reach" if tables else "none"
        known = {t: dimensioned.get(t) for t in sorted(tables)}
        site_global = sorted(t for t, d in known.items() if d is False)
        tenant_scoped = sorted(t for t, d in known.items() if d is True)
        unknown = sorted(t for t, d in known.items() if d is None)
        verdict = verdict_for(site_global, tenant_scoped, unknown)
        rows.append(
            {
                "route": f"{method.upper()} {route}",
                "handler": handler,
                "file": path,
                "verdict": verdict,
                "site_global": site_global,
                "tenant_scoped": tenant_scoped,
                "unknown": unknown,
                "delegates_followed": sorted(followed),
                "depth_bound_hit": truncated,
                "delegates_unresolved": sorted(unresolved),
                "arm": arm,
                "admission": admission.get(f"{method.upper()} {route}", "not-censused"),
            }
        )
    rows.sort(key=lambda r: (str(r["verdict"]), str(r["route"])))
    return rows


def report(rows: list[dict[str, object]]) -> None:
    counts: dict[str, int] = {}
    for row in rows:
        counts[str(row["verdict"])] = counts.get(str(row["verdict"]), 0) + 1
    print(f"mutating routes: {len(rows)}")
    print("  " + " · ".join(f"{v} {n}" for v, n in sorted(counts.items())))
    reaching = [r for r in rows if r["site_global"]]
    precise = [r for r in reaching if r["arm"] == "walk"]
    print(f"\n  routes whose reach includes a SITE-GLOBAL table: {len(reaching)} "
          f"({len(precise)} seen by the precise walk, "
          f"{len(reaching) - len(precise)} only by the module-reach over-approximation)")

    print("\n== SITE-GLOBAL WRITES BY ADMISSION — the join clause 2 asks for ==")
    by_gate: dict[str, list[dict[str, object]]] = {}
    for row in precise:
        by_gate.setdefault(str(row["admission"]), []).append(row)
    for gate in sorted(by_gate):
        print(f"\n-- admitted by `{gate}`: {len(by_gate[gate])} routes")
        for row in sorted(by_gate[gate], key=lambda r: str(r["route"])):
            print(f"   {str(row['route']):<54} {', '.join(row['site_global'])}")  # type: ignore[arg-type]

    for verdict in sorted(counts):
        print(f"\n-- {verdict}: {counts[verdict]}")
        for row in [r for r in rows if r["verdict"] == verdict]:
            tables = ", ".join(
                list(row["site_global"]) + list(row["tenant_scoped"]) + list(row["unknown"])  # type: ignore[arg-type]
            )
            print(f"   {str(row['route']):<52} [{row['arm']}/{row['admission']}] {tables}")


_USE_RECORDED = object()

BASELINE = ".doctrine/shared_registry_baseline.tsv"

#: The live documents that restate this census's counts. ⛔ `.13.4`'s standing
#: corpus rule — *a correction is not complete until the LIVE-DOCUMENT corpus
#: that restates it has been censused* — turned into a mechanism instead of a
#: habit: when the population moves, the refusal names where the old number is
#: still written.
RESTATING_DOCUMENTS = (
    "CHANGELOG.md",
    "LIVE_STATUS.md",
    "MEMORY.md",
    "docs/TASK_TREE.md",
    "docs/tasks/SIGNOFF-REPAIR.md",
)


def baseline_rows(rows: list[dict[str, object]]) -> list[str]:
    """The site-global writers as `route<TAB>admission<TAB>tables`, sorted.

    ⭐ THE SET, NOT THE COUNT. A count is not an identity: one route added and
    one removed holds `42` while the population changes underneath it. The
    published numbers are re-derived FROM this file's rows rather than stored
    beside them, so the two cannot disagree.
    """
    out = []
    for row in rows:
        if not row["site_global"]:
            continue
        out.append("\t".join([
            str(row["route"]),
            str(row.get("admission", "?")),
            ",".join(row["site_global"]),  # type: ignore[arg-type]
            str(row.get("arm", "?")),
        ]))
    return sorted(out)


def load_baseline(path: Path | None = None) -> list[str] | None:
    path = path if path is not None else ROOT / BASELINE
    if not path.exists():
        return None
    return sorted(
        l.rstrip("\n") for l in path.read_text(encoding="utf-8").splitlines()
        if l.strip() and not l.lstrip().startswith("#")
    )


def baseline_drift(rows: list[dict[str, object]],
                   recorded: list[str] | None) -> tuple[list[str], list[str]]:
    """`(rows that appeared, rows that vanished or changed)`."""
    if recorded is None:
        return ([], [])
    live = set(baseline_rows(rows))
    was = set(recorded)
    return (sorted(live - was), sorted(was - live))


def check(rows: list[dict[str, object]],
          recorded: list[str] | None | object = _USE_RECORDED) -> int:
    """⛔ The gate refuses what the census could not classify, never what it found.

    A site-global write is a FINDING for a leaf to adjudicate, not a breach to
    block a commit on — `SIGNOFF-REPAIR.11.9` rejects a gate that presents a
    standing backlog. What must never happen silently is a route this instrument
    cannot classify, because that is the census under-counting its own
    population while reporting a tidy number.
    """
    blind = [r for r in rows if r["verdict"] in ("undetermined", "unseen-write")]
    # ⛔ The baseline is a PARAMETER so a control can drive the gate without the
    # real one, and so an arm testing the blind-route path is not also asserting
    # the whole live population. Defaulting it to the recorded file keeps the
    # command-line behaviour unchanged.
    if recorded is _USE_RECORDED:
        recorded = load_baseline()
    appeared, vanished = baseline_drift(rows, recorded)  # type: ignore[arg-type]
    if not blind and not appeared and not vanished:
        precise = [r for r in rows if r["site_global"] and r.get("arm") == "walk"]
        identity = [r for r in precise if r.get("admission") == "identity only"]
        print(f"SHARED-REGISTRY-WRITES: {len(rows)} mutating routes, all classified; "
              f"{len(precise)} site-global writers ({len(identity)} on identity alone), "
              f"matching the recorded baseline")
        return 0
    if appeared or vanished:
        print("SHARED-REGISTRY-WRITES: the site-global write population has MOVED.",
              file=sys.stderr)
        for row in appeared:
            print(f"    + {row}", file=sys.stderr)
        for row in vanished:
            print(f"    - {row}", file=sys.stderr)
        print(f"""
  The baseline pins the SET, not the count, because a count is not an identity:
  one route added and one removed leaves the number unchanged while the
  population moves underneath it.

  Re-derive, adjudicate what changed, then refresh {BASELINE}. ⛔ The counts are
  restated in these live documents and a correction is not complete until they
  have been censused ({'`SIGNOFF-REPAIR.13.4`'}):
""" + "\n".join(f"      {d}" for d in RESTATING_DOCUMENTS), file=sys.stderr)
    for row in blind:
        print(
            f"SHARED-REGISTRY-WRITES: cannot classify {row['route']} "
            f"({row['file']}) — writes {', '.join(row['unknown'])}, "  # type: ignore[arg-type]
            f"which no migration creates",
            file=sys.stderr,
        )
    print("""
  A table this census cannot find in `migrations/` has no known tenant
  dimension, so the route's reach is unknown — and an unknown reach counted as
  a tidy number is the census under-reporting its own population.

  Either the table is created somewhere this instrument does not read, or the
  write is a false positive of the statement scan. Fix whichever it is; do not
  widen the verdict to make this green.""", file=sys.stderr)
    return 1


def self_test() -> int:
    arms: list[tuple[str, bool]] = []

    # ── The write-statement scan ─────────────────────────────────────────────
    arms.append(("an INSERT is a write",
                 WRITE_VERB.findall("INSERT INTO resolver_capabilities (a) VALUES ($1)")
                 == [("INSERT INTO", "resolver_capabilities")]))
    arms.append(("an UPDATE is a write",
                 WRITE_VERB.findall("UPDATE node_leases SET x = 1") == [("UPDATE", "node_leases")]))
    arms.append(("a DELETE is a write",
                 WRITE_VERB.findall("DELETE FROM site_grants WHERE id = $1")
                 == [("DELETE FROM", "site_grants")]))
    # 4 ⛔ NEGATIVE — a SELECT is not a write. Without this the census would
    #   classify every reading route as a mutation and the population would be
    #   the whole API.
    arms.append(("a SELECT is NOT a write",
                 WRITE_VERB.findall("SELECT a FROM resolver_capabilities WHERE b = $1") == []))
    # 5 ⭐ the schema qualifier and ONLY are stripped, or the same table reads as
    #   two different ones depending on how the statement was spelled.
    arms.append(("a schema-qualified table is the same table",
                 WRITE_VERB.findall("DELETE FROM public.site_audit") == [("DELETE FROM", "site_audit")]))
    arms.append(("UPDATE ONLY names the same table",
                 WRITE_VERB.findall("UPDATE ONLY tenants SET a = 1") == [("UPDATE", "tenants")]))
    # 7 several statements in one body are all found
    arms.append(("every statement in a body is found",
                 len(WRITE_VERB.findall(
                     "INSERT INTO a (x) VALUES (1); UPDATE b SET y = 2; DELETE FROM c")) == 3))

    # ── The imported primitive, exercised rather than trusted ────────────────
    dimensioned = RB.tenant_dimensioned_tables()
    # 8 ⭐ THE COUPLING IS ASSERTED. If the sibling census stops answering this
    #   question, this instrument must fail loudly rather than classify
    #   everything as site-global — which is what an empty dict would do.
    arms.append(("the imported tenant-dimension primitive returns a real corpus",
                 len(dimensioned) > 20))
    # 9 a known SITE-GLOBAL table, named by `.11.9.1.1.1`
    arms.append(("resolver_capabilities is known to carry NO tenant dimension",
                 dimensioned.get("resolver_capabilities") is False))
    # 10 ⛔ NEGATIVE — and a known tenant-dimensioned one, or arm 9 passes under
    #    a primitive that answers False for everything.
    arms.append(("a tenant-dimensioned table is known to carry one",
                 any(v for v in dimensioned.values())))
    arms.append(("workflow_profiles is known to carry NO tenant dimension",
                 dimensioned.get("workflow_profiles") is False))

    # ── The verdict classification ───────────────────────────────────────────
    verdict_of = verdict_for

    arms.append(("a write to an untenanted table alone is site-global",
                 verdict_of(["resolver_capabilities"], [], []) == "site-global"))
    arms.append(("a write to a tenanted table alone is tenant-scoped",
                 verdict_of([], ["threads"], []) == "tenant-scoped"))
    # 14 ⭐ MIXED IS ITS OWN CLASS, not folded into either. A route that writes
    #    both kinds cannot be adjudicated by the site-global half alone.
    arms.append(("a route writing both kinds is mixed, not site-global",
                 verdict_of(["resolver_capabilities"], ["threads"], []) == "mixed"))
    # 15 ⛔ NEGATIVE — an unknown table dominates, so the census can never report
    #    a confident verdict over a table it could not find.
    arms.append(("an unknown table makes the verdict undetermined",
                 verdict_of(["resolver_capabilities"], ["threads"], ["mystery"]) == "undetermined"))

    # ── The gate ─────────────────────────────────────────────────────────────
    clean = [{"route": "POST /x", "handler": "h", "file": "f", "verdict": "site-global",
              "site_global": ["resolver_capabilities"], "tenant_scoped": [], "unknown": [],
              "delegates_followed": []}]
    # 16 ⭐ A SITE-GLOBAL WRITE DOES NOT FAIL THE GATE. It is a finding for a
    #    leaf, and `.11.9` rejects a gate that ships a standing backlog.
    arms.append(("the gate PASSES a site-global finding", check(clean, None) == 0))
    # 17 ⛔ NEGATIVE, observed RED: an unclassifiable route refuses.
    blind = [dict(clean[0], verdict="undetermined", unknown=["mystery"])]
    arms.append(("the gate REFUSES a route it could not classify", check(blind, None) == 1))

    # ── The SQL grammar, every arm a defect this census actually shipped ─────
    # ⛔ Each of these five was live and produced a table name. They are arms,
    # not comments, because a fixed defect with no arm is a defect waiting for
    # the next edit (`docs/knowledge/the-commit-that-reshapes-a-file-blinds-its-guard.md`).
    arms.append(("`DO UPDATE SET` is not a table called `set`",
                 WRITE_VERB.findall("ON CONFLICT (resolver_id) DO UPDATE SET schemes = $2") == []))
    arms.append(("`FOR UPDATE` is not a write",
                 WRITE_VERB.findall("SELECT a FROM t WHERE b = $1 FOR UPDATE") == []))
    arms.append(("`FOR UPDATE OF n` is not a table called `of`",
                 WRITE_VERB.findall("SELECT a FROM t WHERE b = $1 FOR UPDATE OF n") == []))
    # 🔴 THE CROSS-LITERAL MATCH. Joining literals let `… FOR UPDATE` ending one
    #    and `INSERT INTO event_log` beginning the next report a table `insert`.
    body = 'let a = "SELECT x FROM t FOR UPDATE"; let b = "INSERT INTO event_log (y) VALUES (1)";'
    found = {t for lit in sql_literals(body) for _v, t in WRITE_VERB.findall(lit)}
    arms.append(("a match cannot span two string literals", found == {"event_log"}))
    # ⛔ and a real upsert still reports its own table, or the grammar arms are
    #    passed by a regex that matches nothing at all.
    arms.append(("a real upsert still names its table",
                 [t for _v, t in WRITE_VERB.findall(
                     "INSERT INTO resolver_capabilities (a) VALUES ($1) ON CONFLICT (b) DO UPDATE SET c = $2")]
                 == ["resolver_capabilities"]))

    # ── Comments are not SQL ─────────────────────────────────────────────────
    arms.append(("a // comment mentioning a write is not a write",
                 sql_literals('// we INSERT INTO audit here\nlet q = "SELECT 1";') == ["SELECT 1"]))
    arms.append(("a /* block */ comment is not a write",
                 sql_literals('/* DELETE FROM x */ let q = "SELECT 1";') == ["SELECT 1"]))
    # ⭐ and the converse: a URL inside a literal contains `//` and must NOT be
    #    treated as a comment. Stripping comments first would have broken this.
    arms.append(("a // inside a literal does not start a comment",
                 sql_literals('let q = "INSERT INTO t (u) VALUES (\'https://x/y\')";')
                 == ["INSERT INTO t (u) VALUES ('https://x/y')"]))

    # ── The module alias ─────────────────────────────────────────────────────
    # 🔴 The braced form is the one this repository uses, and a regex written
    #    for the plain form matched NONE of them — so every site-registry route
    #    reported "writes nothing".
    arms.append(("the braced `{self as site}` alias resolves",
                 _aliases("use crate::site_authority::{self as site, Reason};").get("site")
                 == "site_authority"))
    arms.append(("the plain `as` alias resolves",
                 _aliases("use crate::regions as r;").get("r") == "regions"))

    # ── The admission join's scope ───────────────────────────────────────────
    # ⛔ Routes outside the admission census's one-file corpus must read
    #    `not-censused`, never `none`: publishing "no admission" about a route
    #    whose admission was never examined is the worst error available here.
    try:
        adm = admission_by_route()
        scoped = "POST /v1/nodes/events" not in adm
    except Exception:                                              # noqa: BLE001
        scoped = False
    arms.append(("a route outside the admission census's corpus is not given a gate",
                 scoped))
    try:
        node_rows = [r for r in collect() if r["route"] == "POST /v1/nodes/events"]
        labelled = bool(node_rows) and node_rows[0]["admission"] == "not-censused"
    except Exception:                                              # noqa: BLE001
        labelled = False
    arms.append(("…and is labelled `not-censused` rather than `none`", labelled))

    # ── The schema-qualified CREATE TABLE, repaired in the imported primitive ─
    # 🔴 Four tables — the whole site-authority family — were invisible to
    #    `tenant_dimensioned_tables()` because its pattern forbade a qualifier.
    arms.append(("a schema-qualified table is known to the tenant-dimension primitive",
                 RB.tenant_dimensioned_tables().get("site_audit") is False))

    # ── The baseline: the SET is pinned, not the count ───────────────────────
    # 🔴 `SIGNOFF-REPAIR.7.1.1.1`. The published 42 and 33 were guarded by
    #    NOTHING while this instrument's core was rewritten three times in one
    #    sitting, and every intermediate count moved. These arms are that gap
    #    closed, and all three drift shapes were observed RED in situ.
    live_rows = collect()
    recorded = baseline_rows(live_rows)
    arms.append(("the live census matches its recorded baseline",
                 baseline_drift(live_rows, load_baseline()) == ([], [])))
    # ⛔ a route APPEARS — the row is missing from the baseline
    arms.append(("a route that APPEARS is refused",
                 baseline_drift(live_rows, [r for r in recorded if "/v1/resolvers" not in r])[0] != []))
    # ⛔ a route VANISHES — the baseline holds a row the census no longer finds
    arms.append(("a route that VANISHES is refused",
                 baseline_drift(live_rows, recorded + ["POST /v1/ghost\tidentity only\tg\twalk"])[1]
                 != []))
    # 🔴 THE ONE A COUNT CANNOT CATCH: same route, changed admission. One row in,
    #    one row out — the number is identical and the population is not.
    changed = [r.replace("pool tenant-admin", "identity only") if "/v1/resolvers" in r else r
               for r in recorded]
    app, van = baseline_drift(live_rows, changed)
    arms.append(("a CHANGED row is refused, though the count is unmoved",
                 len(app) == 1 and len(van) == 1 and len(changed) == len(recorded)))
    # ⛔ NEGATIVE: an absent baseline does not fabricate drift — a fresh clone
    #    that has not recorded one must not read as 68 vanished routes.
    arms.append(("an absent baseline reports no drift rather than total drift",
                 baseline_drift(live_rows, None) == ([], [])))
    # ⭐ and the refusal names where the number is restated, or `.13.4`'s corpus
    #    rule stays a habit instead of a mechanism.
    arms.append(("the restating documents are named and tracked",
                 all((ROOT / d).exists() for d in RESTATING_DOCUMENTS)))
    # ⛔ the published pair is DERIVED from the baseline rows, never carried
    #    beside them, so the file and the number cannot disagree.
    precise = [r for r in live_rows if r["site_global"] and r["arm"] == "walk"]
    # ⛔ 42/33 → 41/32 at `.7.1.2.1` → 39/30 at `.7.1.2.2`, which bound BOTH
    # routing journals to their tenant, so `routing_resolutions` and
    # `routing_recommendations` left the site-global population entirely and
    # `POST /v1/routing/resolve` and `/recommendations` stopped being
    # site-global writers at all. Each movement is a repair, never a recount.
    #
    # ⛔ 42/33 until `SIGNOFF-REPAIR.7.1.2.1`, and that movement has ONE cause
    # rather than a recount: `POST /v1/workflow-profiles` was repaired into a
    # site act, so it left the precise walk (its handler now delegates across a
    # module boundary) AND left `identity only`. The other seven routes the same
    # leaf relabelled `site authority` were already `module-reach`, so they were
    # never in `precise` and neither figure moved for them.
    arms.append(("the published 39/30 are derived from the rows, not stored",
                 len(precise) == 39
                 and sum(1 for r in precise if r["admission"] == "identity only") == 30))

    # ── The live corpus ──────────────────────────────────────────────────────
    try:
        live = collect()
        live_ok = len(live) > 20 and all(r["verdict"] for r in live)
        routes = {str(r["route"]) for r in live}
        has_resolvers = "POST /v1/resolvers" in routes
    except Exception as exc:                                       # noqa: BLE001
        live_ok, has_resolvers = False, False
        print(f"census_shared_registry_writes: live-corpus arm raised {exc!r}")
    arms.append(("live-corpus: the real routes parse and every row has a verdict", live_ok))
    # 19 ⭐ and the route the clause is ABOUT is in the population, or the census
    #    is answering a different question from the one it was built for.
    arms.append(("live-corpus: POST /v1/resolvers is in the population", has_resolvers))
    # 20 ⛔ NEGATIVE: GET routes are excluded, or this becomes the read census.
    try:
        methods = {str(r["route"]).split(" ", 1)[0] for r in collect()}
    except Exception:                                              # noqa: BLE001
        methods = {"GET"}
    arms.append(("live-corpus: no GET route is in the write population", "GET" not in methods))

    passed = sum(1 for _, ok in arms if ok)
    for name, ok in arms:
        print(f"census_shared_registry_writes: {'arm ok' if ok else 'arm FAILED'} — {name}")
    print(f"census_shared_registry_writes --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    rows = collect()
    if mode == "--check":
        return check(rows)
    if mode == "--json":
        print(json.dumps(rows, indent=2, sort_keys=True))
        return 0
    report(rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
