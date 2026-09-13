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


def stem(route: str) -> str:
    """The route without its axum path parameters, for matching prose."""
    return re.sub(r"\{[^}]*\}", "", route).replace("//", "/").rstrip("/")


def classify(routes: list[str], book: str) -> dict[str, list[str]]:
    described, mentioned, absent = [], [], []
    for route in routes:
        s = stem(route)
        if not s:
            absent.append(route)
            continue
        if any(f"{m} {s}" in book for m in METHODS):
            described.append(route)
        elif s in book:
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

    if stem("/v1/calls/{call_id}/respond") != "/v1/calls/respond":
        failures.append(f"path parameters not stripped: {stem('/v1/calls/{call_id}/respond')}")

    book = "the route `GET /v1/described` returns …\nsomewhere /v1/mentioned appears in prose"
    got = classify(["/v1/described", "/v1/mentioned", "/v1/absent"], book)
    if got["described"] != ["/v1/described"]:
        failures.append(f"described misclassified: {got['described']}")
    if got["mentioned"] != ["/v1/mentioned"]:
        failures.append(f"mentioned misclassified: {got['mentioned']}")
    if got["absent"] != ["/v1/absent"]:
        failures.append(f"absent misclassified: {got['absent']}")

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "census_route_documentation --self-test: 4 product routes collected "
        "including a multi-line one, 2 cfg(test) fixtures excluded (one of them "
        "multi-line), parameters stripped, "
        "described/mentioned/absent each separated"
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
    if args.list:
        for name in ("absent", "mentioned", "described"):
            print(f"\n-- {name} ({len(got[name])})")
            for route in got[name]:
                print(f"     {route}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
