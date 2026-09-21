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

⭐ THE CENSUS CAME FIRST AND THE SCHEME FOLLOWED IT (`.11.4.2.6.7`). Of the
twenty rows, **14 carry a machine-expressible claim and 6 are narrative**, so the
scheme is for fourteen rows rather than the three `.11.4.2.6.7` feared. Those
rows now DECLARE their claims in an optional fifth registry field, and `--check`
is the `ROUTE-CONTROL` gate that evaluates every one of them each commit.

⛔ DECLARED, NEVER EXTRACTED — and that is a measurement, not a preference.
`--extraction-control` runs the obvious alternative, pulling doctrine-shaped
tokens and byte-shaped numbers straight out of the prose, and scores it: **5 of
18 operands are not claims at all** (two task-tree names read as doctrine ids,
three HISTORICAL byte figures read as ceilings) while **6 of the 19 real claims
are invisible to it**. Wrong in both directions at once.

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

⭐ AND THE HAND READING IS KEPT HONEST BY DOUBLE-ENTRY. `ADJUDICATION` is what a
person read out of each sentence; the fifth field is what the row declares to a
checker. `--check` refuses when they disagree in EITHER direction — a term the
prose states and the row drops (so the field cannot be emptied to go green), and
a term the row declares that no reading supports (so a term cannot be invented).

    python3 -B scripts/census_route_controls.py                    # the census
    python3 -B scripts/census_route_controls.py --check            # ROUTE-CONTROL
    python3 -B scripts/census_route_controls.py --check --as-of REV
    python3 -B scripts/census_route_controls.py --classes
    python3 -B scripts/census_route_controls.py --extraction-control
    python3 -B scripts/census_route_controls.py --shapes
    python3 -B scripts/census_route_controls.py --self-test

⚠️ `--as-of` exists for ONE purpose: putting a row's claim back against the tree
that refuted it. `--check --as-of 9221467` returns rc=1 naming `LIVE_STATUS.md`,
whose growth claim was false at that commit, while the two sibling ledgers'
identical claims still hold — a control that comes apart from its subject rather
than failing wholesale at an old commit.

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
    History,
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
KINDS = ("ceiling", "doctrine", "growth", "index_entry", "guard_required",
         "build_target", "identity")

# The growth shapes `shape_verdict` produces. ⛔ Quoted from that function rather
# than restated: it is the producer, and a second list here could drift from it.
GROWTH_SHAPES = ("append_only", "at_all_time_high", "below_peak")

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
    # ⭐ THIS ONE WAS THE CENSUS'S *expressible but unanchored* ROW. The kind was
    # always expressible — pin a revision and the comparison is one
    # `git cat-file` — but the sentence started the identity at *once the PHASE-0
    # tree closes*, which no machine can resolve, and the file had already
    # changed three days after that close. The row now names the revision, and
    # the clause quotes it, so prose and operand cannot drift apart.
    "KICKOFF.md": (
        Claim("identity", "anchored at 457d3a7", "457d3a7"),
    ),
    # ⚠️ The document that GOVERNS the resume pointer, and it had no row at all
    # until `.11.4.2.7.2` — the routing closure is anchored at what `README.md`
    # links, and the landing page does not link it. Same anchoring blind spot
    # that left `DEV_NOTES.md` ungoverned until `.11.4.2.5`.
    "MEMORY_ARCHITECTURE.md": (
        Claim("doctrine", "the MEMORY-ARCH doctrine enforces what it defines", "MEMORY-ARCH"),
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
    # 🔴 THE CENSUS'S ONE REFUTED CLAIM. The sentence said `TABLE-ARITY`, and no
    # doctrine of that id is registered — the enforcer carries
    # `TABLE-ARITY-RATCHET`, which does run over staged Markdown, so the control
    # was real and its NAME was wrong. Corrected in the sentence, not papered
    # over with a prefix match: loosening the comparison would hide the next one
    # (`SIGNOFF-REPAIR.11.27`).
    "docs/TASK_TREE.md": (
        Claim("doctrine", "TABLE-ARITY-RATCHET and TASK-TREE-OWNERSHIP doctrines enforce its shape",
              "TABLE-ARITY-RATCHET"),
        Claim("doctrine", "TABLE-ARITY-RATCHET and TASK-TREE-OWNERSHIP doctrines enforce its shape",
              "TASK-TREE-OWNERSHIP"),
    ),
    # ⛔ THE GROWTH CLAIM IS THE CLASS THE ORIGINAL DEFECT LIVED IN. This row once
    # declared `overwritten rather than appended` while 614 of its 632 versions
    # grew and the tip sat at its all-time high, and nothing read it. Each ledger
    # now asserts the shape its own sentence describes, and `shape_verdict` — the
    # function that produced the refutation — is the evaluator.
    "LIVE_STATUS.md": (
        Claim("ceiling", "Threshold 55,000 bytes", "55000"),
        Claim("doctrine", "enforced by LEDGER-RUNWAY", "LEDGER-RUNWAY"),
        Claim("growth", "620,448 bytes became 13,995", "below_peak"),
    ),
    "DEV_NOTES.md": (
        Claim("ceiling", "Threshold 76,000 bytes", "76000"),
        Claim("doctrine", "enforced by LEDGER-RUNWAY", "LEDGER-RUNWAY"),
        Claim("growth", "908,850 bytes to 37,874", "below_peak"),
    ),
    "CHANGELOG.md": (
        Claim("ceiling", "96,000-byte rotation threshold", "96000"),
        Claim("doctrine", "enforced by the README-STABILITY guard", "README-STABILITY"),
        Claim("growth", "rotation = git history", "below_peak"),
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
    # ── the second anchor's rows (`SIGNOFF-REPAIR.11.4.2.7.3.1`) ──────────────
    # Thirteen destinations that had no row for the life of the project, because
    # all three original anchors start at what `README.md` reaches. They are
    # adjudicated here on the same terms as every row above: a clause quoted
    # verbatim where the sentence makes a decidable claim, and an empty tuple
    # where it does not. ⛔ Six of the thirteen are NARRATIVE, and that is a
    # positive classification — writing a claim to fill the field is the defect
    # this double-entry exists to catch.
    "MEMORY.md": (
        Claim("doctrine", "the MEMORY-ARCH doctrine enforces the line and byte caps",
              "MEMORY-ARCH"),
    ),
    # ⚠️ ADJUDICATED NARRATIVE, AND THE SENTENCE SAYS WHY RATHER THAN STAYING
    # SILENT. Its control is an ADMISSION — nothing bounds this file's size —
    # which is not a claim a machine can evaluate, and the measurement behind it
    # is that its three spine peers are unbounded too. Declaring a ceiling to
    # give the field something to hold would be a threshold nobody derived
    # (`.11.6`), so the question is owned by a leaf instead.
    "DOCTRINE_ENFORCEMENT.md": (),
    ".doctrine/": (),
    "docs/knowledge/": (
        Claim("doctrine",
              "LESSON-PROMOTION requires a new dated `DEV_NOTES.md` lesson to land here",
              "LESSON-PROMOTION"),
        Claim("doctrine",
              "the KNOWLEDGE-MAP doctrine derives `KNOWLEDGE_MAP.md` from these sources",
              "KNOWLEDGE-MAP"),
        Claim("index_entry",
              "derives `KNOWLEDGE_MAP.md` from these sources and checks it in sync",
              "KNOWLEDGE_MAP.md"),
    ),
    "docs/runbooks/": (),
    "docs/evidence/": (
        Claim("index_entry",
              "one evidence bundle per file plus a `docs/evidence/INDEX.md` entry",
              "docs/evidence/INDEX.md"),
    ),
    "spec/": (
        Claim("index_entry", "indexed by `spec/README.md`", "spec/README.md"),
    ),
    "docs/TASK_TREE_README.md": (),
    "docs/ci.md": (),
    "docs/compatibility-matrix.md": (
        Claim("doctrine", "which the PROJECT-SPECIFIC doctrine runs every commit",
              "PROJECT-SPECIFIC"),
    ),
    "docs/risks.md": (),
    "docs/parking-lot.md": (),
    "deploy/": (
        Claim("build_target", "the binaries `make release` produces", "release"),
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


def eval_growth(root: Path, path: str, param: str, rev: str = "HEAD") -> tuple[bool | None, str]:
    """Does this file's history have the shape its row declares?

    ⛔ THE EVALUATOR IS `shape_verdict`, THE FUNCTION THAT PRODUCED THE
    REFUTATION. `LIVE_STATUS.md`'s row claimed *overwritten rather than
    appended*, and that function's own docstring says only `below_peak` is
    consistent with such a claim. Writing a second classifier here would let the
    gate and the measurement that found the defect disagree about it.

    `rev` asks the question as of a past commit, which is how the historical
    defect is put back in front of the shipped check.
    """
    if param not in GROWTH_SHAPES:
        return None, f"{param!r} is not a growth shape ({', '.join(GROWTH_SHAPES)})"
    history: History = measure_history(root, path, rev)
    if history.versions == 0:
        return None, f"{path} has no history at {rev}"
    got = shape_verdict(history.added, history.removed, history.current, history.hist_max)
    where = "" if rev == "HEAD" else f" as of {rev}"
    return got == param, (f"{history.versions} versions{where}: {history.grew} grew, "
                          f"{history.shrank} shrank, tip {history.current} against a peak of "
                          f"{history.hist_max} -> {got}")


EVALUATORS = {
    "ceiling": eval_ceiling,
    "doctrine": eval_doctrine,
    "growth": eval_growth,
    "index_entry": eval_index_entry,
    "guard_required": eval_guard_required,
    "build_target": eval_build_target,
    "identity": eval_identity,
}

# Only these kinds mean anything about a PAST tree. `--as-of` says so rather than
# quietly evaluating a doctrine registry that only exists at HEAD.
HISTORICAL_KINDS = ("growth",)


def parse_classes(registry_text: str) -> list[str]:
    """The lifecycle vocabulary, DERIVED from the registry's own header comment.

    ⛔ NOT A LIST IN THIS FILE. The header states the vocabulary — `# classes: a |
    b |` continued over a second comment line — and it is the contract a row's
    author reads. A second copy in Python would be one edit away from disagreeing
    with the sentence that taught the author the word, which is the drift
    `SCAFFOLD-COVERAGE` exists because of.

    ⚠️ The continuation is part of the grammar, not an accident of wrapping: a
    parser that read only the `classes:` line would silently lose the three words
    on the line beneath and start refusing rows that use them.
    """
    names: list[str] = []
    collecting = False
    for line in registry_text.splitlines():
        if not line.startswith("#"):
            break
        body = line.lstrip("#").strip()
        if not collecting:
            if not body.startswith("classes:"):
                continue
            collecting = True
            body = body[len("classes:"):].strip()
        # ⭐ THE TRAILING PIPE IS THE CONTINUATION MARKER, and the first version of
        # this parser guessed instead — it continued onto any comment line that
        # happened to contain a pipe, which works on today's header only because
        # its second line has some. A fixture whose continuation carried a single
        # bare word caught it before the number shipped.
        more = body.endswith("|")
        for token in body.split("|"):
            token = token.strip()
            if token:
                names.append(token)
        if not more:
            break
    return names


def class_verdict(klass: str, vocabulary: list[str]) -> str:
    """`ok` when the declared lifecycle is a word the registry's header defines.

    ⛔ The registry validated this field for NON-EMPTINESS alone until
    `SIGNOFF-REPAIR.11.4.2.8`, exactly as it validated the control sentence
    before `.11.4.2.6.7`. A typo in a class name was a silent reclassification.
    """
    return "ok" if klass in vocabulary else "unknown"


def class_consistency(rows: list[RouteRow]) -> dict[str, dict]:
    """Per class: its rows, and the assertion kinds they declare.

    ⛔ THIS ANSWERS A QUESTION, IT DOES NOT ENFORCE ONE. Whether a class OUGHT to
    imply a mechanism is decided on this measurement (`.11.6`), not assumed by
    the shape of the table.
    """
    out: dict[str, dict] = {}
    for row in rows:
        entry = out.setdefault(row.klass, {"paths": [], "kinds": [], "narrative": 0})
        kinds = sorted({k for k, _ in parse_assertions(row.assertions)[0]})
        entry["paths"].append(row.path)
        entry["kinds"].append(kinds)
        if not kinds:
            entry["narrative"] += 1
    for entry in out.values():
        shared = set(entry["kinds"][0]) if entry["kinds"] else set()
        for kinds in entry["kinds"][1:]:
            shared &= set(kinds)
        entry["shared"] = sorted(shared)
    return out


def parse_assertions(field: str) -> tuple[list[tuple[str, str]], list[str]]:
    """The fifth field → ([(kind, operand)], [malformed terms]).

    ⛔ DECLARED, NEVER EXTRACTED. A census of these twenty controls measured the
    obvious alternative — pull doctrine-shaped tokens and byte-shaped numbers out
    of the prose — at 5 wrong operands in 18, with 6 of the 19 real claims
    invisible to it (`SIGNOFF-REPAIR.11.4.2.6.7.1`). A term a row did not write
    is a term nobody adjudicated.

    Malformed terms are RETURNED rather than skipped: a typo that silently
    vanishes is a declared control nobody reads, which is the defect this whole
    scheme exists to end.
    """
    good: list[tuple[str, str]] = []
    bad: list[str] = []
    for term in field.split():
        kind, sep, operand = term.partition("=")
        if not sep or not kind or not operand or kind not in KINDS:
            bad.append(term)
        else:
            good.append((kind, operand))
    return good, bad


def adjudicated_terms(claims: tuple[Claim, ...]) -> list[tuple[str, str]]:
    """The (kind, operand) pairs the hand reading of the prose produced."""
    return sorted((c.kind, c.param) for c in claims if c.param)


def declaration_drift(adjudged: list[tuple[str, str]],
                      declared: list[tuple[str, str]]) -> tuple[list, list]:
    """(adjudicated but not declared, declared but not adjudicated).

    ⭐ THIS IS DOUBLE-ENTRY, NOT A SECOND COPY. One side is a hand reading of the
    sentence; the other is what the row declares to a checker. They are produced
    by different acts, and their agreement is the property being asserted — a row
    whose terms are edited without re-reading its prose comes apart here.
    """
    a, d = sorted(set(adjudged)), sorted(set(declared))
    return [x for x in a if x not in d], [x for x in d if x not in a]


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


def classes_census(root: Path) -> int:
    """Does a declared LIFECYCLE predict anything about its destination?

    The pressure field got this treatment at `.11.4.2.6.7.1` before a scheme was
    designed for it. This is the same question one field to the left, and the
    answer is a measurement.
    """
    registry_text = (root / REGISTRY).read_text()
    vocabulary = parse_classes(registry_text)
    rows = load_rows(root)
    cc = class_consistency(rows)
    unused = [c for c in vocabulary if c not in cc]

    print(f"=== the lifecycle field — {len(rows)} rows over a {len(vocabulary)}-word "
          f"vocabulary derived from the registry's own header ===\n")
    consistent = 0
    for klass in sorted(cc, key=lambda k: -len(cc[k]["paths"])):
        entry = cc[klass]
        shared = entry["shared"]
        if shared:
            consistent += 1
        print(f"{klass:<22} rows={len(entry['paths']):<3} narrative={entry['narrative']:<3} "
              f"shared kinds={', '.join(shared) if shared else '⛔ none'}")
        for path, kinds in sorted(zip(entry["paths"], entry["kinds"])):
            print(f"    {path:<32} {', '.join(kinds) or '-narrative-'}")
        print()

    print(f"classes in use                     : {len(cc)} of {len(vocabulary)}")
    print(f"  whose rows share an assertion kind: {consistent}")
    for klass in unused:
        print(f"  ⚠️ declared and never used        : {klass}")
    print()
    print("  ⛔ A CLASS THAT SHARES NOTHING IS NOT THEREBY WRONG. This measures whether")
    print("     the field PREDICTS a mechanism, which is the question that decides")
    print("     whether it should bind one. It is not a defect count.")
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


def check(root: Path, rev: str = "HEAD") -> int:
    """ROUTE-CONTROL — every DECLARED assertion is evaluated, and a false one refuses.

    ⛔ It refuses on four things, and the last two are what stop the scheme being
    theatre: a malformed term, an assertion that does not hold, an assertion the
    prose reading found that the row does not declare, and an assertion the row
    declares that no reading of the prose supports. Without the third, the field
    could be emptied to make the gate green; without the fourth, a term could be
    invented that the sentence never claimed.

    `rev` restricts evaluation to the kinds that mean anything about a past tree
    and says which ones it skipped, rather than quietly judging a doctrine
    registry that exists only at HEAD.
    """
    rows = load_rows(root)
    fails = guard_adjudication(rows)
    historical = rev != "HEAD"
    evaluated = skipped = 0

    # The lifecycle field, refused on the same terms as a malformed assertion
    # term: a word the registry's own header does not define is not a
    # classification, it is a typo nobody adjudicated (`.11.4.2.8`).
    vocabulary = parse_classes((root / REGISTRY).read_text())
    if not vocabulary:
        print(f"ROUTE-CONTROL: REFUSED — {REGISTRY} declares no class vocabulary in its "
              f"header, so no row's lifecycle can be judged.", file=sys.stderr)
        return 2
    for row in rows:
        if class_verdict(row.klass, vocabulary) != "ok":
            print(f"ROUTE-CONTROL: {row.path} declares the lifecycle {row.klass!r}, which is "
                  f"not one of {', '.join(vocabulary)}.", file=sys.stderr)
            fails += 1

    for row in rows:
        declared, malformed = parse_assertions(row.assertions)
        for term in malformed:
            print(f"ROUTE-CONTROL: {row.path} declares {term!r}, which is not "
                  f"kind=operand with a kind in {', '.join(KINDS)}.", file=sys.stderr)
            fails += 1

        # ⛔ `.get`, NOT `[...]`. The guard above already REFUSED an unclassified
        #    row by name — and then this line raised `KeyError` on the same row,
        #    so the first real use of this gate printed a correct refusal followed
        #    by a traceback (`SIGNOFF-REPAIR.11.4.2.7.2`). An instrument must
        #    explain its own failure rather than crash after diagnosing it.
        missing, invented = declaration_drift(
            adjudicated_terms(ADJUDICATION.get(row.path, ())), declared)
        for kind, operand in missing:
            print(f"ROUTE-CONTROL: {row.path} — its control states {kind}={operand} and the row "
                  f"does not declare it. A term may not be dropped to make this gate green.",
                  file=sys.stderr)
            fails += 1
        for kind, operand in invented:
            print(f"ROUTE-CONTROL: {row.path} declares {kind}={operand}, which no reading of its "
                  f"control supports. Declare what the sentence claims, or change the sentence.",
                  file=sys.stderr)
            fails += 1

        for kind, operand in declared:
            if historical and kind not in HISTORICAL_KINDS:
                skipped += 1
                continue
            evaluator = EVALUATORS[kind]
            verdict, detail = (evaluator(root, row.path, operand, rev)
                               if kind in HISTORICAL_KINDS else
                               evaluator(root, row.path, operand))
            evaluated += 1
            if verdict is None:
                print(f"ROUTE-CONTROL: {row.path} declares {kind}={operand} and it cannot be "
                      f"evaluated — {detail}. A declared operand must be checkable.",
                      file=sys.stderr)
                fails += 1
            elif not verdict:
                print(f"ROUTE-CONTROL: {row.path} declares {kind}={operand} and IT IS FALSE — "
                      f"{detail}. Repair the destination or withdraw the claim; a control the "
                      f"registry states and nothing holds is the defect this gate exists for.",
                      file=sys.stderr)
                fails += 1

    if fails:
        print(f"ROUTE-CONTROL: {fails} declared-control breach(es) in {REGISTRY}.", file=sys.stderr)
        return 1
    where = "" if not historical else f" as of {rev} ({skipped} non-historical skipped)"
    print(f"ROUTE-CONTROL: OK — {evaluated} declared assertion(s) evaluated across "
          f"{len(rows)} routed rows{where}; every one holds.")
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

    # The declared-term parser. ⛔ A malformed term is RETURNED, never skipped:
    # a typo that silently vanishes is a declared control nobody reads, which is
    # the whole defect.
    check("terms: a well-formed pair",
          parse_assertions("ceiling=55000"), ([("ceiling", "55000")], []))
    check("terms: several, whitespace separated",
          parse_assertions("ceiling=1 doctrine=X"),
          ([("ceiling", "1"), ("doctrine", "X")], []))
    check("terms: an empty field declares nothing", parse_assertions(""), ([], []))
    check("terms: a bare word is malformed", parse_assertions("ceiling"), ([], ["ceiling"]))
    check("terms: an unknown kind is malformed", parse_assertions("vibes=1"), ([], ["vibes=1"]))
    check("terms: an empty operand is malformed", parse_assertions("ceiling="), ([], ["ceiling="]))
    check("terms: a stray sixth field survives as one malformed term",
          parse_assertions("a=1|b=2"), ([], ["a=1|b=2"]))
    check("terms: the good and the bad are separated, not merged",
          parse_assertions("ceiling=5 junk"), ([("ceiling", "5")], ["junk"]))

    # Double-entry between the prose reading and the declaration, both directions.
    check("drift: agreement", declaration_drift([("a", "1")], [("a", "1")]), ([], []))
    check("drift: a term DROPPED from the row is caught",
          declaration_drift([("a", "1")], []), ([("a", "1")], []))
    check("drift: a term INVENTED by the row is caught",
          declaration_drift([], [("a", "1")]), ([], [("a", "1")]))
    check("drift: a CHANGED operand is caught in both directions",
          declaration_drift([("a", "1")], [("a", "2")]), ([("a", "1")], [("a", "2")]))
    check("adjudicated terms skip an unanchored claim",
          adjudicated_terms((Claim("identity", "x", ""), Claim("ceiling", "y", "5"))),
          [("ceiling", "5")])

    # A growth operand outside the producer's vocabulary is unevaluable, not false.
    root0 = repo_root()
    check("growth: an operand that is not a shape is refused as unevaluable",
          eval_growth(root0, "CHANGELOG.md", "shrinking")[0], None)
    check("growth: the shapes are the producer's three",
          sorted(GROWTH_SHAPES), ["append_only", "at_all_time_high", "below_peak"])

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

    # 21-26. The lifecycle field (`SIGNOFF-REPAIR.11.4.2.8`).
    header = ("# format: path|class|control|owner|assertions\n"
              "# classes: alpha | beta |\n"
              "#          gamma\n"
              "# more prose that is not the vocabulary\n"
              "alpha.md|alpha|c|o|\n")
    # ⛔ The continuation line is the arm that matters: a parser reading only the
    #    `classes:` line loses `gamma` and starts refusing rows that use it.
    check("classes: vocabulary spans the continuation line",
          parse_classes(header), ["alpha", "beta", "gamma"])
    check("classes: no header, no vocabulary", parse_classes("a.md|x|y|z|\n"), [])
    check("classes: a declared word is ok", class_verdict("beta", ["alpha", "beta"]), "ok")
    check("classes: an undeclared word is not",
          class_verdict("Beta", ["alpha", "beta"]), "unknown")
    rows_fx = [RouteRow("a", "k", "c", "o", "doctrine=D index_entry=I"),
               RouteRow("b", "k", "c", "o", "doctrine=D"),
               RouteRow("c", "j", "c", "o", "")]
    cc = class_consistency(rows_fx)
    check("classes: shared kinds are the intersection", cc["k"]["shared"], ["doctrine"])
    check("classes: a narrative row is counted", cc["j"]["narrative"], 1)

    # 27. The real registry's own vocabulary is non-empty and covers every row —
    # the live-corpus arm, which a fixture cannot stand in for.
    real_voc = parse_classes((root / REGISTRY).read_text())
    check("shipped: every row's class is in the derived vocabulary",
          sorted({r.klass for r in real if r.klass not in real_voc}), [])

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
    ap.add_argument("--classes", action="store_true",
                    help="census the LIFECYCLE field: does a declared class predict a "
                         "mechanism? (SIGNOFF-REPAIR.11.4.2.8)")
    ap.add_argument("--shapes", action="store_true",
                    help="growth shape per routed file (context, not a claim)")
    ap.add_argument("--check", action="store_true",
                    help="ROUTE-CONTROL: evaluate every declared assertion; refuse on a false one")
    ap.add_argument("--as-of", metavar="REV", default="HEAD",
                    help="evaluate the history-based assertions against a past commit")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    root = repo_root()
    if args.check:
        return check(root, args.as_of)
    if args.classes:
        return classes_census(root)
    if args.extraction_control:
        return extraction_control(root)
    if args.shapes:
        return shapes(root)
    return census(root)


if __name__ == "__main__":
    sys.exit(main())
