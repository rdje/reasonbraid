#!/usr/bin/env python3
"""Census the positional source references (`file.rs:123`) in tracked Markdown.

`SIGNOFF-REPAIR.11.17`. Prose in this repository cites source by position, and
`docs/CLAIM_VERIFICATION.md` §4.1 grades a NAMED INSTANCE as exact with no
tolerance band. A reference is only exact if a reader can resolve it, and a BARE
BASENAME resolves only when that basename names exactly one tracked file.

    python3 -B scripts/census_positional_refs.py            # the classified census
    python3 -B scripts/census_positional_refs.py --check    # gate: no AMBIGUOUS reference
    python3 -B scripts/census_positional_refs.py --json
    python3 -B scripts/census_positional_refs.py --self-test

⛔ THE PATH CHARACTER CLASS INCLUDES THE HYPHEN, and that is not a detail. The
first instrument used `[a-z_0-9/]`, so every `crates/reasonbraid-core/src/x.rs:1`
truncated at the hyphen to `core/src/x.rs:1` and was reported as naming a missing
file: 93 findings, 93 of them the instrument. Printing the DETAIL rather than the
count is what made that visible (`CLAIM_VERIFICATION.md` leg 2 — a search
returning N hits gives you a population, not a count of defects).

⛔ IT DOES NOT CHECK THAT THE LINE EXISTS, deliberately. The second instrument
did, and reported 9 references "past the end of the file" — every one of them a
bare `profiles.rs`/`policy.rs` that it had resolved to `src/` where the prose
meant `tests/`. ⭐ That false positive IS the finding: an instrument guessing the
same way a reader must is the evidence the reference is ambiguous. Line numbers
also drift with every insertion above them, so a line-existence check would be a
permanent flake; ambiguity is a property of the corpus, and that is what this
measures.

⭐ IT GOVERNS ALL TRACKED MARKDOWN, INCLUDING THE DATED LEDGERS, and that is a
decision rather than an oversight. The usual reason to exempt history — a record
must not be rewritten — does not apply, because adding a repo-root-relative path
does not change what the entry SAYS. It says the same thing more precisely, and
the reader who cannot resolve `profiles.rs:5696` is equally stuck whichever
document they found it in. ⚠️ The one edge it accepts: an entry written when only
one `profiles.rs` existed is being disambiguated with today's knowledge. That is
still the right answer for a reader, and the alternative is an exemption list that
has to be maintained and argued about.

⛔ IT GATES AMBIGUITY, NOT DRIFT, and the two are different problems. A line
number was exact at the commit that wrote it and moves with every insertion above
it; several references discharged by `SIGNOFF-REPAIR.11.17` had already drifted
(`mcp_write.rs:149` is now `:169`, `allowlist.rs:150` is now `:183`). Drift cannot
be gated without re-verifying every line on every commit, which is a permanent
flake. WHICH FILE is fixable mechanically and stays fixed; which LINE is not.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The positional reference. The hyphen is IN the class (see the module docstring),
# and so is the dot, so a full `crates/reasonbraid-core/src/authority.rs:562`
# survives intact.
REF_RE = re.compile(r"(?<![A-Za-z0-9_./-])([A-Za-z_0-9./-]+\.(?:rs|sql|py|sh|toml)):(\d+)")

SOURCE_SUFFIXES = (".rs", ".sql", ".py", ".sh", ".toml")

def tracked(pattern: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", pattern], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return [p for p in out.stdout.splitlines() if p]


def basename_index(files: list[str]) -> dict[str, list[str]]:
    index: dict[str, list[str]] = defaultdict(list)
    for path in files:
        index[path.rsplit("/", 1)[-1]].append(path)
    return index


def classify(md_files: list[str], index: dict[str, list[str]], read) -> list[dict]:
    rows: list[dict] = []
    for md in md_files:
        for lineno, line in enumerate(read(md).splitlines(), start=1):
            for ref, pos in REF_RE.findall(line):
                base = ref.rsplit("/", 1)[-1]
                pathed = "/" in ref
                owners = index.get(base, [])
                if pathed:
                    kind = "pathed"
                elif len(owners) == 1:
                    kind = "unique"
                elif len(owners) == 0:
                    # Names no tracked file at all. Could be a file that was
                    # deleted, or an illustrative name in an example.
                    kind = "unresolved"
                else:
                    kind = "ambiguous"
                rows.append(
                    {
                        "md": md,
                        "md_line": lineno,
                        "ref": f"{ref}:{pos}",
                        "basename": base,
                        "kind": kind,
                        "owners": owners,
                    }
                )
    return rows


def collect(read=None, md_files=None, src_files=None) -> list[dict]:
    if read is None:
        def read(p: str) -> str:
            return (ROOT / p).read_text(encoding="utf-8", errors="replace")
    if md_files is None:
        md_files = tracked("*.md")
    if src_files is None:
        src_files = [p for p in tracked("*") if p.endswith(SOURCE_SUFFIXES)]
    return classify(md_files, basename_index(src_files), read)


def report(rows: list[dict]) -> None:
    distinct = {r["ref"] for r in rows}
    counts = Counter(r["kind"] for r in rows)
    print(f"positional source references in tracked Markdown: {len(rows)} occurrence(s), "
          f"{len(distinct)} distinct, across {len({r['md'] for r in rows})} file(s)")
    print("  " + ", ".join(f"{k}={counts.get(k, 0)}"
                           for k in ("pathed", "unique", "ambiguous", "unresolved")))
    amb = [r for r in rows if r["kind"] == "ambiguous"]
    if amb:
        print(f"  AMBIGUOUS: {len(amb)}")
        by_base = Counter(r["basename"] for r in amb)
        for base, n in by_base.most_common():
            print(f"    {base:<28} {n:>3}  ->  " + " | ".join(sorted(set(
                o for r in amb if r["basename"] == base for o in r["owners"]))))


def check(rows: list[dict]) -> int:
    amb = [r for r in rows if r["kind"] == "ambiguous"]
    if not amb:
        print(f"POSITIONAL-REF: OK — no ambiguous positional reference "
              f"({len(rows)} occurrence(s) classified)")
        return 0
    print("POSITIONAL-REF: tracked Markdown cites a source position by a BARE BASENAME that names "
          "more than one tracked file — a reader cannot resolve it, and `docs/CLAIM_VERIFICATION.md` "
          "§4.1 grades a NAMED INSTANCE as exact with no tolerance band.", file=sys.stderr)
    for r in sorted(amb, key=lambda r: (r["md"], r["md_line"])):
        print(f"  {r['md']}:{r['md_line']}  cites  {r['ref']}", file=sys.stderr)
        print(f"      names {len(r['owners'])} tracked files: " + ", ".join(r["owners"]), file=sys.stderr)
    print("\n  Write the reference repo-root-relative, e.g. "
          "`crates/reasonbraid-server/tests/profiles.rs:5696`.", file=sys.stderr)
    return 1


SELF_TEST_SRC = [
    "crates/reasonbraid-server/src/profiles.rs",
    "crates/reasonbraid-server/tests/profiles.rs",
    "crates/reasonbraid-core/src/authority.rs",
    "scripts/only_here.py",
]


def self_test() -> int:
    arms: list[tuple[str, bool]] = []
    index = basename_index(SELF_TEST_SRC)

    def run(text: str, md="docs/x.md"):
        return classify([md], index, lambda _p: text)

    # 1 a bare basename naming two tracked files is AMBIGUOUS
    rows = run("see `profiles.rs:5696` for the control")
    arms.append(("a bare basename naming 2 files is ambiguous",
                 [r["kind"] for r in rows] == ["ambiguous"]))
    # 2 the SAME reference written with a path is not
    rows = run("see `crates/reasonbraid-server/tests/profiles.rs:5696`")
    arms.append(("the same reference with a path is pathed",
                 [r["kind"] for r in rows] == ["pathed"]))
    # 3 ⛔ THE FOUNDING FALSE POSITIVE: a hyphen in the path must not truncate it
    rows = run("`crates/reasonbraid-core/src/authority.rs:562`")
    arms.append(("a hyphenated path survives the character class",
                 len(rows) == 1 and rows[0]["ref"] == "crates/reasonbraid-core/src/authority.rs:562"))
    # 4 a bare basename naming exactly one tracked file resolves
    rows = run("`only_here.py:12` is unambiguous")
    arms.append(("a unique basename resolves", [r["kind"] for r in rows] == ["unique"]))
    # 5 a basename naming NO tracked file is its own class, not an ambiguity
    rows = run("`deleted_thing.rs:9` names nothing")
    arms.append(("a basename naming no tracked file is 'unresolved'",
                 [r["kind"] for r in rows] == ["unresolved"]))
    # 6 ⭐ A DATED LEDGER IS GOVERNED TOO — adding a path does not rewrite a record, it
    #   states the same thing more precisely, so there is no history to protect.
    import contextlib, io
    def quiet(fn, *a):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return fn(*a)
    rows = run("`profiles.rs:1` in a ledger", md="CHANGELOG.md")
    arms.append(("a dated ledger is governed, not exempt", quiet(check, rows) == 1))
    # 7 and so is a live page, by the same rule rather than by a second one
    rows = run("`profiles.rs:1` in a live page", md="docs/book/src/deployment.md")
    arms.append(("a live document is governed by the same rule", quiet(check, rows) == 1))
    # 8 a bare word with no line number is not a positional reference
    rows = run("the file `profiles.rs` holds the control")
    arms.append(("a reference with no line number is not positional", rows == []))
    # 9 ⛔ a VERSION-like token must not parse as a position
    rows = run("pinned at serde 1.0.219 and Cargo.toml:14 is the entry")
    arms.append(("a dotted version is not a source position",
                 [r["ref"] for r in rows] == ["Cargo.toml:14"]))

    passed = sum(1 for _, ok in arms if ok)
    for name, ok in arms:
        print(f"census_positional_refs: {'arm ok' if ok else 'arm FAILED'} — {name}")
    print(f"census_positional_refs --self-test: {passed}/{len(arms)} controls pass")
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
