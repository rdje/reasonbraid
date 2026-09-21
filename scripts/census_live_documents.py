#!/usr/bin/env python3
"""scripts/census_live_documents.py — the live-document and route census.

`SIGNOFF-REPAIR.11.4.2.5`. Phase 2 step 1 of the live-document containment
adoption guide: *enumerate every live document, route destination and
historical terminal, and follow routes transitively — a bounded source
pointing to an uncontrolled destination is not contained.*

⛔ THIS INSTRUMENT PROPOSES NO RULE AND WRITES NOTHING TO THE TREE. `.11.6`
forbids proposing the rule before the population is counted, and the two
earlier attempts in this family were both decided by the measurement rather
than by the shape of the idea (`.11.4.5.3` declined a generator; `.11.9`
declined a gate that would have fired on 114 of 131). This counts.

⭐ THE LIVE-DOCUMENT SET IS DERIVED, NOT RESTATED. It is parsed out of
`scripts/check_lockstep_claim.sh`'s own `LIVE_DOCS=(…)` array, which that gate
already validates against `COMMIT.md`. A second hand-written copy of that list
is the defect this repository has met repeatedly — most recently at
`SCAFFOLD-COVERAGE`, where a list nothing derived had drifted by seven checks.
The doctrine registry is parsed out of `scripts/check_doctrines.sh` for the
same reason: the enforcer is the producer of that population.

⚠️ WHAT THIS CENSUS CANNOT SAY. `names_path` answers *is this surface visible
to that check at all* — a decidable question over the check's source. It does
NOT answer *does that check BOUND this surface*, which is a claim about
behaviour. That question is answered by `--probe-bounds`, which appends real
bytes to the real file, runs the real enforcer, and restores the file
byte-identically. An absence claim owes a positive control in the same run
(`SIGNOFF-REPAIR.11.25.1.1`), and that arm is the positive control.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


def repo_root() -> Path:
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return Path(out.stdout.strip())


# ── pure verdicts (the --self-test arm is ground truth for every one) ──────────

@dataclass(frozen=True)
class RouteRow:
    path: str
    klass: str
    control: str
    owner: str
    assertions: str = ""


def parse_route_registry(text: str) -> list[RouteRow]:
    """`.doctrine/readme_routes.txt` → rows. Comments and blanks are dropped.

    A row short of four fields is returned with empty tails rather than
    skipped: `check_readme_stability.sh` calls that shape MALFORMED and fails
    on it, so a census that silently dropped it would disagree with the gate.

    ⚠️ The FIFTH field is OPTIONAL and holds the row's machine-readable
    assertions (`SIGNOFF-REPAIR.11.4.2.6.7`); an absent one is the empty string,
    which is the adjudication *this control states nothing a machine can
    decide*. ⛔ A SIXTH field and beyond would be silently swallowed by a
    fixed-width split, so the tail is joined back rather than dropped, and
    `ROUTE-CONTROL` refuses the unparseable term that produces.
    """
    rows: list[RouteRow] = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("|")
        parts += [""] * (5 - len(parts))
        rows.append(RouteRow(parts[0], parts[1], parts[2], parts[3], "|".join(parts[4:])))
    return rows


def governing_row(path: str, rows: list[RouteRow]) -> RouteRow | None:
    """Prefix closure, LONGEST match wins.

    The registry's own contract is *"a row's path governs that path and
    everything under it"*. Longest-match matters because `docs/tasks/` and
    `docs/TASK_TREE.md` are both rows and only one of them is a prefix of a
    tree file; taking the first match would make the answer depend on row
    order, which is not part of the contract.
    """
    best: RouteRow | None = None
    for row in rows:
        if row.path and path.startswith(row.path):
            if best is None or len(row.path) > len(best.path):
                best = row
    return best


def parse_live_docs(shell_text: str) -> list[str]:
    """`LIVE_DOCS=(a.md b.md)` out of `scripts/check_lockstep_claim.sh`."""
    m = re.search(r"^LIVE_DOCS=\(([^)]*)\)", shell_text, re.MULTILINE)
    if not m:
        return []
    return [t for t in m.group(1).split() if t]


def parse_doctrine_registry(shell_text: str) -> list[tuple[str, str]]:
    """`scripts/check_doctrines.sh` → [(doctrine id, check path)].

    Both entry forms are read — the `DOCTRINES=(…)` literals and the
    conditional `DOCTRINES+=(…)` appends. ⛔ The second form is the one a
    plain grep missed during `DOC-0090`'s verification pass, which is why it
    is matched explicitly here rather than by scanning for quoted lines.

    🔴 COMMENT LINES ARE SKIPPED, AND UNTIL `.11.4.2.7.3.2.1` THEY WERE NOT.
    The enforcer documents its own format in a comment — *Each entry:
    "ID|what it proves|relative/path/to/check.sh"* — and this function read that
    DESCRIPTION as an INSTANCE, returning a 25th doctrine called `ID` against the
    enforcer's own `${#DOCTRINES[@]}` of 24. ⛔ It was not merely a wrong count:
    `census_route_controls.eval_doctrine` resolves a registry row's declared
    control through here, so a row declaring `doctrine=ID` was ACCEPTED as
    registered and enforced — a gate built to refuse controls nothing enforces,
    confirming one that cannot exist.
    """
    found: list[tuple[str, str]] = []
    for line in shell_text.splitlines():
        if line.lstrip().startswith("#"):
            continue
        for m in re.finditer(r'"([A-Z][A-Z0-9-]*)\|[^"]*\|([^"|]+)"', line):
            found.append((m.group(1), m.group(2).strip()))
    return found


def shape_verdict(added: int, removed: int, current: int, hist_max: int) -> str:
    """Classify a file's growth shape from decidable facts only.

    ⛔ NO THRESHOLD IS USED. Every earlier rule in this family that needed a
    tuned constant had to be re-derived under load (`.11.4.2.1`'s shed rule,
    sharpened three times in six commits). These three outcomes are questions
    a file's own history answers yes or no:

      append_only       — it has never lost a byte.
      at_all_time_high  — it has lost bytes, and the tip is still its maximum.
      below_peak        — the tip is below its historical maximum.

    Only `below_peak` is consistent with a registry row claiming a surface is
    "overwritten rather than appended".
    """
    if removed == 0:
        return "append_only"
    if current >= hist_max:
        return "at_all_time_high"
    return "below_peak"


def pressure_axes(data: bytes) -> tuple[int, int, int]:
    """(lines, bytes, max content-line bytes) — the three INDEPENDENT axes.

    Measured independently because a line cap alone does not bound a file and
    a byte cap alone does not bound a line: `check_readme_stability.sh`'s own
    header records a 60-line file carrying 138,403 bytes with a single line of
    18,816. `lines` counts newline terminators, matching `wc -l`.
    """
    line_count = data.count(b"\n")
    widest = max((len(s) for s in data.split(b"\n")), default=0)
    return line_count, len(data), widest


# ── history measurement ───────────────────────────────────────────────────────

@dataclass
class History:
    versions: int
    grew: int
    shrank: int
    same: int
    added: int
    removed: int
    first: int
    current: int
    hist_max: int


def batch_sizes(lines: list[str]) -> list[int]:
    """`git cat-file --batch-check` output → the sizes, absent objects dropped.

    ⛔ THE DROP IS THE OLD BEHAVIOUR, PRESERVED EXACTLY. The per-version walk
    this replaces tested `returncode == 0` and skipped anything else, which is
    how a commit that DELETED the path — `git log -- path` reports it, and
    `rev:path` does not resolve there — stayed out of the series. Batch mode
    reports the same revision as `<input> missing` on its own line, one line per
    input line and in order, so the skip is the same skip rather than a new rule
    that happens to agree today.
    """
    return [int(line) for line in lines if line.strip().isdigit()]


def version_sizes(root: Path, path: str, rev: str = "HEAD") -> list[int]:
    """Every version's size, oldest first, in ONE `git cat-file` process.

    `rev` truncates the history to what was reachable at that commit, so a
    growth shape can be asked as of a past state — the only way to put a
    registry row's claim back against the tree that refuted it
    (`SIGNOFF-REPAIR.11.4.2.6.7`). ⛔ The default `HEAD` is the behaviour this
    function already had: `git log -- path` walks from HEAD, and naming it
    explicitly was proved to return a byte-identical revision list.

    ⚠️ It used to be one process PER VERSION. Measured on the three ledgers a
    growth assertion must cover (`SIGNOFF-REPAIR.11.4.2.6.7.2`): 1,756 versions
    cost **23.24 s** that way against a **28.74 s** whole-enforcer run, and
    **0.11 s** this way — the same numbers, 211 times cheaper. A gate leg that
    nearly doubles the gate is one people route around (`SIGNOFF-REPAIR.11.5`).
    """
    revs = subprocess.run(
        ["git", "log", "--reverse", "--format=%H", rev, "--", path],
        cwd=root, capture_output=True, text=True,
    ).stdout.split()
    if not revs:
        return []
    out = subprocess.run(
        ["git", "cat-file", "--batch-check=%(objectsize)"],
        cwd=root, capture_output=True, text=True,
        input="".join(f"{rev}:{path}\n" for rev in revs),
    )
    return batch_sizes(out.stdout.splitlines())


def measure_history(root: Path, path: str, rev: str = "HEAD") -> History:
    sizes = version_sizes(root, path, rev)
    if not sizes:
        return History(0, 0, 0, 0, 0, 0, 0, 0, 0)
    grew = shrank = same = added = removed = 0
    for a, b in zip(sizes, sizes[1:]):
        if b > a:
            grew += 1
            added += b - a
        elif b < a:
            shrank += 1
            removed += a - b
        else:
            same += 1
    return History(
        versions=len(sizes), grew=grew, shrank=shrank, same=same,
        added=added, removed=removed,
        first=sizes[0], current=sizes[-1], hist_max=max(sizes),
    )


# ── the bound probe: the positive control for every absence claim ─────────────

PROBE_MARKER = "rb-census-live-documents probe"


def probe_verdict(path: str, rc: int, output: str) -> str:
    """Classify one probe from the enforcer's REASON, never from its exit code.

    ⛔ THE EXIT CODE IS THE ENFORCER'S VERDICT ON THE WHOLE TREE. This function
    exists because reading it as the probed file's bound produced a measurably
    wrong answer: with `DEV_NOTES.md` missing a final newline, this probe
    reported `LIVE_STATUS.md` — a file with no size bound, untouched by the
    commit — as BOUNDED, because `FILE-TERMINATION` had failed somewhere else
    (`SIGNOFF-REPAIR.11.4.2.6.4`).

    ⚠️ And the positive controls did NOT save it. An absence claim owes them
    (`.11.25.1.1`) and this probe carried three — but a breach elsewhere makes
    every surface refuse, so the controls fail in the same direction for the
    same wrong reason. A control only discriminates if it can come apart from
    the thing it is controlling.

    A refusal counts only when some failing line NAMES the probed path.
    """
    if rc == 0:
        return "unbounded"
    for line in output.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("✅"):
            continue
        if path in stripped:
            return "bounded"
    return "refused-for-another-file"


def enforcer(root: Path) -> tuple[int, str]:
    r = subprocess.run(["bash", "scripts/check_doctrines.sh"],
                       cwd=root, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def probe_bound(root: Path, path: str, payload_bytes: int) -> tuple[str, bool]:
    """Append real bytes to the real file, run the real enforcer, restore.

    Returns (verdict, restored byte-identically). The original bytes are held in
    memory and rewritten in `finally`, then the SHA-256 is compared —
    `.11.4.2.4.1` restored four falsifications this way and compared the digest
    each time, because a falsification that leaves the tree changed has measured
    the gate and damaged the repository in the same run.
    """
    target = root / path
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    filler = ("\n<!-- " + PROBE_MARKER + " " + "x" * 64 + " -->").encode()
    block = filler * max(1, payload_bytes // len(filler))
    try:
        target.write_bytes(original + block + b"\n")
        rc, output = enforcer(root)
    finally:
        target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()
    return probe_verdict(path, rc, output), before == after


# ── self-test ─────────────────────────────────────────────────────────────────

def self_test() -> int:
    controls = 0
    fails: list[str] = []

    def check(name: str, got, want) -> None:
        nonlocal controls
        controls += 1
        if got != want:
            fails.append(f"{name}: got {got!r} want {want!r}")

    reg = parse_route_registry(
        "# a comment\n"
        "\n"
        "docs/book/|reader_navigation|mdBook|repo-local\n"
        "docs/|author_overflow|generic|repo-local\n"
        "BROKEN.md|only_two_fields\n"
    )
    check("registry row count", len(reg), 3)
    check("registry comment dropped", [r.path for r in reg][0], "docs/book/")
    check("malformed row kept", (reg[2].path, reg[2].control), ("BROKEN.md", ""))

    # longest-prefix closure, not first match: both rows are prefixes.
    check("longest prefix wins",
          governing_row("docs/book/src/cli.md", reg).path, "docs/book/")
    check("shorter prefix still governs",
          governing_row("docs/adr/001.md", reg).path, "docs/")
    check("ungoverned is None", governing_row("DEV_NOTES.md", reg), None)
    check("empty path never governs",
          governing_row("anything.md", [RouteRow("", "c", "p", "o")]), None)

    check("live docs parsed",
          parse_live_docs("x=1\nLIVE_DOCS=(README.md MEMORY.md DEV_NOTES.md)\ny=2\n"),
          ["README.md", "MEMORY.md", "DEV_NOTES.md"])
    check("live docs absent → empty", parse_live_docs("no array here\n"), [])

    # ⛔ THE FIXTURE CARRIES A COMMENT DESCRIBING THE FORMAT, because the real
    # file does and the tidier fixture is exactly why this went unseen
    # (`docs/knowledge/a-self-test-cannot-be-tidier-than-the-real-input.md`).
    check("doctrine registry both forms, comment not an entry",
          parse_doctrine_registry(
              '# Universal registry. Each entry: "ID|what it proves|relative/path/to/check.sh"\n'
              'DOCTRINES=(\n  "A-ID|what it proves|scripts/a.sh"\n)\n'
              '  # an indented comment: "C-ID|described|scripts/c.sh"\n'
              'DOCTRINES+=("B-ID|proves b|scripts/b.sh")\n'),
          [("A-ID", "scripts/a.sh"), ("B-ID", "scripts/b.sh")])
    check("doctrine registry: a trailing comment on a real entry still parses",
          parse_doctrine_registry('  "D-ID|proves d|scripts/d.sh"  # note\n'),
          [("D-ID", "scripts/d.sh")])

    # shape: the three outcomes, each pinned in both directions.
    check("never lost a byte", shape_verdict(100, 0, 100, 100), "append_only")
    check("lost bytes, tip is peak", shape_verdict(100, 10, 90, 90), "at_all_time_high")
    check("tip below peak", shape_verdict(100, 40, 60, 95), "below_peak")
    check("removal wins over peak order",
          shape_verdict(0, 0, 5, 9), "append_only")

    # pressure axes are independent: same byte count, different widest line.
    check("axes wide line", pressure_axes(b"a" * 50 + b"\n"), (1, 51, 50))
    check("axes many lines", pressure_axes(b"a\n" * 25 + b"\n"), (26, 51, 1))
    check("axes empty", pressure_axes(b""), (0, 0, 0))

    # The probe classification, in all three directions. ⛔ Arm 2 is the defect
    # this function exists for: a real refusal that names a DIFFERENT file.
    named = "  ❌ LEDGER-RUNWAY\n       LEDGER-RUNWAY: DEV_NOTES.md has -161875 bytes of headroom"
    other = "  ❌ PROJECT-SPECIFIC\n       FILE-TERMINATION: DEV_NOTES.md missing its final newline"
    check("a refusal naming the subject is not bounded",
          probe_verdict("DEV_NOTES.md", 1, named), "bounded")
    check("a refusal naming ANOTHER file was read as a bound",
          probe_verdict("LIVE_STATUS.md", 1, other), "refused-for-another-file")
    check("rc=0 is not unbounded", probe_verdict("LIVE_STATUS.md", 0, ""), "unbounded")
    check("rc=0 wins even when the output mentions the file",
          probe_verdict("DEV_NOTES.md", 0, named), "unbounded")
    # ⛔ A PASSING line naming the file must NOT count as a refusal: the enforcer
    # prints one ✅ line per check and those lines carry doctrine descriptions.
    check("a green line naming the file was read as a refusal",
          probe_verdict("DEV_NOTES.md", 1,
                        "  ✅ LESSON-PROMOTION a new dated lesson in DEV_NOTES.md is PROMOTED\n"
                        "  ❌ README-STABILITY\n       README-STABILITY: README.md is 999 lines"),
          "refused-for-another-file")

    # The optional fifth field, and the tail a fixed-width split would swallow.
    five = parse_route_registry(
        "A.md|hot_live|ceiling text|me|ceiling=100 doctrine=X\n"
        "B.md|reader_navigation|narrative text|me\n"
        "C.md|hot_live|odd|me|a=1|b=2\n"
    )
    check("fifth field read", five[0].assertions, "ceiling=100 doctrine=X")
    check("absent fifth field is empty, not missing", five[1].assertions, "")
    check("owner is no longer swallowed by the fifth field", five[0].owner, "me")
    check("a sixth field is kept for the checker to refuse", five[2].assertions, "a=1|b=2")

    # The batch size reader, whose whole job is to preserve a SKIP the previous
    # per-version walk expressed as `returncode != 0` (`.11.4.2.6.7.2`).
    check("batch sizes: plain sizes survive in order",
          batch_sizes(["12", "7", "100"]), [12, 7, 100])
    check("batch sizes: a missing object is DROPPED, not zero",
          batch_sizes(["12", "deadbeef:gone.md missing", "7"]), [12, 7])
    check("batch sizes: nothing but misses is empty",
          batch_sizes(["a:b missing", "c:d missing"]), [])
    check("batch sizes: blank lines are not sizes", batch_sizes(["", "  ", "5"]), [5])
    # ⛔ A dangling-symlink-style entry reports an object id and a type, never a
    # bare number, so the digit test is what separates a size from a report.
    check("batch sizes: an id+type+size line is not a bare size",
          batch_sizes(["1a2b3c blob 44"]), [])

    if fails:
        for f in fails:
            print(f"census_live_documents self-test MISSED: {f}", file=sys.stderr)
        return 1
    print(f"census_live_documents --self-test: {controls} controls pass")
    return 0


# ── report ────────────────────────────────────────────────────────────────────

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--probe-bounds", action="store_true",
                    help="append real bytes to each live document, run the real "
                         "enforcer, restore byte-identically, and report which "
                         "surfaces the gate actually refuses")
    ap.add_argument("--probe-bytes", type=int, default=200_000)
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    root = repo_root()
    rows = parse_route_registry((root / ".doctrine/readme_routes.txt").read_text())
    live = parse_live_docs((root / "scripts/check_lockstep_claim.sh").read_text())
    registry = parse_doctrine_registry((root / "scripts/check_doctrines.sh").read_text())

    sources = {}
    for doctrine_id, check_path in registry:
        p = root / check_path
        sources[doctrine_id] = p.read_text(errors="replace") if p.is_file() else ""

    print(f"=== live-document census — {len(live)} core documents "
          f"(derived from scripts/check_lockstep_claim.sh), "
          f"{len(rows)} route rows, {len(registry)} registered doctrines ===\n")

    total = bounded_total = 0
    for path in live:
        data = (root / path).read_bytes()
        lines, nbytes, widest = pressure_axes(data)
        hist = measure_history(root, path)
        row = governing_row(path, rows)
        namers = sorted(d for d, src in sources.items() if path in src)
        shape = shape_verdict(hist.added, hist.removed, hist.current, hist.hist_max)
        total += nbytes
        print(f"{path}")
        print(f"  axes        lines={lines:,}  bytes={nbytes:,}  widest_line={widest:,}")
        print(f"  history     versions={hist.versions}  grew={hist.grew} "
              f"shrank={hist.shrank} same={hist.same}")
        print(f"              added={hist.added:,}  removed={hist.removed:,}  "
              f"first={hist.first:,} → now={hist.current:,}  shape={shape}")
        print(f"  route row   {row.klass + ' — ' + row.control[:72] if row else '⛔ NONE'}")
        print(f"  named by    {', '.join(namers) if namers else '⛔ no registered check'}")
        print()

    print(f"core live-document corpus: {total:,} bytes")

    if args.probe_bounds:
        print(f"\n=== bound probe — +{args.probe_bytes:,} bytes per surface, "
              f"real enforcer, restored byte-identically ===")
        # ⛔ THE PROBE ESTABLISHES ITS OWN BASELINE AND REFUSES OVER A RED TREE.
        # Every verdict below is a difference from this run; without it, a tree
        # that was already failing makes every surface look bounded, and a
        # refusal is indistinguishable from an instrument with nothing to say.
        base_rc, _ = enforcer(root)
        if base_rc != 0:
            print("  REFUSED: the enforcer is already red on the unmodified tree, so no\n"
                  "  verdict below would be a property of the file it names. Fix the tree\n"
                  "  first — this probe measures a DIFFERENCE and there is no baseline.")
            return 2
        print(f"  baseline: enforcer green on the unmodified tree (rc={base_rc})")
        labels = {
            "bounded": "REFUSED (bounded — the refusal names this file)",
            "unbounded": "⛔ ACCEPTED (unbounded)",
            "refused-for-another-file":
                "⚠️ REFUSED BUT FOR ANOTHER FILE — not a bound on this one",
        }
        for path in live:
            verdict, restored = probe_bound(root, path, args.probe_bytes)
            print(f"  {path:<16} {labels[verdict]:<52} "
                  f"restored={'byte-identical' if restored else '⛔ NOT RESTORED'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
