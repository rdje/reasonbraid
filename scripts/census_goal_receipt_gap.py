#!/usr/bin/env python3
"""GOAL-RECEIPT — census the tracked task trees for work a leaf PROMISED and
never RECEIPTED, and for deferrals handed to a leaf that has since closed.

⛔ WHY THIS EXISTS (`SIGNOFF-REPAIR.11.24`). Two `done` Phase-8 leaves were
found in consecutive slices whose GOAL line named work their own DONE list did
not carry, and the second one's deferral — *the transport rides `.3.4`* — is
why six implemented MCP tools are reachable by no client. Both leaves are
honest: neither Done list claims what it did not ship. The drift is between a
leaf's own two sections, and nothing in this repository compares them.
`TASK-ACCEPTANCE` reads the checklist, `FRONTIER-STATUS` the status line,
`INDEX-FRONTIER` the index. None of them reads the goal.

Three predicates, reported SEPARATELY because they cost different things to
believe:

  duplicate-leaf-id     EXACT. Two leaves sharing one address, in a tree where
                        the frontier, the index and every cross-reference
                        address leaves by id. No judgement.
  stranded-deferral     MECHANICAL. A finished leaf hands a named subject to
                        another leaf, and that leaf is finished too. Whether
                        the subject actually arrived is the hand step; that it
                        has no owner any more is not.
  goal-item-unreceipted JUDGEMENT-BOUND. A finished leaf's goal enumerates an
                        item none of whose distinctive words appears in its own
                        receipt. A goal line legitimately describes more than
                        one leaf delivers, so this is a POPULATION and never a
                        defect count (`docs/CLAIM_VERIFICATION.md` leg 2).

⛔ Mixing them would bury the exact rule in the judgement-bound one's false
positives, which is the failure `SIGNOFF-REPAIR.11.5` names: a gate people
route around is a gate that lies.

Usage:
    python3 -B scripts/census_goal_receipt_gap.py            # the census
    python3 -B scripts/census_goal_receipt_gap.py --check    # populations only, rc
    python3 -B scripts/census_goal_receipt_gap.py --self-test
"""

from __future__ import annotations

import re
import subprocess
import sys
from dataclasses import dataclass, field

# A status meaning the leaf is no longer work. `active` is not one: an active
# leaf can still grow a clause, so a deferral handed to it still has an owner.
FINISHED = {"done", "complete", "closed"}

# Words that carry no subject. Derived from the corpus's own prose register
# rather than from a general stoplist: these are the words this project's goal
# lines use to GLUE items together, so a word here can never identify the item.
GLUE = {
    "the", "a", "an", "and", "or", "of", "to", "in", "on", "at", "by", "for",
    "with", "from", "into", "over", "under", "as", "is", "are", "was", "were",
    "be", "been", "its", "it", "that", "this", "these", "those", "than",
    "then", "never", "not", "no", "own", "same", "each", "every", "all", "any",
    "one", "two", "three", "per", "via", "plus", "minus", "only", "still",
    "new", "old", "first", "last", "next", "more", "less", "up", "down", "out",
    "off", "so", "but", "if", "when", "where", "which", "who", "whose", "what",
    "how", "why", "can", "may", "must", "shall", "will", "would", "should",
    "does", "do", "did", "has", "have", "had", "rather", "instead", "both",
    "either", "neither", "there", "here", "now", "yet", "also", "just",
}

# How this corpus actually spells a deferral, derived by grepping the trees for
# a leaf reference and reading what precedes it — NOT invented. `frontier →` is
# excluded deliberately: it points at what comes next, it does not hand work
# over.
DEFERRAL = re.compile(
    r"(?<!frontier → )\b("
    r"rides?|lands? with|lands? in|lands? at|owned by|deferred to|"
    r"stays? the|is the named|are the named|ride the|carried by"
    r")\b[^.;]{0,80}?`(?P<ref>\.?[0-9][0-9.]*|[A-Z][A-Z0-9-]*(?:\.\d+)+)`",
    re.IGNORECASE,
)

LEAF_ID = r"[A-Z][A-Z0-9-]*(?:\.\d+)+"

# The corpus's idiom for a leaf opened and discharged in one commit.
OPENED_AND_CLOSED = re.compile(r"^- Opened and closed\b", re.M)
CLOSED = "closed"


@dataclass
class Leaf:
    tree: str
    ident: str
    status: str | None
    goal: str
    receipt: str
    line: int
    whole: str = ""
    has_done: bool = False
    duplicates: list[int] = field(default_factory=list)

    @property
    def finished(self) -> bool:
        return (self.status or "") in FINISHED


def tracked_trees(rev: str | None = None) -> list[str]:
    """The tracked top-level trees, from the index — never a hand list.

    ⛔ The pathspec carries `:(glob)` because git's default `*` crosses `/`,
    which would sweep in `docs/tasks/artifacts/` — the trap
    `knowledge-map/scripts/gen_knowledge_map.sh` already documents.
    """
    cmd = ["git", "ls-tree", "-r", "--name-only", rev, "--", "docs/tasks"] if rev else [
        "git", "ls-files", "--", ":(glob)docs/tasks/*.md"]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True).stdout.split()
    return [p for p in out
            if p.endswith(".md") and p.count("/") == 2
            and not p.endswith(("TEMPLATE.md", "BOOTSTRAP.md"))]


def read(path: str, rev: str | None) -> str:
    """A tree's text at `rev`, or from the working copy.

    ⛔ **THIS FLAG IS NOT A CONVENIENCE, it is the only way this instrument can
    be calibrated.** Its two founding instances are `PHASE-8.3.3` and
    `PHASE-8.3.4`, and the commits that FOUND them annotated both leaves with
    the missing words — so at `HEAD` the receipts now contain `reauthorize`,
    `recreate`, `reconcile` and `resources`, and the census sees nothing. An
    instrument measured only against the tree its own repairs have already
    touched is measuring the repair.
    """
    if rev is None:
        return open(path, encoding="utf-8").read()
    return subprocess.run(["git", "show", f"{rev}:{path}"],
                          capture_output=True, text=True, check=True).stdout


def parse_phase(tree: str, text: str) -> list[Leaf]:
    """The `- ID: \\`X\\`` / `Status:` / `Goal:` / `Done:` / `Acceptance:` shape."""
    leaves: list[Leaf] = []
    starts = [m for m in re.finditer(rf"^\s*- ID: `({LEAF_ID})`\s*$", text, re.M)]
    for i, m in enumerate(starts):
        end = starts[i + 1].start() if i + 1 < len(starts) else len(text)
        body = text[m.end(): end]
        status = re.search(r"^\s*Status: `([\w-]+)`", body, re.M)
        goal = field_block(body, "Goal")
        # ⛔ The receipt is the DONE list, plus the sections that record what
        # actually happened. `Acceptance:` alone is NOT a receipt: it states the
        # bar, and a leaf's acceptance legitimately does not repeat its goal's
        # subject — measured, and it was producing 543 hits of which the large
        # majority were leaves whose acceptance simply used other words.
        done = field_block(body, "Done")
        receipt = "\n".join(filter(None, (
            done,
            field_block(body, "Acceptance"),
            field_block(body, "Verification"),
            field_block(body, "Evidence"),
            field_block(body, "Note"),
        )))
        leaves.append(Leaf(
            tree=tree, ident=m.group(1),
            status=status.group(1) if status else None,
            goal=goal, receipt=receipt,
            line=text.count("\n", 0, m.start()) + 1, whole=body,
            has_done=bool(done),
        ))
    return leaves


def field_block(body: str, name: str) -> str:
    """One `Name:` field and its continuation, to the next field at that indent."""
    m = re.search(rf"^(\s*){name}[^:\n]*:(.*)$", body, re.M)
    if not m:
        return ""
    indent = len(m.group(1))
    collected = [m.group(2).strip()]
    for line in body[m.end():].split("\n")[1:]:
        if not line.strip():
            collected.append("")
            continue
        lead = len(line) - len(line.lstrip())
        if lead <= indent and re.match(r"\s*[A-Z][A-Za-z ]*:", line):
            break
        if lead < indent:
            break
        collected.append(line.strip())
    return " ".join(x for x in collected if x)


def parse_headings(tree: str, text: str) -> list[Leaf]:
    """The heading shape: `### X — title`, status in `Status:` or `Opened:`."""
    leaves: list[Leaf] = []
    heads = list(re.finditer(rf"^#{{3,6}} ({LEAF_ID}) — ", text, re.M))
    for i, h in enumerate(heads):
        end = heads[i + 1].start() if i + 1 < len(heads) else len(text)
        body = text[h.end(): end]
        # ⛔ A LEAF'S STATUS IS NOT ALWAYS IN A `Status:` LINE, and this
        # instrument found that out by refusing three leaves it could not
        # classify — `.6.2.3.1`, `.13.1.2` and `.13.4.1`. All three are closed
        # and every reader can see it: they open `- Opened and closed by …`,
        # a real idiom of this corpus for a leaf that was opened and discharged
        # in one commit. The refusal was the instrument's blind spot, not a
        # defect in the trees, and the repair is to learn the idiom rather than
        # to edit three tracked leaves into a shape the script preferred.
        st = (re.search(r"^- Status: `([\w-]+)`", body, re.M)
              or re.search(r"^- Opened: `([\w-]+)`", body, re.M)
              or (OPENED_AND_CLOSED.search(body) and CLOSED))
        goal = re.search(r"^- Goal[^:\n]*:(.*)$", body, re.M)
        leaves.append(Leaf(
            tree=tree, ident=h.group(1),
            status=(st if isinstance(st, str) else st.group(1)) if st else None,
            goal=goal.group(1).strip() if goal else "",
            receipt=body, line=text.count("\n", 0, h.start()) + 1, whole=body,
        ))
    return leaves


def words(text: str) -> set[str]:
    """Distinctive words, normalised: lowercase, plural-stripped, glue removed."""
    found = set()
    for raw in re.findall(r"[A-Za-z][A-Za-z_-]{2,}", text.lower()):
        w = raw[:-1] if raw.endswith("s") and len(raw) > 4 else raw
        if w not in GLUE and len(w) >= 4:
            found.add(w)
    return found


def goal_items(goal: str) -> list[str]:
    """Split a goal line into the items it enumerates.

    The separator is `→` ALONE, and that is a calibrated choice rather than a
    stylistic one. Measured at `fdd3106` — the commit before this project's own
    repairs annotated the two founding leaves — against the population each
    separator set produces and the founding instances it catches:

        → + ; and `, `   population 306   founding items caught 2
        → + ;            population  78   founding items caught 2
        → +              population  51   founding items caught 2
        →                population  12   founding items caught 2

    The narrowest set strictly DOMINATES: every wider one adds between 39 and
    294 rows and catches nothing further.

    ⛔ Two characters were tried and REMOVED, each for a measured reason. An em
    dash introduces an apposition, so splitting on it made every goal's own
    subject look like a second promise. A slash is a PATH separator far more
    often than an enumerator here, and it shattered
    `docs/decisions/parking-lot.md` into three items, two of them the bare word
    `docs`.
    """
    parts = re.split(r"\s*→\s*", goal)
    return [p.strip(" ().") for p in parts if len(p.strip(" ().")) > 3]


def census(paths: list[str], rev: str | None = None) -> dict:
    by_tree: dict[str, list[Leaf]] = {}
    for path in paths:
        text = read(path, rev)
        leaves = parse_phase(path, text)
        if not leaves:
            leaves = parse_headings(path, text)
        by_tree[path] = leaves

    duplicates, stranded, unreceipted, no_receipt = [], [], [], []
    comparable: set = set()
    for path, leaves in by_tree.items():
        seen: dict[str, Leaf] = {}
        for leaf in leaves:
            if leaf.ident in seen:
                duplicates.append((path, leaf.ident, seen[leaf.ident].line, leaf.line))
            else:
                seen[leaf.ident] = leaf
        prefix = seen and next(iter(seen)).split(".")[0]

        for leaf in leaves:
            if not leaf.finished:
                continue
            for m in DEFERRAL.finditer(leaf.whole):
                ref = m.group("ref")
                target = f"{prefix}{ref}" if ref.startswith(".") else ref
                if target == leaf.ident:
                    continue
                other = seen.get(target)
                if other is None or not other.finished:
                    continue
                # ⛔ *The target is finished* is NOT by itself a stranding, and
                # reading it that way would call this project's own routing
                # discipline a defect: a parent that writes `Owned by \`.N\``
                # and sees `.N` close is the discipline WORKING. This predicate
                # publishes a POPULATION to classify, never a defect count.
                #
                # 🔴 A DISCARDED INSTRUMENT, recorded because the discard is the
                # transferable part. An `arrived` column was built to do the
                # classification mechanically: take the clause before the
                # deferral verb as the SUBJECT and look for its words in the
                # target's receipt. Measured over this same population it
                # produced **16 rows with an EMPTY subject** — `Owned by \`.N\``
                # opens its own line, so the sentence split had nothing to the
                # left — and it marked the founding instance
                # `PHASE-8.3.3 → PHASE-8.3.4` as `arrived`, because the word
                # `transport` appears in the target's ROOT-CAUSE box while the
                # transport itself does not exist. An instrument that cannot
                # classify its own founding case may not classify the rest.
                stranded.append((path, leaf.ident, target, squeeze(m.group(0))))
            # ⛔ A receipt you do not have cannot be compared. A leaf with no
            # `Done:` list is counted as `no-receipt` and is NEVER a gap: the
            # predicate is about a promise contradicted by its own receipt, not
            # about a leaf that keeps its record elsewhere.
            if not leaf.goal:
                continue
            if not leaf.has_done:
                no_receipt.append((path, leaf.ident))
                continue
            comparable.add((path, leaf.ident))
            have = words(leaf.receipt)
            for item in goal_items(leaf.goal):
                distinctive = words(item)
                if distinctive and not (distinctive & have):
                    unreceipted.append((path, leaf.ident, squeeze(item), sorted(distinctive)))

    return {
        "trees": len(by_tree),
        "leaves": sum(len(v) for v in by_tree.values()),
        "finished": sum(1 for v in by_tree.values() for l in v if l.finished),
        "unclassified": [
            (p, l.ident) for p, v in by_tree.items() for l in v if l.status is None
        ],
        "duplicate-leaf-id": duplicates,
        "stranded-deferral": stranded,
        "goal-item-unreceipted": unreceipted,
        "no-receipt": no_receipt,
        "comparable": len(comparable),
    }


def squeeze(text: str) -> str:
    return " ".join(text.split())


def report(result: dict, verbose: bool) -> int:
    print(f"tracked trees: {result['trees']}  leaves: {result['leaves']}  "
          f"finished: {result['finished']}")
    print(f"  finished leaves carrying a goal AND a Done list: "
          f"{result['comparable']}  (goal but no Done list: {len(result['no-receipt'])} "
          f"— not comparable, never a gap)")
    for name in ("duplicate-leaf-id", "stranded-deferral", "goal-item-unreceipted"):
        rows = result[name]
        print(f"  {name:24} {len(rows)}")
        if verbose:
            for row in rows:
                print(f"      {row}")
    if result["unclassified"]:
        print(f"\n⛔ {len(result['unclassified'])} leaf/leaves carry NO status and cannot be classified:")
        for path, ident in result["unclassified"][:20]:
            print(f"      {path}: {ident}")
        return 1
    return 0


def self_test() -> int:
    """Two-sided: each predicate fires on a positive fixture and stays silent on
    a negative one that differs in exactly the fact the predicate is about."""
    import tempfile, pathlib
    # ⛔ REPOSITORY-LOCAL, never `TMPDIR` (§13, and the STORAGE-LOCALITY gate
    # refused the first draft of this file for exactly this): the default lands
    # on another volume, and project data — fixtures included — stays on the
    # repository's own.
    scratch = pathlib.Path(__file__).resolve().parents[1] / ".project-data" / "tmp"
    scratch.mkdir(parents=True, exist_ok=True)
    checks: list[tuple[str, bool, bool]] = []

    def run(body: str, key: str) -> int:
        with tempfile.TemporaryDirectory(dir=scratch) as d:
            p = pathlib.Path(d) / "PHASE-T.md"
            p.write_text(body, encoding="utf-8")
            return len(census([str(p)])[key])

    dup_pos = ("  - ID: `PHASE-T.1`\n    Status: `done`\n    Goal: a\n    Done:\n    - a\n"
               "  - ID: `PHASE-T.1`\n    Status: `done`\n    Goal: a\n    Done:\n    - a\n")
    dup_neg = dup_pos.replace("`PHASE-T.1`\n    Status: `done`\n    Goal: a\n    Done:\n    - a\n  - ID: `PHASE-T.1`",
                              "`PHASE-T.1`\n    Status: `done`\n    Goal: a\n    Done:\n    - a\n  - ID: `PHASE-T.2`")
    checks.append(("duplicate-leaf-id fires on a repeated id", run(dup_pos, "duplicate-leaf-id") == 1, True))
    checks.append(("duplicate-leaf-id silent on distinct ids", run(dup_neg, "duplicate-leaf-id") == 0, True))

    base = ("  - ID: `PHASE-T.1`\n    Status: `done`\n    Goal: the widget\n    Done:\n"
            "    - the widget ships; the transport rides `.2`.\n"
            "  - ID: `PHASE-T.2`\n    Status: `{}`\n    Goal: the other\n    Done:\n    - the other ships.\n")
    checks.append(("stranded-deferral fires when the target is done",
                   run(base.format("done"), "stranded-deferral") == 1, True))
    checks.append(("stranded-deferral silent when the target is still open",
                   run(base.format("proposed"), "stranded-deferral") == 0, True))
    checks.append(("stranded-deferral silent when the deferring leaf is open",
                   run(base.format("done").replace("`PHASE-T.1`\n    Status: `done`",
                                                   "`PHASE-T.1`\n    Status: `proposed`"),
                       "stranded-deferral") == 0, True))

    gr_pos = ("  - ID: `PHASE-T.1`\n    Status: `done`\n    Goal: the tools → the resources\n"
              "    Done:\n    - the tools ship.\n")
    gr_neg = gr_pos.replace("    - the tools ship.", "    - the tools ship with their resources.")
    checks.append(("goal-item-unreceipted fires on a promise with no receipt",
                   run(gr_pos, "goal-item-unreceipted") == 1, True))
    checks.append(("goal-item-unreceipted silent when the receipt names it",
                   run(gr_neg, "goal-item-unreceipted") == 0, True))
    checks.append(("goal-item-unreceipted silent on an OPEN leaf",
                   run(gr_pos.replace("Status: `done`", "Status: `proposed`"),
                       "goal-item-unreceipted") == 0, True))

    # An em dash introduces an apposition, not a second promise.
    apposition = ("  - ID: `PHASE-T.1`\n    Status: `done`\n"
                  "    Goal: the tools — the inspection verbs as tools\n    Done:\n    - the tools ship.\n")
    plus = ("  - ID: `PHASE-T.1`\n    Status: `done`\n    Goal: the tools + the resources\n"
            "    Done:\n    - the tools ship.\n")
    checks.append(("an em-dash apposition is not counted as an unreceipted item",
                   run(apposition, "goal-item-unreceipted") == 0, True))
    # The calibrated separator is `→` alone. A `+` list is NOT split, and the
    # arm exists so that widening the separator silently is a RED rather than a
    # population change nobody looks at.
    checks.append(("a `+` list is not split — the separator is calibrated, not stylistic",
                   run(plus, "goal-item-unreceipted") == 0, True))

    # A leaf with no status is refused rather than assumed open.
    with tempfile.TemporaryDirectory(dir=scratch) as d:
        p = pathlib.Path(d) / "PHASE-T.md"
        p.write_text("  - ID: `PHASE-T.1`\n    Goal: a\n    Done:\n    - a\n", encoding="utf-8")
        checks.append(("a statusless leaf is reported rather than assumed",
                       len(census([str(p)])["unclassified"]) == 1, True))
    # The `Opened and closed` idiom is a STATUS, and the arm exists because the
    # instrument refused three real leaves before it knew that.
    with tempfile.TemporaryDirectory(dir=scratch) as d:
        p = pathlib.Path(d) / "T.md"
        p.write_text("#### T.1 — a\n\n- Opened and closed by `T.2`, which measured it.\n",
                     encoding="utf-8")
        r = census([str(p)])
        checks.append(("`Opened and closed` is read as a finished status",
                       r["unclassified"] == [] and r["finished"] == 1, True))

    failed = 0
    for name, got, want in checks:
        ok = got == want
        failed += not ok
        print(f"  {'ok  ' if ok else 'FAIL'}  {name}")
    print(f"{len(checks) - failed}/{len(checks)} controls pass")
    return 1 if failed else 0


def calibrate(window: int) -> int:
    """Replay `duplicate-leaf-id` over the last `window` commits.

    ⛔ Only the EXACT predicate is calibrated, because only an exact predicate
    is a gate candidate. `SIGNOFF-REPAIR.11.6` forbids a threshold before its
    population, and the other two predicates publish populations to classify.
    """
    revs = subprocess.run(
        ["git", "log", f"-{window}", "--format=%H", "--", "docs/tasks"],
        capture_output=True, text=True, check=True).stdout.split()
    blocked = []
    for rev in revs:
        try:
            rows = census(tracked_trees(rev), rev)["duplicate-leaf-id"]
        except subprocess.CalledProcessError:
            continue
        if rows:
            blocked.append((rev[:7], rows))
    print(f"commits touching docs/tasks replayed: {len(revs)}")
    print(f"commits a duplicate-leaf-id gate would BLOCK: {len(blocked)}"
          f" ({100 * len(blocked) / max(1, len(revs)):.1f}%)")
    for rev, rows in blocked:
        for row in rows:
            print(f"    {rev}  {row[1]}  lines {row[2]} and {row[3]}")
    return 0


def gate() -> int:
    """LEAF-ID-UNIQUE — the EXACT predicate, and only that one.

    ⛔ It fails on a duplicate leaf id and on nothing else. The other two
    predicates publish populations a human classifies, and a gate that refused
    on those would be refused right back — `SIGNOFF-REPAIR.11.5`'s rule that a
    gate people route around is a gate that lies.
    ⚠️ A leaf whose status this instrument cannot read is WARNED, never failed:
    a model that has drifted from its corpus must say so without blocking, which
    is how `.11.20`'s census came to be refusing on every run with nobody
    looking.
    """
    result = census(tracked_trees())
    for path, ident in result["unclassified"]:
        print(f"  ⚠️ LEAF-ID-UNIQUE: {path}: {ident} carries no status this "
              f"instrument can read — the model may have drifted from the corpus")
    rows = result["duplicate-leaf-id"]
    if not rows:
        return 0
    print("LEAF-ID-UNIQUE: two leaves share one address.")
    for path, ident, first, second in rows:
        print(f"    {path}: `{ident}` at line {first} and again at line {second}")
    print("""
  A leaf id IS its address: the Current Frontier, the task-tree index and every
  cross-reference resolve work by it. Two leaves sharing one means a reader — or
  a script — reaches whichever comes first.
  ⛔ Nothing else detects this. FRONTIER-STATUS refused the one near-miss this
  project has had, but by accident: its resolver takes the FIRST heading with an
  id, so the row's status was compared against the OLDER leaf's, and the message
  said a row disagreed with its leaf. Had the colliding leaf been `pending` the
  gate would have passed. (SIGNOFF-REPAIR.11.24.)""")
    return 1


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    if "--gate" in sys.argv:
        return gate()
    if "--calibrate" in sys.argv:
        return calibrate(int(sys.argv[sys.argv.index("--calibrate") + 1]))
    rev = None
    if "--at" in sys.argv:
        rev = sys.argv[sys.argv.index("--at") + 1]
        print(f"(censused at {rev})")
    result = census(tracked_trees(rev), rev)
    return report(result, verbose="--check" not in sys.argv)


if __name__ == "__main__":
    sys.exit(main())
