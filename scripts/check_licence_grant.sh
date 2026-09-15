#!/usr/bin/env bash
# scripts/check_licence_grant.sh — LICENCE-GRANT doctrine.
#
# A licence EXPRESSION in a manifest is metadata. It is not a grant. Until the
# texts it names exist in the tree, a reader of this public repository has the
# rights ordinary copyright gives them, which is none of the ones the expression
# appears to offer. That was blocker B5: `MIT OR Apache-2.0` was declared in all
# 13 tracked manifests and no licence text existed anywhere in the repository.
#
# This gate makes the declaration and the texts unable to drift apart again, in
# BOTH directions:
#
#   1. every SPDX identifier any manifest declares has its text at the root;
#   2. every LICENSE-* file at the root is named by the declared expression — so
#      a file cannot linger after the expression changes and keep offering a
#      grant the project no longer makes;
#   3. every literal expression in the tree is the SAME expression — one crate
#      quietly declaring something narrower is a relicensing, not a typo;
#   4. each text is the licence it claims to be, by sentinel phrases rather than
#      by filename, and the MIT copyright line names a real holder.
#
# (2) and (4) are the halves that matter and the halves a hand-check skips. A
# check that only walked the manifests would pass a tree whose LICENSE-MIT had
# been emptied, truncated to its first paragraph, or replaced with a different
# licence under the same name.
#
# ⚠️ Sentinels SPAN each document — opening, numbered body, closing line — and
# that is not decoration. The first draft of this gate used four Apache
# sentinels that all sit in the first FIVE LINES, so `head -5 LICENSE-APACHE`
# passed it while granting nothing: the title block of a licence is not the
# licence. Falsifying the gate against the real tree is what exposed that; the
# self-test had not covered it, and the header comment above it claimed the
# opposite. Any sentinel added later belongs at a position not already covered.
#
# ⚠️ Sentinels are matched against WHITESPACE-NORMALISED text, and the reason is
# a defect this gate found in itself on its first real run: the MIT text crates
# ship is hard-wrapped at about 55 columns, so 'WITHOUT WARRANTY OF ANY KIND'
# spans a line break and a literal substring test failed on a PERFECTLY VALID
# licence. The self-test passed at that moment, because its fixtures were
# single-line prose — tidier than the real file. A fixture prettier than reality
# proves nothing about reality, so one deliberately hard-wrapped case is pinned
# in the cases below and must stay there.
#
# ⛔ What this gate deliberately does NOT do: judge the choice of licence, check
# dependency compatibility (the dependency ledger owns that), or fill the
# `[yyyy] [name of copyright owner]` placeholder in the Apache appendix. That
# placeholder is part of the canonical apache.org document; filling it would
# modify the licence text, and the check below asserts the text is UNMODIFIED.
#
# Whole-tree rather than staged-diff, so the CI backstop is real: a licence file
# deleted while nothing else was staged is still caught.
#
# Self-test: scripts/check_licence_grant.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

python3 -B - "${1:-}" <<'PY'
import subprocess, sys, pathlib, re

SELF_TEST = (sys.argv[1] if len(sys.argv) > 1 else "") == "--self-test"

# SPDX identifier -> the file that must carry its text, and the phrases that
# prove the file IS that licence. Sentinels are chosen to survive reflowing:
# each is a short run on a single line of the canonical text.
# `floor` is 90% of the canonical text's length, measured from the copies shipped
# by crates in this workspace's own dependency graph: MIT 1,055 chars / 25 lines,
# Apache-2.0 10,847 chars / 201 lines. It is a backstop for a truncation that
# happens to keep every sentinel, not the primary test.
SPDX = {
    "MIT": ("LICENSE-MIT", 950, [
        "Permission is hereby granted, free of charge",          # opening grant
        "without restriction, including without limitation",     # scope
        "The above copyright notice and this permission notice",  # condition
        'THE SOFTWARE IS PROVIDED "AS IS"',                      # disclaimer
        "WITHOUT WARRANTY OF ANY KIND",
        "DEALINGS IN THE SOFTWARE",                              # final line
    ]),
    "Apache-2.0": ("LICENSE-APACHE", 9700, [
        "Apache License",                                        # title
        "Version 2.0, January 2004",
        "http://www.apache.org/licenses/",
        "TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION",
        "1. Definitions.",                                       # body start
        "4. Redistribution.",                                    # body middle
        "7. Disclaimer of Warranty.",
        "8. Limitation of Liability.",
        "9. Accepting Warranty or Additional Liability.",        # body end
        "END OF TERMS AND CONDITIONS",
        "APPENDIX: How to apply the Apache License to your work.",
        "distributed under the License is distributed on an",    # final paragraph
    ]),
}
OPERATORS = {"OR", "AND", "WITH"}


def flat(text):
    """Collapse every whitespace run to one space, so a sentinel matches
    regardless of how the canonical text happens to be wrapped."""
    return " ".join(text.split())


def identifiers(expr):
    """The SPDX identifiers in an expression, ignoring operators and grouping."""
    return [tok for tok in re.split(r"[\s()]+|/", expr.strip())
            if tok and tok.upper() not in OPERATORS]


def audit(expressions, files):
    """expressions: {manifest path: literal expression}. files: {name: text}.

    Returns a list of breach strings. Pure: every input is passed in, so the
    self-test exercises the same function the real run does.
    """
    breaches = []
    if not expressions:
        return ["no manifest declares a licence expression at all"]

    distinct = sorted(set(expressions.values()))
    if len(distinct) > 1:
        where = "; ".join(f"{p} says {e!r}" for p, e in sorted(expressions.items()))
        breaches.append(
            f"manifests disagree on the licence expression ({len(distinct)} distinct): {where}")

    declared = set()
    for path, expr in sorted(expressions.items()):
        for ident in identifiers(expr):
            if ident not in SPDX:
                breaches.append(
                    f"{path} declares {ident!r}, which this gate cannot place — "
                    f"add it to SPDX in scripts/check_licence_grant.sh with its sentinels")
                continue
            declared.add(ident)

    # (1) declared -> text present, and (4) the text is that licence.
    for ident in sorted(declared):
        name, floor, sentinels = SPDX[ident]
        text = files.get(name)
        if text is None:
            breaches.append(
                f"{ident} is declared but {name} does not exist — the expression "
                f"is metadata, not a grant")
            continue
        if len(text) < floor:
            breaches.append(
                f"{name} is {len(text)} characters; the canonical {ident} text is "
                f"far longer (floor {floor}) — a truncated licence grants only "
                f"what it still says")
        flattened = flat(text)
        for phrase in sentinels:
            if flat(phrase) not in flattened:
                breaches.append(
                    f"{name} does not read like {ident}: the canonical text "
                    f"contains {phrase!r} and this file does not")
        if ident == "MIT":
            m = re.search(r"^Copyright \(c\) (.+)$", text, re.M)
            if not m:
                breaches.append(
                    f"{name} carries no 'Copyright (c) <year> <holder>' line — "
                    f"MIT grants nothing without one")
            elif not re.search(r"\d{4}", m.group(1)) or re.search(
                    r"\[|\]|<|>|yyyy|name of copyright owner|TODO|FIXME|XXX",
                    m.group(1), re.I):
                breaches.append(
                    f"{name}'s copyright line is still a placeholder: {m.group(0)!r}")

    # (2) text present -> declared. A file nobody declares still reads as an offer.
    expected = {SPDX[i][0] for i in declared}
    for name in sorted(files):
        if name not in expected:
            breaches.append(
                f"{name} exists at the root but no manifest declares its licence — "
                f"either declare it or remove the file; it reads as a grant either way")
    return breaches


if SELF_TEST:
    # Fixtures are the REAL files, so the self-test cannot drift into testing a
    # tidier document than the one shipped. Every mutation below is applied to
    # them. ⛔ If a licence file is missing, the self-test says so rather than
    # inventing a substitute and passing.
    real = {}
    for _n in ("LICENSE-MIT", "LICENSE-APACHE"):
        _p = pathlib.Path(_n)
        if not _p.is_file():
            print(f"SELF-TEST: {_n} is absent, so the fixtures cannot be built",
                  file=sys.stderr)
            sys.exit(1)
        real[_n] = _p.read_text(encoding="utf-8", errors="replace")
    MIT_OK, APACHE_OK = real["LICENSE-MIT"], real["LICENSE-APACHE"]
    BOTH = {"LICENSE-MIT": MIT_OK, "LICENSE-APACHE": APACHE_OK}
    DUAL = {"Cargo.toml": "MIT OR Apache-2.0"}

    def trimmed(drop):
        return {k: v for k, v in BOTH.items() if k != drop}

    # The real MIT file is hard-wrapped; re-flowing it to one line per paragraph
    # must NOT change the verdict. This is the wrap-insensitivity case.
    MIT_REFLOWED = "\n\n".join(
        " ".join(para.split()) for para in MIT_OK.split("\n\n"))

    def head_lines(text, n):
        return "".join(text.splitlines(keepends=True)[:n])

    cases = [
        ("healthy dual-licensed tree", DUAL, BOTH, False),
        # ⛔ PINNED: the real LICENSE-MIT is hard-wrapped and every sentinel
        # spans a line break. Dropping this case restores the false failure
        # that this gate produced against a valid licence on its first run.
        ("the real MIT text re-flowed — wrapping must not change the verdict",
         DUAL, {**BOTH, "LICENSE-MIT": MIT_REFLOWED}, False),
        # ⛔ PINNED: every Apache sentinel of the first draft lived in these five
        # lines, so this stub PASSED. Deleting this case restores that hole.
        ("Apache text truncated to its five-line title block",
         DUAL, {**BOTH, "LICENSE-APACHE": head_lines(APACHE_OK, 5)}, True),
        ("Apache text truncated just before END OF TERMS AND CONDITIONS",
         DUAL, {**BOTH, "LICENSE-APACHE": head_lines(APACHE_OK, 170)}, True),
        ("MIT text truncated to its title and copyright line",
         DUAL, {**BOTH, "LICENSE-MIT": head_lines(MIT_OK, 3)}, True),
        ("expression declared, no text at all", DUAL, {}, True),
        ("MIT declared, MIT text missing", DUAL, trimmed("LICENSE-MIT"), True),
        ("Apache declared, Apache text missing", DUAL, trimmed("LICENSE-APACHE"), True),
        ("file present that nothing declares",
         {"Cargo.toml": "MIT"}, BOTH, True),
        ("MIT text cut at its opening grant",
         DUAL, {**BOTH, "LICENSE-MIT": MIT_OK.split("Permission")[0]}, True),
        ("LICENSE-MIT actually holds the Apache text",
         DUAL, {**BOTH, "LICENSE-MIT": APACHE_OK}, True),
        ("copyright line left as a placeholder", DUAL,
         {**BOTH, "LICENSE-MIT": MIT_OK.replace(
             "2026 Richard DJE", "[yyyy] [name of copyright owner]")}, True),
        ("copyright line missing entirely", DUAL,
         {**BOTH, "LICENSE-MIT": MIT_OK.replace(
             "Copyright (c) 2026 Richard DJE\n", "")}, True),
        ("one crate declares a narrower expression",
         {"Cargo.toml": "MIT OR Apache-2.0",
          "crates/x/Cargo.toml": "Apache-2.0"}, BOTH, True),
        ("an identifier the gate cannot place",
         {"Cargo.toml": "MIT OR Apache-2.0 OR WTFPL"}, BOTH, True),
        ("no manifest declares anything", {}, BOTH, True),
    ]
    fails = 0
    for label, exprs, files, want_breach in cases:
        got = bool(audit(exprs, files))
        if got != want_breach:
            print(f"SELF-TEST: {label!r} -> breach={got}, expected {want_breach}",
                  file=sys.stderr)
            fails += 1
    # The identifier splitter must survive the legacy slash form and grouping.
    for expr, want in [("MIT OR Apache-2.0", ["MIT", "Apache-2.0"]),
                       ("MIT/Apache-2.0", ["MIT", "Apache-2.0"]),
                       ("(MIT OR Apache-2.0)", ["MIT", "Apache-2.0"]),
                       ("MIT AND Apache-2.0", ["MIT", "Apache-2.0"]),
                       ("MIT", ["MIT"])]:
        if identifiers(expr) != want:
            print(f"SELF-TEST: identifiers({expr!r}) -> {identifiers(expr)}, "
                  f"expected {want}", file=sys.stderr)
            fails += 1
    if fails:
        print(f"LICENCE-GRANT: --self-test FAILED ({fails})", file=sys.stderr)
        sys.exit(1)
    print(f"LICENCE-GRANT: --self-test ok ({len(cases)} audit cases, "
          f"5 expression forms)")
    sys.exit(0)

# --- the real run ---
manifests = subprocess.run(
    ["git", "ls-files", "-z", "*Cargo.toml"],
    capture_output=True, text=True, check=True).stdout.split("\0")
manifests = [m for m in manifests if m]

expressions, inherited, undeclared = {}, [], []
for m in manifests:
    text = pathlib.Path(m).read_text(encoding="utf-8", errors="replace")
    lit = re.search(r'^license\s*=\s*"([^"]+)"', text, re.M)
    if lit:
        expressions[m] = lit.group(1)
    elif re.search(r"^license\.workspace\s*=\s*true", text, re.M):
        inherited.append(m)
    elif re.search(r"^\[package\]", text, re.M):
        undeclared.append(m)

files = {p.name: p.read_text(encoding="utf-8", errors="replace")
         for p in sorted(pathlib.Path(".").glob("LICENSE*")) if p.is_file()}

breaches = audit(expressions, files)
for m in undeclared:
    breaches.append(f"{m} has a [package] section and declares no licence at all")

if breaches:
    print("LICENCE-GRANT: the declaration and the granted texts disagree", file=sys.stderr)
    for b in breaches:
        print(f"  - {b}", file=sys.stderr)
    sys.exit(1)

print(f"LICENCE-GRANT: OK — {len(manifests)} manifests "
      f"({len(expressions)} literal, {len(inherited)} inherited) declare "
      f"{sorted(set(expressions.values()))[0]!r}; "
      f"{len(files)} licence text(s) present and matching: {', '.join(sorted(files))}")
PY
