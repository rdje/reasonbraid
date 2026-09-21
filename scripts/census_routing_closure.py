#!/usr/bin/env python3
"""Census what the routing-pressure closure CANNOT see (`SIGNOFF-REPAIR.11.4.2.7.3`).

`scripts/check_readme_stability.sh` requires every routed destination to end at a
governed row in `.doctrine/readme_routes.txt`. It discovers those destinations
from exactly THREE anchors — the landing page's link graph, the guard's own
routing hint, and the paths named inside declared control sentences. A tracked
document that none of the three names never enters the closure: it is neither
refused nor listed, and no lifecycle, owner, ceiling or verifier is ever asked
of it.

🔴 TWO INSTANCES WERE FOUND BY HAND, ONE LANE APART. `DEV_NOTES.md` had no row
for the life of the project (`.11.4.2.5`) while `COMMIT.md` made writing to it
mandatory every commit; `MEMORY_ARCHITECTURE.md` had none until `.11.4.2.7.2`,
while defining the layer-A contract that `MEMORY.md`, `CLAUDE.md` and `AGENTS.md`
are all governed against. Neither was caught by the closure, because the closure
is the thing that cannot see them.

⛔ THIS INSTRUMENT PROPOSES NO RULE AND WRITES NOTHING TO THE TREE. `.11.6`
requires the population before the rule, and this family has twice been decided
by the measurement against the shape of the idea (`.11.4.5.3` declined a
generator; `.11.9` declined a gate that would have fired on 114 of 131). It
counts, scores the candidate anchors, and stops.

⭐ THE CLOSURE IS RUN, NOT RE-IMPLEMENTED. `closure_tokens` lifts the guard's own
`extract_routes`, `routes_from_readme`, `routes_from_hint` and
`routes_from_controls` definitions out of its source and executes them under
`bash`, with ONE named substitution: `"$0"` becomes the guard's path, because the
hint leg reads the guard file and the driver is not that file. A Python
transcription of that pipeline would be a second copy of a gate's verdict, which
is the defect this repository keeps repairing.

⭐ AND THE DERIVATION IS PROVED AGAINST THE GATE'S BEHAVIOUR, not asserted.
`--verify-closure` deletes one row from the real registry, runs the real
enforcer, and restores byte-identically:

  - a row the closure NAMES (`TOOLBOX.md`) must make the gate refuse, naming it;
  - a row the closure CANNOT NAME (`DEV_NOTES.md`) must leave the gate GREEN.

The second arm is the finding itself, demonstrated on the live gate: a governed
row can be removed without the closure noticing, because the closure never
proposed that destination in the first place.

⚠️ WHAT THIS CENSUS DOES NOT SAY. "No row" is not "ungoverned". `README.md` is
the guard's own SUBJECT and is bounded by its line and byte caps directly;
`MEMORY.md` is bounded by `MEMORY-ARCH`. `--probe-bounds` answers the behaviour
question separately by appending real bytes and running the real enforcer, and
it carries its own positive control, because an absence claim owes one in the
same run (`SIGNOFF-REPAIR.11.25.1.1`).

    python3 -B scripts/census_routing_closure.py
    python3 -B scripts/census_routing_closure.py --as-of 386aa64^
    python3 -B scripts/census_routing_closure.py --verify-closure
    python3 -B scripts/census_routing_closure.py --probe-bounds
    python3 -B scripts/census_routing_closure.py --self-test

Self-test: scripts/census_routing_closure.py --self-test
"""

from __future__ import annotations

import argparse
import hashlib
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

# ⛔ IMPORTED, NOT RE-IMPLEMENTED. `parse_route_registry` already matches the
# gate's MALFORMED shape, `governing_row` already implements the registry's
# longest-prefix contract, and `probe_bound` already restores the tree it
# perturbs. A second copy of any of the three is how this family drifts.
from census_live_documents import (  # noqa: E402
    parse_route_registry,
    governing_row,
    probe_bound,
)

GUARD = "scripts/check_readme_stability.sh"
REGISTRY = ".doctrine/readme_routes.txt"
LANDING = "README.md"
BOOTSTRAP = "CLAUDE.md"
SCRATCH = "target/doctrine_scratch/routing_closure"

# The two instances this leaf exists because of, each with the revision at which
# it was invisible — the commit BEFORE the one that added its row by hand.
# ⛔ The revisions are the falsification targets, not decoration: an anchor that
# cannot flag both of these at these commits has not been shown to work.
INSTANCES = (
    ("DEV_NOTES.md", "386aa64^", "SIGNOFF-REPAIR.11.4.2.5"),
    ("MEMORY_ARCHITECTURE.md", "8aadac3", "SIGNOFF-REPAIR.11.4.2.7.2"),
)

# A line that STARTS a shell function definition at column 0 — the boundary
# `function_span` stops at, so one definition can never swallow the next.
DEFINITION_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\(\)")

# The shell functions that MAKE the closure, in the guard's own source.
# `extract_routes` is the shared filter the anchors pipe through, not an anchor.
HELPER_FUNCTIONS = ("extract_routes",)
ANCHOR_FUNCTIONS = (
    "routes_from_readme",
    "routes_from_hint",
    "routes_from_controls",
    "routes_from_tracked_markdown",
)

# ⛔ ONE ANCHOR CANNOT BE RUN AS-IS AT A PAST REVISION, and substituting it is
# the honest repair rather than a silent one. `routes_from_tracked_markdown`
# lists the WORKING INDEX (`git ls-files`), which answers for the checkout and
# not for `--as-of`'s revision — so a historical run would have mixed today's
# file list into an old tree's closure. The census supplies that leg from its own
# revision-aware `tracked_markdown`, and `--verify-closure` proves the two agree
# at `HEAD`, where both questions have the same answer.
LISTING_ANCHOR = "routes_from_tracked_markdown"


def repo_root() -> Path:
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return Path(out.stdout.strip())


# ── pure verdicts (the --self-test arm is ground truth for every one) ──────────

def function_span(shell_text: str, name: str) -> str | None:
    """One shell function's definition, verbatim, from `name()` to its closing `}`.

    ⚠️ The terminator is a line that is EXACTLY `}`, which is the shape every
    function in the guard uses. A nested brace on its own line at column 0 would
    end the span early; none exists, and `closure_tokens` fails loudly under
    `bash -u` rather than silently returning fewer tokens if one ever does.

    🔴 THE SEARCH STOPS AT THE NEXT DEFINITION, and this instrument's own
    self-test is why. Scanning to the first bare `}` made a ONE-LINE function —
    `f() { echo x }`, which has no such line — swallow every line up to the next
    function's closing brace and return a span containing two definitions. That
    is worse than returning nothing: the driver would have run code the census
    never meant to lift. An absent span is reported as absent.
    """
    lines = shell_text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if line.startswith(name + "()"):
            start = i
            break
    if start is None:
        return None
    for j in range(start + 1, len(lines)):
        if lines[j] == "}":
            return "\n".join(lines[start:j + 1])
        if DEFINITION_RE.match(lines[j]):
            return None
    return None


def landing_name(guard_text: str) -> str:
    """The guard's `TARGET` — the one document its second anchor excludes.

    ⛔ DERIVED, NEVER NAMED HERE. The landing page is excluded because it is the
    guard's SUBJECT and its ceiling is the guard's own line and byte caps; a
    registry row would be a second authority for one cap. A census that hardcoded
    `README.md` would keep agreeing with the guard right up until the guard's
    own assignment changed, which is the drift this project keeps repairing.
    """
    m = re.search(r'^TARGET="([^"]+)"', guard_text, re.MULTILINE)
    return m.group(1) if m else ""


def closure_reach(path: str, tokens: set[str]) -> bool:
    """Would the closure ever PROPOSE this path as a destination?

    A token is a concrete path the anchors emitted. A directory token governs
    what is under it — `docs/tasks/` reaches every tree file — which is the same
    prefix reading the guard applies to registry rows, pointed the other way.
    """
    if path in tokens:
        return True
    return any(t.endswith("/") and path.startswith(t) for t in tokens)


def governance_verdict(has_row: bool, row_reachable: bool, path_reachable: bool,
                       excluded: bool = False) -> str:
    """The four states a tracked document can be in, named rather than counted.

    `governed_by_hand` is the defect class this leaf is about: the row exists
    only because a person noticed. `invisible` is the same blind spot with
    nobody having noticed yet.

    🔴 TWO REACHABILITY FACTS, NOT ONE, and the first version of this function
    collapsed them and was wrong by 32. For a document that HAS a row the
    question is whether the CLOSURE would ever have proposed that row's path;
    for one that has none it is whether the closure proposes the document. Asking
    the second question of a collection member reported all 32 `docs/adr/` files
    the landing page does not individually link as `governed_by_hand`, when their
    row is reachable through the two it does — a governed collection misread as
    thirty-two hand-placed rows.
    """
    if excluded:
        # ⛔ NOT A GAP, AND SAYING SO IS THE POINT. The guard's second anchor
        # excludes its own subject by name, so reporting the landing page as
        # `invisible` would make this census carry one permanent false finding —
        # the shape that teaches a reader to skip the last line of the report.
        return "excluded_subject"
    if has_row:
        return "governed_reachable" if row_reachable else "governed_by_hand"
    return "unrouted_reachable" if path_reachable else "invisible"


def minimal_cover(all_paths: list[str], ungoverned: set[str]) -> list[str]:
    """The fewest registry paths that would govern every ungoverned document.

    ⭐ THIS IS THE NUMBER THAT DECIDES THE RULE'S COST, and it is not the
    document count. The registry governs by PREFIX, so a directory whose every
    tracked member is ungoverned needs ONE row, not one per file. For each
    ungoverned path the shallowest such ancestor wins; a directory holding even
    one governed sibling is mixed, so the search goes deeper and may reach the
    file itself.
    """
    cover: set[str] = set()
    for path in sorted(ungoverned):
        chosen = path
        parts = path.split("/")
        for depth in range(1, len(parts)):
            prefix = "/".join(parts[:depth]) + "/"
            members = [q for q in all_paths if q.startswith(prefix)]
            if members and all(q in ungoverned for q in members):
                chosen = prefix
                break
        cover.add(chosen)
    return sorted(cover)


def anchor_verdict(anchor: set[str], instance: str) -> str:
    """`catches` / `misses` — whether a candidate anchor would propose one path."""
    return "catches" if instance in anchor else "misses"


# ── the closure, run out of the guard's own source ─────────────────────────────

# The revision that means "the checkout, as the gate sees it". The gate judges the
# WORKING TREE, so a census whose only answer is `HEAD` answers a different
# question from the check it is about — visibly so while a change is staged.
WORKTREE = "WORKTREE"


def git_show(root: Path, rev: str, path: str) -> str | None:
    if rev == WORKTREE:
        target = root / path
        return target.read_text(errors="replace") if target.is_file() else None
    r = subprocess.run(["git", "show", f"{rev}:{path}"],
                       cwd=root, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def tracked_markdown(root: Path, rev: str) -> list[str]:
    """Every tracked `*.md` at `rev`, from the tree object rather than the index.

    ⛔ ONE ROUTE FOR BOTH CASES. `git ls-files` answers only for the working
    index, so a census that used it at HEAD and `ls-tree` at a past revision
    would be comparing two different questions. `ls-tree -r` answers both.
    """
    cmd = (["git", "ls-files"] if rev == WORKTREE
           else ["git", "ls-tree", "-r", "--name-only", rev])
    r = subprocess.run(cmd, cwd=root, capture_output=True, text=True, check=True)
    return sorted(p for p in r.stdout.splitlines() if p.endswith(".md"))


def closure_tokens(root: Path, rev: str,
                   guard_rev: str | None = None) -> tuple[set[str], list[str]]:
    """Run the guard's OWN anchor pipeline over the tree at `rev`.

    Returns (tokens, the anchor functions that were found). The guard's
    definitions are lifted verbatim; `"$0"` becomes the guard's own path because
    the hint leg reads the guard file and this driver is not that file.

    ⭐ `guard_rev` SEPARATES THE RULE FROM THE TREE IT JUDGES, and that is the
    whole falsification. Asking a past commit's own guard about a past commit's
    tree only re-reports history; asking TODAY'S guard about that tree answers
    the question an acceptance criterion actually poses — *would the anchor we
    are shipping have caught this?*
    """
    guard_text = git_show(root, guard_rev or rev, GUARD)
    landing_text = git_show(root, rev, LANDING)
    registry_text = git_show(root, rev, REGISTRY)
    if guard_text is None or landing_text is None or registry_text is None:
        return set(), []

    scratch = root / SCRATCH
    scratch.mkdir(parents=True, exist_ok=True)
    guard_file = scratch / "guard.sh"
    guard_file.write_text(guard_text)
    (scratch / "landing.md").write_text(landing_text)
    (scratch / "registry.txt").write_text(registry_text)

    spans, found, called = [], [], []
    for name in HELPER_FUNCTIONS + ANCHOR_FUNCTIONS:
        span = function_span(guard_text, name)
        if span is None:
            continue
        spans.append(span.replace('"$0"', '"$GUARD"'))
        found.append(name)
        if name in ANCHOR_FUNCTIONS and name != LISTING_ANCHOR:
            called.append(name)

    driver = "\n".join([
        "set -uo pipefail",
        'GUARD="$1"; TARGET="$2"; INVENTORY="$3"',
        *spans,
        "{ " + "; ".join(called) + "; } | LC_ALL=C sort -u",
    ])
    driver_file = scratch / "closure_driver.sh"
    driver_file.write_text(driver + "\n")

    r = subprocess.run(
        ["bash", str(driver_file), str(guard_file),
         str(scratch / "landing.md"), str(scratch / "registry.txt")],
        cwd=root, capture_output=True, text=True,
    )
    tokens = set(r.stdout.split())
    if LISTING_ANCHOR in found:
        tokens |= set(tracked_markdown(root, rev)) - {landing_name(guard_text)}
    return tokens, found


def bootstrap_tokens(root: Path, rev: str) -> set[str]:
    """Candidate anchor B: the destinations `CLAUDE.md`'s bootstrap list names.

    ⭐ Extracted with the GUARD'S OWN extractor rather than a second rule, since
    "a second anchor" means the same mechanism pointed at another file.
    """
    guard_text = git_show(root, rev, GUARD)
    bootstrap_text = git_show(root, rev, BOOTSTRAP)
    if guard_text is None or bootstrap_text is None:
        return set()
    span = function_span(guard_text, "extract_routes")
    if span is None:
        return set()
    scratch = root / SCRATCH
    scratch.mkdir(parents=True, exist_ok=True)
    (scratch / "bootstrap.md").write_text(bootstrap_text)
    driver = "set -uo pipefail\n" + span + '\nextract_routes < "$1"\n'
    driver_file = scratch / "bootstrap_driver.sh"
    driver_file.write_text(driver)
    r = subprocess.run(["bash", str(driver_file), str(scratch / "bootstrap.md")],
                       cwd=root, capture_output=True, text=True)
    return set(r.stdout.split())


# ── the live-gate controls ────────────────────────────────────────────────────

def enforcer(root: Path) -> tuple[int, str]:
    r = subprocess.run(["bash", "scripts/check_doctrines.sh"],
                       cwd=root, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def drop_row_and_run(root: Path, row_path: str) -> tuple[int, str, bool]:
    """Delete one registry row, run the REAL enforcer, restore byte-identically.

    ⛔ The original bytes are held in memory and rewritten in `finally`, then the
    SHA-256 is compared — a falsification that leaves the tree changed has
    measured the gate and damaged the repository in the same run
    (`SIGNOFF-REPAIR.11.4.2.4.1`).
    """
    target = root / REGISTRY
    original = target.read_bytes()
    before = hashlib.sha256(original).hexdigest()
    kept = [line for line in original.decode().splitlines(keepends=True)
            if not line.startswith(row_path + "|")]
    try:
        target.write_bytes("".join(kept).encode())
        rc, output = enforcer(root)
    finally:
        target.write_bytes(original)
    after = hashlib.sha256(target.read_bytes()).hexdigest()
    return rc, output, before == after


def closure_refusal(output: str, path: str) -> bool:
    """Did the CLOSURE LEG specifically refuse this path?

    🔴 THIS FUNCTION REPLACED ONE THAT READ THE WHOLE ENFORCER, AND THE
    REPLACEMENT WAS FORCED BY A FAILED CONTROL RATHER THAN BY REVIEW. The first
    version asked *does any failing line name this path*, and arm B — which
    predicts the gate stays GREEN when a row outside the closure is removed —
    came back red and naming `DEV_NOTES.md`. The refusal was `ROUTE-CONTROL`'s
    (`adjudicated path 'DEV_NOTES.md' is not a registry row`) and its self-test
    arm; `README-STABILITY` was green throughout, which is exactly the finding
    the arm exists to demonstrate. ⛔ That is `census_live_documents.probe_verdict`'s
    lesson one layer up (`SIGNOFF-REPAIR.11.4.2.6.4`): a breach elsewhere makes
    every surface look refused, so a control must match the LEG it is about.

    The guard emits one exact sentence for a closure breach, and nothing else in
    the enforcer emits it.
    """
    needle = f"unrouted destination: {path}"
    return any(needle in line for line in output.splitlines())


def failing_doctrines(output: str) -> list[str]:
    """The doctrine ids the enforcer marked failed — so a red arm explains itself.

    A control that reports only its own verdict leaves the reader to guess what
    else moved; an instrument must explain its own failure
    (`docs/knowledge/an-instrument-must-explain-its-own-failure.md`).
    """
    ids: list[str] = []
    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("❌"):
            parts = stripped.split()
            if len(parts) > 1:
                ids.append(parts[1])
    return ids


# ── self-test ─────────────────────────────────────────────────────────────────

def self_test() -> int:
    fails = 0
    ran = 0

    def check(label: str, got: object, want: object) -> None:
        # ⛔ The control COUNT is produced here, never restated in prose: a
        # number with no producer drifts (`docs/knowledge/a-restated-number-needs-a-producer.md`).
        nonlocal fails, ran
        ran += 1
        if got != want:
            print(f"self-test MISSED: {label}\n  got:  {got!r}\n  want: {want!r}",
                  file=sys.stderr)
            fails += 1

    # function_span: verbatim extraction, terminated by a bare `}`.
    shell = "\n".join([
        "preamble() { echo no }",
        "extract_routes() { # comment",
        "  tr a b",
        "}",
        "after() {",
        "  :",
        "}",
    ])
    check("span/body", function_span(shell, "extract_routes"),
          "extract_routes() { # comment\n  tr a b\n}")
    check("span/second", function_span(shell, "after"), "after() {\n  :\n}")
    check("span/absent", function_span(shell, "missing"), None)
    # A one-line function whose `}` is not alone on its line has no span, and the
    # absence is reported rather than guessed at.
    check("span/inline", function_span(shell, "preamble"), None)

    # closure_reach: exact tokens, directory tokens, and the near-miss that a
    # plain prefix test would accept.
    tokens = {"ROADMAP.md", "docs/tasks/", "docs/book/"}
    check("reach/exact", closure_reach("ROADMAP.md", tokens), True)
    check("reach/dir", closure_reach("docs/tasks/PHASE-1.md", tokens), True)
    check("reach/absent", closure_reach("DEV_NOTES.md", tokens), False)
    check("reach/file-prefix", closure_reach("ROADMAP.md.bak", tokens), False)

    # governance_verdict: all four states.
    check("verdict/gr", governance_verdict(True, True, False), "governed_reachable")
    check("verdict/hand", governance_verdict(True, False, True), "governed_by_hand")
    check("verdict/ur", governance_verdict(False, False, True), "unrouted_reachable")
    check("verdict/inv", governance_verdict(False, False, False), "invisible")
    # The collection member that the collapsed version got wrong: its own path is
    # unreachable, its row's path is not, and it is governed.
    check("verdict/member", governance_verdict(True, True, False), "governed_reachable")
    # The guard's own subject: no row, unreachable, and NOT a gap.
    check("verdict/excluded", governance_verdict(False, False, False, excluded=True),
          "excluded_subject")
    check("verdict/excluded-wins", governance_verdict(True, True, True, excluded=True),
          "excluded_subject")
    # landing_name is derived from the guard's own assignment.
    check("landing", landing_name('X=1\nTARGET="README.md"\nY=2'), "README.md")
    check("landing/absent", landing_name("no target here"), "")

    # minimal_cover: a wholly-ungoverned directory collapses to ONE row; a mixed
    # directory does not collapse at all; a root file is its own terminal.
    paths = [
        "A.md",
        "docs/one.md", "docs/two.md",
        "notes/a.md", "notes/b.md", "notes/deep/c.md",
    ]
    ungoverned = {"A.md", "docs/two.md", "notes/a.md", "notes/b.md", "notes/deep/c.md"}
    check("cover", minimal_cover(paths, ungoverned), ["A.md", "docs/two.md", "notes/"])
    check("cover/empty", minimal_cover(paths, set()), [])

    # anchor_verdict.
    check("anchor/hit", anchor_verdict({"DEV_NOTES.md"}, "DEV_NOTES.md"), "catches")
    check("anchor/miss", anchor_verdict({"MEMORY.md"}, "DEV_NOTES.md"), "misses")

    # closure_refusal: matches the guard's OWN sentence, and nothing else. The
    # third case is the real output that falsified the first version of this
    # control — another doctrine refusing while naming the path.
    out = "\n".join([
        "README-STABILITY: unrouted destination: TOOLBOX.md — add a governed row",
        "  ❌ PROJECT-SPECIFIC       this project's own doctrine checks",
        "       REFUSED: adjudicated path 'DEV_NOTES.md' is not a registry row.",
        "  ✅ README-STABILITY       README.md stays a stable landing page",
    ])
    check("closure/hit", closure_refusal(out, "TOOLBOX.md"), True)
    check("closure/other-leg", closure_refusal(out, "DEV_NOTES.md"), False)
    check("closure/absent", closure_refusal(out, "MEMORY.md"), False)
    check("failing", failing_doctrines(out), ["PROJECT-SPECIFIC"])

    if fails:
        print(f"census_routing_closure: {fails} of {ran} self-test control(s) failed",
              file=sys.stderr)
        return 1
    print(f"census_routing_closure: self-test ok — {ran} controls "
          "(span + reach + verdict + cover + anchor + closure-leg ground truth)")
    return 0


# ── report ────────────────────────────────────────────────────────────────────

@dataclass
class Survey:
    rev: str
    paths: list[str]
    tokens: set[str]
    anchors_found: list[str]
    rows: list
    verdicts: dict[str, str]


def survey(root: Path, rev: str, guard_rev: str | None = None) -> Survey:
    paths = tracked_markdown(root, rev)
    tokens, found = closure_tokens(root, rev, guard_rev)
    registry_text = git_show(root, rev, REGISTRY) or ""
    rows = parse_route_registry(registry_text)
    def row_reachable(row) -> bool:
        return any(t == row.path or t.startswith(row.path) for t in tokens)

    excluded = landing_name(git_show(root, guard_rev or rev, GUARD) or "")
    verdicts = {}
    for p in paths:
        row = governing_row(p, rows)
        verdicts[p] = governance_verdict(
            row is not None,
            row is not None and row_reachable(row),
            closure_reach(p, tokens),
            excluded=(p == excluded and LISTING_ANCHOR in found),
        )
    return Survey(rev, paths, tokens, found, rows, verdicts)


def resolve(root: Path, rev: str) -> str:
    """`rev` with the commit it names, so every count below is anchored.

    ⛔ THE DEFAULT READS THE COMMITTED TREE, NOT THE WORKING ONE — `git ls-tree`
    and `git show` both answer for a revision, which is what makes `--as-of` the
    same question asked at another commit rather than a second code path. The
    consequence is worth printing rather than explaining: a run made while a
    commit is being prepared reports the tree as it stands at `HEAD`, so a figure
    quoted from it ages the moment that commit lands unless the revision travels
    with it (`docs/knowledge/a-metric-scoped-to-one-record-ages-silently.md`).
    """
    if rev == WORKTREE:
        return "the working tree (what the gate judges)"
    r = subprocess.run(["git", "rev-parse", "--short", rev],
                       cwd=root, capture_output=True, text=True)
    return f"{rev} ({r.stdout.strip()})" if r.returncode == 0 else rev


def report(root: Path, rev: str) -> int:
    s = survey(root, rev)
    if not s.paths:
        print(f"REFUSED: no tracked Markdown at {rev}", file=sys.stderr)
        return 2
    anchored = resolve(root, rev)

    counts = {k: sum(1 for v in s.verdicts.values() if v == k) for k in
              ("governed_reachable", "governed_by_hand", "unrouted_reachable",
               "invisible", "excluded_subject")}
    ungoverned = {p for p, v in s.verdicts.items() if v in ("invisible", "unrouted_reachable")}
    cover = minimal_cover(s.paths, ungoverned)
    root_level = [p for p in s.paths if "/" not in p]

    print(f"=== routing-closure census @ {anchored} — {len(s.paths)} tracked Markdown "
          f"documents, {len(s.rows)} registry rows, {len(s.tokens)} closure tokens ===")
    print(f"    anchors run: {', '.join(s.anchors_found)}\n")

    print("REACH")
    print(f"  the closure proposes          {sum(1 for p in s.paths if closure_reach(p, s.tokens)):>4} "
          f"of {len(s.paths)} tracked documents")
    for key, label in (
        ("governed_reachable", "row, and the closure names it"),
        ("governed_by_hand", "row, and the closure CANNOT name it"),
        ("unrouted_reachable", "no row, closure names it (the gate is red)"),
        ("invisible", "no row, and the closure CANNOT name it"),
        ("excluded_subject", "the guard's own subject, excluded by its anchor"),
    ):
        print(f"  {key:<20} {counts[key]:>4}   {label}")
    print()

    hand = sorted(r.path for r in s.rows
                  if not any(t == r.path or t.startswith(r.path) for t in s.tokens))
    print(f"ROWS THE CLOSURE NEVER NAMES — {len(hand)} of {len(s.rows)}")
    print("  (each exists because a person noticed; nothing here would have asked)")
    for path in hand:
        print(f"    {path}")
    print()

    print(f"UNGOVERNED POPULATION — {len(ungoverned)} documents, "
          f"{len(cover)} minimal covering terminals")
    print("  ⭐ the cost of a rule is the TERMINAL count, not the document count:")
    print("     the registry governs by prefix, so a wholly-ungoverned directory")
    print("     needs one row.")
    for terminal in cover:
        n = sum(1 for p in s.paths if p.startswith(terminal)) if terminal.endswith("/") else 1
        print(f"    {terminal:<40} covers {n:>3}")
    print()

    print(f"ROOT-LEVEL TRACKED MARKDOWN — {len(root_level)} documents "
          "(the leaf's own question)")
    for path in root_level:
        row = governing_row(path, s.rows)
        mark = {"governed_reachable": "  ", "governed_by_hand": "🔴",
                "unrouted_reachable": "⚠️", "invisible": "🔴",
                "excluded_subject": "⚪"}[s.verdicts[path]]
        print(f"  {mark} {path:<28} {s.verdicts[path]:<20} "
              f"row={'yes' if row else 'NONE'}")
    print()

    print("CANDIDATE SECOND ANCHORS, scored against the two known instances")
    candidates = {
        "A root-markdown": {p for p in s.paths if "/" not in p},
        "B bootstrap-list": bootstrap_tokens(root, rev),
        "C all-tracked-markdown": set(s.paths),
    }
    for name, anchor in candidates.items():
        verdicts = [f"{path}: {anchor_verdict(anchor, path)}" for path, _, _ in INSTANCES]
        new_terminals = minimal_cover(
            s.paths, {p for p in ungoverned if p in anchor})
        print(f"  {name:<24} would demand {len(new_terminals):>2} new row(s); "
              f"{'; '.join(verdicts)}")
    print()
    print("  ⚠️ An anchor's score here is its reach at THIS revision. The")
    print("     falsification is `--as-of <rev>` at each instance's own commit,")
    print("     where the document was genuinely ungoverned.")
    return 0


def anchor_check(root: Path, rev: str) -> int:
    """Today's anchor against a past tree: what would it have refused there?

    ⛔ THE REGISTRY IS THAT REVISION'S TOO. Scoring the new rule against an old
    file list but today's rows would credit the anchor with rows this repository
    only has because somebody noticed by hand — the exact thing it exists to make
    unnecessary. Both halves come from `rev`; only the RULE comes from here.
    """
    s = survey(root, rev, guard_rev=WORKTREE)
    if LISTING_ANCHOR not in s.anchors_found:
        print(f"REFUSED: the working tree's guard has no {LISTING_ANCHOR} anchor to apply.",
              file=sys.stderr)
        return 2
    refused = sorted(p for p, v in s.verdicts.items() if v in ("invisible", "unrouted_reachable"))
    print(f"=== today's anchor against {resolve(root, rev)} — "
          f"{len(s.paths)} tracked documents, {len(s.rows)} registry rows at that commit ===")
    print(f"  the anchor would refuse {len(refused)} destination(s) "
          f"({len(minimal_cover(s.paths, set(refused)))} covering terminals)")
    # 🔴 SCORED AT ITS OWN REVISION, and the first version of this arm was not.
    # It required BOTH instances to be refused at EVERY revision, so it reported
    # a failure at `8aadac3` — where `DEV_NOTES.md` already had a row, added one
    # lane earlier, and not refusing it is the anchor behaving correctly. A
    # control that cannot tell "the rule missed it" from "there was nothing to
    # miss" is measuring the calendar, not the rule.
    required = [(path, owner) for path, at, owner in INSTANCES if at == rev]
    context = [(path, at, owner) for path, at, owner in INSTANCES if at != rev]
    for path, owner in required:
        hit = path in refused
        print(f"  {'✅' if hit else '⛔'} {path:<28} "
              f"{'REFUSED by name' if hit else 'NOT refused'} — SCORED "
              f"(ungoverned at this commit; routed by hand at {owner})")
    for path, at, owner in context:
        state = "refused" if path in refused else "already routed here"
        print(f"  ⚪ {path:<28} {state} — context only, its own commit is {at}")
    for path in refused[:6]:
        print(f"     · {path}")
    if len(refused) > 6:
        print(f"     · … and {len(refused) - 6} more")
    if not required:
        print("  ⚠️ no known instance is anchored at this revision, so nothing is scored.")
        return 0
    missed = [path for path, _ in required if path not in refused]
    if missed:
        print(f"  ⛔ the anchor does not catch {', '.join(missed)} at its own commit",
              file=sys.stderr)
        return 1
    return 0


def verify_closure(root: Path) -> int:
    """Prove the derived token set against the real gate's behaviour.

    🔴 IT SURVEYS THE WORKING TREE, AND SURVEYING `HEAD` WAS A REAL DEFECT THIS
    ARM CAUGHT IN ITSELF. The probe perturbs the CHECKOUT and runs the enforcer
    over the CHECKOUT; taking the token set from `HEAD` meant the prediction and
    the measurement came from two different trees, and the moment the second
    anchor was added to the checkout the arm reported the gate as disagreeing
    with a closure that no longer existed.
    """
    s = survey(root, WORKTREE)
    print("=== closure verification — one row removed, real enforcer, restored ===")
    base_rc, _ = enforcer(root)
    print(f"  baseline: enforcer rc={base_rc} on the unmodified tree")
    if base_rc != 0:
        print("  REFUSED: the tree is already red, so neither arm below would be a\n"
              "  property of the row it names. This control measures a DIFFERENCE.")
        return 2

    named = sorted(r.path for r in s.rows
                   if any(t == r.path or t.startswith(r.path) for t in s.tokens)
                   and not r.path.endswith("/"))
    hand = sorted(r.path for r in s.rows
                  if not any(t == r.path or t.startswith(r.path) for t in s.tokens)
                  and not r.path.endswith("/"))
    if not named:
        print("  REFUSED: no row lies inside the closure, so arm A has no subject.")
        return 2

    # ⛔ THE VERDICT IS THE CLOSURE LEG'S, NEVER THE TREE'S EXIT CODE. Removing a
    # row perturbs every check that reads the registry, so `rc` and the other
    # failing doctrines are REPORTED — they say what else the row carries — and
    # never scored.
    # ⭐ ARM B RETIRES ITSELF WHEN THE DEFECT IS REPAIRED, and says so rather than
    # refusing. It demonstrates that a row OUTSIDE the closure can be deleted
    # unnoticed; once the second anchor makes every row reachable there is no
    # such row, and that absence is the repair rather than a broken control. The
    # historical demonstration moves to `--anchor-check <rev>`.
    arms = [("ARM A", named[0], True)]
    if hand:
        arms.append(("ARM B", hand[0], False))

    ok = True
    for label, row_path, want in arms:
        rc, output, restored = drop_row_and_run(root, row_path)
        refused = closure_refusal(output, row_path)
        others = [d for d in failing_doctrines(output)]
        sense = "the closure NAMES it" if want else "the closure CANNOT name it"
        print(f"  {label}  drop `{row_path}` ({sense})")
        print(f"         closure-leg refusal={refused} (wanted {want})  "
              f"rc={rc}  restored={'byte-identical' if restored else '⛔ NO'}")
        if others:
            print(f"         other doctrines red: {', '.join(others)} — what else the row carries")
        if refused is not want or not restored:
            ok = False
            print("         ⛔ the closure leg did not behave as the derived token set predicts")

    if not hand:
        print("  ARM B  no subject: every registry row is inside the closure")
        print("         ⭐ that is the repair, not a gap — the condition this arm")
        print("         demonstrates no longer exists here. Falsify it against the trees")
        print("         where it did: --anchor-check 386aa64^ and --anchor-check 8aadac3.")

    print()
    if ok:
        print("  ⭐ THE ARMS HOLD: the derived closure is the gate's closure. A row inside")
        print("     it cannot be removed without README-STABILITY naming it.")
        return 0
    print("  ⛔ the derivation disagrees with the gate", file=sys.stderr)
    return 1


def probe_bounds(root: Path, payload: int) -> int:
    """Is a no-row root document bounded by ANYTHING? With derived controls.

    ⛔ THE CONTROLS ARE DERIVED FROM THE REGISTRY, NOT CHOSEN. The first version
    of this arm took "a root document that has a row" as its positive control and
    got `AGENTS.md`, which has a row and is bounded by nothing — a control that
    cannot come apart from the thing it controls (`.11.4.2.6.4`). The two
    controls below are selected by what the registry DECLARES:

      - a row declaring `ceiling=<bytes>` MUST refuse — the positive control;
      - a row declaring no ceiling is expected NOT to, which is the finding that
        a row is a governance record and not, by itself, a size bound.
    """
    s = survey(root, WORKTREE)
    root_level = [p for p in s.paths if "/" not in p]
    no_row = [p for p in root_level if governing_row(p, s.rows) is None]
    with_ceiling = sorted(r.path for r in s.rows if "ceiling=" in r.assertions
                          and not r.path.endswith("/"))
    without_ceiling = sorted(
        path for path in root_level
        if (row := governing_row(path, s.rows)) is not None
        and "ceiling=" not in row.assertions
    )
    print(f"=== bound probe — +{payload:,} bytes, real enforcer, restored ===")
    base_rc, _ = enforcer(root)
    if base_rc != 0:
        print("  REFUSED: the enforcer is already red on the unmodified tree.")
        return 2
    print(f"  baseline: enforcer green (rc={base_rc})")
    labels = {
        "bounded": "REFUSED (bounded — the refusal names this file)",
        "unbounded": "⛔ ACCEPTED (bounded by nothing)",
        "refused-for-another-file": "⚠️ REFUSED FOR ANOTHER FILE — no bound on this one",
    }

    def run(path: str) -> str:
        verdict, restored = probe_bound(root, path, payload)
        print(f"    {path:<28} {labels[verdict]:<50} "
              f"restored={'byte-identical' if restored else '⛔ NOT RESTORED'}")
        return verdict

    print("  no-row root documents — the population this census is about:")
    for path in no_row:
        run(path)
    print("  positive control — a row that DECLARES a ceiling (must refuse):")
    control = with_ceiling[0] if with_ceiling else None
    control_verdict = run(control) if control else None
    print("  discriminating control — a row that declares NO ceiling:")
    for path in without_ceiling[:1]:
        run(path)
    print()
    if control_verdict != "bounded":
        print("  ⛔ the positive control did not refuse — every verdict above is suspect",
              file=sys.stderr)
        return 1
    print("  ⭐ The controls discriminate: a declared ceiling refuses, and a row")
    print("     without one does not. So an absent row is NOT the same claim as an")
    print("     absent bound, and this arm reports the two separately.")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--as-of", default=WORKTREE, metavar="REV",
                    help=f"run the census over the tree at REV, or {WORKTREE} (the default) "
                         "for the checkout the gate actually judges — a past REV is the "
                         "falsification leg")
    ap.add_argument("--anchor-check", metavar="REV",
                    help="apply the WORKING TREE's anchor to the tree at REV — the "
                         "falsification: would the rule being shipped have caught the "
                         "instances that were found by hand?")
    ap.add_argument("--verify-closure", action="store_true",
                    help="prove the derived closure against the real gate's behaviour")
    ap.add_argument("--probe-bounds", action="store_true",
                    help="append real bytes to each no-row root document and run "
                         "the real enforcer, with a positive control")
    ap.add_argument("--probe-bytes", type=int, default=200_000)
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    root = repo_root()
    if args.anchor_check:
        return anchor_check(root, args.anchor_check)
    if args.verify_closure:
        return verify_closure(root)
    if args.probe_bounds:
        return probe_bounds(root, args.probe_bytes)
    return report(root, args.as_of)


if __name__ == "__main__":
    sys.exit(main())
