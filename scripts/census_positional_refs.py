#!/usr/bin/env python3
"""Census the positional source references (`file.rs:123`) in tracked Markdown.

`SIGNOFF-REPAIR.11.17`. Prose in this repository cites source by position, and
`docs/CLAIM_VERIFICATION.md` §4.1 grades a NAMED INSTANCE as exact with no
tolerance band. A reference is only exact if a reader can resolve it, and a BARE
BASENAME resolves only when that basename names exactly one tracked file.

    python3 -B scripts/census_positional_refs.py            # the classified census
    python3 -B scripts/census_positional_refs.py --check    # the gate
    python3 -B scripts/census_positional_refs.py --calibrate [N] [kind ...]
    python3 -B scripts/census_positional_refs.py --at <rev> [--json]
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

⭐ A REFERENCE CONTAINING A SLASH IS NOT AUTOMATICALLY RESOLVABLE
(`SIGNOFF-REPAIR.11.17.2`). Until that leaf, `kind = "pathed" if "/" in ref` — a
reference was called resolvable because it LOOKED like a path, and the tracked
file list was never consulted. Measured over the `pathed` class alone: 55 of 267
occurrences named no tracked file, and one of them was a suffix of THREE tracked
files — an ambiguous reference passed by the gate whose entire purpose is refusing
one, which is `BOOK-LINKS`' founding shape inside a gate that cites `BOOK-LINKS`.
A path is now resolved the same way a basename is, and against the same question:
how many tracked files can a reader reach from what is written?

⭐ A DEPENDENCY CITATION IS A CLASS, NOT AN EXEMPTION. `SIGNOFF-REPAIR.11.17.1`
established the crate-and-version qualified form — `gix-0.87.1/src/init.rs:76` —
as the right repair for a third-party citation, because it resolves for a reader
AND dates itself. `Cargo.lock` is the oracle: the segment is `<crate>-<version>`,
and the lock says whether the graph contains that crate at that version. ⛔ So the
class earns a name rather than an exclusion, and it earns a FAILURE MODE with it —
a citation into a version the project no longer depends on is stale, and before
this nothing said so.

⛔ IT DOES NOT CHECK THE DEPENDENCY'S IN-CRATE PATH, and the reason is the gate's
environment rather than the class. Verifying `src/init.rs` inside `gix-0.87.1`
means reading the vendored registry source, which is present only after a fetch —
so the check would pass here, fail on a cold clone, and be waived within a week
(`SIGNOFF-REPAIR.11.5`'s constraint). `Cargo.lock` is tracked and the crate and
version are not; that is the part this can hold for every reader.
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

# A vendored-crate directory: `<name>-<version>`. The split is unambiguous because
# a crate-name segment cannot contain a dot and the version must, so the boundary
# can only fall in one place.
CRATE_VERSION_RE = re.compile(
    r"^(?P<name>[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)*)-(?P<version>\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$"
)

# The kinds `--check` refuses. `partial` and `dependency` resolve for a reader and
# are NOT here; see `check` for why, and `--calibrate` for the price of each.
REFUSED = ("ambiguous", "unresolved", "dependency-stale")

ALL_KINDS = ("pathed", "unique", "partial", "dependency",
             "ambiguous", "unresolved", "dependency-stale")


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


def suffix_index(files: list[str]) -> dict[str, list[str]]:
    """Every PROPER path-suffix of every tracked file, on segment boundaries.

    ⛔ On segment boundaries, which is the whole point. `reasonbraid-core/src/x.rs`
    ends with the STRING `core/src/x.rs` and does not contain it as a path — that
    is the founding false positive (the hyphen truncation) arriving in the suffix
    matcher, and a plain `str.endswith` would resurrect it.
    """
    index: dict[str, list[str]] = defaultdict(list)
    for path in files:
        segments = path.split("/")
        for i in range(1, len(segments)):
            index["/".join(segments[i:])].append(path)
    return index


def lock_packages(text: str) -> dict[str, set[str]]:
    """Crate name -> the versions `Cargo.lock` pins. The oracle for a dependency citation."""
    packages: dict[str, set[str]] = defaultdict(set)
    name: str | None = None
    for line in text.splitlines():
        if line.startswith("name = "):
            name = line.split("=", 1)[1].strip().strip('"')
        elif line.startswith("version = ") and name is not None:
            packages[name].add(line.split("=", 1)[1].strip().strip('"'))
            name = None
    return dict(packages)


class Resolver:
    """Everything a reference is resolved AGAINST, in one injectable object."""

    def __init__(self, src_files: list[str], lock_text: str = "") -> None:
        self.tracked: set[str] = set(src_files)
        self.basenames = basename_index(src_files)
        self.suffixes = suffix_index(src_files)
        self.packages = lock_packages(lock_text)

    def resolve(self, ref: str) -> tuple[str, list[str], dict | None]:
        """-> (kind, owners, dependency). `owners` is what a reader could reach."""
        if "/" not in ref:
            owners = self.basenames.get(ref, [])
            if len(owners) == 1:
                return "unique", owners, None
            if len(owners) == 0:
                # Names no tracked file at all. Could be a file that was
                # deleted, or an illustrative name in an example.
                return "unresolved", [], None
            return "ambiguous", owners, None

        if ref in self.tracked:
            return "pathed", [ref], None

        segments = ref.split("/")
        for i, segment in enumerate(segments):
            matched = CRATE_VERSION_RE.match(segment)
            if not matched:
                continue
            name, version = matched.group("name"), matched.group("version")
            pinned = self.packages.get(name)
            if not pinned:
                continue
            dependency = {"crate": name, "version": version,
                          "path": "/".join(segments[i + 1:]),
                          "pinned": sorted(pinned)}
            if version in pinned:
                return "dependency", [], dependency
            return "dependency-stale", [], dependency

        owners = self.suffixes.get(ref, [])
        if len(owners) == 1:
            return "partial", owners, None
        if len(owners) > 1:
            return "ambiguous", owners, None
        return "unresolved", [], None


def classify(md_files: list[str], resolver: Resolver, read) -> list[dict]:
    rows: list[dict] = []
    for md in md_files:
        for lineno, line in enumerate(read(md).splitlines(), start=1):
            for ref, pos in REF_RE.findall(line):
                kind, owners, dependency = resolver.resolve(ref)
                rows.append(
                    {
                        "md": md,
                        "md_line": lineno,
                        "ref": f"{ref}:{pos}",
                        "path": ref,
                        "basename": ref.rsplit("/", 1)[-1],
                        "kind": kind,
                        "owners": owners,
                        "dependency": dependency,
                    }
                )
    return rows


def collect(read=None, md_files=None, src_files=None, lock_text=None) -> list[dict]:
    if read is None:
        def read(p: str) -> str:
            return (ROOT / p).read_text(encoding="utf-8", errors="replace")
    if md_files is None:
        md_files = tracked("*.md")
    if src_files is None:
        src_files = [p for p in tracked("*") if p.endswith(SOURCE_SUFFIXES)]
    if lock_text is None:
        lock = ROOT / "Cargo.lock"
        lock_text = lock.read_text(encoding="utf-8", errors="replace") if lock.exists() else ""
    return classify(md_files, Resolver(src_files, lock_text), read)


def collect_at(rev: str) -> list[dict]:
    """The same census, over the tree at `rev`.

    ⭐ `TOOLBOX.md`, *"pin a census that your own change will invalidate"*. A
    population this leaf reports and then repairs is unreproducible an hour
    later unless the reader can name the commit — and `SIGNOFF-REPAIR.11.17.2`
    needed exactly that to tell *the corpus grew* from *the number was wrong*
    about its own opening table.
    """
    blobs = Blobs()
    try:
        files = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", rev], cwd=ROOT,
            capture_output=True, text=True, check=True,
        ).stdout.split()
        resolver = Resolver([f for f in files if f.endswith(SOURCE_SUFFIXES)],
                            blobs.read(rev, "Cargo.lock"))
        return classify([f for f in files if f.endswith(".md")], resolver,
                        lambda p: blobs.read(rev, p))
    finally:
        blobs.close()


class Blobs:
    """One long-lived `git cat-file --batch` for a whole calibration run.

    ⛔ `git show` per blob is what made the first calibration unusable: the
    incremental design below needs ~1,700 reads over 200 commits, and 1,700
    process spawns is most of the wall clock. One process, framed replies.
    """

    def __init__(self) -> None:
        self.proc = subprocess.Popen(
            ["git", "cat-file", "--batch"], cwd=ROOT,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        )

    def read(self, sha: str, path: str) -> str:
        assert self.proc.stdin and self.proc.stdout
        self.proc.stdin.write(f"{sha}:{path}\n".encode())
        self.proc.stdin.flush()
        header = self.proc.stdout.readline().decode(errors="replace").split()
        if len(header) < 3:                      # "<name> missing"
            return ""
        size = int(header[2])
        body = self.proc.stdout.read(size)
        self.proc.stdout.read(1)                 # the trailing newline
        return body.decode("utf-8", errors="replace")

    def close(self) -> None:
        assert self.proc.stdin
        self.proc.stdin.close()
        self.proc.wait()


def calibrate(depth: int, kinds: tuple[str, ...] = REFUSED) -> dict:
    """What a gate on `kinds` would have blocked, commit by commit.

    `SIGNOFF-REPAIR.11.6`'s standing requirement, and `SIGNOFF-REPAIR.11.17`'s
    own method — a rule is priced against its population before it is proposed.

    ⛔ INCREMENTAL, AND THE SPLIT IS WHAT MAKES IT EXACT. Re-classifying every
    tracked Markdown file at every commit is 200 x ~350 blobs and does not finish
    in a usable time; the first cut was abandoned after nine minutes
    (`SIGNOFF-REPAIR.11.17.1`). The earlier incremental version bought its speed
    by approximating — it re-scanned only when a source BASENAME entered or left
    the tree, which is sound for a basename gate and wrong for a path one.

    ⭐ The separation that costs nothing: PARSING a reference needs the blob and
    changes only when the file changes; RESOLVING it needs the index and changes
    whenever the tree does. Cache the parse, redo the resolve. Re-classification
    is then free, so every commit is re-classified exactly — no approximation
    left to be wrong about — and the blob reads drop to the ~1,700 Markdown
    revisions that actually exist across 200 commits.
    """
    def run(*args: str) -> str:
        return subprocess.run(list(args), cwd=ROOT, capture_output=True, text=True).stdout

    shas = run("git", "rev-list", "--reverse", "-n", str(depth + 1), "HEAD").split()
    if len(shas) < 2:
        return {"commits_examined": 0, "commits_blocked": 0, "blocked_pct": 0.0,
                "kinds": list(kinds), "detail": []}

    blobs = Blobs()
    try:
        base = shas[0]
        files = set(run("git", "ls-tree", "-r", "--name-only", base).split())
        src = {f for f in files if f.endswith(SOURCE_SUFFIXES)}
        lock_text = blobs.read(base, "Cargo.lock")
        resolver = Resolver(sorted(src), lock_text)

        # md -> the parsed references in it, cached against the blob.
        parsed: dict[str, list[tuple[str, str]]] = {}
        for md in (f for f in files if f.endswith(".md")):
            found = REF_RE.findall(blobs.read(base, md))
            if found:
                parsed[md] = found

        def counts() -> dict[str, int]:
            out: dict[str, int] = {}
            for md, refs in parsed.items():
                n = sum(1 for ref, _ in refs if resolver.resolve(ref)[0] in kinds)
                if n:
                    out[md] = n
            return out

        state = counts()
        blocked, detail = 0, []
        for a, b in zip(shas, shas[1:]):
            reindex = False
            for row in run("git", "diff", "--name-status", a, b).splitlines():
                parts = row.split("\t")
                if len(parts) < 2:
                    continue
                status = parts[0]
                if status.startswith("R") and len(parts) >= 3:
                    changes = [("D", parts[1]), ("A", parts[2])]
                else:
                    changes = [(status[0], parts[-1])]
                for st, path in changes:
                    if path == "Cargo.lock":
                        reindex = True
                    if path.endswith(SOURCE_SUFFIXES):
                        if st == "D":
                            src.discard(path)
                            reindex = True
                        elif st == "A":
                            src.add(path)
                            reindex = True
                    elif path.endswith(".md"):
                        if st == "D":
                            parsed.pop(path, None)
                        else:
                            found = REF_RE.findall(blobs.read(b, path))
                            if found:
                                parsed[path] = found
                            else:
                                parsed.pop(path, None)
            if reindex:
                resolver = Resolver(sorted(src), blobs.read(b, "Cargo.lock"))

            prev, state = state, counts()
            gained = sorted(f for f, c in state.items() if c > prev.get(f, 0))
            if gained:
                blocked += 1
                detail.append({"commit": b[:7], "files": gained})
    finally:
        blobs.close()

    n = len(shas) - 1
    return {
        "commits_examined": n,
        "commits_blocked": blocked,
        "blocked_pct": round(100 * blocked / n, 1) if n else 0.0,
        "kinds": list(kinds),
        "detail": detail,
    }


def report(rows: list[dict]) -> None:
    distinct = {r["ref"] for r in rows}
    counts = Counter(r["kind"] for r in rows)
    print(f"positional source references in tracked Markdown: {len(rows)} occurrence(s), "
          f"{len(distinct)} distinct, across {len({r['md'] for r in rows})} file(s)")
    print("  " + ", ".join(f"{k}={counts.get(k, 0)}" for k in ALL_KINDS))
    for kind in REFUSED:
        bad = [r for r in rows if r["kind"] == kind]
        if not bad:
            continue
        print(f"  {kind.upper()}: {len(bad)}")
        by_ref = Counter(r["path"] for r in bad)
        for path, n in by_ref.most_common():
            owners = sorted({o for r in bad if r["path"] == path for o in r["owners"]})
            print(f"    {path:<48} {n:>3}  ->  " + (" | ".join(owners) if owners else "nothing"))


def check(rows: list[dict]) -> int:
    by_kind = {k: [r for r in rows if r["kind"] == k] for k in REFUSED}
    if not any(by_kind.values()):
        print(f"POSITIONAL-REF: OK — every positional reference resolves "
              f"({len(rows)} occurrence(s) classified)")
        return 0

    if by_kind["unresolved"]:
        # ⛔ The EXAMPLE-versus-CITATION problem, and how it is answered
        # (`SIGNOFF-REPAIR.11.17.1`): NOT by telling them apart, which no
        # instrument can do from a basename. An illustrative example simply may
        # not be WRITTEN in the positional form — this gate's own registry row
        # carried a placeholder and a line number, and was reworded rather than
        # excluded, the same way it reworded itself twice before.
        print("POSITIONAL-REF: tracked Markdown cites a source position that names NO "
              "tracked file — a reader cannot resolve it at all, and it is indistinguishable "
              "from a typo.", file=sys.stderr)
        for r in sorted(by_kind["unresolved"], key=lambda r: (r["md"], r["md_line"])):
            print(f"  {r['md']}:{r['md_line']}  cites  {r['ref']}", file=sys.stderr)
        print("\n  If it names a DEPENDENCY, write it crate-and-version qualified, e.g. "
              "`gix-0.87.1/src/config/cache/init.rs:229` — that resolves AND dates itself.\n"
              "  If it is an illustrative example, reword it so it is not a positional "
              "reference at all.", file=sys.stderr)

    if by_kind["ambiguous"]:
        # ⭐ One message for a bare basename and for a PARTIAL path, deliberately:
        # the defect is the same (a reader reaches more than one file) and so is
        # the repair. `SIGNOFF-REPAIR.11.17.2` found the partial-path half of it
        # being passed by the gate because the reference contained a slash.
        print("\nPOSITIONAL-REF: tracked Markdown cites a source position that names "
              "more than one tracked file — a reader cannot resolve it, and "
              "`docs/CLAIM_VERIFICATION.md` §4.1 grades a NAMED INSTANCE as exact with no "
              "tolerance band.", file=sys.stderr)
        for r in sorted(by_kind["ambiguous"], key=lambda r: (r["md"], r["md_line"])):
            print(f"  {r['md']}:{r['md_line']}  cites  {r['ref']}", file=sys.stderr)
            print(f"      names {len(r['owners'])} tracked files: " + ", ".join(r["owners"]),
                  file=sys.stderr)
        print("\n  Write the reference repo-root-relative, e.g. "
              "`crates/reasonbraid-server/tests/profiles.rs:5696`.", file=sys.stderr)

    if by_kind["dependency-stale"]:
        print("\nPOSITIONAL-REF: tracked Markdown cites a DEPENDENCY at a version "
              "`Cargo.lock` does not pin — the line number was exact against source this "
              "project no longer builds.", file=sys.stderr)
        for r in sorted(by_kind["dependency-stale"], key=lambda r: (r["md"], r["md_line"])):
            dep = r["dependency"]
            print(f"  {r['md']}:{r['md_line']}  cites  {r['ref']}", file=sys.stderr)
            print(f"      {dep['crate']} {dep['version']} — the lock pins "
                  + ", ".join(dep["pinned"]), file=sys.stderr)
        print("\n  Re-verify the line against the pinned version and update the citation, "
              "or say in the sentence that it is a frozen historical reading.", file=sys.stderr)
    return 1


SELF_TEST_SRC = [
    "crates/reasonbraid-server/src/profiles.rs",
    "crates/reasonbraid-server/tests/profiles.rs",
    "crates/reasonbraid-server/tests/support/mod.rs",
    "crates/reasonbraid-browse/tests/support/mod.rs",
    "crates/reasonbraid-core/src/authority.rs",
    "crates/reasonbraid-node/src/channel.rs",
    "scripts/only_here.py",
]

SELF_TEST_LOCK = """\
[[package]]
name = "gix"
version = "0.87.1"

[[package]]
name = "axum-core"
version = "0.5.6"
"""


def self_test() -> int:
    arms: list[tuple[str, bool]] = []
    resolver = Resolver(SELF_TEST_SRC, SELF_TEST_LOCK)

    def run(text: str, md="docs/x.md"):
        return classify([md], resolver, lambda _p: text)

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

    # 10 ⭐ THE SECOND ARM, in the direction that BLOCKS. Arm 5 proves the
    #    CLASSIFIER calls it unresolved; this proves the GATE refuses it. Before
    #    `SIGNOFF-REPAIR.11.17.1` the classifier said `unresolved` and `--check`
    #    returned 0, so the class was measured and not enforced.
    rows = run("`deleted_thing.rs:9` names nothing")
    arms.append(("an UNRESOLVED reference is refused by the gate", quiet(check, rows) == 1))
    # 11 and the negative half: a resolvable corpus must still pass, or arm 10
    #    is satisfied by a gate that refuses everything.
    rows = run("`crates/reasonbraid-server/tests/profiles.rs:5696` and `only_here.py:12`")
    arms.append(("a fully resolvable corpus still passes", quiet(check, rows) == 0))
    # 12 ⛔ A DEPENDENCY citation resolves when it is crate-and-version qualified,
    #    which is the repair this class actually needs: `init.rs:229` named no
    #    tracked file for eight occurrences and was `gix-0.87.1/src/config/cache/
    #    init.rs` all along. Qualified, it carries its own version, so a later
    #    reader knows which source the line number was exact against.
    rows = run("`gix-0.87.1/src/config/cache/init.rs:229` sets `system: use_system`")
    arms.append(("a crate-and-version qualified dependency citation is its own kind",
                 [r["kind"] for r in rows] == ["dependency"]))

    # ---- `SIGNOFF-REPAIR.11.17.2`: a slash is not a resolution -----------------
    # 13 🔴 THE DEFECT THAT OPENED THE LEAF. `tests/support/mod.rs:195` is a suffix
    #    of three tracked files; the old classifier said `pathed` and the gate
    #    passed it. Both directions are pinned: the classifier's verdict here,
    #    and the gate's refusal in arm 14.
    rows = run("the harness writes it (`tests/support/mod.rs:195`)")
    arms.append(("a PARTIAL path naming several tracked files is ambiguous",
                 [r["kind"] for r in rows] == ["ambiguous"]
                 and len(rows[0]["owners"]) == 2))
    # 14 and the gate refuses it — the arm that fails against the pre-fix code
    rows = run("the harness writes it (`tests/support/mod.rs:195`)")
    arms.append(("a partial path naming several tracked files is REFUSED",
                 quiet(check, rows) == 1))
    # 15 ⭐ the NEGATIVE half: a partial path naming exactly ONE tracked file is
    #    `partial` and PASSES. Without this, the rule degenerates into "a path
    #    must be complete", which would condemn 27 standing occurrences that a
    #    reader resolves on the first search.
    rows = run("`reasonbraid-node/src/channel.rs:88` holds the send")
    arms.append(("a partial path naming exactly one tracked file is 'partial'",
                 [r["kind"] for r in rows] == ["partial"]))
    # 16 ⚠️ LABELLED: THIS ARM PASSES BOTH BEFORE AND AFTER THE FIX, and that is
    #    what it is for. Run against the pre-fix classifier it passes because a
    #    slash meant `pathed`; run against this one it passes because `partial`
    #    is accepted. It is the negative half that stops arm 14 being satisfied
    #    by a gate that refuses EVERY partial path — it covers the rule's
    #    boundary, not the defect. (The in-situ falsification of the other ten
    #    is recorded in `SIGNOFF-REPAIR.11.17.2`.)
    rows = run("`reasonbraid-node/src/channel.rs:88` holds the send")
    arms.append(("a partial path naming one tracked file is ACCEPTED",
                 quiet(check, rows) == 0))
    # 17 ⛔ THE FOUNDING FALSE POSITIVE, ARRIVING IN THE SUFFIX MATCHER. A suffix
    #    must fall on a path segment boundary: `core/src/authority.rs` is a STRING
    #    suffix of `crates/reasonbraid-core/src/authority.rs` and not a path one,
    #    so `str.endswith` would have resurrected the 93-false-positive bug as a
    #    93-false-NEGATIVE one.
    rows = run("`core/src/authority.rs:562` was the truncation")
    arms.append(("a string suffix that is not a path suffix does not resolve",
                 [r["kind"] for r in rows] == ["unresolved"]))
    # 18 a path naming nothing at all is unresolved, and refused
    rows = run("`crates/reasonbraid-server/src/gone.rs:4` is not here")
    arms.append(("a complete path naming no tracked file is unresolved",
                 [r["kind"] for r in rows] == ["unresolved"] and quiet(check, rows) == 1))
    # 19 🔴 THE FAILURE MODE THE DEPENDENCY CLASS EARNS. A citation into a version
    #    `Cargo.lock` no longer pins is stale, and before this nothing said so.
    rows = run("`gix-0.86.0/src/init.rs:76` builds the options")
    arms.append(("a dependency citation at an unpinned version is stale",
                 [r["kind"] for r in rows] == ["dependency-stale"]))
    # 20 and the gate refuses it
    rows = run("`gix-0.86.0/src/init.rs:76` builds the options")
    arms.append(("a stale dependency citation is REFUSED", quiet(check, rows) == 1))
    # 21 ⚠️ the negative half of the dependency rule: a crate-version-SHAPED segment
    #    for a crate the lock does not carry is NOT a dependency citation. Without
    #    this the classifier would launder any `<word>-<semver>/…` path into a pass.
    rows = run("`notacrate-1.2.3/src/lib.rs:7` names nothing")
    arms.append(("a crate-shaped segment absent from the lock is not a dependency",
                 [r["kind"] for r in rows] == ["unresolved"]))
    # 22 ⛔ and a HYPHENATED crate name must survive the name/version split, which
    #    is the same class of bug as arm 3 one layer down.
    rows = run("`axum-core-0.5.6/src/ext_traits/request.rs:319` extends the trait")
    arms.append(("a hyphenated crate name splits from its version correctly",
                 [r["kind"] for r in rows] == ["dependency"]
                 and rows[0]["dependency"]["crate"] == "axum-core"))
    # 23 ⭐ an ELIDED registry prefix still resolves: the crate-version segment is
    #    what carries the meaning, wherever it sits in the path.
    rows = run("`.project-data/cargo/registry/src/…/axum-core-0.5.6/src/lib.rs:1`")
    arms.append(("a crate-version segment is found after an elision",
                 [r["kind"] for r in rows] == ["dependency"]))
    # 24 ⭐ LIVE-CORPUS ARM (`TOOLBOX.md`, and `SIGNOFF-REPAIR.11.20`'s finding):
    #    every control above reads a fixture written beside this code, so none of
    #    them notices if the census stops being able to read the real corpus. This
    #    asserts almost nothing about the CONTENT — only that the instrument can
    #    still see a population and a `Cargo.lock` to resolve it against.
    #    ⚠️ LABELLED, like arm 16: it passes before and after the classifier fix,
    #    because it covers a DIFFERENT failure (the census going blind to its
    #    corpus) and not the slash defect. Its own falsification is the one
    #    `SIGNOFF-REPAIR.11.20` ran — break the corpus key and it is the arm that
    #    names itself.
    try:
        live = collect()
        live_ok = len(live) > 0 and bool(Resolver(
            [p for p in tracked("*") if p.endswith(SOURCE_SUFFIXES)],
            (ROOT / "Cargo.lock").read_text(encoding="utf-8", errors="replace"),
        ).packages)
    except Exception as exc:                                  # noqa: BLE001
        live_ok = False
        print(f"census_positional_refs: live-corpus arm raised {exc!r}")
    arms.append(("live-corpus: the real tree is readable to this census", live_ok))

    passed = sum(1 for _, ok in arms if ok)
    for name, ok in arms:
        print(f"census_positional_refs: {'arm ok' if ok else 'arm FAILED'} — {name}")
    print(f"census_positional_refs --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    if mode == "--calibrate":
        rest = argv[2:]
        depth = int(rest.pop(0)) if rest and rest[0].isdigit() else 200
        kinds = tuple(rest) if rest else REFUSED
        unknown = [k for k in kinds if k not in ALL_KINDS]
        if unknown:
            print(f"census: unknown kind(s) {', '.join(unknown)}; "
                  f"known: {', '.join(ALL_KINDS)}", file=sys.stderr)
            return 2
        c = calibrate(depth, kinds)
        print(f"a gate on [{', '.join(c['kinds'])}], over {c['commits_examined']} commits")
        print(f"  commits it would have BLOCKED : {c['commits_blocked']}  ({c['blocked_pct']}%)")
        for d in c["detail"]:
            print(f"     {d['commit']}  {', '.join(d['files'])}")
        return 0
    if mode == "--at":
        if len(argv) < 3:
            print("census: --at needs a revision", file=sys.stderr)
            return 2
        rows = collect_at(argv[2])
        if len(argv) > 3 and argv[3] == "--json":
            print(json.dumps(rows, indent=2, sort_keys=True))
            return 0
        print(f"at {argv[2]}:")
        report(rows)
        return 0
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
