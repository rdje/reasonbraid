#!/usr/bin/env python3
"""Census the server's PRODUCT HTTP routes against what the book documents
(`SIGNOFF-REPAIR.11.8`).

The director reviews the BOOK, not the code (`CLAUDE.md` §7), so a route the
book does not describe is a surface the reviewer cannot see. Nothing derives or
compares the two: `git grep -ln '\\.route(' -- scripts .githooks knowledge-map`
returns zero, so no check reads the route table at all.

⚠️ THE FIRST CENSUS WAS AN UPPER BOUND AND SAID SO. A naive collector reported
"111 routes, 24 named, 87 not" — wrong in both directions at once:

  - it counted FIXTURE routes. `crates/reasonbraid-server/src/fetcher.rs`
    registers `/big`, `/bomb`, `/gz-ok`, `/redir-loop` and friends inside a
    `#[cfg(test)]` module — "a tiny local origin: the routes the refusals and
    the ceilings target" — which is test scaffolding, not API surface. That
    INFLATED the gap;
  - it accepted a passing MENTION as documentation, because it matched the path
    as a substring anywhere in the book. That DEFLATED it.

🔴 AND THE SECOND CENSUS WAS WRONG IN BOTH DIRECTIONS TOO — `SIGNOFF-REPAIR.11.8.1`.
Its key DELETED path parameters, so `/v1/snapshots/{snapshot_id}` and
`/v1/snapshots` collapsed onto one key (the item route credited to the submit
route's line) while `/v1/snapshots/{snapshot_id}/derivations` became
`/v1/snapshots/derivations`, a path no book writes (three documented routes
reported absent). The published `35 described / 68 absent` is superseded by the
counts this file now produces. See `route_key`.

This instrument separates both. A route is a PRODUCT route when it is registered
outside every `#[cfg(test)]` block. A route is DESCRIBED when the book names it
next to an HTTP method (`GET /v1/admin/metrics`), which is the shape a contract
line takes; merely MENTIONED when the path appears without one. The three counts
are reported separately because they answer different questions, and collapsing
them is how the first census went wrong in both directions.

    python3 -B scripts/census_route_documentation.py
    python3 -B scripts/census_route_documentation.py --self-test

Self-test: scripts/census_route_documentation.py --self-test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVER_SRC = "crates/reasonbraid-server/src"
BOOK_SRC = "docs/book/src"
METHODS = ("GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS")


def tracked(pathspec: str) -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", pathspec], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.split()
    return [ROOT / p for p in out]


def test_block_spans(text: str) -> list[tuple[int, int]]:
    """Character ranges covered by `#[cfg(test)]` blocks.

    Brace-depth tracking: a test module is `#[cfg(test)] mod … { … }` and its
    extent is the matching brace, so anything inside is scaffolding whatever it
    looks like. Ranges rather than a line filter, because the route regex below
    must run over the WHOLE text — see `product_routes`.
    """
    spans: list[tuple[int, int]] = []
    depth = 0
    pending = False
    start: int | None = None
    test_depth: int | None = None
    offset = 0
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if test_depth is None and re.match(r"#\[cfg\(test\)\]", stripped):
            pending = True
            start = offset
        opens, closes = line.count("{"), line.count("}")
        if pending and opens:
            test_depth = depth
            pending = False
        depth += opens - closes
        offset += len(line)
        if test_depth is not None and depth <= test_depth:
            spans.append((start or 0, offset))
            test_depth = None
            start = None
    if test_depth is not None:  # unterminated: treat the remainder as test code
        spans.append((start or 0, offset))
    return spans


def product_routes(text: str) -> list[str]:
    """Routes registered OUTSIDE every `#[cfg(test)]` block.

    ⚠️ The regex runs over the WHOLE text, not line by line. `.route(` and its
    path are frequently on separate lines, and a per-line scan silently found
    only 53 of `api.rs`'s 92 — a 42 % undercount that would have been published
    as a smaller documentation gap than the real one. The whitespace class has to
    cross a newline, so the `#[cfg(test)]` exclusion is a character range.
    """
    spans = test_block_spans(text)
    routes: list[str] = []
    for m in re.finditer(r'\.route\(\s*"([^"]+)"', text):
        if any(lo <= m.start() < hi for lo, hi in spans):
            continue
        routes.append(m.group(1))
    return routes


PARAM = re.compile(r"\{[^}]*\}")

# A path continues through word characters, `/`, `{` and `-`. A key may only
# match where the path ENDS, or `/v1/snapshots` matches inside
# `GET /v1/snapshots/{}` and the collection is credited to its item's line.
EDGE_RIGHT = r"(?![\w/{-])"

# ⛔ THERE IS DELIBERATELY NO LEFT EDGE, and the reason is measured rather than
# assumed. One would guard a shorter route matching inside a longer one — but
# `tail_collisions` below reports that hazard at ZERO over this surface, and a
# left edge costs a real answer: the book shows `curl -i "$RB_URL/v1/admin/grants"`
# (`authority.md:375`), where the `L` of the variable is a word character, so
# `/v1/admin/grants` would read `absent` when the book plainly shows it. That is
# the same under-reporting this leaf exists to remove.
#
# The assumption is not left as a comment: the census PRINTS the collision count
# every run, so the day a route's key becomes the tail of another's, the reader
# is told a left edge is now needed instead of quietly getting a wrong number.


def route_key(route: str) -> str:
    """The route with every path parameter NORMALISED, not deleted.

    ⛔ This replaced a `stem()` that deleted parameters outright, and
    `SIGNOFF-REPAIR.11.8.1` measured what that cost — in both directions:

      - `/v1/snapshots/{snapshot_id}` and `/v1/snapshots` both stemmed to
        `/v1/snapshots`, so the book's `POST /v1/snapshots` — the SUBMIT route —
        satisfied the ITEM route's check and it was counted `described` by a
        line documenting a different route. Every collection/item pair in the
        surface had this shape;
      - a parameter in the MIDDLE of a path left a stem no book ever writes:
        `/v1/snapshots/{snapshot_id}/derivations` became
        `/v1/snapshots/derivations`, while the book names it
        `GET /v1/snapshots/{id}/derivations`. Three documented routes read
        `absent`.

    So `.11.8`'s published `35 described / 68 absent` was inflated in BOTH
    columns. Normalising instead — every `{whatever}` to `{}` on both sides —
    matches the book's own spelling and keeps two different routes apart.
    """
    return PARAM.sub("{}", route)


def tail_collisions(routes: list[str]) -> list[tuple[str, str]]:
    """Route pairs where one key is the TAIL of another, at a path boundary.

    The population that would need a left match edge. Reported rather than
    assumed away: it is zero today, and a census that silently depended on that
    would go wrong quietly the day it stopped being true.
    """
    keys = {route: route_key(route) for route in routes}
    return sorted(
        (short, long)
        for short in routes
        for long in routes
        if short != long
        and len(keys[long]) > len(keys[short])
        and keys[long].endswith(keys[short])
    )


def classify(routes: list[str], book: str) -> dict[str, list[str]]:
    book = PARAM.sub("{}", book)
    described, mentioned, absent = [], [], []
    for route in routes:
        key = re.escape(route_key(route))
        if not route_key(route):
            absent.append(route)
            continue
        if any(re.search(f"{re.escape(m)} {key}{EDGE_RIGHT}", book) for m in METHODS):
            described.append(route)
        elif re.search(f"{key}{EDGE_RIGHT}", book):
            mentioned.append(route)
        else:
            absent.append(route)
    return {"described": described, "mentioned": mentioned, "absent": absent}


def self_test() -> int:
    failures: list[str] = []

    source = """
        let app = Router::new().route("/v1/real", get(h)).route("/v1/other", post(h));
        let more = Router::new().route(
            "/v1/multiline",
            get(h),
        );
        #[cfg(test)]
        mod tests {
            fn spawn_origin() {
                Router::new().route("/bomb", get(h)).route(
                    "/ok",
                    get(h),
                );
            }
        }
        fn after() { Router::new().route("/v1/after", get(h)); }
    """
    found = product_routes(source)
    for want in ("/v1/real", "/v1/other", "/v1/after", "/v1/multiline"):
        if want not in found:
            failures.append(f"product route {want} was not collected")
    for fixture in ("/bomb", "/ok"):
        if fixture in found:
            failures.append(f"fixture route {fixture} inside #[cfg(test)] was collected")

    # ⛔ THIS ARM USED TO ASSERT THE DEFECT. It read
    # `stem("/v1/calls/{call_id}/respond") == "/v1/calls/respond"` — codifying a
    # key no book ever writes as the correct answer, which is why the defect
    # survived its own control (`SIGNOFF-REPAIR.11.8.1`).
    if route_key("/v1/calls/{call_id}/respond") != "/v1/calls/{}/respond":
        failures.append(f"parameters not normalised: {route_key('/v1/calls/{call_id}/respond')}")

    book = "the route `GET /v1/described` returns …\nsomewhere /v1/mentioned appears in prose"
    got = classify(["/v1/described", "/v1/mentioned", "/v1/absent"], book)
    if got["described"] != ["/v1/described"]:
        failures.append(f"described misclassified: {got['described']}")
    if got["mentioned"] != ["/v1/mentioned"]:
        failures.append(f"mentioned misclassified: {got['mentioned']}")
    if got["absent"] != ["/v1/absent"]:
        failures.append(f"absent misclassified: {got['absent']}")

    # ── A COLLECTION AND ITS ITEM ARE TWO ROUTES, asserted in BOTH directions.
    # One direction alone passes under a key that is merely too tight.
    only_collection = "submit with `POST /v1/things` and read the result"
    got = classify(["/v1/things", "/v1/things/{thing_id}"], only_collection)
    if got["described"] != ["/v1/things"]:
        failures.append(f"the collection's own line did not describe it: {got}")
    if "/v1/things/{thing_id}" not in got["absent"]:
        failures.append(
            f"THE ITEM ROUTE WAS CREDITED TO THE COLLECTION'S LINE: {got}"
        )

    only_item = "read one with `GET /v1/things/{id}`"
    got = classify(["/v1/things", "/v1/things/{thing_id}"], only_item)
    if got["described"] != ["/v1/things/{thing_id}"]:
        failures.append(f"the item's own line did not describe it: {got}")
    if "/v1/things" not in got["absent"]:
        failures.append(f"THE COLLECTION WAS CREDITED TO THE ITEM'S LINE: {got}")

    # ── A parameter in the MIDDLE of a path must match the book's spelling.
    # The book writes `{id}`; the router writes `{thing_id}`. Deleting the
    # parameter produced `/v1/things/children`, which appears nowhere.
    mid = "traverse with `GET /v1/things/{id}/children` for the tree"
    got = classify(["/v1/things/{thing_id}/children"], mid)
    if got["described"] != ["/v1/things/{thing_id}/children"]:
        failures.append(f"a mid-path parameter did not match the book's spelling: {got}")

    # ── The collision check that stands in for the absent left edge ──
    if tail_collisions(["/v1/things", "/v1/other"]):
        failures.append("tail_collisions reported a collision between unrelated routes")
    if not tail_collisions(["/v1/things", "/v1/scoped/v1/things"]):
        failures.append("tail_collisions missed a key that is the tail of another")
    # And the answer the missing left edge buys: a route behind a variable
    # prefix is still a mention. `authority.md` shows exactly this shape.
    got = classify(["/v1/admin/grants"], 'curl -i "$RB_URL/v1/admin/grants"')
    if got["mentioned"] != ["/v1/admin/grants"]:
        failures.append(f"a route behind a variable prefix was not seen: {got}")

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "census_route_documentation --self-test: 4 product routes collected "
        "including a multi-line one, 2 cfg(test) fixtures excluded (one of them "
        "multi-line), parameters normalised rather than deleted, "
        "described/mentioned/absent each separated, a collection and its item "
        "kept apart in BOTH directions, and a mid-path parameter matched against "
        "the book's own spelling, tail collisions reported in both directions, "
        "and a route behind a variable prefix still seen"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run the instrument's controls")
    parser.add_argument("--list", action="store_true", help="name every route in each class")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    routes: list[str] = []
    for path in tracked(f"{SERVER_SRC}/*.rs") + tracked(f"{SERVER_SRC}/**/*.rs"):
        routes.extend(product_routes(path.read_text()))
    routes = sorted(set(routes))
    book = "\n".join(p.read_text() for p in tracked(f"{BOOK_SRC}/*.md"))
    got = classify(routes, book)

    print(f"product routes (outside #[cfg(test)]): {len(routes)}")
    for name in ("described", "mentioned", "absent"):
        print(f"  {name:10} {len(got[name]):3}")
    print()
    print("  described = the book names it beside an HTTP method (a contract line)")
    print("  mentioned = the path appears, without a method (may be a passing reference)")
    print("  absent    = the book does not name it at all")
    collisions = tail_collisions(routes)
    print()
    if collisions:
        print(f"⚠️  {len(collisions)} route key(s) are the TAIL of another route's key.")
        print("   A short key can now match inside a longer path, so `mentioned`")
        print("   is over-counted and the match needs a left edge. The pairs:")
        for short, long in collisions:
            print(f"     {short}  inside  {long}")
    else:
        print("  0 route keys are the tail of another's, so no left match edge is needed")
    if args.list:
        for name in ("absent", "mentioned", "described"):
            print(f"\n-- {name} ({len(got[name])})")
            for route in got[name]:
                print(f"     {route}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
