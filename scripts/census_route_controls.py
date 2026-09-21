#!/usr/bin/env python3
"""Census the DECLARED pressure controls in the routed-destination registry
(`SIGNOFF-REPAIR.11.4.2.6.7.1`).

`.doctrine/readme_routes.txt` gives every routed destination a row of
`path|class|pressure control|owner`, and the third field is a free-text sentence
saying what holds that destination's growth. `scripts/check_readme_stability.sh`
reads it for exactly ONE purpose — extracting path tokens so they can be
required to end at a governed row — and validates the field itself only for
NON-EMPTINESS. Whether the sentence is TRUE is never asked.

🔴 That is not a theoretical gap. `LIVE_STATUS.md`'s row declared *overwritten
rather than appended* while its own 632 versions showed 614 growing against 17
shrinking with the tip at its all-time high, and the guard was green throughout,
because four non-empty fields is arity rather than truth.

⛔ THIS INSTRUMENT PROPOSES NO SCHEME AND SHIPS NO GATE. `.11.4.2.6.7` may not
design one before its population is counted (`SIGNOFF-REPAIR.11.6`), and its own
constraint is that *a scheme that fits 3 of 20 is a scheme for 3 rows*. This
counts, and the count is the input to that decision.

⛔ THE CLASSIFICATION IS A JUDGEMENT OVER PROSE AND IS CARRIED AS DATA. Deciding
whether a sentence makes a machine-evaluable claim is not itself mechanizable —
that is the same error one layer down. What IS mechanized is everything around
the judgement, and each guard exists because the hand-written half is the part
that rots:

  - `ADJUDICATION` must cover EXACTLY the registry's paths, so a row added or
    removed refuses the census instead of being silently unclassified;
  - every adjudicated clause must be a VERBATIM SUBSTRING of its row's control,
    so rewriting the sentence detaches the judgement rather than leaving a stale
    one attached to words that no longer exist;
  - every claim called evaluable is EVALUATED in the same run, so *expressible*
    is demonstrated rather than asserted.

⭐ AND THE OBVIOUS ALTERNATIVE IS RUN AS A CONTROL RATHER THAN DISMISSED.
`--extraction-control` runs the naive reading — pull doctrine-shaped tokens and
byte-shaped numbers straight out of the prose — and scores it against the
adjudication. It is wrong in both directions at once, which is the measured
reason an operand must be DECLARED by a row rather than extracted from it.

    python3 -B scripts/census_route_controls.py
    python3 -B scripts/census_route_controls.py --extraction-control
    python3 -B scripts/census_route_controls.py --shapes
    python3 -B scripts/census_route_controls.py --self-test

⚠️ `--shapes` is opt-in because it walks every version of every routed file
through the sibling census's `measure_history`, which costs one `git cat-file`
per version. It is the growth axis the historical defect lived on, and it is
reported as CONTEXT — no row declares a growth claim today.

Self-test: scripts/census_route_controls.py --self-test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

# ⛔ IMPORTED, NOT RE-IMPLEMENTED. `parse_route_registry` already matches the
# gate's MALFORMED shape and `shape_verdict` is the function that produced the
# refutation this leaf exists because of. A second copy of either would be the
# defect this repository keeps repairing — most recently at SCAFFOLD-COVERAGE,
# where a list nothing derived had drifted by seven checks.
from census_live_documents import (  # noqa: E402
    RouteRow,
    measure_history,
    parse_doctrine_registry,
    parse_route_registry,
    shape_verdict,
)

REGISTRY = ".doctrine/readme_routes.txt"
ENFORCER = "scripts/check_doctrines.sh"
STABILITY_GUARD = "scripts/check_readme_stability.sh"
MAKEFILE = "Makefile"


def repo_root() -> Path:
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return Path(out.stdout.strip())


# ── the adjudication: a hand judgement, carried as data ───────────────────────

@dataclass(frozen=True)
class Claim:
    """One machine-expressible claim adjudicated out of one control sentence.

    `clause` is quoted VERBATIM from the control and is checked to still be a
    substring of it. `param` is the operand the clause names — empty when the
    sentence makes the claim without naming anything a machine can act on,
    which is itself a finding rather than a classification failure.
    """

    kind: str
    clause: str
    param: str


# Kinds, each with an evaluator below. The vocabulary was READ OFF the twenty
# sentences rather than chosen first: every kind here is present in at least one
# row, and no kind was added that no row uses.
KINDS = ("ceiling", "doctrine", "index_entry", "guard_required", "build_target", "identity")

# ⛔ ONE ENTRY PER REGISTRY PATH, INCLUDING THE NARRATIVE ONES. An empty tuple
# is the adjudication "this control makes no claim a machine could evaluate" and
# is a positive statement, not an omission — which is why coverage is exact-match
# against the registry rather than a subset test.
ADJUDICATION: dict[str, tuple[Claim, ...]] = {
    # The control names the command that produces the destination. A target that
    # disappears makes the sentence false, and `make` can answer whether it exists.
    "docs/book/": (
        Claim("build_target", "mdBook built by `make book`", "book"),
    ),
    # "frozen at v0.4.1" and "changes only per the errata rule" are both claims
    # about WHY a change happens. Nothing decidable: a machine can see that bytes
    # moved, never whether an errata rule authorized them.
    "ROADMAP.md": (),
    # "content identity once the PHASE-0 tree closes" IS an identity claim — the
    # kind is expressible — but the moment it starts from is prose. No revision
    # is named, so nothing can be compared. `param` is deliberately empty.
    "KICKOFF.md": (
        Claim("identity", "content identity once the PHASE-0 tree closes", ""),
    ),
    "CLAUDE.md": (
        Claim("doctrine", "MEMORY-ARCH doctrine verifies the pointer", "MEMORY-ARCH"),
    ),
    "AGENTS.md": (
        Claim("doctrine", "MEMORY-ARCH doctrine verifies the pointer", "MEMORY-ARCH"),
    ),
    # "no append pressure" is the nearest thing to a decidable clause here, and
    # it is a claim about the FUTURE rather than about the file. Its only
    # decidable reading is a growth shape, which `--shapes` reports as context.
    "docs/CLAIM_VERIFICATION.md": (),
    "SECURITY.md": (),
    # "the guard refuses when this file is absent" names a behaviour of a named
    # script, and that script's required-file list is readable.
    "README_POLICY.md": (
        Claim("guard_required", "the guard refuses when this file is absent", STABILITY_GUARD),
    ),
    "scripts/update_scaffold.sh": (),
    # "bounded index `docs/TASK_TREE.md` (one row per tree)" is an index-coverage
    # claim about this directory, answered by the index it names.
    "docs/tasks/": (
        Claim("index_entry", "bounded index `docs/TASK_TREE.md` (one row per tree)",
              "docs/TASK_TREE.md"),
    ),
    "docs/TASK_TREE.md": (
        Claim("doctrine", "TABLE-ARITY and TASK-TREE-OWNERSHIP doctrines enforce its shape",
              "TABLE-ARITY"),
        Claim("doctrine", "TABLE-ARITY and TASK-TREE-OWNERSHIP doctrines enforce its shape",
              "TASK-TREE-OWNERSHIP"),
    ),
    "LIVE_STATUS.md": (
        Claim("ceiling", "Threshold 55,000 bytes", "55000"),
        Claim("doctrine", "enforced by LEDGER-RUNWAY", "LEDGER-RUNWAY"),
    ),
    "DEV_NOTES.md": (
        Claim("ceiling", "Threshold 76,000 bytes", "76000"),
        Claim("doctrine", "enforced by LEDGER-RUNWAY", "LEDGER-RUNWAY"),
    ),
    "CHANGELOG.md": (
        Claim("ceiling", "96,000-byte rotation threshold", "96000"),
        Claim("doctrine", "enforced by the README-STABILITY guard", "README-STABILITY"),
    ),
    "COMMIT.md": (),
    # Declared and, as this census measures, enforced by nothing at all.
    "docs/adr/": (
        Claim("index_entry", "one ADR per file + `docs/adr/INDEX.md` entry", "docs/adr/INDEX.md"),
    ),
    "docs/decisions/": (
        Claim("index_entry", "one record per file + INDEX entry", "docs/decisions/INDEX.md"),
        Claim("doctrine", "MEMORY-ARCH enforces index sync", "MEMORY-ARCH"),
    ),
    "TOOLBOX.md": (),
    "KNOWLEDGE_MAP.md": (
        Claim("doctrine", "KNOWLEDGE-MAP doctrine checks sync against sources", "KNOWLEDGE-MAP"),
    ),
    "knowledge-map/": (
        Claim("doctrine", "KNOWLEDGE-MAP doctrine checks sync", "KNOWLEDGE-MAP"),
    ),
}


# ── pure verdicts (the --self-test arm is ground truth for every one) ─────────

def adjudication_drift(adjudicated: list[str], registry: list[str]) -> tuple[list[str], list[str]]:
    """(registry paths with no adjudication, adjudicated paths not in the registry).

    Exact match in both directions. A subset test would let a NEW row enter the
    registry unclassified, which is the state this census exists to end.
    """
    have = set(adjudicated)
    want = set(registry)
    return sorted(want - have), sorted(have - want)


def unquoted_clauses(rows: list[RouteRow],
                     adjudication: dict[str, tuple[Claim, ...]]) -> list[tuple[str, str]]:
    """Adjudicated clauses that are no longer verbatim in their row's control.

    ⛔ This is what stops the judgement outliving the sentence it judged. A
    control rewritten around its clause detaches the adjudication loudly instead
    of leaving it attached to words the row no longer contains.
    """
    control = {row.path: row.control for row in rows}
    stale: list[tuple[str, str]] = []
    for path, claims in sorted(adjudication.items()):
        text = control.get(path)
        if text is None:
            continue
        for claim in claims:
            if claim.clause not in text:
                stale.append((path, claim.clause))
    return stale


def unknown_kinds(adjudication: dict[str, tuple[Claim, ...]]) -> list[tuple[str, str]]:
    """Adjudicated claims whose kind has no evaluator."""
    return [(p, c.kind) for p, cs in sorted(adjudication.items()) for c in cs if c.kind not in KINDS]


def index_records(names: list[str], index_basename: str) -> list[str]:
    """The record files an index must cover, after the two standing exclusions.

    ⛔ The exclusions are QUOTED from the predicate `check_memory_architecture.sh`
    already applies to `docs/decisions/` — INDEX and TEMPLATE — rather than
    invented here, so this census and that gate cannot disagree about what a
    record is.
    """
    return sorted(n for n in names
                  if n.endswith(".md") and n not in {index_basename, "INDEX.md", "TEMPLATE.md"})


def naive_doctrine_tokens(control: str) -> list[str]:
    """The obvious reading: anything shaped like a doctrine id."""
    return [t for t in re.findall(r"\b[A-Z][A-Z0-9]+(?:-[A-Z0-9]+)+\b", control)
            if not t.startswith("SIGNOFF")]


def naive_threshold_tokens(control: str) -> list[str]:
    """The obvious reading: anything shaped like a byte figure."""
    found: list[str] = []
    for tup in re.findall(r"([0-9][0-9,]{3,})[- ]byte|Threshold ([0-9,]+) bytes", control):
        for tok in tup:
            if tok:
                found.append(tok.replace(",", ""))
    return found


def extraction_scorecard(adjudicated: list[str],
                         extracted: list[str]) -> tuple[list[str], list[str], list[str]]:
    """(agreed, extracted-but-not-a-claim, claimed-but-not-extractable).

    Multiplicity is preserved on purpose: the same doctrine id declared by three
    rows is three claims, and collapsing them would flatter both readings.
    """
    agreed: list[str] = []
    false_positive: list[str] = []
    remaining = list(adjudicated)
    for tok in extracted:
        if tok in remaining:
            remaining.remove(tok)
            agreed.append(tok)
        else:
            false_positive.append(tok)
    return agreed, false_positive, sorted(remaining)


# ── evaluators: one per kind, each answering the claim it was adjudicated from ─

def eval_ceiling(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    target = root / path
    if not target.is_file():
        return None, f"{path} is not a file"
    size = target.stat().st_size
    cap = int(param)
    return size <= cap, f"{size} bytes against a {cap}-byte ceiling"


def eval_doctrine(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    registry = dict(parse_doctrine_registry((root / ENFORCER).read_text()))
    if param not in registry:
        near = sorted(i for i in registry if i.startswith(param) or param.startswith(i))
        hint = f"; closest registered id: {', '.join(near)}" if near else ""
        return False, f"no doctrine {param} is registered by {ENFORCER}{hint}"
    return True, f"{param} is registered, run by {registry[param]}"


def eval_index_entry(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    directory = root / path
    index = root / param
    if not directory.is_dir():
        return None, f"{path} is not a directory"
    if not index.is_file():
        return False, f"the declared index {param} does not exist"
    records = index_records([p.name for p in directory.iterdir()], Path(param).name)
    text = index.read_text()
    missing = [r for r in records if r not in text]
    if missing:
        return False, f"{len(missing)} of {len(records)} records absent from {param}: " \
                      f"{', '.join(missing[:3])}"
    return True, f"{len(records)} records, all named in {param}"


def eval_guard_required(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    """Is this path in the named guard's refuse-rather-than-skip list?

    The list is written with shell variables, so the variable's own assignment
    is resolved before the membership test — reading the loop literally would
    answer about the word `POLICY` rather than about the file.
    """
    script = root / param
    if not script.is_file():
        return None, f"{param} does not exist"
    text = script.read_text()
    loop = re.search(r"^for required in (.+); do$", text, re.MULTILINE)
    if not loop:
        return False, f"{param} has no refuse-rather-than-skip loop"
    names: list[str] = []
    for tok in re.findall(r'"\$([A-Z_]+)"', loop.group(1)):
        assign = re.search(rf'^{tok}="([^"]+)"$', text, re.MULTILINE)
        if assign:
            names.append(assign.group(1))
    ok = path in names
    return ok, f"{param} refuses without: {', '.join(names) or '(none resolved)'}"


def eval_build_target(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    makefile = root / MAKEFILE
    if not makefile.is_file():
        return None, f"{MAKEFILE} does not exist"
    present = re.search(rf"^{re.escape(param)}:", makefile.read_text(), re.MULTILINE) is not None
    return present, f"{MAKEFILE} {'defines' if present else 'does not define'} the {param} target"


def eval_identity(root: Path, path: str, param: str) -> tuple[bool | None, str]:
    """An identity claim with no anchor cannot be evaluated, and says so.

    ⚠️ The kind IS expressible — pin a revision and the comparison is one
    `git cat-file`. What is missing is the operand, and reporting that as
    *narrative* would hide a row that is one field away from being checkable.
    """
    if not param:
        return None, "the control names no revision to compare against"
    out = subprocess.run(["git", "cat-file", "-s", f"{param}:{path}"],
                         cwd=root, capture_output=True, text=True)
    if out.returncode != 0:
        return None, f"{param}:{path} is not readable"
    size = (root / path).stat().st_size
    return size == int(out.stdout.strip()), f"{size} bytes against {param}'s {out.stdout.strip()}"


EVALUATORS = {
    "ceiling": eval_ceiling,
    "doctrine": eval_doctrine,
    "index_entry": eval_index_entry,
    "guard_required": eval_guard_required,
    "build_target": eval_build_target,
    "identity": eval_identity,
}


# ── the census ────────────────────────────────────────────────────────────────

def load_rows(root: Path) -> list[RouteRow]:
    return parse_route_registry((root / REGISTRY).read_text())


def guard_adjudication(rows: list[RouteRow]) -> int:
    """Refuse the census when the hand-written half has come loose."""
    fails = 0
    unclassified, phantom = adjudication_drift(list(ADJUDICATION), [r.path for r in rows])
    for path in unclassified:
        print(f"REFUSED: {REGISTRY} row {path!r} has no adjudication — classify it before "
              f"the census can report a total.", file=sys.stderr)
        fails += 1
    for path in phantom:
        print(f"REFUSED: adjudicated path {path!r} is not a registry row.", file=sys.stderr)
        fails += 1
    for path, clause in unquoted_clauses(rows, ADJUDICATION):
        print(f"REFUSED: {path} — the adjudicated clause {clause!r} is no longer verbatim in "
              f"its control; re-read the sentence before trusting the judgement.", file=sys.stderr)
        fails += 1
    for path, kind in unknown_kinds(ADJUDICATION):
        print(f"REFUSED: {path} — claim kind {kind!r} has no evaluator.", file=sys.stderr)
        fails += 1
    return fails


def census(root: Path) -> int:
    rows = load_rows(root)
    if guard_adjudication(rows) != 0:
        return 2

    by_kind: dict[str, int] = {k: 0 for k in KINDS}
    expressible_rows = 0
    narrative_rows = 0
    evaluable = 0
    unanchored = 0
    holds = 0
    refuted: list[str] = []

    print(f"=== declared pressure controls in {REGISTRY} ({len(rows)} rows) ===")
    for row in rows:
        claims = ADJUDICATION[row.path]
        if not claims:
            narrative_rows += 1
            print(f"  {row.path:28s} [{row.klass}] narrative — no machine-evaluable claim")
            continue
        expressible_rows += 1
        print(f"  {row.path:28s} [{row.klass}]")
        for claim in claims:
            by_kind[claim.kind] += 1
            verdict, detail = EVALUATORS[claim.kind](root, row.path, claim.param)
            if verdict is None:
                unanchored += 1
                mark = "⚠️  unevaluable"
            elif verdict:
                evaluable += 1
                holds += 1
                mark = "✅ holds"
            else:
                evaluable += 1
                refuted.append(f"{row.path} ({claim.kind} {claim.param or '—'})")
                mark = "🔴 REFUTED"
            operand = claim.param or "—"
            print(f"      {claim.kind:14s} {operand:24s} {mark}  {detail}")
            print(f"        quoted: {claim.clause!r}")

    total_claims = sum(by_kind.values())
    print()
    print(f"rows                       : {len(rows)}")
    print(f"  carrying >=1 claim       : {expressible_rows}")
    print(f"  narrative only           : {narrative_rows}")
    print(f"claims                     : {total_claims}")
    for kind in KINDS:
        print(f"  {kind:24s}: {by_kind[kind]}")
    print(f"evaluated now              : {evaluable} ({holds} hold, {len(refuted)} refuted)")
    print(f"expressible but unanchored : {unanchored}")
    for item in refuted:
        print(f"  🔴 refuted: {item}")
    return 0


def extraction_control(root: Path) -> int:
    """Score the naive reading against the adjudication.

    ⛔ THE POINT IS NOT THAT EXTRACTION IS IMPERFECT. It is wrong in BOTH
    directions at once — it invents operands the sentences do not declare and it
    cannot see whole kinds of claim — which is why `.11.4.2.6.7` must give a row
    a declared field rather than parse its prose.
    """
    rows = load_rows(root)
    if guard_adjudication(rows) != 0:
        return 2

    claimed_doctrines = [c.param for cs in ADJUDICATION.values() for c in cs if c.kind == "doctrine"]
    claimed_ceilings = [c.param for cs in ADJUDICATION.values() for c in cs if c.kind == "ceiling"]
    got_doctrines = [t for r in rows for t in naive_doctrine_tokens(r.control)]
    got_ceilings = [t for r in rows for t in naive_threshold_tokens(r.control)]

    d_ok, d_bad, d_missed = extraction_scorecard(claimed_doctrines, got_doctrines)
    c_ok, c_bad, c_missed = extraction_scorecard(claimed_ceilings, got_ceilings)

    invisible = [f"{p} ({c.kind})" for p, cs in sorted(ADJUDICATION.items()) for c in cs
                 if c.kind not in ("doctrine", "ceiling")]

    extracted = len(got_doctrines) + len(got_ceilings)
    agreed = len(d_ok) + len(c_ok)
    wrong = len(d_bad) + len(c_bad)
    print("=== control: the naive reading, scored against the adjudication ===")
    print(f"  doctrine-shaped tokens extracted : {len(got_doctrines)} "
          f"({len(d_ok)} are claims, {len(d_bad)} are not)")
    for tok in d_bad:
        print(f"      🔴 not a claim: {tok}")
    print(f"  byte-shaped numbers extracted    : {len(got_ceilings)} "
          f"({len(c_ok)} are claims, {len(c_bad)} are not)")
    for tok in c_bad:
        print(f"      🔴 not a claim: {tok}")
    print(f"  claims of these two kinds missed : {len(d_missed) + len(c_missed)}")
    print(f"  claims of OTHER kinds, invisible to any such extractor : {len(invisible)}")
    for item in invisible:
        print(f"      ⚠️  {item}")
    rate = (wrong / extracted * 100) if extracted else 0.0
    print()
    print(f"extracted operands {extracted}, agreeing with the adjudication {agreed}, "
          f"wrong {wrong} ({rate:.1f}%)")
    print(f"adjudicated claims {sum(len(cs) for cs in ADJUDICATION.values())}, "
          f"reachable by extraction at all {len(claimed_doctrines) + len(claimed_ceilings)}")
    return 0


def shapes(root: Path) -> int:
    """The growth axis, reported as CONTEXT rather than as a claim.

    No row declares a growth shape today — `LIVE_STATUS.md`'s did, and it was
    the refuted one. This arm exists so the rows that say *frozen* or *no append
    pressure* can be read against what their files actually did.
    """
    rows = load_rows(root)
    print("=== growth shape per routed file (context; no row declares one) ===")
    for row in rows:
        if row.path.endswith("/"):
            print(f"  {row.path:28s} directory — no single-file shape")
            continue
        history = measure_history(root, row.path)
        if history.versions == 0:
            print(f"  {row.path:28s} no history")
            continue
        verdict = shape_verdict(history.added, history.removed, history.current, history.hist_max)
        print(f"  {row.path:28s} v={history.versions:4d} grew={history.grew:4d} "
              f"shrank={history.shrank:3d} cur={history.current:7d} "
              f"max={history.hist_max:7d} -> {verdict}")
    return 0


# ── self-test ────────────────────────────────────────────────────────────────

def self_test() -> int:
    controls = 0
    failures: list[str] = []

    def check(name: str, got, want) -> None:
        nonlocal controls
        controls += 1
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    # 1-3. Drift is reported in BOTH directions, and an exact cover is clean.
    check("drift: clean cover", adjudication_drift(["a", "b"], ["a", "b"]), ([], []))
    check("drift: registry row unclassified", adjudication_drift(["a"], ["a", "b"]), (["b"], []))
    check("drift: adjudication names a phantom", adjudication_drift(["a", "b"], ["a"]), ([], ["b"]))

    # 4-5. A clause must be verbatim in the control it was read from.
    rows = [RouteRow("x.md", "hot_live", "Threshold 55,000 bytes, derived", "me")]
    check("clause: verbatim passes",
          unquoted_clauses(rows, {"x.md": (Claim("ceiling", "Threshold 55,000 bytes", "55000"),)}),
          [])
    check("clause: paraphrase is caught",
          unquoted_clauses(rows, {"x.md": (Claim("ceiling", "Threshold 55000 bytes", "55000"),)}),
          [("x.md", "Threshold 55000 bytes")])

    # 6. A kind with no evaluator is refused rather than silently skipped.
    check("kind: unknown kind caught",
          unknown_kinds({"x.md": (Claim("vibes", "whatever", ""),)}), [("x.md", "vibes")])
    check("kind: every shipped kind has an evaluator", sorted(EVALUATORS), sorted(KINDS))

    # 8-10. The index-record exclusions, which are the gate's own, not new ones.
    check("index: excludes INDEX and TEMPLATE",
          index_records(["INDEX.md", "TEMPLATE.md", "001-a.md", "002-b.md"], "INDEX.md"),
          ["001-a.md", "002-b.md"])
    check("index: excludes a differently named index",
          index_records(["TASK_TREE.md", "PHASE-0.md"], "TASK_TREE.md"), ["PHASE-0.md"])
    check("index: non-markdown is not a record",
          index_records(["a.md", "b.txt", "notes"], "INDEX.md"), ["a.md"])

    # 11-13. The naive extractors, including the exact false positives measured
    # on the real registry: a task-tree name reads as a doctrine id, and a
    # historical byte figure reads as a ceiling.
    check("naive: doctrine shape",
          naive_doctrine_tokens("MEMORY-ARCH verifies it; PHASE-0 closed; SIGNOFF-REPAIR.1 owns"),
          ["MEMORY-ARCH", "PHASE-0"])
    check("naive: threshold shape",
          naive_threshold_tokens("Threshold 76,000 bytes, was 908,850 bytes and a 96,000-byte cap"),
          ["76000", "908850", "96000"])
    check("naive: no shapes at all", naive_doctrine_tokens("plain prose, no ids"), [])

    # 14-16. The scorecard keeps multiplicity and separates the two error kinds.
    check("scorecard: agreement",
          extraction_scorecard(["A", "B"], ["A", "B"]), (["A", "B"], [], []))
    check("scorecard: a false positive",
          extraction_scorecard(["A"], ["A", "PHASE-0"]), (["A"], ["PHASE-0"], []))
    check("scorecard: a missed claim",
          extraction_scorecard(["A", "B"], ["A"]), (["A"], [], ["B"]))
    check("scorecard: multiplicity survives",
          extraction_scorecard(["A", "A"], ["A"]), (["A"], [], ["A"]))

    # 18. The imported verdict is the one that produced the refutation; a local
    # copy of it would be the drift this instrument refuses elsewhere.
    check("imported shape_verdict: at_all_time_high",
          shape_verdict(620448, 17, 620448, 620448), "at_all_time_high")

    # 19-20. The shipped adjudication covers the real registry exactly and every
    # clause is still verbatim — the two guards, run against the real tree.
    root = repo_root()
    real = load_rows(root)
    check("shipped: adjudication covers the registry exactly",
          adjudication_drift(list(ADJUDICATION), [r.path for r in real]), ([], []))
    check("shipped: every clause is still verbatim", unquoted_clauses(real, ADJUDICATION), [])

    if failures:
        for line in failures:
            print(f"census_route_controls --self-test FAILED: {line}", file=sys.stderr)
        return 1
    print(f"census_route_controls --self-test: {controls} controls pass")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--extraction-control", action="store_true",
                    help="score the naive prose reading against the adjudication")
    ap.add_argument("--shapes", action="store_true",
                    help="growth shape per routed file (walks every version; slow)")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    root = repo_root()
    if args.extraction_control:
        return extraction_control(root)
    if args.shapes:
        return shapes(root)
    return census(root)


if __name__ == "__main__":
    sys.exit(main())
