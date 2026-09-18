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
set: the raw grep finds 20 distinct codes, of which `unknown` is exactly this
client-side sentinel, so 19 are emitted. ⛔ Both numbers are a function of the
tree, so do not read them from this line — `--json` re-derives `emitted` and
`client_side` on every run. `SIGNOFF-REPAIR.9.2.1.1` moved them from 19/18.
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
# A bare snake_case string literal, used when reading a producer body.
_CODE_ONLY = re.compile(r'"([a-z_][a-z0-9_]*)"')

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


# ── The SECOND refusal vocabulary (`SIGNOFF-REPAIR.7.2.10`) ────────────────────
# ⛔ `AcquisitionError` carries `kind`, NOT `code`. A different field, a
# different namespace, and — measured, not assumed — it shares not one string
# with the §9.8 registry. So `REASON-CODE-DOC`'s `code:`-keyed census is right
# to exclude it, and the gap that leaves is real: a client branches on these too.
#
# ⭐ ANCHORED ON THE CONSTRUCTION, not on a literal scan and not on a marker
# string. Two earlier keys were both wrong in the same census:
#   - `kind: match &error {` MISSED the `GitError` site, which is written
#     `kind: match error {` — sixteen kinds invisible;
#   - `kind:\s*"…"` also matched `derived_kind:` and `actor_kind:`, sweeping in
#     `chunk`, `database`, `approval` and `decision` from unrelated structs.
# Walking each `AcquisitionError { … }` from its `kind:` to its `message:` has
# neither failure, because it reads the producer rather than a pattern that
# resembles one.
#
# ⛔ A `git grep` for `=> "…"` over the server returns 93 distinct literals, of
# which these are one subset among many unrelated mappings — a POPULATION, not a
# vocabulary (`docs/CLAIM_VERIFICATION.md` leg 2). It is not published as
# anything, and this function is why it does not have to be.
_CODE_ONLY = re.compile(r'"([a-z_][a-z0-9_]*)"')
_KIND_FIELD = re.compile(r"^\s*kind:")
_MESSAGE_FIELD = re.compile(r"^\s*message:")
_LET_KIND = re.compile(r"^\s*let kind = match\b")
# The arms that forward a kind the WORKER chose, so the set is OPEN, not closed.
_PASSTHROUGH = re.compile(r"WorkerRefused\s*\{\s*kind")
# The one `fn kind()` that feeds an acquisition error. `recruitment.rs` has one
# too and it is a different vocabulary entirely (join / decline / defer).
_FETCH_KIND = ("crates/reasonbraid-server/src/fetcher.rs", "pub fn kind(&self) -> &'static str {")


def _brace_body(lines: list[str], start: int) -> list[str]:
    """The brace-matched block beginning at index `start`.

    ⛔ Brace-matched rather than a fixed window: a 50-line window overran
    `FetchError::kind` into the next item and swept up the `snake_case` of a
    `#[serde(rename_all = …)]` attribute.
    """
    depth, out, opened = 0, [], False
    for line in lines[start:]:
        out.append(line)
        depth += line.count("{") - line.count("}")
        opened = opened or "{" in line
        if opened and depth <= 0:
            break
    return out


def _literals(lines: list[str]) -> list[str]:
    out: list[str] = []
    for line in lines:
        if line.lstrip().startswith("//"):
            continue                        # prose, not an arm
        out.extend(_CODE_ONLY.findall(line))
    return out


def acquisition_kinds() -> dict[str, list[str]]:
    """Every `AcquisitionError.kind` the server can emit -> where it is produced."""
    found: dict[str, list[str]] = {}

    def add(code: str, where: str) -> None:
        found.setdefault(code, [])
        if where not in found[code]:
            found[code].append(where)

    for path in rust_sources(EMITTING_CRATES):
        rel = str(path.relative_to(ROOT))
        lines = path.read_text().split("\n")
        for i, line in enumerate(lines):
            if "AcquisitionError {" not in line:
                continue
            k = next((j for j in range(i, min(i + 120, len(lines)))
                      if _KIND_FIELD.match(lines[j])), None)
            if k is None:
                continue
            m = next((j for j in range(k, min(k + 200, len(lines)))
                      if _MESSAGE_FIELD.match(lines[j])), None)
            if m is None:
                continue
            direct = _literals(lines[k:m])
            if direct:
                for code in direct:
                    add(code, rel)
                continue
            # An INDIRECT expression: a local `kind` binding, or `error.kind()`.
            binding = next((j for j in range(k, -1, -1) if _LET_KIND.match(lines[j])), None)
            if binding is not None and binding > k - 60:
                for code in _literals(_brace_body(lines, binding)):
                    add(code, rel)

    # `error.kind()` resolves to the fetcher's own table.
    fpath = ROOT / _FETCH_KIND[0]
    if fpath.is_file():
        flines = fpath.read_text().split("\n")
        start = next((j for j, l in enumerate(flines) if _FETCH_KIND[1] in l), None)
        if start is not None:
            for code in _literals(_brace_body(flines, start)):
                add(code, _FETCH_KIND[0])
    return {k: sorted(v) for k, v in sorted(found.items())}


def acquisition_passthrough() -> list[str]:
    """The sites that forward a worker-chosen kind — why the set is not closed."""
    out = []
    for path in rust_sources(EMITTING_CRATES):
        for n, line in enumerate(path.read_text().split("\n"), start=1):
            if _PASSTHROUGH.search(line):
                out.append(f"{path.relative_to(ROOT)}:{n}")
    return sorted(out)


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
    acq = acquisition_kinds()
    return {
        "emitted": emit,
        "registry": reg,
        "documented": doc,
        "client_side": client_side(),
        "emitted_unregistered": sorted(set(emit) - set(reg)),
        "registered_unemitted": sorted(set(reg) - set(emit)),
        "emitted_undocumented": sorted(set(emit) - set(doc)),
        "acquisition_kinds": acq,
        "acquisition_passthrough": acquisition_passthrough(),
        "acquisition_undocumented": sorted(set(acq) - set(doc)),
        "acquisition_overlap_with_registry": sorted(set(acq) & set(reg)),
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
        # ── ACQUISITION-KIND-DOC, the same rule for the second field ──────
        # ⛔ Registered as its own arm rather than folded into the one above,
        # because the two vocabularies are disjoint and a reader of a failure
        # needs to know WHICH field drifted (`SIGNOFF-REPAIR.7.2.10`).
        missing = c["acquisition_undocumented"]
        if missing:
            print("ACQUISITION-KIND-DOC: an `AcquisitionError.kind` the server emits is "
                  "not in the book's table.", file=sys.stderr)
            for code in missing:
                print(f"    {code}  (produced in {', '.join(c['acquisition_kinds'][code])})",
                      file=sys.stderr)
            print(f"  Add it to {BOOK_PAGE.relative_to(ROOT)}. This is the SECOND refusal",
                  file=sys.stderr)
            print("  vocabulary — `kind`, not `code` — and a client branches on it too.",
                  file=sys.stderr)
            return 1
        print(f"REASON-CODE-DOC: all {len(c['emitted'])} emitted codes are documented.")
        print(f"ACQUISITION-KIND-DOC: all {len(c['acquisition_kinds'])} acquisition kinds "
              f"are documented.")
        return 0

    print(f"§9.8 registry (KnownReasonCode)        : {len(c['registry'])}")
    print(f"codes the SERVER emits                 : {len(c['emitted'])}")
    print(f"  … not in the registry                : {len(c['emitted_unregistered'])}")
    print(f"  … not documented in the book         : {len(c['emitted_undocumented'])}")
    print(f"registry codes never emitted           : {len(c['registered_unemitted'])}")
    print(f"client-side sentinels (NOT wire codes) : {len(c['client_side'])}")
    print()
    print(f"acquisition-error `kind` vocabulary    : {len(c['acquisition_kinds'])}")
    print(f"  … not documented in the book         : {len(c['acquisition_undocumented'])}")
    print(f"  … shared with the §9.8 `code` registry: {len(c['acquisition_overlap_with_registry'])}"
          "   (a DIFFERENT field; disjoint by measurement)")
    print(f"  sites forwarding a worker-chosen kind : {len(c['acquisition_passthrough'])}"
          "   (the set is OPEN)")
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
    ran = 0

    # ⛔ THE TOTAL IS COUNTED, NOT WRITTEN DOWN. This printed a hardcoded `14`,
    # the hazard `TOOLBOX.md` names ("an instrument's own banner is prose too")
    # and the THIRD instance of it repaired in this session — after
    # `census_memory_warnings.py` (`.11.20.1`) and `census_broken_tables.py`
    # (`.11.18.2`), where the literal had already drifted from 21 to 23.
    def check(name, got, want):
        nonlocal ran
        ran += 1
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

    # ---- `SIGNOFF-REPAIR.7.2.10`: the SECOND vocabulary --------------------
    acq = acquisition_kinds()
    # ⭐ THE TWO KEY DEFECTS THIS DERIVATION HAD, pinned so neither returns.
    # 1. A marker of `kind: match &error {` missed the `GitError` site, which
    #    is written WITHOUT the `&` — sixteen kinds invisible.
    check("git-site-is-reached", "resolved_commit_missing" in acq, True)
    check("git-site-is-reached-2", "ref_selector_invalid" in acq, True)
    # 2. A `kind:\s*"…"` regex also matched `derived_kind:` and `actor_kind:`,
    #    sweeping in four strings from unrelated structs.
    for stray in ("chunk", "database", "approval", "decision"):
        check(f"no-stray-{stray}", stray in acq, False)
    # 3. A fixed-length window overran `FetchError::kind` into the next item
    #    and swept up the `snake_case` of a `#[serde(rename_all = …)]`.
    check("no-serde-attribute-leak", "snake_case" in acq, False)
    # ⭐ The two vocabularies are DISJOINT, which is the finding that makes this
    #    a second gate rather than a widening of the first.
    # ⚠️ LABELLED: this is a WATCH arm over the corpus, not a control over this
    #    code — no in-situ neutralization of the derivation fires it. Its red is
    #    a future commit giving an acquisition kind a name the §9.8 registry
    #    already uses, which is the day the two-gate design needs re-arguing.
    check("vocabularies-are-disjoint", sorted(set(acq) & set(reg)), [])
    # ⚠️ NEGATIVE: the derivation must not simply return every snake_case
    #    literal in the server, or "disjoint" is luck and the gate is noise.
    check("not-every-literal", "quota_unconfigured" in acq, False)
    # Every kind names a producing file, so a kind can always be traced.
    check("every-kind-has-a-site", [k for k, v in acq.items() if not v], [])
    # ⭐ The set is OPEN, and the instrument says so rather than implying closure.
    # ⚠️ LABELLED, with `every-kind-has-a-site` and `not-every-literal`: all
    #    three are shape guarantees the derivation makes by construction, so
    #    they pass under every wrong KEY tried above. Their reds are a removed
    #    pass-through detector, a producer list that stops recording sites, and
    #    a derivation widened to every snake_case literal in the server.
    check("passthrough-sites-found", len(acquisition_passthrough()) > 0, True)
    # ⭐ LIVE-CORPUS ARM: every control above reads the real tree, but this one
    #    asserts the population is non-empty — the census going blind to its
    #    producers would otherwise report a tidy, meaningless zero
    #    (`docs/knowledge/the-commit-that-reshapes-a-file-blinds-its-guard.md`).
    check("live-corpus-non-empty", len(acq) > 0, True)

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print(f"census_reason_codes --self-test: {ran} controls pass")
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
