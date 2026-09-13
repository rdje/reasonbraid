#!/usr/bin/env python3
"""Census the §9.8 reason-code registry against the codes the product emits.

`ROADMAP.md` §9.8 publishes the stable error model and
`reasonbraid_core::KnownReasonCode` mirrors it. The product also emits codes
that postdate that list; `ReasonCode::Unknown` preserves them verbatim, which is
the designed forward-compatibility path. What nothing publishes is WHICH codes
the product actually emits, so a client author reading §9.8 has no way to learn
about one short of receiving it.

This computes the three sets and checks the book documents every emitted code
(`SIGNOFF-REPAIR.11.7`).

    python3 -B scripts/census_reason_codes.py              # the census
    python3 -B scripts/census_reason_codes.py --check       # gate: book covers every emitted code
    python3 -B scripts/census_reason_codes.py --json
    python3 -B scripts/census_reason_codes.py --self-test

⛔ EMISSION IS SERVER-SIDE. A `code: "…"` literal in `reasonbraid-node` or
`reasonbraid-cli` is a CLIENT constructing a local error when a response body
will not parse — it never travels the wire. Counting those inflates the emitted
set: the raw grep finds 19 distinct codes, of which `unknown` is exactly this
client-side sentinel, so 18 are emitted.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Crates that SERVE the wire. A code literal anywhere else is a client's own
# local error value, not something a client can receive.
EMITTING_CRATES = ("reasonbraid-server",)

_CODE_LITERAL = re.compile(r'code:\s*"([a-z_]+)"')
_REGISTRY = re.compile(r'KnownReasonCode::[A-Za-z]+ => "([a-z_]+)"')
# The book's table rows: | `code` | … |
_BOOK_ROW = re.compile(r"^\|\s*`([a-z_]+)`\s*\|")

BOOK_PAGE = ROOT / "docs" / "book" / "src" / "errors.md"


def rust_sources(crate_names: tuple[str, ...]) -> list[Path]:
    out: list[Path] = []
    for crate in crate_names:
        src = ROOT / "crates" / crate / "src"
        if src.is_dir():
            out.extend(sorted(src.rglob("*.rs")))
    return out


def emitted() -> dict[str, list[str]]:
    """Every wire code the SERVER emits -> the files emitting it."""
    found: dict[str, list[str]] = {}
    for path in rust_sources(EMITTING_CRATES):
        text = path.read_text()
        for code in set(_CODE_LITERAL.findall(text)):
            found.setdefault(code, []).append(str(path.relative_to(ROOT)))
    return {k: sorted(v) for k, v in sorted(found.items())}


def client_side() -> dict[str, list[str]]:
    """Code literals in the CLIENT crates — local sentinels, never wire codes."""
    found: dict[str, list[str]] = {}
    for crate in ("reasonbraid-node", "reasonbraid-cli"):
        for path in rust_sources((crate,)):
            for code in set(_CODE_LITERAL.findall(path.read_text())):
                found.setdefault(code, []).append(str(path.relative_to(ROOT)))
    return {k: sorted(v) for k, v in sorted(found.items())}


def registry() -> list[str]:
    text = (ROOT / "crates" / "reasonbraid-core" / "src" / "error.rs").read_text()
    return sorted(set(_REGISTRY.findall(text)))


def documented() -> list[str]:
    if not BOOK_PAGE.is_file():
        return []
    return sorted(
        {m.group(1) for line in BOOK_PAGE.read_text().splitlines() if (m := _BOOK_ROW.match(line))}
    )


def census() -> dict:
    emit = emitted()
    reg = registry()
    doc = documented()
    return {
        "emitted": emit,
        "registry": reg,
        "documented": doc,
        "client_side": client_side(),
        "emitted_unregistered": sorted(set(emit) - set(reg)),
        "registered_unemitted": sorted(set(reg) - set(emit)),
        "emitted_undocumented": sorted(set(emit) - set(doc)),
    }


def run(mode: str) -> int:
    c = census()
    if mode == "json":
        print(json.dumps(c, indent=2))
        return 0
    if mode == "check":
        missing = c["emitted_undocumented"]
        if missing:
            print("REASON-CODE-DOC: a code the server emits is not in the book's table.", file=sys.stderr)
            for code in missing:
                print(f"    {code}  (emitted by {', '.join(c['emitted'][code])})", file=sys.stderr)
            print(f"  Add it to {BOOK_PAGE.relative_to(ROOT)} — a code a client can", file=sys.stderr)
            print("  receive and cannot look up is the drift this gate exists to stop.", file=sys.stderr)
            return 1
        print(f"REASON-CODE-DOC: all {len(c['emitted'])} emitted codes are documented.")
        return 0

    print(f"§9.8 registry (KnownReasonCode)        : {len(c['registry'])}")
    print(f"codes the SERVER emits                 : {len(c['emitted'])}")
    print(f"  … not in the registry                : {len(c['emitted_unregistered'])}")
    print(f"  … not documented in the book         : {len(c['emitted_undocumented'])}")
    print(f"registry codes never emitted           : {len(c['registered_unemitted'])}")
    print(f"client-side sentinels (NOT wire codes) : {len(c['client_side'])}")
    print()
    print("EMITTED but not in the §9.8 registry (preserved by ReasonCode::Unknown):")
    for code in c["emitted_unregistered"]:
        print(f"  {code:<30} {', '.join(c['emitted'][code])}")
    print()
    print("REGISTERED but never emitted (deliberate — the registry is the COMPLETE")
    print("§9.8 list, not a subset of what this build happens to use):")
    for code in c["registered_unemitted"]:
        print(f"  {code}")
    print()
    print("CLIENT-SIDE sentinels, excluded from the emitted set:")
    for code, files in c["client_side"].items():
        print(f"  {code:<30} {', '.join(files)}")
    return 0


def self_test() -> int:
    failures = []

    def check(name, got, want):
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    # The registry is read from the real source and is the §9.8 list.
    reg = registry()
    check("registry-size", len(reg), 20)
    check("registry-has-known", "protocol_incompatible" in reg, True)

    # ⛔ The client/server split is the part that can silently inflate the
    # emitted set, so it is asserted in BOTH directions against the real tree.
    emit = emitted()
    client = client_side()
    check("unknown-is-client-side", "unknown" in client, True)
    check("unknown-is-not-emitted", "unknown" in emit, False)
    check("server-code-is-emitted", "quota_unconfigured" in emit, True)
    check("server-code-not-client", "quota_unconfigured" in client, False)
    # Every emitted code names at least one file, so a code can always be traced.
    check("every-emitted-has-a-site", [k for k, v in emit.items() if not v], [])

    # The book-row matcher: a table row, and nothing else.
    check("book-row", _BOOK_ROW.match("| `quota_exceeded` | 429 | … |").group(1), "quota_exceeded")
    check("book-row-needs-backticks", _BOOK_ROW.match("| quota_exceeded | 429 |"), None)
    check("book-row-not-prose", _BOOK_ROW.match("The `quota_exceeded` code is …"), None)
    check("book-row-not-header", _BOOK_ROW.match("| Code | Status | Meaning |"), None)

    # The literal matcher tolerates the spacing the codebase actually uses.
    check("literal-tight", _CODE_LITERAL.findall('code:"a_b"'), ["a_b"])
    check("literal-spaced", _CODE_LITERAL.findall('code:   "a_b"'), ["a_b"])
    check("literal-ignores-non-snake", _CODE_LITERAL.findall('code: "A-B"'), [])

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print("census_reason_codes --self-test: 14 controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    if "--check" in args:
        return run("check")
    return run("json" if "--json" in args else "full")


if __name__ == "__main__":
    raise SystemExit(main())
