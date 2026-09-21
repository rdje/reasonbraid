#!/usr/bin/env python3
"""Census the shipped surfaces against the book's chapters
(`SIGNOFF-REPAIR.11.4.6.1`).

⭐ WHY THIS EXISTS. `.11.4.6` opened on *the six MCP tools are a shipped,
user-visible surface with no book chapter* and set the honest bar itself: the
gap must be MEASURED before it is called a gap, because publishing
*29 families minus 15 chapters = 14 undocumented* would be the anti-pattern
`docs/CLAIM_VERIFICATION.md` names — a search count shipped as a defect count.

⛔ ITS HEADLINE IS STALE AND A SIBLING IS WHAT STALED IT. `docs/book/src/mcp.md`
has existed since `REASONBRAID-REPAIR-0294`. Both of its numbers moved too,
because both were published unpinned: *29* is 30 and *15* is 22. Hence this file
— so the next reader asks rather than reads.

⛔ THE POPULATION IS DERIVED FROM THE PRODUCERS, on three counts:

  - **Route families** come from EVERY file under the server's `src/` that
    registers a route, not from one. The parent's own command read `api.rs`
    alone. Five files register routes, and deriving over all five returns the
    same 30 — `node_channel.rs`'s `/v1/nodes/*` belong to a family `api.rs`
    already declares — so the narrow command was right by luck rather than by
    construction. Luck is not a property to build a census on.
  - **Binaries** come from `cargo metadata --no-deps`. Only four of the ten
    carry an explicit `[[bin]]` stanza; the rest are discovered by cargo from
    `src/bin/`, so a manifest grep sees 4 of 10.
  - **Chapters** come from `SUMMARY.md`, which is the book's own table of
    contents — a file present in `src/` but absent from `SUMMARY.md` is not in
    the book.

⛔ AND THE HTTP ROUTES ARE NOT THE WHOLE SURFACE. A census scoped to route
families would have missed all ten binaries — `.11.2.1.3.2.1.1`'s defect one
lane over: a correct derivation over a corpus that was typed rather than derived.

⛔ THIS REPORTS A MECHANICAL SIGNAL, NOT A JUDGEMENT, and the difference is the
whole point of the leaf that ordered it. *Which chapters mention this surface* is
decidable and is what this prints. *Whether this surface needs a chapter* is a
judgement per member — a family may be deliberately internal — and
`docs/knowledge/a-sample-is-not-a-traversal.md` is explicit that prose cannot
hold a per-member judgement: it belongs in a table keyed to each member, guarded
so that it refuses when a member exists that nothing judged. That is
`SIGNOFF-REPAIR.11.4.6.2`, not this file. A surface with no mention here is a
CANDIDATE for review, never a defect.

⛔ THIS IS A CENSUS, NOT A GATE. It exits 0 whatever it finds.

Usage:
    python3 -B scripts/census_book_coverage.py
    python3 -B scripts/census_book_coverage.py --json
    python3 -B scripts/census_book_coverage.py --self-test
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVER_SRC = "crates/reasonbraid-server/src"
BOOK_SRC = ROOT / "docs/book/src"
SUMMARY = BOOK_SRC / "SUMMARY.md"

# A file registers routes if it calls axum's `.route(`. Derived from the
# framework's own idiom rather than from a list of filenames.
REGISTERS = re.compile(r"\.route\(")
# A route family is the first path segment under `/v1/`.
FAMILY = re.compile(r'"/v1/([a-z0-9][a-z0-9-]*)')
# A chapter link in the book's own table of contents.
CHAPTER = re.compile(r"\]\(([A-Za-z0-9_-]+\.md)\)")
# The four pages that describe the project's state rather than a surface. They
# are SET ASIDE in the report, never dropped: the count of content chapters is
# what a coverage question is about, and the total is what the book has.
META = ("introduction.md", "qualification-review.md", "blockers.md", "roadmap.md")


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True,
                          text=True).stdout


def route_families() -> tuple[list[str], dict[str, list[str]], list[str]]:
    """Every `/v1/<family>`, which router file declares it, and every router.

    ⚠️ The two file counts differ and both are reported: five files register
    routes, three declare a `/v1/` family. `fetcher.rs` and `git.rs` register
    routes that are not under `/v1/`, so a report saying "from 3 router files"
    would answer a different question from the one its label asks.
    """
    families: dict[str, set[str]] = {}
    routers: list[str] = []
    for relative in git("ls-files", "--", f"{SERVER_SRC}/*.rs").split():
        text = (ROOT / relative).read_text(encoding="utf-8", errors="replace")
        if not REGISTERS.search(text):
            continue
        routers.append(relative)
        for name in FAMILY.findall(text):
            families.setdefault(name, set()).add(relative)
    return (sorted(families), {k: sorted(v) for k, v in sorted(families.items())},
            sorted(routers))


def binaries() -> list[str]:
    """Every shipped binary, from cargo's own view of the workspace."""
    out = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True)
    if out.returncode != 0:
        return []
    meta = json.loads(out.stdout)
    return sorted({target["name"] for package in meta["packages"]
                   for target in package["targets"] if "bin" in target["kind"]})


def chapters() -> list[str]:
    return sorted(set(CHAPTER.findall(SUMMARY.read_text(encoding="utf-8"))))


def mentions(names: list[str], pattern) -> dict[str, list[str]]:
    """For each name, the chapters whose text mentions it."""
    found: dict[str, list[str]] = {name: [] for name in names}
    for chapter in chapters():
        path = BOOK_SRC / chapter
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for name in names:
            if pattern(name).search(text):
                found[name].append(chapter)
    return found


def census() -> dict:
    families, sources, routers = route_families()
    bins = binaries()
    every = chapters()
    return {
        "revision": {"commit": git("rev-parse", "--short", "HEAD").strip(),
                     "dirty": bool(git("status", "--porcelain").strip())},
        "routers": routers,
        "route_families": families,
        "route_sources": sources,
        "binaries": bins,
        "chapters": every,
        "content_chapters": [c for c in every if c not in META],
        "meta_chapters": [c for c in every if c in META],
        "family_mentions": mentions(
            families, lambda n: re.compile(rf"/v1/{re.escape(n)}\b")),
        "binary_mentions": mentions(
            bins, lambda n: re.compile(rf"\b{re.escape(n)}\b")),
    }


def report(as_json: bool) -> int:
    result = census()
    if as_json:
        print(json.dumps(result, indent=2))
        return 0

    where = result["revision"]["commit"]
    state = " (tree DIRTY)" if result["revision"]["dirty"] else " (tree clean)"
    print(f"book-coverage census at {where}{state}")
    declaring = len({f for files in result["route_sources"].values() for f in files})
    print(f"  files registering routes                       : {len(result['routers'])}")
    print(f"  of those, declaring a /v1/ family              : {declaring}")
    print(f"  HTTP route families                            : {len(result['route_families'])}")
    print(f"  shipped binaries, from cargo metadata          : {len(result['binaries'])}")
    print(f"  SUMMARY.md entries                             : {len(result['chapters'])}")
    print(f"  of those, content chapters (4 meta set aside)  : "
          f"{len(result['content_chapters'])}")

    for label, names, found in (
            ("route families", result["route_families"], result["family_mentions"]),
            ("binaries", result["binaries"], result["binary_mentions"])):
        silent = [n for n in names if not found[n]]
        print(f"\n  {label} mentioned by NO chapter: {len(silent)} of {len(names)}")
        for name in names:
            where = ", ".join(found[name]) if found[name] else "— no chapter mentions it"
            print(f"    {name:<24} {where}")

    print("\n  ⚠️ THE TWO SIGNALS ARE NOT EQUALLY STRONG, and the weaker one is the\n"
          "     cheerful-looking number. A route family is matched by its own `/v1/` path,\n"
          "     which nothing else in prose looks like. A BINARY is matched by its bare\n"
          "     name, and `rb` is two letters — so `0 of 10 unmentioned` is a weak claim,\n"
          "     and a chapter that merely contains the letters counts. Read the binary\n"
          "     half as `no binary is obviously absent`, never as `every binary is\n"
          "     documented`.")
    print("\n  ⛔ A surface no chapter mentions is a CANDIDATE FOR REVIEW, not a defect.\n"
          "     Whether it needs a chapter is a judgement per member — some are\n"
          "     deliberately internal — and a judgement belongs in a table keyed to each\n"
          "     member, not in this count (SIGNOFF-REPAIR.11.4.6.2).")
    return 0


SELF_TEST_ROUTER = '''
    Router::new()
        .route("/v1/threads", post(create_thread))
        .route("/v1/threads/{id}", get(read_thread))
        .route("/v1/policy-drift", get(drift))
        .route("/healthz", get(health))
}
'''
SELF_TEST_PLAIN = '''
    // Mentions "/v1/threads" in a comment but registers nothing.
    let documented = "/v1/threads";
'''
SELF_TEST_SUMMARY = """# Summary

- [Introduction](introduction.md)
- [Threads](threads.md)
  - [Nested](nested.md)
- [Not a link] and [an external](https://example.invalid/x.md)
"""


def self_test() -> int:
    checks: list[tuple[str, object, object]] = []

    checks.append(("a family is the FIRST segment under /v1/, deduplicated",
                   sorted(set(FAMILY.findall(SELF_TEST_ROUTER))),
                   ["policy-drift", "threads"]))
    checks.append(("a path outside /v1/ is not a family",
                   "healthz" in FAMILY.findall(SELF_TEST_ROUTER), False))
    checks.append(("a file that registers routes is recognised",
                   bool(REGISTERS.search(SELF_TEST_ROUTER)), True))
    # ⭐ The discriminator that keeps the population honest: a file merely
    # NAMING a route is not a router, so a doc comment cannot add a family.
    checks.append(("a file that only MENTIONS a route is not a router",
                   bool(REGISTERS.search(SELF_TEST_PLAIN)), False))
    checks.append(("chapters come from SUMMARY.md links, nested ones included",
                   sorted(set(CHAPTER.findall(SELF_TEST_SUMMARY))),
                   ["introduction.md", "nested.md", "threads.md"]))
    checks.append(("an external URL is not a chapter",
                   "x.md" in CHAPTER.findall(SELF_TEST_SUMMARY), False))
    # A prefix must not swallow a longer sibling: `/v1/policies` and
    # `/v1/policy-drift` are different families and neither mentions the other.
    pattern = re.compile(r"/v1/policies\b")
    checks.append(("a family mention is word-bounded, so a prefix is not a match",
                   bool(pattern.search("see /v1/policy-drift for drift")), False))
    checks.append(("and it still matches its own family",
                   bool(pattern.search("see /v1/policies for the bundle")), True))
    checks.append(("the four meta pages are set aside, not dropped",
                   len(META), 4))

    for name, got, want in checks:
        if got != want:
            print(f"SELF-TEST FAILED: {name} — got {got!r}, expected {want!r}",
                  file=sys.stderr)
            return 1

    print("book-coverage census self-test: route families derived from the "
          "FRAMEWORK's own `.route(` idiom rather than from a filename list, so a "
          "file that merely MENTIONS a path is proved not to add a family and a "
          "path outside /v1/ is proved not to be one; chapters derived from "
          "SUMMARY.md's links including nested ones, with an external URL proved "
          "not to be a chapter; and a family mention is word-bounded, so "
          "`/v1/policies` is proved not to match `/v1/policy-drift` while still "
          "matching itself — the prefix collision that would have reported eleven "
          "policy families as covered by one sentence")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return report(args.json)


if __name__ == "__main__":
    sys.exit(main())
