#!/usr/bin/env bash
# scripts/check_book_links.sh — BOOK-LINKS doctrine.
#
# Every intra-book Markdown link resolves to a file that exists.
#
# `mdbook build` does not validate links, so three dead ones sat in the rendered
# book — the surface the director actually reads — until a census found them.
# Their cause is instructive: the DOCPATH doctrine requires repo-root-relative
# references so no checkout-specific absolute path is embedded, and an author
# applied that rule to INTRA-BOOK navigation, where mdBook resolves relative to
# the source file. `docs/book/src/cli.md` therefore rendered
# `href="docs/book/src/cli-state.html"`, which does not exist; the page is at
# `cli-state.html`. The doctrine's intent was satisfied and the navigation broke.
#
# So this is not a style rule. It is the check that makes the book's own
# navigation an enforced property rather than an assumption. Around thirty links
# resolve in milliseconds.
#
# Out of scope by design: external URLs (a network dependency in a commit hook
# is a flake generator) and prose mentions of repository files, which the book
# writes as inline code rather than as links.
#
# Self-test: scripts/check_book_links.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

SELF_TEST="${1:-}"

python3 -B - "$SELF_TEST" <<'PY'
import pathlib
import re
import sys

LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+?)(?:#[^)]*)?\)")
SKIP = ("http://", "https://", "mailto:", "#")


def targets(text):
    """Every link target in one page that should resolve on disk."""
    for match in LINK.finditer(text):
        target = match.group(1)
        if not target.startswith(SKIP):
            yield target


if sys.argv[1] == "--self-test":
    fails = 0
    cases = [
        ("[a](cli-state.md)", ["cli-state.md"]),
        ("[a](docs/book/src/cli-state.md)", ["docs/book/src/cli-state.md"]),
        ("[a](cli-state.md#anchor)", ["cli-state.md"]),
        ("[a](https://example.org/x)", []),
        ("[a](mailto:x@example.org)", []),
        ("[a](#local-anchor)", []),
        ("see `docs/tasks/whatever.md` in the repository", []),
        ("[a](one.md) and [b](two.md)", ["one.md", "two.md"]),
    ]
    for text, expected in cases:
        got = list(targets(text))
        if got != expected:
            print(f"SELF-TEST: {text!r} -> {got!r}, expected {expected!r}", file=sys.stderr)
            fails += 1
    if fails:
        sys.exit(1)
    print(
        "BOOK-LINKS self-test: 8 extractions verified — anchors stripped, "
        "external schemes skipped, inline-code paths ignored"
    )
    sys.exit(0)

src = pathlib.Path("docs/book/src")
if not src.is_dir():
    sys.exit(0)

broken = []
checked = 0
for page in sorted(src.rglob("*.md")):
    for target in targets(page.read_text()):
        checked += 1
        if not (page.parent / target).resolve().exists():
            broken.append((page, target))

if broken:
    print(
        "BOOK-LINKS: an intra-book link does not resolve. mdbook does not check "
        "these, so a dead link ships to the rendered book.",
        file=sys.stderr,
    )
    for page, target in broken:
        print(f"    {page}: [...]({target})", file=sys.stderr)
    print(
        "  Book pages link to each other by SIBLING filename (cli-state.md). A "
        "repo-root path is correct for prose references written as inline code, "
        "and wrong for navigation: mdbook resolves it relative to the page.",
        file=sys.stderr,
    )
    sys.exit(1)

sys.exit(0)
PY
