#!/usr/bin/env python3
"""Adjudicate every shipped surface against the book, one row per member
(`SIGNOFF-REPAIR.11.4.6.2`).

⭐ WHY THIS EXISTS. `.11.4.6` owes a classification: for each shipped surface,
whether it is (a) covered by a chapter, (b) deliberately internal, or (c) a
genuine gap. ⛔ Only (a) is decidable by machine, and even then only as *some
chapter mentions it*; (b) and (c) are judgements. `docs/knowledge/
a-sample-is-not-a-traversal.md` is explicit about where a per-member judgement
may live:

    If the argument genuinely requires a statement about every member, prose
    will not hold it. Carry the judgement as a table keyed to each member, and
    guard it so that it refuses when a member exists that nothing judged; a
    judgement exists for a member that is gone; a member's inputs have changed
    since it was judged.

The third is the one usually missing, and it is the one that makes a stale
judgement look current. All three are enforced below.

🔴 THE COARSE SIGNAL OVERSTATES COVERAGE, WHICH IS WHY THE VERDICT IS NOT READ
OFF IT. `census_book_coverage.py` asks *does any chapter mention this family*,
and 21 of 30 families pass. Asked per ROUTE — does the book name each of the
family's routes beside a method, which is `census_route_documentation.py`'s own
`described` test — only 14 of 30 have every route documented. Seven families the
coarse signal calls covered carry undocumented routes, `calls` worst at 3 of 4.
⭐ And `SIGNOFF-REPAIR.11.8.1` had already re-derived BY HAND that the collection
`POST /v1/calls` is undocumented, the chapter's line being a contract for the
ITEM route. A family-level mention test structurally cannot see that.

⭐ AND IT UNDER-REPORTS IN THE OTHER DIRECTION TOO, which is the better argument
for a table than any overclaim. `/` is mechanically a bare `mentioned`, while
`docs/book/src/web-ui.md` is an entire chapter about the console served there.
The machine is wrong both ways; the adjudication is what carries the answer.

⛔ THE POPULATION IS DERIVED FROM THE PRODUCERS, on three counts, and the third
is a correction to the population `.11.4.6.1` pinned:

  - **Route families** — every `/v1/<family>` registered outside a `#[cfg(test)]`
    block, over every file under the server's `src/` that calls axum's `.route(`.
  - **Binaries** — `cargo metadata --no-deps`. Only four of the ten carry an
    explicit `[[bin]]` stanza, so a manifest grep sees 4 of 10.
  - **Unfamilied routes** — ⛔ 30 families is every family UNDER `/v1/`, and three
    product routes belong to no `/v1/` family at all: `/`, `/app.js`,
    `/style.css`. A population scoped to `/v1/` misses them exactly as a
    population scoped to route families would have missed all ten binaries —
    `.11.4.6.1`'s own finding, one lane over. They are a third member kind here,
    so every product route is covered by something.

⛔ THE ROUTE CLASSIFIER IS IMPORTED, NEVER RE-IMPLEMENTED. A second copy of
`route_key`/`classify` would drift from the original, which is the exact failure
this table exists to prevent — and that logic has been wrong in both directions
twice already (`.11.8.1`'s key, `.11.8.2`'s separator). One producer, one reader.

⛔ THE WITNESS IS THE EVIDENCE THE JUDGEMENT WAS MADE FROM, and the guard
re-checks that evidence rather than the verdict. Per `docs/knowledge/
an-adjudication-is-keyed-to-the-words-it-judged.md`: put the judged thing in the
key, and refuse when it detaches. Two member kinds carry two evidence kinds, and
forcing one shape would weaken one of them:

  - a ROUTE-FAMILY or ROUTE is judged from a mechanical fact — how many routes it
    has and which of them the book does not name beside a method — so its witness
    is `<total>:<undescribed route keys>` and the guard RECOMPUTES it. A route
    added, removed, documented or undocumented since the judgement detaches it.
  - a BINARY is judged from a sentence somebody read, because there is no `/v1/`
    path to match and `rb` is two letters (the reach limit
    `census_book_coverage.py` states in its own report). Its witness is
    `<chapter>::<verbatim substring>` and the guard asserts the substring is
    still there verbatim. A rewritten sentence detaches it.

⭐ THE REFUSAL IS THE FEATURE. A detached witness is not a bug in the census; it
is the census reporting that a judgement is owed. The message prints the CURRENT
computed witness, so the author can see what changed — but it will not rewrite
the row, because silently re-attaching converts a stale verdict into a
fresh-looking one.

⛔ AND A GAP MUST NAME ITS OWNER. `CLAUDE.md` §15: raising a finding without
owning it is a complaint. A `gap` row whose owner column is empty is refused, so
the table cannot become a backlog nobody holds.

⚠️ WHAT THIS DOES NOT CLAIM. Nothing here says a judgement is CORRECT — only that
it is still about the facts it was made from. And `covered` means every route
carries a contract line, which is a proxy: a line beside a method is not proof
the description is accurate or complete (`.11.8`'s standing limit, unchanged).

Usage:
    python3 -B scripts/census_surface_judgement.py
    python3 -B scripts/census_surface_judgement.py --check
    python3 -B scripts/census_surface_judgement.py --json
    python3 -B scripts/census_surface_judgement.py --self-test
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVER_SRC = "crates/reasonbraid-server/src"
BOOK_SRC = ROOT / "docs/book/src"
LEDGER = ROOT / ".doctrine/book_surface_verdicts.tsv"

KINDS = ("route-family", "route", "binary")
VERDICTS = ("covered", "internal", "gap")
COLUMNS = ("kind", "member", "verdict", "witness", "owner", "reason")
NONE = "-"


def _route_census():
    """`census_route_documentation.py`, imported rather than copied.

    ⛔ Its `route_key`, `classify` and `product_routes` are the single producer
    of what "the book names this route" means. Re-implementing them here would
    put a second copy of a rule that has already been wrong twice (`.11.8.1`'s
    key deleted path parameters; `.11.8.2`'s separator demanded one space) into
    the instrument whose whole purpose is to keep a judgement attached to the
    facts. One producer, one reader.
    """
    path = ROOT / "scripts/census_route_documentation.py"
    spec = importlib.util.spec_from_file_location("census_route_documentation", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True,
                          text=True).stdout


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


FAMILY = re.compile(r"^/v1/([a-z0-9][a-z0-9-]*)")


def route_witnesses() -> dict[tuple[str, str], str]:
    """The mechanical witness for every route family and unfamilied route.

    `<total>:<undescribed route keys, comma-joined>`, or `<total>:-` when the
    book names every one of them beside a method. UNDESCRIBED covers both
    `absent` and `mentioned`: a path appearing without a method is not a
    contract line, and collapsing the two is how the first route census went
    wrong in both directions at once (`.11.8`).
    """
    rd = _route_census()
    routes: list[str] = []
    for relative in git("ls-files", "--", f"{SERVER_SRC}/*.rs").split():
        text = (ROOT / relative).read_text(encoding="utf-8", errors="replace")
        routes.extend(rd.product_routes(text))
    routes = sorted(set(routes))
    book = "\n".join(p.read_text(encoding="utf-8", errors="replace")
                     for p in sorted(BOOK_SRC.glob("*.md")))
    described = set(rd.classify(routes, book)["described"])

    grouped: dict[tuple[str, str], list[str]] = {}
    for route in routes:
        match = FAMILY.match(route)
        key = ("route-family", match.group(1)) if match else ("route", route)
        grouped.setdefault(key, []).append(route)

    witnesses: dict[tuple[str, str], str] = {}
    for key, members in grouped.items():
        undescribed = sorted(rd.route_key(r) for r in members if r not in described)
        witnesses[key] = f"{len(members)}:{','.join(undescribed) or NONE}"
    return witnesses


def population() -> dict[tuple[str, str], str | None]:
    """Every member, with its mechanical witness where one exists.

    A binary has no mechanical witness — there is no `/v1/` path to match and
    `rb` is two letters — so its value is None and its evidence is the quoted
    sentence in the ledger.
    """
    members: dict[tuple[str, str], str | None] = dict(route_witnesses())
    for name in binaries():
        members[("binary", name)] = None
    return members


def load_ledger(path: Path | None = None) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    text = (path or LEDGER).read_text(encoding="utf-8")
    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split("\t")
        row = dict(zip(COLUMNS, fields))
        row["line"] = str(number)
        row["arity"] = str(len(fields))
        rows.append(row)
    return rows


def quote_holds(row: dict[str, str], book_src: Path | None = None) -> str | None:
    """None when the binary's quoted evidence is still verbatim; else why not."""
    witness = row.get("witness", "")
    if "::" not in witness:
        return "witness is not `<chapter>::<verbatim quote>`"
    chapter, quote = witness.split("::", 1)
    path = (book_src or BOOK_SRC) / chapter
    if not path.is_file():
        return f"the chapter it cites is gone: {chapter}"
    if quote not in path.read_text(encoding="utf-8", errors="replace"):
        return f"the quoted sentence is no longer in {chapter} verbatim"
    return None


def adjudicate(members: dict[tuple[str, str], str | None],
               rows: list[dict[str, str]],
               book_src: Path | None = None) -> list[str]:
    """Every refusal, by name. Empty means the table is current."""
    refusals: list[str] = []

    seen: dict[tuple[str, str], dict[str, str]] = {}
    for row in rows:
        if row["arity"] != str(len(COLUMNS)):
            refusals.append(
                f"line {row['line']}: {row['arity']} fields, expected {len(COLUMNS)}"
                f" ({', '.join(COLUMNS)})")
            continue
        key = (row["kind"], row["member"])
        if row["kind"] not in KINDS:
            refusals.append(f"line {row['line']}: kind `{row['kind']}` is not one of "
                            f"{', '.join(KINDS)}")
            continue
        if row["verdict"] not in VERDICTS:
            refusals.append(f"line {row['line']}: {row['member']} carries verdict "
                            f"`{row['verdict']}`, not one of {', '.join(VERDICTS)}")
            continue
        if key in seen:
            refusals.append(f"line {row['line']}: {row['kind']} {row['member']} is "
                            f"judged twice (also line {seen[key]['line']})")
            continue
        seen[key] = row

        # ── a judgement for a member that is GONE ──
        if key not in members:
            refusals.append(
                f"line {row['line']}: {row['kind']} `{row['member']}` no longer "
                f"exists — the judgement outlived its subject")
            continue

        # ── a gap with nobody holding it is a complaint (CLAUDE.md §15) ──
        if row["verdict"] == "gap" and row["owner"] in ("", NONE):
            refusals.append(f"line {row['line']}: {row['member']} is a `gap` naming no "
                            f"owning leaf — a finding nobody owns is a complaint")
        if row["verdict"] in ("gap", "internal") and not row["reason"].strip():
            refusals.append(f"line {row['line']}: {row['member']} is `{row['verdict']}` "
                            f"with no reason")

        # ── the member's INPUTS have changed since it was judged ──
        expected = members[key]
        if expected is None:
            detached = quote_holds(row, book_src)
            if detached:
                refusals.append(f"line {row['line']}: {row['member']} — {detached}")
        elif row["witness"] != expected:
            refusals.append(
                f"line {row['line']}: {row['kind']} `{row['member']}` was judged "
                f"against `{row['witness']}` and is now `{expected}` — re-read it, "
                f"then record the new witness")

    # ── a member that NOTHING judged ──
    for kind, member in sorted(members):
        if (kind, member) not in seen:
            refusals.append(f"{kind} `{member}` is judged by nothing — "
                            f"current witness `{members[(kind, member)] or 'quote owed'}`")
    return refusals


def report(as_json: bool) -> int:
    members = population()
    rows = load_ledger()
    refusals = adjudicate(members, rows)
    by_verdict: dict[str, list[dict[str, str]]] = {v: [] for v in VERDICTS}
    for row in rows:
        if row.get("verdict") in by_verdict:
            by_verdict[row["verdict"]].append(row)

    if as_json:
        print(json.dumps({
            "revision": {"commit": git("rev-parse", "--short", "HEAD").strip(),
                         "dirty": bool(git("status", "--porcelain").strip())},
            "members": {f"{k}:{m}": w for (k, m), w in sorted(members.items())},
            "rows": [{c: r.get(c, "") for c in COLUMNS} for r in rows],
            "refusals": refusals,
        }, indent=2))
        return 1 if refusals else 0

    where = git("rev-parse", "--short", "HEAD").strip()
    state = " (tree DIRTY)" if git("status", "--porcelain").strip() else " (tree clean)"
    print(f"surface judgement at {where}{state}")
    for kind in KINDS:
        total = sum(1 for k, _ in members if k == kind)
        print(f"  {kind:<14} members: {total}")
    print()
    for verdict in VERDICTS:
        names = sorted(f"{r['member']}" for r in by_verdict[verdict])
        print(f"  {verdict:<9} {len(names):>3}  {', '.join(names) if names else '—'}")

    owed = sorted({r["owner"] for r in by_verdict["gap"] if r["owner"] not in ("", NONE)})
    print(f"\n  the gaps are held by {len(owed)} leaf/leaves: {', '.join(owed) or '—'}")
    print("\n  ⛔ `covered` means the book names EVERY route of the member beside an\n"
          "     HTTP method. That is a proxy, not proof the description is accurate\n"
          "     or complete — `SIGNOFF-REPAIR.11.8`'s standing limit, unchanged.")
    print("  ⛔ Nothing here says a judgement is CORRECT, only that it is still about\n"
          "     the facts it was made from. A detached witness is a judgement OWED.")

    if refusals:
        print(f"\n  🔴 {len(refusals)} refusal(s):")
        for refusal in refusals:
            print(f"     {refusal}")
        return 1
    print("\n  ✅ every member is judged, every judgement has a live member, and every\n"
          "     witness still holds")
    return 0


def check() -> int:
    refusals = adjudicate(population(), load_ledger())
    if not refusals:
        return 0
    print("SURFACE-JUDGEMENT: the book-surface adjudication is out of date "
          f"({LEDGER.relative_to(ROOT)})", file=sys.stderr)
    for refusal in refusals:
        print(f"  {refusal}", file=sys.stderr)
    print("  ⛔ Re-read the surface and record a fresh judgement. Do NOT copy the "
          "computed witness over the old row without re-reading — that converts a "
          "stale verdict into a fresh-looking one.", file=sys.stderr)
    return 1


def self_test() -> int:
    failures: list[str] = []
    base = {("route-family", "threads"): "2:-",
            ("route-family", "calls"): "4:/v1/calls,/v1/calls/{}/close",
            ("binary", "rb"): None}

    def row(kind, member, verdict, witness, owner=NONE, reason=""):
        return {"kind": kind, "member": member, "verdict": verdict,
                "witness": witness, "owner": owner, "reason": reason,
                "line": "1", "arity": str(len(COLUMNS))}

    import tempfile
    with tempfile.TemporaryDirectory(dir=ROOT / "target") as tmp:
        book = Path(tmp)
        (book / "cli.md").write_text("the `rb` CLI drives the control plane\n",
                                     encoding="utf-8")
        good = [
            row("route-family", "threads", "covered", "2:-"),
            row("route-family", "calls", "gap", "4:/v1/calls,/v1/calls/{}/close",
                "SIGNOFF-REPAIR.11.4.6.7", "the collection and the close verb"),
            row("binary", "rb", "covered", "cli.md::the `rb` CLI drives the control plane"),
        ]
        got = adjudicate(base, good, book)
        if got:
            failures.append(f"a current table was refused: {got}")

        # ── a member that NOTHING judged ──
        got = adjudicate(base, good[:2], book)
        if not any("judged by nothing" in g for g in got):
            failures.append(f"an unjudged member was accepted: {got}")

        # ── a judgement for a member that is GONE ──
        got = adjudicate(base, good + [row("route-family", "ghost", "covered", "1:-")],
                         book)
        if not any("no longer exists" in g for g in got):
            failures.append(f"a judgement outliving its subject was accepted: {got}")

        # ── THE INPUTS MOVED: the mechanical witness is recomputed, not trusted.
        # This is the guard `a-sample-is-not-a-traversal` says is usually missing
        # and is what makes a stale judgement look current.
        stale = [good[0], row("route-family", "calls", "gap", "4:/v1/calls",
                              "SIGNOFF-REPAIR.11.4.6.7", "r"), good[2]]
        got = adjudicate(base, stale, book)
        if not any("re-read it" in g for g in got):
            failures.append(f"a family whose routes moved kept its verdict: {got}")

        # ── and the same guard for a QUOTED witness, both directions ──
        detached = [good[0], good[1],
                    row("binary", "rb", "covered", "cli.md::the `rb` CLI drives the plane")]
        got = adjudicate(base, detached, book)
        if not any("no longer in cli.md" in g for g in got):
            failures.append(f"a paraphrased quote kept its verdict: {got}")
        missing = [good[0], good[1], row("binary", "rb", "covered", "gone.md::anything")]
        got = adjudicate(base, missing, book)
        if not any("chapter it cites is gone" in g for g in got):
            failures.append(f"a quote citing a deleted chapter was accepted: {got}")

        # ── a gap nobody owns is a complaint (CLAUDE.md §15) ──
        unowned = [good[0], row("route-family", "calls", "gap",
                                "4:/v1/calls,/v1/calls/{}/close", NONE, "r"), good[2]]
        got = adjudicate(base, unowned, book)
        if not any("naming no owning leaf" in g for g in got):
            failures.append(f"an unowned gap was accepted: {got}")

        # ── an internal verdict with no reason ──
        silent = [good[0], row("route-family", "calls", "internal",
                               "4:/v1/calls,/v1/calls/{}/close", NONE, ""), good[2]]
        got = adjudicate(base, silent, book)
        if not any("with no reason" in g for g in got):
            failures.append(f"an unreasoned `internal` was accepted: {got}")

        # ── vocabulary and arity, so a typo is not a silent reclassification ──
        got = adjudicate(base, good[:2] + [row("binary", "rb", "documented", "x::y")], book)
        if not any("not one of" in g for g in got):
            failures.append(f"a verdict outside the vocabulary was accepted: {got}")
        short = dict(good[2]); short["arity"] = "4"
        got = adjudicate(base, good[:2] + [short], book)
        if not any("expected 6" in g for g in got):
            failures.append(f"a short row was accepted: {got}")

        # ── the real ledger's population is derived, never typed ──
        if not any(k == "route" for k, _ in base) and "route" not in KINDS:
            failures.append("the unfamilied-route kind is not in the vocabulary")

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "census_surface_judgement --self-test: a current table passes, and each of "
        "the four guards is proved to REFUSE — a member nothing judged, a judgement "
        "whose member is gone, a member whose INPUTS moved since it was judged "
        "(recomputed for a route family, re-quoted verbatim for a binary, with a "
        "paraphrase and a deleted chapter each caught), and a `gap` naming no owning "
        "leaf; plus an unreasoned `internal`, a verdict outside the vocabulary and a "
        "short row, so a typo cannot be a silent reclassification"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--check", action="store_true",
                        help="refuse (rc=1) when the adjudication is out of date")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.check:
        return check()
    return report(args.json)


if __name__ == "__main__":
    sys.exit(main())
